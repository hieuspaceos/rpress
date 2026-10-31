pub const DDL: &str = sql!("schema__ddl");


/// Columns added after the initial DDL — `ALTER TABLE` migrations applied on
/// every `serve` startup (and import) so existing `.db` files upgrade in
/// place. Idempotent via `PRAGMA table_info`.
const MIGRATIONS: &[(&str, &str, &str)] = &[
    // (table, column, column DDL)
    ("users", "display_name", "TEXT"),
    ("users", "first_name", "TEXT"),
    ("users", "last_name", "TEXT"),
    ("users", "website", "TEXT"),
    ("comments", "author_url", "TEXT"),
    ("comments", "author_ip", "TEXT"),
    ("submissions", "form_slug", "TEXT"),
    ("terms", "parent_id", "INTEGER"),
    ("media", "backup_path", "TEXT"),
    ("menu_items", "title_attr", "TEXT NOT NULL DEFAULT ''"),
    ("menu_items", "target", "TEXT NOT NULL DEFAULT ''"),
    ("menu_items", "classes", "TEXT NOT NULL DEFAULT ''"),
    ("menu_items", "mega", "TEXT NOT NULL DEFAULT ''"),
    ("menu_items", "xfn", "TEXT NOT NULL DEFAULT ''"),
    ("menu_items", "description", "TEXT NOT NULL DEFAULT ''"),
    ("widgets", "lang", "TEXT NOT NULL DEFAULT ''"),
    ("menus", "auto_add", "INTEGER NOT NULL DEFAULT 0"),
    ("sessions", "expires_at", "INTEGER NOT NULL DEFAULT 0"),
    ("sessions", "csrf", "TEXT NOT NULL DEFAULT ''"),
    ("users", "totp_secret", "TEXT NOT NULL DEFAULT ''"),
    ("users", "totp_enabled", "INTEGER NOT NULL DEFAULT 0"),
    ("users", "totp_last_step", "INTEGER NOT NULL DEFAULT -1"),
    ("users", "backup_codes", "TEXT NOT NULL DEFAULT ''"),
];

/// Ordered post-DDL migration steps, recorded in `schema_migrations` so a
/// step runs exactly once per database. Unlike `MIGRATIONS` (additive
/// column triples re-probed every boot), these cover non-additive changes
/// — rebuilds, drops, backfills — where "did it already run" must not be
/// inferred from schema shape.
enum Migration {
    /// Pure SQL batch (idempotent required — a crash between steps in the
    /// same tx is all-or-nothing, but a crash after commit + before the
    /// bookkeeping row must not re-run destructively).
    Sql(&'static str),
    /// Rust step for probes/backfills that SQL alone can't express.
    Fn(fn(&rusqlite::Connection) -> anyhow::Result<()>),
}

const STEPS: &[(&str, Migration)] = &[
    ("m01_dedupe_post_meta", Migration::Fn(|c| dedupe_post_meta(c))),
    ("m02_rebuild_theme_locations_lang", Migration::Fn(|c| rebuild_theme_locations(c))),
    ("m03_backfill_sessions_expiry", Migration::Sql(sql!("schema__backfill_sessions_expiry"))),
    ("m04_backfill_posts_body_html", Migration::Fn(|c| { backfill_body_html(c); Ok(()) })),
    ("m05_drop_post_translations", Migration::Fn(|c| drop_post_translations(c))),
];

/// `schema_migrations` bookkeeping — created before STEPS run so the
/// applied-set is itself transaction-protected.
fn ensure_migrations_table(c: &rusqlite::Connection) -> anyhow::Result<()> {
    c.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations(
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )?;
    Ok(())
}

/// True when `name` was already applied (or backfilled as applied).
fn step_applied(c: &rusqlite::Connection, name: &str) -> bool {
    c.query_row(
        "SELECT 1 FROM schema_migrations WHERE name=?1",
        [name],
        |_| Ok(()),
    )
    .is_ok()
}

/// `theme_locations` pre-lang rebuild: the old PK (location) can't hold
/// Polylang's per-language nav_menu_locations bindings. Detect the legacy
/// shape (no `lang` column) and rebuild — lang backfills from the assigned
/// menu's own lang. No-op on new-shape tables.
fn rebuild_theme_locations(c: &rusqlite::Connection) -> anyhow::Result<()> {
    let has_lang: bool = c
        .prepare(sql!("schema__pragma_theme_locations"))?
        .query_map([], |r| r.get::<_, String>(1))?
        .any(|n| n.map(|n| n == "lang").unwrap_or(false));
    let has_table: bool = c
        .prepare(sql!("schema__select_theme_locations_exists"))?
        .exists([])?;
    if has_table && !has_lang {
        c.execute_batch(sql!("schema__rebuild_theme_locations"))?;
    }
    Ok(())
}

/// Legacy-DB convergence: dbs migrated before the runner existed have the
/// STEPS' effects already applied but no bookkeeping rows. Re-probe the
/// same conditions the steps use and record the no-op steps as applied so
/// every db converges on the same `schema_migrations` contents.
fn backfill_applied(c: &rusqlite::Connection) -> anyhow::Result<()> {
    // m01: dedupe is a no-op once post_meta is clean; detect by dup scan.
    if !step_applied(c, "m01_dedupe_post_meta") {
        let has_dups: bool = c
            .prepare("SELECT 1 FROM post_meta GROUP BY post_id, meta_key HAVING COUNT(*)>1 LIMIT 1")
            .and_then(|mut s| s.exists([]))
            .unwrap_or(false);
        if !has_dups {
            c.execute("INSERT OR IGNORE INTO schema_migrations(name) VALUES('m01_dedupe_post_meta')", [])?;
        }
    }
    // m02: applied iff theme_locations has the lang column (or table absent).
    if !step_applied(c, "m02_rebuild_theme_locations_lang") {
        let has_lang: bool = c
            .prepare(sql!("schema__pragma_theme_locations"))?
            .query_map([], |r| r.get::<_, String>(1))?
            .any(|n| n.map(|n| n == "lang").unwrap_or(false));
        let has_table: bool = c
            .prepare(sql!("schema__select_theme_locations_exists"))?
            .exists([])?;
        if !has_table || has_lang {
            c.execute("INSERT OR IGNORE INTO schema_migrations(name) VALUES('m02_rebuild_theme_locations_lang')", [])?;
        }
    }
    Ok(())
}

/// Pre-DDL cleanup hook: `post_meta` predates the (post_id, meta_key)
/// unique index, so existing dbs may hold dup rows that would make
/// `CREATE UNIQUE INDEX` fail. Dedupes BEFORE `execute_batch(DDL)` runs.
/// Keeper: MIN(rowid) — readers do `LIMIT 1` with no ORDER BY and SQLite's
/// rowid-scan winner is the earliest row, so this preserves whatever value
/// pages most likely rendered before (documented behavior change).
fn dedupe_post_meta(c: &rusqlite::Connection) -> anyhow::Result<()> {
    let exists: bool = c
        .query_row(
            sql!("schema__count_post_meta_table"),
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .unwrap_or(false);
    if !exists {
        return Ok(());
    }
    // Keeper per (post_id, meta_key) group = the row every `LIMIT 1` reader
    // already serves: the covering index sqlite_autoindex_post_meta_1
    // (post_id, meta_key, meta_value) returns MIN(meta_value); MIN(rowid)
    // breaks ties. Verified on work/*.db copies: 65/100 groups disagreed
    // with a plain MIN(rowid) keeper, 0/100 with this one.
    let n = c.execute(
        sql!("schema__delete_post_meta_dupes"),
        [],
    )?;
    if n > 0 {
        eprintln!("migrate: deduped {n} post_meta row(s) before unique index");
    }
    Ok(())
}

/// `m05`: retire the legacy fr/es pair table — backfill any pair rows not
/// already covered by `post_translation_members` (one group per pair),
/// assert coverage, then DROP. Panic-loud on mismatch: never lose a link
/// silently.
fn drop_post_translations(c: &rusqlite::Connection) -> anyhow::Result<()> {
    let has_table: bool = c
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='post_translations'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .is_ok();
    if !has_table {
        return Ok(());
    }
    // Backfill: each (fr,es) pair → group `mig-{fr}-{es}` unless one of the
    // posts already sits in a group (then that group adopts both).
    let pairs: Vec<(Option<i64>, Option<i64>)> = c
        .prepare("SELECT fr_post_id, es_post_id FROM post_translations")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .flatten()
        .collect();
    for (fr, es) in &pairs {
        let group: Option<String> = [fr, es]
            .into_iter()
            .flatten()
            .find_map(|pid| {
                c.query_row(
                    "SELECT group_id FROM post_translation_members WHERE post_id=?1",
                    [pid],
                    |r| r.get::<_, String>(0),
                )
                .ok()
            });
        let group = match group {
            Some(g) => g,
            None => format!("mig-{}-{}", fr.unwrap_or(0), es.unwrap_or(0)),
        };
        for (lang, pid) in [("fr", *fr), ("es", *es)].into_iter().flat_map(|(l, p)| p.map(|p| (l, p))) {
            // Only backfill when the post's lang matches the slot — stale
            // pair rows pointing at same-lang posts are dropped, not kept.
            let ok: bool = c
                .query_row("SELECT 1 FROM posts WHERE id=?1 AND lang=?2", rusqlite::params![pid, lang], |_| Ok(()))
                .is_ok();
            if ok {
                c.execute(
                    "INSERT OR IGNORE INTO post_translation_members(group_id, lang, post_id) VALUES(?1,?2,?3)",
                    rusqlite::params![group, lang, pid],
                )?;
            }
        }
    }
    // Coverage assert: every pair's members must now resolve inside a
    // shared group (or have been skipped as stale/ghost).
    let uncovered: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM post_translations t WHERE
             (t.fr_post_id IS NOT NULL AND EXISTS(SELECT 1 FROM posts WHERE id=t.fr_post_id AND lang='fr')
              AND t.fr_post_id NOT IN (SELECT post_id FROM post_translation_members))
          OR (t.es_post_id IS NOT NULL AND EXISTS(SELECT 1 FROM posts WHERE id=t.es_post_id AND lang='es')
              AND t.es_post_id NOT IN (SELECT post_id FROM post_translation_members))",
            [],
            |r| r.get(0),
        )?;
    anyhow::ensure!(
        uncovered == 0,
        "post_translations backfill incomplete: {uncovered} pair member(s) unresolved — refusing to drop"
    );
    c.execute_batch("DROP TABLE post_translations")?;
    Ok(())
}

/// Apply DDL + additive migrations on an open connection.
/// Apply DDL + additive migrations on an open connection. One transaction:
/// a crash between `ALTER` steps used to leave a half-migrated schema that
/// the idempotent probes could not see or repair — now it's all-or-nothing.
pub fn migrate_conn(c: &rusqlite::Connection) -> anyhow::Result<()> {
    // WAL must be set OUTSIDE any transaction (SQLite forbids switching
    // journal_mode inside one) — it used to live in DDL, which broke the
    // moment this block became transactional.
    c.pragma_update(None, "journal_mode", "WAL")?;
    // Bookkeeping table + legacy-backfill BEFORE the first step tx — its
    // presence is a precondition for step_applied, not part of any step.
    ensure_migrations_table(c)?;
    backfill_applied(c)?;
    let tx = c.unchecked_transaction()?;
    // PRE-DDL steps: dedupe + rebuild must land before execute_batch(DDL)
    // — the unique index fails on dup post_meta, and CREATE IF NOT EXISTS
    // can't fix an old-shape theme_locations. Run them inside this tx and
    // record as applied so the STEPS loop below skips them.
    if !step_applied(c, "m01_dedupe_post_meta") {
        dedupe_post_meta(&tx)?;
        tx.execute("INSERT OR IGNORE INTO schema_migrations(name) VALUES('m01_dedupe_post_meta')", [])?;
    }
    if !step_applied(c, "m02_rebuild_theme_locations_lang") {
        rebuild_theme_locations(&tx)?;
        tx.execute("INSERT OR IGNORE INTO schema_migrations(name) VALUES('m02_rebuild_theme_locations_lang')", [])?;
    }
    tx.execute_batch(DDL)?;
    for (table, col, ddl) in MIGRATIONS {
        let exists: bool = tx
            .prepare(&format!("{}{table})", sql!("schema__pragma_table_info")))?
            .query_map([], |r| r.get::<_, String>(1))?
            .any(|n| n.map(|n| n == *col).unwrap_or(false));
        if !exists {
            tx.execute_batch(&format!("{} {table} ADD COLUMN {col} {ddl}", sql!("schema__alter_table_add_column")))?;
        }
    }
    tx.commit()?;
    // Named steps run AFTER the DDL tx, each in its own transaction so a
    // step commits atomically (a destructive step can't half-land). Each
    // records itself in schema_migrations inside the same tx — replay is
    // impossible by construction (name UNIQUE + checked before run).
    for (name, step) in STEPS {
        if step_applied(c, name) {
            continue;
        }
        let tx = c.unchecked_transaction()?;
        match step {
            Migration::Sql(batch) => tx.execute_batch(batch)?,
            Migration::Fn(f) => f(&tx)?,
        }
        tx.execute(
            "INSERT INTO schema_migrations(name) VALUES(?1)",
            [name],
        )?;
        tx.commit()?;
    }
    Ok(())
}

/// One-time `posts.body_html` backfill. For non-GDLR bodies the imported
/// `body_json` is `[{type:"text-box", value:{content}}]` — its content *is*
/// `post_content`. GDLR bodies are the builder layout; their real
/// `post_content` was dropped at import time and only a re-import can
/// restore it.
fn backfill_body_html(c: &rusqlite::Connection) {
    use serde_json::Value;
    let rows: Vec<(i64, String)> = {
        let mut st = match c.prepare(sql!("schema__select_empty_body_html")) {
            Ok(s) => s,
            Err(_) => return,
        };
        st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map(|m| m.flatten().collect())
            .unwrap_or_default()
    };
    for (id, body) in rows {
        // skip gdlr layouts — their text-boxes are layout content, not
        // post_content.
        if crate::db::misc::body_is_gdlr(&body) {
            continue;
        }
        let Ok(Value::Array(a)) = serde_json::from_str::<Value>(&body) else { continue };
        if a.len() != 1 {
            continue;
        }
        let Some(content) = a[0]
            .get("value")
            .and_then(|v| v.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
        else {
            continue;
        };
        let _ = c.execute(sql!("schema__update_post_body_html"), rusqlite::params![content, id]);
    }
}

/// Run migrations on `db`. Safe on fresh and existing files.
pub fn migrate(db: &std::path::Path) -> anyhow::Result<()> {
    migrate_conn(&crate::db::conn(db)?)
}

#[cfg(test)]
mod tests {
    /// Simulates a pre-M18 `.db`: post_meta WITHOUT the unique index but
    /// WITH dup (post_id, meta_key) rows — `migrate_conn` must dedupe them
    /// before the DDL's CREATE UNIQUE INDEX would otherwise fail.
    #[test]
    fn migrate_dedupes_post_meta_and_indexes() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        // Old-shape table: PK includes meta_value, so dups per key exist.
        c.execute_batch(sql!("schema__test_ddl_post_meta"))
        .unwrap();
        super::migrate_conn(&c).unwrap();
        let rows: Vec<(i64, String, String)> = c
            .prepare(sql!("schema__test_select_post_meta_ordered"))
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        // MIN(meta_value) keeper: 'aaa' survives, not first-inserted 'zzz'.
        assert_eq!(
            rows,
            vec![
                (7, "a".to_string(), "aaa".to_string()),
                (7, "b".to_string(), "x".to_string()),
                (8, "a".to_string(), "y".to_string()),
            ]
        );
        // Unique index landed and now rejects a fresh dup pair.
        assert!(
            c.execute(sql!("schema__test_insert_post_meta"), [])
                .is_err(),
            "unique index must reject dup (post_id, meta_key)"
        );
        // Second migrate is a clean no-op.
        super::migrate_conn(&c).unwrap();
    }

    /// `sessions.csrf` exists on both fresh DDL and migrated legacy tables.
    #[test]
    fn sessions_csrf_column_added() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch(sql!("schema__test_ddl_sessions"))
        .unwrap();
        super::migrate_conn(&c).unwrap();
        let cols: Vec<String> = c
            .prepare(sql!("schema__pragma_sessions"))
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert!(cols.iter().any(|n| n == "csrf"), "csrf column missing: {cols:?}");
    }
}
