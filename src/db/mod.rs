/// Compile-time SQL loader: `sql!("posts__select_id_kind")`
/// → contents of `src/db/sql/posts__select_id_kind.sql`.
/// SQL lives in .sql files, never inline, so it can be grepped, reviewed,
/// and formatted independently of Rust.
#[macro_export]
macro_rules! sql {
    ($name:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/db/sql/", $name, ".sql"))
    };
}

// Data access layer — WP `wp-includes/post.php` + `taxonomy.php` equivalent.
// Every SQL statement in the project lives under this module.
//
// Layout:
//   posts.rs  posts/posts_meta/tours: load, save, list, clone, status, quick/bulk update
//   terms.rs  taxonomy: terms table, post_terms assignments, choices
//   media.rs  media table + uploads: list/get/save/replace, image sizes, mime/dims
//   misc.rs   settings/plugin options, widgets, body_json (GoodLayers) helpers
//
// Shared helpers below are `pub(crate)` for the whole `db` tree.
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

pub mod misc;
pub mod media;
pub mod posts;
pub mod comments;
pub mod users;
pub mod submissions;
pub mod menus;
pub mod forms;
pub mod terms;

pub use misc::*;
pub use media::*;
pub use posts::*;
pub use terms::*;
#[allow(unused_imports)]
pub use comments::*;
pub use menus::*;

/// Open `db` with a 3s busy timeout — every write/reader in the project
/// goes through this so a concurrent `save_post` doesn't turn a public
/// render into SQLITE_BUSY. Transactions are BEGIN IMMEDIATE: the write
/// lock is taken up front, inside SQLite's busy-handler, so a concurrent
/// writer *waits out* the timeout instead of the BEGIN DEFERRED
/// read→write upgrade racing a peer and returning SQLITE_BUSY/
/// BUSY_SNAPSHOT immediately — busy_timeout does not cover
/// mid-transaction lock upgrades (parallel admin saves surfaced
/// "database is locked" 400s). Readers are unaffected: a tx that never
/// writes commits without ever needing the lock.
pub(crate) fn conn(db: &std::path::Path) -> Result<Connection> {
    let mut c = Connection::open(db)?;
    c.busy_timeout(std::time::Duration::from_millis(3000))?;
    c.set_transaction_behavior(rusqlite::TransactionBehavior::Immediate);
    // Runtime pragmas (phase-4): NORMAL is the standard WAL-safe mode —
    // corruption-proof, only the last commit can be lost on power fail.
    // temp_store keeps sorts in RAM; a 20MB page cache covers the working
    // set; mmap lets reads bypass page-cache copies on the ~100MB db;
    // a larger autocheckpoint amortizes WAL merge I/O during regen bursts.
    c.pragma_update(None, "synchronous", "NORMAL")?;
    c.pragma_update(None, "temp_store", "MEMORY")?;
    c.pragma_update(None, "cache_size", -20000i64)?;
    c.pragma_update(None, "mmap_size", 268_435_456i64)?;
    c.pragma_update(None, "wal_autocheckpoint", 4000i64)?;
    // FKs ON: posts.menu_id / theme_locations.menu_id declared ON DELETE
    // SET NULL were dead text — deleting a menu left dangling ids. Child
    // inserts must still reference existing parents; callers already do.
    c.pragma_update(None, "foreign_keys", "ON")?;
    Ok(c)
}

/// Alias of `conn` — kept so write-path call sites read intent. All
/// transactions are IMMEDIATE since phase-2.
#[allow(dead_code)]
pub(crate) fn conn_write(db: &std::path::Path) -> Result<Connection> {
    conn(db)
}


/// ASCII slugifier shared by quick-edit, tax save, and tag creation.
pub fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Look up display name for a slug: prefer `terms` registry, fall back to
/// any existing `post_terms` row.
pub(crate) fn term_name(tx: &Connection, tax: &str, slug: &str) -> Result<String> {
    let name: Option<String> = tx
        .query_row(
            sql!("mod__select_name_from_1"),
            params![slug, tax],
            |r| r.get(0),
        )
        .optional()?;
    Ok(name.unwrap_or_else(|| slug.to_string()))
}

/// Attach term `slug` of `tax` to a post in `post_terms`, and register the
/// term in `terms` when it doesn't exist yet.
pub(crate) fn attach_term(tx: &Connection, post_id: i64, tax: &str, slug: &str, name: &str) -> Result<()> {
    tx.execute(
        sql!("mod__insert_or_ignore_2"),
        params![post_id, tax, slug, name],
    )?;
    tx.execute(
        sql!("mod__insert_or_ignore_3"),
        params![tax, slug, name],
    )?;
    Ok(())
}

/// Re-parent children of comments matched by `victim_pred` (a constant SQL
/// predicate on `comments.id`, e.g. `"status=?1"` or `"status='trash' AND
/// created_at < …"`) up to their grandparent — `wp_delete_comment` orphans
/// semantics. Self-parented rows are skipped (they'd reparent to their own
/// parent forever); a cyclic parent chain can still spin a normal chain
/// forever, so passes are capped at 1000 and remaining rows are force-NULLed.
/// Runs on the caller's transaction.
pub(crate) fn reparent_orphans(
    tx: &Connection,
    victim_pred: &str,
    p: impl rusqlite::Params + Copy,
) -> Result<()> {
    let reparent = format!(
        "UPDATE comments SET parent_id = (SELECT c2.parent_id FROM comments c2 WHERE c2.id = comments.parent_id) \
         WHERE comments.id != comments.parent_id AND comments.parent_id IN (SELECT id FROM comments WHERE {victim_pred})"
    );
    for _ in 0..1000 {
        if tx.execute(&reparent, p)? == 0 {
            return Ok(());
        }
    }
    // Cycle: grandchildren in a parent loop can never escape the victim set.
    // Also NULL self-parented rows — a reparent pass can CREATE id==parent_id
    // rows the main loop's self-guard then excludes forever.
    tx.execute(
        &format!("UPDATE comments SET parent_id = NULL WHERE comments.parent_id IN (SELECT id FROM comments WHERE {victim_pred}) OR comments.parent_id = comments.id"),
        p,
    )?;
    Ok(())
}
