//! Plugin seam: one `PluginAdapter` contract for every plugin. A plugin
//! registers a slug, the post `kind`s it owns (each carrying the editor
//! hooks a kind editor needs), optional admin screens, import hooks, a
//! static render hook, and a backend label. Core `post`/`page` kinds are
//! built into the registry — no adapter required — so a site with zero
//! plugins still resolves `kind_def("post")`.
//!
//! Invariant: `REGISTRY` is process-global and compile-time — the plugin
//! set cannot vary per site at runtime. Per-site opt-in is a
//! `Backend::PhpBridge` concern (phase 06), not a registry concern.
//!
//! Contract the core guarantees: `SiteCtx`, `PostForm.extra` (serde-flatten
//! capture of plugin form fields), `post_meta` helpers in `db::posts`,
//! `php_serialize`, the `/admin/wp/{*path}` asset route, WP form chrome
//! (`#post` posts to `/admin/save`, `#publish` is the submit button).
//!
//! The public serve path NEVER loads adapters — it only reads files.

use crate::db::posts::PostForm;
use crate::admin::{App, Sess, SiteCtx};
use crate::db as crud;
use anyhow::Result;
use axum::http::HeaderMap;
use axum::response::Response;
use rusqlite::Connection;
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::LazyLock;

pub(crate) mod spec;
// theme_tevily lives in themes/specs/tevily.toml — the declarative-demo spec.
pub(crate) mod theme_traveltour; // gate-allow: adapter module registration is the contract
pub mod tourmaster; // gate-allow: audit_remediation tests reach save/render helpers
pub(crate) mod goodlayers;
/// `/admin/plugins` index + shared settings-screen helpers.
pub(crate) mod index;
pub(crate) mod chaty;
pub(crate) mod misc;
pub(crate) mod presto_player;
pub(crate) mod readmore;
/// Everything a kind editor needs — the four hooks the old `EditorPlugin`
/// carried, verbatim. `KindDef.editor` is `None` for kinds with no plugin
/// editor (core post/page use the classic editor).
pub(crate) struct EditorHooks {
    /// Hydrate `PostForm.extra` on load (e.g. unserialize meta blobs into
    /// the JSON the metabox expects).
    pub(crate) load: fn(&Connection, i64, &mut PostForm) -> Result<()>,
    /// Panel HTML emitted inside `post-body-content`, after the editor.
    pub(crate) metabox: fn(&SiteCtx, &Sess, &PostForm) -> String,
    /// `<link>`/`<script>` tags appended to the edit page.
    pub(crate) assets: fn() -> &'static str,
    /// Runs inside `crud::save_post`'s connection after the posts row
    /// upsert; plugin reads its fields from `f.extra`.
    pub(crate) save: fn(&Connection, i64, &PostForm) -> Result<()>,
}

/// One registered post kind: what `/admin/<section>` lists and edits.
/// Adding a CPT = one `KindDef` in an adapter (or a built-in core row).
pub(crate) struct KindDef {
    /// `posts.kind` value, e.g. "tour".
    pub(crate) kind: &'static str,
    /// Admin section slug, e.g. "tours" → `/admin/tours`.
    pub(crate) section: &'static str,
    /// Singular noun for the editor heading ("Edit Page").
    pub(crate) noun: &'static str,
    /// List-page title.
    pub(crate) list_title: &'static str,
    /// "Add …" button label.
    pub(crate) add_label: &'static str,
    /// `<thead>` for the list table.
    pub(crate) head: &'static str,
    /// Extra `<td>` columns after the Title cell.
    pub(crate) row_cols: fn(&crud::AdminListRow, &str, &crate::lang::LangConfig) -> String,
    /// Settings key carrying this kind's public URL base (e.g.
    /// `tour_base`); `None` for kinds served at the root. Consumed by
    /// the url-base threading in phase 03.
    #[allow(dead_code)]
    pub(crate) url_base_key: Option<&'static str>,
    /// Fixed public URL base (e.g. "booking" for BABE `to_book`) — wins
    /// over `url_base_key` so kinds whose permalink base is part of the
    /// plugin contract don't share the legacy `tour_base` value.
    pub(crate) url_base: Option<&'static str>,
    /// Taxonomies owned by this kind — `tax_menu` routes their screens
    /// under this kind's section instead of a catch-all.
    pub(crate) taxonomies: &'static [&'static str],
    /// Dashicons class for the sidebar menu entry ("" → no generic entry).
    pub(crate) menu_icon: &'static str,
    /// Editor hooks; `None` → classic editor, no metabox.
    pub(crate) editor: Option<&'static EditorHooks>,
}


/// `kind → url-base prefix` resolved once per entry point (generate,
/// render_slug, emit_site_files, admin serve). Replaces the threaded
/// `tour_base: &str` — one string per `KindDef.url_base_key`, resolved
/// here so downstream fns only ever see the kind they are rendering.
#[allow(dead_code)] // wired by phase-03 threading below
pub(crate) type KindBases = std::collections::HashMap<&'static str, String>;

/// Build the map from a getter over `KindDef.url_base_key`. The legacy
/// `TOUR_BASE`/`--tour-base` value is passed in by the caller today; when
/// a second base-bearing kind lands the getter reads per-key settings.
#[allow(dead_code)]
pub(crate) fn resolve_bases(legacy_tour_base: &str) -> KindBases {
    let mut m = KindBases::new();
    for def in kind_defs() {
        if let Some(b) = def.url_base {
            m.insert(def.kind, b.to_string());
        } else if let Some(_key) = def.url_base_key {
            m.insert(def.kind, legacy_tour_base.to_string());
        }
    }
    m
}

/// Resolved base for `kind`; `None` → root-served kind.
#[allow(dead_code)]
pub(crate) fn base_for<'a>(bases: &'a KindBases, kind: &str) -> Option<&'a str> {
    bases.get(kind).map(|s| s.as_str())
}

/// Single-kind convenience over `resolve_bases`/`base_for` for call sites
/// that still carry the legacy `tour_base: &str` — returns `None` for
/// root-served kinds (no `url_base_key`).
pub(crate) fn kind_url_base(legacy_tour_base: &str, kind: &str) -> Option<String> {
    base_for(&resolve_bases(legacy_tour_base), kind).map(str::to_string)
}

/// Render-side context handed to adapter `render()`: the post row plus
/// its already-rendered page-builder body. Moved here (was
/// `render_tour::PostCtx`) because `render_archive` and the archive-card
/// helpers share the shape; `tour_base` resolves through `KindBases`.
#[allow(dead_code)] // populated by phase-03 render dispatch
pub(crate) struct RenderCtx {
    pub slug: String,
    pub lang: String,
    pub title: String,
    pub id: i64,
    pub site_url: String,
    pub body_html: String,
    pub body_html_rendered: String,
    pub kind: String,
    /// Public URL segment for the owning kind ("" for root kinds).
    pub kind_base: String,
}
/// Request lifetime for screen handlers.
pub(crate) type ScreenFuture<'a> = Pin<Box<dyn Future<Output = Response> + Send + 'a>>;

/// Signature for `/admin/plugins/<slug>/<path>` handlers: dispatch does
/// `need_auth` once — a screen receives the live session + site ctx and
/// picks its own body extractors (`Form`, JSON, multipart…).
pub(crate) type ScreenHandler =
    for<'a> fn(&'a App, &'a Sess, &'a SiteCtx, HeaderMap, &'a str, axum::extract::Request) -> ScreenFuture<'a>;

/// An adapter-owned admin endpoint under `/admin/plugins/<slug>/<path>`.
/// One `ScreenDef` per route; GET+POST both dispatch to `handle`.
pub(crate) struct ScreenDef {
    /// Route path under the plugin's screen root, e.g. "save-option".
    pub(crate) path: &'static str,
    /// Human title for the screen + submenu entry.
    #[allow(dead_code)]
    pub(crate) title: &'static str,
    /// Minimum role allowed to hit this screen — `editor` for in-editor
    /// AJAX (tourmaster metabox buttons), `admin` for settings surfaces.
    pub(crate) min_role: &'static str,
    /// Handle the request; must call `need_auth` itself.
    pub(crate) handle: ScreenHandler,
}



/// Import pipeline hooks (wired into `import.rs` in phase 03).
#[allow(dead_code)]
pub(crate) struct ImportHooks {
    /// Claim a wp `post_type` before the core `kind_of` match runs;
    /// `Some(kind)` takes over the row, `None` falls through to core.
    pub(crate) claim_type: fn(&str) -> Option<&'static str>,
    /// `wp_options` name prefixes this plugin owns (e.g. `"tourmaster_"`)
    /// — core whitelists them into `settings` under the plugin namespace.
    pub(crate) option_prefixes: &'static [&'static str],
    /// Raw access to every wp_posts row — claimed, ignored and skipped
    /// alike — before `kind_for_post_type` runs. Args: `(tx, id,
    /// post_type, post_content)`. For adapters whose data lives in
    /// storage CPTs the core never imports (presto video records).
    pub(crate) post_seen: fn(&Connection, i64, &str, &str) -> Result<()>,
    /// After the posts row is imported: write plugin tables/meta.
    /// Runs inside the import transaction.
    pub(crate) post_imported: fn(&Connection, i64) -> Result<()>,
    /// After the whole import: promotion/denormalization (e.g.
    /// post_meta → `tours` sync). Runs once per import.
    pub(crate) after_import: fn(&Connection) -> Result<()>,
}

/// Where the plugin's code actually runs. A site using only `Rust`/`Data`
/// plugins deploys exactly today's single binary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub(crate) enum Backend {
    /// Native port — tourmaster, submissions.
    Rust,
    /// Read/write plugin DATA only (options, meta, trees); output stays
    /// the already-rendered HTML.
    Data,
    /// Serve captured plugin output; read-only.
    StaticReplay,
    /// Opt-in — forwards to a real WP running the plugin (php-wasm /
    /// Playground / php-cgi). Never present unless explicitly enabled.
    PhpBridge,
}
/// Display metadata for the `/admin/plugins` index — name + one-line
/// blurb, WP plugin-list parity.
pub(crate) struct PluginDescriptor {
    pub(crate) name: &'static str,
    pub(crate) blurb: &'static str,
}

/// Fallback descriptor from the slug (adapters that don't declare one).
impl PluginDescriptor {
    pub(crate) fn of_slug(slug: &'static str) -> Self {
        PluginDescriptor { name: slug, blurb: "" }
    }
}

/// One contract for all plugins. Registration is compile-time: add a
/// `&'static ADAPTER` to `ADAPTERS`; commenting the line out is the off
/// switch — core never names plugins.
#[allow(dead_code)]
pub(crate) trait PluginAdapter: Sync {
    /// Plugin slug, e.g. "tourmaster". Namespaces settings keys
    /// (`plugin:<slug>:<key>`) and the `/admin/plugins/<slug>` screens.
    fn slug(&self) -> &'static str;
    /// Post kinds this adapter owns; may be empty (screens-only plugins).
    fn kinds(&self) -> &'static [KindDef];

    /// Display name + blurb for the plugins index. Slug fallback.
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor::of_slug(self.slug())
    }
    /// Option-key prefixes this adapter's settings screen edits —
    /// informational on the index (warns on unclaimed plugin options).
    fn settings_prefixes(&self) -> &'static [&'static str] {
        &[]
    }
    /// Admin screens under `/admin/plugins/<slug>/…`.
    fn screens(&self) -> &'static [ScreenDef] {
        &[]
    }
    /// Sidebar submenu for this adapter's kind menu: `(href, label)`
    /// pairs. `None` → registry builds the default (list + add + owned
    /// taxonomies).
    fn submenu(&self, _db: &Path, _def: &KindDef) -> Option<Vec<(String, String)>> {
        None
    }
    /// Boot-time schema + data promotion for the plugin's own tables.
    /// Runs once at `serve()` startup only when the adapter is registered
    /// (red-team R7); must be idempotent (`CREATE TABLE IF NOT EXISTS`,
    /// `INSERT OR REPLACE`).
    fn migrate(&self, _db: &Path) -> Result<()> {
        Ok(())
    }
    /// Legacy bare `settings` keys this plugin owns (written by pre-seam
    /// imports). `migrate_all` copies each to `plugin:<slug>:<key>` via an
    /// atomic `INSERT OR IGNORE` — the legacy row is kept so a rollback
    /// loses nothing; dual-read prefers the namespaced copy.
    fn legacy_option_keys(&self) -> &'static [&'static str] {
        &[]
    }
    /// Copy plugin-owned rows when `posts` row `old_id` clones to
    /// `new_id`. Runs inside the clone transaction; a no-op default is
    /// fine — post_meta is already copied by the caller.
    fn clone_post(&self, _tx: &Connection, _new_id: i64, _old_id: i64) -> Result<()> {
        Ok(())
    }
    /// Import hooks; `None` → core import only.
    fn import_hooks(&self) -> Option<&'static ImportHooks> {
        None
    }
    /// wp `post_type`s this adapter deliberately skips at import
    /// (storage CPTs) — detect lists them as `ignored` rather than
    /// suggesting spec work. Empty default.
    fn ignore_types(&self) -> &'static [&'static str] {
        &[]
    }
    /// Shortcode tags this adapter's render path expands natively —
    /// detect reports them as "rpress renders" instead of "needs
    /// render support". Empty default.
    fn rendered_shortcodes(&self) -> &'static [&'static str] {
        &[]
    }
    /// Full single-page render for a post of this adapter's kind —
    /// replaces the `kind == "tour"` dispatch arm in `render_slug`.
    /// `ctx` carries the rendered body + resolved `kind_base`.
    /// `None` → caller falls through to the bare-body path.
    fn render(&self, _conn: &Connection, _templates: &Path, _ctx: &RenderCtx) -> Result<Option<String>> {
        Ok(None)
    }
    /// Canonical permalink slug for a post of this kind. Some kinds keep
    /// a raw stored slug that is degenerate ('', 'x') and derive the
    /// public slug from import-era redirects or the title — WP
    /// `sanitize_title` semantics live with the plugin. `None` → caller
    /// uses the stored `posts.slug` unchanged.
    fn display_slug(&self, _conn: &Connection, _id: i64, _slug: &str, _title: &str) -> Option<String> {
        None
    }
    /// `(key, value)` pairs merged into the site's shared template data
    /// (`site_data`): adapter-owned feeds like `"tours"` or
    /// `"testimonials"`. Runs once per `(conn, lang)` per render.
    /// `site_url` is the caller's render origin (may differ from
    /// `settings.site_url` under local serve); `kind_base` is the
    /// resolved URL base for this adapter's kind (`None`/`""` → root).
    fn site_data_feeds(&self, _conn: &Connection, _lang: &str, _site_url: &str, _kind_base: Option<&str>) -> Vec<(String, serde_json::Value)> {
        Vec::new()
    }
    /// Widget type names + admin labels this adapter registers
    /// (`("tour_list","Tour List")`); core merges them into WIDGET_TYPES.
    fn widget_types(&self) -> &'static [(&'static str, &'static str)] {
        &[]
    }
    /// Admin field specs for a registered widget type; `None` → core
    /// `widget_fields` lookup only.
    fn widget_fields(&self, _typ: &str) -> Option<&'static [crate::widgets::FieldSpec]> {
        None
    }
    /// Theme adapters declare their `ThemeDef` here; every other adapter
    /// returns `None` (default). Public render reads the seeded values
    /// through `site.theme.*` settings — this hook only *writes* at
    /// migrate time and steers `load_env`/`template_name`.
    fn theme(&self) -> Option<&'static ThemeDef> {
        None
    }
    /// Sidebar regions the adapter registers (`sidebar-tour`). Core
    /// merges them with the const `SIDEBARS` list.
    fn sidebars(&self) -> &'static [crate::widgets::Sidebar] {
        &[]
    }
    /// Seed widget rows shipped with the plugin (JSON embedded at build
    /// time); core's seed pass merges them when creating a fresh site.
    fn seeds(&self) -> &'static [crate::widgets::Seed] {
        &[]
    }
    /// Public path segment for an owned taxonomy archive
    /// (`tour_category → "tour-category"`). `None` → core default
    /// (taxonomy name verbatim).
    fn taxonomy_base(&self, _tax: &str) -> Option<&'static str> {
        None
    }
    /// URL segment the canonical archive page for `tax` lives under,
    /// expressed the same way as `taxonomy_base` for callers that need
    /// the owning plugin to name its tax bases.
    fn archive_term_url(&self, _lang_cfg: &crate::lang::LangConfig, _tax: &str, _lang: &str, _slug: &str) -> Option<String> {
        None
    }
    /// Whitelisted wp.db rows the bridge pull copies into rpress.db
    /// after each proxied request (`Backend::PhpBridge` only). Empty —
    /// the default — means *no* writes ever land.
    fn write_specs(&self) -> &'static [crate::admin::bridge::pull::WriteSpec] {
        &[]
    }
    /// Editor boot HTML for posts carrying plugin-owned layout data
    /// (gdlr `body_json`). Called by the post-edit screen when the body
    /// is plugin-formatted; `None` → classic editor only. (phase-07)
    fn editor_boot(&self, _conn: &Connection, _sc: &SiteCtx, _post_id: i64, _body_json: &str, _csrf: &str) -> Option<String> {
        None
    }
    /// `<link>`/`<script>` tags the edit page appends when `editor_boot`
    /// emitted markup. `sc` lets the adapter fold opt-in state into the
    /// emitted URLs (bridge ajax endpoints).
    fn editor_assets(&self, _sc: &SiteCtx) -> Option<String> {
        None
    }
    /// Map the admin path to the path inside the bridge WP (PhpBridge
    /// only): `ajax` → `/wp-admin/admin-ajax.php`. Verbatim default.
    fn bridge_path(&self, rest: &str) -> String {
        format!("/{rest}")
    }
    /// `dir/main.php` inside wp-content/plugins the bridge activates
    /// (PhpBridge only; "" → nothing activated).
    fn wp_plugin_file(&self) -> &'static str {
        ""
    }
    /// Which backend the plugin runs on.
    fn backend(&self) -> Backend;
}

// --- Core built-in kinds -------------------------------------------------

const HEAD_POST: &str = r#"<tr><td class="manage-column column-cb check-column"></td><th scope="col" class="manage-column column-title column-primary">Title</th><th scope="col">Author</th><th scope="col">Categories</th><th scope="col">Languages</th><th scope="col">Comments</th><th scope="col" class="rm-seo-col">SEO Details</th></tr>"#;
const HEAD_PAGE: &str = r#"<tr><td class="manage-column column-cb check-column"></td><th scope="col" class="manage-column column-title column-primary">Title</th><th scope="col">Author</th><th scope="col">Languages</th><th scope="col">Comments</th><th scope="col" class="rm-seo-col">SEO Details</th></tr>"#;

fn post_row_cols(r: &crud::AdminListRow, kind: &str, langs: &crate::lang::LangConfig) -> String {
    format!(
        "<td>—</td><td class=\"categories\">{}</td><td class=\"pll\">{}</td><td class=\"comments\">{}</td>",
        crate::admin::posts::dash(&r.categories),
        crate::admin::posts::pll_cell(r, kind, langs),
        r.comments,
    )
}

fn page_row_cols(r: &crud::AdminListRow, kind: &str, langs: &crate::lang::LangConfig) -> String {
    format!(
        "<td>—</td><td class=\"pll\">{}</td><td class=\"comments\">{}</td>",
        crate::admin::posts::pll_cell(r, kind, langs),
        r.comments,
    )
}

/// Core kinds registered with no adapter — the registry is the single
/// `posts.kind → KindDef` map for built-ins and plugins alike.
static CORE_KINDS: &[KindDef] = &[
    KindDef {
        kind: "post", section: "posts", noun: "Post", list_title: "Posts",
        add_label: "Add Post", head: HEAD_POST, row_cols: post_row_cols,
        url_base_key: None, url_base: None,
        taxonomies: &["category", "post_tag"],
        menu_icon: "dashicons-admin-post",
        editor: None,
    },
    KindDef {
        kind: "page", section: "pages", noun: "Page", list_title: "Pages",
        add_label: "Add Page", head: HEAD_PAGE, row_cols: page_row_cols,
        url_base_key: None, url_base: None, taxonomies: &[], menu_icon: "dashicons-admin-page",
        editor: None,
    },
];

/// Registered adapters — the compile-time plugin list.
static ADAPTERS: &[&'static dyn PluginAdapter] = &[
    &tourmaster::ADAPTER,
    &theme_traveltour::ADAPTER, // gate-allow: registry entry
    // tevily: themes/specs/tevily.toml — TOML spec, no Rust adapter.
    &goodlayers::ADAPTER,
    &presto_player::ADAPTER,
    &readmore::ADAPTER,
    &chaty::ADAPTER,
    &misc::WPDISCUZ,
    &misc::CF7,
    &misc::METASLIDER,
    &misc::SITE_REVIEWS,
    &misc::HEAD_FOOTER,
];

/// Indexed view over `ADAPTERS` + `CORE_KINDS`, built once.
struct Registry {
    by_slug: HashMap<&'static str, &'static dyn PluginAdapter>,
    by_kind: HashMap<&'static str, &'static KindDef>,
    by_section: HashMap<&'static str, &'static KindDef>,
    /// kind → owning adapter (core kinds have none).
    kind_adapter: HashMap<&'static str, &'static dyn PluginAdapter>,
}

static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
    let mut r = Registry {
        by_slug: HashMap::new(),
        by_kind: HashMap::new(),
        by_section: HashMap::new(),
        kind_adapter: HashMap::new(),
    };
    for def in CORE_KINDS {
        assert!(r.by_kind.insert(def.kind, def).is_none(), "duplicate kind {}", def.kind);
        assert!(r.by_section.insert(def.section, def).is_none(), "duplicate section {}", def.section);
    }
    // adapters() merges Rust ADAPTERS ∪ spec adapters — spec kinds go
    // through the identical duplicate-kind/section asserts, so a spec
    // colliding with a Rust adapter (or another spec) panics at startup.
    for adapter in adapters() {
        assert!(r.by_slug.insert(adapter.slug(), *adapter).is_none(), "duplicate slug {}", adapter.slug());
        for def in adapter.kinds() {
            assert_ne!(def.section, "plugins", "kind {} claims reserved section 'plugins'", def.kind);
            assert!(r.by_kind.insert(def.kind, def).is_none(), "kind {} claimed twice", def.kind);
            assert!(r.by_section.insert(def.section, def).is_none(), "section {} claimed twice", def.section);
            r.kind_adapter.insert(def.kind, *adapter);
        }
    }
    r
});

/// `posts.kind → KindDef`. `None` for unregistered kinds — callers must
/// 404, never silently post-ify (audit red-team R1).
pub(crate) fn kind_def(kind: &str) -> Option<&'static KindDef> {
    REGISTRY.by_kind.get(kind).copied()
}

/// Admin section slug → `KindDef` (`posts`, `tours`, …).
pub(crate) fn kind_for_section(section: &str) -> Option<&'static KindDef> {
    REGISTRY.by_section.get(section).copied()
}

/// Plugin slug → adapter (`/admin/plugins/<slug>` dispatch).
pub(crate) fn adapter_for_slug(slug: &str) -> Option<&'static dyn PluginAdapter> {
    REGISTRY.by_slug.get(slug).copied()
}

/// `posts.kind` → owning adapter (core kinds → `None`).
pub(crate) fn adapter_for_kind(kind: &str) -> Option<&'static dyn PluginAdapter> {
    REGISTRY.kind_adapter.get(kind).copied()
}

/// Every registered adapter — Rust `ADAPTERS` plus TOML `SpecAdapter`s
/// (`plugins/specs/*.toml`, embedded by build.rs) — drives feed merges
/// (`site_data_feeds`), claim dispatch, and any "all plugins" pass.
/// `'static` slice built once; order = Rust adapters then specs.
pub(crate) fn adapters() -> &'static [&'static dyn PluginAdapter] {
    static ALL: LazyLock<Vec<&'static dyn PluginAdapter>> = LazyLock::new(|| {
        ADAPTERS.iter().copied().chain(spec::spec_adapters().iter().copied()).collect()
    });
    &ALL
}

/// What a theme adapter contributes to template resolution + chrome.
/// Only theme adapters return `Some(theme())`; `PluginAdapter::theme`
/// defaults to `None`.
#[derive(Clone, Copy)]
#[allow(dead_code)] // TOML themes can't carry the fn-pointer fields
                    // (page_title_wrap/wrap_page_sidebar are data-theme
                    // generic) — keep the allow for future spec fields.
pub struct ThemeDef {
    /// `settings.stylesheet` may be a child theme name
    /// (`traveltour-child`, `tevily_child`) — the adapter decides.
    pub matches_stylesheet: fn(&str) -> bool,
    /// Canonical WP `stylesheet` option value this theme claims —
    /// import writes it into `settings.stylesheet` when the dump omits
    /// the row but a `*_general` option identified the theme, so runtime
    /// `theme_for_site` resolves the same theme import did. Empty = the
    /// theme never claims a stylesheet (DEFAULT_THEME).
    pub stylesheet: &'static str,
    /// Templates dir under `templates/theme/` probed before the shared
    /// `templates/` root by `load_env`/`template_name`.
    pub template_dir: &'static str,
    /// `site.theme.body_suffix` value seeded into settings at migrate.
    pub body_suffix: &'static str,
    /// `site.theme.page_title_wrap` — the archive/page title banner
    /// scaffold; `default_page_title` for data themes, Rust themes own
    /// their byte-parity markup. Args: (bg_style_attr, caption_opt, title).
    pub page_title_wrap: fn(&ThemeDef, &str, Option<&str>, &str) -> String,
    /// Menu locations this theme prefers, in pick order (phase-01).
    pub preferred_locations: &'static [&'static str],
    /// Settings key carrying the theme's taxonomy-map JSON
    /// (`{prefix}taxonomy_map`) — drives `theme_cat_path`/`theme_tag_base`/
    /// `theme_cat_lang`. Empty = generic flat `{tax}/{slug}` archives.
    pub taxonomy_map_key: &'static str,
    /// Compile-time fallback for `taxonomy_map_key` when the settings
    /// row is absent (unit tests, pre-migrate DBs) — the theme ships
    /// `include_str!` of its seed JSON. Empty = no map.
    pub taxonomy_map_json: &'static str,
    /// post_tag archive base when the map doesn't supply one (`etiquette`,
    /// `tag`). Empty = fall through to map / "tag".
    pub tag_base: &'static str,
    /// Widget chrome class prefix (`traveltour-widget` → `widget`) —
    /// appended to the frozen widget wrapper (`widget widget_{base}
    /// {widget_class}`) both at import (`import::widget_settings`) and in
    /// live renderers.
    pub widget_class: &'static str,
    /// Theme option row carrying the theme's general-settings blob —
    /// WP goodlayers-family themes persist it as `{prefix}general`
    /// (`traveltour_general`). Import reads sidebar-position picks and
    /// the chrome strings from it. Empty = the theme has no such option.
    pub general_option: &'static str,
    /// Settings key the import writes the flattened `{general_option}`
    /// payload under (`plugin:theme-traveltour:general`). Paired with
    /// `general_option` — both empty or both set.
    pub general_settings_key: &'static str,
    /// `before_widget` title markup frozen into widget settings_json at
    /// import — the theme's own emit (empty title → empty string);
    /// `default_widget_title` for data/default themes. Arg: raw title
    /// (the impl owns its escape flavor).
    pub widget_title_block: fn(&ThemeDef, &str) -> String,
    /// Static HTML a theme owns for one shortcode tag (`[site_reviews]`
    /// ships a frozen capture). `None` → the expander leaves the tag
    /// verbatim, same as WP on a site without the plugin hook.
    pub shortcode_static: fn(&ThemeDef, &str) -> Option<&'static str>,
    /// Sidebar wrapper classes around page bodies with sidebars.
    pub sidebar_wrap_class: &'static str,
    /// Live-edit builder body class suffix (goodlayers Live Mode).
    pub body_class: &'static str,
    /// Theme option prefix in settings (`traveltour_`).
    pub settings_key_prefix: &'static str,
    /// (taxonomy → lang) pairs — archives for a taxonomy with no post
    /// lang land in this lang instead of primary.
    pub tax_lang: &'static [(&'static str, &'static str)],
    /// Primary↔twin taxonomy pairs — `localized_taxonomy` picks the
    /// member whose `tax_lang` matches the requested language.
    pub tax_pairs: &'static [(&'static str, &'static str)],
    /// Element id of the theme's core stylesheet <link> — the anchor
    /// `inject_post_assets` inserts eztoc CSS before. Empty = append.
    pub css_link_id: &'static str,
    /// Element id of the theme's core inline-script — anchor for
    /// footer asset injection. Empty = append.
    pub js_extra_id: &'static str,
    /// Mega-menu root class (`traveltour-mega-menu` → `mega-menu`).
    pub mega_class: &'static str,
    /// Non-mega flyout marker class on top-level items
    /// (`traveltour-normal-menu` → `normal-menu`).
    pub normal_menu_class: &'static str,
    /// Menu <li> markup variant — `Wp` = superfish/WP default,
    /// `Caret` = elementor-style `<span class="menu-title">` + caret.
    pub menu_renderer: MenuRenderer,
    /// (taxonomy → (plural, singular)) admin labels for theme-claimed
    /// taxonomies (duree/region/… live in `tax_label`'s builtin table).
    pub tax_labels: &'static [(&'static str, &'static str, &'static str)],
    /// Page-with-sidebar scaffold: wraps `body` + `sidebar_html` in the
    /// theme's container markup. `default_page_sidebar` for data themes;
    /// Rust themes preserve their byte-parity wrapper.
    pub wrap_page_sidebar: fn(&ThemeDef, body: &str, sidebar_html: &str, pos: &str) -> String,
}

/// Menu markup family a theme's base template expects.
#[derive(Clone, Copy)]
pub enum MenuRenderer {
    /// WP/superfish: `<li class><a href>{title}</a><ul class="sub-menu">`.
    Wp,
    /// Elementor/tevily: `<a><span class="menu-title">{t}</span>{caret}</a>`
    /// inside `<ul class="submenu-inner">`.
    Caret,
}

/// Generic page-sidebar scaffold — a TOML theme's `sidebar_wrap_class`
/// drives the outer container; inner divs are unbranded.
pub(crate) fn default_page_sidebar(theme: &ThemeDef, body: &str, sidebar_html: &str, pos: &str) -> String {
    format!(
        "<div class=\"{wrap}\"><div class=\"sidebar-wrap clearfix sidebar-style-{pos}\"><div class=\"sidebar-center\"><div class=\"content-area\">{body}</div></div>{sidebar_html}</div>",
        wrap = theme.sidebar_wrap_class,
    )
}

/// Generic page-title banner — unbranded classes; `page_title_wrap` for
/// data themes. `caption` renders only when the archive supplies one.
pub(crate) fn default_page_title(_t: &ThemeDef, bg: &str, caption: Option<&str>, title: &str) -> String {
    let cap = caption.map(|c| format!("<h3 class=\"page-caption\">{c}</h3>")).unwrap_or_default();
    format!("<div class=\"page-title-wrap\"{bg}><div class=\"page-title-container\"><div class=\"page-title-content\">{cap}<h1 class=\"page-title\">{title}</h1></div></div></div>")
}

/// Neutral widget title markup for data/default themes — the plain
/// escape (no brand classes); empty title → empty string.
pub(crate) fn default_widget_title(_t: &ThemeDef, title: &str) -> String {
    if title.is_empty() {
        return String::new();
    }
    format!("<h3 class=\"widget-title\">{}</h3>", crate::html_decode::esc(title))
}

/// A theme with no frozen shortcode captures — the expander leaves the
/// tag verbatim.
pub(crate) fn no_shortcode_static(_t: &ThemeDef, _tag: &str) -> Option<&'static str> {
    None
}

/// Shared taxonomy-map readers — themes with a `taxonomy_map_key` get
/// `cat_path`/`tag_base`/`cat_lang`/`taxonomy_base` for free; themes
/// without one get the generic flat shape (`{tax}/{slug}`, category
/// unnested at root, tag under `tag`).
fn tax_map(conn: &rusqlite::Connection, theme: &ThemeDef) -> serde_json::Value {
    if theme.taxonomy_map_key.is_empty() && theme.taxonomy_map_json.is_empty() {
        return serde_json::Value::Null;
    }
    conn.query_row(
        "SELECT value FROM settings WHERE key=?1",
        [theme.taxonomy_map_key],
        |r| r.get::<_, String>(0),
    )
    .ok()
    .and_then(|s| serde_json::from_str(&s).ok())
    .or_else(|| serde_json::from_str(theme.taxonomy_map_json).ok())
    .unwrap_or(serde_json::Value::Null)
}

/// Parent slug of a category in the theme's map (`None` = root/unknown).
fn map_parent(map: &serde_json::Value, slug: &str) -> Option<String> {
    map["cat_edges"].get(slug)?.as_str().map(str::to_string)
}

/// Hierarchical category archive path for the resolved theme
/// (`grand/parent/child`); generic themes stay flat (`{slug}`).
pub(crate) fn theme_cat_path(conn: &rusqlite::Connection, theme: &ThemeDef, slug: &str) -> String {
    let map = tax_map(conn, theme);
    let mut chain = vec![slug.to_string()];
    let mut cur = slug.to_string();
    let mut guard = 0;
    while let Some(p) = map_parent(&map, &cur) {
        if p == cur { break; }
        chain.insert(0, p.clone());
        cur = p;
        guard += 1;
        if guard > 6 { break; }
    }
    chain.join("/")
}

/// Categories whose parent is `slug` in the theme's map — the inverse
/// of `theme_cat_path`'s walk. Empty for map-less themes.
pub(crate) fn theme_cat_children(conn: &rusqlite::Connection, theme: &ThemeDef, slug: &str) -> Vec<String> {
    let map = tax_map(conn, theme);
    let mut v: Vec<String> = Vec::new();
    if let Some(obj) = map["cat_edges"].as_object() {
        for (c, p) in obj {
            if p.as_str() == Some(slug) {
                v.push(c.clone());
            }
        }
    }
    v
}

/// The post_tag archive base for the resolved theme — the map's
/// `taxonomy_bases.post_tag`, else the theme's `tag_base`, else `tag`.
pub(crate) fn theme_tag_base(conn: &rusqlite::Connection, theme: &ThemeDef) -> String {
    tax_map(conn, theme)["taxonomy_bases"]
        .get("post_tag")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| (!theme.tag_base.is_empty()).then(|| theme.tag_base.to_string()))
        .unwrap_or_else(|| "tag".to_string())
}

/// Public archive base for any taxonomy in the theme's map —
/// `None` → the generic `{tax}` segment.
pub(crate) fn theme_taxonomy_base(conn: &rusqlite::Connection, theme: &ThemeDef, tax: &str) -> Option<String> {
    tax_map(conn, theme)["taxonomy_bases"]
        .get(tax)?
        .as_str()
        .map(str::to_string)
}

/// Whether `slug`'s category chain resolves inside a non-primary root
/// set — the map declares `lang_roots` (`{lang: [root…]}`, N-lang form)
/// or the legacy `es_roots`/`es_roots_lang` pair (kept readable so
/// existing 2-lang sites keep working; `lang_roots` wins when present).
/// No roots → a `-{lang}` slug suffix means that lang's tree when the
/// lang is registered for the site or the theme (`lang()`); primary
/// otherwise.
pub fn theme_cat_lang(conn: &rusqlite::Connection, theme: &ThemeDef, slug: &str, primary: &str) -> Option<String> {
    let map = tax_map(conn, theme);
    // {lang → roots} from the blob, `lang_roots` first, legacy keys as
    // fallback. Missing blob → empty map (the suffix heuristic still runs).
    let mut root_sets: Vec<(String, Vec<String>)> = Vec::new();
    if let Some(obj) = map["lang_roots"].as_object() {
        for (lang, roots) in obj {
            root_sets.push((
                lang.clone(),
                roots.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            ));
        }
    } else if let Some(roots) = map["es_roots"].as_array() {
        let lang = map["es_roots_lang"].as_str().unwrap_or("es").to_string();
        root_sets.push((lang, roots.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()));
    }
    if !root_sets.is_empty() {
        // Walk the parent chain once; a hit on any root set lands in that
        // lang's tree.
        let mut cur = slug.to_string();
        let mut guard = 0;
        loop {
            if let Some((lang, _)) = root_sets.iter().find(|(_, roots)| roots.contains(&cur)) {
                return Some(lang.clone());
            }
            match map_parent(&map, &cur) {
                Some(p) if p != cur => cur = p,
                _ => break,
            }
            guard += 1;
            if guard > 6 { break; }
        }
    }
    // Polylang convention the retired cat_path_is_es kept unconditional:
    // a `-{lang}` slug suffix means the non-primary tree even when the map
    // has no roots entry (theme declares cat_edges only). `lang` covers
    // the site's configured secondary langs plus every lang the theme's
    // tax_lang/blob declarations register — N-lang, no "es" literal.
    for lang in secondary_langs(conn, theme, &map, primary) {
        if slug.ends_with(&format!("-{lang}")) {
            return Some(lang);
        }
    }
    Some(primary.to_string())
}

/// Secondary langs the taxonomy-map readers recognize: configured site
/// languages (minus primary), theme `tax_lang` values, and the legacy
/// `es_roots_lang` blob key. De-duped, insertion order.
fn secondary_langs(conn: &rusqlite::Connection, theme: &ThemeDef, map: &serde_json::Value, primary: &str) -> Vec<String> { // gate-allow: sync — every caller reaches this on a blocking thread (spawn_blocking / build / boot)
    let mut out: Vec<String> = Vec::new();
    let mut push = |l: &str| {
        if !l.is_empty() && l != primary && !out.iter().any(|x| x == l) {
            out.push(l.to_string());
        }
    };
    for l in &crate::lang::config(conn).langs {
        push(l);
    }
    for (_, l) in theme.tax_lang {
        push(l);
    }
    if let Some(l) = map["es_roots_lang"].as_str() {
        push(l);
    }
    if let Some(obj) = map["lang_roots"].as_object() {
        for l in obj.keys() {
            push(l);
        }
    }
    out
}

/// Taxonomy → lang for archive paths with no post-lang signal — the
/// theme's `tax_lang` pairs, else the site primary.
pub(crate) fn theme_tax_lang<'a>(theme: &'a ThemeDef, tax: &str, primary: &'a str) -> &'a str {
    theme
        .tax_lang
        .iter()
        .find(|(t, _)| *t == tax)
        .map(|(_, l)| *l)
        .unwrap_or(primary)
}

/// Taxonomy translation pairs on `ThemeDef::tax_pairs`:
/// (primary-lang taxonomy, twin taxonomy) — ("style","estilo"),
/// ("duree","duracion"). `lang` resolves the twin through `tax_lang`,
/// so callers pass the primary name + lang and get the right taxonomy —
/// no per-lang taxonomy literals outside theme data. `lang == primary`
/// (or an unmapped lang) returns `name`. A pair whose twin maps to a
/// different lang is skipped, not returned — later pairs can still
/// satisfy N-lang themes.
pub fn localized_taxonomy<'a>(theme: &'a ThemeDef, name: &'a str, lang: &str, primary: &str) -> &'a str {
    if lang == primary {
        return name;
    }
    for (a, b) in theme.tax_pairs {
        if *a == name && theme_tax_lang(theme, b, primary) == lang {
            return b;
        }
    }
    name
}

/// Admin label override for theme-claimed taxonomies — `tax_label`
/// consults this before its builtin table.
pub(crate) fn theme_tax_label<'a>(theme: &'a ThemeDef, tax: &str) -> Option<(&'a str, &'a str)> {
    theme
        .tax_labels
        .iter()
        .find(|(t, _, _)| *t == tax)
        .map(|(_, plural, singular)| (*plural, *singular))
}

/// The adapter carrying this stylesheet's theme, if any.
/// `stylesheet` may be a child name — matching is the adapter's
/// `matches_stylesheet`, never a substring guess here.
pub(crate) fn theme_for_stylesheet(stylesheet: &str) -> Option<&'static ThemeDef> {
    ADAPTERS
        .iter()
        .filter_map(|a| a.theme())
        .find(|t| (t.matches_stylesheet)(stylesheet))
}

/// The theme owning `opt` as its general-settings option — adapters
/// first, then TOML themes. Dumps that omit `stylesheet`/`template`
/// still name their theme through `<theme>_general`; import resolves
/// the theme this way before falling back to DEFAULT_THEME.
pub(crate) fn theme_for_general_option(opt: &str) -> Option<&'static ThemeDef> {
    ADAPTERS
        .iter()
        .filter_map(|a| a.theme())
        .find(|t| !t.general_option.is_empty() && t.general_option == opt)
        .or_else(|| themes::theme_for_general_option(opt))
}

/// The adapter carrying this stylesheet's theme — needed where the
/// caller must reach the adapter itself (import runs its `migrate()` on
/// the staged DB), not just its `ThemeDef`.
pub(crate) fn adapter_for_stylesheet(stylesheet: &str) -> Option<&'static dyn PluginAdapter> {
    ADAPTERS
        .iter()
        .find(|a| a.theme().is_some_and(|t| (t.matches_stylesheet)(stylesheet)))
        .copied()
}

/// Neutral fallback when no adapter (Rust or TOML) claims the site's
/// stylesheet — generic flat archives, no brand classes. Sites on a
/// vendored theme we haven't ported still render legibly instead of
/// leaking another theme's chrome.
pub(crate) static DEFAULT_THEME: ThemeDef = ThemeDef {
    matches_stylesheet: |_| true,
    stylesheet: "",
    template_dir: "default",
    body_suffix: "rp-theme-default",
    page_title_wrap: default_page_title,
    preferred_locations: &["primary", "main-menu", "header-navigation"],
    taxonomy_map_key: "",
    taxonomy_map_json: "",
    tag_base: "",
    widget_class: "widget",
    general_option: "",
    general_settings_key: "",
    widget_title_block: default_widget_title,
    shortcode_static: no_shortcode_static,
    sidebar_wrap_class: "content-container sidebar-wrap",
    body_class: "rp-body",
    settings_key_prefix: "",
    tax_lang: &[],
    tax_pairs: &[],
    tax_labels: &[],
    css_link_id: "",
    js_extra_id: "",
    mega_class: "mega-menu",
    normal_menu_class: "normal-menu",
    menu_renderer: MenuRenderer::Wp,
    wrap_page_sidebar: default_page_sidebar,
};

/// The resolved theme for a site — adapter/TOML match first, DEFAULT_THEME
/// otherwise. Callers that rendered "traveltour-or-bust" markup go
/// through this so a new theme onboards as data.
pub fn theme_for_site(conn: &rusqlite::Connection) -> &'static ThemeDef {
    let stylesheet = conn
        .query_row(sql!("render__select_stylesheet"), [], |r| r.get::<_, String>(0))
        .unwrap_or_default();
    theme_for_stylesheet(&stylesheet)
        .or_else(|| themes::theme_for_stylesheet(&stylesheet))
        .unwrap_or(&DEFAULT_THEME)
}

/// TOML theme specs (`themes/specs/*.toml` embedded by build.rs) —
/// declarative subset of ThemeDef for themes without code-level quirks.
pub(crate) mod themes;

/// Every registered `KindDef` in declared order: core first, then
/// adapters — drives generic menu/dashboard rendering.
pub(crate) fn kind_defs() -> Vec<&'static KindDef> {
    CORE_KINDS
        .iter()
        .chain(adapters().iter().flat_map(|a| a.kinds()))
        .collect()
}

/// Run every adapter's `migrate()` at boot — each applies only its own
/// tables/promotions, idempotently — then fold each adapter's declared
/// `legacy_option_keys` into the `plugin:<slug>:` namespace. The fold is
/// `INSERT OR IGNORE` + `SELECT`-copy: legacy rows survive (zero data
/// loss on rollback), already-migrated keys are skipped, and a key that
/// was *partially* namespaced by an earlier run is untouched.
pub(crate) fn migrate_all(db: &Path) { // gate-allow: sync — every caller reaches this on a blocking thread (spawn_blocking / build / boot)
    themes::assert_no_rust_collision();
    for adapter in adapters() {
        if let Err(e) = adapter.migrate(db) {
            eprintln!("plugin migrate {}: {e:#}", adapter.slug());
        }
        if !adapter.legacy_option_keys().is_empty() {
            if let Err(e) = crate::db::misc::migrate_plugin_keys(db, adapter.slug(), adapter.legacy_option_keys()) {
                eprintln!("plugin key migrate {}: {e:#}", adapter.slug());
            }
        }
    }
}

/// Dispatch `/admin/plugins/<slug>/<path>` to a `ScreenDef`; `None` → 404.
pub(crate) fn screen_for(slug: &str, path: &str) -> Option<(&'static dyn PluginAdapter, &'static ScreenDef)> {
    let a = adapter_for_slug(slug)?;
    a.screens()
        .iter()
        .find(|s| s.path == path)
        .map(|s| (a, s))
}

/// Default submenu for a registered kind: list, add-new, then the kind's
/// declared taxonomies. An adapter's `submenu()` overrides this.
pub(crate) fn submenu_for(db: &Path, def: &'static KindDef) -> Vec<(String, String)> { // gate-allow: sync — every caller reaches this on a blocking thread (spawn_blocking / build / boot)
    let adapter = adapter_for_kind(def.kind);
    if let Some(sub) = adapter.and_then(|a| a.submenu(db, def)) {
        return sub;
    }
    let mut sub = vec![
        (format!("/admin/{}", def.section), format!("All {}", def.list_title)),
        (format!("/admin/new/{}", def.kind), def.add_label.to_string()),
    ];
    for t in def.taxonomies {
        let theme = crate::db::conn(db).map(|c| theme_for_site(&c)).unwrap_or(&DEFAULT_THEME);
        let (label, _) = crate::admin::tax_label_for(theme, t);
        sub.push((format!("/admin/tax/{t}"), label));
    }
    sub
}

/// Editor hooks for a kind — the editor-metabox half of the old
/// `plugins::find`. `None` → classic editor, no metabox.
pub(crate) fn editor_for(kind: &str) -> Option<&'static EditorHooks> {
    kind_def(kind).and_then(|d| d.editor)
}

/// Section a taxonomy's admin screens live under: the owning kind's
/// `KindDef.taxonomies`; unclaimed taxonomies keep the pre-registry
/// "everything else → tours" catch-all until phase 02 declares owners.
pub(crate) fn tax_section(tax: &str) -> &'static str {
    // Declared order (CORE first, then ADAPTERS) — HashMap iteration would
    // make a duplicate claim nondeterministic across restarts.
    for def in CORE_KINDS
        .iter()
        .chain(adapters().iter().flat_map(|a| a.kinds()))
    {
        if def.taxonomies.contains(&tax) {
            return def.section;
        }
    }
    "tours"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_registry() {
        // Core kinds resolve with zero plugin involvement.
        assert_eq!(kind_def("post").unwrap().section, "posts");
        assert_eq!(kind_def("page").unwrap().noun, "Page");
        // Plugin kinds come from their adapter.
        assert_eq!(kind_def("tour").unwrap().section, "tours");
        assert!(kind_def("tour").unwrap().editor.is_some());
        // Unknown kinds are errors, not posts (R1).
        assert!(kind_def("bogus").is_none());
        assert!(kind_for_section("bogus").is_none());
        // Section lookup works for both spellings.
        assert_eq!(kind_for_section("tours").unwrap().kind, "tour");
        // Taxonomies route to their owning kind's section.
        assert_eq!(tax_section("category"), "posts");
        assert_eq!(tax_section("duree"), "tours");
        assert_eq!(tax_section("tour-age"), "tours");
        // Real DBs store these two capitalized — routing AND the declared
        // owner must match (membership is asserted too: "tours" is also
        // the catch-all default, so routing alone would be a phantom).
        let tour = kind_def("tour").unwrap();
        for t in ["Activites", "Pays"] {
            assert!(tour.taxonomies.contains(&t), "{t} missing from tour taxonomies");
            assert_eq!(tax_section(t), "tours");
        }
        // Unclaimed keeps the pre-registry "everything else → tours" default.
        assert_eq!(tax_section("portfolio_category"), "tours");
    }
}
