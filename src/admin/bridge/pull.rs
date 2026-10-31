//! Pull-write path (phase-06 T24, option b): after each proxied request
//! the bridge's disposable `wp.db` is opened read-only and whitelisted
//! rows are copied into `rpress.db`. No inbound HTTP from PHP, no write
//! endpoint — the mu-plugin can't touch our DB.
//!
//! `WriteSpec { table, key_column, key_prefix }` declares what the
//! adapter accepts: `{post_meta, meta_key, "gdlr-core"}` or
//! `{settings, key, "plugin:goodlayers:…"}`. Matching is a literal
//! prefix (`instr` — `_` is not a wildcard). Anything else in wp.db is
//! ignored and wiped on the next spawn anyway.

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
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
    /// wp.db `wp_posts` fields → rpress `posts`: `post_content` lands on
    /// `body_html`, `post_status` on `status`, for every resolved
    /// wp_id→rpress post. When the current `body_json` is the generic
    /// import wrapper (single `text-box` element) or empty it is
    /// rebuilt from `post_content` so the render stops serving the
    /// stale tree; GDLR-shaped bodies are untouched. Used by builders
    /// (Elementor) whose compiled output lives in `post_content`, not
    /// meta. `key_prefix` is unused — pass `""`.
    PostFields,
}
/// Whitelist row shape (literal prefix, e.g. `"gdlr-core"` — `_` is an
/// ordinary character now that pull matching uses `instr`, not LIKE).
#[derive(Clone, Copy, Debug)]
pub(crate) struct WriteSpec {
    pub(crate) table: PullTable,
    /// Column carrying the key (`meta_key` / `key`).
    pub(crate) key_column: &'static str,
    /// Literal key prefix — matched with `instr(col, ?1) = 1`, so `_`
    /// is legal. `%` would silently match wrong/no keys → forbidden.
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
    /// - prefix must be non-empty and contain no `%` (under literal
    ///   `instr` matching a `%` is a plain character that would silently
    ///   match wrong keys — a config bug);
    /// - `_` is a literal, so `_elementor` is a valid prefix.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.table == PullTable::BodyJson {
            // Exact meta key — a wildcard here would widen to an
            // arbitrary `posts.body_json` overwrite.
            anyhow::ensure!(
                self.key_map.is_none(),
                "BodyJson writes a column, not a namespaced key"
            );
            anyhow::ensure!(
                !self.key_prefix.contains('%'),
                "BodyJson spec takes an exact meta_key — `%` is not a literal key char"
            );
            return Ok(());
        }
        if self.table == PullTable::PostFields {
            // Fixed column set — no key filtering happens for this table.
            anyhow::ensure!(
                self.key_prefix.is_empty() && self.key_map.is_none(),
                "PostFields spec takes empty key_prefix and no key_map"
            );
            return Ok(());
        }
        if let Some(m) = self.key_map {
            anyhow::ensure!(
                !m.contains('%') && !m.is_empty(),
                "key_map must be a literal prefix, no `%`"
            );
        }
        anyhow::ensure!(!self.key_prefix.is_empty(), "WriteSpec prefix can't be empty");
        anyhow::ensure!(
            !self.key_prefix.contains('%'),
            "WriteSpec prefix is literal — `%` is not a wildcard char"
        );
        anyhow::ensure!(
            matches!(self.key_column, "meta_key" | "key"),
            "WriteSpec key_column must be meta_key or key"
        );
        Ok(())
    }
}

/// `body_json` is the generic import wrapper when it's empty or a
/// single-element `[{"template":"element","type":"text-box",
/// "value":{"content":…}}]` tree (m04_backfill decision rules). Rebuild
/// it around `content`; any other shape (GDLR layout, real editor tree)
/// returns None and is left untouched.
fn generic_text_box(cur: &str, content: &str) -> Option<String> {
    if cur.trim().is_empty() {
        return Some(text_box_json(content));
    }
    if crate::db::misc::body_is_gdlr(cur) {
        return None;
    }
    let serde_json::Value::Array(a) = serde_json::from_str(cur).ok()? else {
        return None;
    };
    if a.len() != 1
        || a[0].get("type").and_then(|t| t.as_str()) != Some("text-box")
        || a[0].get("value").and_then(|v| v.get("content")).and_then(|c| c.as_str()).is_none()
    {
        return None;
    }
    Some(text_box_json(content))
}

fn text_box_json(content: &str) -> String {
    serde_json::json!([{"template":"element","type":"text-box","value":{"content":content}}])
        .to_string()
}

/// sqlite-integration writes the real DB as `.ht.<hash>/.ht.sqlite` inside
/// the mounted database dir on the secret-path (Playground) layout; the
/// PhpCgi runtime instead defines `FQDB = <db_dir>/wp.db` explicitly and
/// the integration stores the file verbatim. Resolve lazily — the file
/// only exists after WP's first write.
pub(crate) fn locate(db_dir: &Path) -> Option<PathBuf> {
    let direct = db_dir.join("wp.db");
    if direct.exists() {
        return Some(direct);
    }
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
    // wp.db is concurrently written by the php -S sidecar — without a
    // timeout a pull during a PHP-side write fails BUSY and the caller
    // silently drops it (`let _ =` in mod.rs).
    src.busy_timeout(std::time::Duration::from_millis(3000))?;
    let dst = crate::db::conn(rpress_db)?;
    let tx = dst.unchecked_transaction()?;
    let mut copied = 0u64;
    let mut skipped = 0u64;
    // wp.db post_id is a WP ID — it resolves to a rpress row ONLY through
    // posts.wp_id. Falling back to raw rowid would stamp the write onto an
    // unrelated post (WP-created posts have no rpress row): skip + count.
    let resolve = |post_id: i64| -> rusqlite::Result<Option<i64>> {
        tx.query_row(
            sql!("bridge_pull__select_id_from_posts"),
            [post_id], |r| r.get(0),
        ).optional()
    };
    for s in specs {
        match s.table {
            PullTable::PostMeta => {
                // wp.db WP schema: wp_postmeta(post_id, meta_key,
                // meta_value). Only keys matching the prefix land;
                // rpress post_meta PK is (post_id, meta_key, meta_value)
                // so delete-then-insert = upsert semantics.
                let mut st = src.prepare(
                    sql!("bridge_pull__select_postmeta_like"),
                )?;
                let rows = st
                    .query_map([s.key_prefix], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (post_id, k, v) in rows {
                    let Some(rpress_id) = resolve(post_id)? else {
                        skipped += 1;
                        continue;
                    };
                    // OR REPLACE (not DELETE+INSERT): a WP post carrying
                    // multiple rows for one meta_key used to crash mid-tx
                    // after the DELETE already ran — half-applied pull.
                    tx.execute(
                        sql!("posts__insert_post_meta"),
                        rusqlite::params![rpress_id, k, v],
                    )?;
                    copied += 1;
                }
            }
            PullTable::Settings => {
                let mut st = src.prepare(
                    sql!("bridge_pull__select_options_like"),
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
                        sql!("bridge_pull__insert_or_replace_settings"),
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
                    sql!("bridge_pull__select_postmeta_eq"),
                )?;
                let rows = st
                    .query_map([s.key_prefix], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (post_id, v) in rows {
                    let Some(rpress_id) = resolve(post_id)? else {
                        skipped += 1;
                        continue;
                    };
                    let body = match crate::php_serialize::unserialize(v.as_bytes()) {
                        Ok(j) => j.to_string(),
                        Err(_) => v,
                    };
                    tx.execute(
                        sql!("bridge_pull__update_body_json"),
                        rusqlite::params![rpress_id, body],
                    )?;
                    copied += 1;
                }
            }
            PullTable::PostFields => {
                // wp_posts → posts: `post_content` → body_html,
                // `post_status` → status. If the current body_json is
                // the generic import wrapper it is rebuilt around the
                // new content so render stops serving a stale tree.
                let mut st = src.prepare(
                    sql!("bridge_pull__select_post_fields"),
                )?;
                let rows = st
                    .query_map([], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for (wp_id, content, status) in rows {
                    let Some(rpress_id) = resolve(wp_id)? else {
                        skipped += 1;
                        continue;
                    };
                    // A real read error must abort, not masquerade as an
                    // empty wrapper (which would rebuild body_json).
                    let cur: String = tx.query_row(
                        sql!("bridge_pull__select_post_body_json"),
                        [rpress_id], |r| r.get(0),
                    )?;
                    if let Some(body) = generic_text_box(&cur, &content) {
                        tx.execute(
                            sql!("bridge_pull__update_body_json"),
                            rusqlite::params![rpress_id, body],
                        )?;
                    }
                    tx.execute(
                        sql!("bridge_pull__update_post_fields"),
                        rusqlite::params![rpress_id, content, status],
                    )?;
                    copied += 1;
                }
            }
        }
    }
    tx.commit()?;
    if skipped > 0 {
        eprintln!("bridge pull: {copied} rows copied, {skipped} skipped (wp_id had no rpress post)");
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_rejects_bad_shapes() {
        let ok = WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: "gdlr-core", key_map: None };
        assert!(ok.validate().is_ok());
        for bad in ["%", "gdlr-%", "gdlr-%%", ""] {
            let s = WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: bad, key_map: None };
            assert!(s.validate().is_err(), "{bad} must be rejected");
        }
        let s = WriteSpec { table: PullTable::PostMeta, key_column: "posts", key_prefix: "x", key_map: None };
        assert!(s.validate().is_err());
        // BodyJson takes an exact meta_key — `%` is never a real key char.
        let exact = WriteSpec { table: PullTable::BodyJson, key_column: "meta_key", key_prefix: "gdlr-core-page-builder", key_map: None };
        assert!(exact.validate().is_ok());
        // `_` is a literal char now — `_elementor` is a valid prefix/key.
        let underscored = WriteSpec { table: PullTable::BodyJson, key_column: "meta_key", key_prefix: "_elementor_data", key_map: None };
        assert!(underscored.validate().is_ok());
        for bad in ["gdlr-%", "gdlr%"] {
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
            w.execute_batch(sql!("bridge_pull__test_seed_wpdb")).unwrap();
        }
        {
            let r = crate::db::conn(&rdb).unwrap();
            crate::schema::migrate_conn(&r).unwrap();
            r.execute(
                sql!("bridge_pull__test_insert_post"),
                [],
            ).unwrap();
            // posts.id in rpress == wp post_id only when wp_id is set;
            // projection writes wp_id as ID — pull sees the projected id,
            // which equals posts.id when wp_id==id (or map: pull joins on
            // posts.wp_id fallback below).
        }
        let specs = [
            WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: "gdlr-core", key_map: None },
            WriteSpec { table: PullTable::Settings, key_column: "key", key_prefix: "gdlr-core", key_map: Some("plugin:goodlayers:") },
            WriteSpec { table: PullTable::BodyJson, key_column: "meta_key", key_prefix: "gdlr-core-page-builder", key_map: None },
        ];
        pull(&wdb, &rdb, &specs).unwrap();
        let r = crate::db::conn(&rdb).unwrap();
        let metas: i64 = r.query_row(
            sql!("bridge_pull__test_count_meta_like"),
            [], |x| x.get(0),
        ).unwrap();
        assert_eq!(metas, 2);
        let secret: i64 = r.query_row(
            sql!("bridge_pull__test_count_meta_secret"), [], |x| x.get(0),
        ).unwrap();
        assert_eq!(secret, 0);
        let opt: i64 = r.query_row(
            sql!("bridge_pull__test_count_setting_plugin"), [], |x| x.get(0),
        ).unwrap();
        assert_eq!(opt, 1);
        let admin: i64 = r.query_row(
            sql!("bridge_pull__test_count_setting_admin"), [], |x| x.get(0),
        ).unwrap();
        assert_eq!(admin, 0);
        let bj: String = r.query_row(sql!("bridge_pull__test_select_body_json"), [], |x| x.get(0)).unwrap();
        assert_eq!(bj, "[{\"a\":1}]");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pull_skips_unknown_wp_id_and_survives_multivalued_meta() {
        // Regression: (a) a wp_id with no rpress post must SKIP, not stamp
        // meta onto an unrelated rowid; (b) duplicate (post_id,meta_key)
        // rows must OR REPLACE, not crash mid-tx.
        let dir = std::env::temp_dir().join(format!("rpress-pull2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let wdb = dir.join("wp.db");
        let rdb = dir.join("rpress.db");
        {
            let w = Connection::open(&wdb).unwrap();
            w.execute_batch(sql!("bridge_pull__test_seed_postmeta")).unwrap();
        }
        {
            let r = crate::db::conn(&rdb).unwrap();
            crate::schema::migrate_conn(&r).unwrap();
            // rpress post id=1 exists but wp_id=7 maps to id=2? No —
            // wp_id 7 resolves to id=1 via posts.wp_id; id=999 resolves
            // to nothing and must not touch post 999-th rowid either.
            r.execute(
                sql!("bridge_pull__test_insert_post_2"),
                [],
            ).unwrap();
        }
        let specs = [WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: "gdlr-core", key_map: None }];
        let copied = pull(&wdb, &rdb, &specs).unwrap();
        assert_eq!(copied, 2); // both rows for wp_id 7 (multi-value meta)
        let r = crate::db::conn(&rdb).unwrap();
        let vals: i64 = r.query_row(
            sql!("bridge_pull__test_count_meta"),
            [], |x| x.get(0),
        ).unwrap();
        assert_eq!(vals, 1, "multi-value meta collapses to one row");
        let ghost: i64 = r.query_row(
            sql!("bridge_pull__test_count_meta_ghost"), [], |x| x.get(0),
        ).unwrap();
        assert_eq!(ghost, 0, "unresolvable wp_id must not stamp meta anywhere");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pull_prefix_is_literal_underscore_not_wildcard() {
        // `_elementor` must match `_elementor*` rows only — `Xelementor_data`
        // would match under the old LIKE semantics and must not now.
        let dir = std::env::temp_dir().join(format!("rpress-pull3-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let wdb = dir.join("wp.db");
        let rdb = dir.join("rpress.db");
        {
            let w = Connection::open(&wdb).unwrap();
            w.execute_batch(sql!("bridge_pull__test_seed_elementor_meta")).unwrap();
        }
        {
            let r = crate::db::conn(&rdb).unwrap();
            crate::schema::migrate_conn(&r).unwrap();
            r.execute(sql!("bridge_pull__test_insert_post_2"), []).unwrap();
        }
        let specs = [WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: "_elementor", key_map: None }];
        let copied = pull(&wdb, &rdb, &specs).unwrap();
        assert_eq!(copied, 2, "only the two real _elementor rows land");
        let r = crate::db::conn(&rdb).unwrap();
        let n: i64 = r.query_row(
            sql!("bridge_pull__test_count_elementor_meta"), [], |x| x.get(0),
        ).unwrap();
        assert_eq!(n, 2, "Xelementor_data must not be pulled (literal `_` match)");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn postfields_pull_updates_html_status_and_regenerates_wrapper() {
        // PostFields: post_content → body_html, post_status → status;
        // generic text-box body_json is rebuilt, GDLR-shaped is not.
        let dir = std::env::temp_dir().join(format!("rpress-pull4-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let wdb = dir.join("wp.db");
        let rdb = dir.join("rpress.db");
        {
            let w = Connection::open(&wdb).unwrap();
            w.execute_batch(sql!("bridge_pull__test_seed_wp_posts")).unwrap();
        }
        {
            let r = crate::db::conn(&rdb).unwrap();
            crate::schema::migrate_conn(&r).unwrap();
            r.execute_batch(sql!("bridge_pull__test_seed_posts_shapes")).unwrap();
        }
        let specs = [WriteSpec { table: PullTable::PostFields, key_column: "meta_key", key_prefix: "", key_map: None }];
        let copied = pull(&wdb, &rdb, &specs).unwrap();
        assert_eq!(copied, 3, "ghost wp_id 999 skipped");
        let r = crate::db::conn(&rdb).unwrap();
        let row = |id: i64| r.query_row(
            sql!("bridge_pull__test_select_post_row"), [id],
            |x| Ok((x.get::<_, String>(0)?, x.get::<_, String>(1)?, x.get::<_, String>(2)?)),
        ).unwrap();
        let (html, status, json) = row(1);
        assert_eq!(html, "<div>edited</div>");
        assert_eq!(status, "publish");
        assert_eq!(json, "[{\"template\":\"element\",\"type\":\"text-box\",\"value\":{\"content\":\"<div>edited</div>\"}}]",
            "generic wrapper rebuilt around new post_content");
        let (html, status, json) = row(2);
        assert_eq!(html, "<p>gdlr</p>");
        assert_eq!(status, "draft");
        assert_eq!(json, "[{\"template\":\"element\",\"type\":\"slider\",\"value\":{}}]",
            "non-wrapper body_json untouched");
        let (html, _, json) = row(3);
        assert_eq!(html, "<p>fresh</p>");
        assert_eq!(json, "[{\"template\":\"element\",\"type\":\"text-box\",\"value\":{\"content\":\"<p>fresh</p>\"}}]",
            "empty body_json becomes a text-box wrapper");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
