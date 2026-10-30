//! Verbatim port of `tourmaster_html_option::get_element` and its per-type
//! renderers — `reference/tourmaster/framework/function/html-option.php`
//! lines 51–642 — plus the tab iteration of
//! `tourmaster_page_option::get_option_tab`
//! (`reference/tourmaster/framework/function/page-option.php` lines 106–124).
//!
//! What the PHP emits vs. what the served JS (`templates/admin/wp/tourmaster/
//! framework/js/html-option.js`, read by `get_val()` at lines 491–528) needs:
//!  - every scalar input carries `data-type` + `data-slug`; values ride on the
//!    element (`val()`, `:checked`), never a `name` attr (PHP only emits `name`
//!    when `with-name` is set, which the tour meta box never does).
//!  - `custom` fields emit ONLY the container div plus
//!    `.tourmaster-html-option-custom-{settings,options,value}` `data-value`
//!    JSON spans (PHP lines 584–608); `html-option.js` `rebind()` (lines
//!    464–486) reads those and builds tabs/gallery/group-discount DOM
//!    client-side. PHP has no server-side item renderer (`get_tabs_item`
//!    exists only as JS `tourmaster_tabs` templates, js lines 967–1157), so
//!    none is ported.
//!  - top-level `title` / `description` / `checkboxes` / `tabs` field types
//!    have NO case in the PHP switch (they exist only inside `custom` option
//!    definitions); they fall through to the wrapper + title/description
//!    divs exactly like PHP's `default: break;`.
//!
//! Context the PHP gets from `get_the_ID()` (post id for ajax buttons) and
//! `add_query_arg('download_tour_ical', …)` (the `{ical_url}` placeholder in
//! `ical-description`) is supplied by `set_context()` — the parent calls it
//! once per render before `render_tab`.

use serde_json::{Map, Value};
use std::cell::RefCell;

use super::schema::{Field, Tab};
use crate::admin::esc;

#[derive(Default)]
struct Ctx {
    post_id: String,
    ical_url: String,
    /// Registered sidebars for `options_source` sidebar/sidebar-default
    /// comboboxes (set per metabox render; empty ⇒ legacy none/default).
    sidebars: Vec<(String, String)>,
}

thread_local! {
    static CTX: RefCell<Ctx> = RefCell::new(Ctx::default());
}

/// Per-render context the PHP pulls from globals:
/// `post_id` → `get_the_ID()` for `data-post-id` on ajax buttons
/// (html-option.php:213,524); `ical_url` → substitutes the `{ical_url}`
/// placeholder baked into `ical-description`'s description
/// (tour-option.php:186,359–361).
#[allow(dead_code)]
pub fn set_context(post_id: &str, ical_url: &str) {
    CTX.with(|c| {
        c.borrow_mut().post_id = post_id.to_string();
        c.borrow_mut().ical_url = ical_url.to_string();
    });
}

/// Feed the registered-sidebar list for the next metabox render
/// (`options: "sidebar"` comboboxes enumerate the widget registry; the
/// radioimage `'options' => 'sidebar'` layout picker is untouched).
pub fn set_sidebars(list: &[(String, String)]) {
    CTX.with(|c| {
        c.borrow_mut().sidebars = list.to_vec();
    });
}

/// page-option.php `get_option_tab` lines 108–124: the per-tab field loop.
/// `values` is the merged `tourmaster-tour-option` dict (single-meta values
/// already merged in by the caller — PHP's `$option['value']` resolution,
/// page-option.php:111–116).
pub fn render_tab(t: &Tab, values: &Map<String, Value>) -> String {
    let mut out = String::new();
    for f in t.fields {
        let v = values.get(f.slug).cloned().unwrap_or(Value::Null);
        out.push_str(&render_field(f, &v));
    }
    out
}

/// html-option.php `get_element` lines 51–181.
pub fn render_field(f: &Field, value: &Value) -> String {
    if f.ftype.is_empty() || f.ftype == "customizer-description" {
        return String::new();
    }

    // column opening — lines 56–70
    if f.ftype == "column" {
        let mut column_class = format!(
            "tourmaster-column-{} ",
            if f.column_size == 0 { 6 } else { f.column_size }
        );
        if f.bottom_divider {
            column_class.push_str("tourmaster-column-bottom-divider");
        }
        let mut ret = format!("<div class=\"{}\">", esc(&column_class));
        if !f.right_divider.is_empty() {
            ret.push_str("<div class=\"tourmaster-column-right-divider\" ></div>");
            // PHP emits the icon unless right-divider === true
            if f.right_divider != "true" {
                ret.push_str(&format!(
                    "<i class=\"tourmaster-column-right-divider-icon {}\" ></i>",
                    esc(f.right_divider)
                ));
            }
        }
        return ret;
    }

    // closing column — lines 73–77
    if f.ftype == "column-close" {
        let mut ret = String::from("</div>");
        if f.clear {
            ret.push_str("<div class=\"clear\"></div>");
        }
        return ret;
    }

    // normal elements — lines 80–180
    let mut wrapper_class = f.wrapper_class.to_string();
    wrapper_class.push_str(" tourmaster-html-option-");
    wrapper_class.push_str(f.ftype.trim());
    if f.ftype == "custom" {
        if let Some(it) = f.item_type {
            wrapper_class.push_str(" tourmaster-html-option-custom-");
            wrapper_class.push_str(it);
        }
    }

    // f.condition is already raw json_encode output — emit verbatim.
    // (`condition-wrapper` has no Field prop; unused by the tour schema.)
    let mut condition = String::new();
    if let Some(c) = f.condition {
        condition = format!("data-condition=\"{}\" ", esc(c));
    }

    let mut ret = format!(
        "<div class=\"tourmaster-html-option-item {}-item\" {} >",
        esc(&wrapper_class),
        condition
    );

    if !f.title.is_empty() {
        ret.push_str(&format!(
            "<div class=\"tourmaster-html-option-item-title\" >{}</div>",
            f.title
        ));
    }

    let is_group_discount = f.ftype == "custom" && f.item_type == Some("group-discount");
    if !f.description.is_empty() && is_group_discount {
        ret.push_str(&format!(
            "<div class=\"tourmaster-html-option-item-description\" >{}</div>",
            describe(f.description)
        ));
    }

    ret.push_str("<div class=\"tourmaster-html-option-item-input\">");
    ret.push_str(&render_inner(f, value));
    ret.push_str("</div>");

    if !f.description.is_empty() && !is_group_discount {
        ret.push_str(&format!(
            "<div class=\"tourmaster-html-option-item-description\" >{}</div>",
            describe(f.description)
        ));
    }

    // PHP `$settings['options'] == 'skin'` — carried by options_source.
    if f.options_source == Some("skin") {
        ret.push_str("<div class=\"tourmaster-html-option-skin-edit\" >Create Skin<i class=\"fa fa-plus-circle\" ></i></div>");
    }

    ret.push_str("<div class=\"clear\"></div>");
    ret.push_str("</div>");
    ret
}

/// Description text is emitted raw (PHP prints it unescaped); only the
/// `{ical_url}` placeholder is substituted.
fn describe(d: &str) -> String {
    if d.contains("{ical_url}") {
        CTX.with(|c| d.replace("{ical_url}", &c.borrow().ical_url))
    } else {
        d.to_string()
    }
}

/// The `switch($settings['type'])` body — html-option.php lines 106–165.
fn render_inner(f: &Field, value: &Value) -> String {
    match f.ftype {
        "text" => text(f, value),
        "button" => button(f),
        "time" => time(f, value),
        "datepicker" => datepicker(f, value),
        "textarea" => textarea(f, value),
        "combobox" => combobox(f, value),
        "multi-combobox" => multi_combobox(f, value),
        "checkbox" => checkbox(f, value),
        "radioimage" => radioimage(f, value),
        "upload" => upload(f, value),
        "colorpicker" => colorpicker(f, value),
        "fontslider" => fontslider(f, value),
        "custom" => custom(f, value),
        "manage-review" | "add-review" => review_notice(),
        _ => String::new(),
    }
}

/// Non-goal: no ajax review CRUD. PHP's `manage_review`/`add_review`
/// (html-option.php:505–542) queried bookings; we emit the same "no review"
/// wrapper class with the not-migrated notice.
fn review_notice() -> String {
    "<div class=\"tourmaster-html-option-admin-no-review\" >Reviews are managed on the bookings screen (not migrated).</div>"
        .to_string()
}

/// JSON value → the PHP string form ($settings['value'] is usually a string).
fn val_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// PHP `isset($settings['value'])` — Null counts as unset.
fn set(v: &Value) -> bool {
    !v.is_null()
}

/// PHP `!empty($settings['value'])` — Null/""/"0"/empty-array all empty.
fn nonempty(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::String(s) => !s.is_empty() && s != "0",
        Value::Number(n) => n.as_f64().map(|x| x != 0.0).unwrap_or(false),
        Value::Bool(b) => *b,
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// `json_encode()` with PHP's default flags: `/` → `\/`, `<` `>` `&` `'`
/// → \uXXXX, all non-ASCII → \uXXXX. serde_json emits compact JSON; the
/// escapes below are identical to PHP's output for attribute embedding.
fn php_json(v: &Value) -> String {
    let s = serde_json::to_string(v).unwrap_or_default();
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '/' => out.push_str("\\/"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\'' => out.push_str("\\u0027"),
            c if !c.is_ascii() => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{:04x}", u));
                }
            }
            c => out.push(c),
        }
    }
    out
}

fn pairs_json(pairs: &[(&'static str, &'static str)]) -> Value {
    let mut o = Map::new();
    for (k, v) in pairs {
        o.insert((*k).to_string(), Value::String((*v).to_string()));
    }
    Value::Object(o)
}

/// Serialize one Field to the option-definition shape the JS templates read
/// (html-option.js lines 1058–1154): `type`, `title`, `options` (object),
/// `settings`, `wrapper_class`, `title_color`, `condition`, `description`,
/// `default`, `data-input-type`. PHP emits absent keys absent.
fn field_def_json(f: &Field) -> Value {
    let mut o = Map::new();
    if !f.title.is_empty() {
        o.insert("title".into(), Value::String(f.title.into()));
    }
    o.insert("type".into(), Value::String(f.ftype.into()));
    if f.ftype == "tabs" {
        // nested tabs: per-item field defs keyed by slug (js 1126–1131)
        o.insert("options".into(), children_json(f.children));
        if !f.settings.is_empty() {
            o.insert("settings".into(), pairs_json(f.settings));
        }
    } else if !f.options.is_empty() {
        o.insert("options".into(), pairs_json(f.options));
    }
    if !f.wrapper_class.is_empty() {
        o.insert("wrapper_class".into(), Value::String(f.wrapper_class.into()));
    }
    if !f.title_color.is_empty() {
        o.insert("title_color".into(), Value::String(f.title_color.into()));
    }
    if let Some(c) = f.condition {
        if let Ok(v) = serde_json::from_str::<Value>(c) {
            o.insert("condition".into(), v);
        }
    }
    if !f.description.is_empty() {
        o.insert("description".into(), Value::String(f.description.into()));
    }
    if !f.default.is_empty() {
        o.insert("default".into(), Value::String(f.default.into()));
    }
    if !f.data_input_type.is_empty() {
        o.insert(
            "data-input-type".into(),
            Value::String(f.data_input_type.into()),
        );
    }
    Value::Object(o)
}

/// PHP `options` array of a `custom` field → JSON object keyed by slug.
fn children_json(children: &[Field]) -> Value {
    let mut o = Map::new();
    for f in children {
        o.insert(f.slug.to_string(), field_def_json(f));
    }
    Value::Object(o)
}

/// `data-input-type` suffix shared by text/time/custom (PHP lines 198,239,245,593).
fn dit(f: &Field) -> String {
    if f.data_input_type.is_empty() {
        String::new()
    } else {
        format!(" data-input-type=\"{}\"", esc(f.data_input_type))
    }
}

/// Dynamic option sources the PHP resolves at render time
/// (`combobox` lines 297–317). The registered sidebars come from the
/// metabox ctx (set_sidebars); `tour_service` posts stay out of scope and
/// thumbnail-size uses stock WP intermediate sizes.
fn combobox_options(f: &Field) -> Vec<(String, String)> {
    match f.options_source {
        Some("sidebar") => {
            let mut v = vec![("none".into(), "None".into())];
            v.extend(CTX.with(|c| c.borrow().sidebars.clone()));
            v
        }
        Some("sidebar-default") => {
            let mut v = vec![
                ("none".into(), "None".into()),
                ("default".into(), "Default".into()),
            ];
            v.extend(CTX.with(|c| c.borrow().sidebars.clone()));
            v
        }
        Some("thumbnail-size") => vec![
            ("full".into(), "full size".into()),
            ("thumbnail".into(), "thumbnail 150-150".into()),
            ("medium".into(), "medium 300-300".into()),
            ("medium_large".into(), "medium_large 768-0".into()),
            ("large".into(), "large 1024-1024".into()),
        ],
        // 'number' (data-min/data-max) and 'post_type' (tour_service posts)
        // have no Field props / no db handle here — emit an empty select.
        _ => f
            .options
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Per-type renderers — same order as html-option.php.
// PHP `with-name` is never set by the tour meta box → `name` attrs omitted.
// WP `checked()`/`selected()` emit leading-space single-quoted attributes.
// ---------------------------------------------------------------------------

/// lines 188–203
fn text(f: &Field, value: &Value) -> String {
    let v = if set(value) { val_str(value) } else { f.default.to_string() };
    format!(
        "<input type=\"text\" class=\"tourmaster-html-option-text\" data-type=\"text\" data-slug=\"{}\" value=\"{}\" {} />",
        esc(f.slug),
        esc(&v),
        dit(f)
    )
}

/// lines 206–220. `data-ajax-url` uses the CMS ajax endpoint the parent
/// registered for this metabox (html_option_val.text.ajaxurl in mod.rs).
fn button(f: &Field) -> String {
    let mut ret = String::from("<div class=\"tourmaster-html-option-button\" ");
    if !f.data_type.is_empty() {
        ret.push_str(&format!(" data-type=\"{}\" ", esc(f.data_type)));
        if f.data_type == "ajax" {
            let post_id = CTX.with(|c| c.borrow().post_id.clone());
            ret.push_str(" data-ajax-url=\"/admin/plugins/tourmaster/save-option\" ");
            ret.push_str(&format!(" data-post-id=\"{}\" ", esc(&post_id)));
        }
    }
    if !f.data_action.is_empty() {
        ret.push_str(&format!(" data-action=\"{}\" ", esc(f.data_action)));
    }
    if !f.data_fields.is_empty() {
        let arr = Value::Array(
            f.data_fields
                .iter()
                .map(|s| Value::String((*s).to_string()))
                .collect(),
        );
        ret.push_str(&format!(" data-fields=\"{}\" ", esc(&php_json(&arr))));
    }
    ret.push_str(" >");
    ret.push_str(f.button_title);
    ret.push_str("</div>");
    ret
}

/// lines 223–252 (note the `data-type="text"  value` double space is PHP's).
fn time(f: &Field, value: &Value) -> String {
    let v = if set(value) { val_str(value) } else { f.default.to_string() };
    let mut parts = v.splitn(2, ':');
    let hh = parts.next().unwrap_or("");
    let mm = parts.next().unwrap_or("");
    format!(
        "<input type=\"text\" class=\"tourmaster-html-option-time tourmaster-input-hh\" placeholder=\"HH\" value=\"{}\" {} />\
         <span class=\"tourmaster-html-option-time-sep\" >:</span>\
         <input type=\"text\" class=\"tourmaster-html-option-time tourmaster-input-mm\" placeholder=\"MM\" value=\"{}\" {} />\
         <input type=\"hidden\" data-slug=\"{}\" data-type=\"text\"  value=\"{}\"  />",
        esc(hh),
        dit(f),
        esc(mm),
        dit(f),
        esc(f.slug),
        esc(&v)
    )
}

/// lines 255–269
fn datepicker(f: &Field, value: &Value) -> String {
    let v = if set(value) { val_str(value) } else { f.default.to_string() };
    format!(
        "<input type=\"text\" class=\"tourmaster-html-option-text tourmaster-html-option-datepicker\" data-type=\"text\" data-slug=\"{}\" value=\"{}\"  />\
         <i class=\"tourmaster-html-option-datepicker-icon fa fa-calendar\" ></i>",
        esc(f.slug),
        esc(&v)
    )
}

/// lines 272–285
fn textarea(f: &Field, value: &Value) -> String {
    let v = if set(value) { val_str(value) } else { f.default.to_string() };
    format!(
        "<textarea class=\"tourmaster-html-option-textarea\" data-type=\"textarea\" data-slug=\"{}\"  >{}</textarea>",
        esc(f.slug),
        esc(&v)
    )
}

/// lines 288–332
fn combobox(f: &Field, value: &Value) -> String {
    let v = if set(value) { val_str(value) } else { f.default.to_string() };
    let mut ret = String::from("<div class=\"tourmaster-custom-combobox\" >");
    ret.push_str(&format!(
        "<select class=\"tourmaster-html-option-combobox\" data-type=\"combobox\" data-slug=\"{}\"  >",
        esc(f.slug)
    ));
    for (k, label) in combobox_options(f) {
        let sel = if v == k { "selected='selected'" } else { "" };
        ret.push_str(&format!(
            "<option value=\"{}\" {} >{}</option>",
            esc(&k),
            sel,
            label
        ));
    }
    ret.push_str("</select></div>");
    ret
}

/// lines 335–360
fn multi_combobox(f: &Field, value: &Value) -> String {
    let sel: Vec<String> = match value {
        Value::Array(a) => a.iter().map(val_str).collect(),
        _ => Vec::new(),
    };
    let mut ret = format!(
        "<select class=\"tourmaster-html-option-multi-combobox\" data-type=\"multi-combobox\" data-slug=\"{}\"  multiple >",
        esc(f.slug)
    );
    for (k, label) in combobox_options(f) {
        let s = if sel.iter().any(|x| *x == k) { "selected" } else { "" };
        ret.push_str(&format!(
            "<option value=\"{}\" {} >{}</option>",
            esc(&k),
            s,
            label
        ));
    }
    ret.push_str("</select>");
    ret
}

/// lines 363–385
fn checkbox(f: &Field, value: &Value) -> String {
    let v = if set(value) {
        val_str(value)
    } else if !f.default.is_empty() {
        f.default.to_string()
    } else {
        "enable".to_string()
    };
    let checked = if v == "enable" { "checked='checked'" } else { "" };
    format!(
        "<label><input type=\"checkbox\" class=\"tourmaster-html-option-checkbox\" data-type=\"checkbox\" data-slug=\"{}\" {}  />\
         <div class=\"tourmaster-html-option-checkbox-appearance tourmaster-noselect\">\
         <span class=\"tourmaster-checkbox-button tourmaster-on\">On</span>\
         <span class=\"tourmaster-checkbox-separator\"></span>\
         <span class=\"tourmaster-checkbox-button tourmaster-off\">Off</span>\
         </div></label>",
        esc(f.slug),
        checked
    )
}

/// lines 388–430. `max-width` has no Field prop (unused by the tour schema),
/// so labels emit the PHP `style=""`-less form.
fn radioimage(f: &Field, value: &Value) -> String {
    // PHP resolves `'options' => 'sidebar'` to sidebar thumbnail images.
    let opts: Vec<(String, String)> = if f.options_source == Some("sidebar") {
        vec![
            ("none".into(), "/admin/wp/tourmaster/framework/images/sidebar/none.jpg".into()),
            ("left".into(), "/admin/wp/tourmaster/framework/images/sidebar/left.jpg".into()),
            ("right".into(), "/admin/wp/tourmaster/framework/images/sidebar/right.jpg".into()),
            ("both".into(), "/admin/wp/tourmaster/framework/images/sidebar/both.jpg".into()),
        ]
    } else {
        f.options
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    };

    let v = if nonempty(value) {
        val_str(value)
    } else if !f.default.is_empty() {
        f.default.to_string()
    } else {
        opts.first().map(|(k, _)| k.clone()).unwrap_or_default()
    };

    let mut ret = String::new();
    for (k, url) in opts {
        let checked = if v == k { " checked='checked'" } else { "" };
        ret.push_str(&format!(
            "<label  ><input class=\"tourmaster-html-option-radioimage\" type=\"radio\" name=\"{}\" data-type=\"radioimage\" data-slug=\"{}\" value=\"{}\" {}/>\
             <div class=\"tourmaster-radioimage-checked\" ></div>\
             <img src=\"{}\" alt=\"{}\" /></label>",
            esc(f.slug),
            esc(f.slug),
            esc(&k),
            checked,
            esc(&url),
            esc(&k)
        ));
    }
    ret
}

/// lines 433–455. `wp_get_attachment_url` is not ported — the stored value is
/// emitted verbatim as the background-image URL (importer keeps URLs).
fn upload(f: &Field, value: &Value) -> String {
    let v = if set(value) { val_str(value) } else { String::new() };
    let active = if v.is_empty() { "" } else { "tourmaster-active" };
    let bg = if v.is_empty() {
        String::new()
    } else {
        format!("background-image: url('{}');", esc(&v))
    };
    format!(
        "<div class=\"tourmaster-html-option-upload-appearance {}\" >\
         <input type=\"hidden\" class=\"tourmaster-html-option-upload\" data-type=\"upload\" data-slug=\"{}\" value=\"{}\"  />\
         <div class=\"tourmaster-upload-image-container\" style=\"{}\" ></div>\
         <div class=\"tourmaster-upload-image-overlay\" >\
         <div class=\"tourmaster-upload-image-button-hover\">\
         <span class=\"tourmaster-upload-image-button tourmaster-upload-image-add\"><i class=\"fa fa-plus\" ></i></span>\
         <span class=\"tourmaster-upload-image-button tourmaster-upload-image-remove\"><i class=\"fa fa-minus\" ></i></span>\
         </div></div></div>",
        active,
        esc(f.slug),
        esc(&v),
        bg
    )
}

/// lines 458–475
fn colorpicker(f: &Field, value: &Value) -> String {
    let default = f.default;
    let v = if set(value) { val_str(value) } else { default.to_string() };
    format!(
        "<input type=\"text\" class=\"tourmaster-html-option-colorpicker\" data-type=\"colorpicker\" data-slug=\"{}\" value=\"{}\" data-default-color=\"{}\"  />",
        esc(f.slug),
        esc(&v),
        esc(default)
    )
}

/// lines 478–502. `data-min`/`data-max`/`data-suffix`/`data-type` have no
/// Field props (unused by the tour schema) — emit the value-bearing form.
fn fontslider(f: &Field, value: &Value) -> String {
    let v = if nonempty(value) || matches!(value, Value::String(s) if s == "0") {
        val_str(value)
    } else if !f.default.is_empty() {
        f.default.to_string()
    } else {
        "0".to_string()
    };
    format!(
        "<input type=\"text\" class=\"tourmaster-html-option-fontslider\" data-type=\"text\" value=\"{}\" data-slug=\"{}\"  />",
        esc(&v),
        esc(f.slug)
    )
}

/// lines 584–608 — container + `data-value` JSON spans only; the served JS
/// (`rebind`, html-option.js:464–486) builds tabs/gallery/group-discount DOM.
fn custom(f: &Field, value: &Value) -> String {
    // PHP line 593: data-input-type here carries a trailing space too
    let dit = if f.data_input_type.is_empty() {
        String::new()
    } else {
        format!(" data-input-type=\"{}\" ", esc(f.data_input_type))
    };
    let mut ret = format!(
        "<div class=\"tourmaster-html-option-custom\" data-type=\"custom\" data-item-type=\"{}\" data-slug=\"{}\" {}>",
        esc(f.item_type.unwrap_or("")),
        esc(f.slug),
        dit
    );
    if !f.settings.is_empty() {
        ret.push_str(&format!(
            "<span class=\"tourmaster-html-option-custom-settings\" data-value=\"{}\" ></span>",
            esc(&php_json(&pairs_json(f.settings)))
        ));
    }
    // PHP `options` of a custom field = its sub-field definitions.
    if !f.children.is_empty() {
        ret.push_str(&format!(
            "<span class=\"tourmaster-html-option-custom-options\" data-value=\"{}\" ></span>",
            esc(&php_json(&children_json(f.children)))
        ));
    } else if !f.options.is_empty() {
        ret.push_str(&format!(
            "<span class=\"tourmaster-html-option-custom-options\" data-value=\"{}\" ></span>",
            esc(&php_json(&pairs_json(f.options)))
        ));
    }
    if nonempty(value) {
        ret.push_str(&format!(
            "<span class=\"tourmaster-html-option-custom-value\" data-value=\"{}\" ></span>",
            esc(&php_json(value))
        ));
    }
    ret.push_str("</div>");
    ret
}
