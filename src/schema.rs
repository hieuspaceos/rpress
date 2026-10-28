pub const DDL: &str = r#"
PRAGMA journal_mode=WAL;

CREATE TABLE IF NOT EXISTS posts(
  id INTEGER PRIMARY KEY,
  wp_id INTEGER UNIQUE,
  slug TEXT,
  lang TEXT,
  title TEXT,
  body_json TEXT,
  body_html TEXT,
  excerpt TEXT,
  status TEXT,
  parent_id INTEGER,
  menu_order INTEGER,
  published_at TEXT,
  modified_at TEXT,
  seo_title TEXT,
  seo_desc TEXT,
  seo_jsonld TEXT,
  featured_media_id INTEGER,
  kind TEXT,
  template TEXT
);

CREATE TABLE IF NOT EXISTS tours(
  post_id INTEGER PRIMARY KEY,
  duration_days INTEGER,
  departure_dates_json TEXT,
  price_range TEXT,
  max_people INTEGER,
  tour_time TEXT,
  gallery_json TEXT,
  rating REAL,
  view_count INTEGER
);


CREATE TABLE IF NOT EXISTS post_terms(
  post_id INTEGER,
  taxonomy TEXT,
  slug TEXT,
  name TEXT,
  PRIMARY KEY(post_id, taxonomy, slug)
);
CREATE INDEX IF NOT EXISTS idx_post_terms_tax ON post_terms(taxonomy, slug);
CREATE INDEX IF NOT EXISTS idx_post_terms_post ON post_terms(post_id, taxonomy);

CREATE TABLE IF NOT EXISTS terms(
  id INTEGER PRIMARY KEY,
  taxonomy TEXT NOT NULL,
  slug TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT DEFAULT '',
  parent_id INTEGER,
  UNIQUE(taxonomy, slug)
);

CREATE TABLE IF NOT EXISTS post_translations(
  fr_post_id INT,
  es_post_id INT
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_trans_pair ON post_translations(fr_post_id, es_post_id);
CREATE INDEX IF NOT EXISTS idx_trans_es ON post_translations(es_post_id);

CREATE TABLE IF NOT EXISTS post_translation_members(
  group_id TEXT NOT NULL,
  lang TEXT NOT NULL,
  post_id INT NOT NULL,
  PRIMARY KEY(group_id, lang),
  UNIQUE(post_id)
);
CREATE INDEX IF NOT EXISTS idx_post_translation_members_group ON post_translation_members(group_id);
CREATE INDEX IF NOT EXISTS idx_post_translation_members_lang ON post_translation_members(lang);

CREATE TABLE IF NOT EXISTS term_translations(
  fr_term_id INTEGER,
  es_term_id INTEGER,
  PRIMARY KEY(fr_term_id)
);

CREATE TABLE IF NOT EXISTS menus(
  id INTEGER PRIMARY KEY,
  wp_term_id INTEGER UNIQUE,
  name TEXT,
  location TEXT,
  lang TEXT,
  auto_add INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS menu_items(
  id INTEGER PRIMARY KEY,
  wp_id INTEGER UNIQUE,
  menu_id INTEGER,
  parent_id INTEGER,
  ord INTEGER,
  label TEXT,
  url TEXT,
  object_id INTEGER,
  item_type TEXT,
  title_attr TEXT NOT NULL DEFAULT '',
  target TEXT NOT NULL DEFAULT '',
  classes TEXT NOT NULL DEFAULT '',
  mega TEXT NOT NULL DEFAULT '',
  xfn TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS theme_locations(
  location TEXT NOT NULL,
  lang TEXT NOT NULL DEFAULT '',
  menu_id INTEGER REFERENCES menus(id) ON DELETE SET NULL,
  PRIMARY KEY(location, lang)
);

CREATE TABLE IF NOT EXISTS media(
  id INTEGER PRIMARY KEY,
  wp_id INTEGER UNIQUE,
  path TEXT,
  title TEXT,
  alt TEXT,
  caption TEXT,
  description TEXT,
  mime TEXT,
  w INTEGER,
  h INTEGER,
  sizes_json TEXT,
  owner TEXT,
  created TEXT
);

CREATE TABLE IF NOT EXISTS comments(
  id INTEGER PRIMARY KEY,
  wp_id INTEGER UNIQUE,
  post_id INTEGER,
  parent_id INTEGER,
  author TEXT,
  email TEXT,
  body TEXT,
  status TEXT,
  created_at TEXT,
  rating INTEGER
);

CREATE TABLE IF NOT EXISTS review_assignments(
  review_id INTEGER,
  post_id INTEGER,
  PRIMARY KEY(review_id, post_id)
);

CREATE TABLE IF NOT EXISTS redirects(
  old_slug TEXT,
  new_url TEXT,
  source TEXT
);
CREATE INDEX IF NOT EXISTS idx_redirects_old ON redirects(old_slug);

CREATE TABLE IF NOT EXISTS revisions(
  id INTEGER PRIMARY KEY,
  post_id INTEGER,
  saved_at TEXT,
  blob TEXT
);

CREATE TABLE IF NOT EXISTS users(
  id INTEGER PRIMARY KEY,
  email TEXT UNIQUE,
  pass_hash TEXT,
  role TEXT,
  created_at TEXT
);

CREATE TABLE IF NOT EXISTS submissions(
  id INTEGER PRIMARY KEY,
  kind TEXT,
  name TEXT,
  page TEXT,
  tour TEXT,
  data_json TEXT,
  ip TEXT,
  status TEXT,
  created_at TEXT
);
CREATE TABLE IF NOT EXISTS forms(
  id            INTEGER PRIMARY KEY,
  slug          TEXT NOT NULL UNIQUE,
  title         TEXT NOT NULL,
  lang          TEXT NOT NULL DEFAULT 'fr',
  kind          TEXT NOT NULL DEFAULT 'form',
  markup        TEXT NOT NULL,
  mail_to       TEXT NOT NULL DEFAULT '',
  mail_from     TEXT NOT NULL DEFAULT '',
  mail_subject  TEXT NOT NULL DEFAULT '',
  mail_body     TEXT NOT NULL DEFAULT '',
  mail2_active  INTEGER NOT NULL DEFAULT 0,
  mail2_to      TEXT NOT NULL DEFAULT '',
  mail2_subject TEXT NOT NULL DEFAULT '',
  mail2_body    TEXT NOT NULL DEFAULT '',
  messages      TEXT NOT NULL DEFAULT '{}',
  additional    TEXT NOT NULL DEFAULT '',
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings(
  key TEXT PRIMARY KEY,
  value TEXT
);

CREATE TABLE IF NOT EXISTS post_meta(
  post_id INTEGER,
  meta_key TEXT,
  meta_value TEXT,
  PRIMARY KEY(post_id, meta_key, meta_value)
);

CREATE TABLE IF NOT EXISTS app_passwords(
  id         INTEGER PRIMARY KEY,
  user_id    INTEGER NOT NULL,
  name       TEXT NOT NULL,
  hash       TEXT NOT NULL,
  created_at TEXT NOT NULL,
  last_used  TEXT,
  last_ip    TEXT,
  UNIQUE(user_id, name)
);
CREATE INDEX IF NOT EXISTS idx_app_passwords_hash ON app_passwords(hash);

CREATE TABLE IF NOT EXISTS sessions(
  token      TEXT PRIMARY KEY,
  user_id    INTEGER NOT NULL,
  ua         TEXT NOT NULL,
  ip         TEXT NOT NULL,
  csrf       TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  last_seen  TEXT NOT NULL,
  expires_at INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);

-- WP-parity widget model (see plans/261026-0426-wp-parity-widgets): the
-- sidebars/widgets render pipeline keeps every frozen sidebar/footer
-- fragment in row-per-instance storage — a single settings row would
-- exceed the ~50KB audit threshold and make save/delete/order non-atomic.
CREATE TABLE IF NOT EXISTS sidebars(
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  lang TEXT NOT NULL DEFAULT 'all'
);

CREATE TABLE IF NOT EXISTS widgets(
  id TEXT PRIMARY KEY,
  sidebar_id TEXT NOT NULL,
  type TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  settings_json TEXT NOT NULL DEFAULT '{}',
  ord INTEGER NOT NULL DEFAULT 0,
  lang TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_widgets_sidebar ON widgets(sidebar_id, ord);
CREATE INDEX IF NOT EXISTS idx_post_meta_key ON post_meta(meta_key);
-- post_meta got a (post_id, meta_key) uniqueness rule (audit M18): readers
-- do `LIMIT 1` per key, so dup rows were dead weight with an arbitrary
-- winner. Existing dbs are deduped in `dedupe_post_meta` BEFORE this DDL
-- runs — the keeper is MIN(rowid), the most likely old LIMIT-1 winner
-- (rowids are assigned in insert order). Behavior change: later insert
-- attempts for an existing (post_id, meta_key) are now ignored/error.
CREATE UNIQUE INDEX IF NOT EXISTS idx_post_meta_pair ON post_meta(post_id, meta_key);
-- Lookup indexes (audit M18): comment threads per post / parent chain,
-- revision lists per post, and the kind+status admin/front filters.
CREATE INDEX IF NOT EXISTS idx_comments_post ON comments(post_id);
CREATE INDEX IF NOT EXISTS idx_comments_parent ON comments(parent_id);
CREATE INDEX IF NOT EXISTS idx_revisions_post ON revisions(post_id);
CREATE INDEX IF NOT EXISTS idx_posts_kind_status_lang ON posts(kind, status, lang);
"#;


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
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='post_meta'",
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
        "DELETE FROM post_meta WHERE rowid IN (\
            SELECT rowid FROM (\
                SELECT rowid, ROW_NUMBER() OVER (\
                    PARTITION BY post_id, meta_key \
                    ORDER BY meta_value IS NOT NULL, meta_value, rowid) rn \
                FROM post_meta) \
            WHERE rn > 1)",
        [],
    )?;
    if n > 0 {
        eprintln!("migrate: deduped {n} post_meta row(s) before unique index");
    }
    Ok(())
}

/// Apply DDL + additive migrations on an open connection.
pub fn migrate_conn(c: &rusqlite::Connection) -> anyhow::Result<()> {
    dedupe_post_meta(c)?;
    /* theme_locations pre-lang rebuild: the old PK (location) can't hold
       Polylang's per-language nav_menu_locations bindings. Detect the
       legacy shape (no `lang` column) and rebuild — lang backfills from
       the assigned menu's own lang. */
    {
        let has_lang: bool = c
            .prepare("PRAGMA table_info(theme_locations)")?
            .query_map([], |r| r.get::<_, String>(1))?
            .any(|n| n.map(|n| n == "lang").unwrap_or(false));
        let has_table: bool = c
            .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='theme_locations'")?
            .exists([])?;
        if has_table && !has_lang {
            c.execute_batch(
                "ALTER TABLE theme_locations RENAME TO theme_locations_old;
                 CREATE TABLE theme_locations(
                   location TEXT NOT NULL,
                   lang TEXT NOT NULL DEFAULT '',
                   menu_id INTEGER REFERENCES menus(id) ON DELETE SET NULL,
                   PRIMARY KEY(location, lang)
                 );
                 INSERT INTO theme_locations(location, lang, menu_id)
                   SELECT o.location, COALESCE((SELECT lang FROM menus WHERE id=o.menu_id),''), o.menu_id
                   FROM theme_locations_old o;
                 DROP TABLE theme_locations_old;",
            )?;
        }
    }
    c.execute_batch(DDL)?;
    for (table, col, ddl) in MIGRATIONS {
        let exists: bool = c
            .prepare(&format!("PRAGMA table_info({table})"))?
            .query_map([], |r| r.get::<_, String>(1))?
            .any(|n| n.map(|n| n == *col).unwrap_or(false));
        if !exists {
            c.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {col} {ddl}"))?;
        }
    }
    // Backfill sessions.expires_at (added above as 0): give live rows the
    // rolling 12h window from last_seen so migration never logs anyone out;
    // rows already idle >12h stay expired (0 < unixepoch()).
    c.execute_batch(
        "UPDATE sessions SET expires_at=unixepoch(last_seen)+43200 WHERE expires_at=0",
    )?;
    // Backfill posts.body_html (wp post_content) for rows imported before
    // the field existed: non-gdlr bodies are a single text-box wrapper —
    // lift its content out; gdlr rows stay empty (their post_content was
    // never imported; a fresh `rpress import` fills it).
    backfill_body_html(c);
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
        let mut st = match c.prepare("SELECT id, COALESCE(body_json,'') FROM posts WHERE COALESCE(body_html,'')='' AND COALESCE(body_json,'')!=''") {
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
        let _ = c.execute("UPDATE posts SET body_html=?1 WHERE id=?2", rusqlite::params![content, id]);
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
        c.execute_batch(
            "CREATE TABLE post_meta(post_id INTEGER, meta_key TEXT, meta_value TEXT,
                PRIMARY KEY(post_id, meta_key, meta_value));
             -- 'zzz' has the SMALLER rowid but the LARGER meta_value: the
             -- covering index serves 'aaa' to LIMIT-1 readers, so 'aaa'
             -- must be the keeper (not MIN(rowid)).
             INSERT INTO post_meta VALUES(7,'a','zzz'),(7,'a','aaa'),(7,'b','x'),(8,'a','y');",
        )
        .unwrap();
        super::migrate_conn(&c).unwrap();
        let rows: Vec<(i64, String, String)> = c
            .prepare("SELECT post_id, meta_key, meta_value FROM post_meta ORDER BY post_id, meta_key")
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
            c.execute("INSERT INTO post_meta VALUES(7,'a','third')", [])
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
        c.execute_batch(
            "CREATE TABLE sessions(token TEXT PRIMARY KEY, user_id INTEGER NOT NULL,
                ua TEXT NOT NULL, ip TEXT NOT NULL, created_at TEXT NOT NULL,
                last_seen TEXT NOT NULL, expires_at INTEGER NOT NULL DEFAULT 0);",
        )
        .unwrap();
        super::migrate_conn(&c).unwrap();
        let cols: Vec<String> = c
            .prepare("PRAGMA table_info(sessions)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert!(cols.iter().any(|n| n == "csrf"), "csrf column missing: {cols:?}");
    }
}
