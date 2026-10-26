//! `forms` table — CF7-style form definitions (slug/title/markup/mail/
//! messages). Seeded once from `intake::FORM_SLUGS`; after that the DB is
//! source of truth so admin edits take effect without a rebuild.

use anyhow::Result;
use rusqlite::{params, OptionalExtension};
use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)] // full CF7 row kept; admin reads subsets
pub struct FormRow {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub lang: String,
    pub kind: String,
    pub markup: String,
    pub mail_to: String,
    pub mail_from: String,
    pub mail_subject: String,
    pub mail_body: String,
    pub mail2_active: bool,
    pub mail2_to: String,
    pub mail2_subject: String,
    pub mail2_body: String,
    pub messages: String,
    pub additional: String,
    pub created_at: String,
    pub updated_at: String,
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<FormRow> {
    Ok(FormRow {
        id: r.get(0)?,
        slug: r.get(1)?,
        title: r.get(2)?,
        lang: r.get(3)?,
        kind: r.get(4)?,
        markup: r.get(5)?,
        mail_to: r.get(6)?,
        mail_from: r.get(7)?,
        mail_subject: r.get(8)?,
        mail_body: r.get(9)?,
        mail2_active: r.get::<_, i64>(10)? != 0,
        mail2_to: r.get(11)?,
        mail2_subject: r.get(12)?,
        mail2_body: r.get(13)?,
        messages: r.get(14)?,
        additional: r.get(15)?,
        created_at: r.get(16)?,
        updated_at: r.get(17)?,
    })
}

const COLS: &str = "id, slug, title, lang, kind, markup, mail_to, mail_from, \
                    mail_subject, mail_body, mail2_active, mail2_to, \
                    mail2_subject, mail2_body, messages, additional, \
                    created_at, updated_at";

pub fn get_by_slug(db: &Path, slug: &str) -> Result<Option<FormRow>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(
        &format!("SELECT {COLS} FROM forms WHERE slug=?1"),
        [slug],
        row,
    )
    .optional()?)
}

#[allow(dead_code)] // id-lookup parity with get_by_slug
pub fn get(db: &Path, id: i64) -> Result<Option<FormRow>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(&format!("SELECT {COLS} FROM forms WHERE id=?1"), [id], row)
        .optional()?)
}

pub fn list(db: &Path) -> Result<Vec<FormRow>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(&format!("SELECT {COLS} FROM forms ORDER BY slug"))?;
    let rows: Vec<FormRow> = st.query_map([], row)?.flatten().collect();
    Ok(rows)
}

pub fn insert(
    db: &Path,
    slug: &str,
    title: &str,
    lang: &str,
    kind: &str,
    markup: &str,
    mail_to: &str,
    mail_from: &str,
    mail_subject: &str,
    mail_body: &str,
    messages: &str,
) -> Result<i64> {
    let c = crate::db::conn(db)?;
    c.execute(
        sql!("forms__insert"),
        params![
            slug, title, lang, kind, markup, mail_to, mail_from, mail_subject,
            mail_body, messages
        ],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn update(
    db: &Path,
    id: i64,
    title: &str,
    markup: &str,
    mail_to: &str,
    mail_from: &str,
    mail_subject: &str,
    mail_body: &str,
    mail2_active: bool,
    mail2_to: &str,
    mail2_subject: &str,
    mail2_body: &str,
    messages: &str,
    additional: &str,
) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(
        sql!("forms__update"),
        params![
            title, markup, mail_to, mail_from, mail_subject, mail_body,
            mail2_active as i64, mail2_to, mail2_subject, mail2_body,
            messages, additional, id
        ],
    )?;
    Ok(())
}

pub fn delete(db: &Path, id: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("forms__delete"), [id])?;
    Ok(())
}
