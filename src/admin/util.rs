//! Shared admin utilities: HTML escaping, percent-encoding, taxonomy
//! labels, and the small crypto helpers (constant-time compare, HMAC,
//! per-site server key) used across every admin screen.
use anyhow::Result;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rand::RngCore;

use super::Sess;
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
pub(crate) fn server_key(db: &std::path::Path) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    let key = "server_key";
    if let Ok(c) = crate::db::conn(db) {
        if let Ok(v) = c.query_row(sql!("intake__select_setting"), [key], |r| r.get::<_, String>(0)) {
            if !v.is_empty() {
                return v;
            }
        }
        let mut b = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut b);
        let v = hex(&b);
        let _ = c.execute(
            sql!("settings__insert_or_ignore"),
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

/// Human label for a taxonomy (shared with tax.rs). Theme-claimed
/// taxonomies override via `ThemeDef::tax_labels`; core + generic
/// taxonomies come from the builtin table.
pub(crate) fn tax_label_for(theme: &crate::plugins::ThemeDef, tax: &str) -> (String, String) {
    if let Some((pl, sg)) = crate::plugins::theme_tax_label(theme, tax) {
        return (pl.to_string(), sg.to_string());
    }
    tax_label(tax)
}

/// Core-taxonomy labels — `tax_label_for` wraps this with the theme's
/// `tax_labels` table when a site/theme conn is available.
pub(crate) fn tax_label(tax: &str) -> (String, String) {
    match tax {
        "category" => ("Categories".into(), "Category".into()),
        "post_tag" => ("Tags".into(), "Tag".into()),
        "style" => ("Style".into(), "Style".into()),
        "tour-activity" => ("Tour Activity".into(), "Tour Activity".into()),
        "tour-age" => ("Tour Age".into(), "Tour Age".into()),
        _ => (tax.replace('-', " "), tax.replace('-', " ")),
    }
}


pub(crate) use crate::html_decode::{esc, esc_attr};

pub(crate) fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
