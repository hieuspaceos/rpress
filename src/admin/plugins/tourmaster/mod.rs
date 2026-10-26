//! Tourmaster editor plugin for `kind == "tour"`.
//! Asset/CSS/JS and markup are ports of `wp-content/plugins/tourmaster`
//! (copied to `vps-ssh/reference/tourmaster`); do not restyle.
use crate::admin::esc;
#[allow(unused_imports)]
use crate::sql;

pub(crate) mod elements;
pub(crate) mod save;
pub(crate) mod schema;

use super::{EditorPlugin, PostForm, Sess, SiteCtx};
use crate::php_serialize::unserialize;
use anyhow::Result;
use rusqlite::{params, Connection};
use serde_json::{Map, Value};

pub(crate) static PLUGIN: EditorPlugin = EditorPlugin {
    kind: "tour",
    load,
    metabox,
    assets,
    save: save::save,
};

/// Hydrate `f.extra["tourmaster-tour-option"]` with the stored dict merged
/// with the single-value metas (so `get_val()` field hydration sees every
/// registered slug in one place).
fn load(c: &Connection, id: i64, f: &mut PostForm) -> Result<()> {
    let mut dict: Map<String, Value> = c
        .query_row(
            sql!("posts__select_meta_value_from"),
            params![id, "tourmaster-tour-option"],
            |r| r.get::<_, String>(0),
        )
        .ok()
        .and_then(|raw| unserialize(raw.as_bytes()).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    for (slug, meta) in save::SINGLE_METAS {
        if let Ok(v) = c.query_row(
            sql!("posts__select_meta_value_from"),
            params![id, meta],
            |r| r.get::<_, String>(0),
        ) {
            dict.insert(slug.to_string(), Value::String(v));
        }
    }
    f.extra.insert(
        "tourmaster-tour-option".to_string(),
        Value::Object(dict).to_string(),
    );
    Ok(())
}

/// Metabox chrome — verbatim port of
/// `framework/function/page-option.php` lines 57–126.
fn metabox(sc: &SiteCtx, s: &Sess, f: &PostForm) -> String {
    let id = f.id.map(|n| n.to_string()).unwrap_or_default();
    // data-post-id for ajax buttons; no ical endpoint exists yet.
    elements::set_context(&id, "");
    // Registered sidebars for the combobox field types (sidebar /
    // sidebar-default) — the page builder's sidebar pickers enumerate the
    // widget registry (const ∪ custom rows).
    if let Ok(conn) = crate::db::conn(&sc.db) {
        if let Ok(list) = crate::widgets::list(&conn) {
            elements::set_sidebars(&list);
        }
    }
    let init = f
        .extra
        .get("tourmaster-tour-option")
        .cloned()
        .unwrap_or_else(|| "{}".to_string());
    let init_json: Value = serde_json::from_str(&init).unwrap_or(Value::Null);
    let values = init_json.as_object().cloned().unwrap_or_default();

    let mut head_tabs = String::new();
    let mut bodies = String::new();
    let mut active = true;
    for tab in schema::TABS {
        let cls = if active { " tourmaster-active" } else { "" };
        // PHP esc_html__ escapes &; titles containing <i> markup stay raw.
        let tab_title = if tab.title.contains('<') {
            tab.title.to_string()
        } else {
            esc(tab.title)
        };
        head_tabs.push_str(&format!(
            r#"<div class="tourmaster-page-option-tab-head-item{cls}" data-tab-slug="{}" >{}</div>"#,
            esc(tab.slug),
            tab_title
        ));
        bodies.push_str(&format!(
            r#"<div class="tourmaster-page-option-tab-content-item{cls}" data-tab-slug="{}" >{}</div>"#,
            esc(tab.slug),
            elements::render_tab(tab, &values)
        ));
        active = false;
    }

    format!(
        r#"<div class="postbox"><h2>Tour Settings</h2><div class="inside" style="padding:0">
<input type="hidden" name="plugin_page_option_security" value="{csrf}" />
<input type="hidden" class="tourmaster-page-option-value" name="tourmaster-tour-option" value="{init}" />
<div class="tourmaster-page-option-head" >
<div class="tourmaster-page-option-head-title"><i class="fa fa-plane"></i>Tour Settings</div>
<div class="tourmaster-page-option-head-save" data-post-id="{id}" data-ajax-url="/admin/api/tour/save-option" data-failed-head="An error occurs" data-failed-message="Please use wordpress update button to update the page instead." ><i class="fa fa-save"></i>Save Settings</div>
</div>
<div class="tourmaster-page-option-content" >
<div class="tourmaster-page-option-tab-head clearfix" id="tourmaster-page-option-tab-head" >{head_tabs}</div>
<div class="tourmaster-page-option-tab-content tourmaster-condition-wrapper" id="tourmaster-page-option-tab-content" >{bodies}</div>
</div></div></div>"#,
        csrf = esc(&s.csrf),
        init = esc(&init),
        id = esc(&id),
    )
}

fn assets() -> &'static str {
    r#"<link rel="stylesheet" href="/admin/wp/gdlr/plugins/fontawesome/font-awesome.css"/>
<link rel="stylesheet" href="/admin/wp/color-picker.min.css"/>
<link rel="stylesheet" href="/admin/wp/tourmaster/framework/css/utility.css"/>
<link rel="stylesheet" href="/admin/wp/tourmaster/framework/css/html-option-meta.css"/>
<script>
var html_option_val = {"text":{"ajaxurl":"/admin/api/tour/save-option","error_head":"An error occurs","error_message":"Please refresh the page to try again.","nonce":"","upload_media":"Select or Upload Media","choose_media":"Use this media"},"tabs":{"title_text":"Item's Title","tab_checkbox_on":"On","tab_checkbox_off":"Off"}};
var tourmaster_utility = {"confirm_head":"Just to confirm","confirm_text":"Are you sure to do this ?","confirm_sub":"* Please noted that this could not be undone.","confirm_yes":"Yes","confirm_no":"No"};
</script>
<script src="/admin/wp/jquery-ui.min.js"></script>
<script src="/admin/wp/iris.min.js"></script>
<script src="/admin/wp/color-picker.min.js"></script>
<script src="/admin/wp/gdlr/wp-media-shim.js"></script>
<script src="/admin/wp/tourmaster/framework/js/utility.js"></script>
<script src="/admin/wp/tourmaster/framework/js/html-option.js"></script>"#
}

