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
pub(super) async fn plugin_screen_root(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    req: axum::extract::Request,
) -> Response {
    plugin_screen(State(app), ConnectInfo(peer), headers, Path((slug, String::new())), req).await
}

pub(super) async fn plugin_screen(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path((slug, rest)): Path<(String, String)>,
    req: axum::extract::Request,
) -> Response {
    let Ok((s, sc)) = need_auth_peer(&app, &headers, Some(peer)).await else {
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
    // C1: adapter screen handlers do sync SQLite (tourmaster save-option
    // tx) inside an async fn — run the whole future on a blocking thread.
    // Nothing inside awaits IO, so it runs start-to-finish there; owned
    // clones move in because the fn borrows app/s/sc.
    let (app2, s2, sc2) = (app.clone(), s.clone(), sc.clone());
    tokio::task::spawn_blocking(move || {
        let fut = (screen.handle)(&app2, &s2, &sc2, headers, &rest, req);
        futures_lite::block_on(fut)
    })
    .await
    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
/// `block_on` for the screen-handler offload above — std-only, no futures dep.
mod futures_lite {
    use std::future::Future;
    use std::pin::pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn noop_raw_waker() -> RawWaker {
        fn no_op(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            noop_raw_waker()
        }
        RawWaker::new(
            std::ptr::null(),
            &RawWakerVTable::new(clone, no_op, no_op, no_op),
        )
    }
    /// Drive a future that never yields IO to completion — screen handlers
    /// are sync work wrapped in `async fn`, so one poll finishes them.
    /// Panics if the inner future pends (that would mean a real .await
    /// slipped in and must run on a worker, not here).
    pub(crate) fn block_on<T>(fut: impl Future<Output = T>) -> T {
        let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
        let mut cx = Context::from_waker(&waker);
        let mut fut = pin!(fut);
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => v,
            Poll::Pending => panic!("adapter screen handler yielded — needs a real runtime"),
        }
    }
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
    let query = req.uri().query().map(str::to_owned);
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
    let mut path = adapter.bridge_path(rest);
    if let Some(q) = query.as_deref().filter(|q| !q.is_empty()) {
        path.push('?');
        path.push_str(q);
    }
    let bridges = sc.bridges.clone();
    // post= / p= query (Elementor post.php / preview) or an edit/post/<id>
    // tail segment (GDLR edit screens). Any other numeric tail — asset
    // URLs like uploads/2025/11/x.jpg — must NOT trigger a reseed.
    let post_id = query
        .as_deref()
        .and_then(|q| {
            q.split('&').find_map(|pair| {
                pair.strip_prefix("post=")
                    .or_else(|| pair.strip_prefix("p="))
                    .and_then(|v| v.parse().ok())
            })
        })
        .or_else(|| {
            let mut segs = rest.rsplit('/');
            let tail = segs.next()?;
            let head = segs.next()?;
            matches!(head, "edit" | "post")
                .then(|| tail.parse::<i64>().ok())
                .flatten()
        });
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
        // Post switch on a live bridge: respawning per post would cost a
        // full WP install each time; reseed instead (post.json + clear the
        // seeded flag; the mu-plugin re-inserts on this very request).
        if let Some(pid) = post_id {
            if st.seeded_post != Some(pid) {
                match bridge::reseed_post(&www, slug, &st.wp_db, &db, pid) {
                    // Only claim seeded when the file+flag actually landed;
                    // false (absent post / non-CGI runtime) leaves the
                    // marker so a later request can retry or respawn.
                    Ok(true) => st.seeded_post = Some(pid),
                    Ok(false) => {}
                    Err(e) => eprintln!("bridge reseed {site}/{slug} post {pid}: {e:#}"),
                }
            }
        }
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
        // HTML/JSON/JS: absolute WP_HOME → mount, then root-relative
        // `/wp-admin/` `/wp-json/` (Elementor ajaxurl). GDLR ajax JSON
        // does not contain those prefixes — rewrite is a no-op there.
        let body = {
            let ct = r.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-type")).map(|(_, v)| v.as_str()).unwrap_or("");
            if ct.contains("text/html") || ct.contains("json") || ct.contains("javascript") {
                let upstream_esc = upstream.replace('/', "\\/");
                let mount_esc = mount.replace('/', "\\/");
                let mut s = String::from_utf8_lossy(&r.body).into_owned();
                s = s.replace(&format!("{upstream}/wp-content/"), "/wp-content/");
                s = s.replace(&format!("{upstream}/wp-includes/"), "/wp-includes/");
                s = s.replace(&format!("{upstream_esc}\\/wp-content\\/"), "\\/wp-content\\/");
                s = s.replace(&format!("{upstream_esc}\\/wp-includes\\/"), "\\/wp-includes\\/");
                s = s.replace(&upstream, &mount).replace(&upstream_esc, &mount_esc);
                s = s.replace("\"/wp-admin/", &format!("\"{mount}/wp-admin/"));
                s = s.replace("'/wp-admin/", &format!("'{mount}/wp-admin/"));
                s = s.replace("\\/wp-admin\\/", &format!("{mount_esc}\\/wp-admin\\/"));
                s = s.replace("\"/wp-json/", &format!("\"{mount}/wp-json/"));
                s = s.replace("'/wp-json/", &format!("'{mount}/wp-json/"));
                s = s.replace("\\/wp-json\\/", &format!("{mount_esc}\\/wp-json\\/"));
                collapse_doubled_mount(&s, &mount, &mount_esc).into_bytes()
            } else {
                r.body
            }
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
/// Collapse `{mount}{mount}` and `{mount_esc}{mount_esc}` pairs left by
/// the URL rewrites above — artifacts of `/wp-admin/` matching inside an
/// already-mounted URL. A pair collapses only when followed by `/wp-`
/// (or `\/wp-` in the escaped pass), a JSON delimiter, or EOS — never
/// two legit adjacent mount URLs.
fn collapse_doubled_mount(s: &str, mount: &str, mount_esc: &str) -> String {
    let pass = |s: &str, twice: &str, single: &str, wp_tail: &str| -> String {
        let mut out = String::with_capacity(s.len());
        let mut rest_s = s;
        while let Some(i) = rest_s.find(twice) {
            out.push_str(&rest_s[..i]);
            let after = &rest_s[i + twice.len()..];
            if after.is_empty()
                || after.starts_with(wp_tail)
                || after.starts_with(['?', '&', '"', '\'', ')', ']', '}', ','])
            {
                out.push_str(single);
            } else {
                out.push_str(twice);
            }
            rest_s = after;
        }
        out.push_str(rest_s);
        out
    };
    let s = pass(s, &format!("{mount}{mount}"), mount, "/wp-");
    pass(&s, &format!("{mount_esc}{mount_esc}"), mount_esc, "\\/wp-")
}

#[cfg(test)]
mod tests {
    #[test]
    fn collapse_artifacts() {
        let m = "/admin/plugins/elementor";
        let me = "\\/admin\\/plugins\\/elementor";
        // Plain doubled mount before a wp path collapses.
        assert_eq!(
            super::collapse_doubled_mount(&format!("{m}{m}/wp-admin/x"), m, me),
            format!("{m}/wp-admin/x")
        );
        // Escaped doubled mount before `\/wp-` collapses — the bug this
        // pass shipped with expected `\\/wp-` (double-backslash).
        assert_eq!(
            super::collapse_doubled_mount(&format!("{me}{me}\\/wp-admin\\/x"), m, me),
            format!("{me}\\/wp-admin\\/x")
        );
        // Legit adjacent mounts (followed by non-wp text) stay.
        assert_eq!(
            super::collapse_doubled_mount(&format!("{m}{m}/edit.php"), m, me),
            format!("{m}{m}/edit.php")
        );
        // JSON delimiter followers collapse.
        for tail in ["\"}", "\",", "?x=1", "']"] {
            assert_eq!(
                super::collapse_doubled_mount(&format!("{m}{m}{tail}"), m, me),
                format!("{m}{tail}"), "tail {tail}"
            );
        }
    }
}
