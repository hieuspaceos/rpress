//! The body_json node-render engine: element dispatch, gdlr CSS helpers,
//! and the per-render template functions (moved out of render.rs — pure
//! move, no behavior change).

#[allow(unused_imports)]
use super::*;

// Per-render custom CSS collected from column/wrapper elements (the PHP
// page builder's `gdlr_core_insert_style` buffer). Flushed as one <style>
// block by the top-level render_nodes call — mirrors WP's aggregated head
// style tag instead of per-element inline <style>s.
thread_local! {
    static CUSTOM_STYLES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}
pub(crate) fn custom_styles_take() -> Vec<String> {
    CUSTOM_STYLES.with(|c| std::mem::take(&mut *c.borrow_mut()))
}
use std::sync::Arc;

pub(crate) const ELEMENT_TYPES: &[&str] = &[
    "accordion",
    "alert-box",
    "audio",
    "background",
    "blockquote",
    "blog",
    "button",
    "call-to-action",
    "chart",
    "code",
    "column",
    "column-service",
    "columnize",
    "content-navigation",
    "countdown",
    "counter",
    "custom-field",
    "custom-menu",
    "divider",
    "event",
    "feature-box",
    "feature-content",
    "featured-gallery",
    "flipbox",
    "food_menu",
    "gallery",
    "hover-box",
    "hover-content",
    "icon",
    "icon-list",
    "image",
    "image-content",
    "instagram",
    "opening-hours",
    "page-list",
    "port-info",
    "post-slider",
    "price-list",
    "price-plan",
    "price-table",
    "promo-box",
    "roadmap",
    "shape-divider",
    "skill-bar",
    "skill-circle",
    "social-network",
    "social-share",
    "space",
    "stunning-text",
    "tab",
    "tab-feature",
    "tab-feature-vertical",
    "tab-feature2",
    "tab-feature3",
    "testimonial",
    "text-box",
    "text-script",
    "timeline",
    "title",
    "toggle-box",
    "tour",
    "tour_search",
    "type-animation",
    "video",
    "widget",
    "wordpress-editor-content",
];


/// Register per-render template functions: media_url/media_alt/style/pad4.
pub(crate) fn register_helpers(
    env: &mut Environment,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
) {
    let media = Arc::new(media.clone());
    let site = Arc::new(site_url.to_string());
    // Reset the per-render custom-style buffer: a fresh Environment means a
    // fresh page render, so nothing collected by a previous render leaks in.
    let _ = custom_styles_take();
    {
        let media = media.clone();
        let site = site.clone();
        env.add_function("media_url", move |v: JVal| -> JVal {
            let out = val_to_i64(&v)
                .and_then(|i| media.get(&i))
                .map(|(p, _)| format!("{}/wp-content/uploads/{}", site, p))
                .unwrap_or_default();
            JVal::from(out)
        });
    }
    {
        let media = media.clone();
        env.add_function("media_alt", move |v: JVal| -> JVal {
            let out = val_to_i64(&v)
                .and_then(|i| media.get(&i))
                .map(|(_, a)| a.clone())
                .unwrap_or_default();
            JVal::from(out)
        });
    }
    // css_url(v): media id or URL -> url("…") suitable as a style() value.
    {
        let media = media.clone();
        let site = site.clone();
        env.add_function("css_url", move |v: JVal| -> String {
            let s = v.to_string();
            if s.is_empty() {
                return String::new();
            }
            let url = val_to_i64(&v)
                .and_then(|i| media.get(&i))
                .map(|(p, _)| format!("{}/wp-content/uploads/{}", site, p))
                .unwrap_or_else(|| s.clone());
            format!("url(\"{url}\")")
        });
    }
    // style('prop', value, 'prop2', value2, …) -> ' style="prop: v; prop2: v2;"'
    env.add_function("style", move |args: Rest<JVal>| -> String {
        let mut css = String::new();
        let mut i = 0;
        while i + 1 < args.len() {
            let prop = args[i].to_string();
            let val = css_value(&args[i + 1]);
            if !prop.is_empty() && !val.is_empty() {
                css.push_str(&format!("{}: {};", prop, val));
            }
            i += 2;
        }
        if css.is_empty() {
            String::new()
        } else {
            format!(" style=\"{css}\"")
        }
    });
    // pad4({"top":..,"right":..,"bottom":..,"left":..}) -> "t r b l" or ""
    env.add_function("pad4", move |v: JVal| -> String {
        let get = |k: &str| -> String {
            v.get_attr(k)
                .ok()
                .filter(|x| !x.is_undefined())
                .map(|x| x.to_string())
                .unwrap_or_default()
        };
        let t = get("top");
        let r = get("right");
        let b = get("bottom");
        let l = get("left");
        if [t.as_str(), r.as_str(), b.as_str(), l.as_str()]
            .iter()
            .all(|s| s.is_empty() || *s == "0px" || *s == "0")
        {
            String::new()
        } else {
            format!("{t} {r} {b} {l}")
        }
    });
    // uid('gdlr-core-column-') -> 'gdlr-core-column-1', sequential per prefix.
    // Mirrors PHP globals like $gdlr_core_column_id / $gdlr_core_wrapper_id so
    // generated wrapper ids stay deterministic in render order.
    {
        let counters = Arc::new(parking_lot::Mutex::new(HashMap::<String, u64>::new()));
        env.add_function("uid", move |v: JVal| -> String {
            let prefix = v.to_string();
            let mut m = counters.lock();
            let n = m.entry(prefix.clone()).or_insert(0);
            *n += 1;
            format!("{prefix}{n}")
        });
    }
    // custom_style(css): buffer element custom CSS (column/wrapper custom
    // styles) for the single aggregated <style> block — WP's
    // gdlr_core_insert_style behavior. Templates call it inside {% if %}; it
    // renders nothing inline.
    env.add_function("custom_style", move |v: JVal| -> String {
        let s = v.to_string();
        if !s.is_empty() {
            CUSTOM_STYLES.with(|c| c.borrow_mut().push(s));
        }
        String::new()
    });
    // gdlr_css('prop', value, ...) -> CSS declarations emulating PHP
    // gdlr_core_esc_style: 4-side {top,right,bottom,left,settings} maps emit
    // shorthand or per-side declarations; corner radius maps emit
    // (moz/webkit-prefixed) border-radius; [color,opacity] pairs emit rgba();
    // gradient props emit linear-gradient(). gdlr_css_imp appends !important.
    env.add_function("gdlr_css", move |args: Rest<JVal>| -> String {
        gdlr_css_impl(&args, false)
    });
    env.add_function("gdlr_css_imp", move |args: Rest<JVal>| -> String {
        gdlr_css_impl(&args, true)
    });
    // gdlr_rgba(color, opacity) -> 'rgba(r,g,b,a)' for hex colors; raw color
    // or '' otherwise.
    env.add_function("gdlr_rgba", move |c: JVal, o: JVal| -> String {
        let color = c.to_string();
        if color.is_empty() {
            return String::new();
        }
        let a: f64 = o.to_string().parse().unwrap_or(1.0);
        match parse_hex(&color) {
            Some((r, g, b)) => format!("rgba({r},{g},{b},{})", fmt_alpha(a)),
            None => color,
        }
    });
}

fn fmt_alpha(a: f64) -> String {
    if a.fract() == 0.0 {
        format!("{}", a as i64)
    } else {
        format!("{a}")
    }
}

fn parse_hex(color: &str) -> Option<(u8, u8, u8)> {
    let h = color.strip_prefix('#')?;
    let parse = |s: &str| u8::from_str_radix(s, 16).ok();
    match h.len() {
        6 => Some((parse(&h[0..2])?, parse(&h[2..4])?, parse(&h[4..6])?)),
        3 => Some((
            parse(&h[0..1].repeat(2))?,
            parse(&h[1..2].repeat(2))?,
            parse(&h[2..3].repeat(2))?,
        )),
        _ => None,
    }
}

fn s_empty(x: &Value) -> bool {
    match x {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        _ => false,
    }
}

fn s_str(x: &Value) -> String {
    match x {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

fn rgba_of(c: Option<&Value>, op: Option<&Value>) -> String {
    let color = c.map(s_str).unwrap_or_default();
    if color.is_empty() {
        return String::new();
    }
    let a: f64 = op.and_then(|v| s_str(v).parse().ok()).unwrap_or(1.0);
    match parse_hex(&color) {
        Some((r, g, b)) => format!("rgba({r},{g},{b},{})", fmt_alpha(a)),
        None => color,
    }
}

fn radius_shorthand(vals: &[String], bang: &str) -> String {
    let j = vals.join(" ");
    format!("border-radius: {j}{bang};-moz-border-radius: {j}{bang};-webkit-border-radius: {j}{bang};")
}

/// Emit CSS declaration(s) for one esc_style-style (prop, value) pair.
fn gdlr_decl(prop: &str, v: &Value, bang: &str) -> String {
    match v {
        Value::Object(m) => {
            let get = |k: &str| m.get(k).map(s_str).unwrap_or_default();
            let settings = get("settings");
            if prop == "border-radius" && m.contains_key("t-left") {
                // corner-map shape: t-left/t-right/b-right/b-left
                if settings == "link" {
                    let t = get("t-left");
                    if t.is_empty() {
                        return String::new();
                    }
                    return radius_shorthand(&[t.clone(), t.clone(), t.clone(), t], bang);
                }
                let cs: Vec<String> = ["t-left", "t-right", "b-right", "b-left"]
                    .iter()
                    .map(|k| get(k))
                    .collect();
                if cs.iter().all(|s| s.is_empty()) {
                    return String::new();
                }
                let vals: Vec<String> = cs
                    .iter()
                    .map(|s| if s.is_empty() { "0px".into() } else { s.clone() })
                    .collect();
                return radius_shorthand(&vals, bang);
            }
            let sides: Vec<String> = ["top", "right", "bottom", "left"].iter().map(|k| get(k)).collect();
            if sides.iter().all(|s| s.is_empty()) {
                return String::new();
            }
            if settings == "link" {
                let t = &sides[0];
                if t.is_empty() {
                    return String::new();
                }
                if prop == "border-radius" {
                    return radius_shorthand(&[t.clone(), t.clone(), t.clone(), t.clone()], bang);
                }
                return format!("{prop}: {t} {t} {t} {t}{bang};");
            }
            if sides.iter().all(|s| !s.is_empty()) {
                if prop == "border-radius" {
                    return radius_shorthand(&sides, bang);
                }
                return format!("{prop}: {}{bang};", sides.join(" "));
            }
            let mut out = String::new();
            for (k, val) in ["top", "right", "bottom", "left"].iter().zip(sides.iter()) {
                if !val.is_empty() {
                    out.push_str(&format!("{prop}-{k}: {val}{bang};"));
                }
            }
            out
        }
        Value::Array(a) => {
            if (prop == "gradient" || prop == "gradient-v") && a.iter().all(|x| x.is_array()) {
                let stops: Vec<String> = a.iter().map(|p| rgba_of(p.get(0), p.get(1))).collect();
                if stops.iter().all(|s| s.is_empty()) {
                    return String::new();
                }
                let deg = if prop == "gradient-v" { "180deg" } else { "90deg" };
                return format!("background: linear-gradient({deg},{}){bang};", stops.join(","));
            }
            if a.len() == 2 {
                let c = rgba_of(a.get(0), a.get(1));
                if c.is_empty() {
                    return String::new();
                }
                return format!("{prop}: {c}{bang};");
            }
            String::new()
        }
        // PHP esc_style prints scalar values with a trailing space ('prop: v ;')
        Value::String(s) => {
            if s.is_empty() {
                String::new()
            } else if bang.is_empty() {
                format!("{prop}: {s} ;")
            } else {
                format!("{prop}: {s}{bang};")
            }
        }
        Value::Number(n) => {
            if bang.is_empty() {
                format!("{prop}: {n} ;")
            } else {
                format!("{prop}: {n}{bang};")
            }
        }
        _ => String::new(),
    }
}

fn gdlr_css_impl(args: &[JVal], important: bool) -> String {
    let bang = if important { " !important" } else { "" };
    let mut css = String::new();
    let mut i = 0;
    while i + 1 < args.len() {
        let prop = args[i].to_string();
        let v = serde_json::to_value(&args[i + 1]).unwrap_or(Value::Null);
        if !s_empty(&v) {
            css.push_str(&gdlr_decl(&prop, &v, bang));
        }
        i += 2;
    }
    css
}

fn val_to_i64(v: &JVal) -> Option<i64> {
    if let Ok(n) = i64::try_from(v.clone()) {
        Some(n)
    } else {
        v.to_string().parse().ok()
    }
}

/// Stringify a style value; 4-side maps become "t r b l".
fn css_value(v: &JVal) -> String {
    if v.is_undefined() || v.is_none() {
        return String::new();
    }
    if v.as_object().is_some() {
        let mut parts = Vec::new();
        for k in ["top", "right", "bottom", "left"] {
            parts.push(
                v.get_attr(k)
                    .ok()
                    .filter(|x| !x.is_undefined())
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "0px".into()),
            );
        }
        return parts.join(" ");
    }
    v.to_string()
}


#[allow(dead_code)]
/// Render a body_json fragment (array or single node) to element HTML.
/// Used by both the static generator and the builder preview endpoint.
pub(crate) fn render_fragment(
    templates: &Path,
    v: &Value,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    tour_base: &str,
) -> Result<String> {
    let mut env = load_env(templates)?;
    register_helpers(&mut env, media, site_url);
    let site = JVal::from_serialize(json!({}));
    let post = JVal::from_serialize(json!({}));
    render_nodes(&mut env, v, media, site_url, &site, "fr", &post, None, false, tour_base)
}

/// render_fragment with a live DB so dynamic elements (blog, tour,
/// testimonial, tour_search, custom-menu…) get a populated `site` context.
pub(crate) fn render_fragment_db(
    conn: &Connection,
    templates: &Path,
    v: &Value,
    site_url: &str,
    lang: &str,
    tour_base: &str,
) -> Result<String> {
    let media = {
        let mut ids = std::collections::HashSet::new();
        collect_media_ids(v, &mut ids);
        media_map_for(conn, &ids)?
    };
    let mut env = load_env(templates)?;
    register_helpers(&mut env, &media, site_url);
    let site = site_data_cached(conn, lang, site_url, tour_base)?;
    let post = JVal::from_serialize(json!({}));
    render_nodes(&mut env, v, &media, site_url, &site, lang, &post, Some(conn), false, tour_base)
}

thread_local! {
    static RENDER_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// render_nodes wrapper: tracks recursion depth so collected element custom
/// styles are emitted once, as a single leading <style> block, by the
/// outermost call (WP prints them in <head>; we keep them ahead of the body
/// markup since this function only returns the body fragment).
pub(crate) fn render_nodes(
    env: &mut Environment,
    v: &Value,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    site: &JVal,
    lang: &str,
    post: &JVal,
    conn: Option<&Connection>,
    top_level: bool,
    tour_base: &str,
) -> Result<String> {
    RENDER_DEPTH.with(|d| d.set(d.get() + 1));
    let result = render_nodes_inner(env, v, media, site_url, site, lang, post, conn, top_level, tour_base);
    let depth = RENDER_DEPTH.with(|d| {
        d.set(d.get() - 1);
        d.get()
    });
    match result {
        Ok(html) if depth == 0 => {
            let styles = custom_styles_take();
            if styles.is_empty() {
                Ok(html)
            } else {
                Ok(format!("<style>{}</style>{html}", styles.concat()))
            }
        }
        r => r,
    }
}

fn render_nodes_inner(
    env: &mut Environment,
    v: &Value,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    site: &JVal,
    lang: &str,
    post: &JVal,
    conn: Option<&Connection>,
    top_level: bool,
    tour_base: &str,
) -> Result<String> {
    match v {
        Value::Array(a) => {
            let mut out = String::new();
            // PHP page-builder.php: column widths accumulate within an item
            // stream; a column is 'first' when it opens a 60-wide row.
            let mut col_count: i64 = 0;
            // Non-wrapper runs (leaf elements and column wrappers) are grouped
            // into <div class="gdlr-core-pbf-section"><div class="gdlr-core-
            // pbf-section-container gdlr-core-container clearfix">…</div></div>
            // exactly like PHP page-builder.php emits for sections.
            let mut section_open = false;
            let close_section = |out: &mut String, open: &mut bool| {
                if *open {
                    out.push_str("</div></div>");
                    *open = false;
                }
            };
            for x in a {
                match x {
                    Value::Object(m) => {
                        let typ = m.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        let is_wrapper = matches!(typ, "background" | "normal" | "sidebar");
                        if is_wrapper {
                            close_section(&mut out, &mut section_open);
                            out.push_str(&render_node(env, m, media, site_url, site, lang, post, false, conn, tour_base)?);
                            col_count = 0;
                            continue;
                        }
                        if top_level && !section_open {
                            out.push_str("<div class=\"gdlr-core-pbf-section\" ><div class=\"gdlr-core-pbf-section-container gdlr-core-container clearfix\" >");
                            section_open = true;
                        }
                        let mut first = false;
                        if typ == "column" || typ == "flexbox" {
                            let w: i64 = m
                                .get("column")
                                .and_then(|c| {
                                    c.as_str()
                                        .and_then(|s| s.parse().ok())
                                        .or_else(|| c.as_i64())
                                })
                                .unwrap_or(0);
                            if col_count == 0 {
                                first = true;
                                col_count = w;
                            } else {
                                col_count += w;
                                if col_count > 60 {
                                    first = true;
                                    col_count = w;
                                }
                            }
                        }
                        let html = render_node(env, m, media, site_url, site, lang, post, first, conn, tour_base)?;
                        // every leaf item rides in <div class="gdlr-core-pbf-element">;
                        // columns carry their own wrapper.
                        if typ == "column" || typ == "flexbox" {
                            out.push_str(&html);
                        } else {
                            out.push_str(&format!("<div class=\"gdlr-core-pbf-element\" >{html}</div>"));
                        }
                    }
                    _ => out.push_str(&render_nodes(env, x, media, site_url, site, lang, post, conn, false, tour_base)?),
                }
            }
            close_section(&mut out, &mut section_open);
            Ok(out)
        }
        Value::Object(m) => render_node(env, m, media, site_url, site, lang, post, false, conn, tour_base),
        _ => Ok(String::new()),
    }
}
fn render_node(
    env: &mut Environment,
    m: &Map<String, Value>,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    site: &JVal,
    lang: &str,
    post: &JVal,
    first_col: bool,
    conn: Option<&Connection>,
    tour_base: &str,
) -> Result<String> {
    let typ = m.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
    let children_val = m.get("items").cloned().unwrap_or(Value::Array(vec![]));
    let children = render_nodes(env, &children_val, media, site_url, site, lang, post, conn, false, tour_base)?;
    let mut opts = match m.get("value") {
        Some(Value::Object(o)) => o.clone(),
        _ => Map::new(),
    };
    if let Some(c) = m.get("column") {
        opts.insert("column".into(), c.clone());
    }
    if first_col {
        opts.insert("first-column".into(), json!(true));
    }
    let column = opts
        .get("column")
        .and_then(|c| c.as_str().map(|s| s.to_string()).or_else(|| c.as_i64().map(|n| n.to_string())))
        .unwrap_or_else(|| m.get("column").and_then(|c| c.as_str()).unwrap_or("60").to_string());

    if typ == "image" {
        if let Some(id) = opts.get("image").and_then(|x| match x {
            Value::Number(n) => n.as_i64(),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }) {
            if let Some((path, alt)) = media.get(&id) {
                opts.insert("src".into(), json!(format!("{site_url}/wp-content/uploads/{path}")));
                if opts.get("alt").and_then(|a| a.as_str()).unwrap_or("").is_empty() {
                    opts.insert("alt".into(), json!(alt));
                }
            }
        }
    }
    if typ == "text-box" {
        if let Some(Value::String(s)) = opts.get_mut("content") {
            // Saved via TinyMCE: content may be entity-encoded — WP decodes on
            // output; mirror that so `<h1>` isn't printed as `&lt;h1&gt;`.
            if s.contains("&lt;") || s.contains("&gt;") {
                *s = crate::admin::gdlr_pb::html_decode_pub(s);
            }
            // gdlr_core_content_filter → do_shortcode + oembed for the small
            // set this site actually uses.
            if s.contains('[') {
                *s = expand_shortcodes(s, site_url);
            }
            // gdlr_core_content_filter also unwraps <p> that encloses a
            // single block-level heading (TinyMCE artifact).
            for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
                for (pre, post) in [
                    (format!("<p><{tag} "), format!("<{tag} ")),
                    (format!("<p><{tag}>"), format!("<{tag}>")),
                ] {
                    if s.contains(&pre) {
                        // strip wrapper only when </hN></p> matches symmetric
                        let close = format!("</{tag}></p>");
                        if s.contains(&close) {
                            *s = s.replace(&pre, &post).replace(&close, &format!("</{tag}>"));
                        }
                    }
                }
            }
        }
    }
    if typ == "sidebar" {
        // Sidebar wrapper: 'sidebar' is positional (left|right|both); the
        // registered sidebar ids live in 'sidebar-left'/'sidebar-right'
        // comboboxes (wrappers.json). Each emitted side column renders
        // the assigned sidebar's widgets via the registry — custom
        // sidebars carry no wrapper (phase-3 decision, documented).
        let sidebar = {
            let v = opts
                .get("sidebar")
                .and_then(|v| v.as_str())
                .unwrap_or("right");
            if v.is_empty() { "right" } else { v }
        };
        let width: i64 = opts
            .get("sidebar-width")
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            .and_then(|s| s.trim_end_matches("px").parse().ok())
            .unwrap_or(20);
        let mut content_w = 60;
        if sidebar == "left" || sidebar == "right" {
            content_w -= width;
        }
        if sidebar == "both" {
            content_w -= 2 * width;
        }
        let mut o = format!(
            "<div class=\"gdlr-core-pbf-sidebar-container clearfix gdlr-core-line-height-0 gdlr-core-js\" >"
        );
        for side in ["left", "right"] {
            if sidebar != side && sidebar != "both" {
                continue;
            }
            let sb_id = opts
                .get(&format!("sidebar-{side}"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let sb_html = if sb_id.is_empty() {
                String::new()
            } else {
                conn.map(|c| {
                    crate::widgets::render_sidebar(c, sb_id, site_url, lang, "", tour_base)
                        .unwrap_or_default()
                })
                .unwrap_or_default()
            };
            o.push_str(&format!(
                "<div class=\"gdlr-core-pbf-sidebar-{side} gdlr-core-column-extend-{side}  gdlr-core-column-{width} gdlr-core-pbf-sidebar-padding  gdlr-core-line-height\" ><div class=\"gdlr-core-sidebar-inner-bg gdlr-core-item-mglr\"></div>{sb_html}</div>"
            ));
        }
        o.push_str(&format!(
            "<div class=\" gdlr-core-column-{content_w} gdlr-core-pbf-sidebar-padding gdlr-core-line-height\" ><div class=\"gdlr-core-page-builder-wrapper-sidebar-content-inner clearfix\" >{children}</div></div></div>"
        ));
        return Ok(o);
    }
    if typ == "blog" {
        // WP_Query equivalent: posts in the chosen category (or tag) for the
        // page language; WP tax_query also covers child terms — we have no
        // parent column, so an empty match falls back to latest posts, which
        // is what the reference homepage shows anyway.
        let posts = conn.and_then(|c| blog_posts(c, &opts, lang, media, site_url, tour_base).ok()).unwrap_or_default();
        let children = posts;
        let opts_v = JVal::from_serialize(&opts);
        return env
            .get_template("elements/blog.html")?
            .render(context! {
                opts => opts_v,
                children => children,
                type => typ,
                column => column,
                site => site.clone(),
                site_url => site_url,
                lang => lang,
                pdb => "40px",
                post => post.clone(),
            })
            .map_err(|e| anyhow::anyhow!("element {typ}: {e}"));
    }
    if matches!(typ, "accordion" | "toggle-box") {
        // gdlr_core_content_filter also runs on tab contents — expand the
        // same shortcode subset so [caption] becomes a <figure>.
        if let Some(Value::Array(tabs)) = opts.get_mut("tabs") {
            for tab in tabs.iter_mut() {
                if let Some(Value::String(s)) = tab.get_mut("content") {
                    if s.contains('[') {
                        *s = expand_shortcodes(s, site_url);
                    }
                }
            }
        }
    }
    let mut gallery_items = JVal::from_serialize(json!([]));
    if typ == "gallery" {
        // gallery ids -> {path, alt, caption, title, w, h} for the template's
        // fitrows grid (grid-no-space: lightgallery link + zoom overlay icon).
        let ids: Vec<i64> = opts
            .get("gallery")
            .and_then(|g| g.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.get("id"))
                    .filter_map(|x| match x {
                        Value::Number(n) => n.as_i64(),
                        Value::String(s) => s.parse().ok(),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut items = Vec::new();
        for id in &ids {
            items.push(
                conn.and_then(|c| media_info(c, *id))
                    .or_else(|| {
                        media.get(id).map(|(p, alt)| {
                            json!({"path": p, "alt": alt, "caption": "", "title": "", "w": 0, "h": 0})
                        })
                    })
                    .unwrap_or_else(|| json!({})),
            );
        }
        gallery_items = JVal::from_serialize(items);
    }

    let tpl_file = if ELEMENT_TYPES.contains(&typ) {
        format!("elements/{typ}.html")
    } else if typ == "normal" {
        "elements/background.html".into()
    } else {
        "elements/fallback.html".into()
    };
    let opts_v = JVal::from_serialize(&opts);
    env.get_template(&tpl_file)?
        .render(context! {
            opts => opts_v,
            children => children,
            type => typ,
            column => column,
            site => site.clone(),
            site_url => site_url,
            lang => lang,
            pdb => "40px",
            post => post.clone(),
            gallery_items => gallery_items,
        })
        .map_err(|e| anyhow::anyhow!("element {typ}: {e}"))
}

