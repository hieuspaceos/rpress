//! `gdlr_core_content_filter` equivalent: shortcode expansion + the media
//! <img> attribute emitter it shells out to (moved out of render.rs — pure
//! move, no behavior change).

#[allow(unused_imports)]
use super::*;

/* ================= gdlr_core_content_filter equivalent ================= */

/// `[tag attr="v" attr2='v']` attribute parser for the shortcode subset.
fn sc_attrs(s: &str) -> Map<String, Value> {
    let mut m = Map::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // key=
        let key_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
            i += 1;
        }
        if i == key_start {
            i += 1;
            continue;
        }
        let key = s[key_start..i].to_string();
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let vstart = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                m.insert(key, json!(s[vstart..i]));
                i += 1;
            } else {
                let vstart = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b']' {
                    i += 1;
                }
                m.insert(key, json!(s[vstart..i]));
            }
        }
    }
    m
}

/// Expand the shortcodes this site's text-boxes actually contain:
/// `[gdlr_core_button]`, `[embed]url[/embed]` (YouTube/Vimeo → iframe),
/// `[caption]…[/caption]` (WP figure markup), `[gdlr_core_icon]`.
/// Unknown tags are left verbatim (WP shows them as text too).
pub(crate) fn expand_shortcodes(s: &str, site_url: &str) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(s.len() + 256);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            // find matching ] for the tag header
            if let Some(end) = s[i..].find(']') {
                let head = &s[i + 1..i + end];
                let tag = head.split_whitespace().next().unwrap_or("");
                match tag {
                    "site_reviews" => {
                        let html = include_str!("../../templates/elements/site_reviews_static.html")
                            .replace("{{ site_url }}", site_url);
                        out.extend_from_slice(html.as_bytes());
                        i += end + 1;
                        continue;
                    }
                    "site_reviews_form" => {
                        let a = sc_attrs(head);
                        let id = a.get("id").and_then(|v| v.as_str()).unwrap_or("lte68mso");
                        let html = include_str!("../../templates/elements/site_reviews_form.html")
                            .replace("{{ form_id }}", id)
                            .replace("{{ post_id }}", "6190")
                            .replace("{{ site_url }}", site_url);
                        out.extend_from_slice(html.as_bytes());
                        i += end + 1;
                        continue;
                    }
                    "contact-form-7" => {
                        let html = include_str!("../../templates/elements/cf7_78de6b9.html")
                            .replace("{{ site_url }}", site_url);
                        out.extend_from_slice(html.as_bytes());
                        i += end + 1;
                        continue;
                    }
                    "gdlr_core_button" => {
                        let a = sc_attrs(head);
                        let gs = |k: &str| a.get(k).and_then(|v| v.as_str()).unwrap_or("");
                        let text = gs("button-text");
                        let link = gs("button-link");
                        let target = gs("button-link-target");
                        let mut style = String::new();
                        for (k, css) in [
                            ("margin-right", "margin-right"),
                            ("margin-left", "margin-left"),
                            ("margin-bottom", "margin-bottom"),
                            ("padding", "padding"),
                        ] {
                            let v = gs(k);
                            if !v.is_empty() && v != "0px" {
                                style.push_str(&format!("{css}: {v};"));
                            }
                        }
                        let tgt = if target.is_empty() || target == "_self" {
                            String::new()
                        } else {
                            format!(" target=\"{}\"", esc_attr(target))
                        };
                        let st = if style.is_empty() { String::new() } else { format!(" style=\"{style}\"") };
                        out.extend_from_slice(format!(
                            "<a class=\"gdlr-core-button gdlr-core-button-shortcode  gdlr-core-button-gradient gdlr-core-button-no-border\" href=\"{}\"{tgt} {st}  ><span class=\"gdlr-core-content\"  >{}</span></a>",
                            esc_attr(link),
                            esc(text)
                        ).as_bytes());
                        i += end + 1;
                        continue;
                    }
                    "embed" => {
                        if let Some(close) = s[i + end + 1..].find("[/embed]") {
                            let inner = &s[i + end + 1..i + end + 1 + close];
                            out.extend_from_slice(embed_html(inner, site_url).as_bytes());
                            i += end + 1 + close + "[/embed]".len();
                            continue;
                        }
                    }
                    "caption" => {
                        if let Some(close) = s[i + end + 1..].find("[/caption]") {
                            let inner = &s[i + end + 1..i + end + 1 + close];
                            // inner: <img …/> followed by the caption text
                            let img_end = inner.find("/>").map(|p| p + 2).unwrap_or(0);
                            let img = &inner[..img_end];
                            let cap = inner[img_end..].trim();
                            let a = sc_attrs(head);
                            let gs = |k: &str| a.get(k).and_then(|v| v.as_str()).unwrap_or("");
                            let att_id = gs("id");
                            let align = {
                                let al = gs("align");
                                if al.is_empty() { "alignnone" } else { al }
                            };
                            let width = gs("width");
                            // img_caption_shortcode: figure + figcaption
                            let wstyle = if width.is_empty() {
                                String::new()
                            } else {
                                format!(" style=\"width: {}px\"", esc_attr(width))
                            };
                            let att_id = esc_attr(att_id);
                            let align = esc_attr(align);
                            out.extend_from_slice(format!(
                                "<figure id=\"{att_id}\" aria-describedby=\"caption-{att_id}\"{wstyle} class=\"wp-caption {align}\">{img}<figcaption id=\"caption-{att_id}\" class=\"wp-caption-text\">{cap}</figcaption></figure>",
                            ).as_bytes());
                            i += end + 1 + close + "[/caption]".len();
                            continue;
                        }
                    }
                    "gdlr_core_icon" => {
                        let a = sc_attrs(head);
                        let gs = |k: &str| a.get(k).and_then(|v| v.as_str()).unwrap_or("");
                        let icon = gs("icon");
                        let mut style = String::new();
                        for (k, css) in [("color", "color"), ("size", "font-size"), ("margin-left", "margin-left"), ("margin-right", "margin-right")] {
                            let v = gs(k);
                            if !v.is_empty() {
                                style.push_str(&format!("{css}: {v};"));
                            }
                        }
                        out.extend_from_slice(format!(
                            "<i class=\"{}\" style=\"{}\" ></i>",
                            esc_attr(icon),
                            esc_attr(&style)
                        ).as_bytes());
                        i += end + 1;
                        continue;
                    }
                    _ => {}
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `[embed]` — YouTube/Vimeo/Spotify URL → iframe, matching wp_oembed output
/// shape (1150×647 like the reference homepage).
fn embed_html(url: &str, _site_url: &str) -> String {
    let u = url.trim();
    let video_id = if let Some(p) = u.split("v=").nth(1) {
        Some(p.split('&').next().unwrap_or(""))
    } else if u.contains("youtu.be/") {
        u.split("youtu.be/").nth(1).map(|x| x.split('?').next().unwrap_or(""))
    } else {
        None
    };
    if let Some(id) = video_id {
        if !id.is_empty() {
            let t = u.split("t=").nth(1).and_then(|x| x.trim_end_matches('s').split('&').next()).and_then(|x| x.parse::<i64>().ok()).unwrap_or(0);
            let start = if t > 0 { format!("?start={t}&amp;feature=oembed") } else { "?feature=oembed".into() };
            return format!(
                "<iframe title=\"YouTube video\" width=\"1150\" height=\"647\" src=\"https://www.youtube.com/embed/{id}{start}\" frameborder=\"0\" allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share\" referrerpolicy=\"strict-origin-when-cross-origin\" allowfullscreen></iframe>",
            );
        }
    }
    u.to_string()
}

pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
pub(crate) fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}

/// Latest published posts in `lang`, optionally narrowed to the element's
/// category/tag slugs (WP `tax_query field=slug`); empty match → latest posts.
/// Returns HTML for `blog.html`'s `{{ children|safe }}`.
pub(crate) fn blog_posts(
    conn: &Connection,
    opts: &Map<String, Value>,
    lang: &str,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    tour_base: &str,
) -> Result<String> {
    let num: i64 = opts
        .get("num-fetch")
        .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_i64()))
        .unwrap_or(10)
        .clamp(1, 24);
    let cats: Vec<String> = opts
        .get("category")
        .map(|v| match v {
            Value::Array(a) => a.iter().filter_map(|x| x.as_str().map(String::from)).collect(),
            Value::String(s) => s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect(),
            _ => vec![],
        })
        .unwrap_or_default();
    let mut rows = query_blog(conn, &cats, lang, num).unwrap_or_default();
    if rows.is_empty() {
        rows = query_blog(conn, &[], lang, num).unwrap_or_default();
    }
    let bstyle = opts
        .get("blog-style")
        .and_then(|v| v.as_str())
        .unwrap_or("blog-column-with-frame");
    if bstyle == "blog-column" {
        // Non-carousel column grid (pb-element-blog.php blog-column branch):
        // item-list > blog-grid > thumbnail + grid-content-wrap; cards carry
        // no frame and the content is empty when excerpt-number is 0.
        let col_size: i64 = opts
            .get("column-size")
            .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_i64()))
            .unwrap_or(60)
            .clamp(1, 60);
        let ncols = (60 / col_size).max(1);
        let grid_style = opts
            .get("blog-column-style")
            .and_then(|v| v.as_str())
            .unwrap_or("style-1");
        let title_size = opts
            .get("blog-title-font-size")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let title_fstyle = opts
            .get("blog-title-font-style")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let excerpt_n: usize = opts
            .get("excerpt-number")
            .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_i64()))
            .unwrap_or(0)
            .max(0) as usize;
        let mut out = String::new();
        for (i, (_id, slug, title, excerpt, feat_id, _published)) in rows.iter().enumerate() {
            let url = page_url(conn, site_url.trim_end_matches('/'), lang, "post", slug, tour_base);
            let img = feat_id
                .and_then(|fid| media_info(conn, fid).map(|d| (fid, d)))
                .map(|(fid, d)| {
                    let p = d.get("path").and_then(|x| x.as_str()).unwrap_or("");
                    let alt = d.get("alt").and_then(|x| x.as_str()).unwrap_or("");
                    let w = d.get("w").and_then(|x| x.as_i64()).unwrap_or(0);
                    let h = d.get("h").and_then(|x| x.as_i64()).unwrap_or(0);
                    format!(
                        "<img src=\"{}/wp-content/uploads/{}\" alt=\"{}\"{} title=\"{}\"{} />",
                        site_url.trim_end_matches('/'),
                        esc_attr(p),
                        esc_attr(alt),
                        if w > 0 { format!(" width=\"{w}\" height=\"{h}\"") } else { String::new() },
                        esc_attr(&wp_title(alt)),
                        media_img_attrs(conn, fid, site_url),
                    )
                })
                .unwrap_or_default();
            let exc = if excerpt_n == 0 {
                String::new()
            } else {
                let t = excerpt.split_whitespace().take(excerpt_n).collect::<Vec<_>>().join(" ");
                if t.is_empty() { String::new() } else { format!("{t}...") }
            };
            let mut title_style = String::new();
            if !title_size.is_empty() {
                title_style.push_str(&format!("font-size: {title_size} ;"));
            }
            if !title_fstyle.is_empty() {
                title_style.push_str(&format!("font-style: {title_fstyle} ;"));
            }
            out.push_str(&format!(
                "<div class=\"gdlr-core-item-list  gdlr-core-item-pdlr gdlr-core-column-{col_size}{}\" >\
<div class=\"gdlr-core-blog-grid gdlr-core-style-{grid_style} \"  >\
<div class=\"gdlr-core-blog-thumbnail gdlr-core-media-image  gdlr-core-opacity-on-hover gdlr-core-zoom-on-hover\"  ><a href=\"{url}\" >{img}</a></div>\
<div class=\"gdlr-core-blog-grid-content-wrap\">\
<h3 class=\"gdlr-core-blog-title gdlr-core-skin-title\" style=\"{title_style}\"  ><a href=\"{url}\" >{}</a></h3>\
<div class=\"gdlr-core-blog-content clearfix\" >{exc}</div></div></div></div>",
                if i as i64 % ncols == 0 { " gdlr-core-column-first" } else { "" },
                esc(title)
            ));
        }
        return Ok(out);
    }
    // pb-element-blog.php (blog-column-with-frame): cards ride a flexslider
    // carousel — <ul class="slides"><li> per post.
    let mut out = String::from(
        "<div class=\"gdlr-core-flexslider flexslider gdlr-core-js-2\" data-type=\"carousel\" data-column=\"4\" data-move=\"3\" data-nav=\"none\" data-nav-parent=\"gdlr-core-blog-item\" ><ul class=\"slides\" >",
    );
    for (id, slug, title, excerpt, feat_id, _published) in rows {
        let url = page_url(conn, site_url.trim_end_matches('/'), lang, "post", &slug, tour_base);
        let img = feat_id
            .and_then(|fid| media.get(&fid).map(|(p, alt)| (fid, p.clone(), alt.clone())))
            .map(|(fid, p, alt)| {
                format!(
                    "<img src=\"{}/wp-content/uploads/{}\" alt=\"{}\"{} />",
                    site_url.trim_end_matches('/'),
                    esc_attr(&p),
                    esc_attr(&alt),
                    media_img_attrs(conn, fid, site_url),
                )
            })
            .unwrap_or_default();
        let exc = excerpt.chars().take(140).collect::<String>();
        let exc = if exc.is_empty() { String::new() } else { format!("{exc}...") };
        out.push_str(&format!(
            "<li class=\"gdlr-core-item-mglr\" ><div class=\"gdlr-core-blog-grid gdlr-core-style-2 gdlr-core-js gdlr-core-blog-grid-with-frame gdlr-core-item-mgb gdlr-core-skin-e-background\" data-sync-height=\"blog-item-{id}\" >\
<div class=\"gdlr-core-blog-thumbnail gdlr-core-media-image gdlr-core-opacity-on-hover gdlr-core-zoom-on-hover\" ><a href=\"{url}\" >{img}</a></div>\
<div class=\"gdlr-core-blog-grid-frame gdlr-core-sync-height-space-position\" >\
<h3 class=\"gdlr-core-blog-title gdlr-core-skin-title\" style=\"font-size: 17px ;font-style: normal ;\" ><a href=\"{url}\" >{}</a></h3>\
<div class=\"gdlr-core-blog-content clearfix\" >{exc}<div class=\"clear\"></div></div></div></div></li>",
            esc(&title)
        ));
    }
    out.push_str("</ul></div>");
    Ok(out)
}
/// WP `wp_get_attachment_image_srcset`+`sizes` as a ready-made attribute
/// fragment (` srcset="…" sizes="…"`, leading space; empty string when the
/// attachment has no usable sizes). Candidate sizes follow WP's rules:
/// aspect ratio within 1% of the full image — this filters out the cropped
/// `thumbnail` automatically, as WP does.
pub(crate) fn media_img_attrs(conn: &Connection, id: i64, site_url: &str) -> String {
    let row = conn
        .query_row(
            "SELECT path, COALESCE(sizes_json,''), COALESCE(w,0), COALESCE(h,0) FROM media WHERE id=?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            },
        )
        .ok();
    let Some((path, sizes_json, w, h)) = row else { return String::new() };
    if w <= 0 || h <= 0 {
        return String::new();
    }
    let site = site_url.trim_end_matches('/');
    let dir = path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let ratio = w as f64 / h as f64;
    // (url, width) candidates, ascending — WP sorts srcset by width.
    let mut entries: Vec<(String, i64)> = Vec::new();
    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&sizes_json) {
        for sz in map.values() {
            let f = sz.get("file").and_then(|v| v.as_str()).unwrap_or("");
            let (sw, sh) = (
                sz.get("width").and_then(|v| v.as_i64()).unwrap_or(0),
                sz.get("height").and_then(|v| v.as_i64()).unwrap_or(0),
            );
            if f.is_empty() || sw <= 0 || sh <= 0 || f.contains("..") || f.contains('/') {
                continue;
            }
            if ((sw as f64 / sh as f64) - ratio).abs() / ratio > 0.01 {
                continue;
            }
            entries.push((format!("{site}/wp-content/uploads/{dir}/{f}"), sw));
        }
    }
    if entries.is_empty() {
        return String::new();
    }
    entries.push((format!("{site}/wp-content/uploads/{path}"), w));
    entries.sort_by_key(|(_, ew)| *ew);
    let srcset = entries
        .iter()
        .map(|(u, ew)| format!("{} {}w", esc_attr(u), ew))
        .collect::<Vec<_>>()
        .join(", ");
    format!(" srcset=\"{srcset}\" sizes=\"auto, (max-width: {w}px) 100vw, {w}px\"")
}

/// One media row as a JSON object: {path, alt, caption, title, w, h}.
/// (Distinct from Phase-2 `media_detail`, which returns the og:image tuple.)
pub(crate) fn media_info(conn: &Connection, id: i64) -> Option<Value> {
    conn.query_row(
        "SELECT path, COALESCE(alt,''), COALESCE(caption,''), COALESCE(title,''), \
         COALESCE(w,0), COALESCE(h,0) FROM media WHERE id=?1",
        [id],
        |r| {
            Ok(json!({
                "path": r.get::<_, String>(0)?,
                "alt": r.get::<_, String>(1)?,
                "caption": r.get::<_, String>(2)?,
                "title": r.get::<_, String>(3)?,
                "w": r.get::<_, i64>(4)?,
                "h": r.get::<_, i64>(5)?,
            }))
        },
    )
    .ok()
}

/// WP wptexturize subset for attribute text: ' -> &#8217;, " - " -> &#8211;.
fn wp_title(s: &str) -> String {
    s.replace('\'', "&#8217;").replace(" - ", " &#8211; ").replace('—', "&#8212;")
}

fn query_blog(
    conn: &Connection,
    cats: &[String],
    lang: &str,
    num: i64,
) -> Result<Vec<(i64, String, String, String, Option<i64>, String)>> {
    let base = "SELECT p.id, p.slug, p.title, COALESCE(p.excerpt,''), p.featured_media_id, COALESCE(p.published_at,'') FROM posts p";
    let (sql, params): (String, Vec<String>) = if cats.is_empty() {
        (
            format!("{base} WHERE p.status='publish' AND p.lang=?1 AND p.kind='post' ORDER BY p.published_at DESC LIMIT ?2"),
            vec![lang.to_string(), num.to_string()],
        )
    } else {
        let ph = cats.iter().enumerate().map(|(i, _)| format!("?{}", i + 3)).collect::<Vec<_>>().join(",");
        (
            format!("{base} JOIN post_terms pt ON pt.post_id=p.id WHERE p.status='publish' AND p.lang=?1 AND p.kind='post' AND pt.taxonomy='category' AND pt.slug IN ({ph}) ORDER BY p.published_at DESC LIMIT ?2"),
            [vec![lang.to_string(), num.to_string()], cats.to_vec()].concat(),
        )
    };
    let mut stmt = conn.prepare(&sql)?;
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let rows = stmt.query_map(params_ref.as_slice(), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<i64>>(4)?,
            r.get::<_, String>(5)?,
        ))
    })?;
    Ok(rows.flatten().collect())
}


