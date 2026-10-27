use anyhow::Result;
use rusqlite::params;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const UPLOAD_LIMIT_BYTES: usize = 25 * 1024 * 1024;
pub const UPLOAD_LIMIT_DISPLAY: &str = "25 MB";

/// Local legacy upload serving policy. New uploads already reject scriptable
/// extensions; restored WP uploads still pass through this guard.
pub fn served_upload_mime(filename: &str) -> (&'static str, bool) {
    match filename
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg" | "jpeg") => ("image/jpeg", false),
        Some("png") => ("image/png", false),
        Some("gif") => ("image/gif", false),
        Some("webp") => ("image/webp", false),
        Some("avif") => ("image/avif", false),
        Some("ico") => ("image/x-icon", false),
        Some("mp4") => ("video/mp4", false),
        Some("webm") => ("video/webm", false),
        Some("pdf") => ("application/pdf", false),
        Some("woff2") => ("font/woff2", false),
        Some("woff") => ("font/woff", false),
        Some("ttf") => ("font/ttf", false),
        Some("otf") => ("font/otf", false),
        Some("eot") => ("application/vnd.ms-fontobject", false),
        // WP writes generated theme/plugin CSS into uploads/ (e.g.
        // traveltour-style-custom.css). CSS is not scriptable on a modern
        // browser, so it serves inline like the WP host does.
        Some("css") => ("text/css", false),
        Some("svg" | "xml" | "html" | "htm" | "js" | "mjs") => {
            ("application/octet-stream", true)
        }
        _ => ("application/octet-stream", true),
    }
}

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

pub fn gdlr_media_json(db: &Path, site: &str) -> Result<String> {
    let conn = crate::db::conn(db)?;
    let mut stmt = conn.prepare("SELECT id, path, COALESCE(alt,''), COALESCE(mime,'') FROM media ORDER BY id DESC")?;
    let rows = stmt.query_map([], |r| {
        Ok(serde_json::json!({
            "id": r.get::<_, i64>(0)?,
            "path": r.get::<_, String>(1)?,
            "alt": r.get::<_, String>(2)?,
            "mime": r.get::<_, String>(3)?,
        }))
    })?;
    let site = site.trim_end_matches('/').to_string();
    let v: Vec<serde_json::Value> = rows
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .map(|mut m| {
            if let Some(p) = m.get("path").and_then(|p| p.as_str()).map(|x| x.to_string()) {
                m.as_object_mut().map(|o| o.insert("url".into(), serde_json::json!(format!("{site}/wp-content/uploads/{p}"))));
            }
            m
        })
        .collect();
    Ok(serde_json::to_string(&v)?)
}


/// wp-admin upload handler: file → `uploads/YYYY/MM/name.ext` + media row.
/// Title defaults to the file stem like WP does.
/// Filename sanitized like `sanitize_file_name` (WP keeps a narrow charset).
pub(crate) fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Upload allowlist (A4): extensions we will store on disk and serve back.
/// Scriptable types (svg/html/xml/js…) are rejected so uploads can never
/// become stored-XSS payloads under our origin. Checked on the *sanitized*
/// name so `evil.svg ` / unicode tricks can't slip an alternate extension.
pub(crate) fn upload_ext_ok(filename: &str) -> bool {
    let safe = sanitize_filename(filename);
    matches!(
        safe.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).as_deref(),
        Some("jpg" | "jpeg" | "png" | "webp" | "gif" | "avif" | "ico" | "mp4" | "webm" | "pdf")
    )
}

pub fn save_upload(db: &Path, uploads: &Path, filename: &str, bytes: &[u8], alt: &str, owner: &str) -> Result<i64> {
    if bytes.len() > UPLOAD_LIMIT_BYTES {
        anyhow::bail!("max {UPLOAD_LIMIT_DISPLAY}");
    }
    if !upload_ext_ok(filename) {
        anyhow::bail!("file type not allowed");
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let (tm, tmd) = (time_ym(now), time_ymd(now));
    let dir = uploads.join(&tm);
    fs::create_dir_all(&dir)?;
    let safe = sanitize_filename(filename);
    // Deduplicate like WP: foo.jpg → foo-1.jpg → foo-2.jpg (stem computed once;
    // recomputing it would chain foo-1-1.jpg…). `create_new` instead of an
    // exists() check so concurrent uploads can't clobber each other (A10).
    let (stem, ext) = safe
        .rsplit_once('.')
        .map(|(a, b)| (a.to_string(), format!(".{b}")))
        .unwrap_or((safe.clone(), String::new()));
    let mut safe = safe;
    let mut n = 0u32;
    let dest = loop {
        let dest = dir.join(&safe);
        match fs::OpenOptions::new().write(true).create_new(true).open(&dest) {
            Ok(mut fh) => {
                if let Err(e) = fh.write_all(bytes) {
                    drop(fh);
                    let _ = fs::remove_file(&dest);
                    return Err(e.into());
                }
                break dest;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                n += 1;
                if n > 1000 {
                    anyhow::bail!("cannot find free filename for {stem}{ext}");
                }
                safe = format!("{stem}-{n}{ext}");
            }
            Err(e) => return Err(e.into()),
        }
    };
    let rel = format!("{tm}/{safe}");
    let mime = mime_of(&safe);
    let (w, h) = if mime.starts_with("image/") { image_dims(bytes) } else { (None, None) };
    let title = Path::new(&safe).file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    // WP `wp_generate_attachment_metadata`: derive the configured sizes
    // beside the original; decode/encoder failures degrade to {} rather
    // than fail the upload.
    let sizes_json = if mime.starts_with("image/") {
        write_image_sizes(db, uploads, &rel).unwrap_or_else(|_| "{}".into())
    } else {
        "{}".into()
    };
    let c = crate::db::conn(db)?;
    // No id: SQLite assigns the rowid — MAX(id)+1 raced under concurrent
    // uploads (audit M12). Unlink on insert failure so a failed INSERT
    // never orphans the file on disk.
    let ins = c.execute(
        "INSERT INTO media(wp_id, path, title, alt, caption, description, mime, w, h, sizes_json, owner, created)
         VALUES(NULL,?1,?2,?3,'','',?4,?5,?6,?9,?7,?8)",
        params![rel, title, alt, mime, w, h, owner, tmd, sizes_json],
    );
    if ins.is_err() {
        let _ = fs::remove_file(&dest);
        ins?;
    }
    Ok(c.last_insert_rowid())
}

/// Overwrite an attachment's bytes (image-editor "Save"). Keeps path/id;
/// refreshes mime + dimensions, drops stale sizes_json. First edit copies the
/// on-disk original to `media/{id}_original.{ext}` under uploads and records
/// it in `backup_path` so Restore Original can bring it back.
/// Ordering (audit): sizes are computed from the NEW bytes BEFORE the main
/// file is written and stale derived files are unlinked, so a mid-flight
/// failure can never leave the row pointing at size files that no longer
/// match the on-disk original.
pub fn media_replace_upload(db: &Path, uploads: &Path, id: i64, bytes: &[u8]) -> Result<()> {
    if bytes.len() > UPLOAD_LIMIT_BYTES {
        anyhow::bail!("max {UPLOAD_LIMIT_DISPLAY}");
    }
    let c = crate::db::conn(db)?;
    let row: Option<(String, Option<String>, String)> = c
        .query_row("SELECT path, backup_path, COALESCE(sizes_json,'') FROM media WHERE id=?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .ok();
    let Some((path, backup, old_sizes_json)) = row else { anyhow::bail!("media {id} not found") };
    if path.contains("..") || path.starts_with('/') {
        anyhow::bail!("bad path");
    }
    if !mime_of(&path).starts_with("image/") {
        anyhow::bail!("not an image");
    }
    // First edit only: stash the pristine file beside the upload, keyed by id
    // so a same-name sibling can never collide.
    let backup = backup.unwrap_or_default();
    if backup.is_empty() {
        let src = uploads.join(&path);
        if src.exists() {
            let ext = path.rsplit_once('.').map(|(_, e)| e).unwrap_or("bin");
            let rel = format!("media/{id}_original.{ext}");
            let dir = uploads.join("media");
            fs::create_dir_all(&dir)?;
            fs::copy(&src, dir.join(format!("{id}_original.{ext}")))?;
            c.execute("UPDATE media SET backup_path=?2 WHERE id=?1", params![id, rel])?;
        }
    }
    // Trust the blob, not the extension: the editor can emit PNG for a
    // .webp source; mime must describe the stored bytes.
    let mime = mime_from_bytes(bytes).unwrap_or_else(|| mime_of(&path).to_string());
    let (w, h) = image_dims(bytes);
    // Regenerate the configured sizes from the new bytes BEFORE the main
    // file lands — decode/resize/encode all happens against `bytes`, and
    // the derived writes commit only after the main write succeeds, so a
    // mid-flight failure never swaps in half-updated derived files (WP
    // wp_update_attachment_metadata on image edit; failure falls back to
    // the bare "full" descriptor so delete/grid code keeps working).
    let (sizes, pending) = plan_image_sizes(db, uploads, &path, bytes).unwrap_or_else(|_| {
        (format!(r#"{{"full":{{"filesize":{},"height":{},"width":{},"mime-type":"{}"}}}}"#, bytes.len(), h.map(|v| v.to_string()).unwrap_or("null".into()), w.map(|v| v.to_string()).unwrap_or("null".into()), mime), Vec::new())
    });
    fs::write(uploads.join(&path), bytes)?;
    for (dest, bytes) in &pending {
        fs::write(dest, bytes)?;
    }
    // Unlink stale derived files the new sizes_json no longer references
    // (dimension/format changes rename the size files; orphans would serve
    // stale pixels under a forgotten name).
    let new_files: std::collections::HashSet<String> = serde_json::from_str::<serde_json::Value>(&sizes)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
        .values()
        .filter_map(|s| s.get("file").and_then(|f| f.as_str()).map(String::from))
        .collect();
    if let Ok(serde_json::Value::Object(old)) = serde_json::from_str::<serde_json::Value>(&old_sizes_json) {
        for f in old.values().filter_map(|s| s.get("file").and_then(|f| f.as_str())) {
            if !new_files.contains(f) && !f.contains("..") && !f.starts_with('/') {
                let rel = match path.rsplit_once('/') {
                    Some((d, _)) => format!("{d}/{f}"),
                    None => f.to_string(),
                };
                let _ = fs::remove_file(uploads.join(rel));
            }
        }
    }
    c.execute(
        "UPDATE media SET w=?2, h=?3, mime=?4, sizes_json=?5 WHERE id=?1",
        params![id, w, h, mime, sizes],
    )?;
    Ok(())
}

/// Copy `backup_path` back over the live file ("Restore Original") and
/// refresh mime/dimensions/sizes_json from the restored bytes. The backup
/// itself is kept — WP keeps `backup_sizes`, and repeated restores must work.
pub fn media_restore(db: &Path, uploads: &Path, id: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    let row: Option<(String, Option<String>)> = c
        .query_row("SELECT path, backup_path FROM media WHERE id=?1", [id], |r| Ok((r.get(0)?, r.get(1)?)))
        .ok();
    let Some((path, backup)) = row else { anyhow::bail!("media {id} not found") };
    let backup = backup.unwrap_or_default();
    for p in [&path, &backup] {
        if p.is_empty() || p.contains("..") || p.starts_with('/') {
            anyhow::bail!("no backup for media {id}");
        }
    }
    let bytes = fs::read(uploads.join(&backup))?;
    fs::write(uploads.join(&path), &bytes)?;
    let (w, h) = image_dims(&bytes);
    let mime = mime_from_bytes(&bytes).unwrap_or_else(|| mime_of(&path).to_string());
    let sizes = write_image_sizes(db, uploads, &path).unwrap_or_else(|_| {
        format!(r#"{{"full":{{"filesize":{},"height":{},"width":{},"mime-type":"{}"}}}}"#, bytes.len(), h.map(|v| v.to_string()).unwrap_or("null".into()), w.map(|v| v.to_string()).unwrap_or("null".into()), mime)
    });
    c.execute(
        "UPDATE media SET w=?2, h=?3, mime=?4, sizes_json=?5 WHERE id=?1",
        params![id, w, h, mime, sizes],
    )?;
    Ok(())
}

/// Detect mime from magic bytes (PNG/JPEG/GIF/WebP). Fallback: None.
fn mime_from_bytes(b: &[u8]) -> Option<String> {
    if b.len() > 8 && &b[..8] == b"\x89PNG\r\n\x1a\n" {
        Some("image/png".into())
    } else if b.len() > 3 && b[0] == 0xFF && b[1] == 0xD8 && b[2] == 0xFF {
        Some("image/jpeg".into())
    } else if b.len() > 6 && (&b[..6] == b"GIF87a" || &b[..6] == b"GIF89a") {
        Some("image/gif".into())
    } else if b.len() > 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        Some("image/webp".into())
    } else {
        None
    }
}

/// Backfill media columns added after the table first shipped. Runs every
/// boot; ALTER errors mean the column already exists.
pub fn ensure_media_cols(db: &Path) -> Result<()> {
    let c = crate::db::conn(db)?;
    for (col, dd) in [("title", ""), ("description", ""), ("created", ""), ("owner", ""), ("backup_path", "")] {
        let _ = c.execute(&format!("ALTER TABLE media ADD COLUMN {col} TEXT DEFAULT '{dd}'"), []);
    }
    // Backfill created "YYYY/MM" from the path prefix ("2023/12/file.png").
    let _ = c.execute(
        "UPDATE media SET created=substr(path,1,7)
         WHERE (created IS NULL OR created='') AND path GLOB '[0-9][0-9][0-9][0-9]/[0-9][0-9]/*'",
        [],
    );
    Ok(())
}

/// Upgrade month-only `created` ("YYYY/MM") to full dates using the local
/// file's mtime. Remote-only rows keep the month — the day is unknowable.
pub fn media_backfill_dates(db: &Path, uploads: &Path) -> Result<u64> {
    let c = crate::db::conn(db)?;
    let mut s = c.prepare("SELECT id, path FROM media WHERE created GLOB '[0-9][0-9][0-9][0-9]/[0-9][0-9]'")?;
    let rows: Vec<(i64, String)> = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    drop(s);
    let mut n = 0u64;
    for (id, path) in rows {
        let p = uploads.join(&path);
        let Ok(meta) = fs::metadata(&p) else { continue };
        let Ok(mtime) = meta.modified() else { continue };
        let Ok(secs) = mtime.duration_since(UNIX_EPOCH).map(|d| d.as_secs()) else { continue };
        c.execute("UPDATE media SET created=?2 WHERE id=?1", params![id, time_ymd(secs)])?;
        n += 1;
    }
    Ok(n)
}

#[derive(Debug, Clone)]
pub struct MediaRow {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub alt: String,
    pub caption: String,
    pub description: String,
    pub mime: String,
    pub w: Option<i64>,
    pub h: Option<i64>,
    pub created: String,
    pub sizes_json: String,
    pub owner: String,
    pub backup_path: String,
}

fn media_row(r: &rusqlite::Row) -> rusqlite::Result<MediaRow> {
    Ok(MediaRow {
        id: r.get(0)?,
        path: r.get(1)?,
        title: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
        alt: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
        caption: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
        description: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
        mime: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
        w: r.get(7)?,
        h: r.get(8)?,
        created: r.get::<_, Option<String>>(9)?.unwrap_or_default(),
        sizes_json: r.get::<_, Option<String>>(10)?.unwrap_or_default(),
        owner: r.get::<_, Option<String>>(11)?.unwrap_or_default(),
        backup_path: r.get::<_, Option<String>>(12)?.unwrap_or_default(),
    })
}

const MEDIA_COLS: &str = "id, path, title, alt, caption, description, mime, w, h, created, sizes_json, owner, backup_path";

/// Library list with WP filters: mime group, YYYY/MM month, search, sort.
/// `mime_group`: "" | "image" | "application/pdf" | "other" (not image/pdf);
/// any other value filters by the "<group>/" prefix (audio, video, ...).
pub fn media_list(
    db: &Path,
    mime_group: &str,
    month: &str,
    search: &str,
    sort: &str,
    offset: i64,
    limit: i64,
) -> Result<(Vec<MediaRow>, i64)> {
    let c = crate::db::conn(db)?;
    let mut wh = String::from("WHERE 1=1");
    let mut ps: Vec<rusqlite::types::Value> = Vec::new();
    match mime_group {
        "" => {}
        "other" => wh.push_str(" AND COALESCE(mime,'') NOT LIKE 'image/%' AND COALESCE(mime,'') != 'application/pdf'"),
        "application/pdf" => wh.push_str(" AND COALESCE(mime,'')='application/pdf'"),
        _ => {
            wh.push_str(" AND COALESCE(mime,'') LIKE ?");
            ps.push(format!("{mime_group}/%").into());
        }
    }
    if !month.is_empty() {
        // Validate YYYY/MM strictly — LIKE treats % and _ as wildcards, and a
        // raw user pattern would match unintended months.
        let valid = month.len() == 7
            && month.as_bytes()[4] == b'/'
            && month.bytes().enumerate().all(|(i, b)| i == 4 || b.is_ascii_digit());
        if valid {
            wh.push_str(" AND (COALESCE(created,'') LIKE ? OR COALESCE(path,'') LIKE ?)");
            ps.push(format!("{month}%").into());
            ps.push(format!("{month}/%").into());
        }
    }
    if !search.is_empty() {
        wh.push_str(" AND (path LIKE ? ESCAPE '\\' OR title LIKE ? ESCAPE '\\' OR alt LIKE ? ESCAPE '\\' OR caption LIKE ? ESCAPE '\\' OR description LIKE ? ESCAPE '\\')");
        let p = format!("%{}%", search.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        ps.push(p.clone().into());
        ps.push(p.clone().into());
        ps.push(p.clone().into());
        ps.push(p.clone().into());
        ps.push(p.into());
    }
    let total: i64 = {
        let mut s = c.prepare(&format!("SELECT COUNT(*) FROM media {wh}"))?;
        s.query_row(rusqlite::params_from_iter(ps.iter()), |r| r.get(0))?
    };
    let order = match sort {
        "date_asc" => "created ASC, id ASC",
        "title" => "title COLLATE NOCASE ASC, id DESC",
        _ => "created DESC, id DESC",
    };
    let mut s = c.prepare(&format!("SELECT {MEDIA_COLS} FROM media {wh} ORDER BY {order} LIMIT ? OFFSET ?"))?;
    let mut all = ps.clone();
    all.push(limit.into());
    all.push(offset.into());
    let rows = s.query_map(rusqlite::params_from_iter(all), media_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((rows, total))
}

/// Distinct upload months for the "All dates" filter (YYYY/MM → displayed).
pub fn media_months(db: &Path) -> Result<Vec<String>> {
    let c = crate::db::conn(db)?;
    let mut s = c.prepare(
        "SELECT DISTINCT substr(COALESCE(NULLIF(created,''), path),1,7) AS m
         FROM media WHERE m GLOB '[0-9][0-9][0-9][0-9]/[0-9][0-9]' ORDER BY m DESC",
    )?;
    let rows = s.query_map([], |r| r.get::<_, String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn media_get(db: &Path, id: i64) -> Result<Option<MediaRow>> {
    let c = crate::db::conn(db)?;
    let mut s = c.prepare(&format!("SELECT {MEDIA_COLS} FROM media WHERE id=?1"))?;
    let row = s.query_map([id], media_row)?.next().transpose()?;
    Ok(row)
}

pub fn media_update(db: &Path, id: i64, title: &str, alt: &str, caption: &str, description: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(
        "UPDATE media SET title=?2, alt=?3, caption=?4, description=?5 WHERE id=?1",
        params![id, title, alt, caption, description],
    )?;
    Ok(())
}

/// Row removal. Returns `(path, sizes_json, backup_path)` so the caller can
/// unlink only files the DB actually owns — never glob-delete by name pattern.
pub fn media_delete(db: &Path, id: i64) -> Result<Option<(String, String, String)>> {
    let c = crate::db::conn(db)?;
    let row: Option<(String, String, String)> = c
        .query_row(
            "SELECT path, COALESCE(sizes_json,''), COALESCE(backup_path,'') FROM media WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok();
    c.execute("DELETE FROM media WHERE id=?1", [id])?;
    // Clear featured-image references so lists don't dangle.
    let _ = c.execute("UPDATE posts SET featured_media_id=NULL WHERE featured_media_id=?1", [id]);
    Ok(row)
}

/// Posts that use this media as featured image → "Uploaded to" column.
pub fn media_attached_to(db: &Path, id: i64) -> Result<Vec<(i64, String)>> {
    let c = crate::db::conn(db)?;
    let mut s = c.prepare(
        "SELECT id, COALESCE(NULLIF(title,''),'(no title)') FROM posts WHERE featured_media_id=?1 ORDER BY id DESC LIMIT 5",
    )?;
    let rows = s.query_map([id], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// "Detach": remove featured-image reference from a post.
pub fn media_detach(db: &Path, media_id: i64, post_id: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(
        "UPDATE posts SET featured_media_id=NULL WHERE id=?1 AND featured_media_id=?2",
        params![post_id, media_id],
    )?;
    Ok(())
}

/// Minimal image dimension sniffing — avoids an image crate for a header read.
pub fn image_dims(b: &[u8]) -> (Option<i64>, Option<i64>) {
    // PNG: 8-byte sig, IHDR width/height BE at 16..24
    if b.len() > 24 && &b[..8] == b"\x89PNG\r\n\x1a\n" {
        let w = u32::from_be_bytes(b[16..20].try_into().unwrap()) as i64;
        let h = u32::from_be_bytes(b[20..24].try_into().unwrap()) as i64;
        return (Some(w), Some(h));
    }
    // GIF: logical screen descriptor at 6..10 LE
    if b.len() > 10 && &b[..6] == b"GIF89a" || b.len() > 10 && &b[..6] == b"GIF87a" {
        let w = u16::from_le_bytes(b[6..8].try_into().unwrap()) as i64;
        let h = u16::from_le_bytes(b[8..10].try_into().unwrap()) as i64;
        return (Some(w), Some(h));
    }
    // JPEG: scan SOF markers
    if b.len() > 4 && b[0] == 0xFF && b[1] == 0xD8 {
        let mut i = 2usize;
        while i + 9 < b.len() {
            if b[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = b[i + 1];
            let len = u16::from_be_bytes(b[i + 2..i + 4].try_into().unwrap()) as usize;
            if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
                let h = u16::from_be_bytes(b[i + 5..i + 7].try_into().unwrap()) as i64;
                let w = u16::from_be_bytes(b[i + 7..i + 9].try_into().unwrap()) as i64;
                return (Some(w), Some(h));
            }
            i += 2 + len.max(1);
        }
    }
    // WebP: VP8X (24..30) or VP8 (frame header) — only VP8X covered
    if b.len() > 30 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP" && &b[12..16] == b"VP8X" {
        let w = (u32::from(b[24]) | u32::from(b[25]) << 8 | u32::from(b[26]) << 16) as i64 + 1;
        let h = (u32::from(b[27]) | u32::from(b[28]) << 8 | u32::from(b[29]) << 16) as i64 + 1;
        return (Some(w), Some(h));
    }
    (None, None)
}

fn civil(unix: u64) -> (i32, u32, u32) {
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

fn time_ym(unix: u64) -> String {
    let (y, m, _) = civil(unix);
    format!("{y:04}/{m:02}")
}

fn time_ymd(unix: u64) -> String {
    let (y, m, d) = civil(unix);
    format!("{y:04}/{m:02}/{d:02}")
}

/// "YYYY-MM-DD HH:MM:SS" UTC now — same textual shape SQLite's
/// `datetime('now')` stores, so lexicographic compares line up.
pub fn now_sql() -> String {
    let unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (y, m, d) = civil(unix);
    let s = unix % 86400;
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}


/// WP `wp_generate_attachment_metadata` for our size set: the Settings→Media
/// values (thumbnail/medium/large, plus WP's hidden `medium_large` 768w —
/// it's what makes content srcsets useful and can't be disabled in core).
/// Writes `dir/stem-{w}x{h}.ext` next to the original and returns the
/// sizes_json map WP stores in `_wp_attachment_metadata["sizes"]`.
/// Failures (undecodable upload, encoder gap like AVIF) return the map
/// built so far — an attachment with partial/no sizes is still valid.
pub fn write_image_sizes(db: &Path, uploads: &Path, path: &str) -> Result<String> {
    let bytes = fs::read(uploads.join(path))?;
    let (sizes, pending) = plan_image_sizes(db, uploads, path, &bytes)?;
    // Existing-file path (upload/regenerate): write immediately.
    for (dest, bytes) in pending {
        fs::write(&dest, &bytes)?;
    }
    Ok(sizes)
}

/// Compute the WP size set for `bytes` as `path`'s attachment: returns the
/// sizes_json map plus the pending `(dest, encoded bytes)` writes.
/// `media_replace_upload` needs the decode/resize done BEFORE the main
/// file is swapped (audit ordering): plan from the new bytes, write the
/// main file, then commit the pending size writes.
fn plan_image_sizes(db: &Path, uploads: &Path, path: &str, bytes: &[u8]) -> Result<(String, Vec<(PathBuf, Vec<u8>)>)> {
    let geti = |k: &str, d: u32| -> u32 {
        crate::db::comments::setting_get(db, k)
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(d)
    };
    // (name, box_w, box_h, crop)
    let specs = [
        ("thumbnail", geti("thumbnail_size_w", 150), geti("thumbnail_size_h", 150), geti("thumbnail_crop", 1) != 0),
        ("medium", geti("medium_size_w", 300), geti("medium_size_h", 300), false),
        ("medium_large", geti("medium_large_size_w", 768), geti("medium_large_size_h", 0), false),
        ("large", geti("large_size_w", 1024), geti("large_size_h", 1024), false),
    ];
    let img = image::load_from_memory(bytes)?;
    let (sw, sh) = (img.width(), img.height());
    let (dir, stem) = path
        .rsplit_once('/')
        .map(|(d, f)| (d, f.rsplit_once('.').map(|(s, _)| s).unwrap_or(f)))
        .unwrap_or(("", path.rsplit_once('.').map(|(s, _)| s).unwrap_or(path)));
    let ext = path.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
    let format = match ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => Some(image::ImageFormat::Jpeg),
        "png" => Some(image::ImageFormat::Png),
        "gif" => Some(image::ImageFormat::Gif),
        "webp" => Some(image::ImageFormat::WebP),
        _ => None,
    };
    let Some(format) = format else { return Ok(("{}".into(), Vec::new())) };
    let mut sizes = serde_json::Map::new();
    let mut pending: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    for (name, tw, th, crop) in specs {
        if tw == 0 && th == 0 {
            continue;
        }
        // WP image_resize_dimensions: uncropped sizes only scale down inside
        // the box; a 0 bound is unbounded (medium_large_h = 0).
        let sub = if crop {
            if sw <= tw || sh <= th {
                continue; // source smaller than crop box: WP skips the size
            }
            img.resize_to_fill(tw, th, image::imageops::FilterType::Lanczos3)
        } else {
            let bw = if tw == 0 { u32::MAX } else { tw };
            let bh = if th == 0 { u32::MAX } else { th };
            if sw <= bw && sh <= bh {
                continue; // already smaller: no derived file
            }
            img.resize(bw, bh, image::imageops::FilterType::Lanczos3)
        };
        let (dw, dh) = (sub.width(), sub.height());
        if dw == 0 || dh == 0 || (dw == sw && dh == sh) {
            continue;
        }
        let file = format!("{stem}-{dw}x{dh}.{ext}");
        let dest = if dir.is_empty() {
            uploads.join(&file)
        } else {
            uploads.join(dir).join(&file)
        };
        let mut buf = std::io::Cursor::new(Vec::new());
        if sub.write_to(&mut buf, format).is_err() {
            continue;
        }
        let bytes = buf.into_inner();
        sizes.insert(
            name.to_string(),
            serde_json::json!({"file": file, "width": dw, "height": dh, "mime-type": mime_of(&file), "filesize": bytes.len()}),
        );
        pending.push((dest, bytes));
    }
    Ok((serde_json::Value::Object(sizes).to_string(), pending))
}

/// Rebuild sizes_json (+ the size files) for every local image attachment —
/// the backfill behind POST /admin/media/regenerate. Remote-only rows (no
/// file on disk) are skipped. Returns (regenerated, skipped_or_failed).
pub fn media_regenerate_sizes(db: &Path, uploads: &Path) -> Result<(u64, u64)> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare("SELECT id, path FROM media WHERE mime LIKE 'image/%'")?;
    let rows: Vec<(i64, String)> = st
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1).unwrap_or_default())))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let (mut done, mut skipped) = (0u64, 0u64);
    for (id, path) in rows {
        if path.contains("..") || path.starts_with('/') || !uploads.join(&path).is_file() {
            skipped += 1;
            continue;
        }
        match write_image_sizes(db, uploads, &path) {
            Ok(sizes) => {
                c.execute("UPDATE media SET sizes_json=?2 WHERE id=?1", params![id, sizes])?;
                done += 1;
            }
            Err(_) => skipped += 1,
        }
    }
    Ok((done, skipped))
}

fn mime_of(name: &str) -> &'static str {
    match Path::new(name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
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

