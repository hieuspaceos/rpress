use anyhow::Result;
use super::misc::*;
use super::terms::*;
use super::{attach_term, reparent_orphans, slugify, term_name};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use std::path::Path;

#[derive(Default, Deserialize)]
pub struct PostForm {
    pub id: Option<i64>,
    pub kind: String,
    pub title: String,
    pub slug: String,
    pub lang: String,
    #[serde(default)]
    pub excerpt: String,
    #[serde(default)]
    pub seo_title: String,
    #[serde(default)]
    pub seo_desc: String,
    pub status: String,
    /// Editor "Published on" date (datetime-local "YYYY-MM-DDTHH:MM" or full
    /// "YYYY-MM-DD HH:MM[:SS]"). Empty = keep/now.
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub featured_media_id: String,
    #[serde(default)]
    pub body_json: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub es_post_id: String,

    #[serde(default)]
    pub csrf: String,
    #[serde(default)]
    pub terms: String,
    #[serde(default, deserialize_with = "de_cat")]
    pub cat: Vec<String>,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub rank_math: String,
    /// Plugin-owned form fields (e.g. `tourmaster-tour-option`): captured by
    /// serde flatten so the core struct never names plugin keys.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, String>,
}

/// `%var%` substitution for Rank Math SEO templates. Data uses only
/// `%title%`, `%sep%`, `%sitename%` (audit 2026-10-23); `%sep%` is "-" —
/// WP rendered titles as `Title - <blogname>` — site_name falls back to
/// `blogname` (imported verbatim), then an empty string.
pub fn resolve_seo(tpl: &str, title: &str, sitename: &str) -> String {
    tpl.replace("%title%", title)
        .replace("%sitename%", sitename)
        .replace("%sep%", "-")
}

fn site_name_conn(c: &Connection) -> String {
    c.query_row(
        "SELECT value FROM settings WHERE key='site_name' LIMIT 1",
        [],
        |r| r.get::<_, String>(0),
    )
    .ok()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| {
        c.query_row("SELECT value FROM settings WHERE key='blogname' LIMIT 1", [], |r| r.get::<_, String>(0))
            .unwrap_or_default()
    })
}

pub fn site_name(db: &Path) -> String {
    crate::db::conn(db)
        .ok()
        .map(|c| site_name_conn(&c))
        .unwrap_or_default()
}

/// PHP `serialize(array<string>)` for `rank_math_robots`.
/// `["index","nofollow"]` → `a:2:{i:0;s:5:"index";i:1;s:8:"nofollow";}`.
pub fn php_ser_string_array(items: &[String]) -> String {
    let mut out = format!("a:{}:{{", items.len());
    for (i, s) in items.iter().enumerate() {
        out += &format!("i:{};s:{}:\"{}\";", i, s.len(), s);
    }
    out.push('}');
    out
}

/// Read `rank_math_robots` PHP array → `["index","nofollow",…]`.
/// Unparseable/missing → empty vec (metabox shows no checks).
pub fn robots_decode(raw: &str) -> Vec<String> {
    match crate::php_serialize::unserialize(raw.as_bytes()) {
        Ok(serde_json::Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        Ok(serde_json::Value::Object(m)) => m
            .values()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

/// `cat` is ONE string: checked categories joined by '\u{1f}' (unit
/// separator), serialized by an inline script on submit — serde_urlencoded
/// can bind neither repeated keys nor `cat[]` names to a Vec.
fn de_cat<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    use serde::Deserialize;
    let s = String::deserialize(d).unwrap_or_default();
    Ok(s.split('\u{1f}')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect())
}

pub fn load_post(db: &Path, id: i64) -> Result<Option<PostForm>> {
    let c = crate::db::conn(db)?;
    let row = c
        .query_row(
            sql!("posts__select_id_kind"),
            [id],
            |r| {
                let feat: Option<i64> = r.get(9)?;
                Ok(PostForm {
                    id: Some(r.get(0)?),
                    kind: r.get(1)?,
                    title: r.get(2)?,
                    slug: r.get(3)?,
                    lang: r.get(4)?,
                    excerpt: r.get(5)?,
                    seo_title: r.get(6)?,
                    seo_desc: r.get(7)?,
                    status: r.get(8)?,
                    featured_media_id: feat.map(|n| n.to_string()).unwrap_or_default(),
                    body_json: r.get(10)?,
                    content: r.get(11)?,
                    ..Default::default()
                })
            },
        )
        .optional()?;
    let Some(mut p) = row else { return Ok(None) };
    if let Ok(Some(d)) = c
        .query_row(sql!("posts__select_coalesce_published_at"), [id], |r| r.get::<_, String>(0))
        .optional()
    {
        p.published_at = d;
    }
    if let Ok(es) = c.query_row(
        sql!("posts__select_es_post_id_from"),
        [id],
        |r| r.get::<_, Option<i64>>(0),
    ) {
        p.es_post_id = es.map(|n| n.to_string()).unwrap_or_default();
    }
    if let Ok(ts) = terms_for(db, id) {
        p.terms = ts
            .iter()
            .map(|(a, b, c)| format!("{a}:{b}:{c}"))
            .collect::<Vec<_>>()
            .join("\n");
    }
    if let Some(e) = crate::admin::plugins::editor_for(&p.kind) {
        (e.load)(&c, id, &mut p)?;
    }
    // body_json stays builder/layout data — the editor's `#content` is the
    // WP post_content stored in body_html (loaded in the SELECT above),
    // never derived from builder JSON.
    Ok(Some(p))
}

pub struct AdminListRow {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub lang: String,
    pub status: String,
    pub categories: String,
    pub fr_id: Option<i64>,
    pub es_id: Option<i64>,
    pub comments: i64,
    pub duree: String,
    pub region: String,
    pub style: String,
    pub profil: String,
    pub views: i64,
}

#[derive(Default)]
pub struct ListCounts {
    pub all: i64,
    pub publish: i64,
    pub pending: i64,
    pub future: i64,
    pub draft: i64,
    pub trash: i64,
    pub fr: i64,
    pub es: i64,
}

pub fn list_admin(
    db: &Path,
    kind: &str,
    status: &str,
    lang: &str,
    tax: &str,
    cat: &str,
    term_id: i64,
    search: &str,
    page: i64,
    per_page: i64,
) -> Result<(Vec<AdminListRow>, i64, ListCounts)> {

    let c = crate::db::conn(db)?;
    let counts_sql = c.query_row(
        sql!("posts__select_coalesce_sum"),
        [kind],
        |r| {
            Ok(ListCounts {
                all: r.get(0)?,
                publish: r.get(1)?,
                pending: 0,
                future: 0,
                draft: r.get(2)?,
                trash: r.get(3)?,
                fr: r.get(4)?,
                es: r.get(5)?,
            })
        },
    )?;
    // Pending/future counts live outside the .sql file (that dir is owned by
    // the render fix-up) — scalar COUNTs inline like `site_name_conn` above.
    let pending: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM posts WHERE kind=?1 AND status='pending'",
            [kind],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let future: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM posts WHERE kind=?1 AND status='future'",
            [kind],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let counts = ListCounts { pending, future, ..counts_sql };
    // WHERE built once, shared by a slim COUNT and the full row SELECT —
    // counting over the row SELECT would evaluate 8 correlated subqueries
    // (comments, translations, 4 taxonomy lookups, view_count) per matched
    // row just to get a number.
    let mut wh = String::from(" WHERE p.kind=?1");
    let mut sql = String::from(
        "SELECT p.id, COALESCE(p.slug,''), COALESCE(p.title,''), COALESCE(p.lang,''), COALESCE(p.status,''),
            COALESCE((SELECT group_concat(name, ', ') FROM post_terms t WHERE t.post_id=p.id AND t.taxonomy='category'),''),
            (SELECT fr_post_id FROM post_translations tr WHERE tr.fr_post_id=p.id OR tr.es_post_id=p.id LIMIT 1),
            (SELECT es_post_id FROM post_translations tr WHERE tr.fr_post_id=p.id OR tr.es_post_id=p.id LIMIT 1),
            (SELECT COUNT(*) FROM comments c WHERE c.post_id=p.id),
            COALESCE((SELECT name FROM post_terms t WHERE t.post_id=p.id AND t.taxonomy=CASE WHEN p.lang='es' THEN 'duracion' ELSE 'duree' END LIMIT 1),''),
            COALESCE((SELECT name FROM post_terms t WHERE t.post_id=p.id AND t.taxonomy=CASE WHEN p.lang='es' THEN 'region-es' ELSE 'region' END LIMIT 1),''),
            COALESCE((SELECT name FROM post_terms t WHERE t.post_id=p.id AND t.taxonomy=CASE WHEN p.lang='es' THEN 'estilo' ELSE 'style' END LIMIT 1),''),
            COALESCE((SELECT name FROM post_terms t WHERE t.post_id=p.id AND t.taxonomy=CASE WHEN p.lang='es' THEN 'perfil' ELSE 'profil' END LIMIT 1),''),
            COALESCE((SELECT view_count FROM tours WHERE post_id=p.id),0)
         FROM posts p",
    );
    let mut args: Vec<rusqlite::types::Value> =
        vec![rusqlite::types::Value::Text(kind.to_string())];
    match status {
        "publish" | "pending" | "future" | "draft" | "trash" => {
            wh.push_str(" AND p.status=?");
            args.push(rusqlite::types::Value::Text(status.to_string()));
        }
        _ => wh.push_str(" AND p.status!='trash'"),
    }
    if lang == "fr" || lang == "es" {
        wh.push_str(" AND p.lang=?");
        args.push(rusqlite::types::Value::Text(lang.to_string()));
    }
    if term_id > 0 {
        // /admin/tax count links carry a terms.id — join through terms so the
        // filter binds the term's real taxonomy+slug. `tax` still applies
        // when the caller passes both.
        wh.push_str(" AND EXISTS (SELECT 1 FROM post_terms t JOIN terms tm ON tm.taxonomy=t.taxonomy AND tm.slug=t.slug WHERE t.post_id=p.id AND tm.id=?)");
        args.push(rusqlite::types::Value::Integer(term_id));
        if !tax.is_empty() {
            wh.push_str(" AND tm.taxonomy=?");
            args.push(rusqlite::types::Value::Text(tax.to_string()));
        }
        wh.push(')');
    } else if !tax.is_empty() {
        wh.push_str(" AND EXISTS (SELECT 1 FROM post_terms t WHERE t.post_id=p.id AND t.taxonomy=?)");
        args.push(rusqlite::types::Value::Text(tax.to_string()));
    }
    if !cat.is_empty() {
        wh.push_str(" AND EXISTS (SELECT 1 FROM post_terms t WHERE t.post_id=p.id AND t.taxonomy='category' AND t.slug=?)");
        args.push(rusqlite::types::Value::Text(cat.to_string()));
    }
    if !search.is_empty() {
        wh.push_str(" AND (p.title LIKE ? OR p.slug LIKE ?)");
        let pat = format!("%{search}%");
        args.push(rusqlite::types::Value::Text(pat.clone()));
        args.push(rusqlite::types::Value::Text(pat));
    }

    let total: i64 = {
        let count_sql = format!("SELECT COUNT(*) FROM posts p{wh}");
        let mut st = c.prepare(&count_sql)?;
        st.query_row(rusqlite::params_from_iter(args.iter()), |r| r.get(0))?
    };
    sql.push_str(&wh);
    sql.push_str(" ORDER BY p.published_at DESC LIMIT ? OFFSET ?");
    let page = page.max(1);
    let offset = (page - 1) * per_page;
    args.push(rusqlite::types::Value::Integer(per_page));
    args.push(rusqlite::types::Value::Integer(offset));

    let mut st = c.prepare(&sql)?;
    let rows = st
        .query_map(rusqlite::params_from_iter(args.iter()), |r| {
            Ok(AdminListRow {
                id: r.get(0)?,
                slug: r.get(1)?,
                title: r.get(2)?,
                lang: r.get(3)?,
                status: r.get(4)?,
                categories: r.get(5)?,
                fr_id: r.get(6)?,
                es_id: r.get(7)?,
                comments: r.get(8)?,
                duree: r.get(9)?,
                region: r.get(10)?,
                style: r.get(11)?,
                profil: r.get(12)?,
                views: r.get(13)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((rows, total, counts))
}

/// Quick edit: update title/slug/status and replace category + tag terms only.
/// `cats` = comma list of category slugs; `tags` = comma list of tag names/slugs.
pub fn quick_update(
    db: &Path,
    id: i64,
    title: &str,
    slug: &str,
    status: &str,
    lang: &str,
    cats: &[String],
    tags: &[String],
) -> Result<()> {
    let c = crate::db::conn(db)?;
    let tx = c.unchecked_transaction()?;
    // Revision snapshot before the update — WP takes a revision on every
    // quick-edit too; save_post already snapshots via the same SQL.
    tx.execute(sql!("posts__insert_into_revisions"), [id])?;
    if lang.is_empty() {
        tx.execute(
            sql!("posts__update_posts_set"),
            params![title.trim(), slug.trim(), status, id],
        )?;
    } else {
        tx.execute(
            sql!("posts__update_posts_set_6"),
            params![title.trim(), slug.trim(), status, lang, id],
        )?;
    }
    tx.execute(
        sql!("posts__delete_from_post_terms"),
        [id],
    )?;
    for s in cats {
        let s = s.trim();
        if s.is_empty() { continue; }
        let name = term_name(&tx, "category", s)?;
        attach_term(&tx, id, "category", s, &name)?;
    }
    for t in tags {
        let t = t.trim();
        if t.is_empty() { continue; }
        let slug = slugify(t);
        if slug.is_empty() { continue; }
        attach_term(&tx, id, "post_tag", &slug, t)?;
    }
    tx.commit()?;
    Ok(())
}

/// Bulk edit: status change (when non-empty) + additive categories/tags.
pub fn bulk_apply(db: &Path, id: i64, status: &str, cats: &[String], tags: &[String]) -> Result<()> {
    let c = crate::db::conn(db)?;
    let tx = c.unchecked_transaction()?;
    // Revision snapshot — bulk edit is a save like any other (see quick_update).
    tx.execute(sql!("posts__insert_into_revisions"), [id])?;
    if matches!(status, "publish" | "future" | "draft" | "pending" | "trash") {
        tx.execute(
            sql!("posts__update_posts_set_8"),
            params![status, id],
        )?;
    }
    for s in cats {
        let s = s.trim();
        if s.is_empty() { continue; }
        let name = term_name(&tx, "category", s)?;
        attach_term(&tx, id, "category", s, &name)?;
    }
    for t in tags {
        let t = t.trim();
        if t.is_empty() { continue; }
        let slug = slugify(t);
        if slug.is_empty() { continue; }
        attach_term(&tx, id, "post_tag", &slug, t)?;
    }
    tx.commit()?;
    Ok(())
}

/// Duplicates a post (row + terms + meta + tour row + translation links).
/// `as_draft` maps to WP's "New Draft" action; plain clone keeps status.
pub fn clone_post(db: &Path, id: i64, as_draft: bool) -> Result<i64> {
    let c = crate::db::conn(db)?;
    let tx = c.unchecked_transaction()?;
    let (_slug, status): (String, String) = tx.query_row(
        sql!("posts__select_slug_status"), [id], |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    tx.execute(
        sql!("posts__insert_into_posts"),
        rusqlite::params![
            if as_draft { "draft" } else { status.as_str() },
            id,
        ],
    )?;
    let new_id = tx.last_insert_rowid();
    tx.execute(sql!("posts__insert_into_post_terms"), rusqlite::params![new_id, id])
        .map_err(|e| anyhow::anyhow!("clone post_terms {new_id}<={id}: {e}"))?;
    tx.execute(sql!("posts__insert_into_post_meta"), rusqlite::params![new_id, id])
        .map_err(|e| anyhow::anyhow!("clone post_meta {new_id}<={id}: {e}"))?;
    let kind: String = tx
        .query_row(sql!("verify__select_kind_from"), [new_id], |r| r.get(0))?;
    if let Some(a) = crate::admin::plugins::adapter_for_kind(&kind) {
        a.clone_post(&tx, new_id, id)?;
    }
    tx.commit()?;
    Ok(new_id)
}

/// post_tag names for the Quick-Edit prefill.
pub fn tag_names_for(db: &Path, id: i64) -> Result<Vec<String>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("posts__select_name_from_2"))?;
    let rows = st.query_map([id], |r| r.get(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// post.lang for Quick-Edit prefill.
pub fn lang_of(db: &Path, id: i64) -> Result<String> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("posts__select_coalesce_lang"), [id], |r| r.get(0))
        .unwrap_or_default())
}


/// media.path for the featured-image preview.
pub fn media_path(db: &Path, id: i64) -> Result<Option<String>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("posts__select_path_from"), [id], |r| r.get(0)).ok())
}

/// published_at fallback to modified_at, for the editor meta box.
pub fn published_at(db: &Path, id: i64) -> Result<Option<String>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("posts__select_coalesce_published_at"), [id], |r| r.get(0)).ok())
}

pub fn set_post_status(db: &Path, id: i64, status: &str) -> Result<()> {
    if !matches!(status, "publish" | "future" | "draft" | "pending" | "trash") {
        anyhow::bail!("bad status");
    }
    let c = crate::db::conn(db)?;
    c.execute(sql!("posts__update_posts_set_14"), params![status, id])?;
    Ok(())
}

/// Hard-delete the posts matched by `ids_sql` (a SQL fragment producing an
/// id set, e.g. `"SELECT id FROM posts WHERE status='trash' AND kind=?1"`),
/// plus every dependent row — schema has no FK cascades (WP
/// `wp_delete_post(force=true)` semantics). Caller wraps in a transaction.
fn delete_posts_in(tx: &Connection, ids_sql: &str, p: impl rusqlite::Params) -> Result<u64> {
    let ids: Vec<i64> = tx
        .prepare(&format!("SELECT id FROM ({ids_sql})"))?
        .query_map(p, |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if ids.is_empty() {
        return Ok(0);
    }
    let ph = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let run = |table: &str, col: &str| -> rusqlite::Result<usize> {
        tx.execute(
            &format!("DELETE FROM {table} WHERE {col} IN ({ph})"),
            rusqlite::params_from_iter(ids.iter()),
        )
    };
    run("comments", "post_id")?;
    run("post_terms", "post_id")?;
    run("post_meta", "post_id")?;
    run("tours", "post_id")?;
    run("revisions", "post_id")?;
    run("review_assignments", "post_id")?;
    run("post_translation_members", "post_id")?;
    tx.execute(
        &format!("DELETE FROM post_translations WHERE fr_post_id IN ({ph}) OR es_post_id IN ({ph})"),
        rusqlite::params_from_iter(ids.iter().chain(ids.iter())),
    )?;
    // Groups left with <2 members after the member sweep are dead pairs —
    // a lone member can't translate to anything, and its row would re-link
    // to a future post reusing the rowid.
    tx.execute(
        "DELETE FROM post_translation_members WHERE group_id IN (\
            SELECT group_id FROM post_translation_members GROUP BY group_id HAVING COUNT(*) < 2)",
        [],
    )?;
    let n = run("posts", "id")?;
    Ok(n as u64)
}
/// Post ids matched by an `ids_sql` fragment (same contract as
/// `delete_posts_in`). Callers that hard-delete use this first so the
/// on-disk files can still be resolved and removed — `generate::regen_post`
/// needs the row to exist.
pub fn ids_matching(db: &Path, ids_sql: &str, p: impl rusqlite::Params) -> Result<Vec<i64>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(ids_sql)?;
    let ids = st
        .query_map(p, |r| r.get::<_, i64>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(ids)
}

/// "Empty Trash" on a posts list view: hard-deletes every trashed row of
/// `kind` (and its dependents) in one transaction.
pub fn empty_trash(db: &Path, kind: &str) -> Result<u64> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    let n = delete_posts_in(
        &tx,
        "SELECT id FROM posts WHERE status='trash' AND kind=?1",
        params![kind],
    )?;
    tx.commit()?;
    Ok(n)
}

/// WP `_wp_scheduled_delete`: hard-delete posts/comments trashed > 30 days.
/// Runs inside a generate call so it fires even when nobody logs in.
/// Trash age is `modified_at` (posts) / `created_at` (comments) — both are
/// `datetime('now')` stamps, so a lexicographic `<` compares correctly.
pub fn purge_old_trash(db: &Path) -> Result<u64> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    // Re-parent children of aged trash comments before the delete, matching
    // `comments::empty_status`. Cyclic parent chains bail to force-NULL in
    // the helper after 1000 passes — the loop alone spun forever.
    reparent_orphans(
        &tx,
        "status='trash' AND created_at < datetime('now','-30 days')",
        [],
    )?;
    let cn = tx.execute(
        "DELETE FROM comments WHERE status='trash' AND created_at < datetime('now','-30 days')",
        [],
    )?;
    let pn = delete_posts_in(
        &tx,
        "SELECT id FROM posts WHERE status='trash' AND modified_at < datetime('now','-30 days')",
        [],
    )?;
    tx.commit()?;
    Ok(cn as u64 + pn)
}

/// Persist Rank Math metas posted by the metabox. `f.rank_math` is a JSON
/// object `{"rank_math_focus_keyword":"…", "rank_math_robots":["index",…],
/// "rank_math_schema_type":"Article", "primary":{"duree":"slug",…}}` built
/// by the editor's inline script (serde_urlencoded can't bind these names).
/// Robots serializes to the PHP array WP uses; `primary` keys are taxonomy
/// slugs mapped to their `rank_math_primary_{TaxLabel}` key.
/// `tax_map`: `(taxonomy_slug, label)` pairs for the post's taxonomies.
pub fn save_rankmath(c: &Connection, id: i64, f: &PostForm, tax_map: &[(String, String)]) -> Result<()> {
    let raw = f.rank_math.trim();
    if raw.is_empty() {
        return Ok(());
    }
    // Malformed JSON must not wipe existing metas (audit: reviewer) — bail.
    let vals: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| anyhow::anyhow!("rank_math JSON: {e}"))?;
    let obj = vals.as_object();
    let set = |key: &str, value: &str| -> Result<()> {
        c.execute(sql!("posts__delete_post_meta"), params![id, key])?;
        c.execute(sql!("posts__insert_post_meta"), params![id, key, value])?;
        Ok(())
    };
    let s = |k: &str| obj.and_then(|o| o.get(k)).and_then(|v| v.as_str()).unwrap_or("");

    set("rank_math_focus_keyword", s("rank_math_focus_keyword"))?;
    set("rank_math_title", s("rank_math_title"))?;
    set("rank_math_description", s("rank_math_description"))?;
    set("rank_math_canonical_url", s("rank_math_canonical_url"))?;

    // robots: JSON array → PHP serialized array (empty → empty array).
    let robots: Vec<String> = obj
        .and_then(|o| o.get("rank_math_robots"))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    set("rank_math_robots", &php_ser_string_array(&robots))?;

    // schema: rich_snippet is 'on'/'off' in real data. rank_math_schema_{Type}
    // rows hold a PHP-serialized config object — preserve any existing blob;
    // only a *switch* deletes the other types, and a selected type with no
    // row gets a minimal default object.
    set("rank_math_rich_snippet", if s("rank_math_rich_snippet") == "on" { "on" } else { "off" })?;
    let st = s("rank_math_schema_type");
    for t in ["Article", "BlogPosting", "VideoObject"] {
        if t != st {
            c.execute(sql!("posts__delete_post_meta"), params![id, format!("rank_math_schema_{t}")])?;
        }
    }
    if !st.is_empty() {
        let key = format!("rank_math_schema_{st}");
        let exists: Option<String> = c
            .query_row(
                "SELECT meta_value FROM post_meta WHERE post_id=?1 AND meta_key=?2 LIMIT 1",
                params![id, key],
                |r| r.get(0),
            )
            .optional()?;
        if exists.is_none() {
            let default = "a:4:{s:8:\"headline\";s:11:\"%seo_title%\";s:11:\"description\";s:17:\"%seo_description%\";s:6:\"author\";a:2:{s:5:\"@type\";s:6:\"Person\";s:4:\"name\";s:0:\"\";}s:13:\"datePublished\";s:20:\"%date(Y-m-dTH:i:sP)%\";}";
            set(&key, default)?;
        }
    }

    // social
    for k in [
        "rank_math_facebook_title",
        "rank_math_facebook_description",
        "rank_math_facebook_image",
        "rank_math_twitter_title",
        "rank_math_twitter_description",
        "rank_math_twitter_image",
        "rank_math_twitter_card_type",
    ] {
        set(k, s(k))?;
    }

    set("rank_math_pillar_content", if s("rank_math_pillar_content") == "on" { "on" } else { "" })?;

    // primary term per taxonomy: meta key uses the *label* spelling
    // (`rank_math_primary_Durée`, `rank_math_primary_category`…).
    if let Some(pm) = obj.and_then(|o| o.get("primary")).and_then(|v| v.as_object()) {
        for (tax, label) in tax_map {
            let key = format!("rank_math_primary_{label}");
            let v = pm.get(tax).and_then(|x| x.as_str()).unwrap_or("");
            set(&key, v)?;
        }
    }
    // score: full analysis on the *submitted* values (overrides, since the
    // metas we just wrote describe the same state). Site URL matters for
    // internal/external link classification.
    let site_url = c
        .query_row("SELECT value FROM settings WHERE key='site_url' LIMIT 1", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap_or_default();
    let body_text = if f.body_json.trim().is_empty() {
        String::new()
    } else {
        gdlr_text(&f.body_json)
    };
    let body_for_analysis = if body_text.trim().is_empty() {
        format!("{} {}", f.content, f.excerpt)
    } else {
        body_text
    };
    let ana = seo_analysis(
        c,
        id,
        &site_url,
        Some(s("rank_math_focus_keyword")),
        Some(s("rank_math_title")),
        Some(s("rank_math_description")),
        Some(&body_for_analysis),
    );
    set("rank_math_seo_score", &ana.score.to_string())?;
    Ok(())
}

/* ================= Rank Math analysis ================= */

/// One analysis check line: label + pass/fail (readability groups may also
/// Label + pass/fail; pass=None renders a neutral bullet.
#[derive(Debug, Clone)]
pub struct SeoCheck {
    pub label: String,
    pub pass: Option<bool>,
}

#[derive(Debug, Default)]
pub struct SeoAnalysis {
    pub score: i64,
    pub keyword: String,
    pub basic: Vec<SeoCheck>,
    pub additional: Vec<SeoCheck>,
    pub title_readability: Vec<SeoCheck>,
    pub content_readability: Vec<SeoCheck>,
    pub links_internal: i64,
    pub links_external: i64,
    pub links_inbound: i64,
    pub words: i64,
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// All text-like values inside a gdlr `body_json` tree (element `content`,
/// `text`, `description`, `caption`, `title` fields) concatenated.
fn gdlr_text(body_json: &str) -> String {
    let v: serde_json::Value = match serde_json::from_str(body_json) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let mut out = String::new();
    fn walk(v: &serde_json::Value, out: &mut String) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, x) in m {
                    match (k.as_str(), x) {
                        ("content" | "text" | "description" | "caption", serde_json::Value::String(s)) => {
                            out.push_str(s);
                            out.push(' ');
                        }
                        _ => walk(x, out),
                    }
                }
            }
            serde_json::Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    walk(&v, &mut out);
    out
}

fn count_links(html: &str) -> (Vec<String>, i64) {
    // crude `<a href>` scan — body text is already tag-soup, no parser needed.
    let mut urls = Vec::new();
    let lc = html.to_lowercase();
    let mut rest = lc.as_str();
    while let Some(i) = rest.find("href=") {
        let tail = &rest[i + 5..];
        let q = tail.chars().next().unwrap_or(' ');
        if q == '"' || q == '\'' {
            if let Some(j) = tail[1..].find(q) {
                urls.push(tail[1..1 + j].to_string());
            }
        }
        rest = &tail[1..];
    }
    let imgs = lc.matches("<img").count() as i64;
    (urls, imgs)
}

/// Concatenate string literals of a PHP-serialized blob (`s:NN:"..."`),
/// keeping only values that look like real content (letters, >25 chars).
fn php_ser_text(blob: &str) -> String {
    let mut out = String::new();
    let b = blob.as_bytes();
    let mut i = 0;
    while i + 2 < b.len() {
        if b[i] == b's' && b[i + 1] == b':' {
            let mut j = i + 2;
            let mut n: usize = 0;
            while j < b.len() && b[j].is_ascii_digit() {
                n = n * 10 + (b[j] - b'0') as usize;
                j += 1;
            }
            if j + 2 < b.len() && b[j] == b':' && b[j + 1] == b'"' {
                let start = j + 2;
                let end = start + n;
                if end <= b.len() {
                    if let Ok(s) = std::str::from_utf8(&b[start..end]) {
                        if s.len() > 25
                            && s.chars().any(|c| c.is_ascii_alphabetic())
                            && !s.starts_with("a:")
                        {
                            out.push_str(s);
                            out.push(' ');
                        }
                    }
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// Full Rank Math-style analysis for one post. Basic + Additional sum to
/// 100 pts; Title/Content readability are informational groups.
/// `kw_override` lets the save path score *submitted* values before they
/// are persisted (list/read paths pass `None` → stored metas).
pub fn seo_analysis(
    c: &Connection,
    id: i64,
    site_url: &str,
    kw_override: Option<&str>,
    title_override: Option<&str>,
    desc_override: Option<&str>,
    body_override: Option<&str>,
) -> SeoAnalysis {
    let mut a = SeoAnalysis::default();
    let meta = |k: &str| -> String {
        c.query_row(
            "SELECT meta_value FROM post_meta WHERE post_id=?1 AND meta_key=?2 LIMIT 1",
            params![id, k],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_default()
    };
    let (title, slug, body_json, excerpt): (String, String, String, String) = c
        .query_row(
            "SELECT title, slug, COALESCE(body_json,''), COALESCE(excerpt,'') FROM posts WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap_or_default();

    let sitename = site_name_conn(c);
    let kw_raw = kw_override.map(str::to_string).unwrap_or_else(|| meta("rank_math_focus_keyword"));
    a.keyword = kw_raw.split(',').next().unwrap_or("").trim().to_string();
    let kw = a.keyword.to_lowercase();

    let tpl = title_override
        .map(str::to_string)
        .unwrap_or_else(|| meta("rank_math_title"));
    let seo_t = resolve_seo(
        if tpl.is_empty() { "%title% %sep% %sitename%" } else { &tpl },
        &title,
        &sitename,
    );
    let seo_d = desc_override.map(str::to_string).unwrap_or_else(|| meta("rank_math_description"));

    let pb = meta("gdlr-core-page-builder");
    let raw_html = body_override
        .map(str::to_string)
        .unwrap_or_else(|| format!("{} {}", gdlr_text(&body_json), php_ser_text(&pb)));
    let plain = strip_tags(&format!("{raw_html} {excerpt}")).to_lowercase();
    let words: Vec<&str> = plain.split_whitespace().collect();
    a.words = words.len() as i64;

    let (urls, imgs) = count_links(&raw_html);
    let host = site_url.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');
    for u in &urls {
        if u.starts_with("http") {
            if u.contains(host) { a.links_internal += 1 } else { a.links_external += 1 }
        } else if u.starts_with('/') {
            a.links_internal += 1;
        }
    }
    if !slug.is_empty() {
        a.links_inbound = c
            .query_row(
                "SELECT COUNT(DISTINCT p.id) FROM posts p LEFT JOIN post_meta pm
                   ON pm.post_id=p.id AND pm.meta_key='gdlr-core-page-builder'
                 WHERE p.id != ?1 AND p.status='publish'
                   AND (p.body_json LIKE '%'||?2||'%' OR p.body_json LIKE '%'||?3||'%'
                        OR pm.meta_value LIKE '%'||?2||'%' OR pm.meta_value LIKE '%'||?3||'%')",
                params![id, format!("href=\"/{slug}"), format!("{host}/{slug}")],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0);
    }

    let kw_slug = kw.replace(' ', "-");
    let mut score = 0i64;
    let mut basic = Vec::new();
    let mut push = |v: &mut Vec<SeoCheck>, pts: i64, pass: bool, label: String| {
        if pass {
            score += pts;
        }
        v.push(SeoCheck { label, pass: Some(pass) });
    };

    // ---- Basic SEO (60)
    push(&mut basic, 10, !kw.is_empty() && seo_t.to_lowercase().contains(&kw),
        "Hurray! You're using Focus Keyword in the SEO Title.".into());
    push(&mut basic, 10, !kw.is_empty() && seo_d.to_lowercase().contains(&kw),
        "Focus Keyword used inside SEO Meta Description.".into());
    push(&mut basic, 10, !kw.is_empty() && slug.to_lowercase().contains(&kw_slug),
        "Focus Keyword used in the URL.".into());
    let head10 = plain.chars().take(plain.len() / 10 + 1).collect::<String>();
    push(&mut basic, 10, !kw.is_empty() && head10.contains(&kw),
        "Focus Keyword appears in the first 10% of the content.".into());
    push(&mut basic, 10, !kw.is_empty() && plain.contains(&kw),
        "Focus Keyword found in the content.".into());
    push(&mut basic, 10, a.words >= 600,
        format!("Content is {} words long. Good job!", a.words));

    // ---- Additional (40)
    let mut additional = Vec::new();
    let lc = raw_html.to_lowercase();
    let mut headings = String::new();
    for tag in ["<h2", "<h3", "<h4"] {
        let mut rest = lc.as_str();
        while let Some(i) = rest.find(tag) {
            let tail = &rest[i..];
            if let Some(j) = tail.find('>') {
                if let Some(k) = tail[j..].find(&format!("</{}", &tag[1..])) {
                    headings.push_str(&tail[j + 1..j + 1 + k]);
                    headings.push(' ');
                }
            }
            rest = &tail[1..];
        }
    }
    let headings = strip_tags(&headings).to_lowercase();
    push(&mut additional, 8, !kw.is_empty() && headings.contains(&kw),
        "Focus Keyword found in the subheading(s).".into());
    let density = if a.words > 0 && !kw.is_empty() {
        100.0 * plain.matches(&kw).count() as f64 * kw.split_whitespace().count() as f64 / a.words as f64
    } else { 0.0 };
    push(&mut additional, 8, (0.5..=2.5).contains(&density),
        format!("Keyword Density is {density:.2}% which is {}optimum.", if (0.5..=2.5).contains(&density) { "" } else { "not " }));
    let alt_hit = !kw.is_empty() && lc.contains("alt=") && lc.contains(&kw);
    push(&mut additional, 8, imgs > 0 && alt_hit,
        format!("{imgs} image(s) found; keyword in ALT text: {}.", if alt_hit { "yes" } else { "no" }));
    push(&mut additional, 8, a.links_internal > 0 || a.links_inbound > 0,
        format!("Internal links: {} outgoing, {} inbound.", a.links_internal, a.links_inbound));
    push(&mut additional, 8, a.links_external > 0,
        format!("{} external link(s) found.", a.links_external));
    a.basic = basic;
    a.additional = additional;
    a.score = score;

    // ---- Title readability (informational)
    let tl = seo_t.chars().count();
    a.title_readability = vec![
        SeoCheck { label: format!("SEO title is {tl} characters long."), pass: Some((30..=65).contains(&tl)) },
        SeoCheck { label: "Title starts with the Focus Keyword.".into(), pass: Some(!kw.is_empty() && seo_t.to_lowercase().trim_start().starts_with(&kw)) },
        SeoCheck { label: "Title contains a number.".into(), pass: Some(seo_t.chars().any(|c| c.is_ascii_digit())) },
        SeoCheck { label: "Title contains a power word.".into(), pass: Some(POWER_WORDS.iter().any(|w| seo_t.to_lowercase().contains(w))) },
    ];

    // ---- Content readability (informational)
    let paras = raw_html.split("</p>").filter(|p| !strip_tags(p).trim().is_empty()).count();
    let has_media = imgs > 0 || lc.contains("<iframe") || lc.contains("youtube") || lc.contains("vimeo");
    a.content_readability = vec![
        SeoCheck { label: "Content contains media (images or video).".into(), pass: Some(has_media) },
        SeoCheck { label: format!("{paras} paragraph(s)."), pass: Some(paras >= 3) },
        SeoCheck { label: "Content uses short paragraphs (≤120 words avg).".into(),
            pass: Some(paras == 0 || a.words / paras.max(1) as i64 <= 120) },
        SeoCheck { label: "Content has a Table of Contents (heading anchors).".into(),
            pass: Some(has_toc(&lc)) },
    ];
    a
}

fn has_toc(lc: &str) -> bool {
    lc.contains("ez-toc")
        || lc.contains("id=\"toc\"")
        || lc.matches("</h2>").count() + lc.matches("</h3>").count() >= 4
}

const POWER_WORDS: &[&str] = &[
    "meilleur", "best", "guide", "ultime", "ultimate", "essentiel", "essential",
    "top", "secret", "gratuit", "free", "complet", "complete", "incroyable",
    "parfait", "perfect", "authentique", "incontournable", "conseils", "tips",
];


/// Save a post inside a transaction (A15): the posts row, plugin saves,
/// translations, terms, and Rank Math metas commit or roll back together —
/// a meta failure can never leave a half-saved post.
pub fn save_post(db: &Path, f: &PostForm) -> Result<i64> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    let id = save_post_on(db, &tx, f)?;
    // Rank Math meta save joins the same transaction (reads the just-written
    // post_terms for its taxonomy→meta-key map).
    crate::admin::rankmath::save_conn(&tx, id, f)
        .map_err(|e| anyhow::anyhow!("rankmath: {e}"))?;
    tx.commit()?;
    Ok(id)
}

/// `save_post` body on a caller-owned connection. Reads that still go
/// through `db` helpers (load_post/terms_for) run on a second read
/// connection — safe under WAL because they only read pre-save state.
fn save_post_on(db: &Path, c: &Connection, f: &PostForm) -> Result<i64> {
    if !matches!(f.status.as_str(), "publish" | "future" | "draft" | "pending" | "trash") {
        anyhow::bail!("bad status");
    }
    // Editable publish date (phase 6.3): "YYYY-MM-DDTHH:MM" from the
    // datetime-local input normalizes to the stored "YYYY-MM-DD HH:MM:SS".
    // Empty keeps the existing value (or `now` on first save).
    let pub_at = norm_datetime(&f.published_at);
    // Empty date field → the stored value still decides scheduling (WP's
    // "publish immediately" is just a past date, and saves that never
    // render the input must not silently unschedule a 'future' post).
    let eff_date: Option<String> = pub_at.clone().or_else(|| {
        f.id.and_then(|id| {
            c.query_row(
                "SELECT published_at FROM posts WHERE id=?1",
                [id],
                |r| r.get::<_, Option<String>>(0),
            )
            .ok()
            .flatten()
        })
    });
    // WP `post_status` derivation: a 'publish' save dated in the future is
    // scheduled ('future'); a 'future' save moved to a past date publishes
    // immediately. Draft/pending/trash saves keep their status either way.
    let status = effective_status(&f.status, eff_date.as_deref());

    let mut body = f.body_json.clone();
    if body.trim().is_empty() {
        if let Some(id) = f.id {
            if let Ok(Some(old)) = load_post(db, id) {
                body = old.body_json;
            }
        }
    }
    // `#content` is WP post_content → posts.body_html. body_json (the GDLR
    // layout) only gets the merge for non-builder bodies; a gdlr body's
    // free text is never woven back into the layout JSON.
    if !f.content.trim().is_empty() && !body_is_gdlr(&body) {
        body = body_json_with_html(&body, &f.content);
    }

    if body.trim().is_empty() {
        body = "[]".into();
    }
    if serde_json::from_str::<serde_json::Value>(&body).is_err() {
        anyhow::bail!("body_json must be valid JSON");
    }
    let feat: Option<i64> = f.featured_media_id.trim().parse().ok();
    // SEO columns store the *resolved* title/description. When the Rank
    // Math metabox posted its fields, resolve `rank_math_title`/
    // `rank_math_description` with %var% substitution; otherwise preserve
    // the existing column values so a plain save never wipes them.
    let (mut seo_t, mut seo_d) = (f.seo_title.clone(), f.seo_desc.clone());
    let rm_raw = f.rank_math.trim();
    let rm_v = if rm_raw.is_empty() {
        None
    } else {
        serde_json::from_str::<serde_json::Value>(rm_raw).ok()
    };
    if let Some(v) = &rm_v {
        let sitename = site_name(db);
        let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
        let t = s("rank_math_title");
        seo_t = resolve_seo(if t.is_empty() { "%title% %sep% %sitename%" } else { &t }, &f.title, &sitename);
        seo_d = s("rank_math_description");
    }
    // No metabox payload (or unparseable one — save_rankmath bails on the
    // same payload): preserve each column independently so plain saves
    // never wipe stored SEO values.
    if rm_v.is_none() {
        if let Some(id0) = f.id {
            if seo_t.is_empty() || seo_d.is_empty() {
                if let Ok(Some(old)) = load_post(db, id0) {
                    if seo_t.is_empty() { seo_t = old.seo_title; }
                    if seo_d.is_empty() { seo_d = old.seo_desc; }
                }
            }
        }
    }
    let id = if let Some(id) = f.id {
        c.execute(
            sql!("posts__insert_into_revisions"),
            [id],
        )?;
        c.execute(
            sql!("posts__update_posts_set_16"),
            params![
                f.title, f.slug, f.lang, null_empty(&f.excerpt), null_empty(&seo_t),
                null_empty(&seo_d), status, feat, body, pub_at.as_deref(), f.content, id
            ],
        )?;
        id
    } else {
        // No explicit id: SQLite assigns the rowid — a MAX(id)+1 pre-read
        // races any other writer (audit M12). Read it back after insert.
        c.execute(
            sql!("posts__insert_into_posts_18"),
            params![
                f.slug, f.lang, f.title, body, null_empty(&f.excerpt), status,
                null_empty(&seo_t), null_empty(&seo_d), feat, f.kind, pub_at.as_deref(),
                f.content
            ],
        )?;
        c.last_insert_rowid()
    };
    if let Some(e) = crate::admin::plugins::editor_for(&f.kind) {
        (e.save)(c, id, f)?;
    }
    let es: Option<i64> = f.es_post_id.trim().parse().ok();
    c.execute(sql!("posts__delete_from_post_translations"), [id])?;
    if f.lang == "fr" {
        c.execute(
            sql!("posts__insert_into_post_translations"),
            params![id, es],
        )?;
        // Render/hreflang read post_translation_members (import-written);
        // mirror admin links so front/translation lookups see them too.
        sync_translation_members(c, id, es)?;
    }
    if f.kind == "post" {
        // cat[] + tags fields are authoritative for posts; other taxonomies
        // (language, ...) are preserved inside terms_payload. terms_payload
        // reads pre-save terms via a second read connection (WAL-safe).
        set_terms_on(c, id, &terms_payload(db, id, f))?;
    } else if !f.terms.trim().is_empty() {
        // non-post kinds keep raw terms if a form submits them; otherwise
        // their taxonomies (duree/region/style/language) stay untouched
        set_terms_on(c, id, &f.terms)?;
    }


    Ok(id)
}

/// Mirror the admin translation link into post_translation_members so the
/// N-language render lookups (hreflang, front pages, missing-translation
/// redirects) see admin-maintained links, not just import-written ones.
///
/// Membership semantics mirror the pair table: unlink removes only this
/// post's own row — other group members keep their link (a >2-language
/// group loses a member, not the group). Group id = the partner's
/// existing group, else `p{id}`.
fn sync_translation_members(c: &Connection, id: i64, other: Option<i64>) -> Result<()> {
    c.execute("DELETE FROM post_translation_members WHERE post_id=?1", [id])?;
    let Some(other) = other else { return Ok(()) };
    let group: Option<String> = c
        .query_row(
            "SELECT group_id FROM post_translation_members WHERE post_id=?1",
            [other],
            |r| r.get(0),
        )
        .ok();
    let group = group.unwrap_or_else(|| format!("p{id}"));
    let other_lang: String = c
        .query_row("SELECT lang FROM posts WHERE id=?1", [other], |r| r.get(0))
        .unwrap_or_default();
    if other_lang.is_empty() {
        return Ok(());
    }
    // Claim the (group, lang) slot: a different post already sitting there
    // loses its membership — scrub its post_translations pair too, or the
    // two tables disagree (front_id_for_lang's paired fallback would
    // resolve a link the group no longer has).
    let evict = |lang: &str, new_pid: i64| -> Result<()> {
        let evicted: Option<i64> = c
            .query_row(
                "SELECT post_id FROM post_translation_members WHERE group_id=?1 AND lang=?2",
                params![group, lang],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(ep) = evicted.filter(|&p| p != new_pid) {
            c.execute("DELETE FROM post_translation_members WHERE post_id=?1", [ep])?;
            c.execute(
                "DELETE FROM post_translations WHERE fr_post_id=?1 OR es_post_id=?1",
                [ep],
            )?;
        }
        Ok(())
    };
    evict("fr", id)?;
    evict(&other_lang, other)?;
    c.execute(
        "INSERT OR REPLACE INTO post_translation_members(group_id, lang, post_id) VALUES(?1,'fr',?2)",
        params![group, id],
    )?;
    c.execute(
        "INSERT OR REPLACE INTO post_translation_members(group_id, lang, post_id) VALUES(?1,?2,?3)",
        params![group, other_lang, other],
    )?;
    Ok(())
}



fn terms_payload(db: &Path, id: i64, f: &PostForm) -> String {
    if f.kind != "post" {
        return f.terms.clone();
    }
    let mut lines = Vec::new();
    if let Ok(old) = terms_for(db, id) {
        for (tax, slug, name) in old {
            if tax != "category" && tax != "post_tag" {
                lines.push(format!("{tax}:{slug}:{name}"));
            }
        }
    }
    for c in &f.cat {
        let mut p = c.splitn(2, ':');
        let slug = p.next().unwrap_or("").trim();
        let name = p.next().unwrap_or(slug).trim();
        let bad = |s: &str| s.is_empty() || s.contains(':') || s.contains('\n') || s.contains('\r');
        if !bad(slug) && !bad(name) {
            lines.push(format!("category:{slug}:{name}"));
        }
    }
    for tag in f.tags.split(',') {
        let name = tag.trim();
        if name.is_empty() || name.contains(':') || name.contains('\n') || name.contains('\r') {
            continue;
        }
        let slug = super::slugify(name);
        if slug.trim_matches('-').is_empty() {
            continue;
        }
        lines.push(format!("post_tag:{slug}:{name}"));
    }
    lines.join("\n")
}

/* ================= Dashboard widgets ================= */

/// Minimal row for dashboard widgets: id + title + a preformatted date
/// (publish date for recent posts, modified date for drafts).
pub struct DashPostRow {
    pub id: i64,
    pub title: String,
    pub date: String,
}

/// "Recently Published": `limit` newest published posts by publish date
/// (published_at, falling back to modified_at).
pub fn recent_posts(db: &Path, limit: i64) -> Result<Vec<DashPostRow>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("posts__dashboard_recent"))?;
    let rows = st
        .query_map([limit], |r| {
            Ok(DashPostRow {
                id: r.get(0)?,
                title: r.get(1)?,
                date: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// "Your Recent Drafts": `limit` newest post drafts authored by `user_id`.
/// Authorship lives in post_meta.author_id (posts has no author column).
pub fn drafts(db: &Path, user_id: i64, limit: i64) -> Result<Vec<DashPostRow>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("posts__dashboard_drafts"))?;
    let rows = st
        .query_map(params![user_id.to_string(), limit], |r| {
            Ok(DashPostRow {
                id: r.get(0)?,
                title: r.get(1)?,
                date: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/* ================= Scheduling & revisions (phase 6) ================= */

/// Normalize a `<input type="datetime-local">` value to SQLite's
/// `YYYY-MM-DD HH:MM:SS`. Accepts "…T…" or space separator, optional
/// seconds. Malformed input → None (keeps the stored value, never bails —
/// a fat-fingered date must not block a save).
pub fn norm_datetime(s: &str) -> Option<String> {
    let s = s.trim().replace('T', " ");
    if s.is_empty() {
        return None;
    }
    let ok_date = s.get(..10).is_some_and(|d| {
        d.len() == 10
            && d.as_bytes()[4] == b'-'
            && d.as_bytes()[7] == b'-'
            && d.bytes().filter(|b| b.is_ascii_digit()).count() == 8
    });
    let time = s.get(11..).unwrap_or("").trim();
    let parts: Vec<&str> = time.split(':').collect();
    if !ok_date || parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    if !parts.iter().all(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    let date = &s[..10];
    Some(if parts.len() == 3 {
        format!("{date} {}", parts.join(":"))
    } else {
        format!("{date} {}:00", parts[..2].join(":"))
    })
}

/// WP `post_status` derivation given the (effective) publish date:
/// 'publish' + future date → 'future'; 'future' + past/empty date →
/// 'publish'. Every other status passes through.
pub fn effective_status(status: &str, published_at: Option<&str>) -> String {
    let future = published_at
        .map(|d| d > crate::db::misc::now_sql().as_str())
        .unwrap_or(false);
    match (status, future) {
        ("publish", true) => "future".into(),
        ("future", false) => "publish".into(),
        (s, _) => s.to_string(),
    }
}

/// `check_future_posts` — flip due 'future' posts to 'publish'.
/// Runs on every admin request (get_sess); the SELECT+UPDATE are cheap
/// indexed scans. Returns `(id, slug, lang, kind)` of the posts that
/// flipped so the caller can regenerate their single output files.
pub fn publish_due(db: &Path) -> Result<Vec<(i64, String, String, String)>> {
    let c = crate::db::conn(db)?;
    let tx = c.unchecked_transaction()?;
    let mut st = tx.prepare(sql!("posts__publish_due"))?;
    let rows: Vec<(i64, String, String, String)> = st
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1).unwrap_or_default(),
                r.get::<_, String>(2).unwrap_or_default(),
                r.get::<_, String>(3).unwrap_or_default(),
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(st);
    if !rows.is_empty() {
        tx.execute(sql!("posts__publish_due_set"), [])?;
    }
    tx.commit()?;
    Ok(rows)
}

/// Decode a revisions.blob: post-6.3 rows store a JSON object
/// `{title, excerpt, body_json}`; the earlier writer stored the bare
/// body_json array. Both read the same here.
pub(crate) fn revision_parts(blob: &str) -> (String, String, String) {
    if let Ok(serde_json::Value::Object(m)) = serde_json::from_str::<serde_json::Value>(blob) {
        let g = |k: &str| m.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        return (g("title"), g("excerpt"), g("body_json"));
    }
    (String::new(), String::new(), blob.to_string())
}

/// (id, saved_at, byte_len) per revision, newest first.
pub fn revisions_for(db: &Path, post_id: i64) -> Result<Vec<(i64, String, i64)>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("revisions__select_for_post"))?;
    let rows = st
        .query_map([post_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// One revision: `(post_id, saved_at, title, excerpt, body_json)`.
pub fn revision_get(db: &Path, rid: i64) -> Result<Option<(i64, String, String, String, String)>> {
    let c = crate::db::conn(db)?;
    let row = c
        .query_row(sql!("revisions__get"), [rid], |r| {
            Ok((
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2).unwrap_or_default(),
                r.get::<_, String>(3).unwrap_or_default(),
            ))
        })
        .optional()?;
    Ok(row.map(|(post_id, saved_at, blob)| {
        let (t, e, b) = revision_parts(&blob);
        (post_id, saved_at, t, e, b)
    }))
}

/// Revision count for the editor "N revisions" metabox link.
pub fn revision_count(db: &Path, post_id: i64) -> i64 {
    crate::db::conn(db)
        .ok()
        .and_then(|c| {
            c.query_row(sql!("revisions__count"), [post_id], |r| r.get(0))
                .ok()
        })
        .unwrap_or(0)
}

/// Restore a revision onto its post (WP `_wp_restore_post_revision`):
/// snapshot the current row as a new revision first, then copy the
/// revision's title/excerpt/body back. Terms, metas, slug, status are
/// untouched — matching WP, which restores title+content+excerpt only.
/// The revision SELECT runs inside the transaction on ONE connection —
/// the old two-connection read-then-write could restore a revision row
/// already deleted between the SELECT and the UPDATE (audit TOCTOU).
pub fn restore_revision(db: &Path, rid: i64) -> Result<i64> {
    let c = crate::db::conn(db)?;
    let tx = c.unchecked_transaction()?;
    let row = tx
        .query_row(sql!("revisions__get"), [rid], |r| {
            Ok((
                r.get::<_, i64>(1)?,
                r.get::<_, String>(3).unwrap_or_default(),
            ))
        })
        .optional()?;
    let Some((post_id, blob)) = row else {
        anyhow::bail!("revision {rid} not found")
    };
    let (title, excerpt, body) = revision_parts(&blob);
    if body.is_empty() {
        anyhow::bail!("revision {rid} has no body");
    }
    tx.execute(sql!("posts__insert_into_revisions"), [post_id])?;
    tx.execute(
        "UPDATE posts SET title=COALESCE(NULLIF(?2,''), title), \
         excerpt=COALESCE(NULLIF(?3,''), excerpt), body_json=?4, \
         modified_at=datetime('now') WHERE id=?1",
        params![post_id, title, excerpt, body],
    )?;
    tx.commit()?;
    Ok(post_id)
}


#[cfg(test)]
mod tests {
    use super::*;

    fn memdb() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(crate::schema::DDL).unwrap();
        c
    }

    fn post(c: &Connection, id: i64, lang: &str) {
        c.execute(
            "INSERT INTO posts(id, slug, lang, title, body_json, status, kind) VALUES(?1,?1,?2,'t','[]','publish','page')",
            params![id, lang],
        )
        .unwrap();
    }

    fn members(c: &Connection, post_id: i64) -> Vec<(String, String)> {
        let mut s = c
            .prepare("SELECT group_id, lang FROM post_translation_members WHERE post_id=?1")
            .unwrap();
        s.query_map([post_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    }

    #[test]
    fn link_writes_members_and_unlink_removes_own_row() {
        let c = memdb();
        post(&c, 10, "fr");
        post(&c, 11, "es");
        post(&c, 20, "fr");
        post(&c, 21, "es");
        post(&c, 22, "de");

        // Import-style 3-language group g3 plus a link 10↔11 (group p10).
        c.execute_batch(
            "INSERT INTO post_translation_members(group_id,lang,post_id) VALUES('g3','fr',20),('g3','es',21),('g3','de',22);
             INSERT INTO post_translations(fr_post_id,es_post_id) VALUES(20,21);",
        )
        .unwrap();
        sync_translation_members(&c, 10, Some(11)).unwrap();
        assert_eq!(members(&c, 10), vec![("p10".into(), "fr".into())]);
        assert_eq!(members(&c, 11), vec![("p10".into(), "es".into())]);

        // Re-link 10↔21: 10 joins g3 and takes its 'fr' slot → 20 is
        // evicted; 20's member row AND its post_translations pair are
        // removed, while 22 keeps its group membership and 11 keeps hers.
        sync_translation_members(&c, 10, Some(21)).unwrap();
        assert_eq!(members(&c, 10), vec![("g3".into(), "fr".into())]);
        assert_eq!(members(&c, 21), vec![("g3".into(), "es".into())]);
        assert_eq!(members(&c, 22), vec![("g3".into(), "de".into())]);
        assert!(members(&c, 20).is_empty(), "evicted member lingers");
        assert_eq!(members(&c, 11), vec![("p10".into(), "es".into())]);
        let pairs: i64 = c
            .query_row("SELECT COUNT(*) FROM post_translations WHERE fr_post_id=20 OR es_post_id=20", [], |r| r.get(0))
            .unwrap();
        assert_eq!(pairs, 0, "evicted post's pair row left behind");

        // Unlink removes only 10's own row — the rest of g3 stays grouped.
        sync_translation_members(&c, 10, None).unwrap();
        assert!(members(&c, 10).is_empty());
        assert_eq!(members(&c, 21), vec![("g3".into(), "es".into())]);
        assert_eq!(members(&c, 22), vec![("g3".into(), "de".into())]);
    }
}

