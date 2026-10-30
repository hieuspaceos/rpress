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
            // A site that moved OFF traveltour must not keep its seeds:
            // stale rows make a tevily site emit traveltour chrome.
            c.execute(
                "DELETE FROM settings WHERE key LIKE 'plugin:theme-traveltour:%'",
                [],
            )?;
            return Ok(());
        }
        c.execute(
            "INSERT OR IGNORE INTO settings(key, value) VALUES(?1, ?2)",
            rusqlite::params!["plugin:theme-traveltour:taxonomy_map", TAXONOMY_MAP_JSON],
        )?;
        carry_custom_css(&c, db)?;
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
    fn theme(&self) -> Option<&'static crate::plugins::ThemeDef> {
        use crate::plugins::ThemeDef;
        static DEF: ThemeDef = ThemeDef {
            matches_stylesheet: matches_traveltour,
            template_dir: "traveltour",
            body_suffix: "wp-theme-traveltour wp-child-theme-traveltour-child gdlr-core-body tourmaster-body metaslider-plugin traveltour-body traveltour-body-front traveltour-full  traveltour-with-sticky-navigation gdlr-core-link-to-lightbox",
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


/// Theme-generated css (`wp-content/uploads/traveltour-style-custom.css`)
/// is a site asset, not repo data: rewrite its own-origin `url()` refs to
/// site-relative paths, emit the `.scoped.css` twin Live Mode needs, and
/// record the link path (`plugin:theme:css_upload_path`) the base
/// template reads through `site.chrome.theme_css`. Absent file → key
/// removed, never an aucoeur fallback.
/// Site-generated `*-style-custom.css` files the base templates link:
/// (settings key the ctx reads, uploads filename). Tourmaster's two
/// files live here too — same carry semantics as the theme's own css.
const CUSTOM_CSS: &[(&str, &str)] = &[
    ("plugin:theme:css_upload_path", "traveltour-style-custom.css"),
    ("plugin:tourmaster:css_upload_path", "tourmaster-style-custom.css"),
    ("plugin:tourmaster:css_global_upload_path", "tourmaster-global-style-custom.css"),
    ("plugin:tourmaster:css_room_upload_path", "tourmaster-room-style-custom.css"),
];

fn carry_custom_css(c: &Connection, db: &Path) -> Result<()> {
    let www = db
        .parent()
        .map(|p| p.join("www"))
        .unwrap_or_default();
    for (key, name) in CUSTOM_CSS {
        let file = www.join("wp-content/uploads").join(name);
        if !file.is_file() {
            c.execute("DELETE FROM settings WHERE key=?1", [key])?;
            continue;
        }
        carry_one_css(c, &file, key, name)?;
    }
    Ok(())
}

fn carry_one_css(c: &Connection, file: &Path, key: &str, name: &str) -> Result<()> {
    // Own-origin hosts to strip: live site_url + imported site_url +
    // url_aliases. `strip_origin` re-anchors on the live site_url;
    // relative `url(...)` refs pass through untouched.
    let setting = |k: &str| -> String {
        c.query_row("SELECT value FROM settings WHERE key=?1", [k], |r| {
            r.get::<_, String>(0)
        })
        .unwrap_or_default()
    };
    let site_url = setting("site_url");
    let mut aliases: Vec<String> = setting("url_aliases")
        .split(',')
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .collect();
    let asset = setting("asset_origin");
    if !asset.is_empty() {
        aliases.push(asset);
    }
    let css = std::fs::read_to_string(file)?;
    // Theme css references only the site's own assets: any host whose
    // `url()` path starts `/wp-content/` is this site's origin
    // (covers test-subdomain mismatches like aucoeurvietnam.com vs
    // test.aucoeurvietnam.com where url_aliases is empty).
    let host_of = |u: &str| -> Option<String> {
        let rest = u
            .strip_prefix("http://")
            .or_else(|| u.strip_prefix("https://"))?;
        let end = rest.find('/').unwrap_or(rest.len());
        Some(rest[..end].to_string())
    };
    for (i, _) in css.match_indices("http") {
        let tail = &css[i..];
        let end = tail
            .find(|c: char| c == ')' || c == '\'' || c == '"' || c.is_whitespace() || c == ';')
            .unwrap_or(tail.len());
        let u = &tail[..end];
        if let Some(h) = host_of(u) {
            if u.contains("/wp-content/") && !aliases.contains(&h) {
                aliases.push(h);
            }
        }
    }
    // Cheap url( ...) walker: exact per-URL handling via strip_origin
    // keeps foreign hosts (cdn fonts etc.) absolute.
    let mut out = String::with_capacity(css.len());
    let mut rest = css.as_str();
    while let Some(p) = rest.find("url(") {
        out.push_str(&rest[..p + 4]);
        let tail = &rest[p + 4..];
        let (url, rest_off) = match tail.chars().next() {
            Some(q @ ('"' | '\'')) => {
                // `url("X")`: inner starts past the open quote; `close`
                // is the matching close quote (keep both in output).
                let inner = &tail[1..];
                let close = inner.find(q).unwrap_or(inner.len());
                (&inner[..close], 1 + close)
            }
            _ => {
                let close = tail.find(')').unwrap_or(tail.len());
                (&tail[..close], close)
            }
        };
        let stripped = crate::url_util::strip_origin(url, &site_url, &aliases.join(","));
        // Re-anchor re-absolute → path-only for portability across envs:
        let local = stripped
            .strip_prefix(site_url.trim_end_matches('/'))
            .unwrap_or(&stripped);
        out.push_str(local);
        rest = &tail[rest_off..];
    }
    out.push_str(rest);
    if out != css {
        std::fs::write(&file, &out)?;
    }
    let scoped = file.with_file_name(format!(
        "{}.scoped.css",
        name.strip_suffix(".css").unwrap_or(name)
    ));
    std::fs::write(&scoped, crate::css_scope::scope_css(&out))?;
    c.execute(
        "INSERT OR REPLACE INTO settings(key, value) VALUES(?1, ?2)",
        rusqlite::params![key, format!("wp-content/uploads/{name}")],
    )?;
    Ok(())
}
