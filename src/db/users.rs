//! User data access — backs wp-admin users.php/user-new.php/user-edit.php.
//! All statements live in `src/db/sql/users__*.sql`.
use anyhow::Result;
use rusqlite::{params, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;

/// users table row (name columns added by the users-name migration).
#[allow(dead_code)] // full user row; admin screens read subsets
pub struct UserRow {
    pub id: i64,
    pub email: String,
    pub role: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub website: String,
    pub created_at: String,
}

/// users.php filter: `role` empty = all, `s` = search substring over
/// email + name columns (WP searches login/email/url/name).
pub struct Filter<'a> {
    pub role: &'a str,
    pub s: &'a str,
    pub limit: i64,
    pub offset: i64,
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<UserRow> {
    Ok(UserRow {
        id: r.get(0)?,
        email: r.get(1)?,
        role: r.get(2)?,
        display_name: r.get(3)?,
        first_name: r.get(4)?,
        last_name: r.get(5)?,
        website: r.get(6)?,
        created_at: r.get(7)?,
    })
}

/// role → count map for the `subsubsub` views bar.
pub fn counts_by_role(db: &Path) -> Result<HashMap<String, i64>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("users__count_by_role"))?;
    let mut out = HashMap::new();
    let mut rows = st.query([])?;
    while let Some(r) = rows.next()? {
        out.insert(r.get::<_, String>(0)?, r.get::<_, i64>(1)?);
    }
    Ok(out)
}

/// Total rows matching the filter — needed before building the pager.
pub fn count(db: &Path, role: &str, s: &str) -> Result<i64> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("users__count_filtered"), params![role, s], |r| r.get(0))?)
}

/// One page of filtered users, ordered by email (WP orders by login).
pub fn list(db: &Path, f: &Filter) -> Result<Vec<UserRow>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("users__select_filtered"))?;
    let rows = st.query_map(params![f.role, f.s, f.limit, f.offset], row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(db: &Path, id: i64) -> Result<Option<UserRow>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("users__select_by_id"), [id], row).optional()?)
}

/// Create a user; `pass_hash` comes from `crate::auth::hash_password`.
/// Fails on duplicate email (UNIQUE) — caller maps to an error notice.
pub fn create(
    db: &Path,
    email: &str,
    pass_hash: &str,
    role: &str,
    display_name: &str,
    first_name: &str,
    last_name: &str,
    website: &str,
) -> Result<i64> {
    let c = crate::db::conn(db)?;
    c.execute(
        sql!("users__insert"),
        params![email, pass_hash, role, display_name, first_name, last_name, website],
    )?;
    Ok(c.last_insert_rowid())
}

/// Profile/name/role update — never touches pass_hash (see `update_password`).
pub fn update(
    db: &Path,
    id: i64,
    email: &str,
    display_name: &str,
    first_name: &str,
    last_name: &str,
    website: &str,
    role: &str,
) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(
        sql!("users__update"),
        params![id, email, display_name, first_name, last_name, website, role],
    )?;
    Ok(())
}

pub fn update_password(db: &Path, id: i64, pass_hash: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__update_pass"), params![id, pass_hash])?;
    Ok(())
}

pub fn delete(db: &Path, id: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__delete"), [id])?;
    Ok(())
}

/// Count of `role='admin'` rows — feeds the last-admin hardening guard.
pub fn count_admins(db: &Path) -> Result<i64> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("users__count_admins"), [], |r| r.get(0))?)
}

pub fn change_role(db: &Path, id: i64, role: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__update_role"), params![id, role])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 4 additions: posts-count column, Application Passwords, Sessions (B),
// per-user prefs (color scheme / visual editor / toolbar).
// ---------------------------------------------------------------------------

/// users.php "Posts" column: published posts whose `post_meta.author_id`
/// equals this user's id. `posts` has no author column — authorship is a
/// meta row (see `misc::insert_draft`).
pub fn posts_count(db: &Path, user_id: i64) -> Result<i64> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("users__posts_count"), [user_id.to_string()], |r| r.get(0))?)
}

/// One `app_passwords` row for the Edit User table.
#[allow(dead_code)] // display fields kept for parity
pub struct AppPassRow {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub created_at: String,
    pub last_used: String,
    pub last_ip: String,
}

fn app_pass_row(r: &rusqlite::Row) -> rusqlite::Result<AppPassRow> {
    Ok(AppPassRow {
        id: r.get(0)?,
        user_id: r.get(1)?,
        name: r.get(2)?,
        created_at: r.get(3)?,
        last_used: r.get(4)?,
        last_ip: r.get(5)?,
    })
}

/// Store the SHA-256 hex of a new application password. Fails on
/// (user_id, name) duplicates — caller maps to an error notice.
pub fn app_pass_insert(db: &Path, user_id: i64, name: &str, hash: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__app_pass_insert"), params![user_id, name, hash])?;
    Ok(())
}

pub fn app_pass_list(db: &Path, user_id: i64) -> Result<Vec<AppPassRow>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("users__app_pass_list"))?;
    let rows = st.query_map([user_id], app_pass_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// `user_id` scoped so a crafted POST can't revoke another user's password.
pub fn app_pass_revoke(db: &Path, id: i64, user_id: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__app_pass_revoke"), params![id, user_id])?;
    Ok(())
}

/// `Authorization: Basic email:app_pass` lookup — `(app_pass_id, user_id,
/// email, role)` on a hash+email hit. SHA-256 is deterministic so the hash
/// is queryable directly (WP uses wp_check_password; spec mandates SHA-256).
pub fn app_pass_lookup(db: &Path, hash: &str, email: &str) -> Result<Option<(i64, i64, String, String)>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("users__app_pass_lookup"), params![hash, email], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    }).optional()?)
}

/// Stamp `last_used`/`last_ip` after a successful Basic-auth hit.
pub fn app_pass_touch(db: &Path, id: i64, ip: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__app_pass_touch"), params![id, ip])?;
    Ok(())
}

/// One `sessions` row for the user-edit Sessions table.
#[allow(dead_code)] // session row kept for parity
pub struct SessionRow {
    pub token: String,
    pub user_id: i64,
    pub ua: String,
    pub ip: String,
    pub created_at: String,
    pub last_seen: String,
}

fn session_row(r: &rusqlite::Row) -> rusqlite::Result<SessionRow> {
    Ok(SessionRow {
        token: r.get(0)?,
        user_id: r.get(1)?,
        ua: r.get(2)?,
        ip: r.get(3)?,
        created_at: r.get(4)?,
        last_seen: r.get(5)?,
    })
}

/// Record a login: token → user + UA + IP + per-session CSRF. `ttl_secs`
/// backs the remember-me window so the persisted row dies with the cookie.
/// `ON CONFLICT` refreshes `last_seen` (and rolls a fixed 12h expiry) so a
/// rehydrated token keeps one row; csrf is written only on insert — the
/// row's token is session identity, never regenerated in place.
pub fn session_put(db: &Path, token: &str, user_id: i64, ua: &str, ip: &str, csrf: &str, ttl_secs: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__session_put"), params![token, user_id, ua, ip, csrf, ttl_secs])?;
    Ok(())
}

pub fn session_list(db: &Path, user_id: i64) -> Result<Vec<SessionRow>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("users__session_list"))?;
    let rows = st.query_map([user_id], session_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Rehydrate a session after restart: `(token, user_id, email, role, csrf)` —
/// `None` when the token is unknown (expired/revoked/never persisted). csrf
/// is '' for rows written before the column existed — the caller mints and
/// persists a fresh one (see `session_set_csrf`).
pub fn session_get(db: &Path, token: &str) -> Result<Option<(String, i64, String, String, String)>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("users__session_get"), [token], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    }).optional()?)
}

/// Persist a csrf minted on rehydration of a pre-column ('' csrf) row, so the
/// same token keeps one stable csrf across further restarts.
pub fn session_set_csrf(db: &Path, token: &str, csrf: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute("UPDATE sessions SET csrf=?1 WHERE token=?2 AND COALESCE(csrf,'')=''", params![csrf, token])?;
    Ok(())
}

/// Refresh `last_seen` — cheap write on authed admin requests (decision B).
pub fn session_touch(db: &Path, token: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__session_touch"), [token])?;
    Ok(())
}

pub fn session_delete(db: &Path, token: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__session_delete"), [token])?;
    Ok(())
}

/// "Log Out Other Sessions" — keep `keep` alive, drop the rest.
pub fn session_delete_others(db: &Path, user_id: i64, keep: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__session_delete_others"), params![user_id, keep])?;
    Ok(())
}

/// User deleted → all their session rows go too.
pub fn session_delete_user(db: &Path, user_id: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__session_delete_user"), [user_id])?;
    Ok(())
}

/// Per-user admin prefs persisted as `settings.user_prefs_{user_id}` JSON —
/// same mechanism as `tax_screen_{user_id}_{tax}` (terms.rs).
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct UserPrefs {
    /// `admin-schemes.min.css` body-class suffix: modern/light/blue/…
    #[serde(default = "default_scheme")]
    pub color_scheme: String,
    /// WP `rich_editing` — unchecked disables TinyMCE on the classic editor.
    #[serde(default = "default_true")]
    pub rich_editing: bool,
    /// WP `admin_bar_front` — no public toolbar in this build (persisted
    /// for parity, read nowhere).
    #[serde(default = "default_true")]
    pub show_toolbar: bool,
}

fn default_scheme() -> String {
    "modern".into()
}
fn default_true() -> bool {
    true
}

/// Schemes vendored in `admin-schemes.min.css` — anything else falls back
/// to `modern` so a tampered pref can't inject a class.
pub const SCHEMES: &[(&str, &str)] = &[
    ("modern", "Default"),
    ("light", "Light"),
    ("blue", "Blue"),
    ("midnight", "Midnight"),
    ("sunrise", "Sunrise"),
    ("ectoplasm", "Ectoplasm"),
    ("ocean", "Ocean"),
    ("coffee", "Coffee"),
];

pub fn user_prefs(db: &Path, user_id: i64) -> UserPrefs {
    crate::db::conn(db)
        .ok()
        .and_then(|c| {
            c.query_row(
                sql!("terms__select_screen_setting"),
                [format!("user_prefs_{user_id}")],
                |r| r.get::<_, String>(0),
            )
            .ok()
        })
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or(UserPrefs { color_scheme: default_scheme(), rich_editing: true, show_toolbar: true })
}

pub fn save_user_prefs(db: &Path, user_id: i64, prefs: &UserPrefs) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(
        sql!("terms__upsert_screen_setting"),
        params![format!("user_prefs_{user_id}"), serde_json::to_string(prefs)?],
    )?;
    Ok(())
}

/// Orphan cleanup on user delete (FK declared but SQLite pragmas aren't
/// relied on for cascades) — wired from `admin::users_delete` so a
/// re-created same-email account never inherits dead credentials.
pub fn app_pass_delete_user(db: &Path, user_id: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("users__app_pass_delete_user"), [user_id])?;
    Ok(())
}
