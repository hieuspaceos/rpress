//! Tourmaster editor plugin for `kind == "tour"`.
//! Asset/CSS/JS and markup are ports of `wp-content/plugins/tourmaster`
//! (copied to `vps-ssh/reference/tourmaster`); do not restyle.
use crate::admin::esc;
#[allow(unused_imports)]
use crate::sql;

pub(crate) mod elements;
pub(crate) mod save;
pub(crate) mod schema;
pub(crate) mod screens;

use super::{Backend, EditorHooks, KindDef, PluginAdapter, PostForm, ScreenDef, Sess, SiteCtx};
use crate::php_serialize::unserialize;
use anyhow::Result;
use rusqlite::{params, Connection};
use serde_json::{Map, Value};
use std::path::Path;

/// Editor half of the tourmaster adapter — the old `EditorPlugin` fields.
pub(crate) static EDITOR: EditorHooks = EditorHooks {
    load,
    metabox,
    assets,
    save: save::save,
};

const HEAD_TOUR: &str = r#"<tr><td class="manage-column column-cb check-column"></td><th scope="col" class="manage-column column-title column-primary">Title</th><th scope="col">Durée</th><th scope="col">Región</th><th scope="col">Style</th><th scope="col">Profil</th><th scope="col">Views</th><th scope="col" class="rm-seo-col">SEO Details</th></tr>"#;

fn tour_row_cols(r: &crate::admin::crud::AdminListRow, _kind: &str) -> String {
    format!(
        "<td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>",
        crate::admin::posts::dash(&r.duree),
        crate::admin::posts::dash(&r.region),
        crate::admin::posts::dash(&r.style),
        crate::admin::posts::dash(&r.profil),
        r.views,
    )
}

static KINDS: &[KindDef] = &[KindDef {
    kind: "tour",
    section: "tours",
    noun: "Tour",
    list_title: "Tour",
    add_label: "Add New Tour",
    head: HEAD_TOUR,
    row_cols: tour_row_cols,
    url_base_key: Some("tour_base"),
    taxonomies: &[
        "duree", "duracion", "region", "region-es", "style", "estilo",
        "profil", "perfil", "activites", "Activites", "pays", "Pays",
        "tour-activity", "tour-age", "viet",
    ],
    menu_icon: "dashicons-palmtree",
    editor: Some(&EDITOR),
}];

/// Plugin-owned table — WP `tourmaster`'s tours summary table, synced
/// from post_meta by `promote`. Core `schema.rs` keeps the same
/// `CREATE TABLE IF NOT EXISTS` for back-compat on existing DBs.
const TOURS_DDL: &str = "CREATE TABLE IF NOT EXISTS tours(
  post_id INTEGER PRIMARY KEY,
  duration_days INTEGER,
  departure_dates_json TEXT,
  price_range TEXT,
  max_people INTEGER,
  tour_time TEXT,
  gallery_json TEXT,
  rating REAL,
  view_count INTEGER
)";

pub(crate) struct Tourmaster;

impl PluginAdapter for Tourmaster {
    fn slug(&self) -> &'static str {
        "tourmaster"
    }
    fn kinds(&self) -> &'static [KindDef] {
        KINDS
    }
    fn screens(&self) -> &'static [ScreenDef] {
        screens::SCREENS
    }
    fn submenu(&self, db: &Path, _def: &KindDef) -> Option<Vec<(String, String)>> {
        Some(submenu(db))
    }
    fn migrate(&self, db: &Path) -> Result<()> {
        let c = crate::db::conn(db)?;
        c.execute_batch(TOURS_DDL)?;
        let n = promote(&c)?;
        if n > 0 {
            eprintln!("tourmaster: {n} rows promoted");
        }
        Ok(())
    }
    fn clone_post(&self, tx: &Connection, new_id: i64, old_id: i64) -> Result<()> {
        tx.execute(sql!("posts__insert_into_tours"), params![new_id, old_id])
            .map_err(|e| anyhow::anyhow!("clone tours {new_id}<={old_id}: {e}"))?;
        Ok(())
    }
    fn backend(&self) -> Backend {
        Backend::Rust
    }
}

pub(crate) static ADAPTER: Tourmaster = Tourmaster;

/// Tour submenu mirrors tourmaster: All/Add Tour, Add New Filter, then
/// every non-post taxonomy present in `terms`/`post_terms` (custom tour
/// taxs + theme taxonomies like Activites/Pays).
fn submenu(db: &Path) -> Vec<(String, String)> {
    let mut sub = vec![
        ("/admin/tours".into(), "All Tour".into()),
        ("/admin/new/tour".into(), "Add New Tour".into()),
        ("/admin/plugins/tourmaster/new-filter".into(), "Add New Filter".into()),
    ];
    for t in crate::db::list_taxonomies(db).unwrap_or_default() {
        if t == "category" || t == "post_tag" {
            continue;
        }
        let (label, _) = crate::admin::tax_label(&t);
        sub.push((format!("/admin/tax/{t}"), label));
    }
    sub
}

/// `promote_tours`: sync post_meta → `tours` summary table for every
/// `kind='tour'` post missing a row (WP tourmaster's on-save denormalizer).
fn promote(c: &Connection) -> Result<u64> {
    let ids: Vec<i64> = {
        let mut s = c.prepare(sql!("posts__select_id_from"))?;
        let rows = s.query_map([], |r| r.get(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let mut n = 0u64;
    for id in ids {
        let meta = |k: &str| -> Option<String> {
            c.query_row(sql!("posts__select_meta_value_from"), params![id, k], |r| r.get(0)).ok()
        };
        let duration: Option<i64> = meta("tourmaster-tour-duration").and_then(|s| s.parse().ok());
        let price = meta("tourmaster-tour-price-range");
        let max_p: Option<i64> = meta("tourmaster-max-people").and_then(|s| s.parse().ok());
        let dates = meta("tourmaster-tour-date");
        let views: Option<i64> = meta("tourmaster-view-count").and_then(|s| s.parse().ok());
        let rating: Option<f64> = meta("tourmaster-tour-rating").and_then(|s| s.parse().ok());
        c.execute(
            sql!("posts__insert_into_tours_24"),
            params![id, duration, price, max_p, dates, views, rating],
        )?;
        n += 1;
    }
    Ok(n)
}

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
<div class="tourmaster-page-option-head-save" data-post-id="{id}" data-ajax-url="/admin/plugins/tourmaster/save-option" data-failed-head="An error occurs" data-failed-message="Please use wordpress update button to update the page instead." ><i class="fa fa-save"></i>Save Settings</div>
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
var html_option_val = {"text":{"ajaxurl":"/admin/plugins/tourmaster/save-option","error_head":"An error occurs","error_message":"Please refresh the page to try again.","nonce":"","upload_media":"Select or Upload Media","choose_media":"Use this media"},"tabs":{"title_text":"Item's Title","tab_checkbox_on":"On","tab_checkbox_off":"Off"}};
var tourmaster_utility = {"confirm_head":"Just to confirm","confirm_text":"Are you sure to do this ?","confirm_sub":"* Please noted that this could not be undone.","confirm_yes":"Yes","confirm_no":"No"};
</script>
<script src="/admin/wp/jquery-ui.min.js"></script>
<script src="/admin/wp/iris.min.js"></script>
<script src="/admin/wp/color-picker.min.js"></script>
<script src="/admin/wp/gdlr/wp-media-shim.js"></script>
<script src="/admin/wp/tourmaster/framework/js/utility.js"></script>
<script src="/admin/wp/tourmaster/framework/js/html-option.js"></script>"#
}

