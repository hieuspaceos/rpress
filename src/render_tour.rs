//! Single tour page renderer (kind='tour').
//!
//! Wraps the page-builder body in the tourmaster single-tour scaffold:
//! hero header, booking/enquiry bar with login forms, view counter,
//! sidebar widgets, read-more — all extracted verbatim from the live
//! traveltour theme output (templates/reference/tour.html).
//!
//! `page()` takes the body_json already rendered by `render_nodes` (the
//! orchestrator calls render_slug's machinery, then hands control here).

#[allow(unused_imports)]
use crate::sql;
use anyhow::Result;
#[allow(unused_imports)]
use minijinja::context;
use rusqlite::Connection;
use serde_json::json;
use std::path::Path;

use crate::render::{load_env, media_map};

/// Context handed in by the caller: the post row plus its already-rendered
/// page-builder body (`body_html_rendered`).
pub struct PostCtx {
    pub slug: String,
    pub lang: String,
    pub title: String,
    pub id: i64,
    pub site_url: String,
    pub body_html: String,
    pub body_html_rendered: String,
    pub kind: String,
    /// URL segment for the tour CPT (threaded from SiteEnv/CLI).
    pub tour_base: String,
}
/// Render one tour single page. `templates` is the template dir (for
/// `single_tour.html` / base); the booking sidebar comes from the
/// `sidebar-tour` widget rows.
pub(crate) fn page(conn: &Connection, templates: &Path, ctx: &PostCtx) -> Result<String> {
    let media = media_map(conn)?;

    // featured image for hero background + JSON-LD image
    let (featured_id, excerpt, seo_title, seo_desc): (Option<i64>, String, String, String) = conn
        .query_row(sql!("tour__select_post_row"), [ctx.id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .unwrap_or((None, String::new(), String::new(), String::new()));

    let featured_img = featured_id
        .and_then(|mid| media.get(&mid))
        .map(|(p, _)| format!("{}/wp-content/uploads/{}", ctx.site_url, p))
        .unwrap_or_default();

    // view counter + price. WP increments the counter before rendering, so
    // the displayed value is stored+1 (live capture: 6811 stored → 6812 shown).
    let (view_count, price_range): (i64, String) = conn
        .query_row(sql!("tour__select_tours_row"), [ctx.id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap_or((0, String::new()));
    let view_count = view_count + 1;

    let meta = |key: &str| -> String {
        conn.query_row(sql!("tour__select_meta_value"), rusqlite::params![ctx.id, key], |r| {
            r.get::<_, String>(0)
        })
        .unwrap_or_default()
    };
    let price = {
        let p = meta("tourmaster-tour-price");
        if p.is_empty() {
            price_range
        } else {
            p
        }
    };
    let description = {
        let d = meta("rank_math_description");
        if !d.is_empty() {
            d
        } else if !seo_desc.is_empty() {
            seo_desc.clone()
        } else {
            excerpt.clone()
        }
    };

    // Permalink slug: some rows carry degenerate slugs (''/'x'), WP re-derives
    // post_name from the title — replicate with accent-stripping slugify.
    let display = display_slug(conn, ctx.id, &ctx.slug, &ctx.title);
    let tour_path = format!("{}/{}/{}/", url_prefix(&ctx.lang), ctx.tour_base, display);
    let canonical = format!("{}{}", ctx.site_url, tour_path);

    let alternates = translation_urls(conn, ctx.id, &ctx.site_url, &ctx.tour_base);
    let hreflang_fr = alternates.iter().find(|(l, _)| l == "fr").map(|(_, u)| u.clone()).unwrap_or_default();
    let hreflang_es = alternates.iter().find(|(l, _)| l == "es").map(|(_, u)| u.clone()).unwrap_or_default();
    let hreflang_links = hreflang_links(&alternates);

    let jsonld = product_jsonld(&ctx.title, ctx.id, &canonical, &description, &price, &featured_img, &site_name_of(conn));

    let env = load_env(templates)?;
    let site_name = conn
        .query_row(
            "SELECT value FROM settings WHERE key='site_name' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "rpress".into());
    let seo_title = if seo_title.is_empty() {
        ctx.title.clone()
    } else {
        crate::db::resolve_seo(&seo_title, &ctx.title, &site_name)
    };
    let seo_desc_out = if seo_desc.is_empty() {
        description.clone()
    } else {
        crate::db::resolve_seo(&seo_desc, &ctx.title, &site_name)
    };

    // --- shared head/chrome vars (Phase 2: meta block, links, nav, flags) --
    let mut tctx = crate::render::chrome_vars(
        conn,
        &ctx.site_url,
        ctx.id,
        &ctx.kind,
        &ctx.slug,
        &ctx.lang,
        &ctx.title,
        &canonical,
        &seo_desc_out,
        &ctx.body_html,
        &hreflang_fr,
        &hreflang_es,
        &ctx.tour_base,
    )?;
    // seo_title/title hit <title> + meta content attributes — pre-escaped
    // at the boundary (base templates run AutoEscape::None).
    let sidebar_tour = crate::widgets::render_sidebar(conn, "sidebar-tour", &ctx.site_url, &ctx.lang, &ctx.slug, &ctx.tour_base)?;
    for (k, v) in serde_json::json!({
        "seo_title": esc_attr(&seo_title),
        "seo_desc": seo_desc_out,
        "canonical": canonical,
        "hreflang_fr": hreflang_fr,
        "hreflang_es": hreflang_es,
        "hreflang_links": hreflang_links,
        "lang": ctx.lang,
        "site_url": ctx.site_url,
        "id": ctx.id,
        "title": esc(&ctx.title),
        "featured_img": featured_img,
        "view_count": view_count,
        "tour_path": tour_path,
        "jsonld": jsonld,
        "body": ctx.body_html_rendered,
        "sidebar_tour": sidebar_tour,
    })
    .as_object()
    .into_iter()
    .flatten()
    {
        tctx.insert(k.clone(), v.clone());
    }

    let navx = match tctx.get("navx") {
        Some(serde_json::Value::Object(m)) => m.clone(),
        _ => serde_json::Map::new(),
    };
    let html = env
        .get_template("single_tour.html")?
        .render(minijinja::Value::from_serialize(serde_json::Value::Object(tctx)))
        .map_err(|e| anyhow::anyhow!("render single_tour.html: {e}"))?;
    Ok(crate::render::apply_navx(&html, &ctx.site_url, &navx))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}

fn url_prefix(lang: &str) -> String {
    if lang == "fr" || lang.is_empty() { String::new() } else { format!("/{lang}") }
}

/// Slug usable in URLs. The DB slug can be degenerate ('', 'x') after late
/// renames. Recovery: for each `_wp_old_slug` (in insertion order) look up the
/// post's own redirect row — import wrote them grouped per post, so the n-th
/// `old-slug` redirect for that slug belongs to the n-th post (by id) holding
/// that `_wp_old_slug`. Falls back to accent-stripped title slugify.
pub(crate) fn display_slug(conn: &Connection, id: i64, slug: &str, title: &str) -> String {
    if slug.len() >= 3 {
        return slug.to_string();
    }
    if let Ok(mut rows) = conn.prepare(sql!("tour__select_old_slugs")) {
        if let Ok(mapped) = rows.query_map([id], |r| r.get::<_, String>(0)) {
            for old in mapped.flatten() {
                if let Ok(new_url) = conn.query_row::<String, _, _>(
                    sql!("tour__select_redirect"),
                    rusqlite::params![id, old],
                    |r| r.get(0),
                ) {
                    let seg = new_url
                        .trim_matches('/')
                        .rsplit('/')
                        .next()
                        .unwrap_or("")
                        .to_string();
                    if !seg.is_empty() {
                        return seg;
                    }
                }
            }
        }
    }
    let s = wp_slugify(title);
    if s.is_empty() {
        slug.to_string()
    } else {
        s
    }
}

/// WP `sanitize_title`-ish: lowercase, strip accents, non-alnum → '-'.
fn wp_slugify(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'à' | 'â' | 'ä' | 'á' | 'ã' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
            'ç' | 'ć' | 'č' => 'c',
            'đ' | 'ď' => 'd',
            'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'ī' | 'ı' => 'i',
            'ñ' | 'ń' => 'n',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' | 'ō' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ư' => 'u',
            'ý' | 'ÿ' => 'y',
            'œ' => 'o',
            'æ' => 'a',
            c if c.is_ascii_alphanumeric() => c,
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn translation_urls(conn: &Connection, id: i64, site: &str, tour_base: &str) -> Vec<(String, String)> {
    let cfg = crate::lang::config(conn);
    let group: Option<String> = conn
        .query_row("SELECT group_id FROM post_translation_members WHERE post_id=?1 LIMIT 1", [id], |r| r.get(0))
        .ok();
    let Some(group) = group else { return Vec::new(); };
    let mut stmt = match conn.prepare("SELECT lang, post_id FROM post_translation_members WHERE group_id=?1 ORDER BY lang") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map([group], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|(lang, pid)| {
            slug_of(conn, pid).map(|(slug, kind)| {
                let title: String = conn.query_row("SELECT title FROM posts WHERE id=?1", [pid], |r| r.get(0)).unwrap_or_default();
                let display = if kind == "tour" { display_slug(conn, pid, &slug, &title) } else { slug };
                (lang.clone(), crate::lang::page_url(&cfg, site, &lang, &kind, &display, tour_base))
            })
        })
        .collect()
}

fn hreflang_links(alternates: &[(String, String)]) -> String {
    alternates
        .iter()
        .map(|(lang, url)| format!("<link rel=\"alternate\" href=\"{}\" hreflang=\"{}\" />", esc_attr(url), esc_attr(lang)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn slug_of(conn: &Connection, id: i64) -> Option<(String, String)> {
    conn.query_row(
        sql!("render__select_slug_kind"),
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .ok()
}

/// schema.org Product JSON-LD emitted by tourmaster. PHP json_encode style:
/// slashes escaped as \/, non-ASCII as \uXXXX.
/// settings.site_name with a generic default — single lookup shared by
/// JSON-LD brand and seo_title resolution.
fn site_name_of(conn: &Connection) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key='site_name' LIMIT 1",
        [],
        |r| r.get::<_, String>(0),
    )
    .unwrap_or_else(|_| "rpress".into())
}
fn product_jsonld(
    title: &str,
    id: i64,
    url: &str,
    description: &str,
    price: &str,
    image: &str,
    brand: &str,
) -> String {
    let valid_until = format!("{}-01-01", current_year() + 1);
    let v = json!({
        "@context": "http://schema.org",
        "@type": "Product",
        "name": title,
        "productID": format!("tour-{id}"),
        "brand": brand,
        "sku": "1",
        "url": url,
        "description": description,
        "offers": {
            "@type": "Offer",
            "url": url,
            "price": price,
            "priceValidUntil": valid_until,
            "availability": "http://schema.org/InStock",
            "priceCurrency": "USD",
        },
        "image": image,
    });
    let s = serde_json::to_string(&v).unwrap_or_else(|_| "{}".into());
    let s = s.replace('/', "\\/");
    // PHP default escapes non-ASCII as \uXXXX
    s.chars()
        .flat_map(|c| {
            if (c as u32) > 0x7e {
                let mut buf = [0u16; 2];
                c.encode_utf16(&mut buf)
                    .iter()
                    .map(|u| format!("\\u{u:04x}"))
                    .collect::<Vec<_>>()
            } else {
                vec![c.to_string()]
            }
        })
        .collect()
}

/// Current UTC year from system time (days→civil, no chrono).
fn current_year() -> i64 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    civil_year(secs / 86400)
}

/// Howard Hinnant's civil-from-days → year.
fn civil_year(z: i64) -> i64 {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    yoe + era * 400
}
