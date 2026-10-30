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

/// Apply DDL + additive migrations on an open connection.
/// Apply DDL + additive migrations on an open connection. One transaction:
/// a crash between `ALTER` steps used to leave a half-migrated schema that
/// the idempotent probes could not see or repair — now it's all-or-nothing.
pub fn migrate_conn(c: &rusqlite::Connection) -> anyhow::Result<()> {
    // WAL must be set OUTSIDE any transaction (SQLite forbids switching
    // journal_mode inside one) — it used to live in DDL, which broke the
    // moment this block became transactional.
    c.pragma_update(None, "journal_mode", "WAL")?;
    let tx = c.unchecked_transaction()?;
    dedupe_post_meta(&tx)?;
    /* theme_locations pre-lang rebuild: the old PK (location) can't hold
       Polylang's per-language nav_menu_locations bindings. Detect the
       legacy shape (no `lang` column) and rebuild — lang backfills from
       the assigned menu's own lang. */
    {
        let has_lang: bool = tx
            .prepare(sql!("schema__pragma_theme_locations"))?
            .query_map([], |r| r.get::<_, String>(1))?
            .any(|n| n.map(|n| n == "lang").unwrap_or(false));
        let has_table: bool = tx
            .prepare(sql!("schema__select_theme_locations_exists"))?
            .exists([])?;
        if has_table && !has_lang {
            tx.execute_batch(sql!("schema__rebuild_theme_locations"))?;
        }
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
    // Backfill sessions.expires_at (added above as 0): give live rows the
    // rolling 12h window from last_seen so migration never logs anyone out;
    // rows already idle >12h stay expired (0 < unixepoch()).
    tx.execute_batch(sql!("schema__backfill_sessions_expiry"))?;
    // Backfill posts.body_html (wp post_content) for rows imported before
    // the field existed: non-gdlr bodies are a single text-box wrapper —
    // lift its content out; gdlr rows stay empty (their post_content was
    // never imported; a fresh `rpress import` fills it).
    backfill_body_html(&tx);
    tx.commit()?;
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
