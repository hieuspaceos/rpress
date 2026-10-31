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
use std::sync::Arc;
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
    // rusqlite + template render are synchronous — the same ~140ms miss
    // that public_page already offloads (H14); a cold `/` must not block
    // a tokio worker either (audit P1).
    let (db, tpl, base, tb) = (sc.db.clone(), sc.templates.clone(), site_url(sc), tour_base(sc).to_string());
    let rendered = tokio::task::spawn_blocking(move || {
        let conn = crate::db::conn(&db).ok()?;
        let id = crate::render::front_id(&conn)?;
        crate::render::render_slug_by_id_public(&conn, &tpl, id, &base, &tb).ok()
    })
    .await
    .unwrap_or(None);
    match rendered {
        // Publish-gated render: a draft page_on_front never reaches 200.
        Some((h, status)) if status == "publish" => {
            write_cache(sc, "index.html", &h);
            html_response(h)
        }
        _ => not_found(sc).await,
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
    // lang config is a sync conn open + SELECT — offload like every other
    // miss-path lookup (audit-261031 C1); conn Err keeps the 404 shape.
    let db_l = sc.db.clone();
    let lang_cfg = match tokio::task::spawn_blocking(move || {
        let conn = crate::db::conn(&db_l).ok()?;
        Some(crate::lang::config(&conn))
    })
    .await
    .ok()
    .flatten()
    {
        Some(c) => c,
        None => return not_found(sc).await,
    };
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
        // DB lookups + cache write are sync — same offload as the render
        // below (audit M-miss): don't run conn-opens on a tokio worker.
        let (sc_c, clean_c) = (sc.clone(), clean.to_string());
        let md_result = tokio::task::spawn_blocking(move || {
            md_for_slug(&sc_c, &clean_c).and_then(|(md_id, md)| {
                let conn = crate::db::conn(&sc_c.db).ok()?;
                let canonical = crate::render::post_canonical_path(&conn, md_id, tour_base(&sc_c));
                Some((md, canonical))
            })
        })
        .await
        .ok()
        .flatten();
        if let Some((md, canonical)) = md_result {
            // Canonical-rel gate (M22): a .md request on a non-canonical
            // alias 301s to the canonical .md instead of writing a stale
            // alias copy.
            let req = format!("/{}/", clean.trim_matches('/'));
            if let Some(c) = canonical {
                if c != req {
                    return moved_permanently(&format!("{}.md", c.trim_end_matches('/')));
                }
            }
            let (sc_c, rel, md_c) = (sc.clone(), format!("{clean}/index.md"), md.clone());
            let _ = tokio::task::spawn_blocking(move || write_cache(&sc_c, &rel, &md_c)).await;
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
    // Single-flight: N concurrent misses on the same path trigger one
    // render (audit §3 H4). Followers wait on the watch channel then fall
    // through to the file-cache path — the leader's write-back lands there.
    // A leader that ended in 404/301/non-publish leaves nothing cached, so
    // a follower that misses after changed() loops back: it either follows
    // the next leader or becomes it — never renders unregistered, and its
    // guard only removes ITS OWN sender (a successor leader is safe).
    let mut guard_tx = None;
    loop {
        let next = {
            let mut infl = app.inflight_renders.lock();
            match infl.entry(clean.to_string()) {
                std::collections::hash_map::Entry::Occupied(e) => Some(e.get().subscribe()),
                std::collections::hash_map::Entry::Vacant(v) => {
                    let (tx, _rx) = tokio::sync::watch::channel(None);
                    v.insert(tx.clone());
                    guard_tx = Some(tx);
                    None
                }
            }
        };
        match next {
            None => break, // we are the leader
            Some(mut rx) => {
                let _ = rx.changed().await;
                if let Some(b) = read_out_async(sc, format!("{clean}/index.html")).await {
                    return file_response(&headers, "text/html; charset=utf-8".to_string(), b);
                }
                // Leader failed or served a redirect/404 — loop: follow the
                // next leader if one registered meanwhile, else lead.
            }
        }
    }

    // RAII guard: whatever happens below (early returns included), the
    // inflight key must not leak or waiters hang forever. Removal is
    // ownership-checked — only OUR sender is taken out, so a successor
    // leader's entry survives a stale guard drop.
    struct InflightGuard {
        map: Arc<Mutex<HashMap<String, tokio::sync::watch::Sender<Option<()>>>>>,
        key: String,
        tx: tokio::sync::watch::Sender<Option<()>>,
    }
    impl Drop for InflightGuard {
        fn drop(&mut self) {
            let mut m = self.map.lock();
            if m.get(&self.key).map(|t| t.same_channel(&self.tx)).unwrap_or(false) {
                m.remove(&self.key);
            }
            drop(m);
            let _ = self.tx.send(Some(()));
        }
    }
    let _inflight_guard = InflightGuard {
        map: app.inflight_renders.clone(),
        key: clean.to_string(),
        tx: guard_tx.expect("leader always holds a sender"),
    };
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
            let (sc_c, rel, h_c) = (sc.clone(), format!("{clean}/index.html"), h.clone());
            let _ = tokio::task::spawn_blocking(move || write_cache(&sc_c, &rel, &h_c)).await;
            html_response(h)
        }
        Err(_) => {
            // The lang variant doesn't exist but a translation-group sibling
            // does → 301 to the canonical permalink, else 404 (Polylang).
            // Same blocking-work offload as the render above.
            let (db, lang_s_c, lookup_c, kind_c, tb_c) = (
                sc.db.clone(), lang_s.clone(), lookup.to_string(), kind.to_string(), tour_base(sc).to_string(),
            );
            let target = tokio::task::spawn_blocking(move || {
                let conn = crate::db::conn(&db).ok()?;
                let cfg = crate::lang::config(&conn);
                crate::render::translation_canonical(&conn, &cfg, &lang_s_c, &lookup_c, &kind_c, &tb_c)
            })
            .await
            .ok()
            .flatten();
            if let Some(target) = target {
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
fn md_for_slug(sc: &SiteCtx, clean: &str) -> Option<(i64, String)> { // gate-allow: blocking by contract — only called inside spawn_blocking (public_page .md miss)
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


/// Resolve `rel` under `root` to a safe canonical file path — the same
/// confinement as `safe_read_under` but returns the path for streaming
/// instead of reading bytes (audit M-range: whole-file fs::read buffered
/// every upload into memory; streaming + Range fixes both).
fn safe_resolve_under(root: &std::path::Path, rel: &str) -> Option<PathBuf> {
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
    Some(candidate)
}

/// Stream `path` with `Accept-Ranges` + single-range `206` support,
/// metadata ETag, `no-cache` revalidation semantics matching
/// `file_response`. Reads 64KB chunks — the file never lands fully in
/// memory (audit M-range).
async fn stream_file(headers: &HeaderMap, path: PathBuf, mime: &str) -> Response {
    let meta = match tokio::fs::metadata(&path).await {
        Ok(m) => m,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    let len = meta.len();
    // Weak ETag from len+mtime — no content hash needed for a revalidate;
    // mtime resolution covers the write-back-on-edit window.
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let etag = format!("W/\"{len:x}-{mtime:x}\"");
    let base_headers = [
        (header::CONTENT_TYPE, mime.to_string()),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
        (header::CACHE_CONTROL, "no-cache".to_string()),
        (header::ETAG, etag.clone()),
        (header::ACCEPT_RANGES, "bytes".to_string()),
    ];
    if headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').any(|t| t.trim() == etag))
        .unwrap_or(false)
    {
        return (base_headers, StatusCode::NOT_MODIFIED).into_response();
    }
    // Single-range only (`bytes=a-b` / `a-` / `-b`); multipart ranges fall
    // back to 200 — full body is a legal answer.
    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("bytes="))
        .and_then(|spec| {
            let (a, b) = spec.split_once('-')?;
            if spec.contains(',') {
                return None;
            }
            let (start, end) = if a.is_empty() {
                let n: u64 = b.parse().ok()?;
                if n == 0 {
                    return None;
                }
                (len.saturating_sub(n), len.saturating_sub(1))
            } else {
                let s: u64 = a.parse().ok()?;
                let e = if b.is_empty() { len.saturating_sub(1) } else { b.parse().ok()? };
                (s, e.min(len.saturating_sub(1)))
            };
            if start > end || start >= len {
                return None;
            }
            Some((start, end))
        });
    let (status, start, end) = match range {
        Some((s, e)) => (StatusCode::PARTIAL_CONTENT, s, e),
        None => (StatusCode::OK, 0, len.saturating_sub(1)),
    };
    let stream_len = if len == 0 { 0 } else { end - start + 1 };
    let mut f = match tokio::fs::File::open(&path).await {
        Ok(f) => f,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    if start > 0 {
        use tokio::io::AsyncSeekExt;
        if f.seek(std::io::SeekFrom::Start(start)).await.is_err() {
            return StatusCode::RANGE_NOT_SATISFIABLE.into_response();
        }
    }
    let stream = futures_util::stream::unfold((f, stream_len), |(mut f, mut left)| async move {
        if left == 0 {
            return None;
        }
        let take = left.min(64 * 1024) as usize;
        let mut buf = vec![0u8; take];
        use tokio::io::AsyncReadExt;
        match f.read(&mut buf).await {
            Ok(0) | Err(_) => None,
            Ok(n) => {
                buf.truncate(n);
                left -= n as u64;
                Some((Ok::<_, std::io::Error>(buf), (f, left)))
            }
        }
    });
    let mut res = Response::builder().status(status);
    for (k, v) in base_headers {
        res = res.header(k, v);
    }
    res = res.header(header::CONTENT_LENGTH, stream_len.to_string());
    if status == StatusCode::PARTIAL_CONTENT {
        res = res.header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{len}"));
    }
    res.body(Body::from_stream(stream))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
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
/// Handles `host`, `host:port`, `[v6]`, `[v6]:port`: a bracketed literal
/// yields everything up to `]`; otherwise a trailing `:port` is stripped
/// only when the suffix parses as u16 (bare v6 with colons stays whole).
#[doc(hidden)]
pub fn request_host(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(|h| {
            if let Some(rest) = h.strip_prefix('[') {
                return rest.split(']').next().unwrap_or(rest).to_lowercase();
            }
            /* `host:port` strips a u16 port; a bare v6 literal also ends in
               `:<digits>` — when the head still contains ':' the suffix is
               part of the address, not a port. */
            match h.rsplit_once(':') {
                Some((host, port)) if !host.contains(':') && port.parse::<u16>().is_ok() => {
                    host.to_lowercase()
                }
                _ => h.to_lowercase(),
            }
        })
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
pub(crate) fn publish_due(sc: &SiteCtx) { // gate-allow: blocking by contract — only called inside the minute-tick spawn_blocking
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
pub(crate) fn purge_alias_caches(db: &PathBuf, out: &PathBuf) { // gate-allow: blocking by contract — only called inside serve()'s boot spawn_blocking
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
    let mut f: Option<PathBuf> = if asset_ext_ok(&path) {
        safe_resolve_under(&sc.templates.join("admin/wp"), &path)
    } else {
        None
    };
    // `gdlr/<rest>` is the GoodLayers asset namespace: plugin assets live in
    // the site's own wp-content (vendored, runs inside the bridge too).
    // Order: goodlayers-core first (framework/js, include/js, images), then
    // any sibling plugin dir (tourmaster/tourmaster.css et al.). Gated by
    // the same allowlist — vendored source stays unreadable.
    if f.is_none() {
        if let Some(rest) = path.strip_prefix("gdlr/") {
            if asset_ext_ok(rest) {
                let plugins = sc.www.join("wp-content/plugins");
                let bundled = sc.templates.join("wp-plugins").join("goodlayers-core");
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
                    if let Some(x) = safe_resolve_under(&root, rest) {
                        f = Some(x);
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
    if f.is_none() {
        if let Some((slug, rest)) = path.split_once('/') {
            if asset_ext_ok(rest) && plugins::adapter_for_slug(slug).is_some() {
                for root in [
                    sc.www.join("wp-content/plugins").join(slug),
                    // Site copy absent → the vendored bundle under
                    // templates/wp-plugins/<slug>/ (same fallback the
                    // bridge mounts; tourmaster framework css/js/img).
                    sc.templates.join("wp-plugins").join(slug),
                ] {
                    if let Some(x) = safe_resolve_under(&root, rest) {
                        f = Some(x);
                        break;
                    }
                }
            }
        }
    }
    let Some(file) = f else { return StatusCode::NOT_FOUND.into_response() };
    let ext = PathBuf::from(&path).extension().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let ext = ext.as_str();
    let ct = match ext {
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
    // Streamed + Range + ETag via stream_file (audit M-range/M-assets).
    // SVGs additionally get a null-script CSP — same-origin SVG executes
    // script when navigated to directly; `attachment` was rejected
    // because <img>/CSS loads keep working under CSP but break under a
    // download disposition (audit L2).
    let mut r = stream_file(&headers, file, ct).await;
    if ext == "svg" {
        r.headers_mut().insert(
            header::CONTENT_SECURITY_POLICY,
            axum::http::HeaderValue::from_static("script-src 'none'; style-src 'unsafe-inline'"),
        );
    }
    r
}

/// `/wp-content/uploads/*` — imported media lives under `www/` when copied
/// locally. Serve safe media inline; force scriptable legacy extensions to
/// download as octet-stream so copied WP uploads cannot become same-origin XSS.
/// A miss 307s to the asset origin only through the `asset_redirect_target`
/// gate (same-site or `RPRESS_ASSET_ORIGINS`-listed host), else 404.
pub(crate) async fn uploads(State(app): State<App>, headers: HeaderMap, Path(path): Path<String>) -> Response {
    let sc = site_for(&app, &headers);
    let root = sc.www.join("wp-content/uploads");
    if let Some(file) = safe_resolve_under(&root, &path) {
        let (mime, attachment) = crud::served_upload_mime(&path); // gate-allow: pure mime/extension map (db/media.rs) — no DB, no I/O
        let mut r = stream_file(&headers, file, mime).await;
        if attachment {
            let name = path.rsplit('/').next().unwrap_or("download.bin").replace('"', "_");
            if let Ok(v) = format!("attachment; filename=\"{name}\"").parse() {
                r.headers_mut().insert(header::CONTENT_DISPOSITION, v);
            }
        }
        return r;
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
    match safe_resolve_under(&sc.www.join("wp-content/uploads"), &path)
        .or_else(|| safe_resolve_under(&sc.templates.join("front-css"), &path))
    {
        Some(f) => stream_file(&headers, f, "text/css; charset=utf-8").await,
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
