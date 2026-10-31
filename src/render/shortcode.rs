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
/// `[caption]…[/caption]` (WP figure markup), `[gdlr_core_icon]`,
/// `[metaslider id=N]` (MetaSlider flex output; needs `conn` to reach the
/// imported `ml-slider`/`ml-slide` rows — left verbatim without it).
/// Unknown tags are left verbatim (WP shows them as text too).
/// `post_id` = the enclosing post for `[site_reviews_form]`'s hidden
/// `_post_id` field — `None` for chrome/term contexts (form posts land on
/// the `_page` fallback in intake).
pub(crate) fn expand_shortcodes(s: &str, site_url: &str, conn: Option<&Connection>, post_id: Option<i64>) -> String {
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
                        let html = include_str!("../../templates/theme/traveltour/site_reviews_static.html") // gate-allow: reference-theme static capture
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
                            .replace("{{ post_id }}", &post_id.map(|p| p.to_string()).unwrap_or_default())
                            .replace("{{ site_url }}", site_url);
                        out.extend_from_slice(html.as_bytes());
                        i += end + 1;
                        continue;
                    }
                    "contact-form-7" => {
                        let a = sc_attrs(head);
                        let id = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        // CF7 ≥5.9 ids are the first 7 chars of the form's
                        // `_hash` meta; older dumps may carry numeric wp_ids.
                        // `cf7hash:<id>` → forms.slug (seeded at import).
                        let slug = conn.and_then(|c| c.query_row(
                            sql!("intake__select_setting"),
                            [format!("cf7hash:{id}")], |r| r.get::<_, String>(0)).ok())
                            .unwrap_or_else(|| id.to_string());
                        let markup = conn.and_then(|c| c.query_row(
                            sql!("shortcode__select_form_markup"),
                            [&slug], |r| r.get::<_, String>(0)).ok());
                        if let Some(m) = markup {
                            let html = crate::forms_markup::form_html(&m, &slug, site_url);
                            out.extend_from_slice(html.as_bytes());
                            i += end + 1;
                        }
                        // Unknown/resolved-miss: CF7 renders nothing for
                        // an unknown form — consume the tag rather than
                        // leaking `[contact-form-7 …]` verbatim.
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
                    "metaslider" => {
                        let a = sc_attrs(head);
                        let id = a.get("id").and_then(|v| {
                            v.as_str().and_then(|s| s.trim_matches('"').parse::<i64>().ok())
                                .or_else(|| v.as_i64())
                        });
                        if let (Some(id), Some(c)) = (id, conn) {
                            if let Some(html) = metaslider_html(c, id, site_url) {
                                out.extend_from_slice(html.as_bytes());
                                i += end + 1;
                                continue;
                            }
                        }
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

/// `[metaslider id=N]` → the plugin's FlexSlider front markup. Slides are
/// `ml-slide` posts (status publish) tied to the `ml-slider` post through
/// the `ml-slider` taxonomy (term slug = slider id), ordered by menu_order
/// — exactly WP_Query's ordering for the slider's slide query. `None` when
/// the slider isn't imported or has no live slides (shortcode left
/// verbatim, same as WP's "slideshow not found" no-op).
fn metaslider_html(conn: &Connection, id: i64, site_url: &str) -> Option<String> {
    let (title, settings_raw, theme) = conn
        .query_row(
            sql!("shortcode__select_slider"),
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .ok()?;
    let settings = settings_raw
        .as_deref()
        .and_then(|raw| crate::php_serialize::unserialize(raw.as_bytes()).ok());
    let sget = |k: &str| -> String {
        settings
            .as_ref()
            .and_then(|v| v.get(k))
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            .unwrap_or_default()
    };
    // Slide query: WP_Query(post_type=ml-slide, tax=ml-slider:{id},
    // orderby=menu_order). _thumbnail_id is the slide's image attachment.
    let mut stmt = conn
        .prepare(
            sql!("shortcode__select_slides"),
        )
        .ok()?;
    let rows = stmt
        .query_map([id.to_string()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })
        .ok()?;
    let slides: Vec<(i64, String, Option<i64>)> = rows
        .flatten()
        .map(|(sid, date, t)| (sid, date, t.and_then(|s| s.parse::<i64>().ok())))
        .collect();
    if slides.is_empty() {
        return None;
    }
    let sw = sget("width");
    let sh = sget("height");
    let theme_cls = theme
        .filter(|t| !t.is_empty() && t != "none")
        .map(|t| format!("ms-theme-{}", t))
        .unwrap_or_else(|| "ms-theme-default".to_string());
    let carousel = sget("carouselMode") == "true";
    // Sliders saved under the newer MetaSlider settings schema (the
    // mobileNavigation_*/ariaLive keys exist) emit nav-hidden twice when
    // navigation is off; the legacy 39-key schema emits it once.
    let new_schema = settings
        .as_ref()
        .and_then(|v| v.get("mobileNavigation_desktop"))
        .is_some();
    let nav_hidden = if sget("navigation") == "false" { " nav-hidden" } else { "" };
    let carousel_cls = if carousel { " has-carousel-mode" } else { "" };
    let nav_hidden2 = if new_schema && !nav_hidden.is_empty() { " nav-hidden" } else { "" };
    let css_class = sget("cssClass");
    let extra_cls = if css_class.is_empty() { String::new() } else { format!(" {}", esc_attr(&css_class)) };
    let site = site_url.trim_end_matches('/');

    // smartCrop truthy → MetaSlider's own meta-slider-resized-WxH crop img;
    // "disabled" → WP wp_get_attachment_image markup (srcset/sizes/etc) at
    // full size. Both observed on the origin (27875 smart, 31605 disabled).
    let smart_crop = {
        let v = sget("smartCrop");
        v == "true" || v == "1"
    };
    // WP wp_get_attachment_image context flags: the first 3 content images
    // skip lazy loading (omit_lazy_loading_threshold(3)) and the very first
    // gets fetchpriority="high"; images after that get loading="lazy".
    // Per-slider approximation — the slider is the page's first content on
    // every observed usage.
    let fetch_attr = |n: usize| -> &str {
        if n == 0 { " fetchpriority=\"high\"" } else if n >= 3 { " loading=\"lazy\"" } else { "" }
    };
    let mut lis: Vec<String> = Vec::new();
    for (slide_id, date, thumb) in &slides {
        let media = thumb.and_then(|t| {
            conn.query_row(
                sql!("shortcode__select_media_row"),
                [t],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, i64>(5)?,
                        r.get::<_, i64>(6)?,
                    ))
                },
            )
            .ok()
        });
        let Some((path, alt, mtitle, sizes_json, caption, mw, mh)) = media else {
            continue; // slide image not imported — WP emits nothing either
        };
        if path.is_empty() {
            continue;
        }
        let dir = path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
        if smart_crop {
            // meta-slider-resized-WxH is the plugin's on-the-fly crop
            // registered from the slider's width/height.
            let size_key = format!("meta-slider-resized-{sw}x{sh}");
            let file = serde_json::from_str::<serde_json::Value>(&sizes_json)
                .ok()
                .and_then(|v| {
                    v.get(&size_key)
                        .and_then(|sz| sz.get("file"))
                        .and_then(|f| f.as_str())
                        .map(|f| f.to_string())
                });
            let src = match &file {
                Some(f) => format!("{dir}/{f}"),
                None => path.clone(),
            };
            let filename = src.rsplit('/').next().unwrap_or("");
            let cap = if caption.is_empty() {
                String::new()
            } else {
                format!("<div class=\"caption-wrap\"><div class=\"caption\">{caption}</div></div>")
            };
            lis.push(format!(
                "<li style=\"display: none; width: 100%;\" class=\"slide-{slide_id} ms-image \" aria-roledescription=\"slide\" data-date=\"{}\" data-filename=\"{}\" data-slide-type=\"image\"><img src=\"{}/wp-content/uploads/{}\" height=\"{}\" width=\"{}\" alt=\"{}\" class=\"slider-{id} slide-{slide_id} msDefaultImage\" title=\"{}\" />{cap}</li>",
                esc_attr(date),
                esc_attr(filename),
                site,
                esc_attr(&src),
                esc_attr(&sh),
                esc_attr(&sw),
                esc_attr(&alt),
                esc_attr(&mtitle),
            ));
        } else {
            // WP wp_get_attachment_image: full-size src, ratio-matched srcset
            // (WP_TOLERANCE, not the strict 1% media_img_attrs uses for
            // figures — the observed srcset keeps medium 300x84 on 694x194).
            let filename = path.rsplit('/').next().unwrap_or("");
            let mut entries: Vec<(String, i64)> = Vec::new();
            if mw > 0 {
                entries.push((format!("{site}/wp-content/uploads/{path}"), mw));
            }
            if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&sizes_json) {
                for sz in map.values() {
                    let f = sz.get("file").and_then(|v| v.as_str()).unwrap_or("");
                    let (sw2, sh2) = (
                        sz.get("width").and_then(|v| v.as_i64()).unwrap_or(0),
                        sz.get("height").and_then(|v| v.as_i64()).unwrap_or(0),
                    );
                    if f.is_empty() || sw2 <= 0 || sh2 <= 0 || f.contains("..") || f.contains('/') || mw <= 0 || mh <= 0 {
                        continue;
                    }
                    if ((sw2 as f64 * mh as f64 - mw as f64 * sh2 as f64).abs() / (mw as f64 * sh2 as f64)) > 0.002 {
                        continue;
                    }
                    entries.push((format!("{site}/wp-content/uploads/{dir}/{f}"), sw2));
                }
            }
            entries.sort_by(|a, b| b.1.cmp(&a.1)); // WP srcset desc
            let fetch = fetch_attr(lis.len());
            let extra = if entries.is_empty() {
                String::new()
            } else {
                let srcset = entries
                    .iter()
                    .map(|(u, w)| format!("{} {}w", esc_attr(u), w))
                    .collect::<Vec<_>>()
                    .join(", ");
                // WP 6.7 auto-sizes: "auto, " prefix only on lazy images.
                let auto = if fetch.contains("lazy") { "auto, " } else { "" };
                format!(" srcset=\"{srcset}\" sizes=\"{auto}(max-width: {mw}px) 100vw, {mw}px\"")
            };
            let cap = if caption.is_empty() {
                String::new()
            } else {
                format!("<div class=\"caption-wrap\"><div class=\"caption\">{caption}</div></div>")
            };
            lis.push(format!(
                "<li style=\"display: none; width: 100%;\" class=\"slide-{slide_id} ms-image \" aria-roledescription=\"slide\" data-date=\"{}\" data-filename=\"{}\" data-slide-type=\"image\"><img width=\"{}\" height=\"{}\" src=\"{}/wp-content/uploads/{}\" class=\"slider-{id} slide-{slide_id} msDefaultImage\" alt=\"{}\" rel=\"\" title=\"{}\" decoding=\"async\"{}{} />{cap}</li>",
                esc_attr(date),
                esc_attr(filename),
                mw,
                mh,
                site,
                esc_attr(&path),
                esc_attr(&alt),
                esc_attr(&mtitle),
                fetch,
                extra,
            ));
        }
    }
    if lis.is_empty() {
        return None;
    }
    Some(format!(
        "<div id=\"metaslider-id-{id}\" style=\"width: 100%; margin: 0 auto;\" class=\"ml-slider-3-113-1 metaslider metaslider-flex metaslider-{id} ml-slider{carousel_cls} {theme_cls}{nav_hidden}{nav_hidden2}{extra_cls}\" role=\"region\" aria-label=\"{}\" data-height=\"{}\" data-width=\"{}\">\n    <div id=\"metaslider_container_{id}\">\n        <div id=\"metaslider_{id}\">\n            <ul class='slides'>\n                {}\n            </ul>\n        </div>\n        \n    </div>\n</div>{assets}",
        esc_attr(&title),
        esc_attr(&sh),
        esc_attr(&sw),
        lis.join("\n                "),
        assets = metaslider_assets(id, site, &sget),
    ))
}

/// Enqueue + init block the plugin prints via `wp_enqueue_script` +
/// `wp_add_inline_script` (WP hardcodes `metaslider-flex-slider-js-after`
/// per slider). We have no enqueue pipeline — the shortcode emits the tags
/// inline right after the markup, which is equivalent since jQuery is
/// already loaded in <head>.
fn metaslider_assets(id: i64, site: &str, sget: &dyn Fn(&str) -> String) -> String {
    let a = "/wp-content/plugins/ml-slider/assets";
    let bool_ = |v: &str| -> &'static str { if v == "true" { "true" } else { "false" } };
    // WP option → flexslider option (flex type only; other slider types
    // never reach here — slides query is flex-only).
    let navigation = bool_(&sget("navigation"));
    let (control_nav, direction_nav) = if navigation == "false" {
        ("true", "false") // plugin hides the filmstrip, keeps direction nav
    } else {
        ("false", "true")
    };
    let carousel = sget("carouselMode") == "true";
    let carousel_opts = if carousel {
        format!(
            ",\n                itemWidth:{},\n                minItems:2,\n                maxItems:0,\n                move:1,\n                itemMargin:{}",
            sget("width"),
            sget("carouselMargin")
        )
    } else {
        String::new()
    };
    format!(
        "<link rel='stylesheet' id='metaslider-flex-slider-css' href='{site}{a}/sliders/flexslider/flexslider.css?ver=3.113.1' media='all' property='stylesheet' />\n\
         <script id=\"metaslider-public-js\" src=\"{site}{a}/metaslider/script.min.js?ver=3.113.1\"></script>\n\
         <script id=\"metaslider-easing-js\" src=\"{site}{a}/easing/jQuery.easing.min.js?ver=3.113.1\"></script>\n\
         <script id=\"metaslider-flex-slider-js\" src=\"{site}{a}/sliders/flexslider/jquery.flexslider.min.js?ver=3.113.1\"></script>\n\
         <script id=\"metaslider-flex-slider-js-after\">\n\
         var metaslider_{id} = function($) {{$('#metaslider_{id}').addClass('flexslider');\n\
         \x20   $('#metaslider_{id}').flexslider({{\n\
         \x20       slideshowSpeed:{delay},\n\
         \x20       animation:'{anim}',\n\
         \x20       controlNav:{control_nav},\n\
         \x20       directionNav:{direction_nav},\n\
         \x20       pauseOnHover:{hover},\n\
         \x20       direction:'{dir}',\n\
         \x20       reverse:{rev},\n\
         \x20       keyboard:{kbd},\n\
         \x20       touch:{touch},\n\
         \x20       animationSpeed:{speed},\n\
         \x20       prevText:\"Previous\",\n\
         \x20       nextText:\"Next\",\n\
         \x20       smoothHeight:{smooth},\n\
         \x20       fadeFirstSlide:{fade},\n\
         \x20       easing:\"{easing}\",\n\
         \x20       slideshow:{auto},\n\
         \x20       pausePlay:false{carousel}\n\
         \x20   }});\n\
         \x20   $(document).trigger('metaslider/initialized', '#metaslider_{id}');\n\
         }};\n\
         var timer_metaslider_{id} = function() {{\n\
         \x20   var slider = !window.jQuery ? window.setTimeout(timer_metaslider_{id}, 100) : !jQuery.isReady ? window.setTimeout(timer_metaslider_{id}, 1) : metaslider_{id}(window.jQuery);\n\
         }};\n\
         timer_metaslider_{id}();\n\
         </script>",
        delay = sget("delay"),
        anim = if carousel { "slide" } else { "fade" },
        hover = bool_(&sget("hoverPause")),
        dir = sget("direction"),
        rev = bool_(&sget("reverse")),
        kbd = bool_(&sget("keyboard")),
        touch = bool_(&sget("touch")),
        speed = sget("animationSpeed"),
        smooth = bool_(&sget("smoothHeight")),
        fade = bool_(&sget("firstSlideFadeIn")),
        easing = sget("easing"),
        auto = bool_(&sget("autoPlay")),
        carousel = carousel_opts,
    )
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

use crate::html_decode::{esc, esc_attr};

/// Latest published posts in `lang`, optionally narrowed to the element's
/// category/tag slugs (WP `tax_query field=slug`); empty match → latest posts.
/// Returns HTML for `blog.html`'s `{{ children|safe }}`.
pub(crate) fn blog_posts(
    conn: &Connection,
    opts: &Map<String, Value>,
    lang: &str,
    _media: &HashMap<i64, (String, String)>,
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
            sql!("shortcode__select_media_size"),
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
        sql!("shortcode__select_media_info"),
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
    let base = sql!("shortcode__select_blog_base");
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


