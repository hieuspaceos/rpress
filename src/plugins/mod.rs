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
pub(crate) mod theme_tevily;
pub(crate) mod theme_traveltour;
pub(crate) mod tourmaster;
pub(crate) mod goodlayers;
/// `/admin/plugins` index + shared settings-screen helpers.
pub(crate) mod index;
pub(crate) mod chaty;
pub(crate) mod misc;
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
    pub(crate) row_cols: fn(&crud::AdminListRow, &str) -> String,
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

const HEAD_POST: &str = r#"<tr><td class="manage-column column-cb check-column"></td><th scope="col" class="manage-column column-title column-primary">Title</th><th scope="col">Author</th><th scope="col">Categories</th><th scope="col">FR / ES</th><th scope="col">Comments</th><th scope="col" class="rm-seo-col">SEO Details</th></tr>"#;
const HEAD_PAGE: &str = r#"<tr><td class="manage-column column-cb check-column"></td><th scope="col" class="manage-column column-title column-primary">Title</th><th scope="col">Author</th><th scope="col">FR / ES</th><th scope="col">Comments</th><th scope="col" class="rm-seo-col">SEO Details</th></tr>"#;

fn post_row_cols(r: &crud::AdminListRow, kind: &str) -> String {
    format!(
        "<td>—</td><td class=\"categories\">{}</td><td class=\"pll\">{}</td><td class=\"comments\">{}</td>",
        crate::admin::posts::dash(&r.categories),
        crate::admin::posts::pll_cell(r, kind),
        r.comments,
    )
}

fn page_row_cols(r: &crud::AdminListRow, kind: &str) -> String {
    format!(
        "<td>—</td><td class=\"pll\">{}</td><td class=\"comments\">{}</td>",
        crate::admin::posts::pll_cell(r, kind),
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
    &theme_traveltour::ADAPTER,
    &theme_tevily::ADAPTER,
    &goodlayers::ADAPTER,
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
#[allow(dead_code)] // body_suffix/page_title_wrap/preferred_locations are
                    // consumed by the phase-04/06 chrome+menu work; declared
                    // now so the contract is stable.
pub(crate) struct ThemeDef {
    /// `settings.stylesheet` may be a child theme name
    /// (`traveltour-child`, `tevily_child`) — the adapter decides.
    pub matches_stylesheet: fn(&str) -> bool,
    /// Templates dir under `templates/theme/` probed before the shared
    /// `templates/` root by `load_env`/`template_name`.
    pub template_dir: &'static str,
    /// `site.theme.body_suffix` value seeded into settings at migrate.
    pub body_suffix: &'static str,
    /// `site.theme.page_title_wrap` value ("" → no wrap).
    pub page_title_wrap: &'static str,
    /// Menu locations this theme prefers, in pick order (phase-01).
    pub preferred_locations: &'static [&'static str],
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
pub(crate) fn migrate_all(db: &Path) {
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
pub(crate) fn submenu_for(db: &Path, def: &'static KindDef) -> Vec<(String, String)> {
    let adapter = adapter_for_kind(def.kind);
    if let Some(sub) = adapter.and_then(|a| a.submenu(db, def)) {
        return sub;
    }
    let mut sub = vec![
        (format!("/admin/{}", def.section), format!("All {}", def.list_title)),
        (format!("/admin/new/{}", def.kind), def.add_label.to_string()),
    ];
    for t in def.taxonomies {
        let (label, _) = crate::admin::tax_label(t);
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
