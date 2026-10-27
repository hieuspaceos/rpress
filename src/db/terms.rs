use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
#[derive(Clone)]
pub struct TermRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub count: i64,
    /// 0 = top level. `terms.parent_id` added by Phase 8 (see schema ALTER).
    pub parent_id: i64,
}

pub fn list_terms(db: &Path, tax: &str) -> Result<Vec<TermRow>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(
        sql!("terms__select_t_id"),
    )?;
    let rows = st
        .query_map([tax], |r| {
            Ok(TermRow {
                id: r.get(0)?,
                slug: r.get(1)?,
                name: r.get(2)?,
                description: r.get(3)?,
                count: r.get(4)?,
                parent_id: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Stored `parent_id` (0 when none) — lets `save_term` preserve the parent
/// when the submitting form has no parent field.
fn old_parent(db: &Path, id: i64) -> Option<i64> {
    crate::db::conn(db)
        .ok()?
        .query_row(sql!("terms__select_parent_id"), [id], |r| r.get(0))
        .optional()
        .ok()
        .flatten()
}

pub fn save_term(db: &Path, tax: &str, id: Option<i64>, name: &str, slug: &str, desc: &str, parent: Option<i64>) -> Result<i64> {
    if name.trim().is_empty() || slug.trim().is_empty() {
        anyhow::bail!("name and slug required");
    }
    // Cycle guard: a term cannot be its own parent or a descendant of
    // itself — clamp those to root, mirroring WP `wp_insert_term` behavior.
    // `None` on update = the form sent no parent field (Quick Edit) → keep
    // the stored value.
    let parent = match (id, parent) {
        (Some(id), Some(p)) if p == id || descendants(db, id).unwrap_or_default().contains(&p) => 0,
        (Some(_), Some(p)) => p,
        (Some(id), None) => old_parent(db, id).unwrap_or(0),
        (None, p) => p.unwrap_or(0),
    };
    let c = crate::db::conn(db)?;
    if let Some(id) = id {
        let old_slug: Option<String> = c
            .query_row(sql!("terms__select_slug_from"), [id], |r| r.get(0))
            .optional()?;
        c.execute(
            sql!("terms__update_terms_set"),
            params![name.trim(), slug.trim(), desc.trim(), parent, id],
        )?;
        if let Some(old) = old_slug {
            if old != slug.trim() {
                c.execute(
                    sql!("terms__update_post_terms_set"),
                    params![slug.trim(), name.trim(), tax, old],
                )?;
            } else {
                c.execute(
                    sql!("terms__update_post_terms_set_5"),
                    params![name.trim(), tax, slug.trim()],
                )?;
            }
        }
        Ok(id)
    } else {
        c.execute(
            sql!("terms__insert_into_terms"),
            params![tax, slug.trim(), name.trim(), desc.trim(), parent],
        )?;
        Ok(c.last_insert_rowid())
    }
}

/// Self + every descendant id via recursive CTE — used to disable unsafe
/// Parent options on the term edit screen and to clamp submitted parents.
pub fn descendants(db: &Path, id: i64) -> Result<Vec<i64>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("terms__select_descendants"))?;
    let v = st.query_map([id], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(v)
}

/// Top-20 terms by usage — feeds the "Choose from the most used" cloud on
/// non-hierarchical taxonomy screens.
pub fn top_terms(db: &Path, tax: &str) -> Result<Vec<(String, i64)>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("terms__select_top"))?;
    let v = st
        .query_map([tax], |r| Ok((r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(v)
}

pub fn delete_term(db: &Path, id: i64) -> Result<()> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    // Resolve the term's join key BEFORE deleting: post_terms carries no
    // term_id, so the cascade deletes by (taxonomy, slug). A missing id is
    // still a clean no-op (matches the old row-only delete).
    let (tax, slug): (String, String) = tx
        .query_row(
            "SELECT taxonomy, slug FROM terms WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or_default();
    // WP reparents orphaned children to the deleted term's own parent.
    tx.execute(sql!("terms__reparent_children"), [id])?;
    tx.execute(sql!("terms__delete_from_terms"), [id])?;
    // H13: dead term slugs must not keep rendering/counting on archives.
    if !tax.is_empty() {
        tx.execute(
            "DELETE FROM post_terms WHERE taxonomy=?1 AND slug=?2",
            params![tax, slug],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Per-user Screen Options for one taxonomy screen, stored as JSON in
/// `settings.tax_screen_{user_id}_{tax}`: {"hide":[cols], "per_page":n}.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct TaxScreenOpts {
    #[serde(default)]
    pub hide: Vec<String>,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_per_page() -> i64 {
    20
}

pub fn screen_opts(db: &Path, user_id: i64, tax: &str) -> TaxScreenOpts {
    crate::db::conn(db)
        .ok()
        .and_then(|c| {
            c.query_row(
                sql!("terms__select_screen_setting"),
                [format!("tax_screen_{user_id}_{tax}")],
                |r| r.get::<_, String>(0),
            )
            .ok()
        })
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or(TaxScreenOpts { hide: Vec::new(), per_page: 20 })
}

pub fn save_screen_opts(db: &Path, user_id: i64, tax: &str, opts: &TaxScreenOpts) -> Result<()> {
    let c = crate::db::conn(db)?;
    c.execute(
        sql!("terms__upsert_screen_setting"),
        params![format!("tax_screen_{user_id}_{tax}"), serde_json::to_string(opts)?],
    )?;
    Ok(())
}

pub fn get_term(db: &Path, id: i64) -> Result<Option<TermRow>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(
        sql!("terms__select_id_slug"),
        [id],
        |r| Ok(TermRow { id: r.get(0)?, slug: r.get(1)?, name: r.get(2)?, description: r.get(3)?, count: r.get(4)?, parent_id: r.get(5)? }),
    ).optional()?)
}

/// ASCII slugifier shared by quick-edit, tax save, and tag creation.
pub fn term_slugs_for(db: &Path, post_id: i64, tax: &str) -> Result<Vec<String>> {
    let c = crate::db::conn(db)?;
    let mut s = c.prepare(sql!("terms__select_slug_from_9"))?;
    let v = s.query_map(params![post_id, tax], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(v)
}

/// Quick edit: update title/slug/status and replace category + tag terms only.
/// `cats` = comma list of category slugs; `tags` = comma list of tag names/slugs.
pub fn terms_for(db: &Path, post_id: i64) -> Result<Vec<(String, String, String)>> {
    let c = crate::db::conn(db)?;
    let mut s = c.prepare(sql!("terms__select_taxonomy_slug"))?;
    let v = s
        .query_map([post_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(v)
}

/// Assign `post_terms` on a caller-owned connection — `save_post_on` calls
/// this inside the post-save transaction so term writes join it instead of
/// racing it on a second connection (which would SQLITE_BUSY under a held
/// write lock).
pub fn set_terms_on(c: &Connection, post_id: i64, raw: &str) -> Result<()> {
    // lines "taxonomy:slug:name"
    c.execute(sql!("terms__delete_from_post_terms_11"), [post_id])?;
    for line in raw.lines() {
        let mut p = line.splitn(3, ':');
        let tax = p.next().unwrap_or("");
        let slug = p.next().unwrap_or("");
        let name = p.next().unwrap_or(slug);
        if tax.is_empty() || slug.is_empty() {
            continue;
        }
        c.execute(
            sql!("terms__insert_or_ignore"),
            params![post_id, tax, slug, name],
        )?;
    }
    Ok(())
}

pub fn term_choices(db: &Path, tax: &str, keep: &[String]) -> Result<Vec<(String, String, bool)>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(
        sql!("terms__select_slug_name"),
    )?;
    let mut out: Vec<(String, String, bool)> = st
        .query_map([tax], |r| Ok((r.get(0)?, r.get(1)?, false)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for k in keep {
        if !out.iter().any(|(s, _, _)| s == k) {
            let name = c
                .query_row(
                    sql!("terms__select_name_from"),
                    params![tax, k],
                    |r| r.get::<_, String>(0),
                )
                .unwrap_or_else(|_| k.clone());
            out.push((k.clone(), name, true));
        }
    }
    for row in &mut out {
        row.2 = keep.iter().any(|s| s == &row.0);
    }
    Ok(out)
}


/// Whether the slug is a real taxonomy (has rows in `terms` or `post_terms`).
pub fn taxonomy_exists(db: &Path, tax: &str) -> Result<bool> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(
        sql!("terms__select_exists_1"),
        [tax],
        |r| r.get::<_, i64>(0),
    ).map(|n| n == 1).unwrap_or(false))
}

/// Taxonomy of a term id (needed by tax_edit: terms table is keyed by it).
pub fn taxonomy_of(db: &Path, id: i64) -> Result<Option<String>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row("SELECT taxonomy FROM terms WHERE id=?1", [id], |r| r.get(0)).optional()?)
}

/// All taxonomy slugs present in terms/post_terms, menu order: known
/// tourmaster taxs first (option order), then extras alphabetically.
pub fn list_taxonomies(db: &Path) -> Result<Vec<String>> {
    let c = crate::db::conn(db)?;
    let mut s = c.prepare(
        "SELECT DISTINCT taxonomy FROM terms UNION SELECT DISTINCT taxonomy FROM post_terms",
    )?;
    let mut v: Vec<String> = s.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    if let Ok(raw) = c.query_row(
        "SELECT value FROM settings WHERE key='tour_custom_taxs'",
        [],
        |r| r.get::<_, String>(0),
    ) {
        if let Ok(extra) = serde_json::from_str::<std::collections::BTreeMap<String, String>>(&raw) {
            for k in extra.keys() {
                if !v.iter().any(|x| x == k) {
                    v.push(k.clone());
                }
            }
        }
    }
    const ORDER: &[&str] = &[
        "category", "post_tag", "duree", "region", "style", "profil",
        "duracion", "region-es", "estilo", "perfil", "Activites", "Pays",
    ];
    v.sort_by_key(|t| ORDER.iter().position(|x| *x == t).unwrap_or(ORDER.len()));
    Ok(v)
}
