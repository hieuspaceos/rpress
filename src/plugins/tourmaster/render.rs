//! Single tour page renderer (kind='tour') — moved verbatim from
//! `src/render_tour.rs` (phase 03 plugin seam).
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
use serde_json::{json, Value};
use std::path::Path;

use crate::plugins::RenderCtx;
use crate::render::load_env_theme;

/// Render one tour single page. `templates` is the template dir (for
/// `single_tour.html` / base); the booking sidebar comes from the
/// `sidebar-tour` widget rows.
pub(crate) fn page(conn: &Connection, templates: &Path, ctx: &RenderCtx) -> Result<String> {
    // featured image for hero background + JSON-LD image (lazy lookup —
    // one row, not the whole media table).
    let (featured_id, excerpt, seo_title, seo_desc): (Option<i64>, String, String, String) = conn
        .query_row(sql!("tour__select_post_row"), [ctx.id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .unwrap_or((None, String::new(), String::new(), String::new()));

    let featured_img = featured_id
        .and_then(|mid| crate::render::media_lookup(conn, mid))
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
    let tour_path = format!("{}/{}/{}/", url_prefix(conn, &ctx.lang), ctx.kind_base, display);
    let canonical = format!("{}{}", ctx.site_url, tour_path);

    let alternates = translation_urls(conn, ctx.id, &ctx.site_url, &ctx.kind_base);
    let hreflang_links = hreflang_links(&alternates);

    let jsonld = product_jsonld(&ctx.title, ctx.id, &canonical, &description, &price, &featured_img, &site_name_of(conn));

    let theme_dir = crate::render::theme_template_dir(conn, templates);
    let env = load_env_theme(templates, theme_dir.as_deref())?;
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
        &alternates,
        &ctx.kind_base,
    )?;
    // seo_title/title hit <title> + meta content attributes — pre-escaped
    // at the boundary (base templates run AutoEscape::None).
    let sidebar_tour = crate::widgets::render_sidebar(conn, "sidebar-tour", &ctx.site_url, &ctx.lang, &ctx.slug, &ctx.kind_base)?;
    for (k, v) in serde_json::json!({
        "canonical": canonical,
        "seo_title": esc_attr(&seo_title),
        "seo_desc": seo_desc_out,
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

    tctx.insert(
        "base".into(),
        serde_json::json!(crate::lang::template_name(
            templates,
            theme_dir.as_deref(),
            &crate::lang::config(conn),
            &ctx.lang,
        )),
    );
    let navx = match tctx.get("navx") {
        Some(serde_json::Value::Object(m)) => m.clone(),
        _ => serde_json::Map::new(),
    };
    let html = env
        .get_template("single_tour.html")?
        .render(minijinja::Value::from_serialize(serde_json::Value::Object(tctx)))
        .map_err(|e| anyhow::anyhow!("render single_tour.html: {e}"))?;
    Ok(crate::render::apply_navx(&html, &ctx.site_url, &navx, conn, &ctx.lang))
}

use crate::html_decode::{esc, esc_attr};

fn url_prefix(conn: &Connection, lang: &str) -> String {
    crate::lang::url_prefix(&crate::lang::config(conn), lang)
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

/// Adapter-trait view: always `Some` — the computed display slug already
/// covers the stored-slug fallback (`slug` returned verbatim when no
/// recovery path yields a better value).
pub(crate) fn display_slug_opt(conn: &Connection, id: i64, slug: &str, title: &str) -> Option<String> {
    Some(display_slug(conn, id, slug, title))
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

fn translation_urls(conn: &Connection, id: i64, site: &str, kind_base: &str) -> Vec<(String, String)> {
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
                // sibling kind dispatch: the group members all carry this
                // adapter's kind, but a stale row must never post-ify.
                let display = crate::plugins::adapter_for_kind(&kind)
                    .and_then(|a| a.display_slug(conn, pid, &slug, &title))
                    .unwrap_or(slug);
                (lang.clone(), crate::lang::page_url(&cfg, site, &lang, &kind, &display, Some(kind_base)))
            })
        })
        .collect()
}

fn hreflang_links(alternates: &[(String, String)]) -> String {
    alternates
        .iter()
        .filter(|(_, u)| !u.is_empty())
        .map(|(l, u)| format!("<link rel=\"alternate\" href=\"{}\" hreflang=\"{}\" />", esc_attr(u), esc_attr(l)))
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


// ======================= site_data feeds =======================

/// `site.tours` feed + the tourmaster option-derived keys (`search_url`,
/// `search_months`) — moved from `render::site_data` in phase 03. Core
/// merges the `(key, value)` pairs verbatim into the shared site map.
pub(crate) fn site_data_feeds(
    conn: &Connection,
    lang: &str,
    site_url: &str,
    kind_base: &str,
) -> Vec<(String, Value)> {
    let lang_cfg = crate::lang::config(conn);
    let prefix = crate::lang::url_prefix(&lang_cfg, lang);
    // Featured images resolve one point-lookup at a time (lazy media_map).
    let seen = std::cell::RefCell::new(std::collections::HashMap::<i64, Option<(String, String)>>::new());
    let media_row = |mid: i64| -> Option<(String, String)> {
        seen.borrow_mut()
            .entry(mid)
            .or_insert_with(|| crate::render::media_lookup(conn, mid))
            .clone()
    };
    let img = |mid: Option<i64>| -> Value {
        mid.and_then(&media_row)
            .map(|(p, _)| json!(format!("{site_url}/wp-content/uploads/{p}")))
            .unwrap_or(Value::Null)
    };

    // tours (kind='tour' joined to tours table) with term slugs per taxonomy.
    // '-copy' slugs are WP duplicated drafts published post-capture — the
    // reference predates them, so exclude to match live selection.
    let mut tours = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT p.id, p.slug, p.title, COALESCE(p.excerpt,''), p.featured_media_id, \
         t.duration_days, COALESCE(t.price_range,''), COALESCE(t.rating,0), t.max_people, \
         COALESCE(p.published_at,'') \
         FROM posts p JOIN tours t ON t.post_id=p.id \
         WHERE p.kind='tour' AND p.lang=?1 AND p.status='publish' \
         AND p.slug NOT LIKE '%-copy' \
         ORDER BY p.menu_order, p.published_at",
    ) {
        let rows: Vec<(i64, String, String, String, Option<i64>, Option<i64>, String, f64, Option<i64>, String)> = stmt
            .query_map([lang], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<i64>>(4)?,
                    r.get::<_, Option<i64>>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, f64>(7)?,
                    r.get::<_, Option<i64>>(8)?,
                    r.get::<_, String>(9)?,
                ))
            })
            .map(|rs| rs.flatten().collect())
            .unwrap_or_default();
        for (id, slug, title, excerpt, fid, days, price, rating, max_people, pub_at) in rows {
            let mut terms = Vec::new();
            if let Ok(mut tstmt) = conn.prepare(
                "SELECT taxonomy, slug, name FROM post_terms WHERE post_id=?1",
            ) {
                if let Ok(trows) = tstmt.query_map([id], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                }) {
                    for t in trows.flatten() {
                        terms.push(json!({"taxonomy": t.0, "slug": t.1, "name": t.2}));
                    }
                }
            }
            // tour option meta: ribbon, duration text, departure, price, rating
            let opt = conn
                .query_row(
                    "SELECT meta_value FROM post_meta WHERE post_id=?1 \
                     AND meta_key='tourmaster-tour-option' LIMIT 1",
                    [id],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .and_then(|raw| {
                    crate::php_serialize::unserialize(raw.as_bytes()).ok()
                });
            let opt_s = |k: &str| -> String {
                opt.as_ref()
                    .and_then(|v| v.get(k))
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            // promo ribbon falls back to style term names
            let mut promo_text = opt_s("promo-text");
            if promo_text.is_empty() {
                let theme = crate::plugins::theme_for_site(conn);
                let primary = crate::lang::config(conn).primary;
                let style_tax = crate::plugins::localized_taxonomy(theme, "style", lang, &primary);
                promo_text = terms
                    .iter()
                    .filter(|t| {
                        t.get("taxonomy").and_then(|x| x.as_str())
                            == Some(style_tax)
                    })
                    .filter_map(|t| t.get("name").and_then(|x| x.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ");
            }
            let promo_bg = opt_s("promo-text-ribbon-background");
            // rating + reviewer count live in tourmaster-tour-rating meta
            let mut review_count: i64 = 0;
            let mut rating = rating;
            if let Ok(raw) = conn.query_row::<String, _, _>(
                "SELECT meta_value FROM post_meta WHERE post_id=?1 \
                 AND meta_key='tourmaster-tour-rating' LIMIT 1",
                [id],
                |r| r.get(0),
            ) {
                if let Ok(v) = crate::php_serialize::unserialize(raw.as_bytes()) {
                    if let Some(sc) = v
                        .get("score")
                        .and_then(|x| x.as_f64().or_else(|| x.as_i64().map(|i| i as f64)))
                    {
                        rating = sc / 2.0;
                    }
                    review_count = v
                        .get("reviewer")
                        .and_then(|x| x.as_i64())
                        .unwrap_or(0);
                }
            }
            // display price: tour-price-text > tourmaster-tour-price > price_range
            let raw_price = {
                let t = opt_s("tour-price-text");
                if !t.is_empty() {
                    t
                } else {
                    conn.query_row::<String, _, _>(
                        "SELECT meta_value FROM post_meta WHERE post_id=?1 \
                         AND meta_key='tourmaster-tour-price' LIMIT 1",
                        [id],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| price.clone())
                }
            };
            // tour permalinks use display_slug (raw slug can be degenerate,
            // e.g. 'x' on 6416) — the value file path + canonical agree on.
            let tslug = display_slug(conn, id, &slug, &title);
            tours.push(json!({
                "id": id, "slug": slug, "title": title, "excerpt": excerpt,
                "url": format!("{site_url}{prefix}/{kind_base}/{tslug}/"),
                "image": img(fid),
                "alt": fid.and_then(&media_row).map(|(_, a)| a.clone()),
                "duration_days": days, "price": price, "rating": rating,
                "max_people": max_people, "terms": terms,
                "duration_text": opt_s("duration-text"),
                "departure": opt_s("departure-location"),
                "promo_text": promo_text,
                "promo_color": opt_s("promo-text-ribbon-text-color"),
                "promo_bg": promo_bg,
                "promo_bg_alpha": hex_rgba(&promo_bg, 0.5),
                "price_display": fmt_tour_price(&raw_price),
                "review_count": review_count, "date": pub_at,
            }));
        }
    }

    // tourmaster_general options: search page url + month filter list, both
    // consumed by elements/tour_search.html.
    let general: Option<Value> = crate::db::plugin_setting_raw_conn(conn, "tourmaster", "tourmaster_general")
        .and_then(|raw| crate::php_serialize::unserialize(raw.as_bytes()).ok());
    let gen_opt = |key: &str| -> Option<String> {
        general
            .as_ref()
            .and_then(|g| g.get(key))
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            .filter(|s| !s.is_empty())
    };

    // tourmaster_get_template_url('search'): permalink of the 'search-page'
    // option resolved to the current language; home url when unset.
    let search_url = {
        let mut sp = gen_opt("search-page").and_then(|s| s.parse::<i64>().ok());
        if lang != lang_cfg.primary {
            if let Some(sid) = sp {
                // N-lang: resolve the configured page's member in this
                // language (post_translation_members); the legacy fr/es
                // pair table still resolves for sites imported before
                // members sync.
                let member: Option<i64> = conn
                    .query_row(
                        "SELECT m2.post_id FROM post_translation_members m1 \
                         JOIN post_translation_members m2 ON m2.group_id=m1.group_id AND m2.lang=?2 \
                         WHERE m1.post_id=?1 LIMIT 1",
                        rusqlite::params![sid, lang],
                        |r| r.get(0),
                    )
                    .ok();
                sp = member.or(sp); // post_translations retired — group only
            }
        }
        match sp.and_then(|sid| slug_of(conn, sid)) {
            Some((slug, kind)) => crate::render::page_url(conn, site_url, lang, &kind, &slug, kind_base),
            None => format!("{site_url}{prefix}/"),
        }
    };

    // 'month' combobox: N months starting this month (PHP
    // search-month-amount + date_i18n('F Y')); days→civil arithmetic, no chrono.
    let month_amount: i64 = gen_opt("search-month-amount")
        .and_then(|s| s.parse().ok())
        .unwrap_or(12)
        .clamp(0, 36);
    let months = search_months(month_amount, crate::lang::month_names(lang));

    vec![
        ("tours".to_string(), Value::Array(tours)),
        ("search_url".to_string(), json!(search_url)),
        ("search_months".to_string(), Value::Array(months)),
    ]
}
/// 'month' combobox entries: N months starting this month
/// (PHP search-month-amount + date_i18n('F Y')); days→civil arithmetic,
/// no chrono. Absolute month index per slot — the old `y += (mi-1)/12`
/// accumulator compounded the year once per rollover, not per year.
#[doc(hidden)] // pub for tests/audit_remediation.rs — not crate API.
pub fn search_months(month_amount: i64, month_names: &[&str; 12]) -> Vec<Value> {
    let days = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0)) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let (y, m) = (if mp >= 10 { y + 1 } else { y }, mp + 3 - (if mp >= 10 { 12 } else { 0 }));
    let start = y * 12 + (m - 1); // absolute month index
    (0..month_amount)
        .map(|i| {
            let t = start + i;
            let (y, mm) = (t / 12, t % 12 + 1);
            json!({"slug": format!("{y:04}-{mm:02}"),
                   "name": format!("{} {}", month_names[(mm - 1) as usize], y)})
        })
        .collect()
}

/// "#10873b" + alpha -> "rgba(16, 135, 59, 0.5)" for the ribbon corner.
/// `is_ascii` gate: a multi-byte stored color has byte-len 6 but
/// `h[i..i+2]` would slice mid-char — return empty, never panic.
#[doc(hidden)] // pub for tests/audit_remediation.rs — not crate API.
pub fn hex_rgba(hex: &str, alpha: f64) -> String {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 || !h.is_ascii() {
        return String::new();
    }
    let byte = |i: usize| -> Option<u8> { u8::from_str_radix(&h[i..i + 2], 16).ok() };
    match (byte(0), byte(2), byte(4)) {
        (Some(r), Some(g), Some(b)) => format!("rgba({r}, {g}, {b}, {alpha})"),
        _ => String::new(),
    }
}

/// Format a tour price string: bare digits -> "1,825 USD", else raw text.
fn fmt_tour_price(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return String::new();
    }
    if t.chars().all(|c| c.is_ascii_digit()) {
        let mut out = String::new();
        for (i, c) in t.chars().enumerate() {
            if i > 0 && (t.len() - i) % 3 == 0 {
                out.push(',');
            }
            out.push(c);
        }
        format!("{out} USD")
    } else {
        t.to_string()
    }
}
