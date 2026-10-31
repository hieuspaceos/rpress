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
            page_title_wrap: traveltour_page_title,
            preferred_locations: &["header-navigation", "main-menu", "main_navigation", "primary"],
            taxonomy_map_key: "plugin:theme-traveltour:taxonomy_map",
            taxonomy_map_json: TAXONOMY_MAP_JSON,
            tag_base: "etiquette",
            widget_class: "traveltour-widget",
            sidebar_wrap_class: "traveltour-content-container traveltour-container",
            css_link_id: "traveltour-style-core-css",
            js_extra_id: "traveltour-script-core-js-extra",
            mega_class: "traveltour-mega-menu",
            normal_menu_class: "traveltour-normal-menu",
            menu_renderer: super::MenuRenderer::Wp,
            body_class: "traveltour-body traveltour-body-front",
            settings_key_prefix: "traveltour_",
            tax_lang: &[
                ("region-es", "es"), ("perfil", "es"), ("duracion", "es"), ("estilo", "es"),
            ],
            tax_pairs: &[
                ("duree", "duracion"), ("region", "region-es"),
                ("style", "estilo"), ("profil", "perfil"),
            ],
            tax_labels: &[
                ("duree", "Durée", "Durée"), ("duracion", "Durée", "Durée"),
                ("region", "Région", "Région"), ("region-es", "Région", "Région"),
                ("style", "Style", "Style"), ("estilo", "Style", "Style"),
                ("profil", "Profil", "Profil"), ("perfil", "Profil", "Profil"),
                ("activites", "Activités", "Activité"),
                ("pays", "Pays", "Pays"),
            ],
            wrap_page_sidebar: traveltour_page_sidebar,
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

/// Byte-parity page-with-sidebar scaffold — the exact traveltour
/// container chain WP emits (content-container → sidebar-wrap →
/// sidebar-center → content-area); used at render.rs's page branch.
fn traveltour_page_sidebar(_t: &super::ThemeDef, body: &str, sidebar_html: &str, pos: &str) -> String {
    format!(
        "<div class=\"gdlr-core-page-builder-body clearfix\"><div class=\"traveltour-content-container traveltour-container\"><div class=\" traveltour-sidebar-wrap clearfix traveltour-line-height-0 traveltour-sidebar-style-{pos}\" ><div class=\" traveltour-sidebar-center traveltour-column-45 traveltour-line-height\" ><div class=\"traveltour-content-area traveltour-item-pdlr\" >{body}</div></div>{sidebar_html}</div>"
    )
}

/// Byte-parity page-title banner — the exact wrap chain WP/traveltour
/// emits; `caption` is the archive term label (chrome page-title passes
/// `None`).
fn traveltour_page_title(_t: &super::ThemeDef, bg: &str, caption: Option<&str>, title: &str) -> String {
    let cap = caption
        .map(|c| format!("<h3 class=\"traveltour-page-caption\"  >{c}</h3>"))
        .unwrap_or_default();
    format!("<div class=\"traveltour-page-title-wrap  traveltour-style-large traveltour-center-align\"{bg} ><div class=\"traveltour-header-transparent-substitute\" ></div><div class=\"traveltour-page-title-overlay\"  ></div><div class=\"traveltour-page-title-container traveltour-container\" ><div class=\"traveltour-page-title-content traveltour-item-pdlr\"  >{cap}<h1 class=\"traveltour-page-title\"  >{title}</h1></div></div></div>")
}

// ---------- runtime readers -------------------------------------------------

// cat_path/tag_base/cat_children/cat_lang moved to plugins/mod.rs as the
// generic `theme_*` API over `ThemeDef::taxonomy_map_key` — this module's
// only remaining runtime data is the seed JSON (TAXONOMY_MAP_JSON).


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
        let (url, quote, rest_off) = match tail.chars().next() {
            Some(q @ ('"' | '\'')) => {
                // `url("X")`: emit the quote pair — the old code dropped
                // the open quote but kept the close, corrupting rules
                // into `url(path")` (all @font-face fonts 404'd).
                let inner = &tail[1..];
                let close = inner.find(q).unwrap_or(inner.len());
                (&inner[..close], Some(q), 2 + close)
            }
            _ => {
                let close = tail.find(')').unwrap_or(tail.len());
                (&tail[..close], None, close)
            }
        };
        let stripped = crate::url_util::strip_origin(url, &site_url, &aliases.join(","));
        // Re-anchor re-absolute → path-only for portability across envs:
        let local = stripped
            .strip_prefix(site_url.trim_end_matches('/'))
            .unwrap_or(&stripped);
        if let Some(q) = quote {
            out.push(q);
            out.push_str(local);
            out.push(q);
        } else {
            out.push_str(local);
        }
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
