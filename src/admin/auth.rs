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
<link rel="stylesheet" href="/admin/wp/atrium.css">
<script src="/admin/wp/jquery.min.js"></script>
</head><body class="login no-js login-action-login wp-core-ui admin-color-modern">
<script>document.body.className = document.body.className.replace('no-js','js');</script>
{body}
<script>var wp=window.wp||{{}};wp.i18n={{__:function(s){{return s;}},_x:function(s){{return s;}},sprintf:function(f){{var a=[].slice.call(arguments,1);var i=0;return f.replace(/%[sd]/g,function(){{return String(a[i++]);}});}}}};</script>
<script src="/admin/wp/clipboard.min.js"></script>
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
    // Subnet-keyed (`/24`/`/64`): evict-stalest alone lets a ≥10k-IP spray
    // evict its own entries and reset its counter to zero — a shared
    // prefix budget is the counter to that self-eviction.
    let key = intake::login_key(&ip);
    {
        let mut hits = app.login_hits.lock();
        let now = Instant::now();
        // Evict dead entries and cap the map — a rotating-source spray must
        // not grow memory unboundedly (A3). On full, evict the stalest key
        // rather than deny the newcomer (deny-new = attacker locks out all).
        if !hits.contains_key(&key) && hits.len() >= 10_000 {
            hits.retain(|_, v| v.iter().any(|t| now.duration_since(*t) < Duration::from_secs(60)));
            if hits.len() >= 10_000 {
                if let Some(old) = hits
                    .iter()
                    .min_by_key(|(_, v)| v.iter().max().copied())
                    .map(|(k, _)| k.clone())
                {
                    hits.remove(&old);
                }
            }
        }
        let e = hits.entry(key).or_default();
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
    // TOTP gate (phase 4): user with 2FA enabled gets a short-lived
    // pending token instead of a session — the second factor verifies at
    // /admin/2fa before anything is minted. The pending token is the only
    // path to a session from here; no alternate mint exists on this flow.
    if let Some((secret, enabled, last_step, _codes)) = udb::totp_get(&sc.db, id) {
        if enabled && !secret.is_empty() {
            let mut tok = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut tok);
            let tok = hex(&tok);
            let mut pend = app.pending_2fa.lock();
            // Cap: an attacker holding valid passwords (the exact threat
            // model 2FA exists for) must not grow this map unboundedly.
            if pend.len() >= 1000 {
                let now = Instant::now();
                pend.retain(|_, v| now.duration_since(v.1) < PENDING_TTL);
                if pend.len() >= 1000 {
                    if let Some(old) = pend.iter().min_by_key(|(_, v)| v.1).map(|(k, _)| k.clone()) {
                        pend.remove(&old);
                    }
                }
            }
            pend.insert(tok.clone(), (id, Instant::now(), 0usize));
            return Redirect::to(&format!("/admin/2fa?p={tok}")).into_response();
        }
        let _ = (secret, last_step);
    }
    mint_session(&app, sc, &headers, id, &form.log, &role, form.rememberme.as_deref() == Some("forever"), &ip)
}

/// TTL for a pending 2FA token — short enough that a stolen password
/// can't sit half-authenticated; long enough to open the app and type.
const PENDING_TTL: Duration = Duration::from_secs(300);

/// Shared session mint — login (post-password or post-TOTP) and future
/// paths funnel through here so cookie/session/db stay in lockstep.
fn mint_session(
    app: &App,
    sc: &SiteCtx,
    headers: &HeaderMap,
    id: i64,
    email_raw: &str,
    role: &str,
    forever: bool,
    ip: &str,
) -> Response {
    // Session invalidation on login (M20): the cookie's prior session dies
    // before the new one is issued — a stale cookie can never be pinned to
    // a different user, and re-login rotates the csrf.
    if let Some(old) = cookie(&headers, SESSION_COOKIE) {
        app.sessions.lock().remove(&old);
        let _ = udb::session_delete(&sc.db, &old);
    }
    let mut tok = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut tok);
    let tok = hex(&tok);
    let mut csrf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut csrf);
    let csrf = hex(&csrf);
    let ttl: u64 = if forever { 14 * 24 * 3600 } else { 12 * 3600 };
    app.sessions.lock().insert(
        tok.clone(),
        Sess {
            site: sc.slug.clone(),
            site_name: sc.name.clone(),
            user_id: id,
            email: email_raw.to_string(),
            role: role.to_string(),
            db: sc.db.clone(),
            csrf: csrf.clone(),
            exp: Instant::now() + Duration::from_secs(ttl),
            touched: Instant::now(),
        },
    );
    // Sessions persist to the `sessions` table (decision B) so the
    // user-edit Sessions screen and post-restart rehydration have data;
    // csrf rides the row so a mid-edit form survives a restart (M20).
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let _ = udb::session_put(&sc.db, &tok, id, &ua, ip, &csrf, ttl as i64);
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin")
        .header(
            header::SET_COOKIE,
            format!("{}={tok}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={ttl}", SESSION_COOKIE),
        )
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// GET `/admin/2fa?p=<pending>` — second-factor form. Invalid/expired
/// pending tokens get a bare redirect to /admin/login (no form), so the
/// endpoint can't be enumerated to learn whether an account has 2FA.
pub(crate) async fn two_fa_page(
    State(app): State<App>,
    axum::extract::Query(q): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let Some(p) = q.get("p") else { return Redirect::to("/admin/login").into_response() };
    let alive = app.pending_2fa.lock().get(p).map(|v| v.1.elapsed() < PENDING_TTL).unwrap_or(false);
    if !alive {
        return Redirect::to("/admin/login").into_response();
    }
    let body = format!(
        r##"<div id="login">
<h1 class="wp-login-logo"><a href="/admin" title="Two-factor authentication">Two-factor authentication</a></h1>
<form name="loginform" method="post" action="/admin/2fa">
<input type="hidden" name="p" value="{p}">
<p>
<label for="code">Authentication Code</label>
<input type="text" name="code" id="code" class="input" size="20" inputmode="numeric" pattern="[0-9A-Za-z-]*" autocomplete="one-time-code" autofocus>
</p>
<p class="description">6-digit code from your authenticator app, or a backup code (xxxx-xxxx).</p>
<p class="submit">
<input type="submit" class="button button-primary button-large" value="Verify">
</p>
</form>
</div>"##,
        p = esc(p)
    );
    html_login("Two-Factor Authentication", &body).into_response()
}

/// POST `/admin/2fa` — verify TOTP or backup code against the pending
/// token's user, then mint the real session. Atomic against concurrent
/// submits: the user row is re-read at verify (role may have changed
/// mid-flow), and backup-code consumption is a check-and-delete inside
/// one write tx — two parallel POSTs can't both consume the same code.
pub(crate) async fn two_fa_verify(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let p = f.get("p").cloned().unwrap_or_default();
    let code = f.get("code").cloned().unwrap_or_default();
    let sc = site_for(&app, &headers);
    let (user_id, fails) = {
        let pend = app.pending_2fa.lock();
        match pend.get(&p) {
            Some(v) if v.1.elapsed() < PENDING_TTL => (v.0, v.2),
            _ => {
                drop(pend);
                return Redirect::to("/admin/login").into_response();
            }
        }
    };
    if fails >= 5 {
        app.pending_2fa.lock().remove(&p);
        return Redirect::to("/admin/login").into_response();
    }
    let Some((secret, enabled, last_step, _)) = udb::totp_get(&sc.db, user_id) else {
        return Redirect::to("/admin/login").into_response();
    };
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // TOTP first (the normal path); backup codes are the recovery path —
    // consume is atomic so a raced double-submit can't both win.
    let ok = enabled
        && !secret.is_empty()
        && (crate_auth::totp_verify(&secret, &code, now_secs, last_step)
            .map(|step| {
                let _ = udb::totp_step_update(&sc.db, user_id, step);
            })
            .is_some()
            || udb::backup_code_consume(&sc.db, user_id, &sha256_hex(&code.replace(' ', ""))).unwrap_or(false));
    if !ok {
        app.pending_2fa.lock().entry(p.clone()).and_modify(|v| v.2 += 1);
        eprintln!("2fa fail user={user_id} ip={}", client_ip(&headers, Some(peer)));
        return Redirect::to(&format!("/admin/2fa?p={p}&err=1")).into_response();
    }
    // Verified — drop the pending token and mint as usual. `login_hits`
    // was already charged at the password step.
    app.pending_2fa.lock().remove(&p);
    let row: Option<(String, String)> = crate::db::conn(&sc.db)
        .ok()
        .and_then(|c| {
            c.query_row(sql!("users__select_email_role"), [user_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .ok()
        });
    let Some((email, role)) = row else {
        return Redirect::to("/admin/login").into_response();
    };
    let ip = client_ip(&headers, Some(peer));
    mint_session(&app, sc, &headers, user_id, &email, &role, false, &ip)
}

pub(crate) async fn logout(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, f.get("csrf").map(|x| x.as_str()).unwrap_or("")) {
        return r;
    }
    if let Some(tok) = cookie(&headers, SESSION_COOKIE) {
        app.sessions.lock().remove(&tok);
        let _ = udb::session_delete(&sc.db, &tok);
    }
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/login")
        .header(header::SET_COOKIE, CLEAR_SESSION_COOKIE)
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
/// `Sess` is returned — it is NOT inserted into `app.sessions`, so its csrf
/// can't be a stored value: it's HMAC(server_key, user_id:app_pass_hash),
/// deterministic per credential and stable across requests (M20).
pub(crate) fn app_pass_sess(sc: &SiteCtx, headers: &HeaderMap) -> Option<Sess> {
    let hv = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let blob = hv.strip_prefix("Basic ")?;
    let raw = b64_decode(blob)?;
    let creds = String::from_utf8(raw).ok()?;
    let (email, pass) = creds.split_once(':')?;
    let email = email.trim().to_lowercase();
    // WP generates `xxxx xxxx xxxx …`; clients may send it spaced or bare.
    let compact: String = pass.chars().filter(|c| !c.is_whitespace()).collect();
    let pass_hash = sha256_hex(&compact);
    let (ap_id, user_id, uemail, role) =
        udb::app_pass_lookup(&sc.db, &pass_hash, &email).ok().flatten()?;
    let _ = udb::app_pass_touch(&sc.db, ap_id, &client_ip(headers, None));
    let csrf = hmac_sha256_hex(&server_key(&sc.db), &format!("{user_id}:{pass_hash}"));
    Some(Sess {
        site: sc.slug.clone(),
        site_name: sc.name.clone(),
        user_id,
        email: uemail,
        role,
        db: sc.db.clone(),
        csrf,
        exp: Instant::now() + Duration::from_secs(12 * 3600),
        touched: Instant::now()
    })
}


