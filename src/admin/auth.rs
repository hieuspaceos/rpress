#[allow(unused_imports)]
use crate::sql;
use crate::db::users as udb;
use sha2::{Digest, Sha256};
use super::*;
pub(crate) fn html_login(title: &str, body: &str) -> Html<String> {
    Html(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>{title}</title>
<link rel="stylesheet" href="/admin/wp/dashicons.min.css">
<link rel="stylesheet" href="/admin/wp/buttons.min.css">
<link rel="stylesheet" href="/admin/wp/forms.min.css">
<link rel="stylesheet" href="/admin/wp/l10n.min.css">
<link rel="stylesheet" href="/admin/wp/admin-schemes.min.css">
<link rel="stylesheet" href="/admin/wp/wp-tooltip.min.css">
<link rel="stylesheet" href="/admin/wp/login.min.css">
<script src="/admin/wp/jquery.min.js"></script>
</head><body class="login no-js login-action-login wp-core-ui admin-color-modern">
<script>document.body.className = document.body.className.replace('no-js','js');</script>
{body}
<script src="/admin/wp/user-profile.min.js"></script>
</body></html>"#
    ))
}

pub(crate) fn login_html(sc: &SiteCtx, err: &str) -> Html<String> {
    let err = if err.is_empty() {
        String::new()
    } else {
        format!(r#"<div id="login_error"><strong>Error:</strong> {}</div>"#, esc(err))
    };
    let home = esc(&site_url(sc));
    let name = esc(&sc.name);
    html_login(
        &format!("Log In ‹ {name}"),
        &format!(
            r#"<div id="login">
<h1 class="wp-login-logo"><a href="{home}" title="{name}">{name}</a></h1>
{err}
<form name="loginform" id="loginform" action="/api/login" method="post">
<p>
<label for="user_login">Username or Email Address</label>
<input type="text" name="log" id="user_login" class="input" size="20" autocapitalize="off" autocomplete="username" required autofocus>
</p>
<div class="user-pass-wrap">
<label for="user_pass">Password</label>
<div class="wp-pwd">
<input type="password" name="pwd" id="user_pass" class="input password-input" size="20" autocomplete="current-password" required>
<button type="button" class="button button-secondary wp-hide-pw hide-if-no-js" data-toggle="0" aria-label="Show password">
<span class="dashicons dashicons-visibility" aria-hidden="true"></span>
</button>
</div>
</div>
<p class="forgetmenot"><input name="rememberme" type="checkbox" id="rememberme" value="forever"> <label for="rememberme">Remember Me</label></p>
<p class="submit">
<input type="submit" name="wp-submit" id="wp-submit" class="button button-primary button-large" value="Log In">
</p>
</form>
<p id="backtoblog"><a href="{home}">← Go to {name}</a></p>
</div>
<div class="language-switcher">
<form id="language-switcher" action="/admin/login" method="get">
<label for="language-switcher-locales"><span class="dashicons dashicons-translation" aria-hidden="true"></span><span class="screen-reader-text">Language</span></label>
<select name="wp_lang" id="language-switcher-locales"><option value="en_US" lang="en" selected>English (United States)</option></select>
<input type="submit" class="button" value="Change">
</form>
</div>"#
        ),
    )
}

pub(crate) async fn login_page(State(app): State<App>, headers: HeaderMap) -> Html<String> {
    let sc = site_for(&app, &headers);
    login_html(sc, "")
}

#[derive(Deserialize)]
pub(crate) struct LoginForm {
    /// WP field `log`; value is CMS `users.email`.
    log: String,
    pwd: String,
    /// WP `rememberme` — "forever" extends session + cookie to 14 days.
    #[serde(default)]
    rememberme: Option<String>,
}

/// Argon2 hash verified on the unknown-email path so the login attempt has
/// the same cost shape as a real account (built once — it's never the
/// correct hash, only a timing decoy).
fn dummy_hash() -> &'static str {
    static DUMMY: std::sync::LazyLock<String> =
        std::sync::LazyLock::new(|| crate_auth::hash_password("dummy-password").unwrap_or_default());
    &DUMMY
}
pub(crate) async fn login(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Response {
    let sc = site_for(&app, &headers);
    let email = form.log.trim().to_lowercase();
    let password = form.pwd.trim().to_string();
    let ip = client_ip(&headers, Some(peer));
    {
        let mut hits = app.login_hits.lock();
        let now = Instant::now();
        // Evict dead entries and cap the map — a rotating-source spray must
        // not grow memory unboundedly (A3).
        if !hits.contains_key(&ip) && hits.len() >= 10_000 {
            hits.retain(|_, v| v.iter().any(|t| now.duration_since(*t) < Duration::from_secs(60)));
            if hits.len() >= 10_000 {
                return login_html(sc, "Too many attempts, wait a minute").into_response();
            }
        }
        let e = hits.entry(ip.clone()).or_default();
        e.retain(|t| now.duration_since(*t) < Duration::from_secs(60));
        if e.len() >= 20 {
            return login_html(sc, "Too many attempts, wait a minute").into_response();
        }
        e.push(now);
    }
    let conn = match crate::db::conn(&sc.db) {
        Ok(c) => c,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let row: Option<(i64, String, String)> = conn
        .query_row(
            sql!("auth__select_id_pass_hash"),
            [&email],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok();
    let Some((id, hash, role)) = row else {
        // Constant-shape timing (A14): unknown emails still pay one argon2
        // verify so the response time doesn't reveal account existence.
        let _ = crate_auth::verify_password(dummy_hash(), &password);
        eprintln!("login fail email={} ip={}", email, ip);
        return login_html(sc, "Invalid credentials").into_response();

    };
    if !crate_auth::verify_password(&hash, &password) {
        eprintln!("login fail email={} ip={}", email, ip);
        return login_html(sc, "Invalid credentials").into_response();

    }
    let mut tok = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut tok);
    let tok = hex(&tok);
    let mut csrf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut csrf);
    let csrf = hex(&csrf);
    let ttl: u64 = if form.rememberme.as_deref() == Some("forever") {
        14 * 24 * 3600
    } else {
        12 * 3600
    };
    app.sessions.lock().insert(
        tok.clone(),
        Sess {
            site: sc.slug.clone(),
            site_name: sc.name.clone(),
            user_id: id,
            email: form.log.clone(),

            role,
            db: sc.db.clone(),
            csrf,
            exp: Instant::now() + Duration::from_secs(ttl),
        },
    );
    // Sessions persist to the `sessions` table (decision B) so the
    // user-edit Sessions screen and post-restart rehydration have data.
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let _ = udb::session_put(&sc.db, &tok, id, &ua, &ip);
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin")
        .header(
            header::SET_COOKIE,
            format!("cms_session={tok}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={ttl}"),
        )
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub(crate) async fn logout(State(app): State<App>, headers: HeaderMap) -> Response {
    if let Some(tok) = cookie(&headers, "cms_session") {
        app.sessions.lock().remove(&tok);
        let _ = udb::session_delete(&site_for(&app, &headers).db, &tok);
    }
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/login")
        .header(
            header::SET_COOKIE,
            "cms_session=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0",
        )
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// SHA-256 hex of `raw` — Application Password hashes (spec: SHA-256, not
/// argon2 — the value is already 24 random chars, so a fast hash suffices
/// and must stay deterministic for the indexed `app_passwords.hash` lookup).
pub(crate) fn sha256_hex(raw: &str) -> String {
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Minimal RFC 4648 base64 decoder (no padding required) — just enough for
/// the `Authorization: Basic` credential blob; no base64 crate in deps.
fn b64_decode(s: &str) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes: Vec<u8> = s.bytes().filter(|b| *b != b'=' && !b.is_ascii_whitespace()).collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    let (mut acc, mut nbits) = (0u32, 0u32);
    for b in bytes {
        acc = (acc << 6) | val(b)?;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            out.push((acc >> nbits) as u8);
        }
    }
    Some(out)
}

/// WP Application Passwords fallback for `need_auth`: `Authorization: Basic
/// base64(email:xxxx xxxx …)` → SHA-256(spaces stripped) → `app_passwords`
/// lookup. On a hit, `last_used`/`last_ip` are stamped and a transient
/// `Sess` is returned — it is NOT inserted into `app.sessions` and carries
/// a fresh per-request csrf, so mutating POSTs still need a real session.
pub(crate) fn app_pass_sess(sc: &SiteCtx, headers: &HeaderMap) -> Option<Sess> {
    let hv = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let blob = hv.strip_prefix("Basic ")?;
    let raw = b64_decode(blob)?;
    let creds = String::from_utf8(raw).ok()?;
    let (email, pass) = creds.split_once(':')?;
    let email = email.trim().to_lowercase();
    // WP generates `xxxx xxxx xxxx …`; clients may send it spaced or bare.
    let compact: String = pass.chars().filter(|c| !c.is_whitespace()).collect();
    let (ap_id, user_id, uemail, role) =
        udb::app_pass_lookup(&sc.db, &sha256_hex(&compact), &email).ok().flatten()?;
    let _ = udb::app_pass_touch(&sc.db, ap_id, &client_ip(headers, None));
    let mut csrf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut csrf);
    Some(Sess {
        site: sc.slug.clone(),
        site_name: sc.name.clone(),
        user_id,
        email: uemail,
        role,
        db: sc.db.clone(),
        csrf: hex(&csrf),
        exp: Instant::now() + Duration::from_secs(12 * 3600),
    })
}

// Dashboard widgets live in `dashboard.rs`, textually included here so the
// file split lands without touching `mod.rs` — promote to `mod dashboard;`
// (route paths become `dashboard::…`) when the router may be edited.
include!("dashboard.rs");

