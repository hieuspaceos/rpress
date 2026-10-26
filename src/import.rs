//! Import pass: parse dump → in-memory WP model → write rpress.db + report.

use crate::php_serialize::unserialize;
#[allow(unused_imports)]
use crate::sql;
use crate::sqldump::rows as dump_rows;
use anyhow::{Context, Result};
use rusqlite::params;
use serde_json::{json, Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::Write;
use std::path::Path;

const WANTED: &[&str] = &[
    "wp_posts",
    "wp_postmeta",
    "wp_terms",
    "wp_term_taxonomy",
    "wp_term_relationships",
    "wp_comments",
    "wp_commentmeta",
    "wp_options",
    "wp_rank_math_redirections",
    "wp_glsr_ratings",
    "wp_glsr_assigned_posts",
];

#[derive(Default)]
struct Post {
    id: i64,
    post_type: String,
    status: String,
    slug: String,
    title: String,
    content: String,
    excerpt: String,
    parent: i64,
    menu_order: i64,
    date: String,
    modified: String,
    mime: String,
    meta: HashMap<String, Vec<String>>,
}

#[derive(Default)]
struct Report {
    lines: Vec<String>,
    skipped: u64,
    unparsed: u64,
    unser_errors: u64,
}
impl Report {
    fn log(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
}

pub fn run(dump: &Path, db_path: &Path, report_path: &Path) -> Result<()> {
    // ---- Pass 1: load dump into memory ------------------------------------
    let mut terms: HashMap<i64, (String, String)> = HashMap::new(); // term_id -> (name, slug)
    let mut ttax: HashMap<i64, (i64, String, String)> = HashMap::new(); // tt_id -> (term_id, taxonomy, description)
    let mut rels: Vec<(i64, i64)> = Vec::new(); // object_id -> tt_id
    let mut posts: Vec<Post> = Vec::new();
    let mut post_meta: Vec<(i64, String, String)> = Vec::new();
    let mut comments: Vec<(i64, i64, i64, String, String, String, String, String, String, String)> = Vec::new();
    let mut comment_meta: Vec<(i64, String, String)> = Vec::new();
    let mut options: HashMap<String, String> = HashMap::new();
    let mut redirs: Vec<(String, String)> = Vec::new();
    let mut ratings: HashMap<i64, i64> = HashMap::new(); // rating_id -> review post_id
    let mut assigned: Vec<(i64, i64)> = Vec::new(); // rating_id -> assigned post_id

    let mut rep = Report::default();

    dump_rows(dump, WANTED, |table, v| {
        match table {
            "wp_terms" => {
                if let (Some(id), Some(name), Some(slug)) =
                    (v[0].as_i64(), v[1].to_string_lossy(), v[2].to_string_lossy())
                {
                    terms.insert(id, (name, slug));
                }
            }
            "wp_term_taxonomy" => {
                let tt_id = v[0].as_i64().unwrap_or(0);
                let term_id = v[1].as_i64().unwrap_or(0);
                let tax = v[2].to_string_lossy().unwrap_or_default();
                let desc = v[3].to_string_lossy().unwrap_or_default();
                ttax.insert(tt_id, (term_id, tax, desc));
            }
            "wp_term_relationships" => {
                if let (Some(o), Some(t)) = (v[0].as_i64(), v[1].as_i64()) {
                    rels.push((o, t));
                }
            }
            "wp_postmeta" => {
                if let (Some(pid), Some(k), Some(mv)) =
                    (v[1].as_i64(), v[2].to_string_lossy(), v[3].to_string_lossy())
                {
                    post_meta.push((pid, k, mv));
                }
            }
            "wp_comments" => {
                let approved = v[10].as_str().unwrap_or("");
                if !matches!(approved, "1" | "0") {
                    return Ok(()); // spam/trash skipped
                }
                comments.push((
                    v[0].as_i64().unwrap_or(0),
                    v[1].as_i64().unwrap_or(0),
                    v[13].as_i64().unwrap_or(0),
                    v[2].to_string_lossy().unwrap_or_default(),
                    v[3].to_string_lossy().unwrap_or_default(),
                    v[4].to_string_lossy().unwrap_or_default(),
                    v[5].to_string_lossy().unwrap_or_default(),
                    v[8].to_string_lossy().unwrap_or_default(),
                    v[6].to_string_lossy().unwrap_or_default(),
                    approved.to_string(),
                ));
            }
            "wp_commentmeta" => {
                if let (Some(cid), Some(k), Some(mv)) =
                    (v[1].as_i64(), v[2].to_string_lossy(), v[3].to_string_lossy())
                {
                    comment_meta.push((cid, k, mv));
                }
            }
            "wp_options" => {
                if let (Some(k), Some(mv)) =
                    (v[1].to_string_lossy(), v[2].to_string_lossy())
                {
                    // Carry every option the runtime reads so a migrated site
                    // keeps its source site's moderation/media/feed config
                    // instead of falling back to built-in defaults.
                    if matches!(
                        k.as_str(),
                        "page_on_front" | "show_on_front" | "siteurl" | "tourmaster_general"
                            | "blogname" | "rss_use_excerpt"
                            | "require_name_email" | "comment_moderation"
                            | "comment_max_links" | "comment_previously_approved"
                            | "default_comment_status" | "moderation_keys"
                            | "disallowed_keys" | "blacklist_keys"
                            | "close_comments_for_old_posts"
                            | "show_comments_cookies_opt_in" | "thread_comments"
                            | "thumbnail_size_w" | "thumbnail_size_h" | "thumbnail_crop"
                            | "medium_size_w" | "medium_size_h"
                            | "medium_large_size_w" | "medium_large_size_h"
                            | "large_size_w" | "large_size_h" | "uploads_use_yearmonth"
                            | "gdlr-core-page-builder-view-mode"
                    ) {
                        options.insert(k, mv);
                    }
                }
            }
            "wp_rank_math_redirections" => {
                if v[5].as_str() == Some("active") {
                    if let (Some(src), Some(dst)) =
                        (v[1].to_string_lossy(), v[2].to_string_lossy())
                    {
                        redirs.push((src, dst));
                    }
                }
            }
            "wp_glsr_ratings" => {
                if let (Some(rid), Some(rev)) = (v[0].as_i64(), v[1].as_i64()) {
                    ratings.insert(rid, rev);
                }
            }
            "wp_glsr_assigned_posts" => {
                if let (Some(rid), Some(pid)) = (v[0].as_i64(), v[1].as_i64()) {
                    assigned.push((rid, pid));
                }
            }
            "wp_posts" => {
                let pt = v[20].to_string_lossy().unwrap_or_default();
                let st = v[7].to_string_lossy().unwrap_or_default();
                posts.push(Post {
                    id: v[0].as_i64().unwrap_or(0),
                    post_type: pt,
                    status: st,
                    slug: v[11].to_string_lossy().unwrap_or_default(),
                    title: v[5].to_string_lossy().unwrap_or_default(),
                    content: v[4].to_string_lossy().unwrap_or_default(),
                    excerpt: v[6].to_string_lossy().unwrap_or_default(),
                    parent: v[17].as_i64().unwrap_or(0),
                    menu_order: v[19].as_i64().unwrap_or(0),
                    date: v[2].to_string_lossy().unwrap_or_default(),
                    modified: v[14].to_string_lossy().unwrap_or_default(),
                    mime: v[21].to_string_lossy().unwrap_or_default(),
                    meta: HashMap::new(),
                });
            }
            _ => {}
        }
        Ok(())
    })?;

    // fold postmeta into posts
    let idx: HashMap<i64, usize> = posts.iter().enumerate().map(|(i, p)| (p.id, i)).collect();
    for (pid, k, mv) in post_meta {
        if let Some(&i) = idx.get(&pid) {
            posts[i].meta.entry(k).or_default().push(mv);
        }
    }

    // ---- Pass 2: map → SQLite ----------------------------------------------
    let _ = std::fs::remove_file(db_path);
    let conn = crate::db::conn(db_path).context("open rpress.db")?;
    crate::schema::migrate_conn(&conn)?;
    conn.execute_batch(sql!("import__pragma_synchronous_off"))?;
    let tx = conn.unchecked_transaction()?;

    // language map: post_id -> lang; translation pairs: serialized descriptions
    let mut post_lang: HashMap<i64, String> = HashMap::new();
    let mut languages: BTreeSet<String> = BTreeSet::new();
    for (obj, tt_id) in &rels {
        if let Some((_, tax, _)) = ttax.get(tt_id) {
            if tax == "language" {
                if let Some((_, slug)) = ttax.get(tt_id).and_then(|(t, _, _)| terms.get(t)) {
                    languages.insert(slug.clone());
                    post_lang.insert(*obj, slug.clone());
                }
            }
        }
    }
    // post_translations: description = {lang: object_id}. Keep the legacy
    // fr/es pair table for old admin screens, and also populate the generic
    // post_translation_members table for N-language render/hreflang.
    let mut trans_rows: Vec<(Option<i64>, Option<i64>)> = Vec::new();
    let mut trans_groups: Vec<(String, Vec<(String, i64)>)> = Vec::new();
    let mut term_trans_pairs: Vec<(i64, i64)> = Vec::new();
    for (_tt, (term_id, tax, desc)) in &ttax {
        if tax == "post_translations" && desc.starts_with("a:") {
            match unserialize(desc.as_bytes()) {
                Ok(Value::Object(m)) => {
                    let fr = m.get("fr").and_then(|v| v.as_i64());
                    let es = m.get("es").and_then(|v| v.as_i64());
                    let members: Vec<(String, i64)> = m
                        .iter()
                        .filter_map(|(lang, v)| v.as_i64().map(|id| (lang.clone(), id)))
                        .collect();
                    if fr.is_some() || es.is_some() {
                        trans_rows.push((fr, es));
                    }
                    if !members.is_empty() {
                        trans_groups.push((term_id.to_string(), members));
                    } else {
                        rep.log(format!("translation term {} empty group", term_id));
                    }
                }
                Ok(_) => rep.log(format!("translation term {} desc not map", term_id)),
                Err(e) => {
                    rep.unser_errors += 1;
                    rep.log(format!("translation term {} unserialize: {}", term_id, e))
                }
            }
        }
        if tax == "term_translations" && desc.starts_with("a:") {
            if let Ok(Value::Object(m)) = unserialize(desc.as_bytes()) {
                if let (Some(f), Some(e)) =
                    (m.get("fr").and_then(|v| v.as_i64()), m.get("es").and_then(|v| v.as_i64()))
                {
                    term_trans_pairs.push((f, e));
                }
            }
        }
    }

    let kind_of = |t: &str| -> Option<&'static str> {
        match t {
            "post" => Some("post"),
            "page" => Some("page"),
            "tour" => Some("tour"),
            "site-review" => Some("site-review"),
            _ => None,
        }
    };

    let mut stmt_post = tx.prepare(
        sql!("import__insert_into_posts_2"),
    )?;
    let mut stmt_meta = tx.prepare(
        sql!("import__insert_or_ignore_3"),
    )?;
    let mut stmt_media = tx.prepare(
        sql!("import__insert_into_media"),
    )?;
    let mut stmt_pt = tx.prepare(
        sql!("import__insert_or_ignore_5"),
    )?;
    let mut stmt_comment = tx.prepare(
        sql!("import__insert_into_comments_6"),
    )?;
    let mut stmt_ra = tx.prepare(
        sql!("import__insert_or_ignore_7"),
    )?;
    let mut stmt_redir = tx.prepare(sql!("import__insert_into_redirects"))?;
    let mut stmt_menu = tx.prepare(
        sql!("import__insert_into_menus"),
    )?;
    let mut stmt_mi = tx.prepare(
        sql!("import__insert_into_menu_items"),
    )?;
    let mut stmt_trans = tx.prepare(
        sql!("import__insert_or_ignore_11"),
    )?;
    let mut stmt_trans_member = tx.prepare(sql!("import__insert_or_ignore_translation_member"))?;
    let mut stmt_term = tx.prepare(sql!("import__insert_or_replace_terms"))?;
    let mut stmt_ttrans = tx.prepare(
        sql!("import__insert_or_ignore_12"),
    )?;
    let mut stmt_set = tx.prepare(sql!("import__insert_or_replace"))?;

    let mut wp_to_cms: HashMap<i64, i64> = HashMap::new();

    // posts pass
    for p in &posts {
        match kind_of(&p.post_type) {
            None => {
                if p.post_type == "attachment" {
                    import_media(p, &mut stmt_media, &mut rep)?;
                } else if p.post_type == "nav_menu_item" {
                    // handled below with menus
                } else if p.post_type == "revision" {
                    // dropped by design (schema exists; plan keeps revisions
                    // table for CMS-created revisions only)
                } else if p.post_type == "custom_css" && p.status == "publish" {
                    // WP Customizer "Additional CSS" — post_content is the
                    // site's hand-written stylesheet. Renderer emits it as
                    // an inline <style> in <head>, same as wp_custom_css_cb.
                    stmt_set.execute(params!["custom_css", p.content])?;
                    rep.skipped += 1;
                }
                continue;
            }
            Some(kind) => {
                if !matches!(p.status.as_str(), "publish" | "draft") {
                    rep.skipped += 1;
                    rep.log(format!("skip {} {} status={}", p.post_type, p.id, p.status));
                    continue;
                }
                let lang = match post_lang.get(&p.id) {
                    Some(l) => l.clone(),
                    None => {
                        rep.log(format!("WARN post {} has no language term, default fr", p.id));
                        "fr".to_string()
                    }
                };
                let body_json = build_body_json(p, &mut rep);
                let seo_title = meta1(p, "rank_math_title");
                let seo_desc = meta1(p, "rank_math_description");
                let feat: Option<i64> = meta1(p, "_thumbnail_id").and_then(|s| s.parse().ok());
                let template = meta1(p, "_wp_page_template");
                stmt_post.execute(params![
                    p.id,
                    p.slug,
                    lang,
                    p.title,
                    body_json,
                    null_if_empty(&p.excerpt),
                    p.status,
                    null_zero(p.parent),
                    p.menu_order,
                    p.date,
                    p.modified,
                    seo_title,
                    seo_desc,
                    feat,
                    kind,
                    template,
                    // ?17 body_html = real wp post_content (the TinyMCE
                    // field); body_json above stays the GDLR layout.
                    p.content,
                ])?;
                wp_to_cms.insert(p.id, p.id);
                for (k, vals) in &p.meta {
                    for mv in vals {
                        stmt_meta.execute(params![p.id, k, mv])?;
                    }
                }
            }
        }
    }

    // post_terms for imported posts
    for (obj, tt_id) in &rels {
        if !wp_to_cms.contains_key(obj) {
            continue;
        }
        if let Some((term_id, tax, _)) = ttax.get(tt_id) {
            if matches!(
                tax.as_str(),
                "language" | "post_translations" | "term_translations" | "post_format"
            ) {
                continue;
            }
            if let Some((name, slug)) = terms.get(term_id) {
                stmt_pt.execute(params![obj, tax, slug, name])?;
            }
        }
    }

    // Rows keep members that exist as real wp_posts (ghost ids = Polylang
    // pointing at deleted posts). A pair with one ghost collapses; a fully
    // ghost group is dropped.
    // terms table: every wp_term_taxonomy row that is a real site term
    // (language/translation taxonomies are Polylang internals, not terms).
    // id = term_id — WP's cat-item-N classes and term_translations use it.
    for (_tt, (term_id, tax, desc)) in &ttax {
        if matches!(tax.as_str(), "language" | "post_translations" | "term_translations" | "post_format") {
            continue;
        }
        if let Some((name, slug)) = terms.get(term_id) {
            stmt_term.execute(params![term_id, tax, slug, name, desc])?;
        }
    }
    let all_post_ids: std::collections::HashSet<i64> = posts.iter().map(|p| p.id).collect();
    for (f, e) in &trans_rows {
        if f.is_some_and(|id| !all_post_ids.contains(&id)) || e.is_some_and(|id| !all_post_ids.contains(&id)) {
            rep.log(format!("GHOST translation member {:?}/{:?} (deleted post)", f, e));
        }
        let fi = f.filter(|id| all_post_ids.contains(id) && wp_to_cms.contains_key(id));
        let ei = e.filter(|id| all_post_ids.contains(id) && wp_to_cms.contains_key(id));
        if fi.is_some() || ei.is_some() {
            stmt_trans.execute(params![fi, ei])?;
        } else {
            rep.log(format!("translation group {:?}/{:?} no imported members", f, e));
        }
    }
    for (group_id, members) in &trans_groups {
        let mut kept = 0;
        for (lang, id) in members {
            if all_post_ids.contains(id) && wp_to_cms.contains_key(id) {
                stmt_trans_member.execute(params![group_id, lang, id])?;
                kept += 1;
            }
        }
        if kept == 0 {
            rep.log(format!("translation group {group_id} no generic imported members"));
        }
    }
    for (f, e) in &term_trans_pairs {
        stmt_ttrans.execute(params![f, e])?;
    }

    // comments
    let mut rating_of_comment: HashMap<i64, i64> = HashMap::new();
    for (cid, k, mv) in &comment_meta {
        if k == "rating" {
            if let Ok(r) = mv.parse() {
                rating_of_comment.insert(*cid, r);
            }
        }
    }
    for (cid, post_id, parent, author, email, author_url, author_ip, body, date, approved) in &comments {
        if !wp_to_cms.contains_key(post_id) {
            rep.skipped += 1;
            rep.log(format!("skip comment {} on unimported post {}", cid, post_id));
            continue;
        }
        let status = if approved == "1" { "approved" } else { "pending" };
        let rating = rating_of_comment.get(cid).copied();
        stmt_comment.execute(params![cid, post_id, null_zero(*parent), author, email, author_url, author_ip, body, status, date, rating])?;
    }

    // review assignments via glsr tables
    for (rid, pid) in &assigned {
        if let Some(rev) = ratings.get(rid) {
            if wp_to_cms.contains_key(pid) {
                stmt_ra.execute(params![rev, pid])?;
            }
        }
    }

    // redirects
    let mut redir_count = 0u64;
    for (src, dst) in &redirs {
        match unserialize(src.as_bytes()) {
            Ok(Value::Array(list)) => {
                for item in list {
                    if let Value::Object(m) = item {
                        let pattern = m.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
                        if !pattern.is_empty() {
                            stmt_redir.execute(params![pattern, dst, "rank-math"])?;
                            redir_count += 1;
                        }
                    }
                }
            }
            Ok(_) => rep.log(format!("redir sources not array: {}", &src[..src.len().min(60)])),
            Err(e) => {
                rep.unser_errors += 1;
                rep.log(format!("redir unserialize: {} :: {}", e, &src[..src.len().min(60)]))
            }
        }
    }
    for p in &posts {
        if !wp_to_cms.contains_key(&p.id) {
            continue;
        }
        if let Some(vals) = p.meta.get("_wp_old_slug") {
            for old in vals {
                let new = format!("/{}/", p.slug);
                stmt_redir.execute(params![old, new, "old-slug"])?;
                redir_count += 1;
            }
        }
    }
    rep.log(format!("redirects imported: {}", redir_count));

    // menus: nav_menu terms + nav_menu_item posts
    let mut menu_wp: HashMap<i64, i64> = HashMap::new();
    for (tt_id, (term_id, tax, _)) in &ttax {
        if tax != "nav_menu" {
            continue;
        }
        let (name, slug) = terms.get(term_id).cloned().unwrap_or_default();
        let lang = if slug.ends_with("-es") { "es" } else { "fr" };
        stmt_menu.execute(params![term_id, name, slug, lang])?;
        menu_wp.insert(*tt_id, *term_id);
    }
    let mut item_lang: HashMap<i64, i64> = HashMap::new(); // wp item id -> menu db id
    for (obj, tt_id) in &rels {
        if let Some(mid) = menu_wp.get(tt_id) {
            item_lang.insert(*obj, *mid);
        }
    }
    for p in &posts {
        if p.post_type != "nav_menu_item" || p.status != "publish" {
            continue;
        }
        let Some(&mid) = item_lang.get(&p.id) else {
            rep.log(format!("menu item {} not in any menu", p.id));
            continue;
        };
        let type_ = meta1(p, "_menu_item_type").unwrap_or_default();
        let obj_type = meta1(p, "_menu_item_object").unwrap_or_default();
        let item_type = if obj_type == "language" {
            "lang-switcher".to_string()
        } else {
            type_.clone()
        };
        let label = if p.title.is_empty() {
            meta1(p, "_menu_item_title").unwrap_or_default()
        } else {
            p.title.clone()
        };
        let url = if type_ == "custom" {
            meta1(p, "_menu_item_url").unwrap_or_default()
        } else {
            String::new()
        };
        let object_id: Option<i64> = meta1(p, "_menu_item_object_id").and_then(|s| s.parse().ok());
        stmt_mi.execute(params![
            p.id,
            mid,
            meta1(p, "_menu_item_menu_item_parent")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|n| if n == 0 { None } else { Some(n) })
                .flatten(),
            p.menu_order,
            label,
            url,
            object_id,
            item_type,
        ])?;
    }

    if !languages.is_empty() {
        let langs = languages.iter().cloned().collect::<Vec<_>>().join(",");
        let primary = if languages.contains("fr") { "fr".to_string() } else { languages.iter().next().cloned().unwrap_or_else(|| "fr".into()) };
        stmt_set.execute(params!["languages", langs])?;
        stmt_set.execute(params!["primary_lang", primary])?;
    }

    for (k, v) in &options {
        // Rename WP option names to the keys this codebase reads.
        let k = match k.as_str() {
            "siteurl" => "site_url",
            "blogname" => "site_name",
            "blacklist_keys" => "disallowed_keys",
            "close_comments_for_old_posts" => "close_comments",
            "show_comments_cookies_opt_in" => "comment_cookies",
            other => other,
        };
        stmt_set.execute(params![k, v])?;
    }

    drop((
        stmt_post, stmt_meta, stmt_media, stmt_pt, stmt_comment, stmt_ra, stmt_redir, stmt_menu,
        stmt_mi, stmt_trans, stmt_trans_member, stmt_term, stmt_ttrans, stmt_set,
    ));
    tx.commit()?;

    rep.log(format!("skipped rows: {}", rep.skipped));
    rep.log(format!("unserialize errors: {}", rep.unser_errors));
    rep.log(format!("unparsed bodies: {}", rep.unparsed));

    let mut f = File::create(report_path)?;
    for l in &rep.lines {
        writeln!(f, "{}", l)?;
    }
    println!("imported; report at {} ({} lines)", report_path.display(), rep.lines.len());
    Ok(())
}

fn meta1(p: &Post, key: &str) -> Option<String> {
    p.meta.get(key).and_then(|v| v.first().cloned())
}

fn null_if_empty(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_string()) }
}
fn null_zero(n: i64) -> Option<i64> {
    if n == 0 { None } else { Some(n) }
}

/// gdlr serialized blob → JSON; fallback = single text-box with post_content.
fn build_body_json(p: &Post, rep: &mut Report) -> String {
    if let Some(blob) = meta1(p, "gdlr-core-page-builder") {
        if !blob.is_empty() && blob.starts_with("a:") {
            match unserialize(blob.as_bytes()) {
                Ok(v) => return v.to_string(),
                Err(e) => {
                    rep.unparsed += 1;
                    rep.log(format!("post {} gdlr unparse: {}", p.id, e));
                }
            }
        }
    }
    // fallback
    let mut el = Map::new();
    el.insert("template".into(), json!("element"));
    el.insert("type".into(), json!("text-box"));
    let mut val = Map::new();
    val.insert("content".into(), json!(p.content));
    el.insert("value".into(), Value::Object(val));
    json!([Value::Object(el)]).to_string()
}

fn import_media(p: &Post, stmt: &mut rusqlite::Statement, rep: &mut Report) -> Result<()> {
    if p.status != "inherit" {
        rep.skipped += 1;
        return Ok(());
    }
    let meta_raw = meta1(p, "_wp_attachment_metadata").unwrap_or_default();
    let mut w: Option<i64> = None;
    let mut h: Option<i64> = None;
    let mut sizes = json!({});
    let mut path = meta1(p, "_wp_attached_file").unwrap_or_default();
    if let Ok(v) = unserialize(meta_raw.as_bytes()) {
        w = v.get("width").and_then(|x| x.as_i64());
        h = v.get("height").and_then(|x| x.as_i64());
        if path.is_empty() {
            path = v.get("file").and_then(|x| x.as_str()).unwrap_or("").to_string();
        }
        if let Some(Value::Object(m)) = v.get("sizes").cloned() {
            sizes = Value::Object(m);
        }
    }
    if path.is_empty() {
        rep.log(format!("attachment {} has no file path", p.id));
        rep.skipped += 1;
        return Ok(());
    }
    stmt.execute(params![
        p.id,
        path,
        meta1(p, "_wp_attachment_image_alt"),
        null_if_empty(&p.excerpt).or_else(|| null_if_empty(&p.content)),
        p.mime,
        w,
        h,
        sizes.to_string(),
    ])?;
    Ok(())
}
