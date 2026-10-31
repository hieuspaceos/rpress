//! Admin kernel for `rpress serve`: `SiteCtx`/`App`/`Sess` shared state,
//! router construction (`serve()`), and session/auth plumbing. The
//! WP-chrome frame lives in `chrome.rs`, shared helpers in `util.rs`,
//! plugin screen dispatch in `plugin_dispatch.rs`; the public-facing
//! file/asset serving path lives in `serve.rs`. Admin screens live in
//! the sibling modules declared below.
use anyhow::Result;
use crate::{auth as crate_auth, db as crud, generate, intake, schema};
use crate::plugins;
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use parking_lot::{Mutex, RwLock};
use rand::RngCore;
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(crate) mod admin;
mod auth;
mod dashboard;
mod gdlr;
pub(crate) mod bridge;
pub(crate) mod posts;
mod revisions;
mod comments;
mod list;
pub(crate) mod rankmath;
mod users;
mod tax;
mod contact;
mod forms_editor;
mod menus;
mod widgets;
mod settings;
mod backup;
mod chrome;
mod util;
mod plugin_dispatch;
pub(crate) mod serve;
// `pub` facade: the symlink-escape hardening test in tests/hardening.rs
// exercises `rpress::admin::safe_read_under`; the impl lives in serve.rs.
pub use serve::safe_read_under;
// `pub` facade for the audit-remediation regressions in
// tests/audit_remediation.rs — same pattern as safe_read_under above.
#[doc(hidden)]
pub use admin::safe_back;
#[doc(hidden)]
pub use serve::request_host;
// Facade re-exports: code outside src/admin/ already calls
// `crate::admin::{esc, esc_attr, urlenc, tax_label*}` and siblings import
// these via `use super::*` — re-export the moved modules so those names
// keep resolving at the same path after the split.
pub(crate) use chrome::*;
pub(crate) use util::*;

#[derive(Clone)]
pub(crate) struct SiteCtx {
    pub(crate) slug: String,
    /// Comma-separated PhpBridge opt-in slugs from SiteEnv; "" = off.
    pub(crate) plugins_php: String,
    pub(crate) name: String,
    /// Per-kind URL-base prefixes resolved from the site's TOUR_BASE env
    /// (plugins::resolve_bases). Replaces the raw tour-base string.
    pub(crate) kind_bases: plugins::KindBases,
    pub(crate) db: PathBuf,
    pub(crate) templates: PathBuf,
    pub(crate) out: PathBuf,
    /// Persistent data dir: user uploads + vendored plugin assets.
    /// Unlike `out` (disposable cache), `www` must be backed up.
    pub(crate) www: PathBuf,
    /// Per-plugin bridge state (`{site}:{slug}` → BridgeState); bridges
    /// spawn lazily on the first proxied admin request.
    pub(crate) bridges: bridge::Bridges,
    pub(crate) generating: Arc<AtomicBool>,
    /// Settings snapshot (key→value) — a full-table read replaces dozens
    /// of per-call `setting_value` conn opens. Loaded lazily, refreshed
    /// after TTL (bounded staleness for out-of-band writes like bridge
    /// pull), busted eagerly by `bust_cache` (every admin settings save
    /// already funnels through it).
    pub(crate) settings_cache: Arc<RwLock<Option<(Instant, Arc<HashMap<String, String>>)>>>,
    /// `publish_due` last run — throttles the minute-tick future scan
    /// (serve.rs); 30s granularity matches minute-level schedule semantics.
    pub(crate) publish_due_at: Arc<Mutex<Option<Instant>>>,
}

/// How long a cached settings snapshot stays trusted without a DB check —
/// covers writers that bypass `bust_cache` (bridge pull, plugin adapters).
const SETTINGS_TTL: Duration = Duration::from_secs(30);

impl SiteCtx {
    /// Settings map, memory-resident within the TTL. The snapshot is a
    /// single `SELECT key,value` — cheaper than N single-key lookups even
    /// on a warm connection.
    pub(crate) fn settings(&self) -> Arc<HashMap<String, String>> {
        {
            let g = self.settings_cache.read();
            if let Some((at, map)) = g.as_ref() {
                if at.elapsed() < SETTINGS_TTL {
                    return map.clone();
                }
            }
        }
        let mut fresh = HashMap::new();
        if let Ok(c) = crate::db::conn(&self.db) {
            if let Ok(mut s) = c.prepare(sql!("settings__select_all")) {
                if let Ok(rows) = s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
                    for r in rows.flatten() {
                        fresh.insert(r.0, r.1);
                    }
                }
            }
        }
        let fresh = Arc::new(fresh);
        *self.settings_cache.write() = Some((Instant::now(), fresh.clone()));
        fresh
    }

    /// Cached `setting_value` — misses and empties return `default`.
    pub(crate) fn setting(&self, key: &str, default: &str) -> String {
        self.settings()
            .get(key)
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Drop the settings snapshot — call from `bust_cache` after any admin
    /// write that can change a setting.
    pub(crate) fn settings_bust(&self) {
        *self.settings_cache.write() = None;
    }
}

#[derive(Clone)]
pub(crate) struct App {
    pub(crate) sites: Arc<HashMap<String, SiteCtx>>,
    pub(crate) default_site: String,
    pub(crate) sessions: Arc<Mutex<HashMap<String, Sess>>>,
    pub(crate) login_hits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    /// Token → (user_id, issued_at, failed_code_attempts) for the 2FA
    /// half-state between password accept and TOTP verify. Session-less:
    /// a valid password + stolen token still isn't a session.
    pub(crate) pending_2fa: Arc<Mutex<HashMap<String, (i64, Instant, usize)>>>,
    pub(crate) intake_rate: Arc<Mutex<intake::Rate>>,
    /// Separate bucket for miss-render requests so bot scans can't burn
    /// the form-intake rate (or vice versa). Same in-memory semantics.
    pub(crate) miss_rate: Arc<Mutex<intake::Rate>>,
}

pub(crate) const SESSION_COOKIE: &str = "rpress_session";
pub(crate) const CLEAR_SESSION_COOKIE: &str = "rpress_session=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0";

/// Legacy single-string view of `SiteCtx::kind_bases`: the tour kind's
/// URL base (`""` when no adapter registers one). Boundary helper for
/// call sites that still take the pre-KindBases `tour_base: &str`.
pub(crate) fn tour_base(sc: &SiteCtx) -> &str {
    plugins::base_for(&sc.kind_bases, "tour").unwrap_or_default()
}


#[derive(Clone)]
pub(crate) struct Sess {
    pub(crate) site: String,
    pub(crate) site_name: String,
    pub(crate) user_id: i64,
    pub(crate) email: String,
    pub(crate) role: String,
    pub(crate) db: PathBuf,
    pub(crate) csrf: String,
    pub(crate) exp: Instant,
    /// Last time `session_touch` ran (throttles the per-request write).
    pub(crate) touched: Instant,
}

/// `sites`: `(host_header, db, templates, out, www)` — multi-site support.
/// `out` = disposable HTML cache; `www` = persistent uploads/plugins/assets.
pub async fn serve(db: PathBuf, templates: PathBuf, out: PathBuf, www: PathBuf, bind: SocketAddr, site_slug: String, host: String, name: String, tour_base: String, plugins_php: String) -> Result<()> {
    let kind_bases = plugins::resolve_bases(&tour_base);
    // Schema must be current before we serve — the post_meta dedupe +
    // unique-index migration lives here; silently continuing would serve
    // on a wrong schema (audit M18).
    schema::migrate(&db).map_err(|e| anyhow::anyhow!("schema migrate failed: {e}"))?;
    let _ = crud::ensure_media_cols(&db);
    let uploads_root = www.join("wp-content/uploads");
    let _ = crud::media_backfill_dates(&db, &uploads_root);
    plugins::migrate_all(&db);
    // Boot duties that used to live inside `rpress generate`:
    //   1. seed default widgets/sidebars so a fresh DB isn't empty
    //   2. purge 30-day-old trash (WP _wp_scheduled_delete equivalent)
    //   3. emit the utility files (sitemap/robots/feeds/404/search/
    //      redirects.map) — cheap SQL→file, no renders
    if let Ok(boot_conn) = crate::db::conn(&db) {
        let _ = crate::widgets::ensure(&boot_conn);
    }
    crate::sitefx::purge_duties(&db, &out, &tour_base);
    serve::purge_alias_caches(&db, &out);
    {
        let site_url_boot = crate::db::setting_value(&db, "site_url", "/");
        if let Err(e) = crate::sitefx::emit_site_files(&db, &templates, &out, &site_url_boot, &tour_base) {
            eprintln!("emit_site_files: {e:#}");
        }
    }
    let site = SiteCtx {
        slug: site_slug.clone(),
        plugins_php,
        name: name.clone(),
        kind_bases,
        db,
        templates,
        out,
        www,
        bridges: bridge::Bridges::default(),
        generating: Arc::new(AtomicBool::new(false)),
        settings_cache: Arc::new(RwLock::new(None)),
        publish_due_at: Arc::new(Mutex::new(None)),
    };
    let mut sites = HashMap::new();
    sites.insert(host.clone(), site);
    let state = App {
        sites: Arc::new(sites),
        default_site: host,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        login_hits: Arc::new(Mutex::new(HashMap::new())),
        pending_2fa: Arc::new(Mutex::new(HashMap::new())),
        intake_rate: Arc::new(Mutex::new(intake::Rate::default())),
        miss_rate: Arc::new(Mutex::new(intake::Rate::default())),
    };
    let app = Router::new()
        .route("/", get(serve::public_root))
        .route("/admin/login", get(auth::login_page))
        .route("/health", get(serve::health))
        .route("/api/login", post(auth::login))
        .route("/api/logout", post(auth::logout))
        .route("/admin/2fa", get(auth::two_fa_page).post(auth::two_fa_verify))

        .route("/admin", get(dashboard::dashboard).post(dashboard::screen_options_save))
        .route("/admin/draft", post(dashboard::save_draft))
        .route("/admin/health", get(dashboard::health_page))
        .route("/admin/new/{kind}", get(posts::new_post))
        .route("/admin/edit/{id}", get(posts::edit_post))
        .route("/admin/save", post(posts::save_post))
        .route("/admin/trash", post(posts::trash_post))

        .route("/admin/clone", post(posts::clone_post))
        .route("/admin/quickedit", post(posts::quick_edit))
        .route("/admin/postdata/{id}", get(posts::post_data))
        .route("/admin/bulk", post(posts::bulk_posts))
        .route("/admin/posts/empty", post(posts::posts_empty))
        .route("/admin/posts/{id}/preview", get(posts::preview_post))
        .route("/admin/posts/{id}/revisions", get(revisions::revisions_page))
        .route("/admin/revisions/{rid}", get(revisions::revision_diff))
        .route("/admin/revisions/{rid}/restore", post(revisions::revision_restore))
        .route("/admin/seo/cell/{id}", get(rankmath::seo_cell_frag))
        .route("/admin/seo/analysis/{id}", get(rankmath::seo_analysis_frag))
        .route("/admin/tax/new-filter", get(tax::tax_new_filter_redirect).post(tax::tax_new_filter_redirect))
        .route("/admin/tax/{tax}", get(tax::tax_page))
        .route("/admin/tax/save", post(tax::tax_save))
        .route("/admin/tax/edit/{id}", get(tax::tax_edit))
        .route("/admin/tax/delete", post(tax::tax_delete))
        .route("/admin/tax/bulk", post(tax::tax_bulk))
        .route("/admin/media", get(admin::media_page).post(admin::media_screen_options))
        .route("/admin/media/new", get(admin::media_new_page))
        .route("/admin/media/upload", post(admin::media_upload))
        .route("/admin/media/edit/{id}", get(admin::media_edit_page))
        .route("/admin/media/save", post(admin::media_save))
        .route("/admin/media/delete", post(admin::media_delete))
        .route("/admin/media/detach", post(admin::media_detach))
        .route("/admin/plugins", get(plugins::index::index_page))
        .route("/admin/plugins/{slug}/{*rest}", get(plugin_dispatch::plugin_screen).post(plugin_dispatch::plugin_screen))
        .route("/admin/api/tour/save-option", post(plugin_dispatch::save_option_shim))
        .route("/admin/media/bulk", post(admin::media_bulk))
        .route("/admin/media/download/{id}", get(admin::media_download))
        .route("/admin/media/json/{id}", get(admin::media_json))
        .route("/admin/media/replace/{id}", post(admin::media_replace))
        .route("/admin/media/items", get(admin::media_items))
        .route("/admin/media/regenerate", post(admin::media_regenerate))
        .route("/admin/menus", get(menus::menus_page))
        .route("/admin/menus/create", post(menus::menus_create))
        .route("/admin/menus/delete", post(menus::menus_delete))
        .route("/admin/menus/add-items", post(menus::menus_add))
        .route("/admin/menus/move", post(menus::menus_move))
        .route("/admin/tools/backup", get(backup::backup_page).post(backup::backup_run))
        .route("/admin/tools/backup/download/{name}", get(backup::backup_download))
        .route("/admin/menus/save", post(menus::menus_save))
        .route("/admin/menus/locations", post(menus::menus_locations_save))
        .route("/admin/menus/screen", post(menus::menus_screen_save))
        .route("/admin/submissions", get(contact::submissions_redirect))
        .route("/admin/widgets", get(widgets::widgets_page))
        .route("/admin/widgets/add", post(widgets::widgets_add))
        .route("/admin/widgets/save", post(widgets::widgets_save))
        .route("/admin/widgets/delete", post(widgets::widgets_delete))
        .route("/admin/widgets/order", post(widgets::widgets_order))
        .route("/admin/widgets/sidebar/add", post(widgets::sidebar_add))
        .route("/admin/widgets/sidebar/delete", post(widgets::sidebar_delete))
        .route("/admin/submissions/mod", post(contact::submissions_mod_redirect))
        .route("/admin/wp/{*path}", get(serve::wp_asset))
        .route("/wp-content/uploads/{*path}", get(serve::uploads))
        .route("/admin/front-css/{*path}", get(serve::front_css))
        .route("/admin/vendor/htmx.min.js", get(admin::htmx))
        .route("/admin/{section}", get(posts::section))
        .route("/api/v1/gdlr/media", get(gdlr::media_list))
        .route("/admin/api/gdlr/ajax", post(plugin_dispatch::gdlr_ajax))
        .route("/api/generate", post(admin::api_generate))
        .route("/api/generate/status", get(admin::generate_status))
        .route("/api/v1/posts", get(admin::api_posts))
        .route("/api/posts", get(admin::api_posts)) // legacy alias, drop after cutover
        // Intake body cap lives on the ROUTES, not the handler header
        // check — chunked/CL-less bodies bypass a Content-Length peek
        // (audit L1). DefaultBodyLimit is a tower layer; 413s before the
        // extractor reads the body.
        .route("/intake/form/{slug}", post(admin::intake_form).layer(axum::extract::DefaultBodyLimit::max(36 * 1024)))
        .route("/intake/comment", post(admin::intake_comment).layer(axum::extract::DefaultBodyLimit::max(36 * 1024)))
        .route("/intake/review", post(admin::intake_review).layer(axum::extract::DefaultBodyLimit::max(36 * 1024)))
        .route("/form/{slug}", post(admin::intake_form).layer(axum::extract::DefaultBodyLimit::max(36 * 1024)))
        .route("/comment", post(admin::intake_comment).layer(axum::extract::DefaultBodyLimit::max(36 * 1024)))
        .route("/review", post(admin::intake_review).layer(axum::extract::DefaultBodyLimit::max(36 * 1024)))
        .route("/admin/contact", get(contact::forms_page))
        .route("/admin/contact/form/{slug}", get(contact::form_detail))
        .route("/admin/contact/form/{slug}/edit", get(forms_editor::form_edit))
        .route("/admin/contact/form/{slug}/save", post(forms_editor::form_save))
        .route("/admin/contact/new", get(forms_editor::form_new))
        .route("/admin/contact/create", post(forms_editor::form_create))
        .route("/admin/contact/form/delete", post(contact::form_delete))
        .route("/admin/reviews", get(contact::reviews_page))
        .route("/admin/contact/inbox", get(contact::inbox_page))
        .route("/admin/contact/inbox/export", get(contact::inbox_export))
        .route("/admin/contact/inbox/action", post(contact::inbox_action))
        .route("/admin/contact/inbox/bulk", post(contact::inbox_bulk))
        .route("/admin/contact/inbox/{id}", get(contact::detail_page))
        .route("/admin/comments", get(comments::comments_page))
        .route("/admin/comments/action", post(comments::comment_action))
        .route("/admin/comments/bulk", post(comments::comments_bulk))
        .route("/admin/comments/empty", post(comments::comments_empty))
        .route("/admin/comments/reply/{id}", get(comments::reply_page))
        .route("/admin/comments/reply", post(comments::reply_save))
        .route("/admin/comments/edit/{id}", get(comments::edit_page))
        .route("/admin/comments/save", post(comments::edit_save))
        .route("/admin/comments/quickedit", post(comments::quick_edit))
        .route("/admin/comments/screen", post(comments::screen_options))
        .route("/admin/users", get(users::users_page))
        .route("/admin/user-new", get(users::user_new_page))
        .route("/admin/users/create", post(users::users_create))
        .route("/admin/user-edit/{id}", get(users::user_edit_page))
        .route("/admin/users/save", post(users::users_save))
        .route("/admin/users/delete", get(users::users_delete_page).post(users::users_delete))
        .route("/admin/users/bulk", post(users::users_bulk))
        .route("/admin/users/logout-others", post(users::users_logout_others))
        .route("/admin/users/app-pass/add", post(users::users_app_pass_add))
        .route("/admin/users/app-pass/revoke", post(users::users_app_pass_revoke))
        .route("/admin/profile", get(users::profile_page))
        .route("/admin/profile/2fa", get(users::profile_2fa_page))
        .route("/admin/users/2fa/setup", post(users::totp_setup))
        .route("/admin/users/2fa/confirm", post(users::totp_confirm))
        .route("/admin/users/2fa/disable", post(users::totp_disable))
        .route("/admin/users/2fa/regen", post(users::totp_regen))
        .route("/admin/settings/{section}", get(settings::settings_page).post(settings::settings_save))
        // Public site: generated file under out/ wins, else render the slug
        // on demand (mirrors `rpress render slug`), else out/404.html.
        .route("/{*public}", get(serve::public_page))
        // Multipart uploads cap at the shared media upload limit; the small
        // headroom leaves room for multipart framing over the file bytes.
        .layer(axum::extract::DefaultBodyLimit::max((crud::UPLOAD_LIMIT_BYTES + 1024 * 1024) as usize))
        .layer(axum::middleware::from_fn_with_state(state.clone(), clear_dead_session_cookie))
        // Outermost layer: the multi-site Host gate runs before anything
        // that touches a site context.
        .layer(axum::middleware::from_fn_with_state(state.clone(), serve::host_known))
        .with_state(state.clone());
    // Per-minute background tick: bridge TTL reaper + publish_due.
    // publish_due used to piggyback on get_sess — a site with no admin
    // traffic never published scheduled posts; the tick has no such
    // dependency (60s is inside WP's minute-level schedule semantics).
    {
        let sites = state.sites.clone();
        let app_tick = state.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                for sc in sites.values() {
                    serve::publish_due(sc);
                    for k in bridge::reap(&sc.bridges, bridge::TTL_IDLE) {
                        eprintln!("bridge: idle kill {k}");
                    }
                }
                sweep_sessions(&app_tick);
            }
        });
    }
    let listener = tokio::net::TcpListener::bind(bind).await?;
    eprintln!("rpress listening on http://{bind}");
    // Trusted-proxy hygiene: when we bind loopback with no proxy configured
    // but the site is a production URL, anything that stamps a client IP
    // (rate limits, session IP, app-pass last_ip) will record the proxy's
    // loopback address forever — warn once so ops notices the missing
    // RPRESS_TRUSTED_PROXY.
    {
        let su = state.sites.values().next()
            .map(|sc| crate::db::setting_value(&sc.db, "site_url", ""))
            .unwrap_or_default();
        let localhosty = su.is_empty() || su == "/" || su.contains("localhost") || su.contains("127.0.0.1") || su.contains("[::1]");
        if (bind.ip().is_loopback() || bind.ip().is_unspecified()) && TRUSTED_PROXIES.is_empty() && !localhosty {
            eprintln!(
                "rpress: WARNING — bound to loopback and RPRESS_TRUSTED_PROXY is unset, \
                 but site_url is {su}: X-Forwarded-For is ignored and every client IP \
                 (rate limits, session stamps, app-pass last_ip) records the proxy's \
                 loopback address. Set RPRESS_TRUSTED_PROXY=<proxy ip>."
            );
        }
    }
    let sites_down = state.sites.clone();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async move {
            // Ctrl-C or SIGTERM (deploy/systemd) — either must kill every
            // spawned bridge before exit or each restart leaks ~900MB
            // children (the idle reaper only covers in-process timeouts).
            #[cfg(unix)]
            {
                let mut term = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                    Ok(s) => s,
                    Err(_) => { let _ = tokio::signal::ctrl_c().await; return reap_all(&sites_down); }
                };
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = term.recv() => {}
                }
            }
            #[cfg(not(unix))]
            let _ = tokio::signal::ctrl_c().await;
            reap_all(&sites_down);
        })
        .await?;
    Ok(())
}

fn reap_all(sites_down: &Arc<HashMap<String, SiteCtx>>) {
    for sc in sites_down.values() {
        for k in bridge::reap(&sc.bridges, Duration::ZERO) {
            eprintln!("bridge: shutdown kill {k}");
        }
    }
}

/// Resolve the per-site context from the Host header.
/// Unknown hosts fall back to the configured default site — reachable only
/// single-site; multi-site requests pass the `host_known` gate first.
pub(crate) fn site_for<'a>(app: &'a App, headers: &HeaderMap) -> &'a SiteCtx {
    match serve::request_host(headers).as_deref().and_then(|h| app.sites.get(h)) {
        Some(sc) => sc,
        None => &app.sites[&app.default_site],
    }
}

/// Proxy addresses allowed to set X-Forwarded-For. `RPRESS_TRUSTED_PROXY` is a
/// comma-separated IP list (e.g. "127.0.0.1,10.0.0.1"); empty = trust none —
/// spoofed XFF can never bypass rate limits (A3).
static TRUSTED_PROXIES: std::sync::LazyLock<Vec<IpAddr>> = std::sync::LazyLock::new(|| {
    std::env::var("RPRESS_TRUSTED_PROXY")
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
});

/// Client IP for rate limiting and session stamping: the peer socket addr,
/// or the first X-Forwarded-For hop *only* when the peer is a trusted proxy.
pub(crate) fn client_ip(headers: &HeaderMap, peer: Option<SocketAddr>) -> String {
    let peer_ip = peer.map(|p| p.ip());
    if peer_ip.is_some_and(|ip| TRUSTED_PROXIES.contains(&ip)) {
        if let Some(xff) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            return xff;
        }
    }
    peer_ip.map(|ip| ip.to_string()).unwrap_or_else(|| "unknown".into())
}

fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<String> {
    // `Cookie:` may arrive on multiple header lines; `get` only sees the first.
    for value in headers.get_all(header::COOKIE).iter() {
        let Ok(raw) = value.to_str() else { continue };
        for part in raw.split(';') {
            let mut kv = part.trim().splitn(2, '=');
            if kv.next()? == name {
                return kv.next().map(|s| s.to_string());
            }
        }
    }
    None
}

/// All values sent under one cookie name — browsers may carry stale
/// duplicates (different Path/Secure histories), and header order puts the
/// dead one first often enough to log a valid user out. Try each until a
/// live session resolves.
fn cookie_all<'a>(headers: &'a HeaderMap, name: &str) -> Vec<String> {
    let mut out = Vec::new();
    // `Cookie:` may arrive on multiple header lines (some agents split);
    // `get` only sees the first.
    for value in headers.get_all(header::COOKIE).iter() {
        let Ok(raw) = value.to_str() else { continue };
        for part in raw.split(';') {
            let mut kv = part.trim().splitn(2, '=');
            if kv.next() == Some(name) {
                if let Some(v) = kv.next() {
                    if !v.is_empty() && !out.iter().any(|x| x == v) {
                        out.push(v.to_string());
                    }
                }
            }
        }
    }
    out
}

pub(crate) fn get_sess<'a>(app: &'a App, sc: &'a SiteCtx, headers: &HeaderMap) -> Option<Sess> {
    // WP `wp_schedule_publish` equivalent lives on the serve() minute tick
    // (see above): auth must not be required for scheduled posts to go live.
    //
    // Lock discipline (audit P2): the sessions mutex guards MAP OPS ONLY.
    // SQLite calls (session_get/set_csrf/touch — each a fresh conn open +
    // 6 PRAGMAs + query, ~100-500µs) run outside the lock or every admin
    // request serializes behind another request's disk I/O.
    for tok in cookie_all(headers, SESSION_COOKIE) {
        // Rehydrate from the `sessions` table on a map miss — cookie logins
        // now survive a restart (sessions are persisted, decision B).
        if !app.sessions.lock().contains_key(&tok) {
            if let Ok(Some((_t, user_id, email, role, csrf))) = crud::users::session_get(&sc.db, &tok) {
                // Persisted csrf survives restarts so a mid-edit form still
                // submits (M20). '' = row written before the column existed:
                // mint once, persist it, keep it stable from then on.
                let csrf = if csrf.is_empty() {
                    let mut b = [0u8; 16];
                    rand::thread_rng().fill_bytes(&mut b);
                    let v = hex(&b);
                    if let Err(e) = crud::users::session_set_csrf(&sc.db, &tok, &v) {
                        eprintln!("session_set_csrf persist failed for rehydrated token: {e}");
                    }
                    v
                } else {
                    csrf
                };
                // or_insert: a concurrent task may have rehydrated the same
                // token while we were doing DB I/O — keep the existing
                // entry (identical data), don't clobber.
                app.sessions.lock().entry(tok.clone()).or_insert(Sess {
                    site: sc.slug.clone(),
                    site_name: sc.name.clone(),
                    user_id,
                    email,
                    role,
                    db: sc.db.clone(),
                    csrf,
                    exp: Instant::now() + Duration::from_secs(12 * 3600),
                    touched: Instant::now(),
                });
            }
        }
        let (valid, stale) = {
            let map = app.sessions.lock();
            match map.get(&tok) {
                Some(s) => (
                    s.exp > Instant::now() && s.site == sc.slug,
                    s.touched.elapsed() > Duration::from_secs(300),
                ),
                None => (false, false),
            }
        };
        if !valid {
            app.sessions.lock().remove(&tok);
            continue;
        }
        // Rolling expiry, matching session_touch's 12h bump. The DB write
        // runs unlocked; touched is stamped after so a crashed touch can't
        // starve the row (worst case: one redundant UPDATE next request).
        if stale {
            let _ = crud::users::session_touch(&sc.db, &tok);
            if let Some(s) = app.sessions.lock().get_mut(&tok) {
                s.touched = Instant::now();
            }
        }
        let map = app.sessions.lock();
        let s = map.get(&tok)?;
        return Some(Sess {
            site: s.site.clone(),
            site_name: s.site_name.clone(),
            user_id: s.user_id,
            email: s.email.clone(),
            role: s.role.clone(),
            db: sc.db.clone(),
            csrf: s.csrf.clone(),
            exp: s.exp,
            touched: s.touched,
        });
    }
    None
}

/// Minute-tick sweep (audit M-sessions): the in-memory map only evicted
/// expired entries when their token was re-presented, so dead sessions
/// grew forever. Called from the serve() tick alongside publish_due.
/// The table purge is bounded DELETE on already-expired rows.
pub(crate) fn sweep_sessions(app: &App) {
    let now = Instant::now();
    app.sessions.lock().retain(|_, s| s.exp > now);
    for sc in app.sites.values() {
        if let Ok(c) = crate::db::conn(&sc.db) {
            let _ = c.execute(sql!("sessions__delete_expired"), []);
        }
    }
}


/// Public origins that may POST intake forms for this site:
/// the configured `site_url` plus its `edit.` sibling.
pub(crate) fn allowed_origins(sc: &SiteCtx) -> Vec<String> {
    let base = site_url(sc).trim_end_matches('/').to_string();
    let mut out = vec![base.clone()];
    if let Some(rest) = base.strip_prefix("https://") {
        out.push(format!("https://edit.{rest}"));
    }
    out
}

pub(crate) fn site_url(sc: &SiteCtx) -> String {
    sc.setting("site_url", "/")
}

/// Origin for assets we don't hold locally (CSS/JS/fonts/uploads) —
/// `settings.asset_origin`, falling back to `site_url`. Keeps a local
/// `rpress serve` preview fully styled while the files still live on the WP box.
pub(crate) fn asset_origin(sc: &SiteCtx) -> String {
    let v = sc.setting("asset_origin", "");
    if v.is_empty() { site_url(sc) } else { v }
}

/// Middleware: when a request carried an `rpress_session` cookie whose token no
/// longer resolves (expired/deleted/revoked) and the handler bounced to
/// `/admin/login`, append `Set-Cookie: rpress_session=` so the browser stops
/// resending the dead token. Covers the many handlers that replace
/// `need_auth`'s `Err(Response)` with their own login redirect.
async fn clear_dead_session_cookie(
    State(app): State<App>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let tok = cookie(req.headers(), SESSION_COOKIE);
    let mut res = next.run(req).await;
    if let Some(tok) = tok {
        let dead = !app.sessions.lock().contains_key(&tok);
        let bounced = res.headers().get(header::LOCATION).is_some_and(|v| v == "/admin/login");
        if dead && bounced {
            res.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_static(CLEAR_SESSION_COOKIE),
            );
        }
    }
    res
}

pub(crate) fn need_auth<'a>(app: &'a App, headers: &HeaderMap) -> Result<(Sess, &'a SiteCtx), Response> {
    need_auth_peer(app, headers, None)
}

/// `need_auth` + the real peer socket addr — used by handlers that extract
/// `ConnectInfo` so app-password `last_ip` records an address, not "unknown".
pub(crate) fn need_auth_peer<'a>(app: &'a App, headers: &HeaderMap, peer: Option<SocketAddr>) -> Result<(Sess, &'a SiteCtx), Response> {
    let sc = site_for(app, headers);
    if let Some(s) = get_sess(app, sc, headers) {
        return Ok((s, sc));
    }
    // WP Application Passwords: `Authorization: Basic base64(email:app_pass)`
    // authenticates without a session — used by `/api/v1/*` clients.
    if let Some(s) = auth::app_pass_sess(sc, headers, peer) {
        return Ok((s, sc));
    }
    // Stale/expired cookie → clear it so the browser stops resending a dead
    // token (get_sess already dropped it from the map; the DB row is expired
    // or deleted and session_get refuses it).
    if cookie(headers, SESSION_COOKIE).is_some() {
        let mut r = Redirect::to("/admin/login").into_response();
        r.headers_mut().insert(
            header::SET_COOKIE,
            HeaderValue::from_static(CLEAR_SESSION_COOKIE),
        );
        return Err(r);
    }
    Err(Redirect::to("/admin/login").into_response())
}
