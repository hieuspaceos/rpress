//! Small screens-only adapters — plugins whose front-end emit already
//! exists elsewhere (render/shortcode.rs, chrome.rs, site_data.rs); each
//! only needs the WP-style `/admin/plugins/<slug>/settings` home.
//!
//! Covered: wpdiscuz, cf7 (+ cf7sr turnstile), metaslider, site-reviews
//! (glsr), head-footer-code (auhfc). Every field maps to a key the
//! runtime or the WP plugin stored — no consumer-less settings; where a
//! plugin has no meaningful writable keys the screen says so and links
//! to the real surface instead of inventing a form.

use super::index::{save_settings_fields, settings_page_html, FieldKind, FieldSpec, SettingsForm};
use super::{Backend, KindDef, PluginAdapter, PluginDescriptor, ScreenDef, ScreenFuture};
use crate::admin::{App, Sess, SiteCtx};
use axum::{
    extract::{Form, FromRequest, Request},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Redirect},
};

/// One settings screen wiring shared by every adapter below: GET →
/// `settings_page_html`, POST → `save_settings_fields` + redirect.
macro_rules! settings_screen {
    ($fn_name:ident, $redirect:expr, $title:expr, $spec:expr) => {
        fn $fn_name<'a>(
            _app: &'a App,
            s: &'a Sess,
            sc: &'a SiteCtx,
            _h: HeaderMap,
            _rest: &'a str,
            req: Request,
        ) -> ScreenFuture<'a> {
            Box::pin(async move {
                match *req.method() {
                    Method::POST => {
                        let Ok(Form(f)) = Form::<SettingsForm>::from_request(req, &()).await else {
                            return StatusCode::BAD_REQUEST.into_response();
                        };
                        match save_settings_fields(s, sc, &f, $spec) {
                            Ok(()) => Redirect::to($redirect).into_response(),
                            Err(r) => r,
                        }
                    }
                    _ => settings_page_html(s, sc, $redirect, $title, $spec),
                }
            })
        }
    };
}

// ─── wpDiscuz ───────────────────────────────────────────────────────
// Comment rendering is already native (`comments_wpdiscuz.html`); the
// WP plugin's own options live under `wpdiscuz_*` (serialized form
// blobs — edited raw with round-trip validation). Discussion toggles
// stay on /admin/settings/discussion; this screen links there.

pub(crate) struct WpDiscuz;
pub(crate) static WPDISCUZ: WpDiscuz = WpDiscuz;

const WPDISCUZ_SPEC: &[FieldSpec] = &[
    FieldSpec { key: "", label: "", kind: FieldKind::Note("wpDiscuz form + moderation options are stored as serialized option blobs — edit raw; saving validates the round-trip. Core discussion toggles live under Settings → Discussion."), help: "" },
    FieldSpec { key: "wpdiscuz_form", label: "wpdiscuz_form", kind: FieldKind::SerializedTextarea, help: "Form definition blob (fields, layout)." },
    FieldSpec { key: "wpdiscuz_form_general_options", label: "Form general options", kind: FieldKind::SerializedTextarea, help: "Ordering, defaults, labels." },
    FieldSpec { key: "wpdiscuz_form_fields", label: "Form fields", kind: FieldKind::SerializedTextarea, help: "" },
    FieldSpec { key: "wpdiscuz_form_structure", label: "Form structure", kind: FieldKind::SerializedTextarea, help: "" },
    FieldSpec { key: "wpdiscuz_votes", label: "Vote settings", kind: FieldKind::SerializedTextarea, help: "" },
];

settings_screen!(wpdiscuz_settings, "/admin/plugins/wpdiscuz/settings", "wpDiscuz Settings", WPDISCUZ_SPEC);
static WPDISCUZ_SCREENS: &[ScreenDef] = &[ScreenDef { path: "settings", title: "wpDiscuz Settings", handle: wpdiscuz_settings }];

impl PluginAdapter for WpDiscuz {
    fn slug(&self) -> &'static str { "wpdiscuz" }
    fn kinds(&self) -> &'static [KindDef] { &[] }
    fn backend(&self) -> Backend { Backend::Rust }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: "wpDiscuz", blurb: "AJAX comments — rendered natively (comments_wpdiscuz)" }
    }
    fn settings_prefixes(&self) -> &'static [&'static str] { &["wpdiscuz_"] }
    fn screens(&self) -> &'static [ScreenDef] { WPDISCUZ_SCREENS }
}

// ─── Contact Form 7 (+ cf7sr turnstile keys) ────────────────────────
// Forms are rendered from `forms.markup` (see `contact-form-7` arm in
// render/shortcode.rs). Form bodies themselves live under
// /admin/contact — this screen carries only the global option keys the
// dump stored.

pub(crate) struct Cf7;
pub(crate) static CF7: Cf7 = Cf7;

const CF7_SPEC: &[FieldSpec] = &[
    FieldSpec { key: "", label: "", kind: FieldKind::Note("Form definitions (markup, mail) live under Contact → Contact Forms. These are the global keys the dump stored; the turnstile keys are dormant on rpress — reCAPTCHA script is not emitted."), help: "" },
    FieldSpec { key: "cf7sr_ts_key", label: "Turnstile site key", kind: FieldKind::Text, help: "Stored; not emitted (no third-party JS on output)." },
    FieldSpec { key: "cf7sr_ts_secret", label: "Turnstile secret", kind: FieldKind::Text, help: "" },
    FieldSpec { key: "cf7sr_ts_language", label: "Turnstile language", kind: FieldKind::Text, help: "" },
    FieldSpec { key: "cf7sr_ts_message", label: "Turnstile fail message", kind: FieldKind::Text, help: "" },
];

settings_screen!(cf7_settings, "/admin/plugins/cf7/settings", "Contact Form 7 Settings", CF7_SPEC);
static CF7_SCREENS: &[ScreenDef] = &[ScreenDef { path: "settings", title: "CF7 Settings", handle: cf7_settings }];

impl PluginAdapter for Cf7 {
    fn slug(&self) -> &'static str { "cf7" }
    fn kinds(&self) -> &'static [KindDef] { &[] }
    fn backend(&self) -> Backend { Backend::Rust }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: "Contact Form 7", blurb: "Forms + submissions (dynamic renderer)" }
    }
    fn settings_prefixes(&self) -> &'static [&'static str] { &["cf7", "cf7sr_"] }
    fn screens(&self) -> &'static [ScreenDef] { CF7_SCREENS }
}

// ─── MetaSlider ─────────────────────────────────────────────────────
// `[metaslider id=N]` expands to FlexSlider markup from `ml-slider` /
// `ml-slide` posts. The slider-level settings blob is per-post
// (`post_meta`); global defaults under `ml-slider_settings` may be
// absent from the dump — the textarea stays honest.

pub(crate) struct MetaSlider;
pub(crate) static METASLIDER: MetaSlider = MetaSlider;

const METASLIDER_SPEC: &[FieldSpec] = &[
    FieldSpec { key: "", label: "", kind: FieldKind::Note("Each slider's own settings live on its post (Posts → ml-slider). The global defaults blob below is serialized; save validates the round-trip."), help: "" },
    FieldSpec { key: "ml-slider_settings", label: "ml-slider_settings", kind: FieldKind::SerializedTextarea, help: "Global defaults (empty when the dump carried none)." },
];

settings_screen!(metaslider_settings, "/admin/plugins/metaslider/settings", "MetaSlider Settings", METASLIDER_SPEC);
static METASLIDER_SCREENS: &[ScreenDef] = &[ScreenDef { path: "settings", title: "MetaSlider Settings", handle: metaslider_settings }];

impl PluginAdapter for MetaSlider {
    fn slug(&self) -> &'static str { "metaslider" }
    fn kinds(&self) -> &'static [KindDef] { &[] }
    fn backend(&self) -> Backend { Backend::Rust }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: "MetaSlider", blurb: "Image sliders — [metaslider id] → FlexSlider markup" }
    }
    fn settings_prefixes(&self) -> &'static [&'static str] { &["ml-slider_"] }
    fn screens(&self) -> &'static [ScreenDef] { METASLIDER_SCREENS }
}

// ─── Site Reviews (glsr) ────────────────────────────────────────────
// Reviews render via `templates/elements/site_reviews*.html` +
// `reviews.html`; the plugin's own admin stored almost nothing in
// wp_options (glsr_* keys are engine/migration bookkeeping). The screen
// keeps those read-write for completeness and says what they are.

pub(crate) struct SiteReviews;
pub(crate) static SITE_REVIEWS: SiteReviews = SiteReviews;

const SITE_REVIEWS_SPEC: &[FieldSpec] = &[
    FieldSpec { key: "", label: "", kind: FieldKind::Note("Site Reviews' real config lived in its own tables on WP; the dump only carries engine bookkeeping. Review moderation lives under Site Reviews in the admin menu."), help: "" },
    FieldSpec { key: "glsr_migrations", label: "glsr_migrations", kind: FieldKind::SerializedTextarea, help: "Migration log blob — bookkeeping, safe to leave untouched." },
    FieldSpec { key: "glsr_last_migration_run", label: "Last migration run", kind: FieldKind::Text, help: "" },
];

settings_screen!(site_reviews_settings, "/admin/plugins/site-reviews/settings", "Site Reviews Settings", SITE_REVIEWS_SPEC);
static SITE_REVIEWS_SCREENS: &[ScreenDef] = &[ScreenDef { path: "settings", title: "Site Reviews Settings", handle: site_reviews_settings }];

impl PluginAdapter for SiteReviews {
    fn slug(&self) -> &'static str { "site-reviews" }
    fn kinds(&self) -> &'static [KindDef] { &[] }
    fn backend(&self) -> Backend { Backend::Rust }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: "Site Reviews", blurb: "Review shortcodes — rendered natively" }
    }
    fn settings_prefixes(&self) -> &'static [&'static str] { &["glsr_"] }
    fn screens(&self) -> &'static [ScreenDef] { SITE_REVIEWS_SCREENS }
}

// ─── Head & Footer Code (auhfc) ─────────────────────────────────────
// `auhfc_settings_sitewide` already feeds chrome.rs `head_extra`; the
// dump keeps it as a serialized map {head, footer, article}. Serialized
// round-trip on save — same contract as everywhere else.

pub(crate) struct HeadFooter;
pub(crate) static HEAD_FOOTER: HeadFooter = HeadFooter;

const HEAD_FOOTER_SPEC: &[FieldSpec] = &[
    FieldSpec { key: "", label: "", kind: FieldKind::Note("Sitewide snippet blob — feeds the <head> injection (site_head_html in render). Serialized; save validates the round-trip."), help: "" },
    FieldSpec { key: "auhfc_settings_sitewide", label: "Sitewide snippets", kind: FieldKind::SerializedTextarea, help: "Serialized map {head, footer, article}." },
];

settings_screen!(head_footer_settings, "/admin/plugins/head-footer-code/settings", "Head & Footer Settings", HEAD_FOOTER_SPEC);
static HEAD_FOOTER_SCREENS: &[ScreenDef] = &[ScreenDef { path: "settings", title: "Head & Footer Settings", handle: head_footer_settings }];

impl PluginAdapter for HeadFooter {
    fn slug(&self) -> &'static str { "head-footer-code" }
    fn kinds(&self) -> &'static [KindDef] { &[] }
    fn backend(&self) -> Backend { Backend::Rust }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: "Head & Footer Code", blurb: "Sitewide <head>/footer snippet injection" }
    }
    fn settings_prefixes(&self) -> &'static [&'static str] { &["auhfc_"] }
    fn screens(&self) -> &'static [ScreenDef] { HEAD_FOOTER_SCREENS }
}
