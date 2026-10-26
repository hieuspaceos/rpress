//! Comments data access — WP `comment.php`/`wp_delete_comment` equivalent.
//! `list`/`count` share one dynamic WHERE (mirrors `list_admin` in posts.rs);
//! the SELECT prefix lives in `comments__list_base.sql`, clauses are appended
//! here and remain parameterized.
use super::*;
use rusqlite::types::Value;
use std::path::Path;

#[derive(Default)]
pub struct Counts {
    /// Approved + pending (the WP "All" view — spam/trash excluded).
    pub all: i64,
    pub pending: i64,
    pub approved: i64,
    pub spam: i64,
    pub trash: i64,
    /// "Mine" view: rows by `mine_email` in approved/pending (see `counts_for`).
    pub mine: i64,
}

/// One `edit-comments.php` row: comment fields plus the parent post's
/// title/slug/kind/lang for the "In Response To" column.
pub struct CommentRow {
    pub id: i64,
    pub post_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub author: String,
    pub email: String,
    pub author_url: String,
    pub author_ip: String,
    pub body: String,
    pub status: String,
    pub created_at: String,
    pub post_title: String,
    pub post_slug: String,
    pub post_kind: String,
    pub post_lang: String,
    /// Approved-comment bubble on the In Response To cell.
    pub post_comments: i64,
    /// Pending-comment count on the In Response To cell (WP pending bubble).
    pub post_comments_pending: i64,
}
/// List filters: `status` is the view key ("all"/mine/pending/approved/spam/
/// trash); `s` matches author/email/url/ip/body. `limit`/`offset` come from
/// the pager. `by_author_email` adds `c.email = ?` — set only for the "mine"
/// view (comments carry no user FK, so email is the linkage).
pub struct Filter<'a> {
    pub status: &'a str,
    pub post_id: Option<i64>,
    pub s: &'a str,
    pub by_author_email: &'a str,
    pub limit: i64,
    pub offset: i64,
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<CommentRow> {
    Ok(CommentRow {
        id: r.get(0)?,
        post_id: r.get(1)?,
        parent_id: r.get(2)?,
        author: r.get(3)?,
        email: r.get(4)?,
        author_url: r.get(5)?,
        author_ip: r.get(6)?,
        body: r.get(7)?,
        status: r.get(8)?,
        created_at: r.get(9)?,
        post_title: r.get(10)?,
        post_slug: r.get(11)?,
        post_kind: r.get(12)?,
        post_lang: r.get(13)?,
        post_comments: r.get(14)?,
        post_comments_pending: r.get(15)?,
    })
}

/// WHERE clauses + args shared by `list` and `count`. `status` maps the view
/// keys to stored values; "all" and "mine" = everything except spam and
/// trash ("mine" additionally restricts to `by_author_email`).
fn where_parts<'a>(f: &Filter<'a>) -> (String, Vec<Value>) {
    let mut sql = String::new();
    let mut args: Vec<Value> = Vec::new();
    match f.status {
        "pending" | "approved" | "spam" | "trash" => {
            sql.push_str(" AND c.status=?");
            args.push(Value::Text(f.status.to_string()));
        }
        _ => sql.push_str(" AND c.status IN ('approved','pending')"),
    }
    if let Some(pid) = f.post_id {
        sql.push_str(" AND c.post_id=?");
        args.push(Value::Integer(pid));
    }
    if !f.s.is_empty() {
        sql.push_str(" AND (c.author LIKE ? OR c.email LIKE ? OR c.author_url LIKE ? OR c.author_ip LIKE ? OR c.body LIKE ?)");
        let pat = format!("%{}%", f.s);
        for _ in 0..5 {
            args.push(Value::Text(pat.clone()));
        }
    }
    if !f.by_author_email.is_empty() {
        sql.push_str(" AND c.email=?");
        args.push(Value::Text(f.by_author_email.to_string()));
    }
    (sql, args)
}

/// Sidebar badge + list chrome caller: no session email, `mine` stays 0.
pub fn counts(db: &Path) -> Result<Counts> {
    counts_for(db, "")
}

/// `mine_email` feeds the sixth count (the "Mine" view); '' → mine = 0.
pub fn counts_for(db: &Path, mine_email: &str) -> Result<Counts> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("comments__counts"), [mine_email], |r| {
        Ok(Counts {
            all: r.get::<_, Option<i64>>(0)?.unwrap_or(0),
            pending: r.get::<_, Option<i64>>(1)?.unwrap_or(0),
            approved: r.get::<_, Option<i64>>(2)?.unwrap_or(0),
            spam: r.get::<_, Option<i64>>(3)?.unwrap_or(0),
            trash: r.get::<_, Option<i64>>(4)?.unwrap_or(0),
            mine: r.get::<_, Option<i64>>(5)?.unwrap_or(0),
        })
    })?)
}

/// (rows, total) — `total` is the full filtered count, before paging.
pub fn list(db: &Path, f: &Filter) -> Result<(Vec<CommentRow>, i64)> {
    let c = crate::db::conn(db)?;
    let (w, mut args) = where_parts(f);
    let base = format!("{}{}", sql!("comments__list_base"), w);
    let total: i64 = {
        let mut st = c.prepare(&format!("SELECT COUNT(*) FROM ({base}) x"))?;
        st.query_row(rusqlite::params_from_iter(args.iter()), |r| r.get(0))?
    };
    let sql = format!("{base} ORDER BY c.created_at DESC, c.id DESC LIMIT ? OFFSET ?");
    args.push(Value::Integer(f.limit));
    args.push(Value::Integer(f.offset));
    let mut st = c.prepare(&sql)?;
    let rows = st
        .query_map(rusqlite::params_from_iter(args.iter()), row)?
        .flatten()
        .collect();
    Ok((rows, total))
}

pub fn get(db: &Path, id: i64) -> Result<Option<CommentRow>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("comments__get"), [id], row).optional()?)
}

/// `status` must be one of approved/pending/spam/trash — callers whitelist.
pub fn set_status(db: &Path, id: i64, status: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("comments__set_status"), params![status, id])?;
    Ok(())
}

/// Edit-screen + quick-edit save. Pass `created_at=None` to keep the stored
/// date unchanged (quick edit never touches status/date in WP).
pub fn update_fields(
    db: &Path,
    id: i64,
    author: &str,
    email: &str,
    url: &str,
    body: &str,
    status: &str,
    created_at: Option<&str>,
) -> Result<()> {
    let c = crate::db::conn(db)?;
    let ts = match created_at {
        Some(t) => t.to_string(),
        None => c.query_row(
            sql!("comments__select_created_at"),
            [id],
            |r| r.get(0),
        )?,
    };
    c.execute(
        sql!("comments__update_fields"),
        params![author, email, url, body, status, ts, id],
    )?;
    Ok(())
}

/// WP `wp_delete_comment`: children are re-attached to the deleted comment's
/// own parent, then the row goes — one transaction.
pub fn delete_permanent(db: &Path, id: i64) -> Result<()> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    tx.execute(sql!("comments__reparent_children"), [id])?;
    tx.execute(sql!("comments__delete"), [id])?;
    tx.commit()?;
    Ok(())
}

/// Admin reply: approved child of `parent_id`, on the parent's post, authored
/// by the session user (display_name → email fallback), stamped by SQLite.
/// `email` is the session user's — it keeps admin replies inside the "Mine"
/// view, which matches comments on `c.email`.
pub fn insert_reply(db: &Path, parent_id: i64, author: &str, email: &str, body: &str) -> Result<i64> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    let next: i64 = tx.query_row(sql!("intake__select_coalesce_max_3"), [], |r| r.get(0))?;
    tx.execute(
        sql!("comments__insert_reply"),
        params![next, parent_id, author, email, body],
    )?;
    tx.commit()?;
    Ok(next)
}

/// "Empty Spam"/"Empty Trash": hard-deletes every row in `status` after
/// re-parenting each victim's children to the victim's parent — same
/// `wp_delete_comment` semantics as single deletes. One transaction.
pub fn empty_status(db: &Path, status: &str) -> Result<u64> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    // Re-parent until no row references a victim (parents may chain-delete).
    while tx.execute(sql!("comments__reparent_orphans"), [status])? > 0 {}
    let n = tx.execute(sql!("comments__empty_status"), [status])?;
    tx.commit()?;
    Ok(n as u64)
}

/// Reply author name for the session user: display_name, else email.
pub fn user_display_name(db: &Path, user_id: i64, fallback: &str) -> String {
    crate::db::conn(db)
        .ok()
        .and_then(|c| {
            c.query_row(sql!("comments__select_user_display_name"), [user_id], |r| {
                r.get::<_, Option<String>>(0)
            })
            .ok()
            .flatten()
        })
        .unwrap_or_else(|| fallback.to_string())
}

/// Dashboard "Comments" widget: `limit` newest rows in the WP "All" view
/// (approved + pending — spam/trash excluded).
pub fn recent_comments(db: &Path, limit: i64) -> Result<Vec<CommentRow>> {
    let (rows, _) = list(
        db,
        &Filter {
            status: "all",
            post_id: None,
            s: "",
            by_author_email: "",
            limit,
            offset: 0,
        },
    )?;
    Ok(rows)
}

/// Newest pending comment, for the "Awaiting Moderation" preview.
pub fn first_pending(db: &Path) -> Result<Option<CommentRow>> {
    let (rows, _) = list(
        db,
        &Filter {
            status: "pending",
            post_id: None,
            s: "",
            by_author_email: "",
            limit: 1,
            offset: 0,
        },
    )?;
    Ok(rows.into_iter().next())
}

/// `settings` row read — per-user Screen Options JSON lives here
/// (`comment_screen_{user_id}`); `None` when unset.
pub fn setting_get(db: &Path, key: &str) -> Option<String> {
    crate::db::conn(db)
        .ok()?
        .query_row(sql!("render__select_value_from_settings"), [key], |r| r.get(0))
        .ok()
}

/// `settings` row upsert (INSERT OR REPLACE — `key` is the PK).
pub fn setting_set(db: &Path, key: &str, value: &str) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("import__insert_or_replace"), params![key, value])?;
    Ok(())
}
