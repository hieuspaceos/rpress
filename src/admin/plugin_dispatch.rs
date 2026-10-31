//! `/admin/plugins/{slug}/{*rest}` dispatch — ScreenDef routing, the
//! PhpBridge proxy hop, and the legacy ajax 307 shims. Handlers-only
//! module; router wiring stays in `mod.rs::serve`.
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::plugins;

use super::bridge;
use super::chrome::html_admin;
use super::util::check_csrf;
use super::{need_auth_peer, App, SiteCtx, Sess};

/// `/admin/plugins/{slug}/{*rest}` — dispatch to the adapter's
/// `ScreenDef`. Registered before `/admin/{section}`; `"plugins"` is a
/// reserved section name (no KindDef may claim it — red-team R5).
pub(super) async fn plugin_screen(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path((slug, rest)): Path<(String, String)>,
    req: axum::extract::Request,
) -> Response {
    let Ok((s, sc)) = need_auth_peer(&app, &headers, Some(peer)) else {
        return Redirect::to("/admin/login").into_response();
    };
    // Role gate per screen: settings surfaces stay admin-only, in-editor
    // AJAX (tourmaster save-option) allows editors — the old blanket
    // `role != "admin"` made every metabox AJAX post 403 for editors.
    if let Some((_a, screen)) = plugins::screen_for(&slug, &rest) {
        let ok = match screen.min_role {
            "editor" => s.role == "admin" || s.role == "editor",
            // Unknown/future values stay admin-only (deny by default).
            _ => s.role == "admin",
        };
        if !ok {
            return StatusCode::FORBIDDEN.into_response();
        }
    } else if s.role != "admin" {
        // Bridge plugins and unknown screens stay admin-only.
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some(adapter) = plugins::adapter_for_slug(&slug) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // PhpBridge plugins proxy to their isolated WP runtime; every other
    // backend resolves a local ScreenDef. The opt-in env gate is absolute:
    // unlisted slugs never reach spawn. The bridged wp-admin path itself
    // carries no rpress CSRF token — bridged POSTs rely on the session
    // cookie's SameSite=Lax (cross-site POSTs never reach this handler)
    // plus the per-spawn bridge bearer inside the proxy hop.
    if adapter.backend() == plugins::Backend::PhpBridge {
        return bridge_dispatch(&app, &s, sc, adapter, headers, &rest, req).await;
    }
    let Some((_a, screen)) = plugins::screen_for(&slug, &rest) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // POST to an adapter ScreenDef must carry the session CSRF token,
    // same as every first-party admin POST. The bridged path is exempt —
    // vendored wp-admin JS can't know our token (SameSite=Lax reliance
    // documented above + docs/architecture.md). The body is buffered so
    // the handler's own extractor still sees it.
    let mut req = req;
    if req.method() == axum::http::Method::POST {
        let (parts, body) = req.into_parts();
        let bytes = match axum::body::to_bytes(body, 8 * 1024 * 1024).await {
            Ok(b) => b,
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        };
        /* Adapter POSTs must carry the session CSRF token. Tourmaster's
           shipped JS names the field `security` (its nonce), first-party
           screens use `csrf` — accept either. The `action`-exemption is
           NOT repeated here (any non-empty action would bypass the gate
           for every adapter): tourmaster's own handler re-applies its
           WP-parity exemption internally after this gate passes. */
        let sent = serde_urlencoded::from_bytes::<HashMap<String, String>>(&bytes)
            .ok()
            .and_then(|f| f.get("csrf").or_else(|| f.get("security")).cloned())
            .unwrap_or_default();
        if check_csrf(&s, &sent).is_err() {
            return StatusCode::FORBIDDEN.into_response();
        }
        req = axum::extract::Request::from_parts(parts, Body::from(bytes));
    }
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
       `traveltour_color` — gate-allow: example key name in comment) — the theme's option, not `gdlr-core-%`). Pull with
       the union of ALL adapters' write_specs, not just the dispatched one:
       the whitelist is the safety boundary, not the dispatch adapter. */
    let specs: Vec<bridge::pull::WriteSpec> = plugins::adapters()
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
        // Prefer native php -S (~100MB, sandbox-exec confined) over the
        // ~1.2GB WASM playground; fall back when php isn't installed.
        let kind = if bridge::php_cgi::php_bin().is_some() {
            bridge::RuntimeKind::PhpCgi
        } else {
            bridge::RuntimeKind::Playground
        };
        let h = bridge::bridge_for(&bridges, &site, slug, kind, &www, &db, post_id, adapter.wp_plugin_file(), &tpl)?;
        // Entry mutex serializes this bridge's runtime; the global map
        // lock is already released — other sites' bridge traffic flows.
        let mut st = h.lock();
        let token = st.token.clone();
        let mut fwd: Vec<(String, String)> = hdrs
            .iter()
            .filter(|(k, _)| !k.eq_ignore_ascii_case("x-rpress-bridge"))
            .cloned()
            .collect();
        fwd.push(("X-Rpress-Bridge".into(), token));
        let r = st.rt.request(&method, &path, &fwd, &body)?;
        // Pull whitelisted writes in the same task — no drift window.
        // A failed pull on a WRITE loses the edit the user just made —
        // never `let _ =` it and never answer WP's success:1; surface a
        // 502 the editor JS can report. A failed pull on a read-side
        // request (GET/asset) has nothing to lose; warn and serve.
        let is_write = !matches!(method.as_str(), "GET" | "HEAD" | "OPTIONS");
        match bridge::pull::locate(&st.wp_db) {
            Some(wp_db) => {
                if let Err(e) = bridge::pull::pull(&wp_db, &db, &specs) {
                    eprintln!("bridge pull {site}/{slug}: {e:#}");
                    if is_write {
                        // Client gets the summary; the anyhow chain stays in the log.
                        return Ok(axum::response::Response::builder()
                            .status(StatusCode::BAD_GATEWAY)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(format!(
                                r#"{{"success":0,"error":"rpress pull-back failed — your edit was not saved: {e}"}}"#
                            )))
                            .unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response()));
                    }
                }
            }
            None => {
                // wp.db should always exist post-install; a missing pull
                // source on a write silently drops the edit — 502 it.
                eprintln!("bridge pull {site}/{slug}: no pull source for {}", st.wp_db.display());
                if is_write {
                    return Ok(axum::response::Response::builder()
                        .status(StatusCode::BAD_GATEWAY)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(
                            r#"{"success":0,"error":"rpress pull-back unavailable — your edit was not saved"}"#.to_string(),
                        ))
                        .unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response()));
                }
            }
        }
        let mut b = axum::response::Response::builder().status(
            axum::http::StatusCode::from_u16(r.status).unwrap_or(StatusCode::OK),
        );
        // WP issues redirects against WP_HOME (`http://127.0.0.1:<port>`) —
        // pass them through verbatim and the browser hits the raw bridge
        // port, which 403s without the bearer. Rewrite back through the
        // proxy mount instead.
        let mount = format!("/admin/plugins/{slug}");
        let upstream = format!("http://127.0.0.1:{}", st.rt.port());
        for (k, v) in &r.headers {
            let kl = k.to_ascii_lowercase();
            if matches!(kl.as_str(), "transfer-encoding" | "connection" | "content-length") {
                continue;
            }
            let v = if kl == "location" && v.starts_with(&upstream) {
                format!("{}{}", mount, &v[upstream.len()..])
            } else {
                v.clone()
            };
            if let (Ok(n), Ok(val)) = (header::HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(&v)) {
                b = b.header(n, val);
            }
        }
        // HTML bodies carry absolute asset/link URLs built on WP_HOME
        // (http://127.0.0.1:<port>) — rewrite through the mount so the
        // browser never sees the raw bridge port (same `upstream` string
        // as the Location rewrite above).
        let body = if r
            .headers
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case("content-type") && v.contains("text/html"))
        {
            // WP also JSON-escapes host URLs in inline script settings
            // (127.0.0.1\/:port/...) — rewrite both shapes.
            let upstream_esc = upstream.replace('/', "\\/");
            let mount_esc = mount.replace('/', "\\/");
            String::from_utf8_lossy(&r.body)
                .replace(&upstream, &mount)
                .replace(&upstream_esc, &mount_esc)
                .into_bytes()
        } else {
            r.body
        };
        Ok::<_, anyhow::Error>(b.body(Body::from(body)).unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()))
    })
    .await;
    match resp {
        Ok(Ok(r)) => r,
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
pub(super) async fn save_option_shim() -> Response {
    Redirect::temporary("/admin/plugins/tourmaster/save-option").into_response()
}

/// `/admin/api/gdlr/ajax` — legacy vendored-JS ajax URL → bridge screen.
/// 307 keeps POST + body. `gdlr_pb` (the Rust port) was deleted after the
/// real-site parity checklist passed (phase-07).
pub(super) async fn gdlr_ajax() -> Response {
    Redirect::temporary("/admin/plugins/goodlayers/ajax").into_response()
}
