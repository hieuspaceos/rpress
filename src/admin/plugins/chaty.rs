//! Chaty (floating contact button) — screens-only adapter.
//!
//! No post kinds, no bridge. The render-side emit lives in
//! `src/render/site_data.rs` (`chaty_js`/`chaty_css`); this adapter only
//! owns the `/admin/plugins/chaty/settings` screen which writes the same
//! `cht_*`/`chaty_*` option keys the WP plugin used.

use super::index::{save_settings_fields, settings_form_html, FieldKind, FieldSpec, SettingsForm};
use super::{Backend, KindDef, PluginAdapter, PluginDescriptor, ScreenDef, ScreenFuture};
use crate::admin::{chrome, html_page, App, Sess, SiteCtx};
use axum::{
    extract::{Form, FromRequest, Request},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Redirect},
};

pub(crate) struct Chaty;
pub(crate) static ADAPTER: Chaty = Chaty;

impl PluginAdapter for Chaty {
    fn slug(&self) -> &'static str {
        "chaty"
    }
    fn kinds(&self) -> &'static [KindDef] {
        &[]
    }
    fn backend(&self) -> Backend {
        Backend::Rust
    }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: "Chaty",
            blurb: "Floating WhatsApp/contact buttons (chaty.pro)",
        }
    }
    fn settings_prefixes(&self) -> &'static [&'static str] {
        &["cht_", "chaty_"]
    }
    fn screens(&self) -> &'static [ScreenDef] {
        &SCREENS
    }
}

static SCREENS: &[ScreenDef] = &[ScreenDef {
    path: "settings",
    title: "Chaty Settings",
    handle: settings,
}];

fn widget_fields(sfx: &str) -> Vec<FieldSpec> {
    // FieldSpec keys are 'static — per-suffix specs are built from
    // literal match on the suffix (only "" and "_1" exist today).
    if sfx.is_empty() {
        vec![
            FieldSpec { key: "cht_active", label: "Enable widget", kind: FieldKind::Checkbox, help: "Master switch for this widget." },
            FieldSpec { key: "cht_cta", label: "CTA text", kind: FieldKind::Text, help: "e.g. “Discutez avec nous”" },
            FieldSpec { key: "cht_cta_bg_color", label: "CTA background", kind: FieldKind::Text, help: "CSS color" },
            FieldSpec { key: "cht_cta_text_color", label: "CTA text color", kind: FieldKind::Text, help: "CSS color" },
            FieldSpec { key: "cht_position", label: "Position", kind: FieldKind::Select(&[("left", "Left"), ("right", "Right")]), help: "" },
            FieldSpec { key: "chaty_icons_view", label: "Icon layout", kind: FieldKind::Select(&[("vertical", "Vertical"), ("horizontal", "Horizontal")]), help: "" },
            FieldSpec { key: "cht_color", label: "Widget color", kind: FieldKind::Text, help: "CSS color" },
            FieldSpec { key: "cht_icon_color", label: "Widget icon color", kind: FieldKind::Text, help: "CSS color" },
            FieldSpec { key: "cht_widget_size", label: "Widget size (px)", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_bottom_spacing", label: "Bottom spacing (px)", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_side_spacing", label: "Side spacing (px)", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "chaty_default_state", label: "Default state", kind: FieldKind::Select(&[("click", "Expand on click"), ("hover", "Expand on hover")]), help: "" },
            FieldSpec { key: "cht_pending_messages", label: "Pending message badge", kind: FieldKind::Select(&[("off", "Off"), ("on", "On")]), help: "" },
            FieldSpec { key: "cht_number_of_messages", label: "Badge count", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_close_button", label: "Show close button", kind: FieldKind::Select(&[("yes", "Yes"), ("no", "No")]), help: "" },
            FieldSpec { key: "cht_close_button_text", label: "Close button label", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "chaty_attention_effect", label: "Attention effect", kind: FieldKind::Select(&[("waggle", "Waggle"), ("bounce", "Bounce"), ("pulse", "Pulse"), ("", "None")]), help: "" },
            FieldSpec { key: "chaty_custom_css", label: "Custom CSS", kind: FieldKind::Textarea, help: "Raw CSS injected into the widget." },
        ]
    } else {
        vec![
            FieldSpec { key: "cht_active_1", label: "Enable widget", kind: FieldKind::Checkbox, help: "" },
            FieldSpec { key: "cht_cta_1", label: "CTA text", kind: FieldKind::Text, help: "e.g. “Habla con nosotros”" },
            FieldSpec { key: "cht_cta_bg_color_1", label: "CTA background", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_cta_text_color_1", label: "CTA text color", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_position_1", label: "Position", kind: FieldKind::Select(&[("left", "Left"), ("right", "Right")]), help: "" },
            FieldSpec { key: "chaty_icons_view_1", label: "Icon layout", kind: FieldKind::Select(&[("vertical", "Vertical"), ("horizontal", "Horizontal")]), help: "" },
            FieldSpec { key: "cht_color_1", label: "Widget color", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_icon_color_1", label: "Widget icon color", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_widget_size_1", label: "Widget size (px)", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_bottom_spacing_1", label: "Bottom spacing (px)", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_side_spacing_1", label: "Side spacing (px)", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "chaty_default_state_1", label: "Default state", kind: FieldKind::Select(&[("click", "Expand on click"), ("hover", "Expand on hover")]), help: "" },
            FieldSpec { key: "cht_pending_messages_1", label: "Pending message badge", kind: FieldKind::Select(&[("off", "Off"), ("on", "On")]), help: "" },
            FieldSpec { key: "cht_number_of_messages_1", label: "Badge count", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "cht_close_button_1", label: "Show close button", kind: FieldKind::Select(&[("yes", "Yes"), ("no", "No")]), help: "" },
            FieldSpec { key: "cht_close_button_text_1", label: "Close button label", kind: FieldKind::Text, help: "" },
            FieldSpec { key: "chaty_attention_effect_1", label: "Attention effect", kind: FieldKind::Select(&[("waggle", "Waggle"), ("bounce", "Bounce"), ("pulse", "Pulse"), ("", "None")]), help: "" },
            FieldSpec { key: "chaty_custom_css_1", label: "Custom CSS", kind: FieldKind::Textarea, help: "" },
        ]
    }
}

fn settings<'a>(
    _app: &'a App,
    s: &'a Sess,
    sc: &'a SiteCtx,
    _headers: HeaderMap,
    _rest: &'a str,
    req: Request,
) -> ScreenFuture<'a> {
    Box::pin(async move {
        match *req.method() {
            Method::POST => {
                let Ok(Form(f)) = Form::<SettingsForm>::from_request(req, &()).await else {
                    return StatusCode::BAD_REQUEST.into_response();
                };
                // Two independent forms post here — each carries hidden
                // `sfx`. A missing/forged value must NOT silently target
                // widget 0 (its spec would blank every cht_* row).
                let sfx = match f.fields.get("sfx").map(|s| s.as_str()) {
                    Some("") => "",
                    Some("_1") => "_1",
                    _ => return StatusCode::BAD_REQUEST.into_response(),
                };
                match save_settings_fields(s, sc, &f, &widget_fields(sfx)) {
                    Ok(()) => Redirect::to("/admin/plugins/chaty/settings").into_response(),
                    Err(r) => r,
                }
            }
            _ => {
                let Ok(conn) = crate::db::conn(&sc.db) else {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                };
                let val = |k: &str| {
                    conn.query_row("SELECT value FROM settings WHERE key=?1", [k], |r| r.get::<_, String>(0))
                        .unwrap_or_default()
                };
                let mut body = String::from("<h1>Chaty Settings.</h1><hr class=\"wp-header-end\"><style>.badge{display:inline-block;padding:2px 8px;border-radius:10px;font-size:11px;font-weight:600;background:var(--paper-2);color:var(--ink-2)}.badge-native{background:oklch(88% 0.1 155);color:oklch(30% 0.12 160)}</style>");
                for (label, sfx) in [("Widget 1 — FR", ""), ("Widget 2 — ES", "_1")] {
                    let active = val(&format!("cht_active{sfx}")) == "1";
                    body.push_str(&format!(
                        "<h2 style=\"margin:22px 0 6px\">{label} <span class=\"badge {}\">{}</span></h2>",
                        if active { "badge-native" } else { "" },
                        if active { "active" } else { "disabled" },
                    ));
                    body.push_str(&settings_form_html(
                        &conn,
                        "/admin/plugins/chaty/settings",
                        &s.csrf,
                        "",
                        &widget_fields(sfx),
                    ).replacen(
                        "<input type=\"hidden\" name=\"csrf\"",
                        &format!("<input type=\"hidden\" name=\"sfx\" value=\"{sfx}\"><input type=\"hidden\" name=\"csrf\""),
                        1,
                    ));
                }
                html_page(s, "Chaty", &format!("{}<div class=\"wrap\">{}</div>", chrome("plugins", s), body), false).into_response()
            }
        }
    })
}
