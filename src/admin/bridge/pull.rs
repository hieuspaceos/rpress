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
use std::path::Path;

/// Tables the pull may write — a closed enum so a malformed spec can't
/// reach `posts`/`users`/`sessions` or an arbitrary table name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)] // variants populated by PhpBridge adapters' write_specs
pub(crate) enum PullTable {
    PostMeta,
    Settings,
}

/// Whitelist row shape (SQL LIKE prefix, e.g. `"gdlr-core-%"`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct WriteSpec {
    pub(crate) table: PullTable,
    /// Column carrying the key (`meta_key` / `key`).
    pub(crate) key_column: &'static str,
    /// LIKE prefix — always ends with `%`.
    pub(crate) key_prefix: &'static str,
}

impl WriteSpec {
    /// Reject a spec that would widen the blast radius:
    /// - table must be `post_meta`/`settings` (the enum itself enforces);
    /// - prefix must end with `%` and contain no other wildcard char
    ///   (`_` would widen silently);
    /// - prefix must not start with `%` (match-everything) or be empty.
    pub(crate) fn validate(&self) -> Result<()> {
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
                // wp.db schema (projected): post_meta(post_id, meta_key,
                // meta_value). Only keys matching the prefix land.
                let mut st = src.prepare(
                    "SELECT post_id, meta_key, meta_value FROM post_meta WHERE meta_key LIKE ?1",
                )?;
                let rows = st
                    .query_map([s.key_prefix], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (post_id, k, v) in rows {
                    tx.execute(
                        "UPDATE post_meta SET meta_value=?3 WHERE post_id=?1 AND meta_key=?2",
                        rusqlite::params![post_id, k, v],
                    )?;
                    copied += 1;
                }
            }
            PullTable::Settings => {
                let mut st = src.prepare(
                    "SELECT key, value FROM options WHERE key LIKE ?1",
                )?;
                let rows = st
                    .query_map([s.key_prefix], |r| {
                        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (k, v) in rows {
                    tx.execute(
                        "INSERT OR REPLACE INTO settings(key, value) VALUES(?1, ?2)",
                        rusqlite::params![k, v],
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
        let ok = WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: "gdlr-core-%" };
        assert!(ok.validate().is_ok());
        for bad in ["%", "gdlr-%_core%", "gdlr-%%-%", ""] {
            let s = WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: bad };
            assert!(s.validate().is_err(), "{bad} must be rejected");
        }
        let s = WriteSpec { table: PullTable::PostMeta, key_column: "posts", key_prefix: "x%" };
        assert!(s.validate().is_err());
    }
}
