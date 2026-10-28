//! Shared `<head>`/chrome vars + permalink helpers: alternates, hreflang,
//! flags, nav map, meta block (moved out of render.rs — pure move, no
//! behavior change).

#[allow(unused_imports)]
use super::*;

pub(crate) fn translation_urls(conn: &Connection, id: i64, site: &str, tour_base: &str) -> Result<Vec<(String, String)>> {
    let cfg = crate::lang::config(conn);
    let group: Option<String> = conn
        .query_row(
            "SELECT group_id FROM post_translation_members WHERE post_id=?1 LIMIT 1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(group) = group else { return Ok(Vec::new()); };
    let mut stmt = conn.prepare(
        "SELECT lang, post_id FROM post_translation_members WHERE group_id=?1 ORDER BY lang",
    )?;
    let rows = stmt
        .query_map([group], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut out = Vec::new();
    for (lang, pid) in rows {
        if let Some((slug, kind)) = slug_of(conn, pid) {
            let base = crate::admin::plugins::kind_url_base(tour_base, &kind);
            out.push((lang.clone(), crate::lang::page_url(&cfg, site, &lang, &kind, &slug, base.as_deref())));
        }
    }
    Ok(out)
}

pub(crate) fn hreflang_links(alternates: &[(String, String)]) -> String {
    alternates
        .iter()
        .map(|(lang, url)| format!("<link rel=\"alternate\" href=\"{}\" hreflang=\"{}\" />", esc_attr(url), esc_attr(lang)))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn slug_of(conn: &Connection, id: i64) -> Option<(String, String)> {
    conn.query_row(
        sql!("render__select_slug_kind"),
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .ok()
}

pub(crate) fn page_url(conn: &Connection, site: &str, lang: &str, kind: &str, slug: &str, tour_base: &str) -> String {
    let cfg = crate::lang::config(conn);
    let prefix = crate::lang::url_prefix(&cfg, lang);
    match crate::admin::plugins::kind_url_base(tour_base, kind) {
        Some(b) if !b.is_empty() => format!("{site}{prefix}/{b}/{slug}/"),
        _ => format!("{site}{prefix}/{slug}/"),
    }
}

/* ================= shared head/chrome context (Phase 2) ================= */

/// Full media row for an attachment id (featured/gallery): (path, alt,
/// mime, width, height). Used by the og:image block and the gallery element.
pub(crate) fn media_detail(
    conn: &Connection,
    id: i64,
) -> Result<Option<(String, String, String, i64, i64)>> {
    Ok(conn
        .query_row(sql!("render__select_media_og"), [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .ok())
}

/// RankMath-style robots meta. `rank_math_robots` post_meta holds a PHP
/// array like `a:2:{i:0;s:5:"index";i:1;s:8:"nofollow";}`; WP renders it
/// follow/nofollow first, then index, then the fixed snippet suffix.
fn robots_meta(ser: &str) -> String {
    let mut items: Vec<String> = Vec::new();
    if let Ok(Value::Array(a)) = crate::php_serialize::unserialize(ser.as_bytes()) {
        items = a
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }
    let has = |n: &str| items.iter().any(|x| x == n);
    let follow = if has("nofollow") { "nofollow" } else { "follow" };
    let index = if has("noindex") {
        "noindex"
    } else {
        "index"
    };
    format!(
        "{follow}, {index}, max-snippet:-1, max-video-preview:-1, max-image-preview:large"
    )
}

/// RankMath `rank_math_schema_{Type}` post_meta — PHP-serialized schema.org
/// template with `%placeholder%` tokens. Substitute the known tokens with
/// post fields, emit as JSON-LD. Unknown tokens resolve to empty strings;
/// the `metadata` block is RankMath-internal and dropped.
fn post_jsonld(
    conn: &Connection,
    id: i64,
    title: &str,
    desc: &str,
    published_at: &str,
    modified_at: &str,
    img_url: &str,
    site_url: &str,
) -> Option<String> {
    let types = ["Article", "BlogPosting", "VideoObject", "Product", "FAQPage"];
    let mut raw = String::new();
    for t in types {
        let v: String = conn
            .query_row(
                "SELECT COALESCE(meta_value,'') FROM post_meta WHERE post_id=?1 AND meta_key=?2 LIMIT 1",
                rusqlite::params![id, format!("rank_math_schema_{t}")],
                |r| r.get(0),
            )
            .unwrap_or_default();
        if !v.is_empty() {
            raw = v;
            break;
        }
    }
    if raw.is_empty() {
        return None;
    }
    let value = crate::php_serialize::unserialize(raw.as_bytes()).ok()?;
    let author_name = conn
        .query_row(
            "SELECT value FROM settings WHERE key='site_name' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "rpress".into());
    let json = schema_to_json(&value, &|tok: &str| -> String {
        match tok {
            "seo_title" | "title" => title.to_string(),
            "seo_description" | "description" | "excerpt" => desc.to_string(),
            t if t.starts_with("date") => iso8601(published_at),
            t if t.starts_with("modified") => iso8601(modified_at),
            "post_thumbnail" | "image" => img_url.to_string(),
            "name" | "author" => author_name.clone(),
            "url" | "site_url" => site_url.to_string(),
            "keywords" | "categories" | "tags" => String::new(),
            _ => String::new(),
        }
    });
    serde_json::to_string(&json).ok()
}

/// PHP Value → serde_json::Value, walking `%token%` substitutions. Drops
/// the RankMath `metadata` block — it's bookkeeping, not schema output.
fn schema_to_json(v: &Value, sub: &dyn Fn(&str) -> String) -> Value {
    use serde_json::Map;
    match v {
        Value::Object(m) => {
            let mut out = Map::new();
            for (k, val) in m {
                if k == "metadata" {
                    continue;
                }
                out.insert(k.clone(), schema_to_json(val, sub));
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| schema_to_json(x, sub)).collect()),
        Value::String(s) => {
            let mut out = s.clone();
            let mut i = 0usize;
            while let Some(start) = out[i..].find('%') {
                let s0 = i + start;
                match out[s0 + 1..].find('%') {
                    Some(e0) => {
                        let e = s0 + 1 + e0;
                        let tok = out[s0 + 1..e].to_string();
                        let rep = sub(&tok);
                        out = format!("{}{}{}", &out[..s0], rep, &out[e + 1..]);
                        i = s0 + rep.len();
                    }
                    None => break,
                }
            }
            Value::String(out)
        }
        other => other.clone(),
    }
}


/// Percent-encode a query/path value — byte-wise UTF-8, shared with the
/// admin module (`crate::admin::urlenc` is the single implementation).
fn urlenc_path(s: &str) -> String {
    crate::admin::urlenc(s)
}


/// `posts.modified_at` "2025-08-21 11:39:05" → "2025-08-21T11:39:05+00:00".
pub(crate) fn iso8601(dt: &str) -> String {
    let t = dt.trim().replace(' ', "T");
    if t.is_empty() {
        String::new()
    } else {
        format!("{t}+00:00")
    }
}

/// body_class per WP body_class(): the theme suffix comes from the
/// `site_body_suffix` setting (seeded per site/theme), falling back to
/// traveltour's value only when unset.
pub(crate) fn body_class(conn: &Connection, _lang: &str, kind: &str, id: i64, is_front: bool) -> String {
    // Resolution order: explicit `site_body_suffix` seed → the matched
    // theme adapter's `body_suffix` → traveltour (legacy default).
    let suffix = conn
        .query_row("SELECT value FROM settings WHERE key='site_body_suffix'", [], |r| {
            r.get::<_, String>(0)
        })
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let stylesheet: String = conn
                .query_row("SELECT value FROM settings WHERE key='stylesheet'", [], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap_or_default();
            crate::admin::plugins::theme_for_stylesheet(&stylesheet)
                .filter(|d| !d.body_suffix.is_empty())
                .map(|d| d.body_suffix.to_string())
        })
        .unwrap_or_else(|| {
            "wp-theme-traveltour wp-child-theme-traveltour-child gdlr-core-body tourmaster-body metaslider-plugin traveltour-body traveltour-body-front traveltour-full  traveltour-with-sticky-navigation gdlr-core-link-to-lightbox".into()
        });
    let core = if is_front {
        format!("home wp-singular page-template-default page page-id-{id}")
    } else {
        match kind {
            "post" => format!(
                "wp-singular post-template-default single single-post postid-{id} single-format-standard"
            ),
            "tour" => format!(
                "wp-singular tour-template-default single single-tour postid-{id} wp-theme-traveltour wp-child-theme-traveltour-child gdlr-core-body tourmaster-bottom-booking-bar tourmaster-mobile-read-more tourmaster-body metaslider-plugin traveltour-body traveltour-body-front traveltour-full  traveltour-with-sticky-navigation gdlr-core-link-to-lightbox"
            ),
            _ => format!("wp-singular page-template-default page page-id-{id}"),
        }
    };
    if kind == "tour" && !is_front {
        core
    } else {
        format!("{core} {suffix}")
    }
}

/// Header navigation `current-*` classes keyed by `{tax-or-type}/{slug}` —
/// apply_navx matches them against the resolved menu hrefs.
///
/// WP adds for a single tour: `current-tour-ancestor current-menu-parent
/// current-tour-parent` on taxonomy items pointing at one of the tour's
/// terms; for a post, `current-menu-item` on the post object item (and the
/// FR reference shows nothing on the post page — only filled when a menu
/// item actually matches).
pub(crate) fn nav_map(
    conn: &Connection,
    id: i64,
    kind: &str,
    slug: &str,
    is_front: bool,
    lang: &str,
) -> Result<Map<String, Value>> {
    let mut m = Map::new();
    match kind {
        "tour" => {
            /* Only the tour's terms that a menu item actually links to can
               ever get the ancestor classes — intersect the post's terms
               with `menu_items.object_id` taxonomy items (terms.id is the
               WP term id). No hardcoded taxonomy list: theme_data owns it. */
            let mut st = conn.prepare(
                "SELECT DISTINCT pt.taxonomy, pt.slug FROM post_terms pt \
                 JOIN terms t ON t.taxonomy=pt.taxonomy COLLATE NOCASE AND t.slug=pt.slug \
                 JOIN menu_items mi ON mi.item_type='taxonomy' AND mi.object_id=t.id \
                 WHERE pt.post_id=?1",
            )?;
            let rows = st.query_map([id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?;
            for r in rows.flatten() {
                m.insert(
                    format!("{}/{}", r.0, r.1),
                    json!(" current-tour-ancestor current-menu-parent current-tour-parent"),
                );
            }
        }
        "post" => {
            m.insert(format!("post/{slug}"), json!(" current-menu-item"));
        }
        _ => {
            m.insert(
                format!("page/{slug}"),
                json!(" current-menu-item current_page_item"),
            );
        }
    }
    if is_front && lang != crate::lang::config(conn).primary {
        // Non-primary localized front marks its language-root nav item —
        // apply_navx resolves `home/{lang}` to `/{lang}/`. The primary
        // front does not (WP reference).
        m.insert(
            format!("home/{lang}"),
            json!(" current-menu-item current_page_item menu-item-home"),
        );
    }
    Ok(m)
}

/// Shared <head>/chrome template variables for base_fr/base_es. `seo_desc`
/// arrives already resolved by the caller (it differs subtly per kind);
/// everything else is computed here from the post row + post_meta so the
/// three render paths stay consistent.
///
/// Emitted keys (see templates/base_fr.html): html_lang, html_prefix,
/// body_class, is_front, is_contact, robots, og_type, og_desc,
/// og_image_block, og_updated, tw_label1/data1/label2/data2, og_feed_title
///(unused), feed_title, oembed_url, rest_url, shortlink, flags (N-lang
/// [{lang,url,flag}] — templates iterate it), navx, head_css_extra/foot_scripts_extra placeholders.
pub(crate) fn chrome_vars(
    conn: &Connection,
    site_url: &str,
    id: i64,
    kind: &str,
    slug: &str,
    lang: &str,
    title: &str,
    canonical: &str,
    seo_desc: &str,
    body_html: &str,
    alternates: &[(String, String)],
    tour_base: &str,
) -> Result<Map<String, Value>> {
    let (modified_at, featured_id, excerpt, published_at): (String, Option<i64>, String, String) = conn
        .query_row(sql!("render__select_chrome"), [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .unwrap_or_default();
    let meta = |key: &str| -> String {
        conn.query_row(
            "SELECT COALESCE(meta_value,'') FROM post_meta WHERE post_id=?1 AND meta_key=?2 LIMIT 1",
            rusqlite::params![id, key],
            |r| r.get(0),
        )
        .unwrap_or_default()
    };

    let lang_cfg = crate::lang::config(conn);
    // is_front: this post is the localized front for its language, or the
    // canonical landed on a language root (covers fr/es canonical checks
    // the old two-literal compare encoded).
    let is_front = front_id_for_lang(conn, lang) == Some(id)
        || lang_cfg.langs.iter().any(|l| {
            canonical == format!("{}{}/", site_url, crate::lang::url_prefix(&lang_cfg, l))
        });
    /* Contact-form pages: the imported CF7 form slugs, keyed per lang
       (settings.contact_slugs seeded at import). */
    let is_contact = conn
        .query_row("SELECT value FROM settings WHERE key='contact_slugs'", [], |r| {
            r.get::<_, String>(0)
        })
        .ok()
        .and_then(|v| serde_json::from_str::<serde_json::Value>(&v).ok())
        .map(|m| {
            lang_cfg
                .langs
                .iter()
                .any(|l| m.get(l).and_then(|a| a.as_array()).map(|a| a.iter().any(|s| s.as_str() == Some(slug))).unwrap_or(false))
        })
        .unwrap_or(false);
    let is_single = !is_front;

    // --- meta description / og:description -------------------------------
    // seo_desc already resolved by caller; else rank_math_description, then
    // excerpt, then the auto excerpt RankMath generates from post_content.
    let desc = if !seo_desc.is_empty() {
        seo_desc.to_string()
    } else {
        let rm = meta("rank_math_description");
        if !rm.is_empty() {
            rm
        } else if !excerpt.is_empty() {
            excerpt.clone()
        } else {
            markdown::auto_desc(body_html)
        }
    };

    // --- og:image block ---------------------------------------------------
    // featured_media_id → _thumbnail_id → 2000×1010 PNG logo (the RankMath
    // default for pages without an image; the home page sets it explicitly).
    let img_id = featured_id
        .or_else(|| meta("_thumbnail_id").parse::<i64>().ok())
        .or_else(|| {
            conn.query_row("SELECT CAST(value AS INTEGER) FROM settings WHERE key='og_fallback_media'", [], |r| r.get(0))
                .ok()
        })
        .unwrap_or(0);
    /* No featured image → the site's og fallback (import seeds
       `og_fallback_media` from the theme logo attachment). Nothing at all
       → omit the block rather than pin another site's logo. */
    let detail = media_detail(conn, img_id).ok().flatten();
    let img_url = detail
        .as_ref()
        .map(|(p, _, _, _, _)| format!("{site_url}/wp-content/uploads/{p}"))
        .unwrap_or_default();
    let (alt, mime, w, h) = detail
        .as_ref()
        .map(|(_, a, m, w, h)| (a.clone(), m.clone(), *w, *h))
        .unwrap_or_default();
    let og_image_block = if img_url.is_empty() {
        String::new()
    } else {
        format!(
            "<meta property=\"og:image\" content=\"{u}\" />\n\
             <meta property=\"og:image:secure_url\" content=\"{u}\" />\n\
             <meta property=\"og:image:width\" content=\"{w}\" />\n\
             <meta property=\"og:image:height\" content=\"{h}\" />\n\
             <meta property=\"og:image:alt\" content=\"{a}\" />\n\
             <meta property=\"og:image:type\" content=\"{m}\" />",
            u = esc_attr(&img_url),
            w = w,
            h = h,
            a = esc_attr(&alt),
            m = mime,
        )
    };

    // --- twitter label/data pairs -----------------------------------------
    // post: Écrit par + Temps de lecture; page: label1=lecture only; tour &
    // front: none (front emits the static author block in the template).
    let words = markdown::word_count(body_html);
    let (tw_label1, tw_data1, tw_label2, tw_data2) = match kind {
        "post" => (
            crate::lang::t(lang, "written_by").to_string(),
            /* Bylines come from the post's imported author (post_meta
               author_login); archive cards read the same key. */
            meta("author_login"),
            crate::lang::t(lang, "reading_time").to_string(),
            markdown::reading_time(words, lang),
        ),
        "page" => (
            crate::lang::t(lang, "reading_time").to_string(),
            markdown::reading_time(words, lang),
            String::new(),
            String::new(),
        ),
        _ if is_front => {
            // Front twitter card labels: translated static labels + the
            // post's own author byline (wp post_author of the front page).
            (
                crate::lang::t(lang, "written_by").to_string(),
                meta("author_login"),
                crate::lang::t(lang, "reading_time").to_string(),
                markdown::reading_time(words, lang),
            )
        }
        _ => (String::new(), String::new(), String::new(), String::new()),
    };

    // --- flags ------------------------------------------------------------
    // Generic per-language flag row: each configured language gets a flag;
    // the URL is its translation-group alternate (or self canonical when no
    // alternate), except the non-primary root-only alternate is dropped on
    // non-front pages (the old es-only "don't link a bare /es/" rule).
    let href_of = |l: &str| -> String {
        alternates
            .iter()
            .find(|(a, _)| a == l)
            .map(|(_, u)| u.clone())
            .unwrap_or_default()
    };
    let flags: Vec<Value> = lang_cfg
        .langs
        .iter()
        .map(|l| {
            let url = if l == lang {
                canonical.to_string()
            } else if l == &lang_cfg.primary {
                let a = href_of(l);
                // Primary lang with no stored alternate falls back to its
                // root — base_es's historical hardcoded `{{ site_url }}/`.
                if a.is_empty() { format!("{site_url}/") } else { a }
            } else {
                let a = href_of(l);
                let root = format!("{}{}/", site_url, crate::lang::url_prefix(&lang_cfg, l));
                if !is_front && a == root { String::new() } else { a }
            };
            let src = if lang_cfg.langs.len() < 2 {
                String::new() // no switcher on single-language sites
            } else {
                crate::render::site_data::flag_src(conn, site_url, l)
            };
            json!({"lang": l, "url": url, "flag": crate::lang::flag_file(l), "src": src})
        })
        .collect();

    // --- per-page link tags -------------------------------------------------
    let rest_type = match kind {
        "post" => "posts",
        "tour" => "tour",
        _ => "pages",
    };
    let rest_url = format!("{site_url}/wp-json/wp/v2/{rest_type}/{id}");
    let oembed_url = urlenc_path(canonical);
    let shortlink = if is_front {
        canonical.to_string()
    } else {
        format!("{site_url}/?p={id}")
    };

    let mut m = Map::new();
    /* `site` carries the per-lang site_data map (menus, tours feed, …).
       The base templates iterate `site.menus.*` for header/footer nav —
       expose the whole cached map under one key. */
    if let Ok(site) = crate::render::site_data_cached(conn, lang, site_url, tour_base) {
        m.insert(
            "site".into(),
            serde_json::to_value(&site).unwrap_or(Value::Null),
        );
    }
    m.insert("html_lang".into(), json!(crate::lang::html_lang(lang)));
    m.insert("og_locale".into(), json!(crate::lang::html_lang(lang).replace('-', "_")));
    m.insert("kind".into(), json!(kind));
    m.insert(
        "html_prefix".into(),
        json!("prefix=\"og: https://ogp.me/ns#\" class=\"no-js\""),
    );
    m.insert("body_class".into(), json!(body_class(conn, lang, kind, id, is_front)));
    m.insert("is_front".into(), json!(is_front));
    m.insert("is_contact".into(), json!(is_contact));
    m.insert("is_single".into(), json!(is_single));
    m.insert("robots".into(), json!(robots_meta(&meta("rank_math_robots"))));
    m.insert(
        "og_type".into(),
        json!(if is_front { "website" } else { "article" }),
    );
    m.insert("og_desc".into(), json!(esc_attr(&desc)));
    m.insert("og_image_block".into(), json!(og_image_block));
    m.insert("og_image_url".into(), json!(img_url));
    m.insert("og_updated".into(), json!(iso8601(&modified_at)));
    m.insert("tw_label1".into(), json!(tw_label1));
    m.insert("tw_data1".into(), json!(tw_data1));
    m.insert("tw_label2".into(), json!(tw_label2));
    m.insert("tw_data2".into(), json!(tw_data2));
    m.insert("oembed_url".into(), json!(oembed_url));
    m.insert("rest_url".into(), json!(rest_url));
    m.insert("shortlink".into(), json!(shortlink));
    m.insert("flags".into(), json!(flags));
    m.insert("navx".into(), Value::Object(nav_map(conn, id, kind, slug, is_front, lang)?));

    /* `gdlr-core-page-option` enable-header-area=disable → WP renders no
       header/topbar/mobile-header at all and marks the outer wrapper
       `traveltour-header-disable` (origin-verified on pages like
       idees-rencontres-ethniques). */
    let header_disable = {
        let raw = meta("gdlr-core-page-option");
        crate::php_serialize::unserialize(raw.as_bytes())
            .ok()
            .and_then(|v| v.get("enable-header-area").and_then(|x| x.as_str().map(str::to_string)))
            .as_deref()
            == Some("disable")
    };
    m.insert("header_disable".into(), json!(header_disable));

    let page_title_disable = {
        let raw = meta("gdlr-core-page-option");
        crate::php_serialize::unserialize(raw.as_bytes())
            .ok()
            .and_then(|v| v.get("enable-page-title").and_then(|x| x.as_str().map(str::to_string)))
            .as_deref()
            == Some("disable")
    };
    let page_title_html = if kind == "page" && !is_front && !page_title_disable {
        /* Page-title background image: the page's `gdlr-core-page-option`
           `title-background` media id (per-page option, imported into
           post_meta) — media path via media.wp_id lookup. */
        let bg = {
            let raw = meta("gdlr-core-page-option");
            if raw.is_empty() {
                String::new()
            } else {
                crate::php_serialize::unserialize(raw.as_bytes())
                    .ok()
                    .and_then(|v| {
                        let t = v.get("title-background")?;
                        t.as_i64().or_else(|| t.as_str().and_then(|s| s.parse().ok()))
                    })
                    .and_then(|mid| {
                        conn.query_row("SELECT path FROM media WHERE wp_id=?1", [mid], |r| {
                            r.get::<_, String>(0)
                        })
                        .ok()
                    })
                    .map(|p| format!(" style=\"background-image: url({site_url}/wp-content/uploads/{p}) ;\""))
                    .unwrap_or_default()
            }
        };
        format!("<div class=\"traveltour-page-title-wrap  traveltour-style-large traveltour-center-align\"{bg} ><div class=\"traveltour-header-transparent-substitute\" ></div><div class=\"traveltour-page-title-overlay\"  ></div><div class=\"traveltour-page-title-container traveltour-container\" ><div class=\"traveltour-page-title-content traveltour-item-pdlr\"  ><h1 class=\"traveltour-page-title\"  >{}</h1></div></div></div>", esc(title))
    } else {
        String::new()
    };
    m.insert("page_title_html".into(), json!(page_title_html));
    // hooks filled by other render paths/phases; defined so the base
    // template can reference them unconditionally.
    // settings.custom_css = WP "Additional CSS" carried by the importer —
    // emitted inline like wp_custom_css_cb does.
    let custom_css: String = conn
        .query_row("SELECT value FROM settings WHERE key='custom_css'", [], |r| r.get(0))
        .unwrap_or_default();
    /* WP Categories Widget plugin emits `wcw-inlinecss-inline-css` before
       WP Additional CSS (`wp-custom-css`). Ordering matters: this plugin
       default color is blue, and the site's custom CSS overrides it to
       Traveltour/Aucoeur colors. */
    let wcw = include_str!("../admin/plugins/theme_traveltour/seed/wcw-inlinecss.html");
    let custom_css_tag = if custom_css.trim().is_empty() {
        String::new()
    } else {
        format!(
            "<style id='wp-custom-css' type='text/css'>{}</style>",
            custom_css.replace("</style", "<\\/style")
        )
    };
    m.insert("head_css_extra".into(), json!(format!("{wcw}{custom_css_tag}")));
    m.insert("head_css_eztoc".into(), json!(""));
    m.insert("head_css_wpdiscuz".into(), json!(""));
    m.insert("foot_scripts_eztoc".into(), json!(""));
    m.insert("foot_scripts_comment_reply".into(), json!(""));
    m.insert("foot_scripts_wpdiscuz".into(), json!(""));
    // RankMath schema.org JSON-LD (rank_math_schema_{Type} post_meta):
    // unserialize the stored template, substitute %token% fields, emit the
    // <script> tag. Empty when the post carries no schema meta.
    let jsonld = post_jsonld(conn, id, title, &desc, &published_at, &modified_at, &img_url, site_url)
        .map(|j| format!("<script type=\"application/ld+json\">{j}</script>"))
        .unwrap_or_default();
    m.insert("post_jsonld".into(), json!(jsonld));
    /* Auhfc "insert headers/footers" plugin payload: arbitrary head HTML
       per post (org JSON-LD, preload links — content owned by the site,
       kept verbatim). Deserialized `head` field only. */
    let sitewide = conn
        .query_row("SELECT value FROM settings WHERE key='site_head_html'", [], |r| r.get::<_, String>(0))
        .unwrap_or_default();
    let head_extra = crate::php_serialize::unserialize(meta("_auhfc").as_bytes())
        .ok()
        .and_then(|v| v.get("head").and_then(|h| h.as_str().map(str::to_string)))
        .unwrap_or_default();
    let head_extra = format!("{sitewide}{head_extra}");
    m.insert("head_extra".into(), json!(crate::url_util::rewrite_own_origin(
        &head_extra,
        site_url,
        &conn
            .query_row("SELECT value FROM settings WHERE key='site_url'", [], |r| r.get::<_, String>(0))
            .unwrap_or_default(),
    )));
    // Widget registry: the four traveltour-footer-column contents come from
    // the footer-{1..4}-{lang} sidebars; base_fr/base_es keep the column
    // wrapper divs.
    for n in 1..=4 {
        let sidebar = format!("footer-{n}-{lang}");
        let html = crate::widgets::render_sidebar(conn, &sidebar, site_url, lang, slug, tour_base)?;
        m.insert(format!("footer_{n}"), json!(html));
    }
    Ok(m)
}

