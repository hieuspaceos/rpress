use anyhow::Result;
use crate::{auth as crate_auth, db as crud, generate, intake, schema};
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use parking_lot::Mutex;
use rand::RngCore;
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod admin;
mod auth;
mod gdlr;
pub(crate) mod gdlr_pb;
pub(crate) mod plugins;
mod posts;
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

#[derive(Clone)]
pub(crate) struct SiteCtx {
    pub(crate) slug: String,
    pub(crate) name: String,
    /// URL segment for the tour CPT (e.g. "voyage").
    pub(crate) tour_base: String,
    pub(crate) db: PathBuf,
    pub(crate) templates: PathBuf,
    pub(crate) out: PathBuf,
    /// Persistent data dir: user uploads + vendored plugin assets.
    /// Unlike `out` (disposable cache), `www` must be backed up.
    pub(crate) www: PathBuf,
    pub(crate) generating: Arc<AtomicBool>,
}

#[derive(Clone)]
pub(crate) struct App {
    pub(crate) sites: Arc<HashMap<String, SiteCtx>>,
    pub(crate) default_site: String,
    pub(crate) sessions: Arc<Mutex<HashMap<String, Sess>>>,
    pub(crate) login_hits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    pub(crate) intake_rate: Arc<Mutex<intake::Rate>>,
    /// Separate bucket for miss-render requests so bot scans can't burn
    /// the form-intake rate (or vice versa). Same in-memory semantics.
    pub(crate) miss_rate: Arc<Mutex<intake::Rate>>,
}

pub(crate) const SESSION_COOKIE: &str = "rpress_session";
pub(crate) const CLEAR_SESSION_COOKIE: &str = "rpress_session=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0";


pub(crate) struct Sess {
    pub(crate) site: String,
    pub(crate) site_name: String,
    pub(crate) user_id: i64,
    pub(crate) email: String,
    pub(crate) role: String,
    pub(crate) db: PathBuf,
    pub(crate) csrf: String,
    pub(crate) exp: Instant,
}

/// `sites`: `(host_header, db, templates, out, www)` — multi-site support.
/// `out` = disposable HTML cache; `www` = persistent uploads/plugins/assets.
pub async fn serve(db: PathBuf, templates: PathBuf, out: PathBuf, www: PathBuf, bind: SocketAddr, site_slug: String, host: String, name: String, tour_base: String) -> Result<()> {
    let _ = schema::migrate(&db);
    let _ = crud::ensure_media_cols(&db);
    let uploads_root = www.join("wp-content/uploads");
    let _ = crud::media_backfill_dates(&db, &uploads_root);
    let _ = crud::promote_tours(&db);
    // Boot duties that used to live inside `rpress generate`:
    //   1. seed default widgets/sidebars so a fresh DB isn't empty
    //   2. purge 30-day-old trash (WP _wp_scheduled_delete equivalent)
    //   3. emit the utility files (sitemap/robots/feeds/404/search/
    //      redirects.map) — cheap SQL→file, no renders
    if let Ok(boot_conn) = crate::db::conn(&db) {
        let _ = crate::widgets::ensure(&boot_conn);
    }
    crate::sitefx::purge_duties(&db, &out);
    {
        let site_url_boot = crate::db::setting_value(&db, "site_url", "/");
        if let Err(e) = crate::sitefx::emit_site_files(&db, &templates, &out, &site_url_boot) {
            eprintln!("emit_site_files: {e:#}");
        }
    }
    let site = SiteCtx {
        slug: site_slug.clone(),
        name: name.clone(),
        tour_base,
        db,
        templates,
        out,
        www,
        generating: Arc::new(AtomicBool::new(false)),
    };
    let mut sites = HashMap::new();
    sites.insert(host.clone(), site);
    let state = App {
        sites: Arc::new(sites),
        default_site: host,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        login_hits: Arc::new(Mutex::new(HashMap::new())),
        intake_rate: Arc::new(Mutex::new(intake::Rate::default())),
        miss_rate: Arc::new(Mutex::new(intake::Rate::default())),
    };
    let app = Router::new()
        .route("/", get(public_root))
        .route("/admin/login", get(auth::login_page))
        .route("/health", get(health))
        .route("/api/login", post(auth::login))
        .route("/api/logout", post(auth::logout))

        .route("/admin", get(auth::dashboard).post(auth::screen_options_save))
        .route("/admin/draft", post(auth::save_draft))
        .route("/admin/health", get(auth::health_page))
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
        .route("/admin/tax/new-filter", get(tax::tax_new_filter_page).post(tax::tax_new_filter))
        .route("/admin/tax/{tax}", get(tax::tax_page))
        .route("/admin/tax/save", post(tax::tax_save))
        .route("/admin/tax/edit/{id}", get(tax::tax_edit))
        .route("/admin/tax/delete", post(tax::tax_delete))
        .route("/admin/tax/bulk", post(tax::tax_bulk))
        .route("/admin/media", get(admin::media_page))
        .route("/admin/media/new", get(admin::media_new_page))
        .route("/admin/media/upload", post(admin::media_upload))
        .route("/admin/media/edit/{id}", get(admin::media_edit_page))
        .route("/admin/media/save", post(admin::media_save))
        .route("/admin/media/delete", post(admin::media_delete))
        .route("/admin/media/detach", post(admin::media_detach))
        .route("/admin/api/tour/save-option", post(posts::save_tour_option))
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
        .route("/admin/wp/{*path}", get(wp_asset))
        .route("/wp-content/uploads/{*path}", get(uploads))
        .route("/admin/front-css/{*path}", get(front_css))
        .route("/admin/vendor/htmx.min.js", get(admin::htmx))
        .route("/admin/{section}", get(posts::section))
        .route("/api/v1/gdlr/media", get(gdlr::media_list))
        .route("/admin/api/gdlr/ajax", post(gdlr_pb::ajax))
        .route("/api/generate", post(admin::api_generate))
        .route("/api/generate/status", get(admin::generate_status))
        .route("/api/v1/posts", get(admin::api_posts))
        .route("/api/posts", get(admin::api_posts)) // legacy alias, drop after cutover
        .route("/intake/form/{slug}", post(admin::intake_form))
        .route("/intake/comment", post(admin::intake_comment))
        .route("/intake/review", post(admin::intake_review))
        .route("/form/{slug}", post(admin::intake_form))
        .route("/comment", post(admin::intake_comment))
        .route("/review", post(admin::intake_review))
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
        .route("/admin/settings/{section}", get(settings::settings_page).post(settings::settings_save))
        // Public site: generated file under out/ wins, else render the slug
        // on demand (mirrors `rpress render slug`), else out/404.html.
        .route("/{*public}", get(public_page))
        // Multipart uploads cap at the shared media upload limit; the small
        // headroom leaves room for multipart framing over the file bytes.
        .layer(axum::extract::DefaultBodyLimit::max((crud::UPLOAD_LIMIT_BYTES + 1024 * 1024) as usize))
        .layer(axum::middleware::from_fn_with_state(state.clone(), clear_dead_session_cookie))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    eprintln!("rpress listening on http://{bind}");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

/// `GET /` — public front page: `out/index.html` when present, else a live
/// render of the `page_on_front` post — write-back so the homepage warms
/// itself instead of rendering on every cold hit.
async fn public_root(State(app): State<App>, headers: HeaderMap) -> Response {
    let sc = site_for(&app, &headers);
    if let Some(b) = read_out(sc, "index.html") {
        return html_response(b);
    }
    let Ok(conn) = crate::db::conn(&sc.db) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match crate::render::front_ids(&conn).0 {
        Some(id) => match crate::render::render_slug_by_id(&conn, &sc.db, &sc.templates, id, &site_url(sc)) {
            Ok(h) => {
                write_cache(sc, "index.html", &h);
                html_response(h)
            }
            Err(_) => not_found(sc),
        },
        None => not_found(sc),
    }
}

/// `GET /{*public}` — lazy write-back cache.
///
/// 1. `out/{path}.html` → `out/{path}/index.html` → `out/{path}` verbatim
///    (per-extension mime + nosniff).
/// 2. A path carrying a file extension that misses is a missing asset →
///    asset-origin redirect or 404 (never rendered).
/// 3. Extension-less paths normalize (`es/` → lang, `{tour_base}/` →
///    kind=tour) then `render_slug` — publish results write back into the
///    cache so the next hit is a file read; drafts never cache.
/// 4. Junk prefixes (`/wp-*`, `xmlrpc`, `.env` style) die before the render
///    path; misses are rate-limited per-IP to cap render cost.
async fn public_page(State(app): State<App>, headers: HeaderMap, ConnectInfo(peer): ConnectInfo<SocketAddr>, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    let clean = path.trim_matches('/');
    if clean.contains("..") || clean.starts_with('/') || clean.contains('\\') {
        return not_found(sc);
    }
    // .md → the markdown sibling; a .html suffix maps the same way so a
    // permalink URL + ".md" resolves the post (WP-style content negotiation).
    let (mut md_path, mut html_path): (Option<String>, Option<String>) = (None, None);
    let clean_owned;
    if let Some(stripped) = clean.strip_suffix(".md") {
        clean_owned = stripped.trim_end_matches('/').to_string();
        md_path = Some(format!("{clean_owned}/index.md"));
    } else if let Some(stripped) = clean.strip_suffix(".html") {
        clean_owned = stripped.trim_end_matches('/').to_string();
        html_path = Some(format!("{clean_owned}.html"));
    } else {
        clean_owned = clean.to_string();
    }
    let clean = clean_owned.as_str();
    let mut candidates: Vec<String> = Vec::new();
    if let Some(p) = &md_path { candidates.push(p.clone()); }
    // Accept negotiation: a crawler asking for markdown on the canonical
    // HTML URL gets the .md sibling — before the HTML candidates so it
    // isn't shadowed by a cached index.html.
    let wants_md = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|a| a.contains("text/markdown"))
        .unwrap_or(false);
    if wants_md && md_path.is_none() {
        candidates.push(format!("{clean}/index.md"));
    }
    if let Some(p) = &html_path { candidates.push(p.clone()); }
    candidates.push(format!("{clean}.html"));
    candidates.push(format!("{clean}/index.html"));
    // index.md only for explicit .md URL or the Accept-negotiated push
    // above — as a default candidate it would shadow the on-demand HTML
    // render whenever the md is cached.
    if md_path.is_some() { candidates.push(format!("{clean}/index.md")); }
    candidates.push(format!("{clean}/index.xml"));
    candidates.push(clean.to_string());
    for cand in candidates {
        if let Some(b) = read_out(sc, &cand) {
            let mime = if cand.ends_with(".md") { "text/markdown; charset=utf-8".to_string() } else { public_mime(&cand).to_string() };
            return ([(header::CONTENT_TYPE, mime), (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string())], b)
                .into_response();
        }
    }
    // ".md" miss → generate the markdown for this slug on demand (post only;
    // archives/front return 404). Write-back so the next hit is static.
    if md_path.is_some() {
        if let Some(md) = md_for_slug(sc, clean) {
            write_cache(sc, &format!("{clean}/index.md"), &md);
            return ([(header::CONTENT_TYPE, "text/markdown; charset=utf-8".to_string()),
                     (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string())], md).into_response();
        }
        return not_found(sc);
    }
    // A path carrying a file extension that didn't resolve above is a missing
    // asset, not a permalink — redirect to the asset origin (WP box) when it
    // differs from this host, else 404.
    if PathBuf::from(clean).extension().is_some() {
        let origin = asset_origin(sc).trim_end_matches('/').to_string();
        if origin != site_url(sc).trim_end_matches('/') && origin.starts_with("http") {
            return Redirect::temporary(&format!("{origin}/{clean}")).into_response();
        }
        return not_found(sc);
    }
    // Junk-prefix block: WP-scanner paths that can never resolve to a post.
    // Runs before the rate limiter so real lookups still count, but these
    // die cheap either way.
    if is_junk_prefix(clean) {
        return not_found(sc);
    }
    // Rate-limit miss-renders: a render is ~140ms of SQLite+template work —
    // 60/min/IP caps the burn a scanner can cause without touching legit
    // traffic (real visitors almost always hit the cache path above).
    let ip = client_ip(&headers, Some(peer));
    {
        let mut rate = app.miss_rate.lock();
        if !rate.allow(&ip, 60, Duration::from_secs(60)) {
            return (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, "60")]).into_response();
        }
    }
    // URL normalization before the slug lookup:
    //   "es/{slug}"        → lang="es", lookup "{slug}"
    //   "{tour_base}/{sl}" → kind="tour", lookup "{sl}"
    // `clean` still drives the cache path so the write-back lands where the
    // next request will look.
    let (lang, kind, lookup): (&str, &str, &str) = {
        let (l, rest) = match clean.strip_prefix("es/") {
            Some(r) => ("es", r),
            None => ("", clean),
        };
        let tb = format!("{}/", sc.tour_base);
        let (k, sl) = match rest.strip_prefix(tb.as_str()) {
            Some(s) => ("tour", s),
            None => ("", rest),
        };
        (l, k, sl)
    };
    let base = site_url(sc);
    match crate::render::render_slug(&sc.db, &sc.templates, lookup, lang, kind, &base) {
        Ok((h, status)) => {
            if status == "publish" {
                write_cache(sc, &format!("{clean}/index.html"), &h);
            }
            html_response(h)
        }
        Err(_) => not_found(sc),
    }
}

/// `.md` lookup: `es/{slug}` → ES post, `{tour_base}/{slug}` → tour, else
/// post/page. Mirrors the normalization in `public_page`.
fn md_for_slug(sc: &SiteCtx, clean: &str) -> Option<String> {
    let (lang, kind, slug): (&str, &str, &str) = {
        let (l, rest) = match clean.strip_prefix("es/") {
            Some(r) => ("es", r),
            None => ("", clean),
        };
        let tb = format!("{}/", sc.tour_base);
        let (k, sl) = match rest.strip_prefix(tb.as_str()) {
            Some(s) => ("tour", s),
            None => ("", rest),
        };
        (l, k, sl)
    };
    let conn = crate::db::conn(&sc.db).ok()?;
    crate::render::post_markdown(&conn, slug.trim_matches('/'), lang, kind, &site_url(sc))
}

/// WP-scanner paths that can never be a permalink on this CMS — checked
/// only on extension-less misses, so real assets keep their route.
fn is_junk_prefix(clean: &str) -> bool {
    clean.starts_with("wp-admin")
        || clean.starts_with("wp-login")
        || clean.starts_with("xmlrpc")
        || clean.starts_with("wp-includes")
        || clean.starts_with("wp-config")
}

/// Atomic write into the cache dir: tmp+rename so a reader never catches a
/// half-written page. Failures are silent — the response still serves, the
/// next miss just renders again.
fn write_cache(sc: &SiteCtx, rel: &str, html: &str) {
    let dest = sc.out.join(rel);
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = dest.with_extension("tmp");
    if std::fs::write(&tmp, html).is_ok() {
        let _ = std::fs::rename(&tmp, &dest);
    }
}

pub fn safe_read_under(root: &std::path::Path, rel: &str) -> Option<Vec<u8>> {
    if rel.contains("..") || rel.starts_with('/') || rel.contains('\\') {
        return None;
    }
    let root = root.canonicalize().ok()?;
    let candidate = root.join(rel);
    let metadata = std::fs::symlink_metadata(&candidate).ok()?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return None;
    }
    let candidate = candidate.canonicalize().ok()?;
    if !candidate.starts_with(&root) {
        return None;
    }
    std::fs::read(candidate).ok()
}

fn read_out(sc: &SiteCtx, rel: &str) -> Option<Vec<u8>> {
    // `out` = disposable HTML cache. Persistent dirs (user uploads, vendored
    // plugin/theme assets, shared front assets) live under `www` — never
    // inside the cache.
    let base = if rel.starts_with("wp-content/plugins")
        || rel.starts_with("wp-content/themes")
        || matches!(rel, "front.css" | "nav.js" | "llms.txt")
    {
        &sc.www
    } else {
        &sc.out
    };
    safe_read_under(base, rel)
}

fn public_mime(rel: &str) -> &'static str {
    match PathBuf::from(rel).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "map" | "json" => "application/json",
        "jpg" | "jpeg" => "image/jpeg", "png" => "image/png", "gif" => "image/gif",
        "webp" => "image/webp", "avif" => "image/avif", "ico" => "image/x-icon",
        "mp4" => "video/mp4", "webm" => "video/webm", "pdf" => "application/pdf",
        "woff2" => "font/woff2", "woff" => "font/woff", "ttf" => "font/ttf",
        "otf" => "font/otf", "eot" => "application/vnd.ms-fontobject",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn html_response<B: Into<Body>>(b: B) -> Response {
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8"), (header::X_CONTENT_TYPE_OPTIONS, "nosniff")], b.into()).into_response()
}

fn not_found(sc: &SiteCtx) -> Response {
    match read_out(sc, "404.html") {
        Some(b) => (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/html; charset=utf-8"), (header::X_CONTENT_TYPE_OPTIONS, "nosniff")], b).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Resolve the per-site context from the Host header.
/// Unknown hosts fall back to the configured default site (single-site safe).
pub(crate) fn site_for<'a>(app: &'a App, headers: &HeaderMap) -> &'a SiteCtx {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(|h| h.split(':').next().unwrap_or(h).to_lowercase());
    match host.as_deref().and_then(|h| app.sites.get(h)) {
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
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        let mut kv = part.trim().splitn(2, '=');
        if kv.next()? == name {
            return kv.next().map(|s| s.to_string());
        }
    }
    None
}

pub(crate) fn get_sess<'a>(app: &'a App, sc: &'a SiteCtx, headers: &HeaderMap) -> Option<Sess> {
    // WP `wp_schedule_publish` equivalent: the static site has no cron, so
    // the due-scheduled check piggybacks on the auth path — one indexed
    // SELECT (+UPDATE only when something is due) per admin request (6.3).
    // Flipped posts regenerate their single output file off the request
    // thread so publish latency stays one request, not one full generate.
    publish_due(sc);
    let tok = cookie(headers, SESSION_COOKIE)?;
    let mut map = app.sessions.lock();
    // Rehydrate from the `sessions` table on a map miss — cookie logins now
    // survive a restart (sessions are persisted, decision B). On any hit,
    // live or rehydrated, `last_seen` is stamped below.
    if !map.contains_key(&tok) {
        if let Ok(Some((_t, user_id, email, role))) = crud::users::session_get(&sc.db, &tok) {
            let mut csrf = [0u8; 16];
            rand::thread_rng().fill_bytes(&mut csrf);
            map.insert(
                tok.clone(),
                Sess {
                    site: sc.slug.clone(),
                    site_name: sc.name.clone(),
                    user_id,
                    email,
                    role,
                    db: sc.db.clone(),
                    csrf: hex(&csrf),
                    exp: Instant::now() + Duration::from_secs(12 * 3600),
                },
            );
        }
    }
    let valid = match map.get(&tok) {
        Some(s) => s.exp > Instant::now() && s.site == sc.slug,
        None => return None,
    };
    if !valid {
        map.remove(&tok);
        return None;
    }
    // Rolling expiry, matching session_touch's 12h bump below — a live map
    // entry must not outlive its (persisted) inactivity window.
    let _ = crud::users::session_touch(&sc.db, &tok);
    let s = map.get_mut(&tok)?;
    s.exp = Instant::now() + Duration::from_secs(12 * 3600);
    Some(Sess {
        site: s.site.clone(),
        site_name: s.site_name.clone(),
        user_id: s.user_id,
        email: s.email.clone(),
        role: s.role.clone(),
        db: sc.db.clone(),
        csrf: s.csrf.clone(),
        exp: s.exp,
    })
}

/// Flip due 'future' posts to 'publish' and regenerate their files.
/// Runs inside `get_sess` (every admin request): the UPDATE is a no-op
/// when nothing is due; a flip spawns one background thread that writes
/// each post's output file via the single-post regen path.
fn publish_due(sc: &SiteCtx) {
    let Ok(flipped) = crud::publish_due(&sc.db) else { return };
    if flipped.is_empty() {
        return;
    }
    let ids: Vec<i64> = flipped.iter().map(|(id, ..)| *id).collect();
    // The phase-4 single-post regen hook runs this off the request thread.
    admin::regen_posts(sc, &ids);
}

pub(crate) fn html_page(s: &Sess, title: &str, body: &str, htmx: bool) -> Html<String> {
    html_admin(s, title, body, htmx, "index-php")
}

pub(crate) fn html_admin(s: &Sess, title: &str, body: &str, htmx: bool, screen: &str) -> Html<String> {
    let src = if htmx {
        r#"<script src="/admin/vendor/htmx.min.js"></script>"#
    } else {
        ""
    };
    Html(format!(
        r#"<!doctype html><html class="wp-toolbar" lang="en"><head><meta charset="utf-8"><title>{title}</title>
<link rel="stylesheet" href="/admin/wp/dashicons.min.css">
<link rel="stylesheet" href="/admin/wp/admin-bar.min.css">
<link rel="stylesheet" href="/admin/wp/common.min.css">
<link rel="stylesheet" href="/admin/wp/forms.min.css">
<link rel="stylesheet" href="/admin/wp/admin-menu.min.css">
<link rel="stylesheet" href="/admin/wp/dashboard.min.css">
<link rel="stylesheet" href="/admin/wp/list-tables.min.css">
<link rel="stylesheet" href="/admin/wp/edit.min.css">
<link rel="stylesheet" href="/admin/wp/editor.min.css">

<link rel="stylesheet" href="/admin/wp/media.min.css">
<link rel="stylesheet" href="/admin/wp/nav-menus.min.css">
<link rel="stylesheet" href="/admin/wp/l10n.min.css">
<link rel="stylesheet" href="/admin/wp/admin-schemes.min.css">
<link rel="stylesheet" href="/admin/wp/wp-tooltip.min.css">
<link rel="stylesheet" href="/admin/wp/buttons.min.css">
<link rel="stylesheet" href="/admin/wp/colors.min.css">
<link rel="stylesheet" href="/admin/wp/cms-overrides.css">
<script>var ajaxurl='/admin/'; var pagenow='{screen}';</script>
<script>var wp=window.wp||{{}};wp.i18n={{__:function(s){{return s;}},_x:function(s){{return s;}},sprintf:function(f){{var a=[].slice.call(arguments,1);var i=0;return f.replace(/%[sd]/g,function(){{return String(a[i++]);}});}}}};</script>
<script src="/admin/wp/jquery.min.js"></script>
<script src="/admin/wp/hoverintent-js.min.js"></script>
<script src="/admin/wp/common.min.js"></script>
<script src="/admin/wp/admin-bar.min.js"></script>
<script src="/admin/wp/svg-painter.js"></script>
{src}</head>
<body class="wp-admin wp-core-ui js {screen} auto-fold admin-bar {scheme} svg">{body}
{end}</body></html>"#,
        scheme = users::color_class(s),
        end = chrome_end(&s)
    ))
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
    crate::db::setting_value(&sc.db, "site_url", "/")
}

/// Origin for assets we don't hold locally (CSS/JS/fonts/uploads) —
/// `settings.asset_origin`, falling back to `site_url`. Keeps a local
/// `rpress serve` preview fully styled while the files still live on the WP box.
pub(crate) fn asset_origin(sc: &SiteCtx) -> String {
    let v = crate::db::setting_value(&sc.db, "asset_origin", "");
    if v.is_empty() { site_url(sc) } else { v }
}

/// Authenticate and resolve the site in one step.
/// Handlers get `Ok((sess, site_ctx))`; unauthenticated → login redirect.
/// CSRF guard for every mutating POST. `s` from `need_auth`.
pub(crate) fn check_csrf(s: &Sess, sent: &str) -> Result<(), Response> {
    if sent != s.csrf {
        return Err(StatusCode::FORBIDDEN.into_response());
    }
    Ok(())
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
    let sc = site_for(app, headers);
    if let Some(s) = get_sess(app, sc, headers) {
        return Ok((s, sc));
    }
    // WP Application Passwords: `Authorization: Basic base64(email:app_pass)`
    // authenticates without a session — used by `/api/v1/*` clients.
    if let Some(s) = auth::app_pass_sess(sc, headers) {
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

pub(crate) fn chrome(active: &str, s: &Sess) -> String {
    chrome_start(active, s)
}

pub(crate) fn menu_li<A: AsRef<str>, B: AsRef<str>>(
    id: &str,
    href: &str,
    label: &str,
    icon: &str,
    active: &str,
    sub: &[(A, B)],
) -> String {
    menu_li_badge(id, href, label, icon, active, sub, 0)
}

/// `menu_li` + WP pending-count bubble (`awaiting-mod`) when `badge > 0`.
pub(crate) fn menu_li_badge<A: AsRef<str>, B: AsRef<str>>(
    id: &str,
    href: &str,
    label: &str,
    icon: &str,
    active: &str,
    sub: &[(A, B)],
    badge: i64,
) -> String {
    let on = active == id;
    let cur = if on {
        "wp-has-submenu wp-has-current-submenu wp-menu-open menu-top current"
    } else if sub.is_empty() {
        "wp-not-current-submenu menu-top"
    } else {
        "wp-has-submenu wp-not-current-submenu menu-top"
    };
    let aclass = if on {
        "wp-has-current-submenu wp-menu-open current"
    } else if sub.is_empty() {
        ""
    } else {
        "wp-has-submenu"
    };
    let bubble = if badge > 0 {
        format!(" <span class=\"awaiting-mod count-{badge}\"><span class=\"pending-count\" aria-hidden=\"true\">{badge}</span><span class=\"comments-in-moderation-text screen-reader-text\">{badge} pending</span></span>")
    } else {
        String::new()
    };
    let mut out = format!(
        r#"<li id="menu-{id}" class="{cur}"><a href="{href}" class="menu-top {aclass}"><div class="wp-menu-image dashicons-before {icon}"><br></div><div class="wp-menu-name">{label}{bubble}</div></a>"#
    );
    if !sub.is_empty() {
        out.push_str(&format!(
            r#"<ul class="wp-submenu wp-submenu-wrap"><li class="wp-submenu-head" aria-hidden="true">{label}</li>"#
        ));
        for (i, (h, l)) in sub.iter().enumerate() {
            let first = if i == 0 { " wp-first-item" } else { "" };
            out.push_str(&format!(r#"<li class="{first}"><a href="{}">{}</a></li>"#, h.as_ref(), l.as_ref()));
        }
        out.push_str("</ul>");
    }
    out.push_str("</li>");
    out
}

pub(crate) fn chrome_start(active: &str, s: &Sess) -> String {
    let site_name = esc(&s.site_name);
    let email = esc(&s.email);
    let mut html = String::from(
        r###"<div id="wpwrap">
<div id="adminmenumain" role="navigation" aria-label="Main menu">
<a class="screen-reader-shortcut" href="#wpbody-content">Skip to main content</a>
<a class="screen-reader-shortcut" href="#wp-toolbar">Skip to toolbar</a>
<div id="adminmenuback"></div>
<div id="adminmenuwrap">
<ul id="adminmenu">
"###,
    );

    html.push_str(&menu_li("dashboard", "/admin", "Dashboard", "dashicons-dashboard", active, &[] as &[(&str, &str)]));
    html.push_str(&menu_li(
        "posts",
        "/admin/posts",
        "Posts",
        "dashicons-admin-post",
        active,
        &[("/admin/posts", "All Posts"), ("/admin/new/post", "Add Post"), ("/admin/tax/category", "Categories"), ("/admin/tax/post_tag", "Tags")],
    ));
    html.push_str(&menu_li(
        "media",
        "/admin/media",
        "Media",
        "dashicons-admin-media",
        active,
        &[("/admin/media", "Library"), ("/admin/media/new", "Add New")],
    ));
    html.push_str(&menu_li(
        "pages",
        "/admin/pages",
        "Pages",
        "dashicons-admin-page",
        active,
        &[("/admin/pages", "All Pages"), ("/admin/new/page", "Add Page")],
    ));
    let pending_comments = crud::comments::counts(&s.db).map(|c| c.pending).unwrap_or(0);
    html.push_str(&menu_li_badge(
        "comments",
        "/admin/comments",
        "Comments",
        "dashicons-admin-comments",
        active,
        &[] as &[(&str, &str)],
        pending_comments,
    ));
    html.push_str(&menu_li(
        "reviews",
        "/admin/reviews",
        "Site Reviews",
        "dashicons-star-filled",
        active,
        &[] as &[(&str, &str)],
    ));
    html.push_str(&menu_li(
        "tours",
        "/admin/tours",
        "Tour",
        "dashicons-palmtree",
        active,
        &tour_submenu(&s.db),
    ));
    html.push_str(&menu_li(
        "contact",
        "/admin/contact",
        "Contact",
        "dashicons-email",
        active,
        &[
            ("/admin/contact", "Forms"),
            ("/admin/contact/inbox", "Submissions"),
        ],
    ));
    // Appearance: Menus for every role; Widgets is `edit_theme_options`
    // parity → admin-only, matching the /admin/widgets* guards.
    let appearance_sub: Vec<(&str, &str)> = if s.role == "admin" {
        vec![("/admin/menus", "Menus"), ("/admin/widgets", "Widgets")]
    } else {
        vec![("/admin/menus", "Menus")]
    };
    html.push_str(&menu_li(
        "menus",
        "/admin/menus",
        "Appearance",
        "dashicons-admin-appearance",
        active,
        &appearance_sub,
    ));
    if s.role == "admin" {
        html.push_str(&menu_li(
            "users",
            "/admin/users",
            "Users",
            "dashicons-admin-users",
            active,
            &[("/admin/users", "All Users"), ("/admin/user-new", "Add User")],
        ));
    }
    if s.role == "admin" {
        html.push_str(&menu_li(
            "settings",
            "/admin/settings/general",
            "Settings",
            "dashicons-admin-settings",
            active,
            &[
                ("/admin/settings/general", "General"),
                ("/admin/settings/writing", "Writing"),
                ("/admin/settings/reading", "Reading"),
                ("/admin/settings/discussion", "Discussion"),
                ("/admin/settings/media", "Media"),
                ("/admin/settings/permalink", "Permalinks"),
                ("/admin/settings/privacy", "Privacy"),
            ],
        ));
    }

    html.push_str(
        r#"<li id="collapse-menu" class="hide-if-no-js"><button type="button" id="collapse-button" aria-label="Collapse Main menu" aria-expanded="true"><span class="collapse-button-icon" aria-hidden="true"></span><span class="collapse-button-label">Collapse Menu</span></button></li>
</ul>
</div>
</div>
<div id="wpcontent">
<div id="wpadminbar" class="nojq nojs" role="navigation">
<div class="quicklinks" id="wp-toolbar">
<ul id="wp-admin-bar-root-default" class="ab-top-menu">
<li id="wp-admin-bar-site-name" class="menupop"><a class="ab-item" href="/">"#,);
    html.push_str(&site_name);
    html.push_str(r#"</a>
<div class="ab-sub-wrapper"><ul class="ab-submenu"><li id="wp-admin-bar-view-site"><a class="ab-item" href="/">View Site</a></li></ul></div></li>
<li id="wp-admin-bar-new-content" class="menupop"><a class="ab-item" href="/admin/new/post">New</a>
<div class="ab-sub-wrapper"><ul class="ab-submenu">
<li><a class="ab-item" href="/admin/new/post">Post</a></li>
<li><a class="ab-item" href="/admin/new/page">Page</a></li>
<li><a class="ab-item" href="/admin/new/tour">Tour</a></li>
</ul></div></li>
</ul>
<ul id="wp-admin-bar-top-secondary" class="ab-top-secondary ab-top-menu">
<li id="wp-admin-bar-my-account" class="menupop"><a class="ab-item" href="/admin/profile">Howdy, "#,
    );
    html.push_str(&email);
    html.push_str(&format!(
        r#"</a>
<div class="ab-sub-wrapper"><ul class="ab-submenu"><li id="wp-admin-bar-logout"><form method="post" action="/api/logout" style="margin:0"><input type="hidden" name="csrf" value="{csrf}"><button type="submit" class="ab-item" style="background:none;border:0;cursor:pointer;width:100%;text-align:left">Log Out</button></form></li></ul></div></li>
</ul>
</div>
</div>
<div id="wpbody" role="main">
<div id="wpbody-content">
"#,
        csrf = esc(&s.csrf)
    ));
    html
}


pub(crate) fn chrome_end(s: &Sess) -> String {
    let site_name = esc(&s.site_name);
    format!(r#"<div class="clear"></div>
</div><!-- wpbody-content -->
<div class="clear"></div>
</div><!-- wpbody -->
<div class="clear"></div>
</div><!-- wpcontent -->
<div id="wpfooter" role="contentinfo">
<p id="footer-left" class="alignleft">{site_name}</p>
<p id="footer-upgrade" class="alignright">Version 7.1.3 chrome</p>
</div>
</div><!-- wpwrap -->"#)
}
/// Tour submenu mirrors tourmaster: All/Add Tour, Add New Filter, then every
/// non-post taxonomy present in `terms`/`post_terms` (custom tour taxs +
/// theme taxonomies like Activites/Pays).
fn tour_submenu(db: &std::path::Path) -> Vec<(String, String)> {
    let mut sub = vec![
        ("/admin/tours".into(), "All Tour".into()),
        ("/admin/new/tour".into(), "Add New Tour".into()),
        ("/admin/tax/new-filter".into(), "Add New Filter".into()),
    ];
    for t in crud::list_taxonomies(db).unwrap_or_default() {
        if t == "category" || t == "post_tag" {
            continue;
        }
        let (label, _) = tax_label(&t);
        sub.push((format!("/admin/tax/{t}"), label));
    }
    sub
}

/// Human label for a taxonomy (shared with tax.rs).
pub(crate) fn tax_label(tax: &str) -> (String, String) {
    match tax {
        "category" => ("Categories".into(), "Category".into()),
        "post_tag" => ("Tags".into(), "Tag".into()),
        "duree" => ("Durée".into(), "Durée".into()),
        "duracion" => ("Duración".into(), "Duración".into()),
        "region" | "region-es" => ("Región".into(), "Región".into()),
        "style" => ("Style".into(), "Style".into()),
        "estilo" => ("Estilo".into(), "Estilo".into()),
        "profil" => ("Profil".into(), "Profil".into()),
        "perfil" => ("Perfil".into(), "Perfil".into()),
        "activites" | "Activites" => ("Activités".into(), "Activité".into()),
        "pays" | "Pays" => ("Pays".into(), "Pays".into()),
        "tour-activity" => ("Tour Activity".into(), "Tour Activity".into()),
        "tour-age" => ("Tour Age".into(), "Tour Age".into()),
        "viet" => ("Viet".into(), "Viet".into()),
        _ => (tax.replace('-', " "), tax.replace('-', " ")),
    }
}


pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

async fn wp_asset(State(app): State<App>, headers: HeaderMap, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    let b = match safe_read_under(&sc.templates.join("admin/wp"), &path) {
        Some(b) => b,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let ct = match PathBuf::from(&path).extension().and_then(|s| s.to_str()).unwrap_or("") {
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    };
    ([(header::CONTENT_TYPE, ct)], b).into_response()
}

/// `/wp-content/uploads/*` — imported media lives under `www/` when copied
/// locally. Serve safe media inline; force scriptable legacy extensions to
/// download as octet-stream so copied WP uploads cannot become same-origin XSS.
/// Missing local files redirect to `asset_origin` for preview parity.
async fn uploads(State(app): State<App>, headers: HeaderMap, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    let root = sc.www.join("wp-content/uploads");
    if let Some(bytes) = safe_read_under(&root, &path) {
        let (mime, attachment) = crud::served_upload_mime(&path);
        let mut res = Response::builder()
            .header(header::CONTENT_TYPE, mime)
            .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
        if attachment {
            let name = path.rsplit('/').next().unwrap_or("download.bin").replace('"', "_");
            res = res.header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{name}\""),
            );
        }
        return res
            .body(Body::from(bytes))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }
    let origin = asset_origin(sc).trim_end_matches('/').to_string();
    if origin.starts_with("http") {
        Redirect::temporary(&format!("{origin}/wp-content/uploads/{path}")).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
pub(crate) fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// `/admin/front-css/*` — vendored theme/plugin frontend stylesheets, loaded
/// only inside the page-builder Live Mode canvas so it matches the public site.
async fn front_css(State(app): State<App>, headers: HeaderMap, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    match safe_read_under(&sc.templates.join("front-css"), &path) {
        Some(b) => ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], b).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

