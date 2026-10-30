use anyhow::{bail, Context, Result};
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rusqlite::params;
use std::path::Path;

pub fn add_user(db: &Path, email: &str, password: &str, role: &str) -> Result<()> {
    if !matches!(role, "editor" | "admin") {
        bail!("role must be editor|admin");
    }
    if password.len() < 8 {
        bail!("password must be ≥8 chars");
    }
    let conn = crate::db::conn(db)?;
    let hash = hash_password(password)?;
    conn.execute(
        "INSERT INTO users(email, pass_hash, role, created_at) VALUES(?1,?2,?3,datetime('now'))",
        params![email, hash, role],
    )
    .with_context(|| format!("insert user {email}"))?;
    println!("added {email} role={role}");
    Ok(())
}

pub fn set_password(db: &Path, email: &str, password: &str) -> Result<()> {
    let conn = crate::db::conn(db)?;
    let hash = hash_password(password)?;
    let n = conn.execute(
        "UPDATE users SET pass_hash=?1 WHERE email=?2",
        params![hash, email],
    )?;
    if n == 0 {
        bail!("no user {email}");
    }
    println!("updated password for {email}");
    Ok(())
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("argon2: {e}"))?
        .to_string())
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// --- TOTP (RFC 6238, HMAC-SHA-256) --------------------------------------
/// No otpauth crate: the math is ~30 lines on top of the project's inline
/// HMAC-SHA256. `algorithm=SHA256` in the otpauth URI is honoured by
/// Google Authenticator, Aegis, 1Password, Bitwarden (SHA-1 is the
/// historical default; apps that only do SHA-1 won't pair — fall back to
/// sha1+hmac micro-deps if a target app ever fails).

/// base32 (RFC 4648, no padding) decode for TOTP secrets.
pub fn base32_decode(s: &str) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'2'..=b'7' => Some((c - b'2' + 26) as u32),
            _ => None,
        }
    }
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for &c in s.trim().replace('=', "").as_bytes() {
        acc = (acc << 5) | val(c.to_ascii_uppercase())?;
        bits += 5;
        if bits >= 8 {
            out.push((acc >> (bits - 8)) as u8);
            bits -= 8;
        }
    }
    Some(out)
}

/// base32 encode (no padding) — for showing the secret to the user.
pub fn base32_encode(b: &[u8]) -> String {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut out = String::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for &x in b {
        acc = (acc << 8) | x as u32;
        bits += 8;
        while bits >= 5 {
            out.push(A[((acc >> (bits - 5)) & 31) as usize] as char);
            bits -= 5;
        }
    }
    if bits > 0 {
        out.push(A[((acc << (5 - bits)) & 31) as usize] as char);
    }
    out
}

fn hmac_sha256_bytes(key: &[u8], msg: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut k = key.to_vec();
    if k.len() > 64 {
        k = Sha256::digest(&k).to_vec();
    }
    k.resize(64, 0);
    let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();
    let mut h = Sha256::new();
    h.update(&ipad);
    h.update(msg);
    let inner = h.finalize();
    let mut h = Sha256::new();
    h.update(&opad);
    h.update(inner);
    let r = h.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&r);
    out
}

/// TOTP code at a 30-second step (RFC 6238 dynamic truncation).
fn totp_at(secret: &[u8], step: u64) -> u32 {
    let m = hmac_sha256_bytes(secret, &step.to_be_bytes());
    let off = (m[31] & 0x0f) as usize;
    (u32::from_be_bytes([m[off], m[off + 1], m[off + 2], m[off + 3]]) & 0x7fffffff) % 1_000_000
}

/// Verify a 6-digit user code against `secret_b32` at the current time,
/// drift ±1 step, replay-gated by `last_step` (the largest step already
/// consumed). Returns the winning step to persist, or None.
pub fn totp_verify(secret_b32: &str, code: &str, now_secs: u64, last_step: i64) -> Option<i64> {
    let secret = base32_decode(secret_b32)?;
    let code: u32 = code.trim().parse().ok()?;
    let step = now_secs / 30;
    for s in [step, step.wrapping_sub(1), step + 1] {
        if s as i64 > last_step && totp_at(&secret, s) == code {
            return Some(s as i64);
        }
    }
    None
}

/// Fresh 160-bit TOTP secret, base32'd (authenticator-app standard).
pub fn totp_secret_new() -> String {
    let mut b = [0u8; 20];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut b);
    base32_encode(&b)
}

/// 10 one-time backup codes `xxxx-xxxx` → (display Vec, JSON array of
/// sha256 hashes for storage).
pub fn backup_codes_new() -> (Vec<String>, String) {
    use sha2::{Digest, Sha256};
    let mut codes = Vec::new();
    let mut hashes = Vec::new();
    for _ in 0..10 {
        let mut b = [0u8; 4];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut b);
        let c = format!("{:04x}-{:04x}", u16::from_be_bytes([b[0], b[1]]), u16::from_be_bytes([b[2], b[3]]));
        let h = Sha256::digest(c.as_bytes());
        hashes.push(h.iter().map(|x| format!("{x:02x}")).collect::<String>());
        codes.push(c);
    }
    (codes, serde_json::to_string(&hashes).unwrap_or_default())
}
