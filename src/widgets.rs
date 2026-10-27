//! Widget rendering + admin for WP parity.
//!
//! SEED_WIDGETS + src/seed/*.json are the REFERENCE THEME's default widget
//! set (traveltour: sidebar-post-*, sidebar-tour, sidebar-archive-*, footer-*).
//! To ship your own theme, replace `src/seed/` payloads and the SIDEBARS /
//! SEED_WIDGETS tables below — the runtime feature (sidebar/widget CRUD,
//! fragment render, dirty-flag regen) is theme-agnostic.
//! Registered sidebars + widget instances — WP widgets.php parity.
//!
//! Data model (see plans/261026-0426-wp-parity-widgets/):
//!   sidebars(id TEXT PK, name, lang) — the render regions: blog sidebar
//!     (per lang), tour booking bar, tour-archive sidebar (per lang), the
//!     four footer columns per language, plus admin-registered custom
//!     sidebars (GoodLayers dynamic sidebars → POST /admin/widgets/sidebar/*).
//!   widgets(id TEXT PK, sidebar_id, type, title, settings_json, ord) — the
//!     WP widget-instance equivalent. `id` keeps the wp widget id
//!     ("search-4", "wpcategorieswidget-9", …): it is the `id=` attribute in
//!     rendered markup.
//!
//! Two storage shapes in `settings_json`:
//!   * frozen HTML — `{"html": "…verbatim capture…"}` for search, text,
//!     custom_html, media_image, tour_category and any widget whose admin
//!     supplies a Content override. The fragments still contain live
//!     `{{ site_url }}`/`{{ lang }}` jinja, so `render_sidebar` runs
//!     `template_from_str` over them (AutoEscape::None ⇒ byte-identical).
//!   * settings — `categories` widgets store `{taxonomy, title, order[],
//!     children{parent: [slug…]}, show_counts}` and `tour_list` stores
//!     `{mode, manual_ids[], count, title, wrapper, inner}`. Both render
//!     live from DB data (phase 3):
//!       - categories taxonomy='category' builds its hierarchy from
//!         `render_post::cat_parent()` (terms.parent_id was never imported),
//!         counts = COUNT(post_terms JOIN posts status='publish'), and
//!         `cat-item-N` ids from `render_archive::wp_term_id`'s frozen wp map
//!         (extracted from the wpcategorieswidget-9 capture). FR/ES
//!         partitions come from the cat_parent tree's root sets.
//!       - tour_list cards reuse the archive helpers (tour-price-text →
//!         tourmaster-tour-price → price_range; promo-text → style terms;
//!         thumbnail = media sizes_json 'thumbnail' else full).
//!
//! Lang routing: render-side callers keep asking for the phase-1 base ids
//! ("sidebar-post", "sidebar-archive"); `resolve_id` maps them to the
//! per-lang rows (`-fr`/`-es`) so a single renderer picks the language.
//! `sidebar-tour` stays un-suffixed (one shared booking sidebar).

use anyhow::{Context, Result};
use minijinja::{context, Environment};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::HashMap;

/// A registered sidebar region.
pub(crate) struct Sidebar {
    pub id: &'static str,
    pub name: &'static str,
    /// 'fr' | 'es' | 'all' — informational for the admin screen; the
    /// renderer always serves the sidebar id the template asks for.
    pub lang: &'static str,
    /// Markup emitted around the widget sequence (verbatim capture of the
    /// area wrapper divs; empty for footers — their column wrappers stay in
    /// base_fr/base_es — and for admin-registered custom sidebars: rows in
    /// `sidebars` carry no wrapper, which is the accepted phase-3 choice —
    /// GoodLayers custom sidebars render bare widget markup).
    pub before: &'static str,
    pub after: &'static str,
}

const POST_AREA_BEFORE: &str = r#"<div class=" traveltour-sidebar-right traveltour-column-15 traveltour-line-height traveltour-line-height" ><div class="traveltour-sidebar-area traveltour-item-pdlr" >"#;
const POST_AREA_AFTER: &str = r#"</div></div></div></div>"#;

/// Registered sidebar regions (wp `register_sidebar` equivalents). Rows in
/// the `sidebars` table beyond this set are admin-registered custom
/// sidebars (the registry = const ∪ stored).
pub(crate) const SIDEBARS: &[Sidebar] = &[
    Sidebar { id: "sidebar-post-fr", name: "Blog sidebar (FR)", lang: "fr",
        before: POST_AREA_BEFORE,
        after: POST_AREA_AFTER },
    Sidebar { id: "sidebar-post-es", name: "Blog sidebar (ES)", lang: "es",
        before: POST_AREA_BEFORE,
        after: POST_AREA_AFTER },
    Sidebar { id: "sidebar-tour", name: "Tour booking sidebar", lang: "all",
        before: r#"<div class="tourmaster-tour-booking-bar-widget  traveltour-sidebar-area" >"#,
        after: r#"</div>"# },
    Sidebar { id: "sidebar-archive-fr", name: "Tour archive sidebar (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "sidebar-archive-es", name: "Tour archive sidebar (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-1-fr", name: "Footer column 1 (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-2-fr", name: "Footer column 2 (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-3-fr", name: "Footer column 3 (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-4-fr", name: "Footer column 4 (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-1-es", name: "Footer column 1 (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-2-es", name: "Footer column 2 (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-3-es", name: "Footer column 3 (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-4-es", name: "Footer column 4 (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
];

/// Base ids the render layer accepts; resolved to a per-lang row by
/// `resolve_id`. Custom sidebars are queried by their own id (no remap).
fn resolve_id(sidebar_id: &str, lang: &str) -> String {
    match sidebar_id {
        "sidebar-post" | "sidebar-archive" => format!("{sidebar_id}-{lang}"),
        other => other.to_string(),
    }
}

/// Registered widget types (wp `WP_Widget` equivalents).
pub(crate) const WIDGET_TYPES: &[(&str, &str)] = &[
    ("search", "Search"),
    ("categories", "Categories"),
    ("media_image", "Image"),
    ("text", "Text"),
    ("custom_html", "Custom HTML"),
    ("html_embed", "HTML Embed"),
    ("tour_list", "Tour List"),
    ("tour_category", "Tour Category"),
];

/// Admin edit-form field: `key` = settings_json key; `kind` ∈
/// text|textarea|html|checkbox|number|select|taxonomy|list|json|media.
/// `media` renders a hidden id field + hidden `{key}_url` + the shared
/// `media_frame` picker; `taxonomy` fills options from registered
/// taxonomies at render time; `list` is a comma-separated text input
/// stored as a JSON array; `json` is a raw JSON textarea (children map).
pub(crate) struct FieldSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: &'static str,
    pub required: bool,
    /// Static `(value, label)` pairs for `kind="select"`; empty elsewhere.
    pub options: &'static [(&'static str, &'static str)],
    pub help: &'static str,
}

const fn f(key: &'static str, label: &'static str, kind: &'static str, required: bool, help: &'static str) -> FieldSpec {
    FieldSpec { key, label, kind, required, options: &[], help }
}

/// Settings schema per widget type — consumed by the /admin/widgets
/// screen (save validates against this; page renders the inputs).
/// Frozen types keep the `html` raw fragment field (admin-trusted jinja);
/// every dynamic type also exposes `html` as an explicit render override.
pub(crate) fn widget_fields(typ: &str) -> &'static [FieldSpec] {
    const HTML_HELP: &str = "Raw HTML — jinja template ({{ site_url }}, {{ lang }}). Admin-trusted. When set, this fragment renders instead of the live output.";
    const HTML: &[FieldSpec] = &[];
    const CATEGORIES: &[FieldSpec] = &[
        f("taxonomy", "Taxonomy", "taxonomy", true, "Term list rendered live. 'category' builds the blog tree from the captured hierarchy; tour taxonomies use Order/Children below."),
        f("show_counts", "Show post counts", "checkbox", false, "Print the post-count span after each term (blog sidebar style)."),
        f("order", "Term order", "list", false, "Comma-separated slugs; terms not listed append alphabetically (ignored for 'category')."),
        f("children", "Children map", "json", false, "JSON {\"parent-slug\": [\"child-slug\", …]} (ignored for 'category')."),
        f("depth", "Depth", "number", false, "Maximum tree depth (0/empty = unlimited)."),
        f("html", "Content override", "html", false, HTML_HELP),
    ];
    const MEDIA_IMAGE: &[FieldSpec] = &[
        f("image_id", "Image", "media", false, "Pick from the media library — used when Content is empty."),
        f("html", "Content", "html", false, HTML_HELP),
    ];
    const HTML_FIELD: &[FieldSpec] = &[f("html", "Content", "html", false, HTML_HELP)];
    const TOUR_LIST: &[FieldSpec] = &[
        FieldSpec {
            key: "mode", label: "Mode", kind: "select", required: false,
            options: &[("recent", "Recent tours"), ("manual", "Manual selection")],
            help: "Recent = latest published tours for the sidebar language; Manual = the Post IDs below, in order.",
        },
        f("manual_ids", "Post IDs", "list", false, "Comma-separated post ids (manual mode), shown in list order."),
        f("count", "Number of tours", "number", false, "Recent-mode limit (default 10)."),
        f("wrapper", "Outer widget id", "text", false, "id= of the outermost div (defaults to the widget id)."),
        f("inner", "Tour widget id", "text", false, "id= of the inner tourmaster-tour-widget div (defaults to tourmaster-tour-widget-N)."),
        f("html", "Content override", "html", false, HTML_HELP),
    ];
    match typ {
        "search" => HTML_FIELD,
        "categories" => CATEGORIES,
        "media_image" => MEDIA_IMAGE,
        "text" | "custom_html" | "html_embed" | "tour_category" => HTML_FIELD,
        "tour_list" => TOUR_LIST,
        _ => HTML,
    }
}

/// Next `{type}-{n}` instance id (wp widget-numbering parity): max
/// numeric suffix already used for this type + 1, at least 2, skipping
/// collisions. Seeded ids like `wpcategorieswidget-9` keep their own
/// type's counter — `wpcategorieswidget` ≠ `categories`, so a new
/// categories widget starts at `categories-2`.
pub(crate) fn next_instance_id(conn: &Connection, typ: &str) -> Result<String> {
    let mut st = conn.prepare(
        "SELECT CAST(substr(id, length(?1) + 2) AS INTEGER) FROM widgets
         WHERE id LIKE (?1 || '-%')
           AND substr(id, length(?1) + 2) GLOB '[0-9]*'
           AND CAST(substr(id, length(?1) + 2) AS TEXT) = substr(id, length(?1) + 2)",
    )?;
    let max: i64 = st
        .query_map([typ], |r| r.get::<_, i64>(0))?
        .flatten()
        .max()
        .unwrap_or(1);
    let mut n = max + 1;
    loop {
        let id = format!("{typ}-{n}");
        let taken: bool = conn
            .query_row("SELECT COUNT(*) FROM widgets WHERE id=?1", [&id], |r| {
                r.get::<_, i64>(0)
            })?
            > 0;
        if !taken {
            return Ok(id);
        }
        n += 1;
    }
}

/// Sidebar registry = const ∪ stored rows (custom sidebars are plain rows).
/// `(id, name)` pairs in rowid order — the order comboboxes and the
/// /admin/widgets screen enumerate.
pub(crate) fn list(conn: &Connection) -> Result<Vec<(String, String)>> {
    ensure(conn)?;
    let mut st = conn.prepare("SELECT id, name FROM sidebars ORDER BY rowid")?;
    let out = st
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(out)
}

/// Whether `id` is one of the const (theme-registered) regions — custom
/// sidebars may be deleted from admin, these may not.
pub(crate) fn is_const_sidebar(id: &str) -> bool {
    SIDEBARS.iter().any(|s| s.id == id)
}

/// One seeded widget instance.
pub(crate) struct Seed {
    pub(crate) sidebar: &'static str,
    pub(crate) id: &'static str,
    pub(crate) typ: &'static str,
    pub(crate) title: &'static str,
    pub(crate) settings: &'static str,
}

/// Verbatim extraction of the pre-registry sources:
///   sidebar-post-fr ← the old templates/sidebar_post.html FR capture
///                   (search-4, media_image-4; wpcategorieswidget-9 is now a
///                   LIVE categories widget — the frozen capture became the
///                   `category` wp-id map + tree; custom_html-13 is now a
///                   LIVE tour_list widget in manual mode)
///   sidebar-post-es ← phase-3 hand-translation (NO ES capture exists —
///                   new content, not byte-parity)
///   sidebar-tour  ← the old templates/sidebar_tour.html
///   footer-N-{fr,es} ← the traveltour-footer-column blocks in
///                   templates/base_fr.html / base_es.html
///   sidebar-archive-{fr,es} ← the WIDGETS const (FR, verbatim) + ES mirror
///   Manual tour ids resolve the capture's /voyage/ slugs to posts (incl.
///   via _wp_old_slug for renamed tours); post 44901's '-copy' is excluded.
// Reference-theme seed payloads (see file header). Replace for your theme.
mod seed {
    pub const SIDEBAR_POST_FR_SEARCH_4: &str = include_str!("seed/sidebar-post-fr-search-4.json");
    pub const SIDEBAR_POST_FR_WPCATEGORIESWIDGET_9: &str = include_str!("seed/sidebar-post-fr-wpcategorieswidget-9.json");
    pub const SIDEBAR_POST_FR_MEDIA_IMAGE_4: &str = include_str!("seed/sidebar-post-fr-media_image-4.json");
    pub const SIDEBAR_POST_FR_CUSTOM_HTML_13: &str = include_str!("seed/sidebar-post-fr-custom_html-13.json");
    pub const SIDEBAR_POST_ES_SEARCH_5: &str = include_str!("seed/sidebar-post-es-search-5.json");
    pub const SIDEBAR_POST_ES_WPCATEGORIESWIDGET_10: &str = include_str!("seed/sidebar-post-es-wpcategorieswidget-10.json");
    pub const SIDEBAR_POST_ES_MEDIA_IMAGE_5: &str = include_str!("seed/sidebar-post-es-media_image-5.json");
    pub const SIDEBAR_POST_ES_CUSTOM_HTML_14: &str = include_str!("seed/sidebar-post-es-custom_html-14.json");
    pub const SIDEBAR_TOUR_TEXT_12: &str = include_str!("seed/sidebar-tour-text-12.json");
    pub const SIDEBAR_TOUR_TEXT_11: &str = include_str!("seed/sidebar-tour-text-11.json");
    pub const SIDEBAR_TOUR_TOURMASTER_WIDGET_TOUR_CATEGORY_6: &str = include_str!("seed/sidebar-tour-tourmaster-widget-tour-category-6.json");
    pub const FOOTER_1_FR_TEXT_1: &str = include_str!("seed/footer-1-fr-text-1.json");
    pub const FOOTER_2_FR_CUSTOM_HTML_3: &str = include_str!("seed/footer-2-fr-custom_html-3.json");
    pub const FOOTER_2_FR_TEXT_17: &str = include_str!("seed/footer-2-fr-text-17.json");
    pub const FOOTER_3_FR_CUSTOM_HTML_5: &str = include_str!("seed/footer-3-fr-custom_html-5.json");
    pub const FOOTER_3_FR_TEXT_15: &str = include_str!("seed/footer-3-fr-text-15.json");
    pub const FOOTER_4_FR_CUSTOM_HTML_6: &str = include_str!("seed/footer-4-fr-custom_html-6.json");
    pub const FOOTER_1_ES_TEXT_20: &str = include_str!("seed/footer-1-es-text-20.json");
    pub const FOOTER_2_ES_CUSTOM_HTML_9: &str = include_str!("seed/footer-2-es-custom_html-9.json");
    pub const FOOTER_2_ES_TEXT_21: &str = include_str!("seed/footer-2-es-text-21.json");
    pub const FOOTER_3_ES_CUSTOM_HTML_10: &str = include_str!("seed/footer-3-es-custom_html-10.json");
    pub const FOOTER_3_ES_TEXT_22: &str = include_str!("seed/footer-3-es-text-22.json");
    pub const FOOTER_4_ES_CUSTOM_HTML_11: &str = include_str!("seed/footer-4-es-custom_html-11.json");
    pub const SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_5: &str = include_str!("seed/sidebar-archive-fr-wpcategorieswidget-5.json");
    pub const SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_7: &str = include_str!("seed/sidebar-archive-fr-wpcategorieswidget-7.json");
    pub const SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_8: &str = include_str!("seed/sidebar-archive-fr-wpcategorieswidget-8.json");
    pub const SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_6: &str = include_str!("seed/sidebar-archive-fr-wpcategorieswidget-6.json");
    pub const SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_11: &str = include_str!("seed/sidebar-archive-es-wpcategorieswidget-11.json");
    pub const SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_12: &str = include_str!("seed/sidebar-archive-es-wpcategorieswidget-12.json");
    pub const SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_13: &str = include_str!("seed/sidebar-archive-es-wpcategorieswidget-13.json");
    pub const SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_14: &str = include_str!("seed/sidebar-archive-es-wpcategorieswidget-14.json");
}

const SEED_WIDGETS: &[Seed] = &[
    Seed { sidebar: "sidebar-post-fr", id: "search-4", typ: "search", title: "",
        settings: seed::SIDEBAR_POST_FR_SEARCH_4 },
    Seed { sidebar: "sidebar-post-fr", id: "wpcategorieswidget-9", typ: "categories", title: "Catégories",
        settings: seed::SIDEBAR_POST_FR_WPCATEGORIESWIDGET_9 },
    Seed { sidebar: "sidebar-post-fr", id: "media_image-4", typ: "media_image", title: "",
        settings: seed::SIDEBAR_POST_FR_MEDIA_IMAGE_4 },
    Seed { sidebar: "sidebar-post-fr", id: "custom_html-13", typ: "tour_list", title: "Voyages suggérés",
        settings: seed::SIDEBAR_POST_FR_CUSTOM_HTML_13 },
    Seed { sidebar: "sidebar-post-es", id: "search-5", typ: "search", title: "",
        settings: seed::SIDEBAR_POST_ES_SEARCH_5 },
    Seed { sidebar: "sidebar-post-es", id: "wpcategorieswidget-10", typ: "categories", title: "Categorías",
        settings: seed::SIDEBAR_POST_ES_WPCATEGORIESWIDGET_10 },
    Seed { sidebar: "sidebar-post-es", id: "media_image-5", typ: "media_image", title: "",
        settings: seed::SIDEBAR_POST_ES_MEDIA_IMAGE_5 },
    Seed { sidebar: "sidebar-post-es", id: "custom_html-14", typ: "tour_list", title: "Viajes sugeridos",
        settings: seed::SIDEBAR_POST_ES_CUSTOM_HTML_14 },
    Seed { sidebar: "sidebar-tour", id: "text-12", typ: "text", title: "",
        settings: seed::SIDEBAR_TOUR_TEXT_12 },
    Seed { sidebar: "sidebar-tour", id: "text-11", typ: "text", title: "",
        settings: seed::SIDEBAR_TOUR_TEXT_11 },
    Seed { sidebar: "sidebar-tour", id: "tourmaster-widget-tour-category-6", typ: "tour_category", title: "",
        settings: seed::SIDEBAR_TOUR_TOURMASTER_WIDGET_TOUR_CATEGORY_6 },
    Seed { sidebar: "footer-1-fr", id: "text-1", typ: "text", title: "",
        settings: seed::FOOTER_1_FR_TEXT_1 },
    Seed { sidebar: "footer-2-fr", id: "custom_html-3", typ: "custom_html", title: "",
        settings: seed::FOOTER_2_FR_CUSTOM_HTML_3 },
    Seed { sidebar: "footer-2-fr", id: "text-17", typ: "text", title: "",
        settings: seed::FOOTER_2_FR_TEXT_17 },
    Seed { sidebar: "footer-3-fr", id: "custom_html-5", typ: "custom_html", title: "",
        settings: seed::FOOTER_3_FR_CUSTOM_HTML_5 },
    Seed { sidebar: "footer-3-fr", id: "text-15", typ: "text", title: "",
        settings: seed::FOOTER_3_FR_TEXT_15 },
    Seed { sidebar: "footer-4-fr", id: "custom_html-6", typ: "custom_html", title: "",
        settings: seed::FOOTER_4_FR_CUSTOM_HTML_6 },
    Seed { sidebar: "footer-1-es", id: "text-20", typ: "text", title: "",
        settings: seed::FOOTER_1_ES_TEXT_20 },
    Seed { sidebar: "footer-2-es", id: "custom_html-9", typ: "custom_html", title: "",
        settings: seed::FOOTER_2_ES_CUSTOM_HTML_9 },
    Seed { sidebar: "footer-2-es", id: "text-21", typ: "text", title: "",
        settings: seed::FOOTER_2_ES_TEXT_21 },
    Seed { sidebar: "footer-3-es", id: "custom_html-10", typ: "custom_html", title: "",
        settings: seed::FOOTER_3_ES_CUSTOM_HTML_10 },
    Seed { sidebar: "footer-3-es", id: "text-22", typ: "text", title: "",
        settings: seed::FOOTER_3_ES_TEXT_22 },
    Seed { sidebar: "footer-4-es", id: "custom_html-11", typ: "custom_html", title: "",
        settings: seed::FOOTER_4_ES_CUSTOM_HTML_11 },
    Seed { sidebar: "sidebar-archive-fr", id: "wpcategorieswidget-5", typ: "categories", title: "Voyage par region",
        settings: seed::SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_5 },
    Seed { sidebar: "sidebar-archive-fr", id: "wpcategorieswidget-7", typ: "categories", title: "Voyage par profil",
        settings: seed::SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_7 },
    Seed { sidebar: "sidebar-archive-fr", id: "wpcategorieswidget-8", typ: "categories", title: "Voyage par durée",
        settings: seed::SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_8 },
    Seed { sidebar: "sidebar-archive-fr", id: "wpcategorieswidget-6", typ: "categories", title: "Voyage par styles",
        settings: seed::SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_6 },
    Seed { sidebar: "sidebar-archive-es", id: "wpcategorieswidget-11", typ: "categories", title: "Viaje por región",
        settings: seed::SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_11 },
    Seed { sidebar: "sidebar-archive-es", id: "wpcategorieswidget-12", typ: "categories", title: "Viaje por perfil",
        settings: seed::SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_12 },
    Seed { sidebar: "sidebar-archive-es", id: "wpcategorieswidget-13", typ: "categories", title: "Viaje por duración",
        settings: seed::SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_13 },
    Seed { sidebar: "sidebar-archive-es", id: "wpcategorieswidget-14", typ: "categories", title: "Viaje por estilos",
        settings: seed::SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_14 },
];

/// Phase-3 seed version; bump when SEED_WIDGETS' *shape* changes so the
/// one-time upgrade path re-runs (never re-seeds deleted rows — the
/// upgrade only converts the two frozen widgets it owns and inserts the
/// new -es sidebars/widgets with OR IGNORE).
const SEED_VERSION: &str = "3";

/// Ensure the sidebars/widgets schema exists and the seed rows are present.
/// Idempotent: creates the tables only when missing (full migration batch on
/// first touch), seeds only when BOTH tables are empty — deleting a
/// single widget row stays deleted. The phase-3 upgrade (lang-suffixed
/// sidebars, live categories/tour_list) is gated on the exact phase-2
/// settings_json: a pristine seeded row converts, an admin-edited row is
/// left untouched.
pub(crate) fn ensure(conn: &Connection) -> Result<()> {
    // One probe query covers "tables exist" + "both empty": missing tables
    // raise the table-error, we migrate, then re-probe. Steady state = 1
    // query per call (called per sidebar render — kept cheap on purpose).
    fn probe(conn: &Connection) -> std::result::Result<(i64, i64), rusqlite::Error> {
        conn.query_row(
            "SELECT (SELECT COUNT(*) FROM sidebars), (SELECT COUNT(*) FROM widgets)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
    }
    let (n_sidebars, n_widgets) = match probe(conn) {
        Ok(t) => t,
        Err(_) => {
            crate::schema::migrate_conn(conn)?;
            probe(conn)?
        }
    };
    if n_sidebars == 0 && n_widgets == 0 {
        let tx = conn.unchecked_transaction()?;
        for s in SIDEBARS {
            tx.execute(
                "INSERT INTO sidebars(id, name, lang) VALUES(?1, ?2, ?3)",
                rusqlite::params![s.id, s.name, s.lang],
            )?;
        }
        seed_widgets(&tx)?;
        tx.execute(
            "INSERT OR REPLACE INTO settings(key, value) VALUES('widgets_schema_v', ?1)",
            [SEED_VERSION],
        )?;
        tx.commit()?;
        return Ok(());
    }
    upgrade(conn)?;
    Ok(())
}

fn seed_widgets(conn: &Connection) -> Result<()> {
    let mut ords: HashMap<&'static str, i64> = HashMap::new();
    for w in SEED_WIDGETS {
        let ord = ords.entry(w.sidebar).or_insert(0);
        conn.execute(
            "INSERT OR IGNORE INTO widgets(id, sidebar_id, type, title, settings_json, ord)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![w.id, w.sidebar, w.typ, w.title, w.settings, *ord],
        )?;
        *ord += 1;
    }
    Ok(())
}

/// Phase-2 → phase-3 upgrade (idempotent, settings-gated):
///   * rename sidebar-post/-archive rows to the -fr forms and move their
///     widgets along,
///   * insert the new -es sidebars + seeds (INSERT OR IGNORE — an
///     already-present row is never touched),
///   * convert the two pristine frozen widgets to live renderers (exact
///     settings_json match ⇒ admin edits are never clobbered),
///   * backfill any missing const sidebar rows (e.g. a db seeded by an
///     older binary).
fn upgrade(conn: &Connection) -> Result<()> {
    let done: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key='widgets_schema_v'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    if done == SEED_VERSION {
        return Ok(());
    }
    let tx = conn.unchecked_transaction()?;
    for (old, new) in [
        ("sidebar-post", "sidebar-post-fr"),
        ("sidebar-archive", "sidebar-archive-fr"),
    ] {
        tx.execute(
            "UPDATE sidebars SET id=?2 WHERE id=?1 AND NOT EXISTS(SELECT 1 FROM sidebars WHERE id=?2)",
            rusqlite::params![old, new],
        )?;
        tx.execute(
            "UPDATE widgets SET sidebar_id=?2 WHERE sidebar_id=?1",
            rusqlite::params![old, new],
        )?;
    }
    for s in SIDEBARS {
        tx.execute(
            "INSERT OR IGNORE INTO sidebars(id, name, lang) VALUES(?1, ?2, ?3)",
            rusqlite::params![s.id, s.name, s.lang],
        )?;
    }
    // ES widget seeds — INSERT OR IGNORE keeps any rows an admin already
    // created under these ids.
    seed_widgets(&tx)?;
    // Convert the two frozen blog widgets to live renderers, but only when
    // the stored payload is still byte-identical to the phase-2 seed (an
    // admin-edited fragment stays frozen — never silently overwritten).
    tx.execute(
        "UPDATE widgets SET type='categories', title='Catégories', settings_json=?2
         WHERE id='wpcategorieswidget-9' AND settings_json=?1",
        rusqlite::params![LEGACY_CAT9, r#"{"taxonomy": "category", "title": "Catégories", "show_counts": true, "order": ["guide-de-voyage-vietnam", "le-vietnam", "sites-a-explorer"], "children": {"guide-de-voyage-vietnam": ["conseils-pratiques-vietnam", "idees-de-circuit", "ou-dormir", "quand-partir-au-vietnam", "tout-sur-le-visa-vietnam", "voyage-en-asie-du-sud-est"], "idees-de-circuit": ["circuit-au-vietnam", "croisiere", "randonnee-au-vietnam", "top-choses-a-faire-a-voir", "vietnam-en-famille", "voyage-de-luxe-vietnam", "voyage-en-couple-de-noce-vietnam"], "le-vietnam": ["actualites-du-tourisme", "cuisine-vietnamienne", "culture-histoire"], "cuisine-vietnamienne": ["boisson-vietnamienne", "plats-vietnamiens", "recettes-faciles"], "plats-vietnamiens": ["adresses-recommandees"], "sites-a-explorer": ["montagnes-du-nord", "station-balneaire-vietnam", "vietnam-du-centre", "vietnam-du-nord", "vietnam-du-sud", "voyages-au-cambodge"], "montagnes-du-nord": ["bac-ha", "ha-giang", "mai-chau", "mu-cang-chai", "sapa"], "station-balneaire-vietnam": ["ile-de-con-dao", "nha-trang", "phan-thiet", "phu-quoc", "vung-tau"], "vietnam-du-centre": ["da-nang", "hauts-plateaux-vietnam", "hoian", "hue", "quang-binh"], "vietnam-du-nord": ["baie-dhalong", "dien-bien-phu", "hanoi", "ninh-binh", "parc-national-de-cat-ba"], "vietnam-du-sud": ["can-tho", "chau-doc", "delta-du-mekong", "dong-nai", "saigon-ho-chi-minh"], "voyages-au-cambodge": ["angkor-siem-reap", "battambang", "kampot", "phnom-penh"]}}"#],
    )?;
    tx.execute(
        "UPDATE widgets SET type='tour_list', title='Voyages suggérés', settings_json=?2
         WHERE id='custom_html-13' AND settings_json=?1",
        rusqlite::params![LEGACY_CH13, r#"{"mode": "manual", "manual_ids": [15585, 6416, 6598, 6797, 6607, 6603, 6775, 6624, 6618, 7363, 9280, 7370, 7364, 7007, 6853, 9253, 9285], "count": 17, "title": "Voyages suggérés", "wrapper": "custom_html-13", "inner": "tourmaster-tour-widget-2"}"#],
    )?;
    tx.execute(
        "INSERT OR REPLACE INTO settings(key, value) VALUES('widgets_schema_v', ?1)",
        [SEED_VERSION],
    )?;
    tx.commit()?;
    Ok(())
}

/// Phase-2 frozen settings_json payloads (verbatim). The upgrade UPDATEs
/// guard on these so admin-modified fragments are never rewritten.
const LEGACY_CAT9: &str = include_str!("seed/wpcategorieswidget_9_legacy.json");

const LEGACY_CH13: &str = include_str!("seed/custom_html_13_legacy.json");

/// Rows of a sidebar in display order: (id, type, title, settings_json).
fn rows(conn: &Connection, sidebar_id: &str) -> Result<Vec<(String, String, String, String)>> {
    let mut st = conn.prepare(
        "SELECT id, type, title, settings_json FROM widgets WHERE sidebar_id=?1 ORDER BY ord",
    )?;
    let out = st
        .query_map([sidebar_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(out)
}

fn fallback_sidebar_id(conn: &Connection, sid: &str, lang: &str) -> Result<Option<String>> {
    if lang == "fr" || lang == "es" {
        return Ok(None);
    }
    let cfg = crate::lang::config(conn);
    let candidates = [
        sid.strip_suffix(&format!("-{lang}")).map(|base| format!("{base}-{}", cfg.primary)),
        sid.strip_suffix(&format!("-{lang}")).map(|base| format!("{base}-fr")),
    ];
    for candidate in candidates.into_iter().flatten() {
        if !rows(conn, &candidate)?.is_empty() {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

/// Render a frozen fragment: the stored HTML is a minijinja template whose
/// `{{ site_url }}`/`{{ lang }}` resolve against the page context. A single
/// AutoEscape::None environment — `{{ x }}` and `{{ x|safe }}` are the same.
/// The Environment is per-call: `template_from_str` binds the source's
/// lifetime to the env, and fragments are DB strings, not 'static.
/// `pub(crate)` so /admin/widgets/save can dry-run a candidate fragment
/// before writing it (bad jinja must be rejected at save, not discovered
/// mid-generate).
pub(crate) fn render_fragment(html: &str, site_url: &str, lang: &str) -> Result<String> {
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    Ok(env.template_from_str(html)?.render(context! {
        site_url => site_url,
        lang => lang,
    })?)
}

/// Render a sidebar to HTML: `before` + each widget's output + `after`.
/// Widget output: a non-empty `html` setting renders verbatim (frozen or
/// admin override); otherwise the type's live renderer runs — `categories`
/// (live taxonomy tree incl. the cat_parent-derived 'category' tree) and
/// `tour_list` (live tour cards). `current_slug` marks the current-cat on
/// rendered term lists (live WP behaviour).
pub(crate) fn render_sidebar(
    conn: &Connection,
    sidebar_id: &str,
    site_url: &str,
    lang: &str,
    current_slug: &str,
    tour_base: &str,
) -> Result<String> {
    ensure(conn)?;
    let mut sid = resolve_id(sidebar_id, lang);
    let mut widget_rows = rows(conn, &sid)?;
    if widget_rows.is_empty() {
        if let Some(fallback) = fallback_sidebar_id(conn, &sid, lang)? {
            sid = fallback;
            widget_rows = rows(conn, &sid)?;
        }
    }
    let sidebar = SIDEBARS.iter().find(|s| s.id == sid);
    let mut out = String::new();
    if let Some(s) = sidebar {
        out.push_str(s.before);
    }
    for (wid, typ, title, settings) in widget_rows {
        // Malformed settings_json must surface, not silently blank the
        // sidebar (a corrupted row is a bug to fix, not content to drop).
        let s: Value = serde_json::from_str(&settings)
            .with_context(|| format!("widget {wid} in {sid}: bad settings_json"))?;
        let frag = render_widget(conn, &wid, &typ, &title, &s, site_url, lang, current_slug, tour_base)
            .with_context(|| format!("widget {wid} in {sid}: render"))?;
        out.push_str(&frag);
    }
    if let Some(s) = sidebar {
        out.push_str(s.after);
    }
    Ok(out)
}

/// One widget's HTML: frozen `html` first, then the type's live renderer.
fn render_widget(
    conn: &Connection,
    wid: &str,
    typ: &str,
    title: &str,
    s: &Value,
    site_url: &str,
    lang: &str,
    current_slug: &str,
    tour_base: &str,
) -> Result<String> {
    if let Some(html) = s.get("html").and_then(|h| h.as_str()) {
        return render_fragment(html, site_url, lang);
    }
    match typ {
        "categories" => render_categories_widget(conn, wid, s, title, site_url, lang, current_slug),
        "tour_list" => render_tour_list(conn, wid, s, title, site_url, lang, tour_base),
        _ => Ok(String::new()),
    }
}

// ---------------------------------------------------------------------------
// categories widget — live taxonomy list
// ---------------------------------------------------------------------------

/// The capture's display transform for term names (WP esc_html +
/// texturize as printed by wpcategorieswidget): `&<>` escape, ` - ` →
/// ` &#8211; `, `'` → `&#039;`.
fn cat_display_name(raw: &str) -> String {
    crate::render_archive::esc(raw)
        .replace(" - ", " &#8211; ")
        .replace('\'', "&#039;")
}

/// A term node in a rendered category tree.
struct CatNode {
    slug: String,
    name: String,
    id: i64,
    count: i64,
    children: Vec<CatNode>,
}

/// Per-slug published post counts for a taxonomy, filtered to the
/// sidebar's language (Polylang filters WP_Query by lang, so the FR
/// sidebar counts FR posts only). Posts are dual-tagged parent+child so
/// direct counts match WP display.
fn term_counts(conn: &Connection, taxonomy: &str, lang: &str) -> Result<HashMap<String, i64>> {
    let mut st = conn.prepare(crate::sql!("widgets__term_counts"))?;
    let out = st
        .query_map(rusqlite::params![taxonomy, lang], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<std::result::Result<HashMap<_, _>, _>>()?;
    Ok(out)
}

fn term_names(conn: &Connection, taxonomy: &str) -> Result<HashMap<String, (i64, String)>> {
    let mut st = conn.prepare(crate::sql!("archive__select_terms_tax"))?;
    let out = st
        .query_map([taxonomy], |r| {
            Ok((
                r.get::<_, String>(1)?,
                (r.get::<_, i64>(0)?, r.get::<_, String>(2)?),
            ))
        })?
        .collect::<std::result::Result<HashMap<_, _>, _>>()?;
    Ok(out)
}

/// Build the `category` tree for `lang`. Edge source: the widget's own
/// `children` map when present (the live capture deviates from cat_parent
/// in one place — recettes-faciles under cuisine-vietnamienne, not
/// plats-vietnamiens — so the capture wins per-WP-registered-widget); else
/// the cat_parent map (the hierarchy of record — terms.parent_id is NULL
/// on every row). Roots are `order` when given, else the lang roots,
/// name-ordered (WP wp_list_categories orderby=name); children same
/// ordering. A term renders only when it is registered in `terms` AND
/// (has published posts or has included descendants) — wp hide_empty.
fn category_tree(
    names: &HashMap<String, (i64, String)>,
    counts: &HashMap<String, i64>,
    es: bool,
    depth: i64,
    order: &[&str],
    children_map: &Value,
) -> Vec<CatNode> {
    let default_roots: &[&str] = if es {
        &["guia-de-viaje-a-vietnam", "sitios-para-explorar", "viaje-a-vietnam"]
    } else {
        &["guide-de-voyage-vietnam", "le-vietnam", "sites-a-explorer"]
    };
    let roots: Vec<&str> = if order.is_empty() {
        default_roots.to_vec()
    } else {
        order.to_vec()
    };
    let map_edges = children_map.as_object().is_some_and(|m| !m.is_empty());
    let kids_of = |slug: &str| -> Vec<String> {
        if map_edges {
            children_map
                .get(slug)
                .and_then(|c| c.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default()
        } else {
            crate::render_post::cat_children(slug)
                .iter()
                .map(|s| s.to_string())
                .collect()
        }
    };
    let display = |slug: &str| -> Option<String> {
        names.get(slug).map(|(_, n)| n.clone())
    };
    fn build(
        slug: &str,
        depth_left: i64,
        names: &HashMap<String, (i64, String)>,
        counts: &HashMap<String, i64>,
        display: &dyn Fn(&str) -> Option<String>,
        kids_of: &dyn Fn(&str) -> Vec<String>,
    ) -> Option<CatNode> {
        let name = display(slug)?;
        let mut children = Vec::new();
        if depth_left != 0 {
            // Keep only registered terms, sort by name (WP orderby=name).
            let mut kids: Vec<(String, String)> = kids_of(slug)
                .into_iter()
                .filter_map(|c| display(&c).map(|n| (c, n)))
                .collect();
            kids.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
            for (cslug, _) in kids {
                if let Some(n) = build(&cslug, depth_left - 1, names, counts, display, kids_of) {
                    children.push(n);
                }
            }
        }
        let count = counts.get(slug).copied().unwrap_or(0);
        // hide_empty: registered-but-empty leaf terms are skipped.
        if count == 0 && children.is_empty() {
            return None;
        }
        Some(CatNode {
            slug: slug.to_string(),
            name,
            id: crate::render_archive::wp_term_id("category", slug)
                .or_else(|| names.get(slug).map(|(id, _)| *id))
                .unwrap_or(0),
            count,
            children,
        })
    }
    let mut out: Vec<CatNode> = roots
        .iter()
        .filter_map(|r| build(r, depth, names, counts, &display, &kids_of))
        .collect();
    if order.is_empty() {
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }
    out
}

/// Settings-driven tree for non-'category' taxonomies (the archive
/// sidebars' {order, children} shape): ordered slugs first, leftovers
/// alphabetically, children from the widget's frozen map.
fn settings_tree(
    names: &mut HashMap<String, (i64, String)>,
    counts: &HashMap<String, i64>,
    taxonomy: &str,
    order: &[&str],
    children_map: &Value,
    depth: i64,
) -> Vec<CatNode> {
    fn pop(
        slug: &str,
        names: &mut HashMap<String, (i64, String)>,
        counts: &HashMap<String, i64>,
        taxonomy: &str,
        children_map: &Value,
        depth_left: i64,
    ) -> Option<CatNode> {
        let (local, name) = names.remove(slug)?;
        let mut children = Vec::new();
        if depth_left != 0 {
            if let Some(kids_list) = children_map.get(slug).and_then(|c| c.as_array()) {
                for c in kids_list.iter().filter_map(|c| c.as_str()) {
                    if let Some(n) = pop(c, names, counts, taxonomy, children_map, depth_left - 1) {
                        children.push(n);
                    }
                }
            }
        }
        Some(CatNode {
            slug: slug.to_string(),
            name,
            id: crate::render_archive::wp_term_id(taxonomy, slug).unwrap_or(local),
            count: counts.get(slug).copied().unwrap_or(0),
            children,
        })
    }
    let mut out = Vec::new();
    for slug in order {
        if let Some(n) = pop(slug, names, counts, taxonomy, children_map, depth) {
            out.push(n);
        }
    }
    let mut rest: Vec<String> = names.keys().cloned().collect();
    rest.sort();
    for slug in rest {
        if let Some(n) = pop(&slug, names, counts, taxonomy, children_map, depth) {
            out.push(n);
        }
    }
    out
}

/// Live `categories` widget → the wpcategorieswidget markup. `taxonomy`
/// absent (defensive; required by the edit form) renders nothing.
fn render_categories_widget(
    conn: &Connection,
    wid: &str,
    s: &Value,
    row_title: &str,
    site_url: &str,
    lang: &str,
    current_slug: &str,
) -> Result<String> {
    let Some(taxonomy) = s.get("taxonomy").and_then(|t| t.as_str()) else {
        return Ok(String::new());
    };
    let title = s
        .get("title")
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .unwrap_or(row_title);
    let show_counts = s
        .get("show_counts")
        .map(|v| v.as_bool().unwrap_or(false) || v.as_str() == Some("1"))
        .unwrap_or(false);
    // depth: absent/0/<=0 = unlimited (WP depth=0); N caps recursion.
    let depth = s
        .get("depth")
        .and_then(|d| d.as_i64().or_else(|| d.as_str().and_then(|x| x.parse().ok())))
        .filter(|d| *d > 0)
        .unwrap_or(-1);
    let order: Vec<&str> = s
        .get("order")
        .and_then(|o| o.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let children_map = s.get("children").cloned().unwrap_or(json!({}));
    let es = lang == "es";
    let prefix = if es { "/es" } else { "" };

    let counts = term_counts(conn, taxonomy, lang)?;
    let mut names = term_names(conn, taxonomy)?;
    let tree = if taxonomy == "category" {
        category_tree(&names, &counts, es, depth, &order, &children_map)
    } else {
        settings_tree(&mut names, &counts, taxonomy, &order, &children_map, depth)
    };
    if tree.is_empty() {
        return Ok(String::new());
    }

    let mut items = String::new();
    for n in &tree {
        // The node slug slot carries the full link: for 'category' the
        // hierarchical cat_path, for tour taxonomies /{tax}/{slug}/.
        let href = if taxonomy == "category" {
            format!("{site_url}{prefix}/{}/", crate::render_post::cat_path(&n.slug))
        } else {
            format!("{site_url}{prefix}/{}/{}/", taxonomy.to_lowercase(), n.slug)
        };
        emit_top(&mut items, n, &href, taxonomy, show_counts, current_slug, site_url, prefix);
    }

    Ok(format!(
        "<div id=\"{wid}\" class=\"widget widget_wpcategorieswidget traveltour-widget\"><ul id=\"{wid}\"><li class=\"{tax}\"><h3 class=\"widget-title\">{title}</h3><ul>{items}</ul></li></ul></div>",
        tax = if taxonomy == "category" { "categories" } else { taxonomy },
        title = crate::render_archive::esc(title),
    ))
}

/// Top-level node emission — children resolve their own hrefs by walking
/// the same rule (cat_path for 'category', tax path otherwise).
fn emit_top(
    out: &mut String,
    n: &CatNode,
    href: &str,
    taxonomy: &str,
    show_counts: bool,
    current_slug: &str,
    site_url: &str,
    prefix: &str,
) {
    out.push_str(&format!(
        "\t<li class=\"cat-item cat-item-{}{}\"><a{} href=\"{}\">{}</a>",
        n.id,
        if n.slug == current_slug { " current-cat" } else { "" },
        if n.slug == current_slug { " aria-current=\"page\"" } else { "" },
        href,
        cat_display_name(&n.name),
    ));
    if show_counts {
        out.push_str(&format!(" <span class=\"post-count\">{}</span>", n.count));
    }
    if n.children.is_empty() {
        out.push_str("\n</li>\n");
    } else {
        out.push('\n');
        emit_tree(out, n, 1, taxonomy, show_counts, current_slug, site_url, prefix);
        out.push_str("</li>\n");
    }
}

fn emit_tree(
    out: &mut String,
    parent: &CatNode,
    li_depth: usize,
    taxonomy: &str,
    show_counts: bool,
    current_slug: &str,
    site_url: &str,
    prefix: &str,
) {
    let ul_indent = "\t".repeat(li_depth.saturating_sub(1));
    out.push_str(&format!("{ul_indent}<ul class=\"children\">\n"));
    for c in &parent.children {
        let href = if taxonomy == "category" {
            format!("{site_url}{prefix}/{}/", crate::render_post::cat_path(&c.slug))
        } else {
            format!("{site_url}{prefix}/{}/{}/", taxonomy.to_lowercase(), c.slug)
        };
        out.push_str(&format!(
            "\t<li class=\"cat-item cat-item-{}{}\"><a{} href=\"{}\">{}</a>",
            c.id,
            if c.slug == current_slug { " current-cat" } else { "" },
            if c.slug == current_slug { " aria-current=\"page\"" } else { "" },
            href,
            cat_display_name(&c.name),
        ));
        if show_counts {
            out.push_str(&format!(" <span class=\"post-count\">{}</span>", c.count));
        }
        if c.children.is_empty() {
            out.push_str("\n</li>\n");
        } else {
            out.push('\n');
            emit_tree(out, c, li_depth + 1, taxonomy, show_counts, current_slug, site_url, prefix);
            out.push_str("</li>\n");
        }
    }
    out.push_str(&format!("{ul_indent}</ul>\n"));
}

// ---------------------------------------------------------------------------
// tour_list widget — live tour cards
// ---------------------------------------------------------------------------

/// `post_meta` single value (meta1/opt_s equivalent of the archive
/// renderer's helper).
fn meta1(conn: &Connection, post_id: i64, key: &str) -> String {
    conn.query_row(
        crate::sql!("archive__select_meta_value"),
        rusqlite::params![post_id, key],
        |r| r.get::<_, String>(0),
    )
    .unwrap_or_default()
}

/// One tourmaster recent-tour card — the markup captured inside
/// custom_html-13: thumbnail (sizes_json 'thumbnail' → 96×96, else the
/// full image at 128×85), title, ribbon (promo-text or style term names
/// with the tourmaster colours) and the De/Desde price block.
fn tour_card(conn: &Connection, post_id: i64, site_url: &str, lang: &str, tour_base: &str) -> Result<Option<String>> {
    let row: Option<(String, String, Option<i64>)> = conn
        .query_row(
            "SELECT slug, title, featured_media_id FROM posts WHERE id=?1 AND status='publish'",
            [post_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok();
    let Some((slug, title, fid)) = row else {
        // WP drops non-published ids from the query silently.
        return Ok(None);
    };
    let opt = crate::php_serialize::unserialize(meta1(conn, post_id, "tourmaster-tour-option").as_bytes())
        .unwrap_or(Value::Null);
    let os = |k: &str| -> String {
        opt.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string()
    };
    let mut promo_text = os("promo-text");
    if promo_text.is_empty() {
        // same fallback as the archive cards: the tour's style term names
        let style_tax = if lang == "es" { "estilo" } else { "style" };
        let names: Vec<String> = conn
            .prepare(crate::sql!("archive__select_post_term_names"))?
            .query_map(rusqlite::params![post_id, style_tax], |r| r.get::<_, String>(0))?
            .flatten()
            .collect();
        promo_text = names.join(", ");
    }
    // display price: tour-price-text → tourmaster-tour-price → price_range
    let raw_price = {
        let t = os("tour-price-text");
        if !t.is_empty() {
            t
        } else {
            let m = meta1(conn, post_id, "tourmaster-tour-price");
            if !m.is_empty() {
                m
            } else {
                conn.query_row::<String, _, _>(
                    "SELECT COALESCE(price_range,'') FROM tours WHERE post_id=?1",
                    [post_id],
                    |r| r.get(0),
                )
                .unwrap_or_default()
            }
        }
    };
    let display = crate::render_tour::display_slug(conn, post_id, &slug, &title);
    let prefix = if lang == "es" { "/es" } else { "" };
    let url = format!("{site_url}{prefix}/{tour_base}/{display}/");

    // Thumbnail: media sizes_json 'thumbnail' → the 150x150 file rendered
    // at 96×96; otherwise the full image at 128×85 (the two shapes the
    // capture carries).
    let (img_path, iw, ih, alt) = match fid.and_then(|f| {
        conn.query_row(
            "SELECT path, COALESCE(alt,''), sizes_json FROM media WHERE id=?1",
            [f],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)),
        )
        .ok()
    }) {
        Some((path, alt, sizes)) => {
            let thumb = serde_json::from_str::<Value>(&sizes)
                .ok()
                .and_then(|sj| {
                    sj.get("thumbnail")
                        .and_then(|t| t.get("file"))
                        .and_then(|f| f.as_str())
                        .map(|f| f.to_string())
                });
            match thumb {
                Some(file) => {
                    let dir = path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
                    (format!("{dir}/{file}"), 96, 96, alt)
                }
                None => (path, 128, 85, alt),
            }
        }
        None => (String::new(), 0, 0, String::new()),
    };
    let img = if img_path.is_empty() {
        String::new()
    } else {
        format!(
            "<img src=\"{site_url}/wp-content/uploads/{img_path}\" alt=\"{}\" width=\"{iw}\" height=\"{ih}\">",
            crate::render_archive::esc_attr_pub(&alt),
        )
    };

    let promo_bg = os("promo-text-ribbon-background");
    let promo_color = os("promo-text-ribbon-text-color");
    let ribbon = if promo_text.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"tourmaster-thumbnail-ribbon gdlr-core-outer-frame-element\" style=\"color: {};background-color: {};\"><div class=\"tourmaster-thumbnail-ribbon-cornor\" style=\"border-right-color: {};\"></div>{}</div>",
            if promo_color.is_empty() { "#ffffff".to_string() } else { crate::render_archive::esc_attr_pub(&promo_color) },
            crate::render_archive::esc_attr_pub(&promo_bg),
            crate::render_archive::rgba_pub(&promo_bg, 0.5),
            crate::render_archive::esc(&promo_text),
        )
    };
    let price = crate::render_archive::fmt_price_pub(&raw_price);
    let price_html = if price.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"tourmaster-tour-price-wrap \"><span class=\"tourmaster-tour-price\"><span class=\"tourmaster-head\">{}</span><span class=\"tourmaster-tail\">{}</span></span></div>",
            if lang == "es" { "Desde" } else { "De" },
            price,
        )
    };
    let info_class = if ribbon.is_empty() {
        "tourmaster-tour-content-info clearfix"
    } else {
        "tourmaster-tour-content-info clearfix tourmaster-with-ribbon"
    };
    Ok(Some(format!(
        "<div class=\"tourmaster-item-list tourmaster-tour-widget tourmaster-item-pdlr\"><div class=\"tourmaster-tour-widget-inner clearfix\"><div class=\"tourmaster-tour-thumbnail tourmaster-media-image \"><a href=\"{url}\">{img}</a></div><div class=\"tourmaster-tour-content-wrap\"><h3 class=\"tourmaster-tour-title gdlr-core-skin-title\"><a href=\"{url}\">{}</a></h3><div class=\"{info_class}\">{ribbon}{price_html}</div></div></div></div>",
        crate::render_archive::wp_title_pub(&title.replace('<', "&lt;").replace('>', "&gt;")),
    )))
}

/// Live `tour_list` widget: the custom_html-13 wrapper shape (outer
/// custom-html widget div + tourmaster-tour-widget inner) around live
/// cards. `wrapper`/`inner` settings override the emitted ids so seeded
/// rows keep their capture ids.
fn render_tour_list(
    conn: &Connection,
    wid: &str,
    s: &Value,
    row_title: &str,
    site_url: &str,
    lang: &str,
    tour_base: &str,
) -> Result<String> {
    let title = s
        .get("title")
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .unwrap_or(row_title);
    let count: i64 = s
        .get("count")
        .and_then(|c| c.as_i64().or_else(|| c.as_str().and_then(|x| x.parse().ok())))
        .unwrap_or(10);
    let mode = s.get("mode").and_then(|m| m.as_str()).unwrap_or("recent");
    let ids: Vec<i64> = if mode == "manual" {
        s.get("manual_ids")
            .and_then(|a| a.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_i64().or_else(|| v.as_str().and_then(|x| x.parse().ok())))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        let mut st = conn.prepare(crate::sql!("widgets__recent_tours"))?;
        let v: Vec<i64> = st
            .query_map(rusqlite::params![lang, count], |r| r.get::<_, i64>(0))?
            .flatten()
            .collect();
        v
    };
    let mut cards = String::new();
    for id in ids {
        if let Some(card) = tour_card(conn, id, site_url, lang, tour_base)? {
            cards.push_str(&card);
        }
    }
    let heading = if title.is_empty() {
        String::new()
    } else {
        format!(
            "<h2 class=\"traveltour-widget-title\"><span class=\"traveltour-widget-head-text\">{}</span></h2><span class=\"clear\"></span>",
            crate::render_archive::esc(title),
        )
    };
    let wrapper = s
        .get("wrapper")
        .and_then(|w| w.as_str())
        .filter(|w| !w.is_empty())
        .unwrap_or(wid);
    let inner = s
        .get("inner")
        .and_then(|w| w.as_str())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .unwrap_or_else(|| {
            // tourmaster-tour-widget-N: the instance number mirrors the
            // widget id suffix when it has one.
            let n: i64 = wid
                .rsplit('-')
                .next()
                .and_then(|x| x.parse().ok())
                .unwrap_or(2);
            format!("tourmaster-tour-widget-{n}")
        });
    Ok(format!(
        "<div id=\"{wrapper}\" class=\"widget_text widget widget_custom_html traveltour-widget\"><div class=\"textwidget custom-html-widget\"><div id=\"{inner}\" class=\"widget widget_tourmaster-tour-widget traveltour-widget\">{heading}<div class=\"tourmaster-recent-tour-widget tourmaster-tour-item\">{cards}</div></div></div></div>",
        wrapper = crate::render_archive::esc_attr_pub(wrapper),
        inner = crate::render_archive::esc_attr_pub(&inner),
    ))
}

/// Structured widget values for sidebars rendered by a template loop
/// (sidebar-archive-* → `{% for w in widgets %}` in archive.html).
/// `categories` widgets with a taxonomy emit the `{id, taxonomy, title,
/// terms:[…], prefix}` shape the loop consumes; every other widget
/// (frozen html, or a dynamic type that landed on an archive sidebar)
/// emits `{id, html}` — rendered once, same markup the render_sidebar
/// path produces. `terms` resolve live from `terms`, `cat-item-N` keeps
/// wp ids via `render_archive::wp_term_id`, children come from each
/// widget's `children` map (or cat_parent for taxonomy='category').
pub(crate) fn sidebar_widgets(
    conn: &Connection,
    sidebar_id: &str,
    site_url: &str,
    lang: &str,
    tour_base: &str,
) -> Result<Vec<Value>> {
    ensure(conn)?;
    let sid = resolve_id(sidebar_id, lang);
    let es = lang == "es";
    let prefix = if es { "/es" } else { "" };
    let mut out = Vec::new();
    let mut stmt = conn.prepare(crate::sql!("archive__select_terms_tax"))?;
    for (wid, typ, title, settings) in rows(conn, &sid)? {
        // Same policy as render_sidebar: corrupt settings_json propagates
        // (with the widget id) instead of silently emptying the block.
        let s: Value = serde_json::from_str(&settings)
            .with_context(|| format!("widget {wid} in {sid}: bad settings_json"))?;
        let is_live_categories = typ == "categories"
            && s.get("taxonomy").and_then(|t| t.as_str()).is_some()
            && s.get("html").and_then(|h| h.as_str()).is_none();
        if !is_live_categories {
            let html = render_widget(conn, &wid, &typ, &title, &s, site_url, lang, "", tour_base)
                .with_context(|| format!("widget {wid} in {sid}: render"))?;
            if !html.is_empty() {
                out.push(json!({"id": wid, "html": html}));
            }
            continue;
        }
        let taxonomy = s.get("taxonomy").and_then(|t| t.as_str()).unwrap();
        let title_s = s
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or(title.as_str());
        let order: Vec<&str> = s
            .get("order")
            .and_then(|o| o.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let children = s.get("children").cloned().unwrap_or(json!({}));

        let rows = stmt
            .query_map([taxonomy], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut by_slug: HashMap<String, (i64, String)> = rows
            .iter()
            .map(|(id, s2, n)| (s2.clone(), (*id, n.clone())))
            .collect();
        let pop = |slug: &str, by_slug: &mut HashMap<String, (i64, String)>| -> Option<Value> {
            by_slug.remove(slug).map(|(local, name)| {
                let wp = crate::render_archive::wp_term_id(taxonomy, slug).unwrap_or(local);
                json!({"id": wp, "slug": slug, "name": crate::render_archive::esc(&name), "children": []})
            })
        };
        let mut terms: Vec<Value> = Vec::new();
        for slug in &order {
            if let Some(mut t) = pop(slug, &mut by_slug) {
                if let Some(kids_list) = children.get(*slug).and_then(|c| c.as_array()) {
                    let kids: Vec<Value> = kids_list
                        .iter()
                        .filter_map(|c| c.as_str())
                        .filter_map(|c| pop(c, &mut by_slug))
                        .collect();
                    t.as_object_mut()
                        .unwrap()
                        .insert("children".into(), json!(kids));
                }
                terms.push(t);
            }
        }
        let mut rest: Vec<String> = by_slug.keys().cloned().collect();
        rest.sort();
        for slug in rest {
            if let Some(t) = pop(&slug, &mut by_slug) {
                terms.push(t);
            }
        }
        if !terms.is_empty() {
            out.push(json!({"id": wid, "taxonomy": taxonomy, "title": title_s, "terms": terms, "prefix": prefix}));
        }
    }
    Ok(out)
}
