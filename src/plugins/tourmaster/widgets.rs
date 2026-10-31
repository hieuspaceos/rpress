//! Tourmaster-owned widget types, sidebars, seeds, and live renderers —
//! extracted from `src/widgets.rs` (phase-03 seam). Byte-identical markup:
//! the `tour_list`/`tour_category` widget types, the `sidebar-tour` booking
//! sidebar, its three seed payloads, the recent-tour card renderer, and
//! the `widgets__recent_tours` query all live behind the adapter hooks
//! (`widget_types`/`widget_fields`/`sidebars`/`seeds`) and the
//! `plugin_render_for` dispatch below.

use anyhow::Result;
use rusqlite::Connection;
use serde_json::Value;

use crate::widgets::{f, FieldSpec, Seed, Sidebar, WidgetRender, HTML_FIELD, HTML_HELP};

/// Adapter-registered widget types (merged into `widgets::WIDGET_TYPES`).
pub(crate) const WIDGET_TYPES: &[(&'static str, &'static str)] = &[
    ("tour_list", "Tour List"),
    ("tour_category", "Tour Category"),
];

/// `sidebar-tour`: the booking-bar region rendered around a tour page.
/// Its wrapper markup is the verbatim capture the old `SIDEBARS` const
/// carried in core.
pub(crate) const SIDEBARS: &[Sidebar] = &[Sidebar {
    id: "sidebar-tour",
    name: "Tour booking sidebar",
    lang: "all",
    before: r#"<div class="tourmaster-tour-booking-bar-widget  traveltour-sidebar-area" >"#, // gate-allow: verbatim WP widget wrapper
    after: r#"</div>"#,
}];

/// Admin field schemas per widget type (`PluginAdapter::widget_fields`).
/// `tour_category` is a frozen-HTML type like `text`/`custom_html`.
pub(crate) fn widget_fields(typ: &str) -> Option<&'static [FieldSpec]> {
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
        "tour_category" => Some(HTML_FIELD),
        "tour_list" => Some(TOUR_LIST),
        _ => None,
    }
}

mod seed {
    pub const SIDEBAR_TOUR_TEXT_12: &str =
        include_str!("seed/sidebar-tour-text-12.json");
    pub const SIDEBAR_TOUR_TEXT_11: &str =
        include_str!("seed/sidebar-tour-text-11.json");
    pub const SIDEBAR_TOUR_TOURMASTER_WIDGET_TOUR_CATEGORY_6: &str =
        include_str!("seed/sidebar-tour-tourmaster-widget-tour-category-6.json");
    /// The blog sidebars' live `tour_list` seeds (the frozen captures the
    /// phase-3 upgrade converted to live renderers) — plugin-owned widgets
    /// seeding theme-owned sidebars. Payloads moved with the plugin.
    pub const SIDEBAR_POST_FR_CUSTOM_HTML_13: &str =
        include_str!("seed/sidebar-post-fr-custom_html-13.json");
    pub const SIDEBAR_POST_ES_CUSTOM_HTML_14: &str =
        include_str!("seed/sidebar-post-es-custom_html-14.json");
}

/// Seed rows this adapter ships (`PluginAdapter::seeds`): the sidebar-tour
/// instances plus the two `tour_list` widgets that live on the blog
/// sidebars. Order preserves the pre-extraction `SEED_WIDGETS` layout —
/// sidebar-tour rows keep ord 0,1,2 within their sidebar.
pub(crate) const SEEDS: &[Seed] = &[
    Seed { sidebar: "sidebar-post-fr", id: "custom_html-13", typ: "tour_list", title: "Voyages suggérés",
        settings: seed::SIDEBAR_POST_FR_CUSTOM_HTML_13 },
    Seed { sidebar: "sidebar-post-es", id: "custom_html-14", typ: "tour_list", title: "Viajes sugeridos",
        settings: seed::SIDEBAR_POST_ES_CUSTOM_HTML_14 },
    Seed { sidebar: "sidebar-tour", id: "text-12", typ: "text", title: "",
        settings: seed::SIDEBAR_TOUR_TEXT_12 },
    Seed { sidebar: "sidebar-tour", id: "text-11", typ: "text", title: "",
        settings: seed::SIDEBAR_TOUR_TEXT_11 },
    Seed { sidebar: "sidebar-tour", id: "tourmaster-widget-tour-category-6", typ: "tour_category", title: "",
        settings: seed::SIDEBAR_TOUR_TOURMASTER_WIDGET_TOUR_CATEGORY_6 },
];

/// Live-renderer dispatch for adapter-owned widget types — called from
/// `widgets::plugin_widget_render` once the registry has confirmed this
/// adapter declared `typ`. `None` → the type renders empty.
pub(crate) fn plugin_render_for(typ: &str) -> Option<WidgetRender> {
    match typ {
        "tour_list" => Some(render_tour_list),
        _ => None,
    }
}

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
fn tour_card(conn: &Connection, post_id: i64, site_url: &str, lang: &str, kind_base: &str) -> Result<Option<String>> { // gate-allow: sync — every caller reaches this on a blocking thread (spawn_blocking / build / boot)
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
        let theme = crate::plugins::theme_for_site(conn);
        let primary = crate::lang::config(conn).primary;
        let style_tax = crate::plugins::localized_taxonomy(theme, "style", lang, &primary);
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
    let display = super::render::display_slug(conn, post_id, &slug, &title);
    let prefix = crate::lang::url_prefix(&crate::lang::config(conn), lang);
    let url = format!("{site_url}{prefix}/{kind_base}/{display}/");

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
            crate::html_decode::esc_attr(&alt),
        )
    };

    let promo_bg = os("promo-text-ribbon-background");
    let promo_color = os("promo-text-ribbon-text-color");
    let ribbon = if promo_text.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"tourmaster-thumbnail-ribbon gdlr-core-outer-frame-element\" style=\"color: {};background-color: {};\"><div class=\"tourmaster-thumbnail-ribbon-cornor\" style=\"border-right-color: {};\"></div>{}</div>",
            if promo_color.is_empty() { "#ffffff".to_string() } else { crate::html_decode::esc_attr(&promo_color) },
            crate::html_decode::esc_attr(&promo_bg),
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
            crate::lang::t(lang, "from"),
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
        crate::render_archive::wp_title_pub(&crate::admin::esc(&title)),
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
    kind_base: &str,
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
        if let Some(card) = tour_card(conn, id, site_url, lang, kind_base)? {
            cards.push_str(&card);
        }
    }
    let heading = if title.is_empty() {
        String::new()
    } else {
        format!(
            "<h2 class=\"traveltour-widget-title\"><span class=\"traveltour-widget-head-text\">{}</span></h2><span class=\"clear\"></span>", // gate-allow: verbatim WP widget heading
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
        "<div id=\"{wrapper}\" class=\"widget_text widget widget_custom_html traveltour-widget\"><div class=\"textwidget custom-html-widget\"><div id=\"{inner}\" class=\"widget widget_tourmaster-tour-widget traveltour-widget\">{heading}<div class=\"tourmaster-recent-tour-widget tourmaster-tour-item\">{cards}</div></div></div></div>", // gate-allow: verbatim WP widget wrapper
        wrapper = crate::html_decode::esc_attr(wrapper),
        inner = crate::html_decode::esc_attr(&inner),
    ))
}
