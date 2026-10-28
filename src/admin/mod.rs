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
mod dashboard;
mod gdlr;
pub(crate) mod bridge;
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

/// Legacy single-string view of `SiteCtx::kind_bases`: the tour kind's
/// URL base (`""` when no adapter registers one). Boundary helper for
/// call sites that still take the pre-KindBases `tour_base: &str`.
pub(crate) fn tour_base(sc: &SiteCtx) -> &str {
    plugins::base_for(&sc.kind_bases, "tour").unwrap_or_default()
}


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
    purge_alias_caches(&db, &out);
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
        .route("/admin/media", get(admin::media_page))
        .route("/admin/media/new", get(admin::media_new_page))
        .route("/admin/media/upload", post(admin::media_upload))
        .route("/admin/media/edit/{id}", get(admin::media_edit_page))
        .route("/admin/media/save", post(admin::media_save))
        .route("/admin/media/delete", post(admin::media_delete))
        .route("/admin/media/detach", post(admin::media_detach))
        .route("/admin/plugins/{slug}/{*rest}", get(plugin_screen).post(plugin_screen))
        .route("/admin/api/tour/save-option", post(save_option_shim))
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
        .route("/admin/api/gdlr/ajax", post(gdlr_ajax))
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
        // Outermost layer: the multi-site Host gate runs before anything
        // that touches a site context.
        .layer(axum::middleware::from_fn_with_state(state.clone(), host_known))
        .with_state(state.clone());
    // Bridge TTL reaper: kill PHP runtimes idle > 10min. Per-site maps
    // live on SiteCtx; sweep them all every minute.
    {
        let sites = state.sites.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                for sc in sites.values() {
                    for k in bridge::reap(&sc.bridges, bridge::TTL_IDLE) {
                        eprintln!("bridge: idle kill {k}");
                    }
                }
            }
        });
    }
    let listener = tokio::net::TcpListener::bind(bind).await?;
    eprintln!("rpress listening on http://{bind}");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

/// One-time upgrade sweep (audit remediation, phase 1): pre-301 alias
/// cache files keep serving from the cache-hit path even though
/// `/fr/<slug>` now redirects — the file check runs before split_path, so
/// the redirect alone can't retire them. Deletes:
///   - `{out}/{primary_lang}/` — primary-prefix alias trees (nothing
///     legitimate renders under the primary prefix after the 301)
///   - every `{rel}.html` leaf that isn't `index.html`/`404.html` —
///     legacy flat write-backs that survive unpublish (regen only removes
///     `{slug}/index.html`). Loud, idempotent, cheap once swept.
fn purge_alias_caches(db: &PathBuf, out: &PathBuf) {
    let mut removed = 0usize;
    if let Ok(conn) = crate::db::conn(db) {
        let primary = crate::lang::config(&conn).primary;
        let dir = out.join(&primary);
        if !primary.is_empty() && dir.is_dir() && std::fs::remove_dir_all(&dir).is_ok() {
            removed += 1;
        }
    }
    fn sweep(dir: &PathBuf, removed: &mut usize) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for e in entries.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if p.is_dir() {
                sweep(&p, removed);
            } else if name.ends_with(".html") && name != "index.html" && name != "404.html" {
                if std::fs::remove_file(&p).is_ok() {
                    *removed += 1;
                }
            }
        }
    }
    sweep(out, &mut removed);
    if removed > 0 {
        eprintln!("alias-cache purge: removed {removed} stale cache artifact(s) under {}", out.display());
    }
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
    match crate::render::front_id(&conn) {
        Some(id) => match crate::render::render_slug_by_id_public(&conn, &sc.templates, id, &site_url(sc), tour_base(sc)) {
            // Publish-gated render: a draft page_on_front never reaches 200.
            Ok((h, status)) if status == "publish" => {
                write_cache(sc, "index.html", &h);
                html_response(h)
            }
            _ => not_found(sc),
        },
        None => not_found(sc),
    }
}

/// `GET /{*public}` — lazy write-back cache.
///
/// 1. `out/{path}.html` → `out/{path}/index.html` → `out/{path}` verbatim
///    (per-extension mime + nosniff).
/// 2. Primary-lang prefix (`/fr/…` on an fr-primary site) → 301 to the
///    canonical no-prefix URL; a resolved non-primary lang whose post is
///    missing → 301 to the translation-group canonical or 404.
/// 3. A path carrying a file extension that misses is a missing asset →
///    allowlisted asset-origin redirect or 404 (never rendered, junk and
///    dotfile prefixes die first).
/// 4. Extension-less paths normalize (`es/` → lang, `{kind_base}/` →
///    the kind owning that base) then `render_slug` — publish results
///    cache so the next hit is a file read; drafts never cache.
/// 5. Miss-renders are rate-limited per-IP to cap render cost.
///
/// Cache-write contract (phase 5 consumes this enumeration): every rel
/// this handler can leave under `out/` is
///   - `{clean}/index.html`  canonical page render
///   - `{clean}/index.md`    markdown sibling (on-demand .md/Accept)
///   - `{lang}/{clean}/index.html|index.md`  non-primary-lang aliases —
///    same two suffixes under a lang dir, since `clean` keeps the prefix
///   - `index.html`          front page (public_root)
/// Legacy `{clean}.html` flat aliases are read candidates but never
/// written; the serve() boot sweep deletes them.
async fn public_page(State(app): State<App>, headers: HeaderMap, ConnectInfo(peer): ConnectInfo<SocketAddr>, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    let clean = path.trim_matches('/');
    // Path charset allowlist: WP asset filenames legitimately carry
    // uppercase (ElegantIcons.woff/.svg, Font Awesome dirs, …), so the
    // gate can't be lowercase-only. Slug lookup lowercases anyway —
    // uppercase only matters for static-file resolution.
    if clean.contains("..") || clean.starts_with('/') || clean.contains('\\')
        || clean.bytes().any(|b| !matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'/' | b'_' | b'%' | b'.' | b'~' | b'-'))
    {
        return not_found(sc);
    }
    // ".html" permalink aliases are never canonical (M22): real static
    // .html files under the known static subtrees (.well-known ACME,
    // wp-includes, wp-content plugin assets) still serve verbatim; every
    // other .html request 301s to the bare path instead of serving a post
    // or cached alias under the .html URL. ".md" → the sibling below.
    let mut md_path: Option<String> = None;
    let clean_owned;
    if let Some(stripped) = clean.strip_suffix(".md") {
        clean_owned = stripped.trim_end_matches('/').to_string();
        md_path = Some(format!("{clean_owned}/index.md"));
    } else if let Some(stripped) = clean.strip_suffix(".html") {
        let bare = stripped.trim_end_matches('/');
        let static_subtree = bare.starts_with(".well-known/")
            || bare.starts_with("wp-includes/")
            || bare.starts_with("wp-content/");
        if static_subtree {
            let rel = format!("{bare}.html");
            if let Some(b) = read_out(sc, &rel) {
                let mime = public_mime(&rel).to_string();
                // `no-cache` (not `no-store`): browser may cache but must
                // revalidate before reuse — keeps rebuilt css/js fresh on
                // plain reloads; no header meant heuristic caching could
                // pin a stale stylesheet for days.
                return ([(header::CONTENT_TYPE, mime), (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()), (header::CACHE_CONTROL, "no-cache".to_string())], b)
                    .into_response();
            }
        }
        let target = if bare.is_empty() { "/".to_string() } else { format!("/{bare}/") };
        return moved_permanently(&target);
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
            return ([(header::CONTENT_TYPE, mime), (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()), (header::CACHE_CONTROL, "no-cache".to_string())], b)
                .into_response();
        }
    }
    // Cache missed — resolve what this URL is allowed to become before
    // paying for a render.
    let conn = match crate::db::conn(&sc.db) { Ok(c) => c, Err(_) => return not_found(sc) };
    let lang_cfg = crate::lang::config(&conn);
    // Primary-lang prefix (`/fr/<slug>` on an fr-primary site): duplicate of
    // the canonical URL — 301 before any lookup. Pre-existing `{out}/fr/`
    // alias files are swept at serve() boot, so this is only reached on a
    // genuine miss. The .md suffix (already stripped) rides along.
    if let Some(rest) = crate::lang::primary_alias(&lang_cfg, clean) {
        let suffix = if md_path.is_some() { ".md" } else { "" };
        return moved_permanently(&format!("/{rest}{suffix}"));
    }
    // Junk-prefix block: WP-scanner paths that can never resolve to a post.
    // Runs before the rate limiter so real lookups still count, but these
    // die cheap either way.
    if is_junk_prefix(clean) {
        return not_found(sc);
    }
    // WP pagination aliases `/…/page/{N}` (N all digits) 404ed while WP
    // rendered page 2+ of the archive — divergence documented in
    // phase-04: 301 to the archive root instead of rendering a page we
    // don't paginate. Bare `/page/N` → `/`.
    {
        let (rest, leaf) = clean.rsplit_once('/').unwrap_or(("", clean));
        if !leaf.is_empty() && leaf.bytes().all(|b| b.is_ascii_digit()) {
            if rest == "page" {
                return moved_permanently("/");
            }
            if let Some(root) = rest.strip_suffix("/page") {
                return moved_permanently(&format!("/{root}/"));
            }
        }
    }
    // ".md" miss → generate the markdown for this slug on demand (post only;
    // archives/front return 404). ".html" alias requests reach the render
    // below but ride the same canonical-rel gate — they 301 to the
    // canonical permalink instead of serving+caching under the alias (M22).
    if md_path.is_some() {
        if let Some((md_id, md)) = md_for_slug(sc, clean) {
            // Canonical-rel gate (M22): a .md request on a non-canonical
            // alias 301s to the canonical .md instead of writing a stale
            // alias copy.
            let canonical = crate::render::post_canonical_path(&conn, md_id, tour_base(sc));
            let req = format!("/{}/", clean.trim_matches('/'));
            if let Some(c) = canonical {
                if c != req {
                    return moved_permanently(&format!("{}.md", c.trim_end_matches('/')));
                }
            }
            write_cache(sc, &format!("{clean}/index.md"), &md);
            return ([(header::CONTENT_TYPE, "text/markdown; charset=utf-8".to_string()),
                     (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string())], md).into_response();
        }
        return not_found(sc);
    }
    // A path carrying a file extension that didn't resolve above is a missing
    // asset, not a permalink — only the real WP asset trees may bounce to the
    // (settings-controlled, allowlisted) asset origin; every other miss 404s.
    if PathBuf::from(clean).extension().is_some() {
        if redirect_prefix(clean) {
            if let Some(origin) = asset_redirect_target(&asset_origin(sc), &site_url(sc), &ASSET_ORIGIN_ALLOW) {
                return Redirect::temporary(&format!("{origin}/{clean}")).into_response();
            }
        }
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
    //   "{lang}/{slug}"     → lang from settings.languages, lookup "{slug}"
    //   "{kind_base}/{sl}"  → the kind owning that base, lookup "{sl}"
    // `clean` still drives the cache path so the write-back lands where the
    // next request will look.
    let (lang_s, rest) = crate::lang::split_path(&lang_cfg, clean);
    let (kind, lookup) = sc
        .kind_bases
        .iter()
        .filter(|(_, b)| !b.is_empty())
        .find_map(|(k, b)| rest.strip_prefix(format!("{b}/").as_str()).map(|s| (*k, s)))
        .unwrap_or(("", rest));
    let base = site_url(sc);
    // rusqlite + template render are synchronous — a cache-miss render is
    // ~140ms of work, too heavy to sit on a tokio worker (H14). Everything
    // the render needs is cloned up front; the cache write-back still runs
    // on the handler task (pure fs).
    let rendered = {
        let (db, tpl, lookup_s, lang_c, kind_c, base_c, tb_c) = (
            sc.db.clone(),
            sc.templates.clone(),
            lookup.to_string(),
            lang_s.clone(),
            kind.to_string(),
            base.clone(),
            tour_base(sc).to_string(),
        );
        tokio::task::spawn_blocking(move || {
            crate::render::render_slug(&db, &tpl, &lookup_s, &lang_c, &kind_c, &base_c, &tb_c)
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("render join: {e}")))
    };
    match rendered {
        Ok((h, status, canonical)) => {
            // Defense in depth: only publish ever caches — and only publish
            // ever serves, even if a render path slips a status through.
            if status != "publish" {
                return not_found(sc);
            }
            // Canonical-rel gate (M22): only the canonical permalink may
            // write into the cache. Alias renders (old slugs, /category/x,
            // unnested category paths, wrong-prefix term archives) 301 to
            // the canonical URL instead of leaving a second stale copy —
            // ".html" suffixes already normalized above.
            let req = format!("/{}/", clean.trim_matches('/'));
            if !canonical.is_empty() && canonical != req {
                return moved_permanently(&canonical);
            }
            write_cache(sc, &format!("{clean}/index.html"), &h);
            html_response(h)
        }
        Err(_) => {
            // The lang variant doesn't exist but a translation-group sibling
            // does → 301 to the canonical permalink, else 404 (Polylang).
            if let Some(target) = crate::render::translation_canonical(&conn, &lang_cfg, &lang_s, lookup, kind, tour_base(sc)) {
                return moved_permanently(&target);
            }
            not_found(sc)
        }
    }
}

/// `301 Moved Permanently` — axum's `Redirect::permanent` is 308, but the
/// canonical/prefix redirects want the SEO-canonical 301.
fn moved_permanently(target: &str) -> Response {
    (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, target)]).into_response()
}

/// `.md` lookup: `{lang}/{slug}` → translated post, `{kind_base}/{slug}`
/// → the kind owning that base, else post/page. Mirrors the normalization
/// in `public_page`. Returns (post id, markdown) — the id feeds the
/// canonical-rel gate.
fn md_for_slug(sc: &SiteCtx, clean: &str) -> Option<(i64, String)> {
    let conn = crate::db::conn(&sc.db).ok()?;
    let cfg = crate::lang::config(&conn);
    let (lang, rest) = crate::lang::split_path(&cfg, clean);
    let (kind, slug) = sc
        .kind_bases
        .iter()
        .filter(|(_, b)| !b.is_empty())
        .find_map(|(k, b)| rest.strip_prefix(format!("{b}/").as_str()).map(|s| (*k, s)))
        .unwrap_or(("", rest));
    crate::render::post_markdown_id(&conn, slug.trim_matches('/'), &lang, kind, &site_url(sc), tour_base(sc))
}

/// WP-scanner paths that can never be a permalink on this CMS — checked
/// only on extension-less misses (and before the asset-redirect gate), so
/// real assets keep their route. Dotfiles (`.env`, `.git/…`) are probes by
/// definition; `.well-known` stays servable for ACME-style challenges.
/// `wp-includes/` is deliberately NOT junk: it's a real WP asset tree that
/// may legitimately 307 to the origin under `redirect_prefix`.
pub(crate) fn is_junk_prefix(clean: &str) -> bool {
    (clean.starts_with('.') && !clean.starts_with(".well-known"))
        || clean.starts_with("wp-admin")
        || clean.starts_with("wp-login")
        || clean.starts_with("xmlrpc")
        || clean.starts_with("wp-config")
}

/// Only the real WP asset trees may bounce an extension-miss to the asset
/// origin — everything else 404s locally (a bare `/{slug}.php` probe must
/// never become an origin redirect).
pub(crate) fn redirect_prefix(clean: &str) -> bool {
    clean.starts_with("wp-content/")
        || clean.starts_with("wp-includes/")
        || clean.starts_with("uploads/")
}

/// Origins an asset-miss may 307 to, beyond the site's own URL host:
/// `RPRESS_ASSET_ORIGINS` = comma-separated hostnames (e.g. the WP box's
/// domain). Empty = only same-site origins — a settings-written
/// `asset_origin` pointing elsewhere can never turn a probe into an
/// open-redirect to an arbitrary host.
static ASSET_ORIGIN_ALLOW: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| {
    std::env::var("RPRESS_ASSET_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
});

/// Host part of an origin URL ("https://cdn.example.com:8443" → "cdn.example.com").
fn url_host(url: &str) -> String {
    let no_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let hostport = no_scheme.split('/').next().unwrap_or("");
    hostport.split(':').next().unwrap_or("").to_lowercase()
}

/// The origin an asset-miss redirect may target, or None to 404:
/// requires http(s), rejects the site's own URL (a self-redirect loops),
/// and requires the origin host to be the site's host or env-allowlisted.
pub(crate) fn asset_redirect_target(origin: &str, site_url: &str, allow: &[String]) -> Option<String> {
    let origin = origin.trim_end_matches('/');
    if !origin.starts_with("http") || origin == site_url.trim_end_matches('/') {
        return None;
    }
    let (o_host, s_host) = (url_host(origin), url_host(site_url));
    if o_host.is_empty() {
        return None;
    }
    if o_host == s_host && !s_host.is_empty() {
        return Some(origin.to_string());
    }
    allow.contains(&o_host).then(|| origin.to_string())
}

/// Atomic write into the cache dir: unique tmp + rename so a reader never
/// catches a half-written page and two racing misses can't clobber each
/// other's tmp (audit H6 — the old `{file}.tmp` name was deterministic:
/// the slower render's rename could resurrect older content, and orphans
/// were immune to bust_cache_dir). Write/rename failures only cost a
/// re-render on the next miss — logged, never fatal.
static CACHE_TMP_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn write_cache(sc: &SiteCtx, rel: &str, html: &str) {
    let dest = sc.out.join(rel);
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let n = CACHE_TMP_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = dest.with_extension(format!("{}.{}.tmp", std::process::id(), n));
    match std::fs::write(&tmp, html) {
        Ok(()) => {
            if let Err(e) = std::fs::rename(&tmp, &dest) {
                eprintln!("cache rename {} → {}: {e}", tmp.display(), dest.display());
                let _ = std::fs::remove_file(&tmp);
            }
        }
        Err(e) => eprintln!("cache write {}: {e}", tmp.display()),
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
        || rel.starts_with("wp-includes/")
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

/// Request Host header, normalized to a bare lowercase hostname.
fn request_host(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(|h| h.split(':').next().unwrap_or(h).to_lowercase())
}

/// Resolve the per-site context from the Host header.
/// Unknown hosts fall back to the configured default site — reachable only
/// single-site; multi-site requests pass the `host_known` gate first.
pub(crate) fn site_for<'a>(app: &'a App, headers: &HeaderMap) -> &'a SiteCtx {
    match request_host(headers).as_deref().and_then(|h| app.sites.get(h)) {
        Some(sc) => sc,
        None => &app.sites[&app.default_site],
    }
}

/// Multi-site gate: with more than one site configured there is no safe
/// default host, so a request for an unknown (or missing) Host gets 421
/// Misdirected Request instead of silently serving the default site's
/// content. Single-site keeps the fallback — dev requests hit localhost.
async fn host_known(
    State(app): State<App>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    if app.sites.len() > 1
        && !request_host(req.headers()).is_some_and(|h| app.sites.contains_key(&h))
    {
        return StatusCode::MISDIRECTED_REQUEST.into_response();
    }
    next.run(req).await
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
    let tok = cookie(headers, SESSION_COOKIE)?;
    // WP `wp_schedule_publish` equivalent: the static site has no cron, so
    // the due-scheduled check piggybacks on the auth path — after cookie
    // extraction so unauthed requests don't pay the write-check (6.3).
    // Flipped posts regenerate their single output file off the request
    // thread so publish latency stays one request, not one full generate.
    publish_due(sc);
    let mut map = app.sessions.lock();
    // Rehydrate from the `sessions` table on a map miss — cookie logins now
    // survive a restart (sessions are persisted, decision B). On any hit,
    // live or rehydrated, `last_seen` is stamped below.
    if !map.contains_key(&tok) {
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
            map.insert(
                tok.clone(),
                Sess {
                    site: sc.slug.clone(),
                    site_name: sc.name.clone(),
                    user_id,
                    email,
                    role,
                    db: sc.db.clone(),
                    csrf,
                    exp: Instant::now() + Duration::from_secs(12 * 3600),
                    touched: Instant::now(),
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
    // Rolling expiry, matching session_touch's 12h bump — a live map entry
    // must not outlive its inactivity window. The DB write only runs when
    // the session was last touched >5min ago (per-request UPDATE was L).
    let s = map.get_mut(&tok)?;
    if s.touched.elapsed() > Duration::from_secs(300) {
        let _ = crud::users::session_touch(&sc.db, &tok);
        s.touched = Instant::now();
    }
    Some(Sess {
        site: s.site.clone(),
        site_name: s.site_name.clone(),
        user_id: s.user_id,
        email: s.email.clone(),
        role: s.role.clone(),
        db: sc.db.clone(),
        csrf: s.csrf.clone(),
        exp: s.exp,
        touched: s.touched,
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
<link rel="stylesheet" href="/admin/wp/atrium.css">
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
/// CSRF guard for every mutating POST. `s` from `need_auth`. Constant-time
/// compare: a per-byte early-exit `==` leaks the prefix length via timing.
pub(crate) fn check_csrf(s: &Sess, sent: &str) -> Result<(), Response> {
    if !ct_eq(s.csrf.as_bytes(), sent.as_bytes()) {
        return Err(StatusCode::FORBIDDEN.into_response());
    }
    Ok(())
}

/// Constant-time byte equality — `check_csrf` and any token compare go
/// through this (the timing leak is small but free to close).
pub(crate) fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Per-site CSRF key for transient (non-persisted) sessions — app-password
/// Basic auth never writes a sessions row, so its csrf can't be a stored
/// random value; it's HMAC(server_key, user_id:app_pass_hash) instead, so a
/// client holding the app password can compute it but can't forge another
/// user's. Minted into `settings.server_key` on first use.
pub(crate) fn server_key(db: &std::path::Path) -> String {
    let key = "server_key";
    if let Ok(c) = crate::db::conn(db) {
        if let Ok(v) = c.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| r.get::<_, String>(0)) {
            if !v.is_empty() {
                return v;
            }
        }
        let mut b = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut b);
        let v = hex(&b);
        let _ = c.execute(
            "INSERT OR IGNORE INTO settings(key, value) VALUES(?1,?2)",
            rusqlite::params![key, v],
        );
        return v;
    }
    // No db → deterministic-only fallback keeps auth failing closed: csrf
    // can never match a form value without a real key behind it.
    String::new()
}

/// HMAC-SHA256(key, msg) → hex. Minimal RFC 2104 inline — no hmac dep.
pub(crate) fn hmac_sha256_hex(key: &str, msg: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut k = key.as_bytes().to_vec();
    if k.len() > 64 {
        k = Sha256::digest(&k).to_vec();
    }
    k.resize(64, 0);
    let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();
    let mut h = Sha256::new();
    h.update(&ipad);
    h.update(msg.as_bytes());
    let inner = h.finalize();
    let mut h = Sha256::new();
    h.update(&opad);
    h.update(inner);
    hex(&h.finalize())
}

/// Percent-encode a query/path value — byte-wise UTF-8 (the old per-char
/// `format!("%{:02X}", c as u32)` copies encoded code points, not bytes,
/// producing mojibake for non-ASCII; audit M16). Shared by every admin
/// redirect that interpolates user text.
pub(crate) fn urlenc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
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
    // Registered plugin kinds get one generic menu entry each (adapter
    // may override the submenu via `PluginAdapter::submenu`).
    for def in plugins::kind_defs() {
        if def.menu_icon.is_empty() || matches!(def.kind, "post" | "page") {
            continue;
        }
        html.push_str(&menu_li(
            def.section,
            &format!("/admin/{}", def.section),
            def.list_title,
            def.menu_icon,
            active,
            &plugins::submenu_for(&s.db, def),
        ));
    }

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
<script src="/admin/wp/atrium.js" defer></script>
</div><!-- wpwrap -->"#)
}
/// `/admin/plugins/{slug}/{*rest}` — dispatch to the adapter's
/// `ScreenDef`. Registered before `/admin/{section}`; `"plugins"` is a
/// reserved section name (no KindDef may claim it — red-team R5).
async fn plugin_screen(
    State(app): State<App>,
    headers: HeaderMap,
    Path((slug, rest)): Path<(String, String)>,
    req: axum::extract::Request,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let Some(adapter) = plugins::adapter_for_slug(&slug) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // PhpBridge plugins proxy to their isolated WP runtime; every other
    // backend resolves a local ScreenDef. The opt-in env gate is absolute:
    // unlisted slugs never reach spawn.
    if adapter.backend() == plugins::Backend::PhpBridge {
        return bridge_dispatch(&app, &s, sc, adapter, headers, &rest, req).await;
    }
    let Some((_a, screen)) = plugins::screen_for(&slug, &rest) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    (screen.handle)(&app, &s, sc, headers, &rest, req).await
}

/// Proxy `/admin/plugins/<slug>/<rest>` into a PhpBridge runtime.
/// Opt-in gate (`PLUGINS_PHP` env) → spawn-on-demand → forward → pull
/// whitelisted writes. Missing runtime degrades to an install note;
/// the rest of admin is unaffected (R10).
async fn bridge_dispatch(
    _app: &App,
    s: &Sess,
    sc: &SiteCtx,
    adapter: &'static dyn plugins::PluginAdapter,
    headers: HeaderMap,
    rest: &str,
    req: axum::extract::Request,
) -> Response {
    let slug = adapter.slug();
    if !bridge::opt_in(&sc.plugins_php, slug) {
        return html_admin(
            s,
            "Plugin bridge",
            &format!(
                "<div class=\"wrap\"><h1>{} (PHP bridge)</h1>\
                 <p>This plugin runs on the PHP bridge backend and is not enabled for this site.</p>\
                 <p>Add <code>PLUGINS_PHP={}</code> to the site env and restart to enable it.</p></div>",
                slug, slug
            ),
            false,
            "plugins",
        )
        .into_response();
    }
    let method = req.method().to_string();
    let body = match axum::body::to_bytes(req.into_body(), 8 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let hdrs: Vec<(String, String)> = headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();
    let site = sc.slug.clone();
    let www = sc.www.clone();
    let db = sc.db.clone();
    let tpl = sc.templates.clone();
    /* The bridge's wp.db hosts every mounted plugin AND the site's theme —
       writes can land in keys owned by other adapters (e.g. skins write
       `traveltour_color` — the theme's option, not `gdlr-core-%`). Pull with
       the union of ALL adapters' write_specs, not just the dispatched one:
       the whitelist is the safety boundary, not the dispatch adapter. */
    let specs: Vec<crate::admin::bridge::pull::WriteSpec> = plugins::adapters()
        .iter()
        .flat_map(|a| a.write_specs().iter().copied())
        .collect();
    let path = adapter.bridge_path(rest);
    let bridges = sc.bridges.clone();
    // T25: the post being edited is the last numeric path segment
    // (e.g. `/admin/plugins/goodlayers/page-builder/edit/42` → 42). No match
    // → empty wp.db: the plugin screens that need no post context.
    let post_id = rest
        .rsplit('/')
        .find_map(|seg| seg.parse::<i64>().ok());
    let resp = tokio::task::spawn_blocking(move || {
        let h = bridge::bridge_for(&bridges, &site, slug, bridge::RuntimeKind::Playground, &www, &db, post_id, adapter.wp_plugin_file(), &tpl)?;
        // Entry mutex serializes this bridge's runtime; the global map
        // lock is already released — other sites' bridge traffic flows.
        let mut st = h.lock();
        let token = st.token.clone();
        let mut fwd = hdrs.clone();
        fwd.push(("X-Rpress-Bridge".into(), token));
        let r = st.rt.request(&method, &path, &fwd, &body)?;
        // Pull whitelisted writes in the same task — no drift window.
        if let Some(wp_db) = bridge::pull::locate(&st.wp_db) {
            let _ = bridge::pull::pull(&wp_db, &db, &specs);
        }
        Ok::<_, anyhow::Error>(r)
    })
    .await;
    match resp {
        Ok(Ok(r)) => {
            let mut b = axum::response::Response::builder().status(
                axum::http::StatusCode::from_u16(r.status).unwrap_or(StatusCode::OK),
            );
            for (k, v) in &r.headers {
                let kl = k.to_ascii_lowercase();
                if matches!(kl.as_str(), "transfer-encoding" | "connection" | "content-length") {
                    continue;
                }
                if let (Ok(n), Ok(val)) = (header::HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(v)) {
                    b = b.header(n, val);
                }
            }
            b.body(Body::from(r.body)).unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Ok(Err(e)) => html_admin(
            s,
            "Plugin bridge",
            &format!(
                "<div class=\"wrap\"><h1>{} (PHP bridge)</h1>\
                 <p>Bridge unavailable: <code>{}</code></p>\
                 <p>Install Node + <code>@wp-playground/cli</code>, or check the runtime log.</p></div>",
                slug, e
            ),
            false,
            "plugins",
        )
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Legacy tourmaster ajax URL — 307 preserves POST+body; vendored JS and
/// any cached edit pages keep working until assets are republished.
async fn save_option_shim() -> Response {
    Redirect::temporary("/admin/plugins/tourmaster/save-option").into_response()
}

/// `/admin/api/gdlr/ajax` — legacy vendored-JS ajax URL → bridge screen.
/// 307 keeps POST + body. `gdlr_pb` (the Rust port) was deleted after the
/// real-site parity checklist passed (phase-07).
async fn gdlr_ajax() -> Response {
    Redirect::temporary("/admin/plugins/goodlayers/ajax").into_response()
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
    let mut b = safe_read_under(&sc.templates.join("admin/wp"), &path);
    // `gdlr/<rest>` is the GoodLayers asset namespace: plugin assets live in
    // the site's own wp-content (vendored, runs inside the bridge too).
    // Order: goodlayers-core first (framework/js, include/js, images), then
    // any sibling plugin dir (tourmaster/tourmaster.css et al.).
    if b.is_none() {
        if let Some(rest) = path.strip_prefix("gdlr/") {
            let plugins = sc.www.join("wp-content/plugins");
            b = safe_read_under(&plugins.join("goodlayers-core"), rest)
                .or_else(|| safe_read_under(&plugins, rest))
                // `gdlr/tourmaster.css` → `plugins/tourmaster/tourmaster.css`
                // (sibling-plugin asset emitted under the gdlr namespace).
                .or_else(|| {
                    let p = std::path::Path::new(rest);
                    let stem = p.file_stem().and_then(|s| s.to_str())?;
                    safe_read_under(&plugins.join(stem), rest)
                });
        }
    }
    /* `<adapter-slug>/<rest>`: plugin asset namespace for registered
       adapters that vendor their own admin assets under
       `www/wp-content/plugins/<slug>/` (tourmaster framework css/js).
       Only asset extensions are servable — `.php`, `.ini`, `.xml` et al.
       404 so vendored plugin code never becomes downloadable. */
    if b.is_none() {
        if let Some((slug, rest)) = path.split_once('/') {
            const ASSET_EXT: &[&str] = &["css", "js", "woff2", "woff", "ttf", "svg", "png", "jpg", "jpeg", "gif", "ico", "webp"];
            let ext_ok = std::path::Path::new(rest)
                .extension().and_then(|e| e.to_str())
                .map(|e| ASSET_EXT.contains(&e)).unwrap_or(false);
            if ext_ok && plugins::adapter_for_slug(slug).is_some() {
                b = safe_read_under(&sc.www.join("wp-content/plugins").join(slug), rest);
            }
        }
    }
    let Some(b) = b else { return StatusCode::NOT_FOUND.into_response() };
    let ct = match PathBuf::from(&path).extension().and_then(|s| s.to_str()).unwrap_or("") {
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };
    ([(header::CONTENT_TYPE, ct)], b).into_response()
}

/// `/wp-content/uploads/*` — imported media lives under `www/` when copied
/// locally. Serve safe media inline; force scriptable legacy extensions to
/// download as octet-stream so copied WP uploads cannot become same-origin XSS.
/// A miss 307s to the asset origin only through the `asset_redirect_target`
/// gate (same-site or `RPRESS_ASSET_ORIGINS`-listed host), else 404.
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
    // Same gate as public_page's asset-miss redirect: the origin must be
    // http(s), not a self-loop, and same-site or env-allowlisted — else 404.
    if let Some(origin) = asset_redirect_target(&asset_origin(sc), &site_url(sc), &ASSET_ORIGIN_ALLOW) {
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
    // Per-site scoped css wins: `migrate` writes `<name>.scoped.css`
    // twins beside the site's uploads (traveltour-style-custom). Repo
    // `front-css/` stays as the fallback for vendored non-site files.
    match safe_read_under(&sc.www.join("wp-content/uploads"), &path)
        .or_else(|| safe_read_under(&sc.templates.join("front-css"), &path))
    {
        Some(b) => ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], b).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

