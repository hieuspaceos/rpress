//! Tourmaster's adapter-owned admin endpoints, dispatched from
//! `/admin/plugins/tourmaster/<path>`. Screen handlers pick their own
//! extractors from the raw request; auth is `need_auth` like every
//! other admin route.

use super::super::{ScreenDef, ScreenFuture};
use crate::admin::{check_csrf, chrome, html_admin, App};
use crate::db as crud;

use axum::extract::{Form, FromRequest, Request};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect};
use serde::Deserialize;

pub(crate) static SCREENS: &[ScreenDef] = &[
    ScreenDef { path: "save-option", title: "Save Tour Option", min_role: "editor", handle: save_option },
    ScreenDef { path: "new-filter", title: "Add New Filter", min_role: "editor", handle: new_filter },
];

#[derive(Deserialize)]
pub(crate) struct TourOptionAjax {
    #[serde(default)]
    csrf: String,
    #[serde(default)]
    action: String,
    post_id: i64,
    #[serde(default)]
    value: String,
}

/// `POST /admin/plugins/tourmaster/save-option` — the metabox's own
/// ajax save (tourmaster page-option's `tourmaster_save_tour_option`).
fn save_option<'a>( // gate-allow: sync — every caller reaches this on a blocking thread (spawn_blocking / build / boot)
    _app: &'a App,
    s: &'a crate::admin::Sess,
    sc: &'a crate::admin::SiteCtx,
    _headers: HeaderMap,
    _rest: &'a str,
    req: Request,
) -> ScreenFuture<'a> {
    Box::pin(async move {
        let Ok(Form(f)) = Form::<TourOptionAjax>::from_request(req, &()).await else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        // WP-parity: the plugin's field buttons (booking/mail actions)
        // post no nonce — they now 403 at the dispatch CSRF gate before
        // reaching this handler; this branch stays for direct internal
        // calls that bypass dispatch.
        if check_csrf(&s, &f.csrf).is_err() && f.action.is_empty() {
            return axum::Json(serde_json::json!({
                "status": "failed", "head": "Invalid Nonce",
                "message": "Please use the Update button instead."
            }))
            .into_response();
        };
        if !f.action.is_empty() {
            return axum::Json(serde_json::json!({
                "status": "failed", "head": "Not migrated",
                "message": "Bookings and mail are not migrated to this CMS."
            }))
            .into_response();
        }
        let mut pf = crud::PostForm {
            kind: "tour".into(),
            ..Default::default()
        };
        pf.extra
            .insert("tourmaster-tour-option".into(), f.value);
        let out: Result<(), String> = (|| {
            let c = crate::db::conn(&sc.db).map_err(|e| e.to_string())?;
            let kind: String = c
                .query_row(sql!("verify__select_kind_from"), [f.post_id], |r| r.get(0))
                .map_err(|_| "post not found".to_string())?;
            if kind != "tour" {
                return Err("not a tour".into());
            }
            // tx wrap: the plugin save writes several tour-meta rows — a
            // failure mid-save must roll back, not leave a half blob.
            let tx = c.unchecked_transaction().map_err(|e| e.to_string())?;
            let Some(e) = super::super::editor_for(&kind) else {
                return Err(format!("no editor registered for kind {kind}"));
            };
            (e.save)(&tx, f.post_id, &pf).map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())
        })();
        let out = match out {
            Ok(()) => serde_json::json!({"status":"success","head":"Successfully Save","message":""}),
            Err(e) => serde_json::json!({"status":"failed","head":"An error occurs","message":e}),
        };
        axum::Json(out).into_response()
    })
}

#[derive(Deserialize)]
struct NewFilterForm {
    csrf: String,
    label: String,
    slug: String,
}

fn new_filter_html(s: &crate::admin::Sess) -> String {
    format!(
        r#"{chrome}<div class="wrap"><h1>Add New Filter</h1>
<p>Create a custom tour taxonomy (tourmaster filter). It appears under Tour → menu and on the tour edit screen.</p>
<form method="post" action="/admin/plugins/tourmaster/new-filter" class="form-table">
<input type="hidden" name="csrf" value="{csrf}">
<table class="form-table"><tbody>
<tr><th scope="row"><label for="nf-label">Label</label></th>
<td><input id="nf-label" name="label" type="text" class="regular-text" required></td></tr>
<tr><th scope="row"><label for="nf-slug">Slug</label></th>
<td><input id="nf-slug" name="slug" type="text" class="regular-text" placeholder="auto from label"></td></tr>
</tbody></table>
<p class="submit"><button class="button button-primary">Add Filter</button></p>
</form></div>"#,
        chrome = chrome("tours", s),
        csrf = s.csrf,
    )
}

/// `/admin/plugins/tourmaster/new-filter` — custom taxonomy creator
/// (tourmaster's `tourmaster_custom_taxonomy` equivalent). GET renders
/// the form; POST inserts into `plugin:tourmaster:tour_custom_taxs`.
fn new_filter<'a>( // gate-allow: sync — every caller reaches this on a blocking thread (spawn_blocking / build / boot)
    _app: &'a App,
    s: &'a crate::admin::Sess,
    sc: &'a crate::admin::SiteCtx,
    _headers: HeaderMap,
    _rest: &'a str,
    req: Request,
) -> ScreenFuture<'a> {
    Box::pin(async move {
        if req.method() == axum::http::Method::GET {
            return html_admin(&s, "Add New Filter", &new_filter_html(&s), true, "edit-tags").into_response();
        }
        let Ok(Form(f)) = Form::<NewFilterForm>::from_request(req, &()).await else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        if let Err(r) = check_csrf(&s, &f.csrf) {
            return r;
        }
        let slug = if f.slug.trim().is_empty() { crud::slugify(&f.label) } else { f.slug.trim().to_string() };
        // Reserved tax-segment slugs would be shadowed by literal routes
        // (`/admin/tax/save`, …) or this screen — reject up front.
        const RESERVED: [&str; 6] = ["new-filter", "save", "delete", "bulk", "edit", "json"];
        if slug.is_empty() || RESERVED.contains(&slug.as_str()) {
            return StatusCode::BAD_REQUEST.into_response();
        }
        let mut taxs = crate::db::plugin_setting_json_map(&sc.db, "tourmaster", "tour_custom_taxs");
        taxs.insert(slug.clone(), f.label.trim().to_string());
        if crate::db::plugin_setting_json_put_map(&sc.db, "tourmaster", "tour_custom_taxs", &taxs).is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Redirect::to(&format!("/admin/tax/{slug}")).into_response()
    })
}
