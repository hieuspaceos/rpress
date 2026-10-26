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
//   misc.rs   media uploads, menus, body_json (GoodLayers) helpers
//
// Shared helpers below are `pub(crate)` for the whole `db` tree.
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

pub mod misc;
pub mod posts;
pub mod comments;
pub mod users;
pub mod submissions;
pub mod menus;
pub mod forms;
pub mod terms;

pub use misc::*;
pub use posts::*;
pub use terms::*;
#[allow(unused_imports)]
pub use comments::*;
pub use menus::*;

/// Open `db` with a 3s busy timeout — every write/reader in the project
/// goes through this so a concurrent `save_post` doesn't turn a public
/// render into SQLITE_BUSY.
pub(crate) fn conn(db: &std::path::Path) -> Result<Connection> {
    let c = Connection::open(db)?;
    c.busy_timeout(std::time::Duration::from_millis(3000))?;
    Ok(c)
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
