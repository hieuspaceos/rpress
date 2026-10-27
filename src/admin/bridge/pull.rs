//! Pull-write path (phase-06 T24, option b): after each proxied request
//! the bridge's disposable `wp.db` is opened read-only and whitelisted
//! rows are copied into `rpress.db`. No inbound HTTP from PHP, no write
//! endpoint — the mu-plugin can't touch our DB.
//!
//! `WriteSpec { table, key_column, key_prefix }` declares what the
//! adapter accepts: `{post_meta, meta_key, "gdlr-core-%"}` or
//! `{settings, key, "plugin:goodlayers:%"}`. Anything else in wp.db is
//! ignored and wiped on the next spawn anyway.

use anyhow::Result;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Tables the pull may write — a closed enum so a malformed spec can't
/// reach `posts`/`users`/`sessions` or an arbitrary table name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)] // variants populated by PhpBridge adapters' write_specs
pub(crate) enum PullTable {
    /// wp.db `wp_postmeta` → rpress `post_meta`.
    PostMeta,
    /// wp.db `wp_options` → rpress `settings`.
    Settings,
    /// Derived capability (phase-07): after the meta pull, copy the meta
    /// row whose key equals `key_prefix` (exact, no wildcard) into
    /// `posts.body_json` for that post. GoodLayers' save action stores
    /// the layout as post meta — the Rust port wrote it to `body_json`
    /// directly; this keeps `posts.body_json` the render authority.
    BodyJson,
}
/// Whitelist row shape (SQL LIKE prefix, e.g. `"gdlr-core-%"`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct WriteSpec {
    pub(crate) table: PullTable,
    /// Column carrying the key (`meta_key` / `key`).
    pub(crate) key_column: &'static str,
    /// LIKE prefix — always ends with `%`.
    pub(crate) key_prefix: &'static str,
    /// Optional rewrite applied to the key before it lands: the wp.db
    /// key `gdlr-core-skins` becomes `plugin:goodlayers:gdlr-core-skins`
    /// in rpress settings. `None` → verbatim copy. Only a literal
    /// prefix is allowed (no wildcards) — the transform can't smuggle
    /// a key outside the plugin's namespace.
    pub(crate) key_map: Option<&'static str>,
}

impl WriteSpec {
    /// Reject a spec that would widen the blast radius:
    /// - table must be `post_meta`/`settings` (the enum itself enforces);
    /// - prefix must end with `%` and contain no other wildcard char
    ///   (`_` would widen silently);
    /// - prefix must not start with `%` (match-everything) or be empty.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.table == PullTable::BodyJson {
            // Exact meta key — a wildcard here would widen to an
            // arbitrary `posts.body_json` overwrite.
            anyhow::ensure!(
                self.key_map.is_none(),
                "BodyJson writes a column, not a namespaced key"
            );
            anyhow::ensure!(
                !self.key_prefix.contains(['%', '_']),
                "BodyJson spec takes an exact meta_key, no wildcards"
            );
            return Ok(());
        }
        if let Some(m) = self.key_map {
            anyhow::ensure!(
                !m.contains(['%', '_']) && !m.is_empty(),
                "key_map must be a literal prefix, no wildcards"
            );
        }
        anyhow::ensure!(self.key_prefix.ends_with('%'), "WriteSpec prefix must end with %");
        anyhow::ensure!(self.key_prefix.len() > 1, "WriteSpec prefix can't be bare %");
        anyhow::ensure!(
            !self.key_prefix[..self.key_prefix.len() - 1].contains(['%', '_']),
            "WriteSpec prefix can't contain wildcards before the final %"
        );
        anyhow::ensure!(
            matches!(self.key_column, "meta_key" | "key"),
            "WriteSpec key_column must be meta_key or key"
        );
        Ok(())
    }
}

/// sqlite-integration writes the real DB as `.ht.<hash>/.ht.sqlite` inside
/// the mounted database dir. Resolve it lazily — the file only exists
/// after WP's first write.
pub(crate) fn locate(db_dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(db_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path().join(".ht.sqlite"))
        .find(|f| f.exists())
}

/// After a proxied request: open `wp.db` read-only and copy whitelisted
/// rows to `rpress_db`. `specs` come from the adapter; each is validated
/// before a single statement runs.
pub(crate) fn pull(wp_db: &Path, rpress_db: &Path, specs: &[WriteSpec]) -> Result<u64> {
    if specs.is_empty() {
        return Ok(0);
    }
    for s in specs {
        s.validate()?;
    }
    let src = Connection::open_with_flags(wp_db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let dst = crate::db::conn(rpress_db)?;
    let tx = dst.unchecked_transaction()?;
    let mut copied = 0u64;
    for s in specs {
        match s.table {
            PullTable::PostMeta => {
                // wp.db WP schema: wp_postmeta(post_id, meta_key,
                // meta_value). Only keys matching the prefix land;
                // rpress post_meta PK is (post_id, meta_key, meta_value)
                // so delete-then-insert = upsert semantics.
                let mut st = src.prepare(
                    "SELECT post_id, meta_key, meta_value FROM wp_postmeta WHERE meta_key LIKE ?1",
                )?;
                let rows = st
                    .query_map([s.key_prefix], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (post_id, k, v) in rows {
                    // wp.db post_id is the WP ID (== rpress wp_id or id);
                    // translate through posts.wp_id with id fallback.
                    let rpress_id: i64 = tx.query_row(
                        "SELECT COALESCE((SELECT id FROM posts WHERE wp_id=?1 LIMIT 1), ?1)",
                        [post_id], |r| r.get(0),
                    )?;
                    tx.execute("DELETE FROM post_meta WHERE post_id=?1 AND meta_key=?2",
                        rusqlite::params![rpress_id, k])?;
                    tx.execute(
                        "INSERT INTO post_meta(post_id, meta_key, meta_value) VALUES(?1, ?2, ?3)",
                        rusqlite::params![rpress_id, k, v],
                    )?;
                    copied += 1;
                }
            }
            PullTable::Settings => {
                let mut st = src.prepare(
                    "SELECT option_name, option_value FROM wp_options WHERE option_name LIKE ?1",
                )?;
                let rows = st
                    .query_map([s.key_prefix], |r| {
                        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (k, v) in rows {
                    let k = match s.key_map {
                        Some(prefix) => format!("{prefix}{k}"),
                        None => k,
                    };
                    tx.execute(
                        "INSERT OR REPLACE INTO settings(key, value) VALUES(?1, ?2)",
                        rusqlite::params![k, v],
                    )?;
                    copied += 1;
                }
            }
            PullTable::BodyJson => {
                // Exact meta key → posts.body_json for that post. WP stores
                // the value PHP-serialized (`update_post_meta` on an array);
                // our editor/public render expects JSON — unserialize, then
                // re-encode. Non-serialized values pass through verbatim.
                let mut st = src.prepare(
                    "SELECT post_id, meta_value FROM wp_postmeta WHERE meta_key = ?1",
                )?;
                let rows = st
                    .query_map([s.key_prefix], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (post_id, v) in rows {
                    let rpress_id: i64 = tx.query_row(
                        "SELECT COALESCE((SELECT id FROM posts WHERE wp_id=?1 LIMIT 1), ?1)",
                        [post_id], |r| r.get(0),
                    )?;
                    let body = match crate::php_serialize::unserialize(v.as_bytes()) {
                        Ok(j) => j.to_string(),
                        Err(_) => v,
                    };
                    tx.execute(
                        "UPDATE posts SET body_json=?2 WHERE id=?1",
                        rusqlite::params![rpress_id, body],
                    )?;
                    copied += 1;
                }
            }
        }
    }
    tx.commit()?;
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_rejects_bad_shapes() {
        let ok = WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: "gdlr-core-%", key_map: None };
        assert!(ok.validate().is_ok());
        for bad in ["%", "gdlr-%_core%", "gdlr-%%-%", ""] {
            let s = WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: bad, key_map: None };
            assert!(s.validate().is_err(), "{bad} must be rejected");
        }
        let s = WriteSpec { table: PullTable::PostMeta, key_column: "posts", key_prefix: "x%", key_map: None };
        assert!(s.validate().is_err());
        // BodyJson takes an exact meta_key — wildcards rejected.
        let exact = WriteSpec { table: PullTable::BodyJson, key_column: "meta_key", key_prefix: "gdlr-core-page-builder", key_map: None };
        assert!(exact.validate().is_ok());
        for bad in ["gdlr-%", "gdlr-_x", "gdlr%"] {
            let s = WriteSpec { table: PullTable::BodyJson, key_column: "meta_key", key_prefix: bad, key_map: None };
            assert!(s.validate().is_err(), "{bad} must be rejected");
        }
    }

    #[test]
    fn pull_copies_only_whitelisted() {
        let dir = std::env::temp_dir().join(format!("rpress-pull-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let wdb = dir.join("wp.db");
        let rdb = dir.join("rpress.db");
        {
            let w = Connection::open(&wdb).unwrap();
            w.execute_batch(
                "CREATE TABLE wp_postmeta(post_id INT, meta_key TEXT, meta_value TEXT);
                 CREATE TABLE wp_options(option_name TEXT, option_value TEXT);
                 INSERT INTO wp_postmeta VALUES
                    (42,'gdlr-core-page-builder','[{\"a\":1}]'),
                    (42,'gdlr-core-skin','dark'),
                    (42,'secret','x'),
                    (43,'gdlr-core-skin','y');
                 INSERT INTO wp_options VALUES
                    ('gdlr-core-templates','{\"t\":1}'),
                    ('admin_email','a@b.c');",
            ).unwrap();
        }
        {
            let r = crate::db::conn(&rdb).unwrap();
            crate::schema::migrate_conn(&r).unwrap();
            r.execute(
                "INSERT INTO posts(id, wp_id, slug, title, status, kind)
                 VALUES(42, 42, 'p', 'P', 'publish', 'page')",
                [],
            ).unwrap();
            // posts.id in rpress == wp post_id only when wp_id is set;
            // projection writes wp_id as ID — pull sees the projected id,
            // which equals posts.id when wp_id==id (or map: pull joins on
            // posts.wp_id fallback below).
        }
        let specs = [
            WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: "gdlr-core-%", key_map: None },
            WriteSpec { table: PullTable::Settings, key_column: "key", key_prefix: "gdlr-core-%", key_map: Some("plugin:goodlayers:") },
            WriteSpec { table: PullTable::BodyJson, key_column: "meta_key", key_prefix: "gdlr-core-page-builder", key_map: None },
        ];
        pull(&wdb, &rdb, &specs).unwrap();
        let r = crate::db::conn(&rdb).unwrap();
        let metas: i64 = r.query_row(
            "SELECT COUNT(*) FROM post_meta WHERE post_id=42 AND meta_key LIKE 'gdlr-core-%'",
            [], |x| x.get(0),
        ).unwrap();
        assert_eq!(metas, 2);
        let secret: i64 = r.query_row(
            "SELECT COUNT(*) FROM post_meta WHERE meta_key='secret'", [], |x| x.get(0),
        ).unwrap();
        assert_eq!(secret, 0);
        let opt: i64 = r.query_row(
            "SELECT COUNT(*) FROM settings WHERE key='plugin:goodlayers:gdlr-core-templates'", [], |x| x.get(0),
        ).unwrap();
        assert_eq!(opt, 1);
        let admin: i64 = r.query_row(
            "SELECT COUNT(*) FROM settings WHERE key='admin_email'", [], |x| x.get(0),
        ).unwrap();
        assert_eq!(admin, 0);
        let bj: String = r.query_row("SELECT body_json FROM posts WHERE id=42", [], |x| x.get(0)).unwrap();
        assert_eq!(bj, "[{\"a\":1}]");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
