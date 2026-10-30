use anyhow::Result;
use rusqlite::params;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn setting_value(db: &Path, key: &str, default: &str) -> String {
    crate::db::conn(db)
        .ok()
        .and_then(|c| c.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| r.get::<_, String>(0)).ok())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default.to_string())
}

pub fn setting_put(db: &Path, key: &str, value: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(
        "INSERT OR REPLACE INTO settings(key, value) VALUES(?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

pub fn setting_json_value_conn(conn: &rusqlite::Connection, key: &str) -> serde_json::Value {
    conn.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| r.get::<_, String>(0))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null)
}

pub fn setting_json_put_conn(conn: &rusqlite::Connection, key: &str, value: &serde_json::Value) -> Result<()> {
    conn.execute(
        "INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value.to_string()],
    )?;
    Ok(())
}

/// Namespaced settings key for plugin options: `plugin:<slug>:<key>`.
/// The `plugin:` prefix is the write-whitelist boundary the PhpBridge
/// write path (phase 06) enforces mechanically — plugin writes can never
/// touch `posts`, `users`, `sessions`, core `settings` keys, or another
/// plugin's namespace.
pub fn plugin_setting_key(slug: &str, key: &str) -> String {
    format!("plugin:{slug}:{key}")
}

/// Read a plugin option: `plugin:<slug>:<key>` first, legacy bare `key`
/// fallback (transition window until phase 05 migrates existing keys).
pub fn plugin_setting_json_value_conn(conn: &rusqlite::Connection, slug: &str, key: &str) -> serde_json::Value {
    let namespaced = plugin_setting_key(slug, key);
    let get = |k: &str| conn.query_row("SELECT value FROM settings WHERE key=?1", [k], |r| r.get::<_, String>(0));
    // Fall back to the legacy bare key only on a clean miss — a transient
    // error (BUSY/IO) must not make a live option look absent.
    let raw = match get(&namespaced) {
        Err(rusqlite::Error::QueryReturnedNoRows) => get(key),
        other => other,
    };
    raw.ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null)
}

/// Write a plugin option under its namespace — new writes never touch
/// legacy bare keys.
pub fn plugin_setting_json_put_conn(conn: &rusqlite::Connection, slug: &str, key: &str, value: &serde_json::Value) -> Result<()> {
    setting_json_put_conn(conn, &plugin_setting_key(slug, key), value)
}

/// Raw-string variant for non-JSON option blobs (php-serialized values
/// like `tourmaster_general`). Same dual-read order.
pub fn plugin_setting_raw_conn(conn: &rusqlite::Connection, slug: &str, key: &str) -> Option<String> {
    let namespaced = plugin_setting_key(slug, key);
    let get = |k: &str| conn.query_row("SELECT value FROM settings WHERE key=?1", [k], |r| r.get::<_, String>(0));
    match get(&namespaced) {
        Err(rusqlite::Error::QueryReturnedNoRows) => get(key),
        other => other,
    }
    .ok()
}

/// `BTreeMap` variant for map-shaped plugin options (tourmaster's
/// `tour_custom_taxs` slug→label dict). Same dual-read order.
pub fn plugin_setting_json_map(db: &Path, slug: &str, key: &str) -> std::collections::BTreeMap<String, String> {
    crate::db::conn(db)
        .ok()
        .and_then(|c| {
            let namespaced = plugin_setting_key(slug, key);
            let get = |k: &str| c.query_row("SELECT value FROM settings WHERE key=?1", [k], |r| r.get::<_, String>(0));
            match get(&namespaced) {
                Err(rusqlite::Error::QueryReturnedNoRows) => get(key),
                other => other,
            }
            .ok()
        })
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
}

/// Map-shaped plugin-option write — namespaced key only.
pub fn plugin_setting_json_put_map(db: &Path, slug: &str, key: &str, value: &std::collections::BTreeMap<String, String>) -> Result<()> {
    setting_put(db, &plugin_setting_key(slug, key), &serde_json::to_string(value)?)
}


/// One idempotent settings-key fold for an adapter's `legacy_option_keys`:
/// copies each bare `key` to `plugin:<slug>:<key>` when the namespaced row
/// doesn't exist yet. Legacy rows are never deleted — the dual-read
/// helpers prefer the namespaced copy, and a rollback reverts cleanly.
/// Skips keys already `plugin:`-prefixed (partial earlier runs).
pub fn migrate_plugin_keys(db: &Path, slug: &str, keys: &[&str]) -> Result<()> {
    let c = crate::db::conn(db)?;
    let tx = c.unchecked_transaction()?;
    for key in keys {
        if key.starts_with("plugin:") {
            continue;
        }
        tx.execute(
            "INSERT OR IGNORE INTO settings(key, value)
             SELECT ?1, value FROM settings WHERE key = ?2",
            rusqlite::params![plugin_setting_key(slug, key), key],
        )?;
    }
    Ok(tx.commit()?)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> rusqlite::Connection {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute("CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT)", []).unwrap();
        c
    }

    #[test]
    fn plugin_settings_dual_read() {
        let c = conn();
        // Namespaced write lands under plugin:{slug}:, never the bare key.
        plugin_setting_json_put_conn(&c, "goodlayers", "skins", &serde_json::json!({"a":1})).unwrap();
        assert_eq!(
            c.query_row::<String, _, _>("SELECT value FROM settings WHERE key='plugin:goodlayers:skins'", [], |r| r.get(0)).unwrap(),
            r#"{"a":1}"#
        );
        assert!(c.query_row::<String, _, _>("SELECT value FROM settings WHERE key='skins'", [], |r| r.get(0)).is_err());
        // Read order: namespaced wins; bare key only as fallback.
        c.execute("INSERT INTO settings(key,value) VALUES('gdlr-x','\"legacy\"')", []).unwrap();
        assert_eq!(plugin_setting_json_value_conn(&c, "goodlayers", "gdlr-x"), serde_json::json!("legacy"));
        c.execute("INSERT INTO settings(key,value) VALUES('plugin:goodlayers:gdlr-x','\"new\"')", []).unwrap();
        assert_eq!(plugin_setting_json_value_conn(&c, "goodlayers", "gdlr-x"), serde_json::json!("new"));
        // Raw variant for non-JSON blobs (php-serialized options).
        c.execute("INSERT INTO settings(key,value) VALUES('tourmaster_general','a:1:{}')", []).unwrap();
        assert_eq!(plugin_setting_raw_conn(&c, "tourmaster", "tourmaster_general").as_deref(), Some("a:1:{}"));
        assert!(plugin_setting_raw_conn(&c, "tourmaster", "missing").is_none());
    }

    #[test]
    fn migrate_plugin_keys_fold() {
        let dir = std::env::temp_dir().join(format!("rpress-mig-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("t.db");
        let c = rusqlite::Connection::open(&db).unwrap();
        c.execute("CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT)", []).unwrap();
        c.execute("INSERT INTO settings VALUES('tourmaster_general','a:1:{}')", []).unwrap();
        // A partially-namespaced key and an unrelated key must survive.
        c.execute("INSERT INTO settings VALUES('plugin:tourmaster:tour_custom_taxs','{\"a\":\"b\"}')", []).unwrap();
        c.execute("INSERT INTO settings VALUES('site_url','http://x')", []).unwrap();
        c.execute("INSERT INTO settings VALUES('plugin:other:x','1')", []).unwrap();
        drop(c);

        migrate_plugin_keys(&db, "tourmaster", &["tourmaster_general", "tour_custom_taxs"]).unwrap();
        let c = rusqlite::Connection::open(&db).unwrap();
        let n: i64 = c.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 5); // 4 old + 1 folded copy (taxs legacy row absent → nothing to fold)
        // Legacy row kept; namespaced copy identical.
        assert_eq!(
            c.query_row::<String, _, _>("SELECT value FROM settings WHERE key='tourmaster_general'", [], |r| r.get(0)).unwrap(),
            "a:1:{}"
        );
        assert_eq!(
            c.query_row::<String, _, _>("SELECT value FROM settings WHERE key='plugin:tourmaster:tourmaster_general'", [], |r| r.get(0)).unwrap(),
            "a:1:{}"
        );
        // Existing namespaced row never overwritten.
        assert_eq!(
            c.query_row::<String, _, _>("SELECT value FROM settings WHERE key='plugin:tourmaster:tour_custom_taxs'", [], |r| r.get(0)).unwrap(),
            "{\"a\":\"b\"}"
        );
        // Idempotent second run.
        drop(c);
        migrate_plugin_keys(&db, "tourmaster", &["tourmaster_general"]).unwrap();
        let c = rusqlite::Connection::open(&db).unwrap();
        let n2: i64 = c.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0)).unwrap();
        assert_eq!(n2, 5);
        drop(c);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

pub fn setting_json_map(db: &Path, key: &str) -> std::collections::BTreeMap<String, String> {
    crate::db::conn(db)
        .ok()
        .and_then(|c| {
            c.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| r.get::<_, String>(0))
                .ok()
        })
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
}

pub fn setting_json_put_map(db: &Path, key: &str, value: &std::collections::BTreeMap<String, String>) -> Result<()> {
    setting_put(db, key, &serde_json::to_string(value)?)
}

pub fn widget_next_ord(conn: &rusqlite::Connection, sidebar: &str) -> i64 {
    conn.query_row("SELECT COALESCE(MAX(ord)+1, 0) FROM widgets WHERE sidebar_id=?1", [sidebar], |r| r.get(0))
        .unwrap_or(0)
}

pub fn widget_insert(conn: &rusqlite::Connection, id: &str, sidebar: &str, typ: &str, title: &str, settings: &str, ord: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO widgets(id, sidebar_id, type, title, settings_json, ord) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, sidebar, typ, title, settings, ord],
    )?;
    Ok(())
}

pub fn widget_update(conn: &rusqlite::Connection, id: &str, sidebar: &str, title: &str, settings: &str, ord: i64) -> Result<()> {
    conn.execute(
        "UPDATE widgets SET sidebar_id=?1, title=?2, settings_json=?3, ord=?4 WHERE id=?5",
        params![sidebar, title, settings, ord, id],
    )?;
    Ok(())
}

pub fn widget_delete(conn: &rusqlite::Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM widgets WHERE id=?1", [id])?;
    Ok(())
}

pub fn widget_move(conn: &rusqlite::Connection, id: &str, sidebar: &str, ord: i64) -> Result<()> {
    conn.execute("UPDATE widgets SET sidebar_id=?1, ord=?2 WHERE id=?3", params![sidebar, ord, id])?;
    Ok(())
}

pub fn widget_ord(conn: &rusqlite::Connection, id: &str) -> i64 {
    conn.query_row("SELECT ord FROM widgets WHERE id=?1", [id], |r| r.get::<_, i64>(0))
        .unwrap_or(0)
}

pub fn widget_ids_in_sidebar(conn: &rusqlite::Connection, sidebar: &str) -> Vec<String> {
    let mut ids = Vec::new();
    if let Ok(mut st) = conn.prepare("SELECT id FROM widgets WHERE sidebar_id=?1 ORDER BY ord") {
        if let Ok(rows) = st.query_map([sidebar], |r| r.get::<_, String>(0)) {
            // best-effort sidebar list: the flatten intentionally skips row
            // decode errors (no Result channel to propagate through).
            ids.extend(rows.flatten());
        }
    }
    ids
}

pub fn widget_set_ord(conn: &rusqlite::Connection, id: &str, ord: i64, sidebar: Option<&str>) -> Result<()> {
    if let Some(sidebar) = sidebar {
        conn.execute("UPDATE widgets SET ord=?1 WHERE id=?2 AND sidebar_id=?3", params![ord, id, sidebar])?;
    } else {
        conn.execute("UPDATE widgets SET ord=?1 WHERE id=?2", params![ord, id])?;
    }
    Ok(())
}

pub fn sidebar_insert(conn: &rusqlite::Connection, id: &str, name: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO sidebars(id, name, lang) VALUES(?1, ?2, 'all')",
        params![id, name],
    )?;
    Ok(())
}

pub fn sidebar_delete_and_park_widgets(conn: &mut rusqlite::Connection, id: &str, inactive: &str) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    let ord = widget_next_ord(&tx, inactive);
    tx.execute(
        "UPDATE widgets SET sidebar_id=?1, ord=ord+?2 WHERE sidebar_id=?3",
        params![inactive, ord, id],
    )?;
    tx.execute("DELETE FROM sidebars WHERE id=?1", [id])?;
    tx.commit()?;
    Ok(())
}

pub fn pending_comments_count(db: &Path) -> i64 {
    crate::db::conn(db)
        .ok()
        .and_then(|c| c.query_row("SELECT COUNT(*) FROM comments WHERE status='pending'", [], |r| r.get(0)).ok())
        .unwrap_or(0)
}

pub(crate) fn civil(unix: u64) -> (i32, u32, u32) {
    // UTC civil date without extra crate
    let days = unix / 86400;
    let mut y = 1970i32;
    let mut rem = days as i32;
    loop {
        let ly = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        let yl = if ly { 366 } else { 365 };
        if rem < yl {
            break;
        }
        rem -= yl;
        y += 1;
    }
    let md = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 1u32;
    let mut d = rem;
    let ly = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    for (i, &len) in md.iter().enumerate() {
        let len = if i == 1 && ly { 29 } else { len };
        if d < len {
            m = i as u32 + 1;
            break;
        }
        d -= len;
    }
    (y, m, d as u32 + 1)
}

/// "YYYY-MM-DD HH:MM:SS" UTC now — same textual shape SQLite's
/// `datetime('now')` stores, so lexicographic compares line up.
pub fn now_sql() -> String {
    let unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (y, m, d) = civil(unix);
    let s = unix % 86400;
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

pub(crate) fn null_empty(s: &str) -> Option<&str> {
    if s.trim().is_empty() { None } else { Some(s) }
}


pub fn html_from_body_json(s: &str) -> String {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(s) else {
        return String::new();
    };
    let mut out = String::new();
    walk_text(&v, &mut out);
    out
}

pub fn body_is_gdlr(s: &str) -> bool {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(s) else {
        return false;
    };
    gdlr_node(&v)
}

fn gdlr_node(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::Array(a) => a.iter().any(gdlr_node),
        serde_json::Value::Object(m) => {
            if let Some(t) = m.get("type").and_then(|x| x.as_str()) {
                if !matches!(
                    t,
                    "text-box" | "column" | "row" | "wrapper" | "container" | "section" | "element"
                ) {
                    return true;
                }
            }
            m.values().any(gdlr_node)
        }
        _ => false,
    }
}

/// Save a validated body_json for a post (revision snapshot first), without
/// touching other post fields. Used by the builder "Save PB" endpoint.
pub fn save_body_json(db: &Path, post_id: i64, body_json: &str) -> Result<()> {
    let v: serde_json::Value = serde_json::from_str(body_json)?;
    let c = crate::db::conn(db)?;
    let tx = c.unchecked_transaction()?;
    tx.execute(sql!("posts__insert_into_revisions"), params![post_id])?;
    tx.execute(
        sql!("posts__update_posts_set_body_json"),
        params![v.to_string(), post_id],
    )?;
    tx.commit()?;
    Ok(())
}

fn walk_text(v: &serde_json::Value, out: &mut String) {
    match v {
        serde_json::Value::Array(a) => {
            for x in a {
                walk_text(x, out);
            }
        }
        serde_json::Value::Object(m) => {
            if m.get("type").and_then(|t| t.as_str()) == Some("text-box") {
                if let Some(c) = m.get("value").and_then(|v| v.get("content")).and_then(|c| c.as_str()) {
                    if !out.is_empty() {
                        out.push_str("\n");
                    }
                    out.push_str(c);
                }
            }
            for x in m.values() {
                walk_text(x, out);
            }
        }
        _ => {}
    }
}


pub fn body_json_with_html(existing: &str, html: &str) -> String {
    if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(existing) {
        if set_first_textbox(&mut v, html) {
            return v.to_string();
        }
    }
    serde_json::json!([{
        "template": "element",
        "type": "text-box",
        "value": { "content": html }
    }])
    .to_string()
}

fn set_first_textbox(v: &mut serde_json::Value, html: &str) -> bool {
    match v {
        serde_json::Value::Array(a) => {
            for x in a {
                if set_first_textbox(x, html) {
                    return true;
                }
            }
            false
        }
        serde_json::Value::Object(m) => {
            if m.get("type").and_then(|t| t.as_str()) == Some("text-box") {
                if let Some(serde_json::Value::Object(val)) = m.get_mut("value") {
                    val.insert("content".into(), serde_json::Value::String(html.into()));
                    return true;
                }
            }
            let keys: Vec<String> = m.keys().cloned().collect();
            for k in keys {
                if let Some(child) = m.get_mut(&k) {
                    if set_first_textbox(child, html) {
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

/// Quick Draft: inserts a `kind='post'`, `status='draft'` row owned by
/// `author_id` and returns its id. `posts` has no author column, so
/// authorship is stored as `post_meta.author_id` (reported schema need:
/// a real `posts.author_id` column in a future migration).
/// `content` is raw HTML wrapped into the gdlr body_json text-box shape.
pub fn insert_draft(db: &Path, title: &str, content: &str, author_id: i64) -> Result<i64> {
    let c = crate::db::conn(db)?;
    let slug = super::slugify(title.trim());
    let body = body_json_with_html("[]", content.trim());
    // No id: SQLite assigns the rowid — MAX(id)+1 raced under concurrent
    // writers (audit M12). Read it back, then write meta on the real id.
    c.execute(
        sql!("misc__insert_draft"),
        params![slug, title.trim(), body],
    )?;
    let next = c.last_insert_rowid();
    // Fresh id → no existing meta row; plain INSERT is safe.
    c.execute(
        sql!("misc__insert_meta_author"),
        params![next, author_id.to_string()],
    )?;
    Ok(next)
}

