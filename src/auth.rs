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
