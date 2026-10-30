//! Public site serving: the lazy write-back cache in front of
//! `crate::render` (page/asset/file handlers, canonical redirects, MIME +
//! ETag responses, Host gate) plus the serve()-only boot/tick duties
//! (`purge_alias_caches`, `publish_due`). Route registration stays in
//! `crate::admin::serve`; admin/auth handlers live in `crate::admin` proper.
//!
//! Shared request primitives (`site_for`, `client_ip`, session plumbing)
//! stay in `crate::admin` — auth and admin screens consume them too.
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::{admin, asset_origin, client_ip, site_for, site_url, tour_base, App, SiteCtx};
use crate::{db as crud, plugins};

pub(crate) async fn health() -> &'static str {
    "ok"
}

/// `GET /` — public front page: `out/index.html` when present, else a live
/// render of the `page_on_front` post — write-back so the homepage warms
/// itself instead of rendering on every cold hit.
pub(crate) async fn public_root(State(app): State<App>, headers: HeaderMap, ConnectInfo(peer): ConnectInfo<SocketAddr>) -> Response {
    let sc = site_for(&app, &headers);
    if let Some(b) = read_out_async(sc, "index.html".to_string()).await {
        return file_response(&headers, "text/html; charset=utf-8".to_string(), b);
    }
    // Front-page render is the heaviest miss — same 60/min/IP cap as
    // public_page (a cold-cache / used to be unthrottled).
    let ip = client_ip(&headers, Some(peer));
    {
        let mut rate = app.miss_rate.lock();
        if !rate.allow(&ip, 60, Duration::from_secs(60)) {
            return (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, "60")]).into_response();
        }
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
            _ => not_found(sc).await,
        },
        None => not_found(sc).await,
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
pub(crate) async fn public_page(State(app): State<App>, headers: HeaderMap, ConnectInfo(peer): ConnectInfo<SocketAddr>, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    let clean = path.trim_matches('/');
    // Path charset allowlist: WP asset filenames legitimately carry
    // uppercase (ElegantIcons.woff/.svg, Font Awesome dirs, …), so the
    // gate can't be lowercase-only. Slug lookup lowercases anyway —
    // uppercase only matters for static-file resolution.
    if clean.contains("..") || clean.starts_with('/') || clean.contains('\\')
        || clean.bytes().any(|b| !matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'/' | b'_' | b'%' | b'.' | b'~' | b'-'))
    {
        return not_found(sc).await;
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
            if let Some(b) = read_out_async(sc, rel.clone()).await {
                let mime = public_mime(&rel).to_string();
                return file_response(&headers, mime, b);
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
    // Real asset paths serve verbatim — ca07f11 dropped this candidate
    // and every wp-content/themes/plugin css|js 404ed on any site whose
    // asset_origin host isn't env-allowlisted. Gated by the same
    // allowlist as wp_asset: `.php`, `.ini`, manifests et al. can never
    // become same-origin downloads (or disclose vendored source).
    if asset_ext_ok(clean)
        && (redirect_prefix(clean) || matches!(clean, "front.css" | "nav.js" | "favicon.ico" | "robots.txt" | "llms.txt"))
    {
        candidates.push(clean.to_string());
    }
    for cand in candidates {
        if let Some(b) = read_out_async(sc, cand.clone()).await {
            let mime = if cand.ends_with(".md") { "text/markdown; charset=utf-8".to_string() } else { public_mime(&cand).to_string() };
            return file_response(&headers, mime, b);
        }
    }
    // Cache missed — resolve what this URL is allowed to become before
    // paying for a render.
    let conn = match crate::db::conn(&sc.db) { Ok(c) => c, Err(_) => return not_found(sc).await };
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
        return not_found(sc).await;
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
    // A path carrying a file extension that didn't resolve above is a missing
    // asset, not a permalink — only the real WP asset trees may bounce to the
    // (settings-controlled, allowlisted) asset origin; every other miss 404s.
    if PathBuf::from(clean).extension().is_some() {
        if redirect_prefix(clean) {
            if let Some(origin) = asset_redirect_target(&asset_origin(sc), &site_url(sc), &ASSET_ORIGIN_ALLOW) {
                return Redirect::temporary(&format!("{origin}/{clean}")).into_response();
            }
        }
        return not_found(sc).await;
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
        return not_found(sc).await;
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
                return not_found(sc).await;
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
            not_found(sc).await
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

/// Canonicalized lookup roots, memoized — `canonicalize` is a syscall per
/// component on every request otherwise (hot under `read_out`). Roots are
/// process-stable paths (out/www/templates); entries live for the process.
fn canonical_root(root: &std::path::Path) -> Option<PathBuf> {
    static CACHE: std::sync::LazyLock<Mutex<HashMap<PathBuf, Option<PathBuf>>>> =
        std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
    if let Some(v) = CACHE.lock().get(root) {
        return v.clone();
    }
    let v = root.canonicalize().ok();
    CACHE.lock().insert(root.to_path_buf(), v.clone());
    v
}

pub fn safe_read_under(root: &std::path::Path, rel: &str) -> Option<Vec<u8>> {
    if rel.contains("..") || rel.starts_with('/') || rel.contains('\\') {
        return None;
    }
    let root = canonical_root(root)?;
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

/// Weak ETag (FNV-1a over bytes) — per-request hash cost is trivial vs the
/// disk read already paid; conditional GET then lets repeat hits answer
/// 304 instead of re-sending the whole file.
fn etag_of(b: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for &x in b {
        h = (h ^ x as u64).wrapping_mul(0x100000001b3);
    }
    format!("W/\"{h:x}\"")
}

/// 304 when the client's validator matches; else the full body with the
/// validator attached. `no-cache` stays — the point is revalidation.
fn file_response(headers: &HeaderMap, mime: String, b: Vec<u8>) -> Response {
    let etag = etag_of(&b);
    let hit = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').any(|t| t.trim() == etag))
        .unwrap_or(false);
    if hit {
        return ([(header::ETAG, etag), (header::CACHE_CONTROL, "no-cache".to_string())], StatusCode::NOT_MODIFIED).into_response();
    }
    (
        [
            (header::CONTENT_TYPE, mime),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
            (header::CACHE_CONTROL, "no-cache".to_string()),
            (header::ETAG, etag),
        ],
        b,
    )
        .into_response()
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

/// `read_out` off the reactor: every candidate hit is a sync stat+read —
/// under load (or a big-file spray) that blocks tokio workers serving
/// other routes. Wraps the whole path in spawn_blocking; the sync variant
/// stays for call sites already on a blocking thread (bridge proxy).
async fn read_out_async(sc: &SiteCtx, rel: String) -> Option<Vec<u8>> {
    let sc = sc.clone();
    tokio::task::spawn_blocking(move || read_out(&sc, &rel))
        .await
        .ok()
        .flatten()
}

/// Async `safe_read_under` — same rationale as `read_out_async`.
async fn safe_read_async(root: PathBuf, rel: String) -> Option<Vec<u8>> {
    tokio::task::spawn_blocking(move || safe_read_under(&root, &rel))
        .await
        .ok()
        .flatten()
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

async fn not_found(sc: &SiteCtx) -> Response {
    match read_out_async(sc, "404.html".to_string()).await {
        Some(b) => (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/html; charset=utf-8"), (header::X_CONTENT_TYPE_OPTIONS, "nosniff")], b).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Request Host header, normalized to a bare lowercase hostname.
pub(crate) fn request_host(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(|h| h.split(':').next().unwrap_or(h).to_lowercase())
}

/// Multi-site gate: with more than one site configured there is no safe
/// default host, so a request for an unknown (or missing) Host gets 421
/// Misdirected Request instead of silently serving the default site's
/// content. Single-site keeps the fallback — dev requests hit localhost.
pub(crate) async fn host_known(
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

/// `publish_due` min gap per site.
const PUBLISH_DUE_GAP: Duration = Duration::from_secs(30);

/// Flip due 'future' posts to 'publish' and regenerate their files.
/// Runs on the serve() minute tick (it used to piggyback on get_sess —
/// a site with no admin traffic never published): the UPDATE is a no-op
/// when nothing is due; a flip spawns one background thread that writes
/// each post's output file via the single-post regen path.
pub(crate) fn publish_due(sc: &SiteCtx) {
    // Throttle: the UPDATE is cheap but it still opens a write tx + a
    // status='future' scan. 30s granularity is well inside WP's
    // minute-level schedule semantics.
    {
        let mut at = sc.publish_due_at.lock();
        if at.map(|t| t.elapsed() < PUBLISH_DUE_GAP).unwrap_or(false) {
            return;
        }
        *at = Some(Instant::now());
    }
    let Ok(flipped) = crud::publish_due(&sc.db) else { return };
    if flipped.is_empty() {
        return;
    }
    let ids: Vec<i64> = flipped.iter().map(|(id, ..)| *id).collect();
    // The phase-4 single-post regen hook runs this off the request thread.
    admin::regen_posts(sc, &ids);
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
pub(crate) fn purge_alias_caches(db: &PathBuf, out: &PathBuf) {
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

/// Asset extensions servable through `/admin/wp/*` — static frontend bits
/// only. `.php/.ini/.txt/.md/.xml/.pot/.mo/.po/.scss/.ts/.html/.sql/.log`
/// and `.json` (composer.json fingerprints) are NOT here: anything outside
/// this list 404s so vendored plugin code/config never becomes readable.
/// `eot/otf/cur/map` are real (fontawesome, ilightbox, tinymce maps) —
/// the audit confirmed removing them breaks Live Mode icons.
const ALLOWED_ASSET_EXT: &[&str] = &[
    "css", "js", "map", "woff2", "woff", "ttf", "otf", "eot",
    "svg", "png", "jpg", "jpeg", "gif", "ico", "webp", "cur",
];

fn asset_ext_ok(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| ALLOWED_ASSET_EXT.contains(&e))
        .unwrap_or(false)
}

pub(crate) async fn wp_asset(State(app): State<App>, headers: HeaderMap, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    // Admin chrome under templates/admin/wp is gated by the same asset
    // allowlist — vendored manifests/hashes are not for anonymous reads.
    let mut b = if asset_ext_ok(&path) {
        safe_read_async(sc.templates.join("admin/wp"), path.clone()).await
    } else {
        None
    };
    // `gdlr/<rest>` is the GoodLayers asset namespace: plugin assets live in
    // the site's own wp-content (vendored, runs inside the bridge too).
    // Order: goodlayers-core first (framework/js, include/js, images), then
    // any sibling plugin dir (tourmaster/tourmaster.css et al.). Gated by
    // the same allowlist — vendored source stays unreadable.
    if b.is_none() {
        if let Some(rest) = path.strip_prefix("gdlr/") {
            if asset_ext_ok(rest) {
                let plugins = sc.www.join("wp-content/plugins");
                let bundled = sc.templates.join("wp-plugins").join("goodlayers-core");
                // Sequential async fallbacks — each is a spawn_blocking'd
                // stat+read; sync `.or_else(||)` can't await, so this is a
                // plain loop over candidate roots.
                let stem = std::path::Path::new(rest)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let roots = [
                    plugins.join("goodlayers-core"),
                    plugins.clone(),
                    plugins.join(&stem),
                    sc.templates.join("wp-plugins").join(&stem),
                    bundled,
                ];
                for root in roots {
                    if let Some(x) = safe_read_async(root, rest.to_string()).await {
                        b = Some(x);
                        break;
                    }
                }
            }
        }
    }
    /* `<adapter-slug>/<rest>`: plugin asset namespace for registered
       adapters that vendor their own admin assets under
       `www/wp-content/plugins/<slug>/` (tourmaster framework css/js).
       Only asset extensions are servable — `.php`, `.ini`, `.xml` et al.
       404 so vendored plugin code never becomes downloadable. */
    if b.is_none() {
        if let Some((slug, rest)) = path.split_once('/') {
            if asset_ext_ok(rest) && plugins::adapter_for_slug(slug).is_some() {
                for root in [
                    sc.www.join("wp-content/plugins").join(slug),
                    // Site copy absent → the vendored bundle under
                    // templates/wp-plugins/<slug>/ (same fallback the
                    // bridge mounts; tourmaster framework css/js/img).
                    sc.templates.join("wp-plugins").join(slug),
                ] {
                    if let Some(x) = safe_read_async(root, rest.to_string()).await {
                        b = Some(x);
                        break;
                    }
                }
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
pub(crate) async fn uploads(State(app): State<App>, headers: HeaderMap, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    let root = sc.www.join("wp-content/uploads");
    if let Some(bytes) = safe_read_async(root, path.clone()).await {
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

/// `/admin/front-css/*` — vendored theme/plugin frontend stylesheets, loaded
/// only inside the page-builder Live Mode canvas so it matches the public site.
pub(crate) async fn front_css(State(app): State<App>, headers: HeaderMap, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    // `.css` only — the whole uploads tree sat behind this route otherwise:
    // svg/script/originals readable anonymously. `nosniff` pins the type.
    // Per-site scoped css wins (`<name>.scoped.css` twins beside uploads);
    // repo `front-css/` stays the fallback for vendored non-site files.
    if !std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("css"))
        .unwrap_or(false)
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    match safe_read_async(sc.www.join("wp-content/uploads"), path.clone()).await
        .or(safe_read_async(sc.templates.join("front-css"), path.clone()).await)
    {
        Some(b) => (
            [(header::CONTENT_TYPE, "text/css; charset=utf-8".to_string()),
             (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string())],
            b,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
