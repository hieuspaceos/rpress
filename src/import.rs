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
    "wp_termmeta",
    "wp_options",
    "wp_rank_math_redirections",
    "wp_glsr_ratings",
    "wp_glsr_assigned_posts",
];

#[derive(Default)]
struct Post {
    id: i64,
    author: i64,
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
    /* Table prefix varies (`wp_`, `wp_42426_`, …) — detect it once, then
       ask `rows` for `<prefix><suffix>` but report canonical `wp_*` names
       so all downstream matching stays prefix-free. */
    let prefix = crate::sqldump::detect_prefix(dump)?;
    let physical: Vec<String> = WANTED.iter().map(|t| format!("{prefix}{}", &t[3..])).collect();
    let phys: Vec<&str> = physical.iter().map(|s| s.as_str()).collect();
    // ---- Pass 1: load dump into memory ------------------------------------
    let mut terms: HashMap<i64, (String, String)> = HashMap::new(); // term_id -> (name, slug)
    let mut users: HashMap<i64, String> = HashMap::new(); // user_id -> display/login
    let mut ttax: HashMap<i64, (i64, String, String)> = HashMap::new(); // tt_id -> (term_id, taxonomy, description)
    let mut rels: Vec<(i64, i64)> = Vec::new(); // object_id -> tt_id
    let mut posts: Vec<Post> = Vec::new();
    let mut post_meta: Vec<(i64, String, String)> = Vec::new();
    let mut pll_metas: Vec<(i64, String)> = Vec::new();
    let mut comments: Vec<(i64, i64, i64, String, String, String, String, String, String, String)> = Vec::new();
    let mut comment_meta: Vec<(i64, String, String)> = Vec::new();
    let mut options: HashMap<String, String> = HashMap::new();
    let mut redirs: Vec<(String, String)> = Vec::new();
    let mut ratings: HashMap<i64, i64> = HashMap::new(); // rating_id -> review post_id
    let mut assigned: Vec<(i64, i64)> = Vec::new(); // rating_id -> assigned post_id

    let mut rep = Report::default();

    dump_rows(dump, &phys, |table, v| {
        /* canonicalize: `wp_42426_postmeta` → `wp_postmeta` */
        let table: &str = match table.strip_prefix(prefix.as_str()) {
            Some("posts") => "wp_posts",
            Some("postmeta") => "wp_postmeta",
            Some("terms") => "wp_terms",
            Some("term_taxonomy") => "wp_term_taxonomy",
            Some("term_relationships") => "wp_term_relationships",
            Some("comments") => "wp_comments",
            Some("commentmeta") => "wp_commentmeta",
            Some("options") => "wp_options",
            Some("users") => "wp_users",
            _ => table,
        };
        match table {
            "wp_users" => {
                // id -> login (display_name is v[8] but login is what WP
                // renders in bylines); feeds post_meta 'author_login'.
                if let Some(uid) = v[0].as_i64() {
                    let login = v[1].to_string_lossy().unwrap_or_default();
                    let dname = v[8].to_string_lossy().unwrap_or_default();
                    users.insert(uid, if dname.is_empty() { login } else { dname });
                }
            }
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
            "wp_termmeta" => {
                /* Polylang string translations live on the synthetic
                   `pll_language` terms' termmeta — capture raw values;
                   harvested after stmt_set into pll_string_map. */
                if let (Some(tid), Some(k), Some(mv)) = (
                    v[1].as_i64(),
                    v[2].to_string_lossy(),
                    v[3].to_string_lossy(),
                ) {
                    if k == "_pll_strings_translations" {
                        pll_metas.push((tid, mv));
                    }
                }
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
                    // instead of falling back to built-in defaults. Plugin
                    // options whitelist by the adapter's declared prefixes
                    // (`ImportHooks.option_prefixes`).
                    let plugin_opt = crate::admin::plugins::adapters()
                        .iter()
                        .filter_map(|a| a.import_hooks())
                        .flat_map(|h| h.option_prefixes.iter())
                        .any(|p| k.starts_with(p));
                    if plugin_opt || matches!(
                        k.as_str(),
                        "page_on_front" | "show_on_front" | "siteurl"
                            | "stylesheet" | "template" | "active_plugins"
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
                            | "gdlr-core-page-builder-view-mode" | "polylang"
                            | "blogdescription" | "traveltour_general"
                            | "ssb_buttons" | "ssb_settings" | "site_icon" | "auhfc_settings_sitewide"
                    ) || k.starts_with("theme_mods_")
                        || k.starts_with("chaty_")
                        || k.starts_with("cht_") {
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
                    author: v[1].as_i64().unwrap_or(0),
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

    // Some dumps contain duplicate wp_posts ids (stale rows after a failed
    // migration on the source). Keep the first occurrence — an INSERT
    // collision on posts.id would abort the whole import otherwise.
    let mut seen_ids = std::collections::HashSet::new();
    let before = posts.len();
    posts.retain(|p| seen_ids.insert(p.id));
    if posts.len() != before {
        rep.log(format!("skipped {} duplicate wp_posts id(s)", before - posts.len()));
    }

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
    // Site primary language — resolved from Polylang's serialized option
    // (default_language), falling back to the first detected lang. Needed
    // before the post loop: posts without a language term get this lang.
    let primary = options
        .get("polylang")
        .and_then(|v| {
            regex::Regex::new(r#"default_lang(?:uage)?";s:\d+:"([a-z]+)""#)
                .ok()?
                .captures(v)
                .map(|c| c[1].to_string())
        })
        .filter(|l| languages.contains(l))
        .or_else(|| languages.iter().next().cloned())
        .unwrap_or_else(|| "en".into());

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
        // Plugin adapters claim their wp post_types first; core post/page
        // fall through.
        for a in crate::admin::plugins::adapters() {
            if let Some(kind) = a.import_hooks().and_then(|h| (h.claim_type)(t)) {
                return Some(kind);
            }
        }
        match t {
            "post" => Some("post"),
            "page" => Some("page"),
            "site-review" => Some("site-review"),
            _ => None,
        }
    };

    let mut stmt_post = tx.prepare(
        sql!("import__insert_into_posts_2"),
    )?;
    let mut dropped_meta: u64 = 0;
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
    /* Polylang string translations: `_pll_strings_translations` post meta
       holds [non-primary, primary] string pairs for theme/plugin option
       strings (top-bar text, copyright…). The carrier posts have no
       language term so they never import — harvest the pairs now into a
       setting the chrome reads per-lang. langs order: [other, primary]. */
    {
        let mut pairs: Vec<(String, String)> = Vec::new();
        /* Each `_pll_strings_translations` meta lives on a language term:
           its pairs are [other_lang, this_term_lang]. Resolve the
           carrier term's slug so (primary, translated) order is right
           regardless of registration order. */
        for (tid, mv) in &pll_metas {
            let term_lang = ttax
                .values()
                .find(|(t, tax, _)| t == tid && tax == "language")
                .and_then(|(t, _, _)| terms.get(t))
                .map(|(_, slug)| slug.clone())
                .unwrap_or_else(|| primary.clone());
            if let Ok(serde_json::Value::Array(list)) =
                crate::php_serialize::unserialize(mv.as_bytes())
            {
                for pair in list {
                    if let serde_json::Value::Array(p2) = &pair {
                        if p2.len() == 2 {
                            let first = p2[0].as_str().unwrap_or("");
                            let second = p2[1].as_str().unwrap_or("");
                            if first.is_empty() || second.is_empty() {
                                continue;
                            }
                            // element on the term-lang side = [i1];
                            // source (primary) = the other element.
                            let (prim, tr) = if term_lang == primary {
                                (second.to_string(), first.to_string())
                            } else {
                                (first.to_string(), second.to_string())
                            };
                            pairs.push((prim, tr));
                        }
                    }
                }
            }
        }
        if !pairs.is_empty() {
            stmt_set.execute(params![
                "pll_string_map",
                serde_json::to_string(&pairs)?
            ])?;
        }
    }


    // Posts whose body was authored in a page builder rpress can't
    // interpret (Elementor today; the table is the hook for more) — generate
    // harvests their rendered HTML from the origin instead of re-rendering.
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS harvest_posts(post_id INTEGER PRIMARY KEY)",
    )?;
    let mut stmt_h = tx.prepare("INSERT OR IGNORE INTO harvest_posts(post_id) VALUES(?1)")?;

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
                        rep.log(format!("WARN post {} has no language term, default {}", p.id, primary));
                        primary.clone()
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
                if p.meta.contains_key("_elementor_data") {
                    stmt_h.execute(params![p.id])?;
                }
                /* Bylines: WP's post_author → display name survives the
                   import as post_meta 'author_login' (posts has no author
                   column; chrome/archive read it from meta). */
                if let Some(name) = users.get(&p.author) {
                    let _ = stmt_meta.execute(params![p.id, "author_login", name]);
                }
                for (k, vals) in &p.meta {
                    for mv in vals {
                        // OR IGNORE + the (post_id,meta_key) unique index:
                        // extra values for one key silently drop — count
                        // them so the loss is visible in the report.
                        if stmt_meta.execute(params![p.id, k, mv])? == 0 {
                            dropped_meta += 1;
                        }
                    }
                }
                // Plugin per-row hooks run inside the import tx (T13).
                for a in crate::admin::plugins::adapters() {
                    if let Some(h) = a.import_hooks() {
                        (h.post_imported)(&tx, p.id)?;
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
        // Menu lang: Polylang names translated menus `<slug>-<lang>` — take
        // the longest matching lang suffix, fall back to the primary lang.
        let lang = languages
            .iter()
            .filter(|l| slug.ends_with(&format!("-{l}")))
            .max_by_key(|l| l.len())
            .cloned()
            .unwrap_or_else(|| primary.clone());
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

    // Expected counts captured at parse time — `import verify` compares the
    // DB against these instead of per-site constants. Counted from the source
    // collections, not the DB, so dropped rows still surface as mismatches.
    // Monolingual sites have no `language` taxonomy — declare the set as just
    // the primary lang so languages/primary_lang are always written and posts
    // (all assigned `primary`) are never "without lang".
    if languages.is_empty() {
        languages.insert(primary.clone());
    }
    let langs = languages.iter().cloned().collect::<Vec<_>>().join(",");
    stmt_set.execute(params!["languages", langs])?;
    stmt_set.execute(params!["primary_lang", primary.clone()])?;

    {
        let mut expect: std::collections::BTreeMap<String, i64> = Default::default();
        // Mirror the insert filters exactly — count only rows that can
        // land: posts must be a known kind in publish|draft; media must be
        // an inherit attachment with a resolvable file; menu items must be
        // published and assigned to a menu.
        for p in &posts {
            match p.post_type.as_str() {
                "attachment" => {
                    if p.status == "inherit" && !attachment_path(p).is_empty() {
                        *expect.entry("media".into()).or_default() += 1;
                    }
                }
                "nav_menu_item" => {
                    if p.status == "publish" && item_lang.contains_key(&p.id) {
                        *expect.entry("menu_items".into()).or_default() += 1;
                    }
                }
                pt => {
                    if let Some(kind) = kind_of(pt) {
                        if matches!(p.status.as_str(), "publish" | "draft") {
                            let lang = post_lang.get(&p.id).cloned().unwrap_or_else(|| primary.clone());
                            *expect.entry(format!("posts:{kind}:{lang}")).or_default() += 1;
                        }
                    }
                }
            }
        }
        // menus come from nav_menu terms, not wp_posts
        expect.insert("menus".into(), ttax.values().filter(|(_, tax, _)| tax == "nav_menu").count() as i64);
        // Same rule for comments and translation members: rows pointing at
        // non-imported posts (drafts, revisions, attachments) are skipped
        // by design — count only rows that can actually land.
        expect.insert("comments_total".into(),
            comments.iter().filter(|c| wp_to_cms.contains_key(&c.1)).count() as i64);
        expect.insert("redirects".into(), redirs.len() as i64);
        expect.insert("translation_members".into(),
            trans_groups.iter()
                .map(|(_, m)| m.iter().filter(|(_, id)| wp_to_cms.contains_key(id)).count() as i64)
                .sum());
        stmt_set.execute(params!["import_expect", serde_json::to_string(&expect)?])?;
    }

    for (k, v) in &options {
        // Rename WP option names to the keys this codebase reads.
        let k = match k.as_str() {
            "siteurl" => "site_url",
            "blogname" => "site_name",
            "blacklist_keys" => "disallowed_keys",
            "close_comments_for_old_posts" => "close_comments",
            "show_comments_cookies_opt_in" => "comment_cookies",
            /* Theme identity is needed by the PHP bridge (theme filters like
               `gdlr_core_skin_option_slug` only register when the real theme
               is active). Keep WP's option names — no rename: readers use
               `stylesheet`/`template` verbatim. */
            other => other,
        };
        stmt_set.execute(params![k, v])?;
    }

    // Post-import plugin hooks: denormalization/promotion over the
    // committed data (adapter-declared; e.g. a summary table synced from
    // post_meta).

    // nav_menu_locations → theme_locations (per-lang). Child themes store
    // theme_mods under BOTH `theme_mods_{stylesheet}` and
    // `theme_mods_{parent}` — under Polylang each carries that language's
    // binding (aucoeur: parent map main_menu→156 FR, child map
    // main_menu→579 ES), so MERGE all keys: later maps only overwrite
    // (location, lang) pairs they rebind. menu_id = nav_menu term wp_id
    // (menus.id mirrors it); 0 = "no menu" (skip — a NULL row ≠ no row).
    {
        let template = options.get("template").cloned().unwrap_or_default();
        let stylesheet = options.get("stylesheet").cloned().unwrap_or_default();
        let mut keys: Vec<String> = [
            format!("theme_mods_{template}"),
            format!("theme_mods_{stylesheet}"),
        ]
        .into_iter()
        .filter(|k| options.contains_key(k))
        .collect();
        keys.dedup();
        let mut st_loc = tx.prepare(
            "INSERT OR REPLACE INTO theme_locations(location, lang, menu_id) \
             SELECT ?1, COALESCE((SELECT lang FROM menus WHERE id=?2),''), ?2 \
             WHERE EXISTS (SELECT 1 FROM menus WHERE id=?2)",
        )?;
        let mut loc_names: Vec<String> = Vec::new();
        for key in keys {
            let Some(mods_raw) = options.get(&key) else { continue };
            let Ok(serde_json::Value::Object(mods)) =
                crate::php_serialize::unserialize(mods_raw.as_bytes())
            else {
                continue;
            };
            let Some(serde_json::Value::Object(locs)) = mods.get("nav_menu_locations") else {
                continue;
            };
            for (loc, mid) in locs {
                let Some(mid) = mid.as_i64().filter(|id| *id > 0) else { continue };
                st_loc.execute(params![loc, mid])?;
                loc_names.push(loc.clone());
            }
        }
        if !loc_names.is_empty() {
            loc_names.sort();
            loc_names.dedup();
            stmt_set.execute(params![
                "theme_locations_available",
                serde_json::to_string(&loc_names)?
            ])?;
        }
    }
    /* Site-chrome seeds: the base templates read `site.*` ctx vars whose
       data lives in theme option payloads. Pull the few theme-general
       fields chrome needs into namespaced theme settings + verbatim
       settings rows for the plugin payloads (ssb, chaty) rendered
       raw by site_data. */
    {
        let mut st = tx.prepare("INSERT OR REPLACE INTO settings(key, value) VALUES(?1, ?2)")?;
        // traveltour_general → `plugin:theme-traveltour:general` flat JSON + a
        // few flat settings keys the chrome reads directly.
        if let Some(raw) = options.get("traveltour_general") {
            if let Ok(serde_json::Value::Object(m)) =
                crate::php_serialize::unserialize(raw.as_bytes())
            {
                let mut flat = serde_json::Map::new();
                for (k, v) in &m {
                    if let Some(s) = v.as_str() {
                        flat.insert(k.clone(), serde_json::Value::String(s.to_string()));
                    }
                }
                st.execute(params![
                    "plugin:theme-traveltour:general",
                    serde_json::to_string(&serde_json::Value::Object(flat))?
                ])?;
                for (opt_key, set_key) in [
                    ("top-bar-left-text", "topbar_left_html"),
                    ("top-bar-right-text", "topbar_right_html"),
                    ("copyright-left", "copyright_left_html"),
                    ("copyright-right", "copyright_right_html"),
                ] {
                    if let Some(v) = m.get(opt_key).and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                        st.execute(params![set_key, v])?;
                    }
                }
            }
        }
        /* auhfc_settings_sitewide → `site_head_html`: arbitrary sitewide
           head markup (analytics, preconnect, custom meta) applied on every
           page. Same verbatim rule as widget HTML. */
        if let Some(raw) = options.get("auhfc_settings_sitewide") {
            if let Ok(v) = crate::php_serialize::unserialize(raw.as_bytes()) {
                if let Some(head) = v.get("head").and_then(|x| x.as_str()) {
                    st.execute(params!["site_head_html", head])?;
                }
                if let Some(body) = v.get("body").and_then(|x| x.as_str()) {
                    st.execute(params!["site_body_html", body])?;
                }
                if let Some(footer) = v.get("footer").and_then(|x| x.as_str()) {
                    st.execute(params!["site_footer_html", footer])?;
                }
            }
        }
        // Contact-form pages (is_contact): every imported CF7 form slug,
        // grouped per lang — chrome marks `is_contact` on those slugs.
        {
            let mut fq = tx.prepare("SELECT slug, lang FROM forms")?;
            let map: std::collections::BTreeMap<String, Vec<String>> = fq
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
                .flatten()
                .fold(std::collections::BTreeMap::new(), |mut acc, (slug, lang)| {
                    acc.entry(lang).or_default().push(slug);
                    acc
                });
            if !map.is_empty() {
                st.execute(params![
                    "contact_slugs",
                    serde_json::to_string(&map)?
                ])?;
                /* intake reads a flat slug list. */
                st.execute(params![
                    "form_slugs",
                    serde_json::to_string(
                        &map.values().flatten().cloned().collect::<Vec<_>>()
                    )?
                ])?;
            }
        }
    }

    drop((
        stmt_post, stmt_meta, stmt_media, stmt_pt, stmt_comment, stmt_ra, stmt_redir, stmt_menu,
        stmt_mi, stmt_trans, stmt_trans_member, stmt_term, stmt_ttrans, stmt_set, stmt_h,
    ));
    tx.commit()?;

    for a in crate::admin::plugins::adapters() {
        if let Some(h) = a.import_hooks() {
            (h.after_import)(&conn)?;
        }
    }

    // Theme seed: `taxonomy_map` is a Data-backend setting the theme
    // adapter owns — migrate() is idempotent (`INSERT OR IGNORE`), so a
    // previously seeded or imported row wins.
    crate::admin::plugins::PluginAdapter::migrate(
        &crate::admin::plugins::theme_traveltour::ADAPTER,
        db_path,
    )?;

    if dropped_meta > 0 {
        rep.log(format!(
            "post_meta: {dropped_meta} value(s) dropped — the (post_id,meta_key) \
             unique index collapses WP multi-value metas to one row \
             (readers all take LIMIT 1). See CMS-OPERATIONS.md."
        ));
    }
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

/// Resolvable file path for an attachment: `_wp_attached_file`, falling
/// back to the serialized attachment metadata's `file` key.
fn attachment_path(p: &Post) -> String {
    let path = meta1(p, "_wp_attached_file").unwrap_or_default();
    if !path.is_empty() {
        return path;
    }
    meta1(p, "_wp_attachment_metadata")
        .and_then(|raw| unserialize(raw.as_bytes()).ok())
        .and_then(|v| v.get("file").and_then(|x| x.as_str()).map(|s| s.to_string()))
        .unwrap_or_default()
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
    if let Ok(v) = unserialize(meta_raw.as_bytes()) {
        w = v.get("width").and_then(|x| x.as_i64());
        h = v.get("height").and_then(|x| x.as_i64());
        if let Some(Value::Object(m)) = v.get("sizes").cloned() {
            sizes = Value::Object(m);
        }
    }
    let path = attachment_path(p);
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
