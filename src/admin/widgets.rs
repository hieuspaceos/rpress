//! Appearance → Widgets (`widgets.php` parity, accessibility-mode controls).
//!
//! Screen layout: `#widgets-left` holds "Available Widgets" (each type is
//! a `.widget` card whose `.widget-inside` carries title + sidebar pick →
//! `POST /admin/widgets/add`), "Inactive Widgets" and "Custom Sidebars";
//! `#widgets-right` holds one `.widgets-holder-wrap` per sidebar — a
//! `.sidebar-name` header (click collapses) over `.widgets-sortables`
//! widget cards. Each card's `.widget-inside` is the inline edit form
//! (fields from `widgets::widget_fields`) with a `.widget-control-actions`
//! footer: Remove, ↑/↓ moves posting to `/admin/widgets/order`, and a
//! sidebar-assignment select that doubles as move-to-sidebar / restore.
//! `wp_inactive_widgets` is a pseudo-sidebar: Remove parks a widget there
//! preserving settings; removing from it hard-deletes.
//!
//! Every mutation sets `settings.widgets_dirty=1`; the dashboard shows
//! "pending publish" until `generate::run` clears it (static-site analog
//! of WP's immediate publish).
//!
//! Stored `html` fragments are admin-trusted *jinja templates*
//! (`{{ site_url }}`/`{{ lang }}`): save dry-runs `render_fragment` and
//! refuses to persist markup that fails to parse/render.
use super::*;
use serde_json::{json, Value};

/// wp `wp_inactive_widgets` pseudo-sidebar: parked widgets keep their
/// settings; they never render because no template queries the id.
const INACTIVE: &str = "wp_inactive_widgets";

/// Every widget route is admin-only (wp `edit_theme_options` equivalent —
/// editors don't get this screen). Same convention as users.rs/settings.rs.
fn require_admin(s: &Sess) -> Result<(), StatusCode> {
    if s.role == "admin" {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn csrf_ok(s: &Sess, map: &HashMap<String, String>) -> bool {
    map.get("csrf").map(|x| x.as_str()) == Some(s.csrf.as_str())
}

/// `.widgets-holder-wrap` — wp widgets.php sidebar box chrome: a
/// `.sidebar-name` header bar (title left, `.handlediv` toggle arrow
/// right — click collapses the body) and a padded `.inside` body.
/// `title` is passed pre-escaped. `.closed` (toggled by inline JS in
/// WIDGETS_JS) hides `.inside` via common.min.css and flips the arrow.
fn holder_wrap(id: &str, title: &str, inside: String) -> String {
    format!(
        "<div id=\"{id}\" class=\"widgets-holder-wrap\">\
<div class=\"sidebar-name\" onclick=\"wgt_toggle(this)\"><h3>{title}\
<button type=\"button\" class=\"handlediv\" aria-expanded=\"true\">\
<span class=\"screen-reader-text\">Toggle panel</span>\
<span class=\"toggle-indicator\" aria-hidden=\"true\"></span></button></h3></div>\
<div class=\"inside\">{inside}</div></div>"
    )
}

fn mark_dirty(db: &std::path::Path) {
    let _ = crud::comments::setting_set(db, "widgets_dirty", "1");
}

/// (id, sidebar_id, type, title, settings_json) for one widget.
fn widget_row(
    conn: &Connection,
    id: &str,
) -> Option<(String, String, String, String, String)> {
    conn.query_row(
        "SELECT id, sidebar_id, type, title, settings_json FROM widgets WHERE id=?1",
        [id],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
            ))
        },
    )
    .ok()
}

fn sidebar_ids(conn: &Connection) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    if let Ok(mut st) = conn.prepare("SELECT id FROM sidebars ORDER BY rowid") {
        if let Ok(rows) = st.query_map([], |r| r.get::<_, String>(0)) {
            v.extend(rows.flatten());
        }
    }
    v.push(INACTIVE.to_string());
    v
}
/// `GET /admin/widgets` — two-column widget manager. `msg=` surfaces
/// action results; `#w-{id}` anchors land on the touched widget.
pub(crate) async fn widgets_page(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(code) = require_admin(&s) {
        return code.into_response();
    }
    let Ok(conn) = crate::db::conn(&sc.db) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    // Seeds on first admin visit too — generate may never have run here.
    let _ = crate::widgets::ensure(&conn);

    let (notice, kind) = match q.get("msg").map(|x| x.as_str()) {
        Some("added") => ("Widget added.", "success"),
        Some("saved") => ("Widget updated.", "success"),
        Some("removed") => ("Widget moved to Inactive Widgets.", "success"),
        Some("deleted") => ("Widget permanently deleted.", "success"),
        Some("moved") => ("Widget order updated.", "success"),
        Some("required") => ("A required field is missing — nothing was saved.", "error"),
        Some("badsyntax") => (
            "The HTML field contains a template error — nothing was saved. Check {{ }} and {% %} tags.",
            "error",
        ),
        Some("badjson") => ("The Children map must be valid JSON — nothing was saved.", "error"),
        Some("notfound") => ("Unknown widget.", "error"),
        Some("sbadded") => ("Sidebar registered.", "success"),
        Some("sbdeleted") => ("Sidebar deleted — its widgets moved to Inactive.", "success"),
        Some("sbbadid") => ("Sidebar id must be lowercase letters, digits, '-' or '_'.", "error"),
        Some("sbconst") => ("Theme-registered sidebars cannot be deleted.", "error"),
        Some("sbexists") => ("A sidebar with that id already exists.", "error"),
        Some("invalid") => ("Unknown widget type or sidebar.", "error"),
        _ => ("", ""),
    };

    // Sidebars in registry order + the inactive pseudo-sidebar last.
    let sidebars: Vec<(String, String)> = {
        let mut v = Vec::new();
        if let Ok(mut st) = conn.prepare("SELECT id, name FROM sidebars ORDER BY rowid") {
            if let Ok(rows) =
                st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            {
                v.extend(rows.flatten());
            }
        }
        v
    };
    // All widget instances grouped per sidebar, in display order.
    let mut by_sidebar: HashMap<String, Vec<(String, String, String, String, String)>> =
        HashMap::new();
    if let Ok(mut st) = conn.prepare(
        "SELECT id, sidebar_id, type, title, settings_json FROM widgets ORDER BY sidebar_id, ord",
    ) {
        if let Ok(rows) = st.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        }) {
            for w in rows.flatten() {
                by_sidebar.entry(w.1.clone()).or_default().push(w);
            }
        }
    }

    let taxs = crud::list_taxonomies(&sc.db).unwrap_or_default();
    // (id, label) pairs for sidebar selects — the widget form's select
    // shows human names, not raw ids.
    let all_sidebars: Vec<(String, String)> = {
        let mut v = sidebars.clone();
        v.push((INACTIVE.to_string(), "Inactive Widgets".to_string()));
        v
    };

    // --- Left column: Available Widgets ---------------------------------
    let mut sidebar_opts = String::new();
    for (sid, sname) in &sidebars {
        sidebar_opts.push_str(&format!(
            "<option value=\"{}\">{}</option>",
            esc(sid),
            esc(sname)
        ));
    }
    let mut available = String::new();
    for (typ, label) in crate::widgets::WIDGET_TYPES {
        available.push_str(&format!(
            "<div class=\"widget\" id=\"wt-{typ}\">\
<div class=\"widget-top\" onclick=\"wgt_toggle(this)\">\
<div class=\"widget-title\"><h3>{label}</h3>\
<button type=\"button\" class=\"widget-action\" aria-expanded=\"false\">\
<span class=\"toggle-indicator\" aria-hidden=\"true\"></span></button></div></div>\
<div class=\"widget-inside\">\
<form method=\"post\" action=\"/admin/widgets/add\">\
<input type=\"hidden\" name=\"csrf\" value=\"{csrf}\">\
<input type=\"hidden\" name=\"type\" value=\"{typ}\">\
<p><label>Title<br><input type=\"text\" name=\"title\" class=\"widefat\"></label></p>\
<p><label>Add to:<br><select name=\"sidebar\" class=\"widefat\">{sidebar_opts}</select></label></p>\
<p><input type=\"submit\" class=\"button button-primary\" value=\"Add Widget\"></p></form></div></div>",
            label = esc(label),
            csrf = s.csrf,
            typ = esc(typ),
            sidebar_opts = sidebar_opts,
        ));
    }
    let left_available = holder_wrap(
        "available-widgets",
        "Available Widgets",
        format!(
            "<p class=\"description\">Expand a widget type, choose a sidebar, and click Add Widget.</p>\
<div id=\"widget-list\" class=\"widgets-sortables\">{available}</div>"
        ),
    );

    // Inactive Widgets pseudo-sidebar (WP parity): lives at the bottom of
    // the LEFT column; deleting from it is permanent.
    let inactive = by_sidebar.remove(INACTIVE).unwrap_or_default();
    let mut inactive_blocks = String::new();
    for w in &inactive {
        inactive_blocks.push_str(&widget_block(w, &all_sidebars, &taxs, &s.csrf));
    }
    if inactive.is_empty() {
        inactive_blocks.push_str("<p class=\"wgt-empty\">No inactive widgets.</p>");
    }
    let left_inactive = holder_wrap(
        "wp_inactive_widgets",
        &format!("Inactive Widgets <span class=\"widgets-count\">({})</span>", inactive.len()),
        format!(
            "<p class=\"description\">Remove widgets from a sidebar to park them here — settings are kept.</p>\
<div class=\"widgets-sortables\">{inactive_blocks}</div>"
        ),
    );

    // Custom (admin-registered) sidebars: register a new region, or
    // delete a non-const one (its widgets park in Inactive).
    let mut custom_list = String::new();
    for (sid, sname) in &sidebars {
        if crate::widgets::is_const_sidebar(sid) {
            continue;
        }
        custom_list.push_str(&format!(
            "<li>{esc_name} <code>{esc_id}</code> \
<form method=\"post\" action=\"/admin/widgets/sidebar/delete\" style=\"display:inline\">\
<input type=\"hidden\" name=\"csrf\" value=\"{csrf}\"><input type=\"hidden\" name=\"id\" value=\"{esc_id}\">\
<button type=\"submit\" class=\"button-link submitdelete\">Delete</button></form></li>",
            esc_name = esc(sname),
            esc_id = esc(sid),
            csrf = s.csrf,
        ));
    }
    if custom_list.is_empty() {
        custom_list.push_str("<li class=\"wgt-empty\">No custom sidebars.</li>");
    }
    let left_custom = holder_wrap(
        "wgt-custom-sidebars",
        "Custom Sidebars",
        format!(
            "<ul class=\"wgt-sidebar-list\">{custom_list}</ul>\
<form method=\"post\" action=\"/admin/widgets/sidebar/add\">\
<input type=\"hidden\" name=\"csrf\" value=\"{csrf}\">\
<p><label>Id <span class=\"description\">(lowercase, e.g. sidebar-shop)</span><br>\
<input type=\"text\" name=\"id\" class=\"widefat\"></label></p>\
<p><label>Name<br><input type=\"text\" name=\"name\" class=\"widefat\"></label></p>\
<p><input type=\"submit\" class=\"button button-primary\" value=\"Register Sidebar\"></p></form>",
            csrf = s.csrf,
        ),
    );

    // --- Right column: one holder-wrap per sidebar ----------------------
    let mut sidebar_boxes = String::new();
    for (sid, sname) in &sidebars {
        let widgets = by_sidebar.remove(sid).unwrap_or_default();
        let mut blocks = String::new();
        for w in &widgets {
            blocks.push_str(&widget_block(w, &all_sidebars, &taxs, &s.csrf));
        }
        if widgets.is_empty() {
            blocks.push_str("<p class=\"wgt-empty\">Add widgets by expanding them on the left.</p>");
        }
        sidebar_boxes.push_str(&holder_wrap(
            &format!("sidebar-{}", sid.replace('_', "-")),
            &format!("{} <span class=\"widgets-count\">({})</span>", esc(sname), widgets.len()),
            format!("<div class=\"widgets-sortables\">{blocks}</div>"),
        ));
    }

    let mut body = chrome("menus", &s);
    body.push_str("<div class=\"wrap widgets-php\"><h1 class=\"wp-heading-inline\">Widgets</h1><hr class=\"wp-header-end\">");
    if !notice.is_empty() {
        body.push_str(&format!(
            "<div class=\"notice notice-{kind} is-dismissible\"><p>{notice}</p></div>"
        ));
    }
    body.push_str(&format!(
        "<div id=\"widgets-left\">{left_available}{left_inactive}{left_custom}</div>\
<div id=\"widgets-right\">{sidebar_boxes}</div>\
<div class=\"clear\"></div></div>"
    ));
    body.push_str(WIDGETS_STYLE);
    body.push_str("<script src=\"/admin/wp/tinymce/tinymce.min.js\"></script>");
    body.push_str(WIDGETS_JS);
    body.push_str(&admin::media_frame_html(&s.csrf));
    html_admin(&s, "Widgets", &body, false, "widgets-php").into_response()
}

/// One assigned widget as a wp `.widget` card: `.widget-top` header bar
/// (title + muted `: Type` suffix left, `.toggle-indicator` arrow right —
/// click toggles `.open`), `.widget-inside` form area (schema fields +
/// sidebar select), and a `.widget-control-actions` footer — Remove and
/// ↑/↓ moves left, primary Save right.
fn widget_block(
    w: &(String, String, String, String, String),
    all_sidebars: &[(String, String)],
    taxs: &[String],
    csrf: &str,
) -> String {
    let (id, sidebar, typ, title, raw) = w;
    let settings: Value = serde_json::from_str(raw).unwrap_or_else(|_| json!({}));
    let label = crate::widgets::WIDGET_TYPES
        .iter()
        .find(|(t, _)| t == typ)
        .map(|(_, l)| *l)
        .unwrap_or(typ.as_str());
    // Header shows `title: Type`; with no title just the type, and when
    // the title already equals the type no redundant suffix ("Search",
    // not "Search: Search").
    let shown = if title.trim().is_empty() {
        label
    } else {
        title.as_str()
    };
    let type_suffix = if title.trim().is_empty() || title.trim().eq_ignore_ascii_case(label) {
        String::new()
    } else {
        format!("<span class=\"in-widget-title\">: {}</span>", esc(label))
    };
    let inactive = sidebar == INACTIVE;

    // Sidebar-assignment select — doubles as move-to / restore-from-inactive.
    let mut opts = String::new();
    for (sid, sname) in all_sidebars {
        let sel = if sid == sidebar { " selected" } else { "" };
        opts.push_str(&format!(
            "<option value=\"{}\"{sel}>{}</option>",
            esc(sid),
            esc(sname)
        ));
    }

    let mut fields = String::new();
    for f in crate::widgets::widget_fields(typ) {
        fields.push_str(&field_html(f, &settings, id, taxs));
    }

    let remove_label = if inactive { "Delete permanently" } else { "Remove" };
    format!(
        "<div class=\"widget\" id=\"w-{id}\">\
<div class=\"widget-top\" onclick=\"wgt_toggle(this)\">\
<div class=\"widget-title\"><h3>{shown}{type_suffix}</h3>\
<button type=\"button\" class=\"widget-action\" aria-expanded=\"false\">\
<span class=\"toggle-indicator\" aria-hidden=\"true\"></span></button></div></div>\
<div class=\"widget-inside\">\
<form method=\"post\" action=\"/admin/widgets/save\" id=\"wf-{id}\">\
<input type=\"hidden\" name=\"csrf\" value=\"{csrf}\">\
<input type=\"hidden\" name=\"id\" value=\"{id}\">\
<p><label>Title<br><input type=\"text\" name=\"title\" value=\"{title_e}\" class=\"widefat\"></label></p>\
{fields}\
<p><label>Sidebar<br><select name=\"sidebar\" class=\"widefat\">{opts}</select></label></p>\
<div class=\"widget-control-actions wp-clearfix\">\
<div class=\"alignleft\">\
<button type=\"submit\" class=\"button-link submitdelete\" formaction=\"/admin/widgets/delete\">{remove_label}</button>\
<span class=\"wgt-sep\">|</span>\
<button type=\"submit\" class=\"button button-small\" formaction=\"/admin/widgets/order\" name=\"dir\" value=\"up\">&#8593; Up</button> \
<button type=\"submit\" class=\"button button-small\" formaction=\"/admin/widgets/order\" name=\"dir\" value=\"down\">&#8595; Down</button>\
</div>\
<div class=\"alignright\">\
<button type=\"submit\" class=\"button button-primary button-small\">Save</button>\
</div></div></form></div></div>",
        id = esc(id),
        shown = esc(shown),
        type_suffix = type_suffix,
        csrf = csrf,
        title_e = esc(title),
        fields = fields,
        opts = opts,
        remove_label = remove_label,
    )
}

/// One schema-driven field row (`f_{key}` posts to the save handler).
fn field_html(f: &crate::widgets::FieldSpec, settings: &Value, wid: &str, taxs: &[String]) -> String {
    let key = f.key;
    let name = format!("f_{key}");
    let req = if f.required { " <span class=\"required\">*</span>" } else { "" };
    let help = if f.help.is_empty() {
        String::new()
    } else {
        format!("<br><span class=\"description\">{}</span>", esc(f.help))
    };
    let val_str = |k: &str| -> String {
        settings
            .get(k)
            .map(|v| match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .unwrap_or_default()
    };
    let control = match f.kind {
        // WP Text-widget editor chrome: Visual/Code tabs, quicktags toolbar,
        // Add Media button. The textarea always carries RAW jinja — tokens
        // are applied only to TinyMCE's copy (see WIDGETS_JS). Initialized
        // lazily by `wgt_ed_init` when the widget card opens.
        "html" => {
            let eid = format!("wgted-{wid}-{key}");
            format!(
                "<div id=\"wp-{eid}-wrap\" class=\"wp-core-ui wp-editor-wrap html-active\">\
<div class=\"wp-editor-tools hide-if-no-js\">\
<div class=\"wp-media-buttons\">\
<button type=\"button\" class=\"button insert-media add_media\" onclick=\"wgt_ed_media('{eid_e}');return false\">\
<span class=\"wp-media-buttons-icon\"></span> Add Media</button></div>\
<div class=\"wp-editor-tabs\">\
<button type=\"button\" id=\"{eid_e}-tmce\" class=\"wp-switch-editor switch-tmce\" onclick=\"wgt_ed_switch('{eid_e}','visual');return false\">Visual</button>\
<button type=\"button\" id=\"{eid_e}-html\" class=\"wp-switch-editor switch-html\" onclick=\"wgt_ed_switch('{eid_e}','code');return false\">Code</button>\
</div></div>\
<div class=\"wp-editor-container\">\
<div id=\"qt_{eid_e}_toolbar\" class=\"quicktags-toolbar\"></div>\
<textarea class=\"wp-editor-area wgt-jinja-area\" id=\"{eid_e}\" name=\"{name}\" rows=\"8\">{val}</textarea>\
</div></div>",
                eid = esc(&eid),
                eid_e = esc(&eid),
                name = name,
                val = esc(&val_str(key))
            )
        }
        "textarea" => format!(
            "<textarea name=\"{name}\" rows=\"8\" class=\"widefat code\">{}</textarea>",
            esc(&val_str(key))
        ),
        "json" => format!(
            "<textarea name=\"{name}\" rows=\"3\" class=\"widefat code\">{}</textarea>",
            esc(&val_str(key))
        ),
        "list" => {
            let joined = settings
                .get(key)
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| val_str(key));
            format!(
                "<input type=\"text\" name=\"{name}\" value=\"{}\" class=\"widefat\">",
                esc(&joined)
            )
        }
        "checkbox" => {
            let on = match settings.get(key) {
                Some(Value::Bool(b)) => *b,
                Some(Value::String(s)) => s == "1" || s == "on" || s == "true",
                Some(Value::Number(n)) => n.as_i64() == Some(1),
                _ => false,
            };
            format!(
                "<label><input type=\"checkbox\" name=\"{name}\" value=\"1\"{}> {}</label>",
                if on { " checked" } else { "" },
                esc(f.label)
            )
        }
        "number" => format!(
            "<input type=\"number\" name=\"{name}\" value=\"{}\" class=\"small-text\">",
            esc(&val_str(key))
        ),
        "select" | "taxonomy" => {
            let cur = val_str(key);
            let dyn_opts: Vec<(String, String)> = if f.kind == "taxonomy" {
                taxs.iter().map(|t| (t.clone(), crate::admin::tax_label(t).0)).collect()
            } else {
                f.options
                    .iter()
                    .map(|(v, l)| (v.to_string(), l.to_string()))
                    .collect()
            };
            let mut o = String::from("<option value=\"\">— Select —</option>");
            for (v, l) in dyn_opts {
                let sel = if v == cur { " selected" } else { "" };
                o.push_str(&format!("<option value=\"{}\"{sel}>{}</option>", esc(&v), esc(&l)));
            }
            format!("<select name=\"{name}\" class=\"widefat\">{o}</select>")
        }
        "media" => {
            let img = val_str(&format!("{key}_url"));
            let style = if img.is_empty() { "display:none" } else { "" };
            let wid_e = esc(wid);
            let key_e = esc(key);
            format!(
                "<input type=\"hidden\" name=\"{name}\" id=\"wgt-{wid_e}-{key_e}\" value=\"{}\">\
<input type=\"hidden\" name=\"{name}_url\" id=\"wgt-{wid_e}-{key_e}-url\" value=\"{}\">\
<img id=\"wgt-{wid_e}-{key_e}-img\" src=\"{}\" style=\"{style};max-width:100%;height:auto;display:block;margin-bottom:6px\">\
<button type=\"button\" class=\"button\" onclick=\"wgt_media('{wid_e}','{key_e}')\">Select image</button>",
                esc(&val_str(key)),
                esc(&img),
                esc(&img),
            )
        }
        _ => format!(
            "<input type=\"text\" name=\"{name}\" value=\"{}\" class=\"widefat\">",
            esc(&val_str(key))
        ),
    };
    // The html editor contains buttons + divs, so it can't nest inside the
    // <label> — use `for=` on a sibling label instead.
    if f.kind == "html" {
        let for_id = esc(&format!("wgted-{wid}-{key}"));
        return format!(
            "<p><label for=\"{for_id}\">{}{req}</label>{control}{help}</p>",
            esc(f.label)
        );
    }
    format!("<p><label>{}{req}<br>{control}</label>{help}</p>", esc(f.label))
}

/// Page-scoped CSS: `#widgets-left`/`#widgets-right` frame plus the
/// widgets.php chrome that widgets.min.css would carry (not vendored) —
/// `.sidebar-name` header bars, `.widget-top` title bars, open-state
/// toggles and control-action footers. Border/shadow/arrow glyphs come
/// from common.min.css rules already keyed on these class names.
const WIDGETS_STYLE: &str = "<style>\
.wrap.widgets-php{max-width:1400px}\
#widgets-left{float:left;width:30%;max-width:440px;margin-right:20px}\
#widgets-right{float:left;width:65%}\
.widgets-holder-wrap{margin:0 0 20px}\
.sidebar-name{position:relative;padding:8px 15px;border-bottom:1px solid #c3c4c7}\
.sidebar-name h3{display:inline-block;margin:0;font-size:15px;font-weight:600;line-height:1.4}\
.sidebar-name .widgets-count{color:#787c82;font-weight:400}\
.sidebar-name .handlediv{position:absolute;right:0;top:0;width:44px;height:100%;padding:0;border:0;background:0 0;cursor:pointer;color:#787c82}\
.widgets-holder-wrap>.inside{padding:10px;margin:0}\
.widgets-sortables .widget{margin:0 0 10px}\
.widgets-sortables .widget:last-child{margin-bottom:0}\
.widget-top{position:relative;padding:8px 10px;cursor:pointer}\
.js .widget .widget-top{cursor:pointer}\
.widget-title h3{display:inline;margin:0;font-size:15px;font-weight:400;line-height:1.4}\
.widget-title .in-widget-title{color:#646970;font-weight:400}\
.widget-title .widget-action{position:absolute;right:0;top:0;width:36px;height:100%;padding:0;border:0;background:0 0;cursor:pointer}\
.js .widget-inside{display:none}\
.widget-inside{border-top:none;padding:0 10px 10px}\
.widget.open .widget-inside{display:block}\
.widget-inside p{margin:0 0 1em}\
.widget-control-actions{padding:10px 0 0;overflow:hidden}\
.widget-control-actions .alignleft>*{vertical-align:middle}\
.widget-control-actions .wgt-sep{color:#dcdcde;margin:0 2px}\
.widgets-php .submitdelete{color:#a00}\
.widgets-php .submitdelete:hover{color:#dc3232;border:none;background:0 0;box-shadow:none}\
.wgt-empty{color:#787c82;font-style:italic;margin:0}\
.wgt-sidebar-list{margin:0 0 10px}\
.wgt-sidebar-list li{padding:2px 0}\
.required{color:#b32d2e}\
.widget-inside .wp-editor-tools{overflow:hidden}\
.widget-inside .wp-media-buttons{float:left;padding:6px 0 0}\
.widget-inside .wp-editor-container{border:1px solid #dcdcde;clear:both}\
.widget-inside .wp-editor-area{height:250px;font-family:Consolas,Monaco,monospace;font-size:13px}\
.widget-inside .mce-tinymce{border:0;box-shadow:none}\
.widget-inside .mce-edit-area{border-top:1px solid #e5e5e5}\
#mf-frame{display:none}\
</style>";

/// Page scripts: media-frame glue, widget-card toggles, and the lazy
/// TinyMCE setup for `html` fields (WP Text-widget parity).
///
/// JINJA SAFETY: stored `html` is a jinja template (`{{ site_url }}`,
/// `{{ lang }}`, any `{% %}`/`{# #}`). TinyMCE would mangle `{{`/ braces
/// in attribute contexts, so every jinja block is replaced with a
/// `%%TOKEN%%` placeholder before it reaches the editor iframe — the
/// `BeforeSetContent` filter calls `wgt_protect`, the `GetContent` filter
/// calls `wgt_restore`, so `ed.getContent()`/`ed.save()`/`ed.hide()`
/// always yield raw jinja. The textarea ALWAYS holds raw jinja — Code
/// tab and form POST read it directly; a submit-time sweep restores any
/// tokens TinyMCE's own save hook might have left behind.
const WIDGETS_JS: &str = "<script>\
var wgt_mf=null;\
var wgt_tok={};\
function wgt_media(wid,key){wgt_mf='wgt-'+wid+'-'+key;mf_open('widget');}\
function wgt_ed_media(eid){wgt_mf='ed:'+eid;mf_open('widget');\
document.getElementById('mf-set').textContent='Insert into widget';}\
window.mf_select=function(id,thumb){if(!wgt_mf)return;\
if(wgt_mf.indexOf('ed:')===0){var eid=wgt_mf.slice(3);wgt_mf=null;\
fetch('/admin/media/json/'+id).then(function(r){return r.json();}).then(function(d){\
var img='<img src=\"'+(d.url||thumb)+'\" alt=\"'+String(d.alt||'').replace(/\"/g,'&quot;')+'\" />';\
var ed=(typeof tinymce!=='undefined')?tinymce.get(eid):null;\
if(ed&&!ed.hidden){ed.execCommand('mceInsertContent',false,img);}\
else{var ta=document.getElementById(eid);if(ta)ta.value+=img;}});\
return;}\
document.getElementById(wgt_mf).value=id;\
document.getElementById(wgt_mf+'-url').value=thumb;\
var im=document.getElementById(wgt_mf+'-img');if(im){im.src=thumb;im.style.display='block';}};\
function wgt_toggle(el){\
var top=el.closest('.widget-top');\
if(top){var w=top.closest('.widget');var open=w.classList.toggle('open');\
var b=top.querySelector('.widget-action');if(b)b.setAttribute('aria-expanded',open?'true':'false');\
if(open){var tas=w.querySelectorAll('textarea.wgt-jinja-area');\
for(var i=0;i<tas.length;i++)wgt_ed_init(tas[i].id);}\
return;}\
var name=el.closest('.sidebar-name');\
if(name){var wrap=name.parentNode;var closed=wrap.classList.toggle('closed');\
var hb=name.querySelector('.handlediv');if(hb)hb.setAttribute('aria-expanded',closed?'false':'true');}}\
window.getUserSetting=window.getUserSetting||function(n,d){try{var v=localStorage.getItem('wp-'+n);return v===null?d:v;}catch(e){return d;}};\
window.setUserSetting=window.setUserSetting||function(n,v){try{localStorage.setItem('wp-'+n,v);}catch(e){}};\
window.deleteUserSetting=window.deleteUserSetting||function(n){try{localStorage.removeItem('wp-'+n);}catch(e){}};\
function wgt_tokFor(map,raw){for(var k in map){if(k.charAt(0)!=='_'&&map[k]===raw)return k;}\
map._n=(map._n||0)+1;return '%%WJ'+map._n+'%%';}\
function wgt_protect(raw,map){map._n=0;var s=String(raw||'');\
s=s.replace(/\\{\\{\\s*site_url\\s*\\}\\}/g,'%%SITE_URL%%');\
s=s.replace(/\\{\\{\\s*lang\\s*\\}\\}/g,'%%LANG%%');\
s=s.replace(/\\{\\{[^\\}]*\\}\\}|\\{%[^%]*%\\}|\\{#[^#]*#\\}/g,function(m){var t=wgt_tokFor(map,m);map[t]=m;return t;});\
return s;}\
function wgt_restore(s,map){s=String(s||'');\
for(var k in map){if(k.charAt(0)==='_')continue;s=s.split(k).join(map[k]);}\
s=s.split('%%SITE_URL%%').join('{{ site_url }}');\
s=s.split('%%LANG%%').join('{{ lang }}');return s;}\
function wgt_ed_init(id){\
if(typeof tinymce==='undefined'||tinymce.get(id)||window['_wgti_'+id])return;\
window['_wgti_'+id]=1;var ta=document.getElementById(id);if(!ta)return;\
wgt_tok[id]=wgt_tok[id]||{};\
tinymce.init({\
selector:'#'+id,\
skin_url:'/admin/wp/tinymce/skins/lightgray',\
theme:'modern',\
theme_url:'/admin/wp/tinymce/themes/modern/theme.min.js',\
plugins:'charmap colorpicker compat3x directionality fullscreen hr image link lists media paste tabfocus textcolor wpautoresize wpdialogs wpeditimage wpemoji wpgallery wptextpattern wpview wordpress',\
menubar:false,\
branding:false,\
height:250,\
toolbar1:'formatselect bold italic strikethrough bullist numlist blockquote alignleft aligncenter alignright link unlink fullscreen wp_adv',\
toolbar2:'forecolor pastetext removeformat charmap outdent indent undo redo wp_help',\
toolbar3:'',\
wp_autoresize_on:true,\
autoresize_min_height:250,\
relative_urls:false,\
convert_urls:false,\
entity_encoding:'raw',\
content_css:'/admin/wp/editor.min.css,/admin/wp/cms-overrides.css',\
setup:function(ed){\
ed.on('BeforeSetContent',function(e){\
if(typeof e.content==='string')e.content=wgt_protect(e.content,wgt_tok[id]);});\
ed.on('GetContent',function(e){\
if(typeof e.content==='string')e.content=wgt_restore(e.content,wgt_tok[id]);});\
ed.on('init',function(){var t=document.getElementById(id);\
if(t)ed.setContent(t.value);});}});\
wgt_ed_mode(id,'tmce');}\
function wgt_ed_mode(id,m){var wrap=document.getElementById('wp-'+id+'-wrap');if(!wrap)return;\
wrap.classList.toggle('tmce-active',m==='tmce');wrap.classList.toggle('html-active',m==='html');\
var tb=document.getElementById(id+'-tmce'),hb=document.getElementById(id+'-html');\
if(tb)tb.setAttribute('aria-pressed',m==='tmce'?'true':'false');\
if(hb)hb.setAttribute('aria-pressed',m==='html'?'true':'false');}\
function wgt_ed_switch(id,mode){var ta=document.getElementById(id);if(!ta)return;\
var ed=(typeof tinymce!=='undefined')?tinymce.get(id):null;\
if(mode==='code'){\
if(ed&&!ed.hidden){ta.value=ed.getContent();ed.hide();}\
wgt_ed_mode(id,'html');}\
else{wgt_ed_init(id);ed=(typeof tinymce!=='undefined')?tinymce.get(id):null;\
if(ed){ed.setContent(ta.value);ed.show();}\
wgt_ed_mode(id,'tmce');}}\
document.addEventListener('submit',function(e){var f=e.target;\
if(!f||!f.querySelectorAll)return;\
var tas=f.querySelectorAll('textarea.wgt-jinja-area');\
for(var i=0;i<tas.length;i++){var ta=tas[i];\
if((typeof tinymce!=='undefined')&&tinymce.get(ta.id)){\
ta.value=wgt_restore(ta.value,wgt_tok[ta.id]||{});}}});\
</script>";

/// `POST /admin/widgets/add` — `{type, sidebar, title}` → new instance
/// `{type}-{n}` appended to the sidebar. Types carrying `html` get an
/// empty wrapper skeleton so the stored shape matches seeded rows.
pub(crate) async fn widgets_add(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(code) = require_admin(&s) {
        return code.into_response();
    }
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(conn) = crate::db::conn(&sc.db) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let _ = crate::widgets::ensure(&conn);
    let typ = map.get("type").cloned().unwrap_or_default();
    let sidebar = map.get("sidebar").cloned().unwrap_or_default();
    let title = map.get("title").cloned().unwrap_or_default();
    let valid_type = crate::widgets::WIDGET_TYPES.iter().any(|(t, _)| *t == typ);
    if !valid_type || !sidebar_ids(&conn).contains(&sidebar) || sidebar == INACTIVE {
        return Redirect::to("/admin/widgets?msg=invalid").into_response();
    }
    let Ok(id) = crate::widgets::next_instance_id(&conn, &typ) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let settings = default_settings(&typ, &id);
    let ord: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(ord)+1, 0) FROM widgets WHERE sidebar_id=?1",
            [&sidebar],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let ok = conn.execute(
        "INSERT INTO widgets(id, sidebar_id, type, title, settings_json, ord)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, sidebar, typ, title, settings, ord],
    );
    if ok.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    mark_dirty(&sc.db);
    crate::admin::admin::bust_cache(&sc);
    Redirect::to(&format!("/admin/widgets?msg=added#w-{id}")).into_response()
}

/// Initial `settings_json` for a new instance. `html`-storing types get a
/// minimal wrapper matching the seeded shape (`<div id="{id}" class=
/// "widget widget_{type} traveltour-widget">`); `search` gets a working
/// WP search form so a fresh instance renders something real. Settings
/// types (categories) start empty — the edit form requires taxonomy.
fn default_settings(typ: &str, id: &str) -> String {
    let html = match typ {
        "search" => format!(
            "<div id=\"{id}\" class=\"widget widget_search traveltour-widget\">\
<form role=\"search\" method=\"get\" class=\"search-form\" action=\"{{{{ site_url }}}}/\">\
<label><span class=\"screen-reader-text\">Rechercher&#160;:</span>\
<input type=\"search\" class=\"search-field\" placeholder=\"Rechercher&#8230;\" value=\"\" name=\"s\" />\
</label><input type=\"submit\" class=\"search-submit\" value=\"Rechercher\" /></form></div>"
        ),
        "text" | "custom_html" | "html_embed" | "tour_category" | "media_image" | "tour_list" => {
            format!(
                "<div id=\"{id}\" class=\"widget widget_{typ} traveltour-widget\"></div>",
            )
        }
        _ => return "{}".to_string(),
    };
    json!({ "html": html }).to_string()
}

/// `POST /admin/widgets/save` — `{id, title, sidebar, f_*}`: merge schema
/// fields over existing settings (unknown keys preserved), validate
/// required + json fields, dry-run `html` through `render_fragment`,
/// apply title/sidebar move, flag dirty.
pub(crate) async fn widgets_save(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(code) = require_admin(&s) {
        return code.into_response();
    }
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(conn) = crate::db::conn(&sc.db) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let id = map.get("id").cloned().unwrap_or_default();
    let Some((_id, cur_sidebar, typ, _t, raw)) = widget_row(&conn, &id) else {
        return Redirect::to("/admin/widgets?msg=notfound").into_response();
    };

    let mut settings: serde_json::Map<String, Value> =
        serde_json::from_str(&raw).unwrap_or_else(|_| json!({}))
            .as_object()
            .cloned()
            .unwrap_or_default();

    // Merge posted fields over stored settings per the type's schema.
    for f in crate::widgets::widget_fields(&typ) {
        let posted = map.get(&format!("f_{}", f.key));
        match f.kind {
            "checkbox" => {
                settings.insert(f.key.to_string(), Value::Bool(posted.is_some()));
            }
            "html" | "textarea" | "text" | "number" | "select" | "taxonomy" => {
                let v = posted.cloned().unwrap_or_default();
                if v.is_empty() {
                    settings.remove(f.key);
                } else {
                    settings.insert(f.key.to_string(), json!(v));
                }
            }
            "list" => {
                let items: Vec<String> = posted
                    .cloned()
                    .unwrap_or_default()
                    .split(',')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect();
                if items.is_empty() {
                    settings.remove(f.key);
                } else {
                    settings.insert(f.key.to_string(), json!(items));
                }
            }
            "json" => {
                let v = posted.cloned().unwrap_or_default();
                if v.trim().is_empty() {
                    settings.remove(f.key);
                } else if let Ok(parsed) = serde_json::from_str::<Value>(&v) {
                    settings.insert(f.key.to_string(), parsed);
                } else {
                    return Redirect::to(&format!(
                        "/admin/widgets?msg=badjson#w-{}",
                        esc(&id)
                    ))
                    .into_response();
                }
            }
            "media" => {
                let mid = posted.cloned().unwrap_or_default();
                let url = map
                    .get(&format!("f_{}_url", f.key))
                    .cloned()
                    .unwrap_or_default();
                if mid.is_empty() {
                    settings.remove(f.key);
                    settings.remove(&format!("{}_url", f.key));
                } else {
                    settings.insert(f.key.to_string(), json!(mid));
                    settings.insert(format!("{}_url", f.key), json!(url));
                }
            }
            _ => {}
        }
    }

    // Required fields must hold a non-empty value after the merge — except
    // when a frozen `html` fragment carries the whole render (the seeded
    // wpcategorieswidget-9 capture has no taxonomy and must stay editable).
    let html_frozen = settings
        .get("html")
        .and_then(|h| h.as_str())
        .map(|h| !h.trim().is_empty())
        .unwrap_or(false);
    for f in crate::widgets::widget_fields(&typ) {
        if f.required && !html_frozen {
            let missing = settings
                .get(f.key)
                .and_then(|v| v.as_str())
                .map(|v| v.trim().is_empty())
                .unwrap_or(true);
            if missing {
                return Redirect::to(&format!(
                    "/admin/widgets?msg=required#w-{}",
                    esc(&id)
                ))
                .into_response();
            }
        }
    }

    // `html` is admin-trusted *template code*: dry-run the exact render
    // path before persisting so broken jinja never reaches generate.
    if let Some(html) = settings.get("html").and_then(|h| h.as_str()) {
        if crate::widgets::render_fragment(html, "", "fr").is_err() {
            return Redirect::to(&format!(
                "/admin/widgets?msg=badsyntax#w-{}",
                esc(&id)
            ))
            .into_response();
        }
    }
    // media_image convenience: picked image + empty html → synthesize the
    // wrapper the seeded rows carry (phase-1 render shape).
    if typ == "media_image" && settings.get("html").is_none() {
        if let Some(url) = settings
            .get("image_id_url")
            .and_then(|v| v.as_str())
            .filter(|u| !u.is_empty())
        {
            let t = map.get("title").cloned().unwrap_or_default();
            let title_html = if t.is_empty() {
                String::new()
            } else {
                format!(
                    "<h2 class=\"traveltour-widget-title\"><span class=\"traveltour-widget-head-text\">{}</span></h2><span class=\"clear\"></span>",
                    esc(&t)
                )
            };
            settings.insert(
                "html".into(),
                json!(format!(
                    "<div id=\"{id}\" class=\"widget widget_media_image traveltour-widget\">{title_html}<img src=\"{url}\" class=\"image attachment-medium size-medium\" style=\"max-width: 100%; height: auto;\" decoding=\"async\" loading=\"lazy\"></div>"
                )),
            );
        }
    }

    let title = map.get("title").cloned().unwrap_or_default();
    // categories widgets carry the heading inside settings_json (the
    // archive template reads `w.title`) — keep both stores in sync.
    if typ == "categories" {
        settings.insert("title".into(), json!(title));
    }

    // Sidebar reassignment (also the restore-from-inactive path): move =
    // append at the target's end.
    let mut target = map.get("sidebar").cloned().unwrap_or(cur_sidebar.clone());
    if !sidebar_ids(&conn).contains(&target) {
        target = cur_sidebar.clone();
    }
    let ord: i64 = if target == cur_sidebar {
        conn.query_row("SELECT ord FROM widgets WHERE id=?1", [&id], |r| {
            r.get::<_, i64>(0)
        })
        .unwrap_or(0)
    } else {
        conn.query_row(
            "SELECT COALESCE(MAX(ord)+1, 0) FROM widgets WHERE sidebar_id=?1",
            [&target],
            |r| r.get(0),
        )
        .unwrap_or(0)
    };
    let ok = conn.execute(
        "UPDATE widgets SET sidebar_id=?1, title=?2, settings_json=?3, ord=?4 WHERE id=?5",
        rusqlite::params![target, title, Value::Object(settings).to_string(), ord, id],
    );
    if ok.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    mark_dirty(&sc.db);
    crate::admin::admin::bust_cache(&sc);
    Redirect::to(&format!("/admin/widgets?msg=saved#w-{}", esc(&id))).into_response()
}

/// `POST /admin/widgets/delete` — from a real sidebar: park the widget in
/// `wp_inactive_widgets` (settings preserved, wp `remove` parity). From
/// the inactive area: hard delete.
pub(crate) async fn widgets_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(code) = require_admin(&s) {
        return code.into_response();
    }
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(conn) = crate::db::conn(&sc.db) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let id = map.get("id").cloned().unwrap_or_default();
    let Some((_id, sidebar, _t, _ti, _raw)) = widget_row(&conn, &id) else {
        return Redirect::to("/admin/widgets?msg=notfound").into_response();
    };
    if sidebar == INACTIVE {
        let _ = conn.execute("DELETE FROM widgets WHERE id=?1", [&id]);
        mark_dirty(&sc.db);
        return Redirect::to("/admin/widgets?msg=deleted").into_response();
    }
    let ord: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(ord)+1, 0) FROM widgets WHERE sidebar_id=?1",
            [INACTIVE],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let _ = conn.execute(
        "UPDATE widgets SET sidebar_id=?1, ord=?2 WHERE id=?3",
        rusqlite::params![INACTIVE, ord, id],
    );
    mark_dirty(&sc.db);
    crate::admin::admin::bust_cache(&sc);
    Redirect::to("/admin/widgets?msg=removed").into_response()
}

/// `POST /admin/widgets/order` — two modes:
///  * `{id, dir=up|down}` — swap with the adjacent widget (↑/↓ buttons;
///    posts the block's sidebar select, which always holds the widget's
///    current sidebar).
///  * `{sidebar, ids_csv}` — full re-sequence (spec'd JSON/script path).
pub(crate) async fn widgets_order(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(code) = require_admin(&s) {
        return code.into_response();
    }
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(conn) = crate::db::conn(&sc.db) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    // Scope of the reorder — for ids_csv mode the posted sidebar guards
    // against cross-sidebar ord writes (a stale post must not renumber a
    // different region); for swap mode it's the widget's own sidebar.
    let scope_sidebar = map.get("sidebar").cloned().unwrap_or_default();
    let (ordered, apply_sidebar): (Vec<String>, Option<String>) = if let Some(csv) = map.get("ids_csv") {
        (
            csv.split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect(),
            Some(scope_sidebar),
        )
    } else {
        // Single-step move: reorder the sidebar's id list by swapping
        // `id` with its up/down neighbour — read + write inside one tx so
        // a concurrent order post can't interleave (TOCTOU).
        let id = map.get("id").cloned().unwrap_or_default();
        let dir = map.get("dir").map(|x| x.as_str()).unwrap_or("");
        if id.is_empty() || !matches!(dir, "up" | "down") {
            return Redirect::to("/admin/widgets").into_response();
        }
        let Ok(tx) = conn.unchecked_transaction() else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let mut ids: Vec<String> = Vec::new();
        if let Ok(mut st) =
            tx.prepare("SELECT id FROM widgets WHERE sidebar_id=?1 ORDER BY ord")
        {
            if let Ok(rows) = st.query_map([&scope_sidebar], |r| r.get::<_, String>(0)) {
                ids.extend(rows.flatten());
            }
        }
        let Some(pos) = ids.iter().position(|x| x == &id) else {
            return Redirect::to("/admin/widgets?msg=notfound").into_response();
        };
        let swap = match dir {
            "up" => pos.checked_sub(1),
            _ => (pos + 1 < ids.len()).then_some(pos + 1),
        };
        let Some(j) = swap else {
            return Redirect::to(&format!("/admin/widgets#w-{}", esc(&id))).into_response();
        };
        ids.swap(pos, j);
        for (i, wid) in ids.iter().enumerate() {
            let _ = tx.execute(
                "UPDATE widgets SET ord=?1 WHERE id=?2 AND sidebar_id=?3",
                rusqlite::params![i as i64, wid, scope_sidebar],
            );
        }
        let _ = tx.commit();
        mark_dirty(&sc.db);
        return Redirect::to(&format!("/admin/widgets?msg=moved#w-{}", esc(&id)))
            .into_response();
    };

    if ordered.is_empty() {
        return Redirect::to("/admin/widgets").into_response();
    }
    // Rewrite ord for every listed id, in list order (one tx), scoped to
    // the posted sidebar.
    if let Ok(tx) = conn.unchecked_transaction() {
        for (i, wid) in ordered.iter().enumerate() {
            if let Some(sb) = &apply_sidebar {
                let _ = tx.execute(
                    "UPDATE widgets SET ord=?1 WHERE id=?2 AND sidebar_id=?3",
                    rusqlite::params![i as i64, wid, sb],
                );
            } else {
                let _ = tx.execute("UPDATE widgets SET ord=?1 WHERE id=?2", rusqlite::params![i as i64, wid]);
            }
        }
        let _ = tx.commit();
    }
    mark_dirty(&sc.db);
    let anchor = map
        .get("id")
        .map(|id| format!("#w-{}", esc(id)))
        .unwrap_or_default();
    crate::admin::admin::bust_cache(&sc);
    Redirect::to(&format!("/admin/widgets?msg=moved{anchor}")).into_response()
}

/// `POST /admin/widgets/sidebar/add` — register a custom GoodLayers-style
/// sidebar: a `sidebars` row joins the const registry (const ∪ stored).
pub(crate) async fn sidebar_add(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(code) = require_admin(&s) {
        return code.into_response();
    }
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(conn) = crate::db::conn(&sc.db) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let _ = crate::widgets::ensure(&conn);
    let id = map.get("id").cloned().unwrap_or_default();
    let name = {
        let n = map.get("name").cloned().unwrap_or_default();
        if n.trim().is_empty() { id.clone() } else { n }
    };
    let valid = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !valid || id == INACTIVE {
        return Redirect::to("/admin/widgets?msg=sbbadid").into_response();
    }
    if sidebar_ids(&conn).contains(&id) || crate::widgets::is_const_sidebar(&id) {
        return Redirect::to("/admin/widgets?msg=sbexists").into_response();
    }
    // lang 'all': custom sidebars aren't lang-partitioned; pages pick by id.
    if conn
        .execute(
            "INSERT INTO sidebars(id, name, lang) VALUES(?1, ?2, 'all')",
            rusqlite::params![id, name],
        )
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    mark_dirty(&sc.db);
    crate::admin::admin::bust_cache(&sc);
    Redirect::to("/admin/widgets?msg=sbadded").into_response()
}

/// `POST /admin/widgets/sidebar/delete` — unregister a custom sidebar.
/// Const (theme-registered) regions refuse; the deleted sidebar's widgets
/// park in Inactive (settings preserved — wp remove parity).
pub(crate) async fn sidebar_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(code) = require_admin(&s) {
        return code.into_response();
    }
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(conn) = crate::db::conn(&sc.db) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let id = map.get("id").cloned().unwrap_or_default();
    if crate::widgets::is_const_sidebar(&id) {
        return Redirect::to("/admin/widgets?msg=sbconst").into_response();
    }
    if let Ok(tx) = conn.unchecked_transaction() {
        let ord: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(ord)+1, 0) FROM widgets WHERE sidebar_id=?1",
                [INACTIVE],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let _ = tx.execute(
            "UPDATE widgets SET sidebar_id=?1, ord=ord+?2 WHERE sidebar_id=?3",
            rusqlite::params![INACTIVE, ord, id],
        );
        let _ = tx.execute("DELETE FROM sidebars WHERE id=?1", [&id]);
        let _ = tx.commit();
    }
    mark_dirty(&sc.db);
    crate::admin::admin::bust_cache(&sc);
    Redirect::to("/admin/widgets?msg=sbdeleted").into_response()
}
