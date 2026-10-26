//! `submissions` table access — Flamingo "Inbound Messages" backing store.
//! Statuses: `new`/`pending` = Inbox, `approved` (post-moderate), `spam`,
//! `trash`; legacy `delete` reads as `trash`.
use anyhow::Result;
use rusqlite::{params, OptionalExtension};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone)]
pub struct Submission {
    pub id: i64,
    pub kind: String,
    pub name: String,
    pub page: String,
    pub tour: String,
    pub data_json: String,
    pub ip: String,
    pub form_slug: String,
    pub status: String,
    pub created_at: String,
}

/// View/filter params for the inbox list. `status` is the view key
/// ("inbox" | "spam" | "trash"), `sort` is whitelisted before reaching SQL.
#[derive(Clone)]
pub struct Filter {
    pub status: String,
    pub form_slug: String,
    /// "form"|"comment"|"review" (or "" = any). Site Reviews screen uses "review".
    pub kind: String,
    pub s: String,
    /// ISO `YYYY-MM-DD` bounds on `created_at` ("" = unbounded).
    pub from: String,
    pub to: String,
    pub limit: i64,
    pub offset: i64,
    pub sort: String,
    pub dir: String,
}

/// Per-view counts for `ul.subsubsub`. `trash` includes legacy `delete` rows.
pub fn counts(db: &Path) -> Result<HashMap<String, i64>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("submissions__count_by_status"))?;
    let mut out: HashMap<String, i64> = HashMap::new();
    for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?.flatten() {
        *out.entry(r.0).or_default() += r.1;
    }
    Ok(out)
}

/// Per-`form_slug` counts for the CF7-style forms list.
pub fn form_counts(db: &Path) -> Result<HashMap<String, i64>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("submissions__count_per_form"))?;
    let mut out: HashMap<String, i64> = HashMap::new();
    for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?.flatten() {
        *out.entry(r.0).or_default() += r.1;
    }
    Ok(out)
}

/// View key → SQL status fragment (constant strings only, never user input).
fn status_filter(view: &str) -> &'static str {
    match view {
        // Legacy 'delete' rows read as trash.
        "trash" => "status IN ('trash','delete')",
        "spam" => "status = 'spam'",
        // Flamingo Inbox = 'new' ('pending' tolerated for old dumps).
        _ => "status IN ('new','pending')",
    }
}

/// Sortable columns, whitelisted: anything not listed falls back to date desc.
fn order_frag(sort: &str, dir: &str) -> String {
    let d = if dir == "asc" { "ASC" } else { "DESC" };
    let col = match sort {
        "subject" => "name COLLATE NOCASE",
        "from" => "name COLLATE NOCASE",
        _ => "created_at",
    };
    format!("{col} {d}, id {d}")
}

fn map_row(r: &rusqlite::Row) -> rusqlite::Result<Submission> {
    Ok(Submission {
        id: r.get(0)?,
        kind: r.get(1)?,
        name: r.get(2)?,
        page: r.get(3)?,
        tour: r.get(4)?,
        data_json: r.get(5)?,
        ip: r.get(6)?,
        form_slug: r.get(7)?,
        status: r.get(8)?,
        created_at: r.get(9)?,
    })
}

/// Filtered page of submissions + total for the same filter.
pub fn list(db: &Path, f: &Filter) -> Result<(Vec<Submission>, i64)> {
    let c = crate::db::conn(db)?;
    let total: i64 = c.query_row(
        &sql!("submissions__count_filtered").replace("{status_filter}", status_filter(&f.status)),
        params![f.form_slug, f.s, f.kind, f.from, f.to],
        |r| r.get(0),
    )?;
    let sql = sql!("submissions__list_filtered")
        .replace("{status_filter}", status_filter(&f.status))
        .replace("{order}", &order_frag(&f.sort, &f.dir));
    let mut st = c.prepare(&sql)?;
    let rows = st
        .query_map(params![f.form_slug, f.s, f.kind, f.from, f.to, f.limit, f.offset], map_row)?
        .flatten()
        .collect();
    Ok((rows, total))
}

/// Excel/Numbers-safe CSV cell: always `"`-wrapped, inner `"` doubled,
/// CR/LF flattened to spaces so a row can never break the record line.
/// Formula-injection guard (A8): cells starting with `= + - @` or a
/// tab/CR get a leading `'` so Excel imports them as text.
fn csv_cell(s: &str) -> String {
    let guard = matches!(s.chars().next(), Some('=' | '+' | '-' | '@' | '\t' | '\r'));
    let body = s.replace('"', "\"\"").replace('\r', " ").replace('\n', " ");
    format!("\"{}{}\"", if guard { "'" } else { "" }, body)
}

/// String view of a `data_json` value: strings pass through, numbers
/// (e.g. `rating: 5`) stringify, everything else is empty.
fn jstr(map: &Map<String, Value>, key: &str) -> String {
    match map.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// Flamingo-style CSV export of the filtered set, capped at `cap` rows.
/// Columns:
/// `id,date,kind,form,subject,from_name,from_email,page,tour,rating,ip,status,fields_json`
/// — `subject`/`email`/`rating` are flattened out of `data_json`; `fields_json`
/// keeps the raw payload lossless. Overflow emits one truncation-note row.
pub fn export_csv(db: &Path, f: &Filter, cap: i64) -> Result<String> {
    let c = crate::db::conn(db)?;
    let total: i64 = c.query_row(
        &sql!("submissions__count_filtered").replace("{status_filter}", status_filter(&f.status)),
        params![f.form_slug, f.s, f.kind, f.from, f.to],
        |r| r.get(0),
    )?;
    let sql = sql!("submissions__export")
        .replace("{status_filter}", status_filter(&f.status))
        .replace("{order}", &order_frag(&f.sort, &f.dir));
    let mut st = c.prepare(&sql)?;
    let rows: Vec<Submission> = st
        .query_map(params![f.form_slug, f.s, f.kind, f.from, f.to, cap], map_row)?
        .flatten()
        .collect();

    let mut out = String::from(
        "\"id\",\"date\",\"kind\",\"form\",\"subject\",\"from_name\",\"from_email\",\"page\",\"tour\",\"rating\",\"ip\",\"status\",\"fields_json\"\r\n",
    );
    let mut emit = |cells: &[String]| {
        for (i, cell) in cells.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&csv_cell(cell));
        }
        out.push_str("\r\n");
    };
    for r in &rows {
        let map: Map<String, Value> = serde_json::from_str(&r.data_json).unwrap_or_default();
        let subject = jstr(&map, "subject");
        let subject = if subject.is_empty() { jstr(&map, "your-subject") } else { subject };
        let email = jstr(&map, "email");
        let email = if email.is_empty() { jstr(&map, "your-email") } else { email };
        emit(&[
            r.id.to_string(),
            r.created_at.clone(),
            r.kind.clone(),
            r.form_slug.clone(),
            if subject.is_empty() { "(no subject)".into() } else { subject },
            r.name.clone(),
            email,
            r.page.clone(),
            r.tour.clone(),
            jstr(&map, "rating"),
            r.ip.clone(),
            r.status.clone(),
            r.data_json.clone(),
        ]);
    }
    if total > cap {
        emit(&[
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            format!("Truncated: {} row(s) beyond the {} row cap were not exported", total - cap, cap),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ]);
    }
    Ok(out)
}

pub fn get(db: &Path, id: i64) -> Result<Option<Submission>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("submissions__get_one"), [id], map_row).optional()?)
}

/// Direct status write — used by callers that bypass `intake::moderate`.
#[allow(dead_code)]
pub fn set_status(db: &Path, id: i64, status: &str) -> Result<()> {
    anyhow::ensure!(
        matches!(status, "new" | "pending" | "approved" | "spam" | "trash"),
        "bad submission status"
    );
    let c = crate::db::conn(db)?;
    c.execute(sql!("submissions__set_status"), params![status, id])?;
    Ok(())
}

/// Permanent delete ("Delete Permanently" row/bulk action).
#[allow(dead_code)]
pub fn purge(db: &Path, id: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("submissions__delete_one"), [id])?;
    Ok(())
}
