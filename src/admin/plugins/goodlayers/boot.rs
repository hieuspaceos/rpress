//! GoodLayers builder boot DOM + `page_builder_var` localization — the
//! JS contract the vendored `framework/js/page-builder.js` depends on.
//! Moved from `admin::gdlr_pb` (deleted after bridge parity, phase-07):
//! these emitters are pure DOM/JS serialization — no business logic.
//! All admin-ajax behaviour (options, previews, templates, skins, save)
//! now runs inside the real plugin via the PHP bridge.

use super::super::*;
use crate::admin::esc;
use rusqlite::Connection;
use serde_json::{json, Map, Value};

/// URL prefix that mirrors GDLR_CORE_URL (the vendored plugin tree).
pub(crate) const GDLR_URL: &str = "/admin/wp/gdlr";

const COL_LIST: [(&str, &str); 12] = [
    ("60", "1/1"), ("30", "1/2"), ("20", "1/3"), ("40", "2/3"),
    ("15", "1/4"), ("45", "3/4"), ("12", "1/5"), ("24", "2/5"),
    ("36", "3/5"), ("48", "4/5"), ("10", "1/6"), ("50", "5/6"),
];






fn get<'a>(m: &'a Map<String, Value>, k: &str) -> Option<&'a Value> {
    m.get(k).or_else(|| m.get(&k.replace('_', "-")))
}

fn val_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}
/// PHP empty(): "", "0", 0, null, false, [], {} are empty.

fn is_empty(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) => s.is_empty() || s == "0",
        Some(Value::Number(n)) => n.as_i64() == Some(0) && n.as_f64().map(|f| f == 0.0).unwrap_or(false),
        Some(Value::Bool(b)) => !*b,
        Some(Value::Array(a)) => a.is_empty(),
        Some(Value::Object(o)) => o.is_empty(),
    }
}

fn val<'a>(m: &'a Map<String, Value>, k: &str) -> Option<&'a Value> {
    let v = get(m, k)?;
    if is_empty(Some(v)) { None } else { Some(v) }
}

fn s(m: &Map<String, Value>, k: &str) -> String {
    val(m, k).map(val_str).unwrap_or_default()
}

fn enabled(m: &Map<String, Value>, k: &str) -> bool {
    s(m, k) == "enable"
}

fn space_eq(v: Option<&Value>, t: &str, r: &str, b: &str, l: &str, settings: &str) -> bool {
    match v {
        Some(Value::Object(o)) => {
            val_str(o.get("top").unwrap_or(&Value::Null)) == t
                && val_str(o.get("right").unwrap_or(&Value::Null)) == r
                && val_str(o.get("bottom").unwrap_or(&Value::Null)) == b
                && val_str(o.get("left").unwrap_or(&Value::Null)) == l
                && val_str(o.get("settings").unwrap_or(&Value::Null)) == settings
        }
        _ => false,
    }
}

fn obj(v: Option<&Value>) -> Map<String, Value> {
    match v {
        Some(Value::Object(o)) => o.clone(),
        _ => Map::new(),
    }
}


fn rnd_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    ((n.subsec_nanos() ^ (n.as_secs() as u32)) % 10000) as u32
}

/* ---------------- gdlr_core_format_datatype ---------------- */


fn fmt_rgba(hex: &str) -> String {
    let h = hex.replace('#', "");
    let (r, g, b) = if h.len() == 3 {
        let p = |i: usize| u8::from_str_radix(&h[i..i + 1].repeat(2), 16).unwrap_or(0);
        (p(0), p(1), p(2))
    } else {
        let p = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("0"), 16).unwrap_or(0);
        (p(0), p(2), p(4))
    };
    format!("{}, {}, {}", r, g, b)
}

fn image_url(conn: &Connection, image: &Value) -> String {
    match image {
        Value::Number(n) => conn
            .query_row("SELECT path FROM media WHERE id=?1", [n.as_i64().unwrap_or(0)], |r| r.get::<_, String>(0))
            .unwrap_or_default(),
        Value::String(s) => {
            if s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty() {
                conn.query_row("SELECT path FROM media WHERE id=?1", [s.parse::<i64>().unwrap_or(0)], |r| r.get::<_, String>(0))
                    .unwrap_or_default()
            } else {
                s.clone()
            }
        }
        _ => String::new(),
    }
}

/* ---------------- gdlr_core_esc_style ---------------- */

/// `atts` is a list of (key, value) pairs preserving insertion order —
/// PHP arrays are ordered.
type StyleAtts = Vec<(String, Value)>;


fn esc_style(atts: &[(String, Value)]) -> String {
    esc_style_ex(atts, true, false)
}

fn esc_style_inner(atts: &[(String, Value)], important: bool) -> String {
    esc_style_ex(atts, false, important)
}

fn esc_style_ex(atts: &[(String, Value)], wrap: bool, important: bool) -> String {
    let mut att_style = String::new();
    let mut shadow: Option<(&Value, &Value, &Value)> = None;
    for (k, v) in atts {
        match k.as_str() {
            "background-shadow-color" => {
                shadow.get_or_insert((v, &Value::Null, &Value::Null)).0 = v;
            }
            "background-shadow-size" => {
                shadow.get_or_insert((&Value::Null, v, &Value::Null)).1 = v;
            }
            "background-shadow-opacity" => {
                shadow.get_or_insert((&Value::Null, &Value::Null, v)).2 = v;
            }
            _ => {}
        }
    }
    if let Some((color, size, opacity)) = shadow {
        if !is_empty(Some(color)) && !is_empty(Some(size)) {
            let bgs = match size {
                Value::Object(o) => {
                    let x = val_str(o.get("x").unwrap_or(&Value::Null));
                    let x = if x.is_empty() { "0".into() } else { x };
                    let y = val_str(o.get("y").unwrap_or(&Value::Null));
                    let y = if y.is_empty() { "0".into() } else { y };
                    format!("{} {} {}", x, y, val_str(o.get("size").unwrap_or(&Value::Null)))
                }
                v => val_str(v),
            };
            if !bgs.trim().is_empty() {
                let op = if is_empty(Some(opacity)) { "0".into() } else { val_str(opacity) };
                let bgs = format!("{} rgba({}, {}) ", bgs, fmt_rgba(&val_str(color)), op);
                let imp = if important { " !important" } else { "" };
                att_style += &format!("box-shadow: {bgs}{imp}; -moz-box-shadow: {bgs}{imp}; -webkit-box-shadow: {bgs}{imp}; ");
            }
        }
    }

    for (key, v) in atts {
        if key.starts_with("background-shadow-") { continue; }
        if is_empty(Some(v)) { continue; }
        match key.as_str() {
            "skewx" => {
                att_style += &format!("transform: skewX({}deg); -webkit-transform: skewX({}deg);", val_str(v), val_str(v));
            }
            "blur" => {
                let v = val_str(v);
                att_style += &format!("-webkit-filter: blur({v}); -moz-filter: blur({v}); -o-filter: blur({v}); -ms-filter: blur({v}); filter: blur({v});");
            }
            "border-radius" => {
                let radius = match v {
                    Value::Object(o) => {
                        let g = |k: &str| val_str(o.get(k).unwrap_or(&Value::Null));
                        if !g("top").is_empty() || !g("right").is_empty() || !g("bottom").is_empty() || !g("left").is_empty() {
                            format!("{} {} {} {}", nn(&g("top")), nn(&g("right")), nn(&g("bottom")), nn(&g("left")))
                        } else if !g("top-left").is_empty() || !g("top-right").is_empty() || !g("bottom-left").is_empty() || !g("bottom-right").is_empty() {
                            format!("{} {} {} {}", nn(&g("top-left")), nn(&g("top-right")), nn(&g("bottom-right")), nn(&g("bottom-left")))
                        } else if !g("t-left").is_empty() || !g("t-right").is_empty() || !g("b-left").is_empty() || !g("b-right").is_empty() {
                            format!("{} {} {} {}", nn(&g("t-left")), nn(&g("t-right")), nn(&g("b-right")), nn(&g("b-left")))
                        } else { String::new() }
                    }
                    v => val_str(v),
                };
                if !radius.is_empty() {
                    att_style += &format!("border-radius: {radius}; -moz-border-radius: {radius}; -webkit-border-radius: {radius};");
                }
            }
            "gradient" | "gradient-v" => {
                if let Value::Array(a) = v {
                    if a.len() > 1 {
                        let dir = if key == "gradient-v" { "to right, " } else { "" };
                        let col = |x: &Value| -> String {
                            match x {
                                Value::Array(p) if p.len() > 1 => format!("rgba({}, {})", fmt_rgba(&val_str(&p[0])), val_str(&p[1])),
                                _ => val_str(x),
                            }
                        };
                        let stops = if a.len() > 2 && !is_empty(a.get(2)) {
                            format!("{}, {}, {}", col(&a[0]), col(&a[1]), col(&a[2]))
                        } else {
                            format!("{}, {}", col(&a[0]), col(&a[1]))
                        };
                        att_style += &format!("background: linear-gradient({dir}{stops}); -moz-background: linear-gradient({dir}{stops}); -o-background: linear-gradient({dir}{stops}); -webkit-background: linear-gradient({dir}{stops});");
                    }
                }
            }
            "background" | "background-color" => {
                let imp = if important { " !important" } else { "" };
                match v {
                    Value::Array(a) if !a.is_empty() => {
                        let op = if a.len() > 1 { val_str(&a[1]) } else { "1".into() };
                        att_style += &format!("{key}: rgba({}, {}) {imp};", fmt_rgba(&val_str(&a[0])), op);
                    }
                    _ => att_style += &format!("{key}: {} {imp};", val_str(v)),
                }
            }
            "background-image" => {
                let imp = if important { " !important" } else { "" };
                match v {
                    Value::String(sv) if sv == "none" => att_style += "background-image: none;",
                    Value::Number(n) => {
                        // numeric → media lookup happens at call site (conn);
                        // here URLs are expected pre-resolved.
                        att_style += &format!("background-image: url({}) {imp};", n);
                    }
                    _ => att_style += &format!("background-image: url({}) {imp};", val_str(v)),
                }
            }
            "padding" | "margin" | "border-width" => {
                let imp = if important { " !important" } else { "" };
                match v {
                    Value::Object(o) => {
                        let (t, r, b, l) = (
                            val_str(o.get("top").unwrap_or(&Value::Null)),
                            val_str(o.get("right").unwrap_or(&Value::Null)),
                            val_str(o.get("bottom").unwrap_or(&Value::Null)),
                            val_str(o.get("left").unwrap_or(&Value::Null)),
                        );
                        if !t.is_empty() && !r.is_empty() && !b.is_empty() && !l.is_empty() {
                            att_style += &format!("{key}: {t} {r} {b} {l}{imp};");
                        } else {
                            for (pos, val) in o {
                                if pos == "settings" { continue; }
                                let vs = val_str(val);
                                if !vs.is_empty() || vs == "0" {
                                    if key == "border-width" {
                                        att_style += &format!("border-{pos}-width: {vs}{imp};");
                                    } else {
                                        att_style += &format!("{key}-{pos}: {vs}{imp};");
                                    }
                                }
                            }
                        }
                    }
                    _ => att_style += &format!("{key}: {}{imp};", val_str(v)),
                }
            }
            _ => {
                let imp = if important { " !important" } else { "" };
                let sv = match v {
                    Value::Array(a) => a.first().map(val_str).unwrap_or_default(),
                    _ => val_str(v),
                };
                att_style += &format!("{key}: {sv} {imp};");
            }
        }
    }

    if !att_style.is_empty() {
        if wrap {
            return format!("style=\"{}\" ", att_style.replace('"', "&quot;"));
        }
        return att_style;
    }
    String::new()
}

fn nn(v: &str) -> String {
    if v.is_empty() { "0".into() } else { v.to_string() }
}


fn animation_atts(m: &Map<String, Value>) -> String {
    let anim = s(m, "animation");
    if anim.is_empty() || anim == "none" { return String::new(); }
    let loc = s(m, "animation-location");
    let loc = if loc.is_empty() { "0.8".into() } else { loc };
    let dur = s(m, "animation-duration");
    let dur = if dur.is_empty() { "600ms".into() } else { dur };
    format!(
        "data-gdlr-animation=\"{}\" data-gdlr-animation-duration=\"{}\" data-gdlr-animation-offset=\"{}\" ",
        esc(&anim), esc(&dur), esc(&loc)
    )
}

/* ---------------- option defs ---------------- */


fn defs(sc: &SiteCtx) -> (Value, Value) {
    let dir = sc.templates.join("admin/wp/gdlr");
    let mut e: Value = std::fs::read_to_string(dir.join("elements.json"))
        .ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_else(|| json!({}));
    let mut w: Value = std::fs::read_to_string(dir.join("wrappers.json"))
        .ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_else(|| json!({}));
    strip_origin_urls(&mut e);
    strip_origin_urls(&mut w);
    (e, w)
}

/// These JSONs are dumps captured off a live WP — thumbnail/image strings
/// may still carry `http://<host>/wp-content/plugins/goodlayers-core/…`.
/// Rewrite to the root-relative tail so `GDLR_URL` prepending lands on the
/// vendored asset namespace; external URLs (docs links, CDNs) untouched.
fn strip_origin_urls(v: &mut Value) {
    match v {
        Value::String(s) => {
            if s.starts_with("http://") || s.starts_with("https://") {
                if let Some(i) = s.find("/wp-content/plugins/goodlayers-core") {
                    *s = s[i + "/wp-content/plugins/goodlayers-core".len()..].to_string();
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(strip_origin_urls),
        Value::Object(o) => o.values_mut().for_each(strip_origin_urls),
        _ => {}
    }
}

fn icons(sc: &SiteCtx) -> Value {
    std::fs::read_to_string(sc.templates.join("admin/wp/gdlr/icons.json"))
        .ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_else(|| json!({}))
}

// GoodLayers options live under the `plugin:goodlayers:` settings
// namespace; reads fall back to the legacy bare key until phase 05's
// key migration (write whitelist boundary, phase 06).

fn get_setting(conn: &Connection, key: &str) -> Value {
    crate::db::plugin_setting_json_value_conn(conn, "goodlayers", key)
}

fn pb_items(conn: &Connection, defs_e: &Value, items: &Value) -> String {
    let mut ret = String::new();
    if let Value::Array(a) = items {
        for node in a {
            ret.push_str(&pb_item(conn, defs_e, node));
        }
    }
    ret
}

fn pb_item(conn: &Connection, defs_e: &Value, node: &Value) -> String {
    let Value::Object(n) = node else { return String::new() };
    let template = s(n, "template");
    if template.is_empty() || template == "template" { return String::new(); }
    match template.as_str() {
        "wrapper" => wrapper_template(conn, n),
        "element" => element_template(conn, defs_e, n),
        _ => String::new(),
    }
}

/// gdlr_core_page_builder_element::get_template

fn element_template(conn: &Connection, defs_e: &Value, n: &Map<String, Value>) -> String {
    let typ = s(n, "type");
    let settings = defs_e.get(&typ).and_then(|e| e.get("settings")).cloned().unwrap_or_else(|| json!({}));
    let title = settings.get("title").and_then(|t| t.as_str()).unwrap_or("").to_string();
    let value = get(n, "value").cloned().unwrap_or(Value::Null);
    let dv = if is_empty(Some(&value)) { String::new() } else { format!("data-value=\"{}\" ", esc(&value.to_string())) };
    let dt = if typ.is_empty() { String::new() } else { format!("data-type=\"{}\" ", esc(&typ)) };

    let mut preview = String::new();
    if !typ.is_empty() && !settings.is_null() {
        preview = element_preview_html(conn, &typ, &value);
    }

    format!(
        "<div class=\"gdlr-core-page-builder-item\" data-template=\"element\" {dt}{dv}><div class=\"gdlr-core-page-builder-item-content\"><div class=\"gdlr-core-page-builder-item-head clearfix\"><span class=\"gdlr-core-page-builder-item-head-item-title\">{title}</span><i class=\"fa fa-remove gdlr-core-page-builder-item-remove\" ></i><i class=\"fa fa-copy gdlr-core-page-builder-item-copy\" ></i><i class=\"fa fa-download gdlr-core-page-builder-item-save\" ></i><i class=\"fa fa-edit gdlr-core-page-builder-item-edit\" ></i></div><div class=\"gdlr-core-page-builder-item-container\"><div class=\"gdlr-core-page-builder-item-container-item\"><i class=\"fa fa-align-justify gdlr-core-page-builder-item-container-item-icon\"></i><span class=\"gdlr-core-page-builder-item-container-item-title\">{title}</span></div><div class=\"gdlr-core-page-builder-item-container-preview\">{preview}<div class=\"gdlr-core-page-builder-item-container-overlay\"></div></div><i class=\"fa fa-edit gdlr-core-page-builder-item-edit\" ></i></div></div></div>",
        title = esc(&title)
    )
}

/// Element preview inside the builder canvas. PHP calls each element's
/// `get_preview`; our equivalent is the CMS frontend renderer (stable,
/// and identical to what the public site outputs).

fn element_preview_html(conn: &Connection, typ: &str, value: &Value) -> String {
    let node = json!({"template": "element", "type": typ, "value": value});
    let tour_base = PB_TOUR_BASE.with(|t| t.borrow().clone());
    crate::render::render_fragment_db(conn, &std::path::PathBuf::from("templates"), &node, "", "fr", &tour_base)
        .unwrap_or_default()
}
// Site tour_base for element previews — set alongside PB_DEFS by callers
// that hold a SiteCtx (deep canvas recursion can't carry it otherwise).
thread_local! {
    static PB_TOUR_BASE: std::cell::RefCell<String> = std::cell::RefCell::new(String::from("voyage"));
}

/// gdlr_core_page_builder_wrapper::get_template — dispatch per type,
/// honouring sync-template substitution.

fn wrapper_template(conn: &Connection, n: &Map<String, Value>) -> String {
    let mut n = n.clone();
    if let Some(slug) = val(&n, "sync-template") {
        let syncs = get_setting(conn, "gdlr-core-pb-sync-template");
        if let Some(opts) = syncs.get(&val_str(slug)).and_then(|t| t.get("value")) {
            n = obj(Some(opts));
        } else {
            n.remove("sync-template");
        }
    }
    match s(&n, "type").as_str() {
        "column" => column_template(conn, &n),
        "sidebar" => sidebar_template(conn, &n),
        _ => background_template(conn, &n),
    }
}

/// pb-wrapper-column.php::get_template

fn column_template(conn: &Connection, n: &Map<String, Value>) -> String {
    let value = obj(get(n, "value"));

    let mut custom_style = String::new();
    let mut column_id = s(&value, "id");
    if let Some(c) = val(&value, "hover-border-color") {
        custom_style += &format!(
            "#custom_style_id:hover .gdlr-core-pbf-column-content-margin, #custom_style_id:hover .gdlr-core-pbf-column-content-margin .gdlr-core-pbf-background-wrap, #custom_style_id:hover .gdlr-core-pbf-column-content-margin .gdlr-core-pbf-background-frame{{ border-color: {} !important; }}",
            val_str(c)
        );
    }
    if let Some(w) = val(&value, "hover-border-width") {
        custom_style += &format!(
            "#custom_style_id:hover .gdlr-core-pbf-column-content-margin .gdlr-core-pbf-background-wrap{{ {} }}",
            esc_style_inner(&[("border-width".into(), w.clone())], true)
        );
    }
    if enabled(&value, "enable-move-up-shadow-effect") {
        let mut tmp = esc_style_inner(&[
            ("background-shadow-size".into(), get(&value, "frame-hover-shadow-size").cloned().unwrap_or(Value::Null)),
            ("background-shadow-color".into(), get(&value, "frame-hover-shadow-color").cloned().unwrap_or(Value::Null)),
            ("background-shadow-opacity".into(), get(&value, "frame-hover-shadow-opacity").cloned().unwrap_or(Value::Null)),
            ("z-index".into(), json!(1)),
        ], false);
        if let Some(l) = val(&value, "move-up-effect-length") {
            tmp += &format!("transform: translate3d(0, -{}, 0); ", val_str(l));
        }
        if !tmp.is_empty() {
            custom_style += &format!("#custom_style_id:hover .gdlr-core-move-up-with-shadow{{ {tmp} }}");
        }
    }
    if !custom_style.is_empty() {
        if column_id.is_empty() {
            column_id = format!("gdlr-core-column-{}", rnd_id());
        }
        custom_style = format!("<style>{}</style>", custom_style.replace("custom_style_id", &column_id));
    }

    let zero_sp = |v: Option<&Value>| space_eq(v, "0px", "0px", "0px", "0px", "link");
    let empty_br = |v: Option<&Value>| space_eq(v, "", "", "", "", "link");
    let mut wrapper_style: StyleAtts = vec![
        ("min-height".into(), get(&value, "min-height").cloned().unwrap_or(Value::Null)),
        ("margin".into(), if space_eq(get(&value, "margin"), "0px", "0px", "0px", "0px", "link") { Value::Null } else { get(&value, "margin").cloned().unwrap_or(Value::Null) }),
        ("padding".into(), if zero_sp(get(&value, "padding")) { Value::Null } else { get(&value, "padding").cloned().unwrap_or(Value::Null) }),
        ("background-shadow-size".into(), get(&value, "background-shadow-size").cloned().unwrap_or(Value::Null)),
        ("background-shadow-color".into(), get(&value, "background-shadow-color").cloned().unwrap_or(Value::Null)),
        ("background-shadow-opacity".into(), get(&value, "background-shadow-opacity").cloned().unwrap_or(Value::Null)),
        ("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }),
    ];
    let mut content_style: StyleAtts = vec![
        ("max-width".into(), get(&value, "max-width").cloned().unwrap_or(Value::Null)),
    ];

    // background
    let mut wrapper_background = String::new();
    let bg_type = s(&value, "background-type");
    if !bg_type.is_empty() {
        if bg_type == "color" {
            let mut atts: StyleAtts = vec![
                ("background-color".into(), get(&value, "background-color").cloned().unwrap_or(Value::Null)),
                ("opacity".into(), {
                    let o = get(&value, "background-opacity").cloned().unwrap_or(Value::Null);
                    if is_empty(Some(&o)) || val_str(&o) == "1" { Value::Null } else { o }
                }),
                ("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }),
            ];
            let style = s(&value, "background-color-style");
            if (style == "gradient" || style == "gradient-v") && !is_empty(get(&value, "background-gradient-color")) {
                atts.push((style.clone(), json!([
                    [s(&value, "background-color"), s(&value, "background-normal-color-opacity").parse().ok().map(|_: String| s(&value, "background-normal-color-opacity")).unwrap_or_else(|| "0".into())],
                    [s(&value, "background-gradient-color"), if s(&value, "background-gradient-color-opacity").is_empty() { "0".into() } else { s(&value, "background-gradient-color-opacity") }]
                ])));
            }
            if s(&value, "border-type") == "outer-border" {
                atts.push(("border-width".into(), get(&value, "border-width").cloned().unwrap_or(Value::Null)));
                atts.push(("border-color".into(), get(&value, "border-color").cloned().unwrap_or(Value::Null)));
                atts.push(("border-style".into(), get(&value, "border-style").cloned().unwrap_or(Value::Null)));
            }
            wrapper_background = format!("<div class=\"gdlr-core-pbf-background-wrap\" {} ></div>", esc_style(&atts));
        } else if bg_type == "image" {
            let img = get(&value, "background-image").map(|v| image_url(conn, v)).unwrap_or_default();
            let mut bgi: StyleAtts = vec![
                ("opacity".into(), {
                    let o = get(&value, "background-opacity").cloned().unwrap_or(Value::Null);
                    if is_empty(Some(&o)) || val_str(&o) == "1" { Value::Null } else { o }
                }),
                ("background-image".into(), Value::String(img)),
            ];
            let img_style = s(&value, "background-image-style");
            if !img_style.is_empty() {
                if img_style == "cover" {
                    bgi.push(("background-size".into(), json!("cover")));
                    bgi.push(("background-repeat".into(), json!("no-repeat")));
                } else if img_style == "contain" {
                    bgi.push(("background-size".into(), json!("contain")));
                } else {
                    bgi.push(("background-repeat".into(), json!(img_style.clone())));
                }
                let pos = s(&value, "background-image-position");
                if !pos.is_empty() {
                    if pos == "custom" {
                        let pc = obj(get(&value, "background-image-position-custom"));
                        let (x, y) = (s(&pc, "x"), s(&pc, "y"));
                        if !x.is_empty() || !y.is_empty() {
                            bgi.push(("background-position".into(), json!(format!("{x} {y}"))));
                        }
                    } else {
                        bgi.push(("background-position".into(), json!(pos.replace('-', " "))));
                    }
                }
            }
            let parallax = s(&value, "parallax-speed") == "1";
            if parallax { bgi.push(("background-attachment".into(), json!("fixed"))); }
            if s(&value, "background-filter") == "blur" {
                if let Some(b) = val(&value, "background-blur-size") {
                    bgi.push(("blur".into(), b.clone()));
                }
            }
            for (k, _) in [
                ("background-shadow-size", "background-shadow-size"),
                ("background-shadow-color", "background-shadow-color"),
                ("background-shadow-opacity", "background-shadow-opacity"),
            ] {
                wrapper_style.push((k.into(), get(&value, k).cloned().unwrap_or(Value::Null)));
            }
            wrapper_style.push(("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }));

            let mut bg_atts: StyleAtts = vec![
                ("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }),
                ("background-color".into(), get(&value, "background-color").cloned().unwrap_or(Value::Null)),
            ];
            let style = s(&value, "background-color-style");
            if (style == "gradient" || style == "gradient-v") && !is_empty(get(&value, "background-gradient-color")) {
                bg_atts.push((style.clone(), json!([
                    [s(&value, "background-color"), if s(&value, "background-color-opacity").is_empty() { "0".into() } else { s(&value, "background-color-opacity") }],
                    [s(&value, "background-gradient-color"), if s(&value, "background-gradient-color-opacity").is_empty() { "0".into() } else { s(&value, "background-gradient-color-opacity") }]
                ])));
            }
            let mut out = String::new();
            if !is_empty(get(&value, "background-image-bottom-offset")) || !is_empty(get(&value, "background-image-top-offset")) {
                out += &format!("<div class=\"gdlr-core-pbf-background-wrap\" {} ></div>", esc_style(&bg_atts));
                bg_atts.retain(|(k, _)| k != "background-color" && k != "gradient" && k != "gradient-v");
                if let Some(v) = val(&value, "background-image-bottom-offset") { bg_atts.push(("bottom".into(), v.clone())); }
                if let Some(v) = val(&value, "background-image-top-offset") { bg_atts.push(("top".into(), v.clone())); }
            }
            out += &format!("<div class=\"gdlr-core-pbf-background-wrap\" {} >", esc_style(&bg_atts));
            if !parallax {
                out += &format!("<div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} data-parallax-speed=\"{}\"  ></div>", esc_style(&bgi), if s(&value, "parallax-speed").is_empty() { "0".into() } else { s(&value, "parallax-speed") });
            } else {
                out += &format!("<div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} ></div>", esc_style(&bgi));
            }
            out += "</div>";
            wrapper_background = out;
        } else if bg_type == "pattern" {
            let pattern = if s(&value, "background-pattern").is_empty() { "pattern-1".into() } else { s(&value, "background-pattern") };
            let mut bgi: StyleAtts = vec![
                ("background-image".into(), json!(format!("{GDLR_URL}/include/images/pattern/{pattern}.png"))),
                ("background-repeat".into(), json!("repeat")),
                ("background-position".into(), json!("center")),
                ("opacity".into(), {
                    let o = get(&value, "background-opacity").cloned().unwrap_or(Value::Null);
                    if is_empty(Some(&o)) || val_str(&o) == "1" { Value::Null } else { o }
                }),
            ];
            let parallax = s(&value, "parallax-speed") == "1";
            if parallax { bgi.push(("background-attachment".into(), json!("fixed"))); }
            let mut out = format!("<div class=\"gdlr-core-pbf-background-wrap\" {} >", esc_style(&[
                ("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }),
            ]));
            if s(&value, "border-type") == "outer-border" {
                wrapper_style.push(("border-width".into(), get(&value, "border-width").cloned().unwrap_or(Value::Null)));
                wrapper_style.push(("border-color".into(), get(&value, "border-color").cloned().unwrap_or(Value::Null)));
                wrapper_style.push(("border-style".into(), get(&value, "border-style").cloned().unwrap_or(Value::Null)));
            }
            if !parallax {
                out += &format!("<div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} data-parallax-speed=\"{}\"  ></div>", esc_style(&bgi), if s(&value, "parallax-speed").is_empty() { "0".into() } else { s(&value, "parallax-speed") });
            } else {
                out += &format!("<div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} ></div>", esc_style(&bgi));
            }
            out += "</div>";
            wrapper_background = out;
        } else {
            // video / html5-video
            let mut bg_atts: StyleAtts = vec![
                ("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }),
            ];
            if s(&value, "border-type") == "outer-border" {
                bg_atts.push(("border-width".into(), get(&value, "border-width").cloned().unwrap_or(Value::Null)));
                bg_atts.push(("border-color".into(), get(&value, "border-color").cloned().unwrap_or(Value::Null)));
                bg_atts.push(("border-style".into(), get(&value, "border-style").cloned().unwrap_or(Value::Null)));
            }
            let fallback = get(&value, "background-video-image").map(|v| image_url(conn, v)).unwrap_or_default();
            let mut out = format!("<div class=\"gdlr-core-pbf-background-wrap\" {} >", esc_style(&bg_atts));
            out += &format!("<div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} data-parallax-speed=\"{}\"  data-video-fallback=\"{}\"  >", esc_style(&[
                ("background-position".into(), json!("center")),
                ("background-size".into(), json!("cover")),
            ]), if s(&value, "parallax-speed").is_empty() { "0".into() } else { s(&value, "parallax-speed") }, esc(&fallback));
            out += "<div class=\"gdlr-core-pbf-background-video\" data-background-type=\"video\" >";
            if bg_type == "video" {
                if let Some(u) = val(&value, "background-video-url") {
                    out += &pb_video(&val_str(u), "100%", "100%", true);
                }
            } else if bg_type == "html5-video" {
                out += "<video autoplay loop muted >";
                for (k, mime) in [("background-video-url-mp4", "video/mp4"), ("background-video-url-webm", "video/webm"), ("background-video-url-ogg", "video/ogg")] {
                    if let Some(u) = val(&value, k) {
                        out += &format!("<source src=\"{}\" type=\"{mime}\">", esc(&val_str(u)));
                    }
                }
                out += "</video>";
            }
            out += "</div></div></div>";
            wrapper_background = out;
        }
    }

    // inner border
    if s(&value, "border-type") == "inner-border" {
        wrapper_background += &format!("<div class=\"gdlr-core-pbf-background-frame\" {} ></div>", esc_style(&[
            ("margin".into(), get(&value, "border-pre-spaces").cloned().unwrap_or(json!("0px"))),
            ("border-width".into(), get(&value, "border-width").cloned().unwrap_or(Value::Null)),
            ("border-style".into(), get(&value, "border-style").cloned().unwrap_or(Value::Null)),
            ("border-color".into(), get(&value, "border-color").cloned().unwrap_or(Value::Null)),
        ]));
    }

    let col_num = s(n, "column");
    let col_title = COL_LIST.iter().find(|(k, _)| *k == col_num).map(|(_, v)| *v).unwrap_or("");
    let mut column_class = "gdlr-core-page-builder-column".to_string();
    if !col_num.is_empty() { column_class += &format!(" gdlr-core-column-{col_num}"); }
    let dv = if is_empty(get(n, "value")) { String::new() } else { format!("data-value=\"{}\"", esc(&get(n, "value").unwrap().to_string())) };
    let dc = if col_num.is_empty() { String::new() } else { format!("data-column=\"{}\"", esc(&col_num)) };
    let dskin = if s(&value, "skin").is_empty() { String::new() } else { format!("data-skin=\"{}\" ", esc(&s(&value, "skin"))) };
    let did = if column_id.is_empty() { String::new() } else { format!(" id=\"{}\" ", esc(&column_id)) };

    let mut item_class = String::new();
    let ext = s(&value, "background-extending");
    if !ext.is_empty() && ext != "none" { item_class += &format!(" gdlr-core-column-extend-{ext}"); }
    if !s(&value, "class").is_empty() { item_class += &format!(" {}", s(&value, "class")); }
    if !s(&value, "effect").is_empty() { item_class += &format!(" gdlr-core-effect-{}", s(&value, "effect")); }
    if enabled(&value, "enable-move-up-shadow-effect") { item_class += " gdlr-core-move-up-with-shadow"; }
    if enabled(&value, "enable-left-right-spaces") { item_class += " gdlr-core-item-mglr gdlr-core-item-mgb"; }
    if !s(&value, "overflow").is_empty() || enabled(&value, "enable-move-up-shadow-effect") {
        if !empty_br(get(&value, "border-radius")) {
            wrapper_style.retain(|(k, _)| k != "border-radius");
            wrapper_style.push(("border-radius".into(), get(&value, "border-radius").cloned().unwrap_or(Value::Null)));
        }
        if !s(&value, "overflow").is_empty() {
            wrapper_style.push(("overflow".into(), json!(s(&value, "overflow"))));
        }
    }

    let mut extra = String::new();
    if let Some(sh) = val(&value, "sync-height") {
        extra += &format!(" data-sync-height=\"{}\" ", esc(&val_str(sh)));
        if enabled(&value, "centering-sync-height-content") { extra += " data-sync-height-center"; }
    }
    if s(&value, "effect") == "3d" {
        if let Some(d) = val(&value, "3d-rotate-degree") {
            extra += &format!(" data-max-deg=\"{}\"", esc(&val_str(d)));
        }
        if let Some(z) = val(&value, "3d-content-z-pos") {
            content_style.push(("transform".into(), json!(format!("translateZ({})", val_str(z)))));
        }
    }

    let mut column = format!(
        "<div class=\"{}\" data-template=\"wrapper\" data-type=\"column\" {dc} {dv} {dskin}{did}>",
        esc(&column_class)
    );
    column += "<div class=\"gdlr-core-page-builder-column-content\">";
    column += "<div class=\"gdlr-core-page-builder-column-head\">";
    column += "<div class=\"gdlr-core-page-builder-column-size\"><i class=\"fa fa-plus gdlr-core-page-builder-column-increase\" ></i><i class=\"fa fa-minus gdlr-core-page-builder-column-decrease\" ></i></div>";
    column += &format!("<span class=\"gdlr-core-page-builder-column-head-title\" >{col_title}</span>");
    column += "<div class=\"gdlr-core-page-builder-column-settings\"><i class=\"fa fa-edit gdlr-core-page-builder-column-edit\" ></i><i class=\"fa fa-copy gdlr-core-page-builder-column-copy\" ></i><i class=\"fa fa-download gdlr-core-page-builder-column-save\" ></i><i class=\"fa fa-remove gdlr-core-page-builder-column-remove\" ></i></div>";
    column += "</div>";
    column += &format!("<div class=\"gdlr-core-page-builder-column-content-margin gdlr-core-pbf-column-content-margin gdlr-core-js {}\" {}{} >", esc(&item_class), esc_style(&wrapper_style), extra);
    column += &wrapper_background;
    if s(&value, "border-type") == "right-divider" {
        let h = s(&value, "divider-height");
        let half: i64 = h.trim_end_matches("px").parse().unwrap_or(0) / 2;
        column += &format!("<div class=\"gdlr-core-page-builder-column-right-divider\" {} ></div>", esc_style(&[
            ("height".into(), json!(h)),
            ("margin-top".into(), json!(format!("-{half}px"))),
            ("border-color".into(), get(&value, "border-color").cloned().unwrap_or(Value::Null)),
            ("border-style".into(), get(&value, "border-style").cloned().unwrap_or(Value::Null)),
        ]));
    }
    column += &format!("<div class=\"gdlr-core-page-builder-column-content-wrap gdlr-core-pbf-column-content gdlr-core-sync-height-content gdlr-core-js clearfix\" {} {} >", esc_style(&content_style), animation_atts(&value));
    column += "<div class=\"gdlr-core-page-builder-column-container gdlr-core-column-sortable\">";
    if let Some(items) = get(n, "items") {
        column += &pb_items_scoped(conn, items);
    }
    column += "</div></div></div></div>";
    column += &custom_style;
    column += &format!("<script id=\"gdlr-core-preview-column-background-{}\" >\njQuery(document).ready(function(){{\n\tvar parallax_bg_script = jQuery('#gdlr-core-preview-column-background-{}');\n\n\tparallax_bg_script.parent().gdlr_core_parallax_background().gdlr_core_fluid_video();\n\tparallax_bg_script.parent().gdlr_core_ux();\n\t{}\n\tjQuery(window).trigger('scroll');\n}});\n</script>",
        rnd_id(), rnd_id(),
        if !s(&value, "sync-height").is_empty() { "new gdlr_core_sync_height(parallax_bg_script.closest('.gdlr-core-page-builder-body'));" } else { "" });
    column += "</div>";
    column
}

/// Same as pb_items but pulls element defs lazily per call (column recursion
/// happens inside wrapper_template which doesn't carry defs through in PHP —
/// defs are global there; here we re-read cheap JSON per page render).

fn pb_items_scoped(conn: &Connection, items: &Value) -> String {
    // defs for element titles come from a thread-local-style param; to keep the
    // signature simple we re-dispatch with a cached static.
    PB_DEFS.with(|d| {
        let defs_e = d.borrow().clone();
        pb_items(conn, &defs_e, items)
    })
}
thread_local! {
    static PB_DEFS: std::cell::RefCell<Value> = std::cell::RefCell::new(json!({}));
}

/// pb-wrapper-background.php::get_template

fn background_template(conn: &Connection, n: &Map<String, Value>) -> String {
    let value = obj(get(n, "value"));
    let zero_sp = |v: Option<&Value>| space_eq(v, "0px", "0px", "0px", "0px", "link");
    let empty_br = |v: Option<&Value>| space_eq(v, "", "", "", "", "link");
    let unlink_pad = |v: Option<&Value>| space_eq(v, "60px", "0px", "30px", "0px", "unlink");

    let mut wrapper_style: StyleAtts = vec![
        ("margin".into(), if zero_sp(get(&value, "margin")) { Value::Null } else { get(&value, "margin").cloned().unwrap_or(json!([])) }),
        ("padding".into(), if unlink_pad(get(&value, "padding")) { Value::Null } else { get(&value, "padding").cloned().unwrap_or(Value::Null) }),
        ("min-height".into(), get(&value, "min-height").cloned().unwrap_or(Value::Null)),
    ];
    if let Some(mw) = val(&value, "max-width-wrapper") {
        wrapper_style.push(("max-width".into(), mw.clone()));
        if let Some(Value::Object(m)) = wrapper_style.iter_mut().find(|(k, _)| k == "margin").map(|(_, v)| v) {
            if !matches!(m.get("left"), Some(Value::Null) | None) || true {
                m.insert("left".into(), json!("auto"));
                m.insert("right".into(), json!("auto"));
            }
        } else {
            wrapper_style.iter_mut().for_each(|_| {});
            wrapper_style.push(("margin".into(), json!({"left": "auto", "right": "auto"})));
        }
    }
    let content_style: StyleAtts = vec![];

    // background (same switch as column, wrapper variant)
    let mut wrapper_background = String::new();
    let bg_type = s(&value, "background-type");
    if !bg_type.is_empty() {
        if bg_type == "color" {
            for (k, _) in [
                ("border-radius", ()), ("background-color", ()), ("background-shadow-size", ()),
                ("background-shadow-color", ()), ("background-shadow-opacity", ()),
            ] {
                if k == "border-radius" {
                    wrapper_style.push((k.into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }));
                } else {
                    wrapper_style.push((k.into(), get(&value, k).cloned().unwrap_or(Value::Null)));
                }
            }
            let style = s(&value, "background-color-style");
            if style == "gradient" || style == "gradient-v" {
                wrapper_style.push((style.clone(), json!([
                    s(&value, "background-color"),
                    s(&value, "background-gradient-color")
                ])));
            }
        } else if bg_type == "image" {
            let img = get(&value, "background-image").map(|v| image_url(conn, v)).unwrap_or_default();
            let mut bgi: StyleAtts = vec![
                ("opacity".into(), get(&value, "background-opacity").cloned().unwrap_or(Value::Null)),
                ("background-image".into(), Value::String(img)),
            ];
            let img_style = s(&value, "background-image-style");
            if !img_style.is_empty() {
                if img_style == "cover" {
                    bgi.push(("background-size".into(), json!("cover")));
                    bgi.push(("background-repeat".into(), json!("no-repeat")));
                } else if img_style == "contain" {
                    bgi.push(("background-size".into(), json!("contain")));
                } else {
                    bgi.push(("background-repeat".into(), json!(img_style.clone())));
                }
                let pos = s(&value, "background-image-position");
                if !pos.is_empty() {
                    if pos == "custom" {
                        let pc = obj(get(&value, "background-image-position-custom"));
                        let (x, y) = (s(&pc, "x"), s(&pc, "y"));
                        if !x.is_empty() || !y.is_empty() {
                            bgi.push(("background-position".into(), json!(format!("{x} {y}"))));
                        }
                    } else {
                        bgi.push(("background-position".into(), json!(pos.replace('-', " "))));
                    }
                }
            }
            let parallax = s(&value, "parallax-speed") == "1";
            if parallax { bgi.push(("background-attachment".into(), json!("fixed"))); }
            if s(&value, "background-filter") == "blur" {
                if let Some(b) = val(&value, "background-blur-size") { bgi.push(("blur".into(), b.clone())); }
            }
            for k in ["background-shadow-size", "background-shadow-color", "background-shadow-opacity"] {
                wrapper_style.push((k.into(), get(&value, k).cloned().unwrap_or(Value::Null)));
            }
            wrapper_style.push(("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }));

            let mut bg_atts: StyleAtts = vec![
                ("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }),
                ("background-color".into(), get(&value, "background-color").cloned().unwrap_or(Value::Null)),
            ];
            let style = s(&value, "background-color-style");
            if (style == "gradient" || style == "gradient-v") && !is_empty(get(&value, "background-gradient-color")) {
                bg_atts.push((style.clone(), json!([
                    [s(&value, "background-color"), if s(&value, "background-color-opacity").is_empty() { "0".into() } else { s(&value, "background-color-opacity") }],
                    [s(&value, "background-gradient-color"), if s(&value, "background-gradient-color-opacity").is_empty() { "0".into() } else { s(&value, "background-gradient-color-opacity") }]
                ])));
            }
            let mut out = String::new();
            if !is_empty(get(&value, "background-image-bottom-offset")) || !is_empty(get(&value, "background-image-top-offset")) {
                out += &format!("<div class=\"gdlr-core-pbf-background-wrap\" {} ></div>", esc_style(&bg_atts));
                bg_atts.retain(|(k, _)| k != "background-color" && k != "gradient" && k != "gradient-v");
                if let Some(v) = val(&value, "background-image-bottom-offset") { bg_atts.push(("bottom".into(), v.clone())); }
                if let Some(v) = val(&value, "background-image-top-offset") { bg_atts.push(("top".into(), v.clone())); }
            }
            out += &format!("<div class=\"gdlr-core-pbf-background-wrap\" {} >", esc_style(&bg_atts));
            if !parallax {
                out += &format!("<div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} data-parallax-speed=\"{}\"  ></div>", esc_style(&bgi), if s(&value, "parallax-speed").is_empty() { "0".into() } else { s(&value, "parallax-speed") });
            } else {
                out += &format!("<div class=\"gdlr-core-pbf-background\" {} ></div>", esc_style(&bgi));
            }
            out += "</div>";
            wrapper_background = out;
        } else if bg_type == "pattern" {
            let pattern = if s(&value, "background-pattern").is_empty() { "pattern-1".into() } else { s(&value, "background-pattern") };
            let mut bgi: StyleAtts = vec![
                ("background-image".into(), json!(format!("{GDLR_URL}/include/images/pattern/{pattern}.png"))),
                ("background-repeat".into(), json!("repeat")),
                ("background-position".into(), json!("center")),
                ("opacity".into(), {
                    let o = get(&value, "pattern-opacity").cloned().unwrap_or(json!("1"));
                    if val_str(&o) == "1" { Value::Null } else { o }
                }),
            ];
            let parallax = s(&value, "parallax-speed") == "1";
            if parallax { bgi.push(("background-attachment".into(), json!("fixed"))); }
            let mut out = format!("<div class=\"gdlr-core-pbf-background-wrap\" {} >", esc_style(&[
                ("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }),
                ("background-shadow-size".into(), get(&value, "background-shadow-size").cloned().unwrap_or(Value::Null)),
                ("background-shadow-color".into(), get(&value, "background-shadow-color").cloned().unwrap_or(Value::Null)),
                ("background-shadow-opacity".into(), get(&value, "background-shadow-opacity").cloned().unwrap_or(Value::Null)),
            ]));
            if !parallax {
                out += &format!("<div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} data-parallax-speed=\"{}\"  ></div>", esc_style(&bgi), if s(&value, "parallax-speed").is_empty() { "0".into() } else { s(&value, "parallax-speed") });
            } else {
                out += &format!("<div class=\"gdlr-core-pbf-background\" {} ></div>", esc_style(&bgi));
            }
            out += "</div>";
            wrapper_background = out;
        } else {
            // video
            let mut out = format!("<div class=\"gdlr-core-pbf-background-wrap\" {} >", esc_style(&[
                ("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }),
                ("background-shadow-size".into(), get(&value, "background-shadow-size").cloned().unwrap_or(Value::Null)),
                ("background-shadow-color".into(), get(&value, "background-shadow-color").cloned().unwrap_or(Value::Null)),
                ("background-shadow-opacity".into(), get(&value, "background-shadow-opacity").cloned().unwrap_or(Value::Null)),
            ]));
            let fallback = get(&value, "background-video-image").map(|v| image_url(conn, v)).unwrap_or_default();
            out += &format!("<div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} data-parallax-speed=\"{}\"  data-video-fallback=\"{}\"  >", esc_style(&[
                ("background-position".into(), json!("center")),
                ("background-size".into(), json!("cover")),
            ]), if s(&value, "parallax-speed").is_empty() { "0".into() } else { s(&value, "parallax-speed") }, esc(&fallback));
            out += "<div class=\"gdlr-core-pbf-background-video\" data-background-type=\"video\" >";
            if bg_type == "video" {
                if let Some(u) = val(&value, "background-video-url") {
                    out += &pb_video(&val_str(u), "100%", "100%", true);
                }
            } else if bg_type == "html5-video" {
                out += "<video autoplay loop muted >";
                for (k, mime) in [("background-video-url-mp4", "video/mp4"), ("background-video-url-webm", "video/webm"), ("background-video-url-ogg", "video/ogg")] {
                    if let Some(u) = val(&value, k) {
                        out += &format!("<source src=\"{}\" type=\"{mime}\">", esc(&val_str(u)));
                    }
                }
                out += "</video>";
            }
            out += "</div></div></div>";
            wrapper_background = out;
        }
    }

    // border
    let border_type = s(&value, "border-type");
    if border_type == "outer-border" {
        wrapper_style.push(("border-width".into(), get(&value, "border-width").cloned().unwrap_or(Value::Null)));
        wrapper_style.push(("border-color".into(), get(&value, "border-color").cloned().unwrap_or(Value::Null)));
        wrapper_style.push(("border-style".into(), get(&value, "border-style").cloned().unwrap_or(Value::Null)));
    } else if border_type == "inner-border" {
        wrapper_background += &format!("<div class=\"gdlr-core-pbf-background-frame\" {} ></div>", esc_style(&[
            ("margin".into(), get(&value, "border-pre-spaces").cloned().unwrap_or(json!("0px"))),
            ("border-width".into(), get(&value, "border-width").cloned().unwrap_or(Value::Null)),
            ("border-style".into(), get(&value, "border-style").cloned().unwrap_or(Value::Null)),
            ("border-color".into(), get(&value, "border-color").cloned().unwrap_or(Value::Null)),
        ]));
    }

    // marquee
    let mut marquee = String::new();
    if enabled(&value, "enable-marquee") && !s(&value, "marquee-text").is_empty() {
        let mut atts: StyleAtts = vec![
            ("color".into(), get(&value, "marquee-text-color").cloned().unwrap_or(Value::Null)),
            ("opacity".into(), get(&value, "marquee-opacity").cloned().unwrap_or(Value::Null)),
            ("font-size".into(), get(&value, "marquee-font-size").cloned().unwrap_or(Value::Null)),
            ("font-weight".into(), get(&value, "marquee-font-weight").cloned().unwrap_or(Value::Null)),
            ("letter-spacing".into(), get(&value, "marquee-font-letter-spacing").cloned().unwrap_or(Value::Null)),
        ];
        if !s(&value, "marquee-position").is_empty() && !s(&value, "marquee-position-offset").is_empty() {
            if s(&value, "marquee-position") == "top" {
                atts.push(("top".into(), get(&value, "marquee-position-offset").cloned().unwrap_or(Value::Null)));
                atts.push(("bottom".into(), json!("auto")));
            } else {
                atts.push(("bottom".into(), get(&value, "marquee-position-offset").cloned().unwrap_or(Value::Null)));
                atts.push(("top".into(), json!("auto")));
            }
        }
        marquee = format!("<div class=\"gdlr-core-pbf-wrapper-marquee gdlr-core-marquee gdlr-core-title-font gdlr-core-js\" {} {} >{}</div>",
            if s(&value, "marquee-speed").is_empty() { String::new() } else { format!("data-speed=\"{}\" ", esc(&s(&value, "marquee-speed"))) },
            if s(&value, "marquee-direction").is_empty() { esc_style(&atts) } else { format!("data-direction=\"{}\" {}", esc(&s(&value, "marquee-direction")), esc_style(&atts)) },
            s(&value, "marquee-text"));
    }

    let dv = if is_empty(get(n, "value")) { String::new() } else { format!("data-value=\"{}\" ", esc(&get(n, "value").unwrap().to_string())) };
    let dst = if s(n, "sync-template").is_empty() { String::new() } else { format!("data-sync-template=\"{}\" ", esc(&s(n, "sync-template"))) };
    let dskin = if s(&value, "skin").is_empty() { String::new() } else { format!("data-skin=\"{}\" ", esc(&s(&value, "skin"))) };

    let mut wrapper = format!("<div class=\"gdlr-core-page-builder-wrapper gdlr-core-page-builder-background-wrapper\" data-template=\"wrapper\" data-type=\"background\" {dv}{dst}{dskin}>");
    wrapper += "<div class=\"gdlr-core-page-builder-wrapper-content\" ><div class=\"gdlr-core-page-builder-wrapper-head\"><span class=\"gdlr-core-page-builder-wrapper-head-title\" >Wrapper</span><div class=\"gdlr-core-page-builder-wrapper-settings\"><i class=\"fa fa-edit gdlr-core-page-builder-wrapper-edit\" ></i><i class=\"fa fa-copy gdlr-core-page-builder-wrapper-copy\" ></i><i class=\"fa fa-download gdlr-core-page-builder-wrapper-save\" ></i><i class=\"fa fa-refresh gdlr-core-page-builder-wrapper-sync\" ></i><i class=\"fa fa-remove gdlr-core-page-builder-wrapper-remove\" ></i></div></div>";

    let mut container_class = String::new();
    let mut container_style: StyleAtts = vec![];
    match s(&value, "content-layout").as_str() {
        "" | "boxed" => container_class += "gdlr-core-container",
        "custom" => {
            container_class += "gdlr-core-container-custom";
            if let Some(mw) = val(&value, "max-width") { container_style.push(("max-width".into(), mw.clone())); }
        }
        "full" => {
            if !enabled(&value, "enable-space") { container_class += "gdlr-core-pbf-wrapper-full-no-space"; }
            else { container_class += "gdlr-core-pbf-wrapper-full"; }
        }
        _ => container_class += "gdlr-core-container",
    }

    let mut inner_class = String::new();
    let mut inner_atts: StyleAtts = vec![];
    if enabled(&value, "enable-container-background") {
        inner_class = " gdlr-core-pbf-wrapper-container-inner".to_string();
        inner_atts = vec![
            ("margin".into(), if space_eq(get(&value, "container-margin"), "0px", "0px", "0px", "0px", "link") { json!([]) } else { get(&value, "container-margin").cloned().unwrap_or(json!([])) }),
            ("padding".into(), if space_eq(get(&value, "container-padding"), "0px", "0px", "0px", "0px", "link") { json!([]) } else { get(&value, "container-padding").cloned().unwrap_or(json!([])) }),
            ("background-color".into(), get(&value, "container-background-color").cloned().unwrap_or(Value::Null)),
            ("border-radius".into(), get(&value, "container-border-radius").cloned().unwrap_or(Value::Null)),
            ("background-shadow-size".into(), get(&value, "container-shadow-size").cloned().unwrap_or(Value::Null)),
            ("background-shadow-color".into(), get(&value, "container-shadow-color").cloned().unwrap_or(Value::Null)),
            ("background-shadow-opacity".into(), get(&value, "container-shadow-opacity").cloned().unwrap_or(Value::Null)),
            ("z-index".into(), get(&value, "container-z-index").cloned().unwrap_or(Value::Null)),
        ];
        let grad = s(&value, "container-background-gradient");
        if !s(&value, "container-background-color").is_empty() && !grad.is_empty() && grad != "none" {
            if grad == "top" || grad == "bottom" {
                let min_op = if s(&value, "container-background-gradient-min-opacity").is_empty() { json!(0) } else { json!(s(&value, "container-background-gradient-min-opacity")) };
                inner_atts.push(("gradient".into(), json!([
                    [s(&value, "container-background-color"), if grad == "top" { min_op.clone() } else { json!(1) }],
                    [s(&value, "container-background-color"), if grad == "bottom" { min_op } else { json!(1) }]
                ])));
            } else if (grad == "gradient" || grad == "gradient-v") && !s(&value, "container-background-gradient-color").is_empty() {
                inner_atts.push((grad.clone(), json!([
                    s(&value, "container-background-color"),
                    s(&value, "container-background-gradient-color")
                ])));
            }
        }
        let cm = obj(get(&value, "container-margin"));
        let (l, r) = (s(&cm, "left"), s(&cm, "right"));
        if (!l.is_empty() && l != "0px") || (!r.is_empty() && r != "0px") {
            let lw: i64 = l.trim_end_matches("px").parse().unwrap_or(0);
            let rw: i64 = r.trim_end_matches("px").parse().unwrap_or(0);
            inner_atts.push(("width".into(), json!(format!("calc(100% - {}px)", lw + rw))));
        }
    }

    wrapper += &format!("<div class=\"gdlr-core-page-builder-wrapper-content-margin {}\" {} >", esc(&s(&value, "class")), esc_style(&wrapper_style));
    wrapper += &wrapper_background;
    wrapper += &marquee;

    // wrapper gradients
    let wgrad = s(&value, "wrapper-background-gradient");
    if ["both", "top", "bottom"].contains(&wgrad.as_str()) {
        for dir in ["top", "bottom"] {
            if wgrad == "both" || wgrad == dir {
                let size_key = format!("wrapper-background-{dir}-gradient-size");
                let (a, b) = if dir == "top" { (json!(100), json!(0)) } else { (json!(0), json!(100)) };
                let color = if s(&value, "wrapper-background-gradient-color").is_empty() { "#000".into() } else { s(&value, "wrapper-background-gradient-color") };
                wrapper += &format!("<div class=\"gdlr-core-page-builder-wrapper-{dir}-gradient\" {} ></div>", esc_style(&[
                    ("height".into(), get(&value, &size_key).cloned().unwrap_or(json!(0))),
                    ("opacity".into(), get(&value, "wrapper-background-gradient-opacity").cloned().unwrap_or(Value::Null)),
                    ("gradient".into(), json!([[color, val_str(&a)], [color, val_str(&b)]])),
                    ("border-radius".into(), if empty_br(get(&value, "border-radius")) { Value::Null } else { get(&value, "border-radius").cloned().unwrap_or(Value::Null) }),
                ]));
            }
        }
    }

    if enabled(&value, "enable-background-image-overlay") {
        wrapper += &format!("<div class=\"gdlr-core-wrapper-bg-overlay gdlr-core-pos-{}\" {} ></div>",
            if s(&value, "background-image-overlay-section").is_empty() { "left".into() } else { esc(&s(&value, "background-image-overlay-section")) },
            esc_style(&[
                ("background-image".into(), Value::String(get(&value, "background-image-overlay").map(|v| image_url(conn, v)).unwrap_or_default())),
                ("margin".into(), if space_eq(get(&value, "background-image-overlay-margin"), "0px", "0px", "0px", "0px", "link") { json!([]) } else { get(&value, "background-image-overlay-margin").cloned().unwrap_or(json!([])) }),
            ]));
    }
    if enabled(&value, "enable-background-particle") {
        wrapper += "<div class=\"gdlr-core-js gdlr-core-particle-bg\" ></div>";
    }
    wrapper += &format!("<div class=\"gdlr-core-page-builder-wrapper-content-wrap gdlr-core-pbf-wrapper-content gdlr-core-js\" {} {} >", esc_style(&content_style), animation_atts(&value));
    wrapper += &format!("<div class=\"{} clearfix\" {}>", esc(&container_class), esc_style(&container_style));
    wrapper += &format!("<div class=\"gdlr-core-page-builder-wrapper-container gdlr-core-wrapper-sortable clearfix{}\" {}>", esc(&inner_class), esc_style(&inner_atts));
    if let Some(img) = val(&value, "container-background-image") {
        wrapper += &format!("<div class=\"gdlr-core-page-builder-wrapper-container-bg-image\" {} ></div>", esc_style(&[
            ("background-image".into(), Value::String(image_url(conn, img))),
            ("opacity".into(), get(&value, "container-background-image-opacity").cloned().unwrap_or(Value::Null)),
            ("border-radius".into(), get(&value, "container-border-radius").cloned().unwrap_or(Value::Null)),
        ]));
    }
    if let Some(items) = get(n, "items") {
        wrapper += &pb_items_scoped(conn, items);
    }
    wrapper += "</div></div></div></div></div>";
    wrapper += &format!("<script id=\"gdlr-core-preview-background-{}\" >\njQuery(document).ready(function(){{\n\tvar background_wrap = jQuery('#gdlr-core-preview-background-{}').parent();\n\tbackground_wrap.gdlr_core_parallax_background().gdlr_core_particle_background().gdlr_core_marquee().gdlr_core_fluid_video().gdlr_core_ux();\n\tjQuery(window).trigger('scroll');\n}});\n</script>", rnd_id(), rnd_id());
    wrapper += "</div>";
    wrapper
}

/// pb-wrapper-sidebar.php::get_template

fn sidebar_template(conn: &Connection, n: &Map<String, Value>) -> String {
    let value = obj(get(n, "value"));
    let sidebar = if s(&value, "sidebar").is_empty() { "right".into() } else { s(&value, "sidebar") };
    let sidebar_width: i64 = if s(&value, "sidebar-width").is_empty() { 20 } else { s(&value, "sidebar-width").trim_end_matches("px").parse().unwrap_or(20) };
    let mut container_width = 60;
    if sidebar == "left" || sidebar == "right" { container_width -= sidebar_width; }
    if sidebar == "both" { container_width -= 2 * sidebar_width; }

    let dv = if is_empty(get(n, "value")) { String::new() } else { format!("data-value=\"{}\" ", esc(&get(n, "value").unwrap().to_string())) };
    let mut wrapper = format!(
        "<div class=\"gdlr-core-page-builder-wrapper gdlr-core-page-builder-sidebar-wrapper\" data-template=\"wrapper\" data-type=\"sidebar\" data-sidebar=\"{}\" {dv}>",
        esc(&sidebar)
    );
    wrapper += "<div class=\"gdlr-core-page-builder-wrapper-content\"><div class=\"gdlr-core-page-builder-wrapper-head\"><span class=\"gdlr-core-page-builder-wrapper-head-title\" >Sidebar Wrapper</span><div class=\"gdlr-core-page-builder-wrapper-settings\"><i class=\"fa fa-edit gdlr-core-page-builder-wrapper-edit\" ></i><i class=\"fa fa-copy gdlr-core-page-builder-wrapper-copy\" ></i><i class=\"fa fa-download gdlr-core-page-builder-wrapper-save\" ></i><i class=\"fa fa-remove gdlr-core-page-builder-wrapper-remove\" ></i></div></div>";

    let container_style: StyleAtts = vec![
        ("margin".into(), if space_eq(get(&value, "margin"), "0px", "0px", "0px", "0px", "link") { Value::Null } else { get(&value, "margin").cloned().unwrap_or(Value::Null) }),
    ];
    let mut container_class = String::new();
    match s(&value, "content-layout").as_str() {
        "" | "boxed" => container_class += "gdlr-core-container",
        "full" => container_class += "gdlr-core-pbf-sidebar-wrapper-full",
        _ => container_class += "gdlr-core-container",
    }
    if !s(&value, "class").is_empty() { container_class += &format!(" {}", s(&value, "class")); }

    wrapper += &format!("<div class=\"gdlr-core-page-builder-wrapper-sidebar-container clearfix gdlr-core-line-height-0 gdlr-core-js {}\" {} >", esc(&container_class), esc_style(&container_style));

    let mut content_class = format!(" gdlr-core-column-{container_width} gdlr-core-pbf-sidebar-padding gdlr-core-line-height");
    if sidebar == "left" { content_class += " gdlr-core-column-extend-right"; }
    else if sidebar == "right" { content_class += " gdlr-core-column-extend-left"; }
    if s(&value, "background-area") == "item" {
        wrapper += &sidebar_background(conn, &value);
    }
    wrapper += &format!("<div class=\"gdlr-core-page-builder-wrapper-sidebar-content {}\" {} >", esc(&content_class), esc_style(&[
        ("padding".into(), if space_eq(get(&value, "padding"), "60px", "0px", "30px", "0px", "unlink") { Value::Null } else { get(&value, "padding").cloned().unwrap_or(Value::Null) }),
    ]));
    if s(&value, "background-area").is_empty() {
        wrapper += &sidebar_background(conn, &value);
    }
    wrapper += &format!("<div class=\"gdlr-core-page-builder-wrapper-sidebar-content-inner gdlr-core-wrapper-sortable clearfix\" {} >",
        if s(&value, "skin").is_empty() { String::new() } else { format!("data-skin=\"{}\" ", esc(&s(&value, "skin"))) });
    if let Some(items) = get(n, "items") {
        wrapper += &pb_items_scoped(conn, items);
    }
    wrapper += "</div></div>";

    let sidebar_class = format!(" gdlr-core-column-{sidebar_width} gdlr-core-pbf-sidebar-padding  gdlr-core-line-height");
    for side in ["left", "right"] {
        if sidebar == side || sidebar == "both" {
            wrapper += &format!("<div class=\"gdlr-core-page-builder-wrapper-sidebar-{side} gdlr-core-column-extend-{side} {}\" {}{} >", esc(&sidebar_class),
                esc_style(&[("padding".into(), if space_eq(get(&value, "sidebar-padding"), "60px", "0px", "30px", "0px", "unlink") { Value::Null } else { get(&value, "sidebar-padding").cloned().unwrap_or(Value::Null) })]),
                if s(&value, "sticky-sidebar-top-offset").is_empty() { String::new() } else { format!("data-sticky-offset=\"{}\" ", esc(&s(&value, "sticky-sidebar-top-offset"))) });
            if let Some(c) = val(&value, "sidebar-background-color") {
                wrapper += &format!("<div class=\"gdlr-core-pbf-background-wrap\" {} ></div>", esc_style(&[("background-color".into(), c.clone())]));
            }
            wrapper += "<div class=\"gdlr-core-sidebar-inner-bg gdlr-core-item-mglr\"></div></div>";
        }
    }
    wrapper += "</div></div>";
    wrapper += &format!("<script id=\"gdlr-core-preview-sidebar-background-{}\" >\njQuery(document).ready(function(){{\n\tvar background_wrap = jQuery('#gdlr-core-preview-sidebar-background-{}').parent();\n\tnew gdlr_core_sidebar_wrapper( background_wrap );\n\tbackground_wrap.gdlr_core_parallax_background().gdlr_core_ux();\n\tjQuery(window).trigger('scroll');\n}});\n</script>", rnd_id(), rnd_id());
    wrapper += "</div>";
    wrapper
}

/// pb-wrapper-sidebar.php::get_background (shared 4-type background switch)

fn sidebar_background(conn: &Connection, value: &Map<String, Value>) -> String {
    let bg_type = s(value, "background-type");
    if bg_type.is_empty() { return String::new(); }
    if bg_type == "color" {
        if let Some(c) = val(value, "background-color") {
            return format!("<div class=\"gdlr-core-pbf-background-wrap\" {} ></div>", esc_style(&[("background-color".into(), c.clone())]));
        }
        return String::new();
    }
    if bg_type == "image" {
        let img = get(value, "background-image").map(|v| image_url(conn, v)).unwrap_or_default();
        let mut bgi: StyleAtts = vec![("background-image".into(), Value::String(img))];
        let img_style = s(value, "background-image-style");
        if !img_style.is_empty() {
            if img_style == "cover" { bgi.push(("background-size".into(), json!("cover"))); }
            else { bgi.push(("background-repeat".into(), json!(img_style.clone()))); }
            let pos = s(value, "background-image-position");
            if !pos.is_empty() { bgi.push(("background-position".into(), json!(pos.replace('-', " ")))); }
        }
        return format!("<div class=\"gdlr-core-pbf-background-wrap\"><div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} data-parallax-speed=\"{}\"  ></div></div>",
            esc_style(&bgi), if s(value, "parallax-speed").is_empty() { "0".into() } else { s(value, "parallax-speed") });
    }
    if bg_type == "pattern" {
        let pattern = if s(value, "background-pattern").is_empty() { "pattern-1".into() } else { s(value, "background-pattern") };
        let bgi: StyleAtts = vec![
            ("background-image".into(), json!(format!("{GDLR_URL}/include/images/pattern/{pattern}.png"))),
            ("background-repeat".into(), json!("repeat")),
            ("background-position".into(), json!("center")),
            ("opacity".into(), if s(value, "pattern-opacity").is_empty() { json!("1") } else { get(value, "pattern-opacity").cloned().unwrap() }),
        ];
        return format!("<div class=\"gdlr-core-pbf-background-wrap\"><div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} data-parallax-speed=\"{}\"  ></div></div>",
            esc_style(&bgi), if s(value, "parallax-speed").is_empty() { "0".into() } else { s(value, "parallax-speed") });
    }
    // video
    let fallback = get(value, "background-video-image").map(|v| image_url(conn, v)).unwrap_or_default();
    let mut out = format!("<div class=\"gdlr-core-pbf-background-wrap\"><div class=\"gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js\" {} data-parallax-speed=\"{}\"  data-video-fallback=\"{}\"  >",
        esc_style(&[("background-position".into(), json!("center")), ("background-size".into(), json!("cover"))]),
        if s(value, "parallax-speed").is_empty() { "0".into() } else { s(value, "parallax-speed") }, esc(&fallback));
    out += "<div class=\"gdlr-core-pbf-background-video\" data-background-type=\"video\" >";
    if bg_type == "video" {
        if let Some(u) = val(value, "background-video-url") {
            out += &pb_video(&val_str(u), "100%", "100%", true);
        }
    } else if bg_type == "html5-video" {
        out += "<video autoplay loop muted >";
        for (k, mime) in [("background-video-url-mp4", "video/mp4"), ("background-video-url-webm", "video/webm"), ("background-video-url-ogg", "video/ogg")] {
            if let Some(u) = val(value, k) {
                out += &format!("<source src=\"{}\" type=\"{mime}\">", esc(&val_str(u)));
            }
        }
        out += "</video>";
    }
    out += "</div></div></div>";
    out
}

/// gdlr_core_get_video — youtube/vimeo/self-hosted iframe (background mode).

fn pb_video(video: &str, width: &str, height: &str, background: bool) -> String {
    if video.contains("youtube") || video.contains("youtu.be") {
        let id = if video.contains("youtube") {
            video.split(['?', '&']).find_map(|p| p.strip_prefix("v=")).unwrap_or("").to_string()
        } else {
            video.split("youtu.be/").nth(1).unwrap_or("").split(['?', '&', '/']).next().unwrap_or("").to_string()
        };
        let mut url = format!("//www.youtube-nocookie.com/embed/{id}?wmode=transparent");
        if background {
            url += &format!("&autoplay=1&controls=0&showinfo=0&rel=0&enablejsapi=1&loop=1&playlist={id}");
        }
        return format!("<iframe src=\"{}\" width=\"{}\" height=\"{}\" data-player-type=\"youtube\" allowfullscreen ></iframe>", esc(&url), esc(width), esc(height));
    }
    if video.contains("vimeo") {
        let url = if video.contains("player.vimeo") {
            video.to_string()
        } else {
            let id = video.split("vimeo.com/").nth(1).unwrap_or("").chars().take_while(|c| c.is_ascii_digit()).collect::<String>();
            format!("//player.vimeo.com/video/{id}?title=0&byline=0&portrait=0")
        };
        let url = if background { format!("{url}&autopause=0&autoplay=1&loop=1&api=1&background=1&muted=1") } else { url };
        return format!("<iframe src=\"{}\" width=\"{}\" height=\"{}\" data-player-type=\"vimeo\" allowfullscreen ></iframe>", esc(&url), esc(width), esc(height));
    }
    if video.starts_with("http://") || video.starts_with("https://") {
        return format!("<video src=\"{}\" width=\"{}\" height=\"{}\" controls muted loop playsinline></video>", esc(video), esc(width), esc(height));
    }
    String::new()
}

/* ================= gdlr_core_html_option::get_element ================= */


fn icon_font_title(t: &str) -> String {
    match t {
        "font-awesome" => "Font Awesome",
        "fa6" => "Font Awesome 6",
        "fa5" => "Font Awesome 5",
        "elegant-font" => "Elegant Font",
        "ionicons" => "Ionicons",
        "simpleline" => "Simple Line Icon",
        "gdlr-custom-icon" => "Custom Icon",
        "gdlr-cannabis" => "Custom Cannabis Icon",
        "gdlr-travel" => "Custom Travel Icon",
        _ => t,
    }.to_string()
}
// byte-wise encoding shared via `urlenc` (mod.rs) — `urlencoding_simple`
// encoded code points not UTF-8 bytes (M16-adjacent mojibake).


/// gdlr_core_page_builder_options::get_content — the options lightbox body.

fn section_head_list(conn: &Connection, sc: &SiteCtx, defs_e: &Value, defs_w: &Value, post_id: i64) -> (String, String) {
    // Returns (nav-items html, head-content html) in PHP section order:
    // wrapper, element, template, custom-template, sync-template.
    let mut nav = String::new();
    let mut content = String::new();
    let sections = [
        ("wrapper", "nav-wrapper.png", "Wrapper / Columns"),
        ("element", "nav-elements.png", "Elements"),
        ("template", "nav-template.png", "Templates"),
        ("custom-template", "nav-custom-template.png", "Custom Templates"),
        ("sync-template", "nav-sync-template.png", "Sync Templates"),
    ];
    for (slug, icon, title) in sections {
        nav += &format!(
            "<div class=\"gdlr-core-page-builder-head-nav-item {}\" data-nav-type=\"{slug}\" ><img src=\"{GDLR_URL}/framework/images/page-builder/{icon}\" />{}</div>",
            if slug == "wrapper" { "gdlr-core-active" } else { "" }, esc(title));
        let cls = format!("gdlr-core-page-builder-head-content-{slug}{}", if slug == "wrapper" { " gdlr-core-active" } else { "" });
        content += &format!("<div class=\"{cls}\" data-content-type=\"{slug}\" >");
        match slug {
            "wrapper" => {
                content += "<div class=\"gdlr-core-page-builder-head-content-wrapper-left\">";
                for wt in ["background", "sidebar", "flexbox"] {
                    let st = defs_w.get(wt).and_then(|w| w.get("settings")).cloned().unwrap_or(json!({}));
                    let thumb = val_str(st.get("thumbnail").unwrap_or(&Value::Null));
                    let thumb = if thumb.starts_with('/') { format!("{GDLR_URL}{thumb}") } else { format!("{GDLR_URL}/{thumb}") };
                    content += &format!("<div class=\"gdlr-core-page-builder-head-content-wrapper-item gdlr-core-pb-list-draggable\" data-template=\"wrapper\" data-type=\"{wt}\" ><div class=\"gdlr-core-page-builder-head-content-wrapper-item-thumbnail gdlr-core-media-image\" ><img src=\"{}\" alt=\"\" /></div><div class=\"gdlr-core-page-builder-head-content-wrapper-item-title\" >{}</div></div>",
                        esc(&thumb), esc(&val_str(st.get("title").unwrap_or(&json!(wt)))));
                }
                content += "</div><div class=\"gdlr-core-page-builder-head-content-wrapper-right\">";
                let col_settings = defs_w.get("column").and_then(|w| w.get("settings")).cloned().unwrap_or(json!({}));
                let img_dir = val_str(col_settings.get("image_dir").unwrap_or(&json!("")));
                if let Value::Object(list) = col_settings.get("list").cloned().unwrap_or(json!({})) {
                    for (num, size) in list {
                        let size = val_str(&size);
                        let img = format!("{GDLR_URL}{img_dir}/column-{}.png", size.trim().replace('/', "-"));
                        content += &format!("<div class=\"gdlr-core-page-builder-head-content-wrapper-item gdlr-core-pb-list-draggable\" data-template=\"wrapper\" data-type=\"column\" data-column=\"{num}\" ><div class=\"gdlr-core-page-builder-head-content-wrapper-item-thumbnail gdlr-core-media-image\" ><img src=\"{}\" alt=\"\" /></div><div class=\"gdlr-core-page-builder-head-content-wrapper-item-title\" >{}</div></div>", esc(&img), esc(&size));
                    }
                }
                content += "<div class=\"clear\"></div></div><div class=\"clear\"></div>";
            }
            "element" => {
                content += "<input type=\"text\" placeholder=\"Search Elements\" class=\"gdlr-core-page-builder-head-content-search\" />";
                content += "<div class=\"gdlr-core-page-builder-head-content-element-container\" >";
                if let Value::Object(els) = defs_e {
                    for (slug, el) in els {
                        let st = el.get("settings").cloned().unwrap_or(json!({}));
                        // default section hides elements registered for other
                        // areas (settings.type non-empty & not 'default')
                        if let Some(t) = st.get("type") {
                            let show = match t {
                                Value::String(s) => s == "default",
                                Value::Array(a) => a.iter().any(|x| val_str(x) == "default"),
                                _ => true,
                            };
                            if !show { continue; }
                        }
                        let title = val_str(st.get("title").unwrap_or(&json!("")));
                        let icon = val_str(st.get("icon").unwrap_or(&json!("")));
                        content += &format!("<div class=\"gdlr-core-page-builder-head-content-element-item gdlr-core-pb-list-draggable\" data-template=\"element\" data-type=\"{}\" data-title=\"{}\" >", esc(slug), esc(&title));
                        if !icon.is_empty() {
                            let icon_cls = if icon.starts_with("fa-") { format!("fa {icon}") } else { icon.clone() };
                        content += &format!("<div class=\"gdlr-core-page-builder-head-content-element-item-icon\" ><i class=\"{}\" ></i></div>", esc(&icon_cls));
                        }
                        content += &format!("<div class=\"gdlr-core-page-builder-head-content-element-item-title\" >{title}</div></div>");
                    }
                }
                content += "<div class=\"clear\"></div></div>";
            }
            "template" => {
                content += "<input type=\"text\" placeholder=\"Search Templates\" class=\"gdlr-core-page-builder-head-content-search\" />";
                content += "<div class=\"gdlr-core-page-builder-head-content-template-type\" >";
                content += "<div class=\"gdlr-core-template-type-button gdlr-core-active\" data-type=\"page\" >Pre-Built Pages</div>";
                content += "<div class=\"gdlr-core-template-type-button\" data-type=\"block\" >Pre-Built Blocks</div>";
                content += "</div>";
                for t in ["page", "block"] {
                    content += &format!("<div class=\"gdlr-core-page-builder-head-content-template-container {}\" data-type=\"{t}\" ><div class=\"clear\"></div></div>",
                        if t == "page" { "gdlr-core-active" } else { "" });
                }
            }
            "custom-template" | "sync-template" => {
                let which = slug;
                content += &format!("<input type=\"text\" placeholder=\"Search Templates\" class=\"gdlr-core-page-builder-head-content-search\" /><div class=\"gdlr-core-page-builder-head-content-{which}-container clearfix\" >");
                let key = if which == "custom-template" { "gdlr-core-pb-custom-template" } else { "gdlr-core-pb-sync-template" };
                if let Value::Object(list) = get_setting(conn, key) {
                    for (tslug, opt) in &list {
                        let title = val_str(opt.get("title").unwrap_or(&json!(tslug)));
                        let ttype = val_str(opt.get("type").unwrap_or(&json!("")));
                        content += &format!("<div class=\"gdlr-core-page-builder-head-content-{which}-item gdlr-core-pb-list-draggable\" data-template=\"{which}\" data-title=\"{}\" data-type=\"{}\" data-template-slug=\"{}\" ><span class=\"gdlr-core-page-builder-head-content-{which}-title\" >{title}</span><div class=\"gdlr-core-page-builder-head-content-{which}-remove\" ><i class=\"fa fa-remove\" ></i></div></div>",
                            esc(&title), esc(&ttype), esc(tslug));
                    }
                }
                content += "</div>";
            }
            _ => {}
        }
        content += "</div>";
    }
    let _ = (sc, post_id);
    (nav, content)
}


fn page_builder_var(defs_e: &Value, defs_w: &Value, post_id: i64) -> String {
    // wp_localize_script equivalent.
    let mut var = json!({
        "post_id": post_id,
        "settings": { "undo_times": 20 },
        "text": {
            "item_added": "Item Added!",
            "ajaxurl": "/admin/api/gdlr/ajax",
            "error_head": "An error occurs",
            "error_message": "Please refresh the page to try again. If the problem still persists, please contact administrator for this.",
            "undo_end": "Unable to undo changes",
            "redo_end": "Unable to redo changes",
        },
        "template": {}
    });
    let tpl = var.get_mut("template").unwrap().as_object_mut().unwrap();
    tpl.insert("template".into(), json!("<div class=\"gdlr-core-page-builder-template\" data-template=\"template\" ><div class=\"gdlr-core-page-builder-template-inner\" ></div></div>"));
    // element stub
    tpl.insert("element".into(), json!(concat!(
        "<div class=\"gdlr-core-page-builder-item\" data-template=\"element\" >",
        "<div class=\"gdlr-core-page-builder-item-content\">",
        "<div class=\"gdlr-core-page-builder-item-head clearfix\">",
        "<span class=\"gdlr-core-page-builder-item-head-item-title\"></span>",
        "<i class=\"fa fa-remove gdlr-core-page-builder-item-remove\" ></i>",
        "<i class=\"fa fa-copy gdlr-core-page-builder-item-copy\" ></i>",
        "<i class=\"fa fa-download gdlr-core-page-builder-item-save\" ></i>",
        "<i class=\"fa fa-edit gdlr-core-page-builder-item-edit\" ></i>",
        "</div>",
        "<div class=\"gdlr-core-page-builder-item-container\">",
        "<div class=\"gdlr-core-page-builder-item-container-item\">",
        "<i class=\"fa fa-align-justify gdlr-core-page-builder-item-container-item-icon\"></i>",
        "<span class=\"gdlr-core-page-builder-item-container-item-title\"></span>",
        "</div>",
        "<div class=\"gdlr-core-page-builder-item-container-preview\">",
        "<div class=\"gdlr-core-page-builder-item-container-overlay\"></div>",
        "</div>",
        "<i class=\"fa fa-edit gdlr-core-page-builder-item-edit\" ></i>",
        "</div></div></div>")));
    // wrapper stubs: each pb-wrapper class get_template() with no options.
    // JS only uses these when dragging *without* column attr? In practice the
    // JS builds wrapper via column template clone; still provide empty stubs
    // for each registered wrapper type.
    for wt in ["background", "sidebar", "column", "flexbox"] {
        tpl.insert(wt.into(), json!(format!("<div class=\"gdlr-core-page-builder-wrapper\" data-template=\"wrapper\" data-type=\"{wt}\" ></div>")));
    }
    tpl.insert("custom_template_lb".into(), json!(concat!(
        "<div class=\"gdlr-core-custom-template-lb-content-wrapper\">",
        "<div class=\"gdlr-core-custom-template-lb-content\">",
        "<div class=\"gdlr-core-custom-template-lb-head\">",
        "<i class=\"fa fa-save\" ></i>",
        "<span class=\"gdlr-core-head\" >Save Custom Template</span>",
        "<div class=\"gdlr-core-custom-template-lb-head-close\" ></div>",
        "</div>",
        "<div class=\"gdlr-core-custom-template-lb-body\">",
        "<div class=\"gdlr-core-custom-template-name-title\" >Template Name :</div>",
        "<input type=\"text\" class=\"gdlr-core-custom-template-name\" />",
        "<div class=\"clear\" ></div>",
        "<div class=\"gdlr-core-custom-template-save\" >Save Template</div>",
        "</div></div></div>")));
    tpl.insert("custom_template_no_text_head".into(), json!("Undefined Template Name"));
    tpl.insert("custom_template_no_text_message".into(), json!("Please fill in the template name to proceed."));
    var.as_object_mut().unwrap().insert("sync_template".into(), json!({
        "sync_template_remove_head": "You are removing synced wrapper!",
        "sync_template_remove_message": "Please note that by doing this, all synced wrapper will be unsynced and they will be independent. Please make sure before doing this. This process can't be undone."
    }));
    // column width list
    let mut col_list = Vec::new();
    if let Some(Value::Object(list)) = defs_w.get("column").and_then(|w| w.get("settings").and_then(|s| s.get("list"))) {
        for (n, s) in list {
            col_list.push((n.parse::<u32>().unwrap_or(0), val_str(s)));
        }
        col_list.sort();
    }
    let col_map: Map<String, Value> = col_list.into_iter().map(|(n, s)| (n.to_string(), json!(s))).collect();
    var.as_object_mut().unwrap().insert("column".into(), Value::Object(col_map));
    let _ = defs_e;
    var.to_string()
}

/// Metabox DOM + scripts for /posts/:id/edit when body is gdlr (or new).

pub(crate) fn boot(conn: &Connection, sc: &SiteCtx, post_id: i64, body_json: &str, csrf: &str) -> String {
    let (defs_e, defs_w) = defs(sc);
    PB_DEFS.with(|d| *d.borrow_mut() = defs_e.clone());
    PB_TOUR_BASE.with(|t| *t.borrow_mut() = crate::admin::tour_base(sc).to_string());

    let saved: Value = serde_json::from_str(body_json).unwrap_or(json!([]));
    let items_html = pb_items(conn, &defs_e, &saved);
    let (nav, content) = section_head_list(conn, sc, &defs_e, &defs_w, post_id);
    let view_mode = "live";

    let mut out = String::new();
    out += &format!("<input type=\"hidden\" id=\"page_builder_security\" value=\"{}\"/>", esc(csrf));
    out += &format!("<input type=\"hidden\" id=\"gdlr-core-page-builder-val\" name=\"gdlr-core-page-builder\" value=\"{}\"/>", esc(&saved.to_string()));
    out += "<div class=\"gdlr-core-page-builder\" id=\"gdlr-core-page-builder\" >";
    out += "<div class=\"gdlr-core-page-builder-head-sub\"></div>";
    out += "<div class=\"gdlr-core-page-builder-head\">";
    out += "<div class=\"gdlr-core-page-builder-head-nav\" id=\"gdlr-core-page-builder-head-nav\">";
    out += &nav;
    out += "<div class=\"gdlr-core-page-builder-head-full-screen\" id=\"gdlr-core-page-builder-head-full-screen\"  ></div>";
    out += &format!("<div class=\"gdlr-core-page-builder-head-nav-update\" id=\"gdlr-core-page-builder-head-nav-update\" data-post-id=\"{post_id}\" ><i class=\"fa fa-save\"></i>Save PB</div>");
    out += &format!("<div class=\"gdlr-core-page-builder-head-action-live-mode-alt\" id=\"gdlr-core-page-builder-head-action-live-mode-alt\" data-mode=\"{view_mode}\" ><span class=\"gdlr-core-page-builder-head-action-live-mode-button gdlr-core-active\" data-live-mode=\"edit\" >Edit</span><span class=\"gdlr-core-page-builder-head-action-live-mode-button\" data-live-mode=\"preview\" >Preview</span></div>");
    out += &format!("<div class=\"gdlr-core-page-builder-head-action-mode-section-alt\" id=\"gdlr-core-page-builder-head-action-mode-section\" data-mode=\"{view_mode}\" ><div class=\"gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-block\" data-mode=\"block\" ><i class=\"fa fa-th-large\" ></i><span class=\"gdlr-core-head\">Block Mode</span></div><div class=\"gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-preview\" data-mode=\"preview\" ><i class=\"fa fa-eye\" ></i><span class=\"gdlr-core-head\">Preview Mode</span></div><div class=\"gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-live\" data-mode=\"live\" ><i class=\"fa fa-tv\" ></i><span class=\"gdlr-core-head\">Live Mode</span></div></div>");
    out += "<div class=\"gdlr-core-page-builder-head-action-repeat-alt\" id=\"gdlr-core-page-builder-head-action-redo-alt\" ><i class=\"fa fa-repeat\" ></i></div>";
    out += "<div class=\"gdlr-core-page-builder-head-action-repeat-alt\" id=\"gdlr-core-page-builder-head-action-undo-alt\" ><i class=\"fa fa-undo\" ></i></div>";
    out += "<div class=\"clear\"></div>";
    out += "</div>"; // gdlr-core-page-builder-head-nav
    out += "<div class=\"gdlr-core-page-builder-head-content\" id=\"gdlr-core-page-builder-head-content\" >";
    out += &content;
    out += "</div>";
    // action bar
    out += "<div class=\"gdlr-core-page-builder-head-action-bar\" >";
    out += "<div class=\"gdlr-core-page-builder-head-action-bar-left\" >";
    out += "<div class=\"gdlr-core-page-builder-head-action-button\" id=\"gdlr-core-page-builder-head-action-reset\" ><i class=\"fa fa-remove\" ></i>Reset</div>";
    out += "<div class=\"gdlr-core-page-builder-head-action-button\" id=\"gdlr-core-page-builder-head-action-save-template\" ><i class=\"fa fa-download\" ></i>Save Template</div>";
    out += "<div class=\"clear\"></div></div>";
    out += "<div class=\"gdlr-core-page-builder-head-action-bar-right\" >";
    out += &format!("<div class=\"gdlr-core-page-builder-head-action-mode-section\" data-mode=\"{view_mode}\" ><div class=\"gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-block\" data-mode=\"block\" ><i class=\"fa fa-th-large\" ></i><span class=\"gdlr-core-head\">Block Mode</span></div><div class=\"gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-preview\" data-mode=\"preview\" ><i class=\"fa fa-eye\" ></i><span class=\"gdlr-core-head\">Preview Mode</span></div><div class=\"gdlr-core-page-builder-head-action-mode gdlr-core-page-builder-head-action-mode-live\" data-mode=\"live\" ><i class=\"fa fa-tv\" ></i><span class=\"gdlr-core-head\">Live Mode</span></div></div>");
    out += "<div class=\"gdlr-core-page-builder-head-action-live-mode\" id=\"gdlr-core-page-builder-head-action-live-mode\" ><span class=\"gdlr-core-page-builder-head-action-live-mode-button gdlr-core-active\" data-live-mode=\"edit\" >Edit</span><span class=\"gdlr-core-page-builder-head-action-live-mode-button\" data-live-mode=\"preview\" >Preview</span></div>";
    out += "<div class=\"gdlr-core-page-builder-head-action-repeat\" id=\"gdlr-core-page-builder-head-action-undo\" ><i class=\"fa fa-undo\" ></i></div>";
    out += "<div class=\"gdlr-core-page-builder-head-action-repeat\" id=\"gdlr-core-page-builder-head-action-redo\" ><i class=\"fa fa-repeat\" ></i></div>";
    out += "<div class=\"clear\"></div></div><div class=\"clear\"></div></div>";
    out += "</div>"; // head
    out += "<div class=\"gdlr-core-page-builder-body gdlr-core-no-dragging gdlr-core-body gdlr-core-pb-livemode gdlr-core-live-edit traveltour-body traveltour-body-front\" id=\"gdlr-core-page-builder-body\" data-mode=\"live\" >";
    out += "<div class=\"gdlr-core-page-builder-container gdlr-core-container gdlr-container-sortable clearfix\" id=\"gdlr-core-page-builder-container\" >";
    out += &items_html;
    out += "</div></div></div>";

    // page_builder_var + boot assets (scripts are <link>/<script> tags the
    // caller embeds into the admin layout).
    let icons_val = icons(sc);
    out += &format!("<script>var page_builder_var = {};var html_option_val = {};var gdlr_core_tmce = {};</script>",
        page_builder_var(&defs_e, &defs_w, post_id), html_option_var(&icons_val, csrf),
        serde_json::to_string(&tmce_init_html()).unwrap_or_default());
    out
}

/// Asset tags — appended once into the post-edit page head.
/// jQuery already loaded by html_admin; load only PB-specific deps.

pub(crate) fn asset_tags() -> String {
    let g = GDLR_URL;
    format!(
        r#"<script>
/* Live Mode: the admin canvas (~860px) is narrower than the site's 1350px
   container; zoom it so the preview matches the frontend layout. */
function gdlrLiveScale(){{
  var body=document.getElementById('gdlr-core-page-builder-body');
  var cont=document.getElementById('gdlr-core-page-builder-container');
  if(!body||!cont)return;
  if(body.classList.contains('gdlr-core-pb-livemode')){{
    var s=Math.min(1,body.clientWidth/1350);
    cont.style.width='1350px';cont.style.zoom=s;
  }}else{{cont.style.width='';cont.style.zoom='';}}
}}
window.addEventListener('resize',gdlrLiveScale);
function gdlrLiveInit(){{
  gdlrLiveScale();
  var pb=document.getElementById('gdlr-core-page-builder');
  if(pb)pb.addEventListener('click',function(){{setTimeout(gdlrLiveScale,50);}});
}}
if(document.readyState==='loading'){{document.addEventListener('DOMContentLoaded',gdlrLiveInit);}}else{{gdlrLiveInit();}}
</script>
<link rel="stylesheet" href="{g}/plugins/fontawesome/font-awesome.css"/>
<link rel="stylesheet" href="{g}/plugins/elegant/elegant-font.css"/>
<link rel="stylesheet" href="{g}/plugins/style.css"/>
<link rel="stylesheet" href="{g}/framework/css/utility.css"/>
<link rel="stylesheet" href="{g}/framework/css/html-option-small.css"/>
<link rel="stylesheet" href="{g}/framework/css/page-builder.css"/>
<link rel="stylesheet" href="{g}/include/css/page-builder.css"/>
<link rel="stylesheet" href="{g}/tourmaster.css"/>
<link rel="stylesheet" href="/admin/front-css/gdlr-travel.scoped.css"/>
<link rel="stylesheet" href="/admin/front-css/tourmaster-global-style-custom.scoped.css"/>
<link rel="stylesheet" href="/admin/front-css/style-core.scoped.css"/>
<link rel="stylesheet" href="/admin/front-css/traveltour-style-custom.scoped.css"/>
<link rel="stylesheet" href="/admin/front-css/tourmaster-style-custom.scoped.css"/>
<link rel="stylesheet" href="/admin/wp/color-picker.min.css"/>
<script>var wp=window.wp||{{}};wp.i18n={{__:function(s){{return s;}},_x:function(s){{return s;}},sprintf:function(f){{var a=[].slice.call(arguments,1);var i=0;return f.replace(/%[sd]/g,function(){{return String(a[i++]);}});}}}};</script>
<script src="/admin/wp/jquery-ui.min.js"></script>
<script src="/admin/wp/iris.min.js"></script>
<script src="/admin/wp/color-picker.min.js"></script>
<script src="/admin/wp/gdlr/jquery-ready-shim.js"></script>
<script src="{g}/framework/js/utility.js"></script>
<script>var gdlr_core_pbf = {{"admin":true,"video":{{"width":"640","height":"360"}},"ajax_url":"/admin/api/gdlr/ajax","ilightbox_skin":"dark"}};</script>
<script src="{g}/plugins/script.js"></script>
<script src="{g}/include/js/page-builder.js"></script>
<script src="{g}/framework/js/html-option.js"></script>
<script src="/admin/wp/gdlr/wp-media-shim.js"></script>
<script src="/admin/wp/gdlr/adapter.js"></script>
<script src="{g}/framework/js/page-builder.js"></script>"#,
        g = g
    )
}

/* ================= nested form parser (jQuery $.ajax data -> map) ========= */

/// Parse `security=x&action=y&value%5Ba%5D=z` into nested Value.

fn icons_js_html(icons_val: &Value) -> String {
    let mut ret = String::from("<div class=\"gdlr-core-custom-combobox gdlr-core-html-option-icons-type-combobox\" ><select class=\"gdlr-core-html-option-combobox gdlr-core-html-option-icons-type-select\" >");
    let types: Vec<String> = icons_val.as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default();
    for t in &types {
        ret += &format!("<option value=\"{}\" >{}</option>", esc(t), esc(&icon_font_title(t)));
    }
    ret += "</select></div>";
    ret += "<input type=\"text\" class=\"gdlr-core-html-option-text gdlr-core-html-option-icons-search\" placeholder=\"Search Icons\" />";
    ret += "<div class=\"gdlr-core-html-option-icons-type-wrapper\" >";
    let mut first = true;
    for t in &types {
        ret += &format!("<div class=\"gdlr-core-html-option-icons-type {}\" data-icon-type=\"{}\" >", if first { "gdlr-core-active" } else { "" }, esc(t));
        if let Some(Value::Array(l)) = icons_val.get(t) {
            for i in l {
                ret += &format!("<i class=\"{}\" ></i>", esc(&val_str(i)));
            }
        }
        ret += "</div>";
        first = false;
    }
    ret += "</div>";
    ret
}


fn html_option_var(icons_val: &Value, csrf: &str) -> String {
    json!({
        "icon_list": icons_js_html(icons_val),
        "text": {
            "ajaxurl": "/admin/api/gdlr/ajax",
            "error_head": "An error occurs",
            "error_message": "Please refresh the page to try again. If the problem still persists, please contact administrator for this.",
            "nonce": csrf,
            "upload_media": "Select or Upload Media",
            "choose_media": "Use this media"
        },
        "tabs": {
            "title_text": "Item's Title",
            "tab_checkbox_on": "On",
            "tab_checkbox_off": "Off"
        },
        "skin": {
            "input": "Skin Name",
            "empty_input": "Please fill the name in skin name box to create new skin.",
            "duplicate_input": "This skin name has already been assigned, please try filling another name.",
            "description": "* Please fill english character for skin name with no special characters. The skin you're created can be used in Color/Background Wrapper Section"
        },
        "fontupload": {
            "none": "You don't have any font uploaded",
            "font_name": "Font Name",
            "font_name_p": "Fill in font name in English",
            "eot": "EOT Font", "ttf": "TTF Font", "woff": "Woff Font",
            "font_weight": "Font Weight", "font_style": "Font Style",
            "button": "Upload"
        },
        "thumbnail_sizing": {
            "name": "Thumbnail Name", "width": "Width (px)", "height": "Height (px)",
            "add": "Add Thumbnail",
            "empty_input": "Please fill all required fields",
            "description": "*After creating new thumbnail, you have to regenerate the thumbnail for old images."
        }
    }).to_string()
}

/// gdlr_core_html_option::tmce_init — editor wrap HTML for the dynamic tinymce
/// field, plus a tinyMCEPreInit config matching the post editor's.

fn tmce_init_html() -> String {
    let mut s = String::new();
    s += "<div id=\"wp-gdlr_core_tmce-wrap\" class=\"wp-core-ui wp-editor-wrap html-active\">";
    s += "<div id=\"wp-gdlr_core_tmce-editor-tools\" class=\"wp-editor-tools hide-if-no-js\">";
    s += "<div id=\"wp-gdlr_core_tmce-media-buttons\" class=\"wp-media-buttons\">";
    s += "<a href=\"#\" class=\"button insert-media add_media\" data-editor=\"gdlr_core_tmce\" title=\"Add Media\"><span class=\"wp-media-buttons-icon\"></span> Add Media</a>";
    s += "</div>";
    s += "<div class=\"wp-editor-tabs\">";
    s += "<button type=\"button\" id=\"gdlr_core_tmce-tmce\" class=\"wp-switch-editor switch-tmce\" data-wp-editor-id=\"gdlr_core_tmce\">Visual</button>";
    s += "<button type=\"button\" id=\"gdlr_core_tmce-html\" class=\"wp-switch-editor switch-html\" data-wp-editor-id=\"gdlr_core_tmce\">Text</button>";
    s += "</div></div>";
    s += "<div id=\"wp-gdlr_core_tmce-editor-container\" class=\"wp-editor-container\">";
    s += "<div id=\"qt_gdlr_core_tmce_toolbar\" class=\"quicktags-toolbar\"></div>";
    s += "<textarea style=\"height:300px;\" class=\"wp-editor-area\" autocomplete=\"off\" cols=\"40\" name=\"gdlr_core_tmce\" id=\"gdlr_core_tmce\"></textarea>";
    s += "</div></div>";
    s
}

