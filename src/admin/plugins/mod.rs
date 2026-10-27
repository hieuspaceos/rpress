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
use crate::admin::{crud, Sess, SiteCtx};
use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;


pub(crate) mod tourmaster;

use std::sync::LazyLock;
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
    /// Taxonomies owned by this kind — `tax_menu` routes their screens
    /// under this kind's section instead of a catch-all.
    pub(crate) taxonomies: &'static [&'static str],
    /// Editor hooks; `None` → classic editor, no metabox.
    pub(crate) editor: Option<&'static EditorHooks>,
}

/// An adapter-owned admin screen under `/admin/plugins/<slug>/…`
/// (dispatch lands in phase 02).
#[allow(dead_code)]
pub(crate) struct ScreenDef {
    /// Route path under the plugin's screen root, e.g. "settings".
    pub(crate) path: &'static str,
    /// Human title for the screen + submenu entry.
    pub(crate) title: &'static str,
    /// Render the screen body; `conn` is the site db.
    pub(crate) render: fn(&SiteCtx, &Sess, &Connection) -> Result<String>,
}

/// Import pipeline hooks (wired into `import.rs` in phase 03).
#[allow(dead_code)]
pub(crate) struct ImportHooks {
    /// Claim a wp `post_type` before the core `kind_of` match runs;
    /// `Some(kind)` takes over the row, `None` falls through to core.
    pub(crate) claim_type: fn(&str) -> Option<&'static str>,
    /// After the posts row is imported: write plugin tables/meta.
    /// Runs inside the import transaction.
    pub(crate) post_imported: fn(&Connection, i64) -> Result<()>,
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
    /// Admin screens under `/admin/plugins/<slug>/…`.
    fn screens(&self) -> &'static [ScreenDef] {
        &[]
    }
    /// Import hooks; `None` → core import only.
    fn import_hooks(&self) -> Option<&'static ImportHooks> {
        None
    }
    /// Optional static render for a post of this adapter's kind.
    fn render(&self, _conn: &Connection, _post: &PostForm) -> Option<String> {
        None
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
        url_base_key: None,
        taxonomies: &["category", "post_tag"],
        editor: None,
    },
    KindDef {
        kind: "page", section: "pages", noun: "Page", list_title: "Pages",
        add_label: "Add Page", head: HEAD_PAGE, row_cols: page_row_cols,
        url_base_key: None, taxonomies: &[], editor: None,
    },
];

/// Registered adapters — the compile-time plugin list.
static ADAPTERS: &[&'static dyn PluginAdapter] = &[&tourmaster::ADAPTER];

/// Indexed view over `ADAPTERS` + `CORE_KINDS`, built once.
struct Registry {
    by_slug: HashMap<&'static str, &'static dyn PluginAdapter>,
    by_kind: HashMap<&'static str, &'static KindDef>,
    by_section: HashMap<&'static str, &'static KindDef>,
}

static REGISTRY: LazyLock<Registry> = LazyLock::new(|| {
    let mut r = Registry {
        by_slug: HashMap::new(),
        by_kind: HashMap::new(),
        by_section: HashMap::new(),
    };
    for def in CORE_KINDS {
        r.by_kind.insert(def.kind, def);
        r.by_section.insert(def.section, def);
    }
    for adapter in ADAPTERS {
        r.by_slug.insert(adapter.slug(), *adapter);
        for def in adapter.kinds() {
            r.by_kind.insert(def.kind, def);
            r.by_section.insert(def.section, def);
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

/// Plugin slug → adapter (`/admin/plugins/<slug>` dispatch, phase 02).
#[allow(dead_code)]
pub(crate) fn adapter_for_slug(slug: &str) -> Option<&'static dyn PluginAdapter> {
    REGISTRY.by_slug.get(slug).copied()
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
        .chain(ADAPTERS.iter().flat_map(|a| a.kinds()))
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
