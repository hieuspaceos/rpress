//! `theme-traveltour` — the traveltour theme as a `Backend::Data` plugin.
//!
//! Owns what used to be frozen Rust constants: the category hierarchy
//! (`CAT_TREE`), the `etiquette/` post_tag base, the ES root list, and the
//! sidebar/footer seed payloads. All of it is data: `taxonomy_map.json`
//! (below) seeds `plugin:theme-traveltour:taxonomy_map` at migrate time;
//! the seed JSONs ship in `seed/` and apply through the widget seed pass.
//!
//! Runtime readers (`cat_path`, `tag_base`, `taxonomy_map`) read the
//! settings row — term CRUD never touches the map (R9); a theme update
//! is a reseed, not an edit.

use super::{Backend, PluginAdapter};
use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub(crate) mod widgets_theme;

/// The taxonomy map embedded at build time — the writer's seed payload.
/// Runtime readers go through `taxonomy_map(conn)`, not this const.
const TAXONOMY_MAP_JSON: &str = include_str!("taxonomy_map.json");

pub(crate) static ADAPTER: ThemeTraveltour = ThemeTraveltour;
/// Stylesheet normalization: child-theme dirs (`traveltour-child-*`)
/// still resolve to the parent's bases.
fn matches_traveltour(s: &str) -> bool {
    // Dash + underscore child-theme separators (`traveltour_child`).
    s == "traveltour"
        || s == "traveltour-child"
        || s == "traveltour_child"
        || s.starts_with("traveltour-child-")
        || s.starts_with("traveltour_child-")
        || s.starts_with("traveltour_child_")
}

pub(crate) struct ThemeTraveltour;

impl PluginAdapter for ThemeTraveltour {
    fn slug(&self) -> &'static str {
        "theme-traveltour"
    }
    /// No post kinds — pure data plugin.
    fn kinds(&self) -> &'static [super::KindDef] {
        &[]
    }
    /// Seed `taxonomy_map` when absent (idempotent `INSERT OR IGNORE`
    /// semantics: an existing row — admin-tuned or previously seeded —
    /// is left alone).
    fn migrate(&self, db: &Path) -> Result<()> {
        let c = crate::db::conn(db)?;
        // Seed only when this site's stylesheet claims traveltour — a
        // tevily/core site gets no traveltour taxonomy_map or widget rows.
        let stylesheet: String = c
            .query_row("SELECT value FROM settings WHERE key='stylesheet'", [], |r| r.get(0))
            .unwrap_or_default();
        if !matches_traveltour(&stylesheet) {
            return Ok(());
        }
        c.execute(
            "INSERT OR IGNORE INTO settings(key, value) VALUES(?1, ?2)",
            rusqlite::params!["plugin:theme-traveltour:taxonomy_map", TAXONOMY_MAP_JSON],
        )?;
        Ok(())
    }
    fn sidebars(&self) -> &'static [crate::widgets::Sidebar] {
        widgets_theme::SIDEBARS
    }
    fn seeds(&self) -> &'static [crate::widgets::Seed] {
        widgets_theme::SEEDS
    }
    /// `post_tag → "etiquette"` comes from the seeded map at runtime —
    /// `taxonomy_base` here only covers the compile-time dispatch point
    /// before a DB is reachable; keep it aligned with the JSON.
    fn taxonomy_base(&self, tax: &str) -> Option<&'static str> {
        (tax == "post_tag").then_some("etiquette")
    }
    fn backend(&self) -> Backend {
        Backend::Data
    }
    fn theme(&self) -> Option<&'static crate::admin::plugins::ThemeDef> {
        use crate::admin::plugins::ThemeDef;
        static DEF: ThemeDef = ThemeDef {
            matches_stylesheet: matches_traveltour,
            template_dir: "traveltour",
            body_suffix: "",
            page_title_wrap: "",
            preferred_locations: &["header-navigation", "main-menu", "main_navigation", "primary"],
        };
        Some(&DEF)
    }

    /* Theme option rows written inside the bridge (skins persist into
       `traveltour_color`, style settings etc.). Pulled into the
       `plugin:theme-traveltour:` namespace so rpress state mirrors the
       WP-side option without a per-key allowlist. */
    fn write_specs(&self) -> &'static [crate::admin::bridge::pull::WriteSpec] {
        static SPECS: &[crate::admin::bridge::pull::WriteSpec] = &[
            crate::admin::bridge::pull::WriteSpec {
                table: crate::admin::bridge::pull::PullTable::Settings,
                key_column: "key",
                key_prefix: "traveltour%",
                key_map: Some("plugin:theme-traveltour:"),
            },
        ];
        SPECS
    }
}

// ---------- runtime readers -------------------------------------------------

/// The theme's taxonomy map as a JSON object. `None` before `migrate()`
/// has run (unit tests, half-imported DBs) — callers fall back to the
/// compile-time shape the map was generated from, so behavior never
/// changes mid-phase.
pub(crate) fn taxonomy_map(conn: &Connection) -> Option<serde_json::Value> {
    crate::db::plugin_setting_raw_conn(conn, "theme-traveltour", "taxonomy_map")
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Fallback map — identical content to `taxonomy_map.json`. Used when the
/// settings row is missing so `cat_path` never has to be optional.
fn map_or_default(conn: &Connection) -> serde_json::Value {
    taxonomy_map(conn).unwrap_or_else(|| serde_json::from_str(TAXONOMY_MAP_JSON).unwrap())
}

/// Parent slug of a category (`None` = root or unknown).
fn cat_parent_in(map: &serde_json::Value, slug: &str) -> Option<String> {
    map["cat_edges"].get(slug)?.as_str().map(str::to_string)
}

/// Full hierarchical archive path for a category slug
/// (`grand/parent/child`) — the moved `render_post::cat_path`.
pub(crate) fn cat_path(conn: &Connection, slug: &str) -> String {
    let map = map_or_default(conn);
    let mut chain = vec![slug.to_string()];
    let mut cur = slug.to_string();
    let mut guard = 0;
    while let Some(p) = cat_parent_in(&map, &cur) {
        if p == cur {
            break;
        }
        chain.insert(0, p.clone());
        cur = p;
        guard += 1;
        if guard > 6 {
            break;
        }
    }
    chain.join("/")
}

/// Every category whose parent is `slug` (categories widget tree).
pub(crate) fn cat_children(conn: &Connection, slug: &str) -> Vec<String> {
    let map = map_or_default(conn);
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

/// Whether `cat_path` resolves through the ES category tree.
pub(crate) fn cat_path_is_es(conn: &Connection, slug: &str) -> bool {
    let map = map_or_default(conn);
    let roots: Vec<String> = map["es_roots"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let mut cur = slug.to_string();
    let mut guard = 0;
    loop {
        if roots.contains(&cur) {
            return true;
        }
        match cat_parent_in(&map, &cur) {
            Some(p) if p != cur => {
                cur = p;
            }
            _ => break,
        }
        guard += 1;
        if guard > 6 {
            break;
        }
    }
    slug.ends_with("-es")
}

/// Public archive base for a taxonomy (`post_tag → "etiquette"`), theme
/// data-driven. Returns `None` for unmapped taxonomies.
pub(crate) fn taxonomy_base_in(conn: &Connection, tax: &str) -> Option<String> {
    let map = map_or_default(conn);
    map["taxonomy_bases"].get(tax)?.as_str().map(str::to_string)
}

/// The post_tag archive base — `tag_base()` convenience wrapper.
pub(crate) fn tag_base(conn: &Connection) -> String {
    taxonomy_base_in(conn, "post_tag").unwrap_or_else(|| "etiquette".to_string())
}

