//! Import pass: parse dump → in-memory WP model → write rpress.db + report.

use crate::php_serialize::unserialize;
#[allow(unused_imports)]
use crate::sql;
use crate::sqldump::rows as dump_rows;
use anyhow::{Context, Result};
use rusqlite::params;
use serde_json::{json, Map, Value};
use std::collections::{BTreeSet, HashMap, HashSet};
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
    "wp_users",
    "wp_rank_math_redirections",
    "wp_glsr_ratings",
    "wp_snippets",
    "wp_glsr_assigned_posts",
];

/// Column `n` of an INSERT row, with table+row context on short rows —
/// a malformed row bails cleanly instead of panicking on `v[n]`.
fn need<'a>(v: &'a [crate::sqldump::Val], n: usize, table: &str) -> Result<&'a crate::sqldump::Val> {
    v.get(n).with_context(|| {
        format!("{table}: row too short — {n}-th column missing ({} cols)", v.len())
    })
}

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

/// `force=false` refuses to overwrite a database that already has rows —
/// re-running `import` used to silently delete a live site. `force=true`
/// is the `site build` path (fresh workdir semantics).
pub fn run(dump: &Path, db_path: &Path, report_path: &Path, force: bool) -> Result<()> {
    /* Import stages into `<db>.import` and renames over `db_path` at the
       very end. A previous killed run may leave the stale file (with its
       WAL trio) — remove it on entry so the staging check starts clean. */
    let stage = db_path.with_file_name(format!(
        "{}.import",
        db_path.file_name().unwrap_or_default().to_string_lossy()
    ));
    let _ = std::fs::remove_file(&stage);
    let _ = std::fs::remove_file(format!("{}-wal", stage.display()));
    let _ = std::fs::remove_file(format!("{}-shm", stage.display()));

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
    // Code Snippets rows (id, code) — enabled wp_footer/wp_head snippets
    // carry literal HTML the theme never stored (floating-button rail).
    let mut snippets: Vec<(i64, String)> = Vec::new();
    // term_id -> parent term_id from wp_term_taxonomy.parent (terms.parent_id
    // stays NULL — the taxonomy_map is the category hierarchy of record;
    // widget include-mode needs the real WP edges).
    let mut term_parents: HashMap<i64, i64> = HashMap::new();

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
            Some("termmeta") => "wp_termmeta",
            Some("options") => "wp_options",
            Some("users") => "wp_users",
            Some("rank_math_redirections") => "wp_rank_math_redirections",
            Some("glsr_ratings") => "wp_glsr_ratings",
            Some("glsr_assigned_posts") => "wp_glsr_assigned_posts",
            Some("snippets") => "wp_snippets",
            _ => table,
        };
        match table {
            "wp_users" => {
                // id -> login (display_name is col 8 but login is what WP
                // renders in bylines); feeds post_meta 'author_login'.
                if let Some(uid) = need(&v, 0, table)?.as_i64() {
                    let login = need(&v, 1, table)?.to_string_lossy().unwrap_or_default();
                    let dname = need(&v, 8, table)?.to_string_lossy().unwrap_or_default();
                    users.insert(uid, if dname.is_empty() { login } else { dname });
                }
            }
            "wp_terms" => {
                if let (Some(id), Some(name), Some(slug)) = (
                    need(&v, 0, table)?.as_i64(),
                    need(&v, 1, table)?.to_string_lossy(),
                    need(&v, 2, table)?.to_string_lossy(),
                ) {
                    terms.insert(id, (name, slug));
                }
            }
            "wp_term_taxonomy" => {
                let tt_id = need(&v, 0, table)?.as_i64().unwrap_or(0);
                let term_id = need(&v, 1, table)?.as_i64().unwrap_or(0);
                let tax = need(&v, 2, table)?.to_string_lossy().unwrap_or_default();
                let desc = need(&v, 3, table)?.to_string_lossy().unwrap_or_default();
                if let Some(parent) = need(&v, 4, table)?.as_i64().filter(|p| *p > 0) {
                    term_parents.insert(term_id, parent);
                }
                ttax.insert(tt_id, (term_id, tax, desc));
            }
            "wp_snippets" => {
                // (id, name, description, code, tags, scope, priority,
                // active, …) — active rows only; PHP blocks are filtered
                // to their literal HTML at settings write time.
                if let (Some(id), Some(code)) = (need(&v, 0, table)?.as_i64(), need(&v, 3, table)?.to_string_lossy()) {
                    if need(&v, 7, table)?.as_str() == Some("1") || need(&v, 7, table)?.as_i64() == Some(1) {
                        snippets.push((id, code));
                    }
                }
            }
            "wp_term_relationships" => {
                if let (Some(o), Some(t)) = (need(&v, 0, table)?.as_i64(), need(&v, 1, table)?.as_i64()) {
                    rels.push((o, t));
                }
            }
            "wp_postmeta" => {
                if let (Some(pid), Some(k), Some(mv)) = (
                    need(&v, 1, table)?.as_i64(),
                    need(&v, 2, table)?.to_string_lossy(),
                    need(&v, 3, table)?.to_string_lossy(),
                )
                {
                    post_meta.push((pid, k, mv));
                }
            }
            "wp_comments" => {
                let approved = need(&v, 10, table)?.as_str().unwrap_or("");
                if !matches!(approved, "1" | "0") {
                    return Ok(()); // spam/trash skipped
                }
                comments.push((
                    need(&v, 0, table)?.as_i64().unwrap_or(0),
                    need(&v, 1, table)?.as_i64().unwrap_or(0),
                    need(&v, 13, table)?.as_i64().unwrap_or(0),
                    need(&v, 2, table)?.to_string_lossy().unwrap_or_default(),
                    need(&v, 3, table)?.to_string_lossy().unwrap_or_default(),
                    need(&v, 4, table)?.to_string_lossy().unwrap_or_default(),
                    need(&v, 5, table)?.to_string_lossy().unwrap_or_default(),
                    need(&v, 8, table)?.to_string_lossy().unwrap_or_default(),
                    need(&v, 6, table)?.to_string_lossy().unwrap_or_default(),
                    approved.to_string(),
                ));
            }
            "wp_termmeta" => {
                /* Polylang string translations live on the synthetic
                   `pll_language` terms' termmeta — capture raw values;
                   harvested after stmt_set into pll_string_map. */
                if let (Some(tid), Some(k), Some(mv)) = (
                    need(&v, 1, table)?.as_i64(),
                    need(&v, 2, table)?.to_string_lossy(),
                    need(&v, 3, table)?.to_string_lossy(),
                ) {
                    if k == "_pll_strings_translations" {
                        pll_metas.push((tid, mv));
                    }
                }
            }
            "wp_commentmeta" => {
                if let (Some(cid), Some(k), Some(mv)) = (
                    need(&v, 1, table)?.as_i64(),
                    need(&v, 2, table)?.to_string_lossy(),
                    need(&v, 3, table)?.to_string_lossy(),
                )
                {
                    comment_meta.push((cid, k, mv));
                }
            }
            "wp_options" => {
                if let (Some(k), Some(mv)) = (
                    need(&v, 1, table)?.to_string_lossy(),
                    need(&v, 2, table)?.to_string_lossy(),
                )
                {
                    // Carry every option the runtime reads so a migrated site
                    // keeps its source site's moderation/media/feed config
                    // instead of falling back to built-in defaults. Plugin
                    // options whitelist by the adapter's declared prefixes
                    // (`ImportHooks.option_prefixes`).
                    let plugin_opt = crate::plugins::adapters()
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
                            | "sidebars_widgets"
                    ) || k.starts_with("theme_mods_")
                        || k.starts_with("chaty_")
                        || k.starts_with("cht_")
                        /* Plugin option prefixes kept verbatim — the admin
                           settings screens read/write these keys. */
                        || k.starts_with("wpdiscuz_")
                        || k.starts_with("glsr_")
                        || k.starts_with("cf7sr_")
                        || k.starts_with("ml-slider_")
                        /* Widget instances: `sidebars_widgets` maps each WP
                           sidebar to its ordered widget ids; `widget_{id_base}`
                           holds the instance payloads those ids index into. */
                        || k.starts_with("widget_") {
                        options.insert(k, mv);
                    }
                }
            }
            "wp_rank_math_redirections" => {
                if need(&v, 5, table)?.as_str() == Some("active") {
                    if let (Some(src), Some(dst)) = (
                        need(&v, 1, table)?.to_string_lossy(),
                        need(&v, 2, table)?.to_string_lossy(),
                    ) {
                        redirs.push((src, dst));
                    }
                }
            }
            "wp_glsr_ratings" => {
                if let (Some(rid), Some(rev)) = (need(&v, 0, table)?.as_i64(), need(&v, 1, table)?.as_i64()) {
                    ratings.insert(rid, rev);
                }
            }
            "wp_glsr_assigned_posts" => {
                if let (Some(rid), Some(pid)) = (need(&v, 0, table)?.as_i64(), need(&v, 1, table)?.as_i64()) {
                    assigned.push((rid, pid));
                }
            }
            "wp_posts" => {
                let pt = need(&v, 20, table)?.to_string_lossy().unwrap_or_default();
                let st = need(&v, 7, table)?.to_string_lossy().unwrap_or_default();
                posts.push(Post {
                    id: need(&v, 0, table)?.as_i64().unwrap_or(0),
                    author: need(&v, 1, table)?.as_i64().unwrap_or(0),
                    post_type: pt,
                    status: st,
                    slug: need(&v, 11, table)?.to_string_lossy().unwrap_or_default(),
                    title: need(&v, 5, table)?.to_string_lossy().unwrap_or_default(),
                    content: need(&v, 4, table)?.to_string_lossy().unwrap_or_default(),
                    excerpt: need(&v, 6, table)?.to_string_lossy().unwrap_or_default(),
                    parent: need(&v, 17, table)?.as_i64().unwrap_or(0),
                    menu_order: need(&v, 19, table)?.as_i64().unwrap_or(0),
                    date: need(&v, 2, table)?.to_string_lossy().unwrap_or_default(),
                    modified: need(&v, 14, table)?.to_string_lossy().unwrap_or_default(),
                    mime: need(&v, 21, table)?.to_string_lossy().unwrap_or_default(),
                    meta: HashMap::new(),
                });
            }
            _ => {}
        }
        Ok(())
    })?;

    /* Every INSERT'd table the dump carries that import has no arm for —
       logged once so a future missing canonicalize arm (or a plugin table
       we should learn) is loud instead of a silent drop. */
    if let Ok(dump_tables) = crate::sqldump::insert_tables(dump) {
        let wanted: std::collections::HashSet<String> =
            phys.iter().map(|s| s.to_string()).collect();
        let mut unhandled: Vec<String> = dump_tables
            .difference(&wanted)
            .cloned()
            .collect();
        unhandled.sort();
        for t in &unhandled {
            rep.log(format!("WARN no import arm for `{t}` — rows skipped"));
            eprintln!("import: no import arm for `{t}` — rows skipped");
        }
    }

    // Some dumps contain duplicate wp_posts ids (stale rows after a failed
    // migration on the source). Keep the first occurrence — an INSERT
    // collision on posts.id would abort the whole import otherwise.
    let mut seen_ids = std::collections::HashSet::new();
    let before = posts.len();
    posts.retain(|p| seen_ids.insert(p.id));
    if posts.len() != before {
        rep.log(format!("skipped {} duplicate wp_posts id(s)", before - posts.len()));
    }

    /* Same stale-duplicate guard for comments: one repeated comment id
       used to roll back the whole import on the comments wp_id UNIQUE.
       Nav-menu items and media inherit the posts dedupe (their wp_id IS
       the post id); menus dedupe by term id at insert time below. */
    let mut seen_comments = std::collections::HashSet::new();
    let before = comments.len();
    comments.retain(|c| seen_comments.insert(c.0));
    if comments.len() != before {
        rep.log(format!("skipped {} duplicate wp_comments id(s)", before - comments.len()));
    }

    // fold postmeta into posts
    let idx: HashMap<i64, usize> = posts.iter().enumerate().map(|(i, p)| (p.id, i)).collect();
    for (pid, k, mv) in post_meta {
        if let Some(&i) = idx.get(&pid) {
            posts[i].meta.entry(k).or_default().push(mv);
        }
    }

    // ---- Pass 2: map → SQLite ----------------------------------------------
    // Guard: a second `import` over a live db deletes users/sessions/
    // submissions along with the imported rows — require `--force` and say
    // what will be lost. An empty-but-existing file is fine to overwrite.
    if db_path.is_file() && !force {
        if let Ok(existing) = rusqlite::Connection::open(db_path) {
            let has_rows = [
                "SELECT COUNT(*) FROM posts",
                "SELECT COUNT(*) FROM users",
                "SELECT COUNT(*) FROM sessions",
                "SELECT COUNT(*) FROM submissions",
            ]
            .iter()
            .any(|q| existing.query_row(q, [], |r| r.get::<_, i64>(0)).unwrap_or(0) > 0);
            anyhow::ensure!(
                !has_rows,
                "{} already contains content — re-importing deletes users, sessions and form submissions. Pass --force to overwrite.",
                db_path.display()
            );
        }
    }
    // ---- Pass 2: map → SQLite (staged) ------------------------------------
    // All writes land in `<db>.import`; `db_path` is only replaced by the
    // atomic `rename` at the end — a killed or panicked import leaves the
    // prior DB byte-for-byte intact.
    let conn = crate::db::conn(&stage).context("open staged rpress.db")?;
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

    // post_translations: description = {lang: object_id} → generic
    // post_translation_members only (the legacy fr/es pair table retired;
    // members is the single N-language model — phase 4).
    let mut trans_groups: Vec<(String, Vec<(String, i64)>)> = Vec::new();
    let mut term_trans_pairs: Vec<(i64, i64)> = Vec::new();
    for (_tt, (term_id, tax, desc)) in &ttax {
        if tax == "post_translations" && desc.starts_with("a:") {
            match unserialize(desc.as_bytes()) {
                Ok(Value::Object(m)) => {
                    let members: Vec<(String, i64)> = m
                        .iter()
                        .filter_map(|(lang, v)| v.as_i64().map(|id| (lang.clone(), id)))
                        .collect();
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
        for a in crate::plugins::adapters() {
            if let Some(kind) = a.import_hooks().and_then(|h| (h.claim_type)(t)) {
                return Some(kind);
            }
        }
        match t {
            "post" => Some("post"),
            "page" => Some("page"),
            "site-review" => Some("site-review"),
            // MetaSlider: slider CPT + its slide children (linked via the
            // `ml-slider` taxonomy, term slug = slider post id). No routes
            // — generate only selects post/page/tour; [metaslider] reads
            // these rows from the DB at expand time.
            "ml-slider" => Some("ml-slider"),
            "ml-slide" => Some("ml-slide"),
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
        "CREATE TABLE IF NOT EXISTS harvest_posts(post_id INTEGER PRIMARY KEY);
         CREATE TABLE IF NOT EXISTS harvested_pages(
             post_id INTEGER PRIMARY KEY,
             html TEXT NOT NULL,
             fetched_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS harvested_fragments(
             post_id INTEGER PRIMARY KEY,
             html TEXT NOT NULL,
             fetched_at TEXT NOT NULL
         )",
    )?;
    let mut stmt_h = tx.prepare("INSERT OR IGNORE INTO harvest_posts(post_id) VALUES(?1)")?;

    // Spec `harvest = true` kinds: (post_id, lang, slug, origin_base)
    // collected during the posts pass, fetched AFTER tx.commit() —
    // never hold the write tx across HTTP.
    let mut harvest_targets: Vec<(i64, String, String, String)> = Vec::new();
    // Posts whose builder JSON carries a `revolution-slider` element:
    // (post_id, lang, slug) — same post-commit fetch, but only the
    // <rs-module-wrap> fragment is stored (the page itself renders
    // normally; the fragment fills the element's fallback slot).
    let mut slider_targets: Vec<(i64, String, String)> = Vec::new();

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
                } else if p.post_type == "wpcf7_contact_form" {
                    /* CF7 form definitions → `forms` (the Contact screen reads
                       this table; without it every site imports a dead
                       "No contact forms found." list). `_form` is CF7's
                       markup, `_mail`/`_mail_2`/`_messages` are PHP-serialized
                       option blobs, `_additional_settings` stays verbatim.
                       Slug: WP post_name is empty on CF7 — `form-{id}` keeps
                       the contact-form-7 shortcode id semantics. */
                    if p.status == "publish" {
                        let lang = post_lang.get(&p.id).cloned().unwrap_or_else(|| primary.clone());
                        let slug = if p.slug.trim().is_empty() {
                            format!("form-{}", p.id)
                        } else {
                            p.slug.clone()
                        };
                        let mail = meta1(p, "_mail").unwrap_or_default();
                        let mail2 = meta1(p, "_mail_2").unwrap_or_default();
                        let messages = meta1(p, "_messages").unwrap_or_default();
                        let msgs_json = if messages.is_empty() {
                            "{}".to_string()
                        } else {
                            crate::php_serialize::unserialize(messages.as_bytes())
                                .ok()
                                .and_then(|v| serde_json::to_string(&v).ok())
                                .unwrap_or_else(|| "{}".to_string())
                        };
                        let mail_str = |raw: &str, key: &str| -> String {
                            if raw.is_empty() { return String::new(); }
                            crate::php_serialize::unserialize(raw.as_bytes())
                                .ok()
                                .and_then(|v| v.get(key).and_then(|x| x.as_str()).map(String::from))
                                .unwrap_or_default()
                        };
                        let mail2_active: i64 = if mail2.is_empty() {
                            0
                        } else {
                            crate::php_serialize::unserialize(mail2.as_bytes())
                                .ok()
                                .and_then(|v| v.get("active").and_then(|x| x.as_bool().or(x.as_i64().map(|n| n != 0))).map(|b| b as i64))
                                .unwrap_or(0)
                        };
                        let _ = tx.execute(
                            "INSERT OR REPLACE INTO forms(slug, title, lang, kind, markup, \
                             mail_to, mail_from, mail_subject, mail_body, mail2_active, \
                             mail2_to, mail2_subject, mail2_body, messages, additional, \
                             created_at, updated_at) \
                             VALUES(?1,?2,?3,'form',?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,datetime('now'),datetime('now'))",
                            params![
                                slug, p.title, lang,
                                meta1(p, "_form").unwrap_or_else(|| p.content.clone()),
                                mail_str(&mail, "recipient"), mail_str(&mail, "sender"),
                                mail_str(&mail, "subject"), mail_str(&mail, "body"),
                                mail2_active,
                                mail_str(&mail2, "recipient"), mail_str(&mail2, "subject"), mail_str(&mail2, "body"),
                                msgs_json, meta1(p, "_additional_settings").unwrap_or_default()
                            ],
                        ).map_err(|e| rep.log(format!("cf7 import {} failed: {e}", p.id)));
                        // `[contact-form-7 id="…"]` carries the first 7
                        // chars of the form's `_hash` postmeta (CF7 ≥5.9
                        // switched shortcode ids off numeric post ids).
                        // Store hash7 → slug so render resolves it. Char-
                        // safe slice — a non-ASCII hash must not panic.
                        if let Some(h7) = meta1(p, "_hash").and_then(|h| h.get(..7).map(str::to_string)) {
                            let _ = stmt_set.execute(params![
                                format!("cf7hash:{h7}"), slug
                            ]);
                        }
                        // Legacy numeric-id shortcodes resolve on wp_id.
                        let _ = stmt_set.execute(params![
                            format!("cf7hash:{}", p.id), slug
                        ]);
                        rep.log(format!("cf7 form {} → {}", p.id, slug));
                    }
                    continue;
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
                if p.status == "publish" {
                    if let Some((_, base)) = crate::plugins::spec::spec_harvest_bases()
                        .iter()
                        .find(|(k, _)| *k == kind)
                    {
                        harvest_targets.push((p.id, lang.clone(), p.slug.clone(), base.to_string()));
                    }
                }
                if p.status == "publish" && body_json.contains("revolution-slider") {
                    slider_targets.push((p.id, lang.clone(), p.slug.clone()));
                }
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
                        // WP metas can repeat a key; the collapse is
                        // deterministic first-value-wins (dump order ==
                        // meta_id order == WP's scalar LIMIT-1 read and
                        // `dedupe_post_meta`'s MIN(rowid) keeper). Count
                        // the drops so the loss is visible in the report.
                        if stmt_meta.execute(params![p.id, k, mv])? == 0 {
                            dropped_meta += 1;
                        }
                    }
                }
                // Plugin per-row hooks run inside the import tx (T13).
                for a in crate::plugins::adapters() {
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

    // menus: nav_menu terms + nav_menu_item posts. Dedupe on term id —
    // duplicate nav_menu rows in the dump collide on menus.id/wp_term_id
    // the same way duplicate wp_posts ids did.
    let mut menu_wp: HashMap<i64, i64> = HashMap::new();
    let mut seen_menu_terms = std::collections::HashSet::new();
    for (tt_id, (term_id, tax, _)) in &ttax {
        if tax != "nav_menu" {
            continue;
        }
        if !seen_menu_terms.insert(*term_id) {
            rep.log(format!("skipped duplicate nav_menu term {}", term_id));
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
        /* Menu labels may embed HTML (traveltour mega-menu image items:
           `<img src="{origin}/wp-content/uploads/…">text`) — rewrite the
           site's own host to `{{ site_url }}` like widget bodies so menu
           images serve from the local asset space instead of hotlinking
           the origin. `url`/type fields stay verbatim — resolve_menu_url
           handles link rewrites separately. */
        let label = rewrite_own(
            &label,
            &options.get("siteurl").cloned().unwrap_or_default(),
        );
        let url = if type_ == "custom" {
            meta1(p, "_menu_item_url").unwrap_or_default()
        } else {
            String::new()
        };
        let object_id: Option<i64> = meta1(p, "_menu_item_object_id").and_then(|s| s.parse().ok());
        // `_menu_item_classes` is a serialized array of extra <li> classes
        // (e.g. traveltour-mega-menu) — WP joins them onto the synthetic
        // menu-item-* set; store verbatim for the renderer to merge.
        let classes: Vec<String> = p
            .meta
            .get("_menu_item_classes")
            .map(|vals| {
                vals.iter()
                    .filter_map(|v| unserialize(v.as_bytes()).ok())
                    .filter_map(|v| match v {
                        Value::Array(items) => Some(
                            items
                                .iter()
                                .filter_map(|x| x.as_str().map(str::to_string))
                                .collect::<Vec<_>>()
                                .join(" "),
                        ),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        // `gdlr_core_nav_menu_custom` (serialized assoc) carries the
        // traveltour mega-menu config (enable-mega-menu, section sizes) —
        // stored raw; the renderer parses the fields it needs.
        let mega = meta1(p, "gdlr_core_nav_menu_custom").unwrap_or_default();
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
            classes.join(" "),
            mega,
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

    /* WP widget instances → `widgets` rows. `sidebars_widgets` gives each WP
       sidebar's ordered instance ids; `widget_{id_base}` holds the instance
       payload (title/text/filter/…). Rows are built before `import_expect`
       so the verify key mirrors what actually landed. Site data wins over
       the theme/plugin seed rows: imported ids collide with seed ids on
       dumps that authored them (INSERT OR REPLACE below). */
    let site_langs: Vec<String> = languages.iter().cloned().collect();
    let widget_rows = build_widget_rows(
        &tx, &options, &site_langs, &primary, &term_parents, &mut rep)?;

    {
        let mut stmt_w = tx.prepare(
            "INSERT OR REPLACE INTO widgets(id, sidebar_id, type, title, settings_json, ord, lang) \
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;
        let mut stmt_sb = tx.prepare(
            "INSERT OR IGNORE INTO sidebars(id, name, lang) VALUES(?1, ?2, ?3)",
        )?;
        for w in &widget_rows {
            stmt_sb.execute(params![w.sidebar, w.sidebar, "all"])?;
            stmt_w.execute(params![
                w.id, w.sidebar, w.typ, w.title, w.settings, w.ord, w.lang
            ])?;
        }
    }

    /* Imported widget data is complete — stamp the seed-schema version so
       `widgets::ensure()` short-circuits its upgrade instead of grafting
       the theme's static seed rows onto real site widgets. Only when the
       dump declared its widget layout at all: without `sidebars_widgets`
       there is nothing to protect from seeding. */
    if options.contains_key("sidebars_widgets") {
        stmt_set.execute(params!["widgets_schema_v", crate::widgets::SEED_VERSION])?;
    }

    /* Pin the dump's origin for render-time rewriting (`rewrite_origin`,
       menu alias lists): import-only pipelines otherwise leave
       own-domain links pointing at the source host. OR IGNORE keeps a
       value configure already pinned. */
    if let Some(o) = options.get("siteurl").filter(|o| !o.is_empty()) {
        let mut stmt_oi = tx.prepare(
            "INSERT OR IGNORE INTO settings(key, value) VALUES(?1, ?2)",
        )?;
        stmt_oi.execute(params!["legacy_origin", o])?;
        stmt_oi.execute(params!["asset_origin", o])?;
    }

    /* Enabled Code Snippets carry literal HTML blocks the theme never
       stored (the aucoeur floating-button rail + its lang strip).
       Bucket by the WP hook the snippet registers so chrome emits them
       at the same page position. */
    {
        let (mut foot, mut head) = (String::new(), String::new());
        for (_id, code) in &snippets {
            let html = snippet_html(code, options.get("siteurl").map(String::as_str).unwrap_or(""));
            if html.is_empty() {
                continue;
            }
            if code.contains("wp_head") {
                head.push_str(&html);
            } else {
                foot.push_str(&html);
            }
        }
        if !foot.is_empty() {
            stmt_set.execute(params!["snippets_footer_html", foot])?;
        }
        if !head.is_empty() {
            stmt_set.execute(params!["snippets_head_html", head])?;
        }
    }

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
        expect.insert("widgets".into(), widget_rows.len() as i64);
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
        stmt_mi, stmt_trans_member, stmt_term, stmt_ttrans, stmt_set, stmt_h,
    ));
    tx.commit()?;

    for a in crate::plugins::adapters() {
        if let Some(h) = a.import_hooks() {
            (h.after_import)(&conn)?;
        }
    }

    // Spec harvest kinds: origin-rendered HTML fetched ONCE here —
    // port-once semantics mean the WP origin only has to be live during
    // import. Rows land in `harvested_pages`; generate emits them
    // verbatim (dead-origin regenerates keep working). Fetches run
    // post-commit so the write tx never spans HTTP.
    if let Some(origin) = options.get("siteurl").filter(|o| !o.is_empty()) {
        if !harvest_targets.is_empty() {
            let lang_cfg = crate::lang::config(&conn);
            let mut fetched: Vec<(i64, String)> = Vec::new();
            let mut failed = 0u64;
            for (id, lang, slug, base) in &harvest_targets {
                let lp = crate::lang::rel_prefix(&lang_cfg, lang);
                // Root-served kind (origin_base ""): origin path is
                // "{lp}/{slug}/" — no empty base segment.
                let path = match (lp.is_empty(), base.is_empty()) {
                    (true, true) => format!("{slug}/"),
                    (true, false) => format!("{base}/{slug}/"),
                    (false, true) => format!("{lp}/{slug}/"),
                    (false, false) => format!("{lp}/{base}/{slug}/"),
                };
                match crate::harvest::page(origin, &path, "{{ site_url }}") {
                    Ok(html) => fetched.push((*id, html)),
                    Err(e) => {
                        failed += 1;
                        rep.log(format!("[harvest] {path} failed: {e:#} — bare-body fallback"));
                    }
                }
            }
            // Harvested pages carry CF7 forms re-pointed at
            // `/form/enquiry` (harvest.rs) — seed that endpoint's form
            // row so intake validation + submission storage work. Rows
            // marked additional='harvest-seed' are ours: re-seeded on
            // every import (markup fixes propagate); rows without the
            // marker (admin-edited, imported CF7) are never touched.
            if !fetched.is_empty() {
                let primary = crate::lang::setting(&conn, "primary_lang");
                conn.execute(
                    "DELETE FROM forms WHERE slug='enquiry' AND additional='harvest-seed'",
                    [],
                )?;
                conn.execute(
                    "INSERT OR IGNORE INTO forms(slug, title, lang, kind, markup, \
                     mail_to, mail_from, mail_subject, mail_body, messages, additional, \
                     created_at, updated_at) \
                     VALUES('enquiry', 'Tour enquiry', ?1, 'form', \
                     '[text* your-name][email* your-email][text* your-phone]\
                      [text your-subject][textarea your-message][submit \"Send\"]', \
                     '', '', '', '', '{}', 'harvest-seed', datetime('now'), datetime('now'))",
                    params![if primary.is_empty() { "fr".to_string() } else { primary }],
                )?;
            }
            if !fetched.is_empty() {
                // Fresh connection: `conn` stays immutable (callers hold
                // &conn for after_import hooks just above).
                let mut conn2 = crate::db::conn(&stage)?;
                let tx2 = conn2.transaction()?;
                {
                    let mut ins = tx2.prepare(
                        "INSERT OR REPLACE INTO harvested_pages(post_id, html, fetched_at) \
                         VALUES(?1, ?2, datetime('now'))",
                    )?;
                    for (id, html) in &fetched {
                        ins.execute(params![id, html])?;
                    }
                }
                tx2.commit()?;
            }
            rep.log(format!(
                "harvested_pages: {} stored, {} failed",
                fetched.len(),
                failed
            ));
        }

        // Posts whose builder carries a revolution-slider element: fetch
        // the origin page once and keep only the <rs-module-wrap>
        // fragment (revslider slide data lives in plugin tables outside
        // the dump — rendered markup is the portable copy). The page
        // itself still renders via the builder; the fragment fills the
        // element's output at render time.
        if !slider_targets.is_empty() {
            let lang_cfg = crate::lang::config(&conn);
            let front: i64 = conn
                .query_row("SELECT CAST(value AS INTEGER) FROM settings WHERE key='page_on_front'", [], |r| r.get(0))
                .unwrap_or(0);
            let mut frags: Vec<(i64, String)> = Vec::new();
            for (id, lang, slug) in &slider_targets {
                let lp = crate::lang::rel_prefix(&lang_cfg, lang);
                let path = if *id == front {
                    if lp.is_empty() { String::new() } else { format!("{lp}/") }
                } else if lp.is_empty() {
                    format!("{slug}/")
                } else {
                    format!("{lp}/{slug}/")
                };
                match crate::harvest::revslider_fragment(origin, &path, "{{ site_url }}") {
                    Ok(Some(f)) => frags.push((*id, f)),
                    Ok(None) => rep.log(format!("[slider] {path} no rs-module-wrap — fallback stays")),
                    Err(e) => rep.log(format!("[slider] {path} failed: {e:#} — fallback stays")),
                }
            }
            if !frags.is_empty() {
                let mut conn3 = crate::db::conn(&stage)?;
                let tx3 = conn3.transaction()?;
                {
                    let mut ins = tx3.prepare(
                        "INSERT OR REPLACE INTO harvested_fragments(post_id, html, fetched_at) \
                         VALUES(?1, ?2, datetime('now'))",
                    )?;
                    for (id, html) in &frags {
                        ins.execute(params![id, html])?;
                    }
                }
                tx3.commit()?;
            }
            rep.log(format!("harvested_fragments: {} slider blocks stored", frags.len()));
        }
    }

    // Theme seed: `taxonomy_map` is a Data-backend setting the theme
    // adapter owns — migrate() is idempotent (`INSERT OR IGNORE`), so a
    // previously seeded or imported row wins.
    crate::plugins::PluginAdapter::migrate(
        &crate::plugins::theme_traveltour::ADAPTER,
        &stage,
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

    /* Close the staged DB (checkpoints its WAL), then publish it
       atomically: `rename` swaps `<db>.import` over `db_path` so readers
       and re-runs never see a half-imported database. */
    drop(conn);
    let _ = std::fs::remove_file(format!("{}-wal", stage.display()));
    let _ = std::fs::remove_file(format!("{}-shm", stage.display()));
    std::fs::rename(&stage, db_path)
        .with_context(|| format!("publish {} → {}", stage.display(), db_path.display()))?;

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
        p.title,
    ])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// WP widgets import — sidebars_widgets + widget_* options → widgets table
// ---------------------------------------------------------------------------

/// One widgets-table row produced from a WP widget instance. `lang` is
/// the emitted copy's language for lang-scoped regions (footers, the
/// synthesized sidebar-post/archive), the instance's own `pll_lang` (or
/// '' when untagged) for Fixed regions — '' renders on every language,
/// a pll tag only on its own.
struct WidgetRow {
    id: String,
    sidebar: String,
    typ: String,
    title: String,
    settings: String,
    ord: i64,
    lang: String,
}

/// Sidebar routing per WP sidebar id. `Lang` regions expand per site
/// language (`-{lang}` suffix — the renderer resolves `sidebar-post` /
/// `footer-N` per page language); `Fixed` keeps a literal region id.
#[derive(Clone, Copy)]
enum SidebarTarget {
    Lang(&'static str),
    Fixed(&'static str),
}

/// WP sidebar id → rpress sidebar routing. `Lang` regions expand per site
/// language (`-{lang}` suffix — the renderer resolves `footer-N` per page
/// language); `Fixed` keeps a literal region id. The blog-family sidebars
/// (`blog`, `blog-left`, `blog-right`, `gdlr-core-sidebar-preset`) import
/// verbatim under their own WP ids: the theme option (`traveltour_general`)
/// then picks which of them feed the `sidebar-post-*` / `sidebar-archive-*`
/// render regions — see `synthesize_theme_regions`. Everything not listed
/// is a dump-internal region (`wp_inactive_widgets`, `itsec_cron`,
/// mega-menu internals, `array_version`, …) and imports nothing.
fn sidebar_target(wp_sidebar: &str) -> Option<SidebarTarget> {
    match wp_sidebar {
        "footer-1" => Some(SidebarTarget::Lang("footer-1")),
        "footer-2" => Some(SidebarTarget::Lang("footer-2")),
        "footer-3" => Some(SidebarTarget::Lang("footer-3")),
        "footer-4" => Some(SidebarTarget::Lang("footer-4")),
        "blog" => Some(SidebarTarget::Fixed("blog")),
        "blog-left" => Some(SidebarTarget::Fixed("blog-left")),
        "blog-right" => Some(SidebarTarget::Fixed("blog-right")),
        "gdlr-core-sidebar-preset" => {
            Some(SidebarTarget::Fixed("gdlr-core-sidebar-preset"))
        }
        "single-tour" => Some(SidebarTarget::Fixed("sidebar-tour")),
        "tour-list" => Some(SidebarTarget::Fixed("tour-list")),
        "payment" => Some(SidebarTarget::Fixed("payment")),
        _ => None,
    }
}

/// Build every widgets-table row the dump's options describe. Instances in
/// Polylang-scoped regions (footers) land on their own `pll_lang` column —
/// an untagged instance clones to every site language (WP renders it
/// everywhere; clones take a `-{lang}` id suffix since `widgets.id` is the
/// primary key). `Fixed` regions (blog sidebars, single-tour, tour-list,
/// payment) import one verbatim row per instance under the WP sidebar id;
/// blog-family rows are then redistributed onto the theme-selected
/// `sidebar-{post,archive}-{lang}` render regions by
/// `synthesize_theme_regions`.
fn build_widget_rows(
    tx: &rusqlite::Transaction,
    options: &HashMap<String, String>,
    site_langs: &[String],
    primary: &str,
    term_parents: &HashMap<i64, i64>,
    rep: &mut Report,
) -> Result<Vec<WidgetRow>> {
    let mut out: Vec<WidgetRow> = Vec::new();
    let Some(raw) = options.get("sidebars_widgets") else {
        return Ok(out);
    };
    let sw = match unserialize(raw.as_bytes()) {
        Ok(Value::Object(m)) => m,
        Ok(_) => {
            rep.log("sidebars_widgets: not an assoc — no widgets imported".to_string());
            return Ok(out);
        }
        Err(e) => {
            rep.unser_errors += 1;
            rep.log(format!("sidebars_widgets unserialize: {e}"));
            return Ok(out);
        }
    };
    let origin = options.get("siteurl").cloned().unwrap_or_default();

    // `widget_{type}` options parse once per type — an id base can appear
    // in several sidebars (blog-left AND blog-right both list search-4…).
    let mut wopt: HashMap<String, Option<Value>> = HashMap::new();
    /* Per-target running ord: for Lang regions the cloned loop emits one
       sidebar's instances consecutively per lang, so the counter equals
       the WP array index for each accepted instance. Synthesized theme
       regions get their own counter below. */
    let mut ords: HashMap<String, i64> = HashMap::new();
    let mut skipped: Vec<String> = Vec::new();

    /* Blog-family instances are staged (verbatim WP sidebar → instance
       payloads) so the theme-selected regions can re-emit them per lang
       after the loop — their render sidebar is theme config, not the
       dump's own sidebar list. */
    let blog_ids = ["blog", "blog-left", "blog-right", "gdlr-core-sidebar-preset"];
    let mut staged: HashMap<
        String,
        Vec<(String, String, Map<String, Value>, String)>,
    > = HashMap::new();

    /* `widgets.id` is the table PK — every emit site draws from one set
       (`free_id` falls back `inst` → `inst-{lang}` → `inst-{lang}-{n}`),
       so the same wp id on two accepted sidebars never silently replaces
       an earlier row. Lang copies emit primary-first so the home lang
       keeps the bare wp id. */
    let mut used_ids: HashSet<String> = HashSet::new();
    let mut ordered: Vec<String> = site_langs.to_vec();
    ordered.sort_by_key(|l| *l != primary);

    for (wp_sidebar, list) in &sw {
        let Some(target) = sidebar_target(wp_sidebar) else {
            continue;
        };
        let Value::Array(ids) = list else {
            continue;
        };
        for idv in ids {
            let Some(inst_id) = idv.as_str() else {
                continue;
            };
            let Some((base, num)) = inst_id.rsplit_once('-') else {
                continue;
            };
            if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
                continue; // not a `{type}-{n}` instance id
            }
            let opt_key = format!("widget_{base}");
            if !wopt.contains_key(&opt_key) {
                let parsed = options.get(&opt_key).map(|raw| {
                    match unserialize(raw.as_bytes()) {
                        Ok(v) => v,
                        Err(e) => {
                            rep.unser_errors += 1;
                            rep.log(format!("{opt_key} unserialize: {e}"));
                            Value::Null
                        }
                    }
                });
                wopt.insert(opt_key.clone(), parsed);
            }
            let parsed = wopt.get(&opt_key).unwrap();
            let Some(Value::Object(instances)) = parsed.as_ref() else {
                skipped.push(inst_id.to_string());
                continue;
            };
            let Some(inst) = instances.get(num).and_then(|v| v.as_object()) else {
                skipped.push(inst_id.to_string());
                continue;
            };
            let inst_pll = inst
                .get("pll_lang")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Blog-family instance: stash for the theme-region synthesis;
            // its render sidebar comes from traveltour_general, not the
            // dump's own list.
            if blog_ids.contains(&wp_sidebar.as_str()) {
                staged
                    .entry(wp_sidebar.clone())
                    .or_default()
                    .push((inst_id.to_string(), base.to_string(), inst.clone(), inst_pll.to_string()));
                continue;
            }

            /* Langs this instance renders on: pll_lang pins it to one
               language; untagged instances render everywhere (primary
               first so home lang keeps the plain wp id). Fixed regions
               emit a single verbatim row resolved against the instance's
               own lang (or primary). `free_id` allocates the row's PK —
               the same wp id can land in two accepted sidebars. */
            let target_langs: Vec<&str> = match target {
                SidebarTarget::Fixed(_) => {
                    vec![if inst_pll.is_empty() { primary } else { inst_pll }]
                }
                SidebarTarget::Lang(_) if inst_pll.is_empty() => ordered
                    .iter()
                    .map(String::as_str)
                    .collect(),
                SidebarTarget::Lang(_) => vec![inst_pll],
            };
            for lang in target_langs {
                let sidebar = match target {
                    SidebarTarget::Lang(b) => format!("{b}-{lang}"),
                    SidebarTarget::Fixed(id) => id.to_string(),
                };
                let row_id = free_id(&mut used_ids, inst_id, lang);
                let Some((typ, settings)) = widget_settings(
                    tx, base, inst_id, inst, lang, primary, &origin, term_parents,
                ) else {
                    continue;
                };
                // Fixed rows carry the instance's own pll tag ('' untagged)
                // so mixed-lang WP sidebars render per page language; Lang
                // copies carry their emitted lang.
                let row_lang = match target {
                    SidebarTarget::Fixed(_) => inst_pll.to_string(),
                    SidebarTarget::Lang(_) => lang.to_string(),
                };
                let ord = ords.entry(sidebar.clone()).or_insert(0);
                out.push(WidgetRow {
                    id: row_id,
                    sidebar,
                    typ,
                    title: widget_title_text(inst),
                    settings,
                    ord: *ord,
                    lang: row_lang,
                });
                *ord += 1;
            }
        }
    }

    /* Traveltour assigns blog-family WP sidebars to the post / archive
       sidebar positions in `traveltour_general`. Redistribute the staged
       instances onto `sidebar-{post,archive}-{lang}` accordingly —
       verbatim `blog*` rows are only kept for sidebars no position
       selected. */
    synthesize_theme_regions(
        tx,
        &mut out,
        &staged,
        &mut used_ids,
        &ordered,
        options,
        primary,
        &origin,
        term_parents,
        rep,
    )?;
    if !skipped.is_empty() {
        rep.log(format!(
            "widgets: {} instance(s) without a saved payload skipped ({})",
            skipped.len(),
            skipped.join(", ")
        ));
    }
    rep.log(format!("widgets imported: {}", out.len()));
    Ok(out)
}

/// Traveltour assigns WP blog-family sidebars to the post / archive
/// positions in `traveltour_general`: `{pos}-sidebar` picks the
/// position(s) (right|left|both|none), `{pos}-sidebar-{left,right}` name
/// the WP sidebar feeding each slot (`pos` = "blog" for the post sidebar,
/// "archive-blog" for the archive sidebar). The chosen instances are
/// copied onto `sidebar-{region}-{lang}` per site language — pll_lang
/// tagged instances join only their own lang, untagged ones every lang.
/// `widgets.id` is the PK: the first copy keeps the wp id, later copies
/// take `-{lang}` (then `-{lang}-{n}`) suffixes — the same rule the pll
/// clones use. Sources feeding a position keep no verbatim `blog*` rows
/// (every selected instance moved); dumps whose theme config lacks the
/// keys entirely keep the verbatim rows and synthesize nothing.
fn synthesize_theme_regions(
    tx: &rusqlite::Transaction,
    out: &mut Vec<WidgetRow>,
    staged: &HashMap<String, Vec<(String, String, Map<String, Value>, String)>>,
    used_ids: &mut HashSet<String>,
    langs: &[String],
    options: &HashMap<String, String>,
    primary: &str,
    origin: &str,
    term_parents: &HashMap<i64, i64>,
    rep: &mut Report,
) -> Result<()> {
    let mut used: HashSet<String> = HashSet::new();
    for (region, pos) in [("sidebar-post", "blog"), ("sidebar-archive", "archive-blog")] {
        let Some(sources) = theme_sidebar_sources(options, pos, rep) else {
            continue;
        };
        used.extend(sources.iter().cloned());
        let mut ords: HashMap<String, i64> = HashMap::new();
        for lang in langs.iter().map(String::as_str) {
            for src in &sources {
                let Some(list) = staged.get(src) else {
                    continue;
                };
                for (inst_id, base, inst, inst_pll) in list {
                    if !inst_pll.is_empty() && inst_pll != lang {
                        continue;
                    }
                    let Some((typ, settings)) = widget_settings(
                        tx, base, inst_id, inst, lang, primary, origin, term_parents,
                    ) else {
                        continue;
                    };
                    let id = free_id(used_ids, inst_id, lang);
                    let sidebar = format!("{region}-{lang}");
                    let ord = ords.entry(sidebar.clone()).or_insert(0);
                    out.push(WidgetRow {
                        id,
                        sidebar,
                        typ,
                        title: widget_title_text(inst),
                        settings,
                        ord: *ord,
                        lang: lang.to_string(),
                    });
                    *ord += 1;
                }
            }
        }
    }
    /* Blog sidebars no position selected keep their rows verbatim under
       the WP sidebar id (untranslated — lang resolves against the
       instance's own pll_lang, else the site's primary). */
    for (src, list) in staged {
        if used.contains(src) {
            continue;
        }
        let mut ord = 0i64;
        for (inst_id, base, inst, inst_pll) in list {
            let lang = if inst_pll.is_empty() {
                primary
            } else {
                inst_pll.as_str()
            };
            let Some((typ, settings)) = widget_settings(
                tx, base, inst_id, inst, lang, primary, origin, term_parents,
            ) else {
                continue;
            };
            let id = free_id(used_ids, inst_id, lang);
            out.push(WidgetRow {
                id,
                sidebar: src.clone(),
                typ,
                title: widget_title_text(inst),
                settings,
                ord,
                lang: inst_pll.clone(),
            });
            ord += 1;
        }
    }
    Ok(())
}

/// traveltour_general → the WP sidebar ids one position selects. `None`
/// when the option is absent, unparseable, or never mentions this
/// position — the verbatim blog rows are then kept as imported.
/// A missing/empty position flag defaults to the theme's "right".
fn theme_sidebar_sources(
    options: &HashMap<String, String>,
    pos: &str,
    rep: &mut Report,
) -> Option<Vec<String>> {
    let raw = options.get("traveltour_general")?;
    let flat: HashMap<String, String> = match unserialize(raw.as_bytes()) {
        Ok(Value::Object(m)) => m
            .into_iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
            .collect(),
        _ => return None,
    };
    let left_k = format!("{pos}-sidebar-left");
    let right_k = format!("{pos}-sidebar-right");
    if !flat.contains_key(&left_k) && !flat.contains_key(&right_k) {
        return None;
    }
    let flag = flat
        .get(&format!("{pos}-sidebar"))
        .map(String::as_str)
        .unwrap_or("");
    let flag = if flag.is_empty() { "right" } else { flag };
    let mut srcs = Vec::new();
    if flag == "left" || flag == "both" {
        srcs.extend(flat.get(&left_k).cloned());
    }
    if flag == "right" || flag == "both" {
        srcs.extend(flat.get(&right_k).cloned());
    }
    if flag == "both" {
        rep.log(format!("widgets: {pos} position=both — left+right sidebars merged"));
    }
    Some(srcs.into_iter().filter(|s| !s.is_empty()).collect())
}

/// Allocate a widgets-table PK for a wp instance id on `lang`: bare id
/// when free, else `id-lang`, then `id-lang-{n}`. `used` tracks every row
/// emitted in this build — a wp id listed on two accepted sidebars (or
/// one instance copied onto post+archive regions) must never silently
/// REPLACE the earlier row.
fn free_id(used: &mut HashSet<String>, inst: &str, lang: &str) -> String {
    if !used.contains(inst) {
        used.insert(inst.to_string());
        return inst.to_string();
    }
    let mut n = 1i64;
    loop {
        let cand = if n == 1 {
            format!("{inst}-{lang}")
        } else {
            format!("{inst}-{lang}-{n}")
        };
        if !used.contains(&cand) {
            used.insert(cand.clone());
            return cand;
        }
        n += 1;
    }
}

/// Widget title: every WP widget uses `title` except wpcategorieswidget
/// which keys it `wcw_title`. Stored raw — renderers apply their own
/// title transform (frozen rows embed `widget_title_esc` output instead).
fn widget_title_text(inst: &Map<String, Value>) -> String {
    inst.get("title")
        .or_else(|| inst.get("wcw_title"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Instance → (rpress widget type, settings_json). `None` = drop the row
/// (a plugin widget with no importable payload renders nothing useful —
/// same rule WP applies to unconfigured instances).
fn widget_settings(
    tx: &rusqlite::Transaction,
    base: &str,
    inst_id: &str,
    inst: &Map<String, Value>,
    lang: &str,
    primary: &str,
    origin: &str,
    term_parents: &HashMap<i64, i64>,
) -> Option<(String, String)> {
    let gs = |k: &str| inst.get(k).and_then(|v| v.as_str()).unwrap_or("");
    let title = widget_title_text(inst);
    let body = {
        let t = gs("text");
        if t.is_empty() { gs("content") } else { t }
    };
    // Shared content pipeline: own-origin → {{ site_url }}, the gdlr
    // shortcode subset, then the generic expander for the rest.
    let render_body = |s: &str| -> String {
        let b = rewrite_own(s, origin);
        let b = expand_widget_shortcodes(&b);
        crate::render::shortcode::expand_shortcodes(&b, "{{ site_url }}", Some(&**tx), None)
    };

    match base {
        // Frozen-HTML types: the fragment is a minijinja template —
        // own-origin URLs become {{ site_url }} so deploys re-anchor.
        "text" => {
            if body.is_empty() {
                return None;
            }
            /* WP widget_text: the wpautop filter runs only when the
               instance's `filter` flag is set; do_shortcode always runs
               (after autop — `[gdlr_core_title]` lines wrap like any text
               line, matching the filter=1 captures). */
            let mut b = rewrite_own(body, origin);
            if truthy(inst.get("filter")) {
                b = widget_autop(&b);
            }
            b = expand_widget_shortcodes(&b);
            b = crate::render::shortcode::expand_shortcodes(&b, "{{ site_url }}", Some(&**tx), None);
            Some((
                "text".into(),
                json!({ "html": format!(
                    "<div id=\"{inst_id}\" class=\"widget widget_text traveltour-widget\">{tb}<div class=\"textwidget\">{b}</div></div>",
                    tb = traveltour_title_block(&title),
                )})
                .to_string(),
            ))
        }
        "custom_html" => {
            if body.is_empty() {
                return None;
            }
            Some((
                "custom_html".into(),
                json!({ "html": format!(
                    "<div id=\"{inst_id}\" class=\"widget_text widget widget_custom_html traveltour-widget\">{tb}<div class=\"textwidget custom-html-widget\">{b}</div></div>",
                    tb = traveltour_title_block(&title),
                    b = render_body(body),
                )})
                .to_string(),
            ))
        }
        "block" => {
            if body.is_empty() {
                return None;
            }
            let b = strip_block_comments(body);
            Some((
                "block".into(),
                json!({ "html": format!(
                    "<div id=\"{inst_id}\" class=\"widget widget_block traveltour-widget\">{tb}<div class=\"textwidget\">{b}</div></div>",
                    tb = traveltour_title_block(&title),
                    b = render_body(&b),
                )})
                .to_string(),
            ))
        }
        "search" => {
            /* get_search_form() markup; Polylang prefixes the form action
               for non-primary languages (the seed captures show `/es/`
               on the ES sidebar, bare on FR). */
            let (label, ph, submit) = match lang {
                "fr" => ("Rechercher\u{a0}:", "Rechercher…", "Rechercher"),
                "es" => ("Buscar:", "Buscar…", "Buscar"),
                _ => ("Search:", "Search …", "Search"),
            };
            let action = if lang == primary {
                "{{ site_url }}/".to_string()
            } else {
                format!("{{{{ site_url }}}}/{lang}/")
            };
            Some((
                "search".into(),
                json!({ "html": format!(
                    "<div id=\"{inst_id}\" class=\"widget widget_search traveltour-widget\">{tb}<form role=\"search\" method=\"get\" class=\"search-form\" action=\"{action}\">\n\t\t\t\t<label>\n\t\t\t\t\t<span class=\"screen-reader-text\">{label}</span>\n\t\t\t\t\t<input type=\"search\" class=\"search-field\" placeholder=\"{ph}\" value=\"\" name=\"s\" />\n\t\t\t\t</label>\n\t\t\t\t<input type=\"submit\" class=\"search-submit\" value=\"{submit}\" />\n\t\t\t</form></div>",
                    tb = traveltour_title_block(&title),
                )})
                .to_string(),
            ))
        }
        "media_image" => {
            let inner = media_image_html(tx, inst_id, inst, &title, origin)?;
            Some(("media_image".into(), json!({ "html": inner }).to_string()))
        }
        "wpcategorieswidget" => {
            let mut s = Map::new();
            s.insert(
                "taxonomy".into(),
                json!(match gs("wcw_taxonomy_type") {
                    "" => "category",
                    t => t,
                }),
            );
            s.insert("title".into(), json!(title));
            s.insert("show_counts".into(), json!(gs("wcw_hide_count") != "1"));
            /* include-mode freezes the selected term ids — resolve slugs
               against the just-imported terms table and rebuild the WP
               hierarchy (term_taxonomy.parent) restricted to the
               selection: `order` holds only selected roots (a selected
               child of a selected parent nests under it, WP
               hierarchical-list behaviour), `children` the real edges. */
            if gs("wcw_action_on_cat") == "include" {
                if let Some(Value::Array(sel)) = inst.get("wcw_selected_categories") {
                    let pairs: Vec<(i64, String)> = sel
                        .iter()
                        .filter_map(|v| {
                            v.as_i64()
                                .or_else(|| v.as_str().and_then(|x| x.parse().ok()))
                        })
                        .filter_map(|tid| {
                            tx.query_row(
                                "SELECT slug FROM terms WHERE id=?1",
                                [tid],
                                |r| Ok((tid, r.get::<_, String>(0)?)),
                            )
                            .ok()
                        })
                        .collect();
                    let sel_ids: HashSet<i64> = pairs.iter().map(|(id, _)| *id).collect();
                    if !sel_ids.is_empty() {
                        // child slug list per parent slug — children keep
                        // the widget's selected-order, only when both ends
                        // of the edge are selected.
                        let mut children: Map<String, Value> = Map::new();
                        for (tid, slug) in &pairs {
                            let kids: Vec<Value> = pairs
                                .iter()
                                .filter(|(cid, _)| {
                                    term_parents.get(cid) == Some(tid)
                                })
                                .map(|(_, s)| json!(s))
                                .collect();
                            children.insert(slug.clone(), Value::Array(kids));
                        }
                        // Selected child under a selected parent is not a
                        // root — it renders inside <ul class="children">.
                        let child_slugs: HashSet<&str> = pairs
                            .iter()
                            .filter(|(cid, _)| {
                                term_parents
                                    .get(cid)
                                    .is_some_and(|p| sel_ids.contains(p))
                            })
                            .map(|(_, s)| s.as_str())
                            .collect();
                        let roots: Vec<&str> = pairs
                            .iter()
                            .map(|(_, s)| s.as_str())
                            .filter(|s| !child_slugs.contains(s))
                            .collect();
                        s.insert("order".into(), json!(roots));
                        s.insert("children".into(), Value::Object(children));
                    }
                }
            }
            Some(("categories".into(), Value::Object(s).to_string()))
        }
        "tourmaster-tour-widget" => {
            /* Tourmaster "Recent Tours" — the tour_list live renderer
               reproduces its markup (outer custom_html wrapper + recent
               cards; num-fetch → count). */
            let mut s = Map::new();
            s.insert("mode".into(), json!("recent"));
            s.insert("count".into(), json!(gs("num-fetch")));
            s.insert("title".into(), json!(title));
            Some(("tour_list".into(), Value::Object(s).to_string()))
        }
        "tourmaster-widget-tour-category" => {
            /* Frozen type like its seed row: the WP capture was an empty
               div (the category list renders into the tour template, not
               the widget). */
            Some((
                "tour_category".into(),
                json!({ "html": format!(
                    "<div id=\"{inst_id}\" class=\"widget widget_tourmaster-widget-tour-category traveltour-widget\">{tb}</div>",
                    tb = traveltour_title_block(&title),
                )})
                .to_string(),
            ))
        }
        /* Core types with no live renderer/frozen seed shape: keep the
           scalar settings for admin visibility; they render empty until a
           renderer claims the type. */
        "tag_cloud" | "recent-posts" | "nav_menu" | "categories"
        | "recent-comments" | "archives" | "pages" | "meta" | "rss" => {
            Some((base.to_string(), widget_scalars(inst)))
        }
        _ => {
            /* Unknown plugin widget: keep whatever markup it authored
               (text/content) in the generic traveltour wrapper, else
               drop — nothing faithful to store. */
            if body.is_empty() {
                return None;
            }
            Some((
                base.to_string(),
                json!({ "html": format!(
                    "<div id=\"{inst_id}\" class=\"widget widget_{base} traveltour-widget\">{tb}<div class=\"textwidget\">{b}</div></div>",
                    tb = traveltour_title_block(&title),
                    b = render_body(body),
                )})
                .to_string(),
            ))
        }
    }
}

/// Code Snippets PHP → the literal HTML it echoes. Blocks appear between
/// `?>` … `<?php` (the standard function-body pattern); the last block
/// may close on `<?php\n}` — cut at the PHP marker either way.
/// Own-origin URLs become `{{ site_url }}` so deploys re-anchor.
fn snippet_html(code: &str, origin: &str) -> String {
    let mut out = String::new();
    for seg in code.split("?>").skip(1) {
        let html = match seg.find("<?php") {
            Some(i) => &seg[..i],
            None => seg,
        };
        let html = html.trim();
        if html.contains('<') {
            out.push_str(html);
            out.push('\n');
        }
    }
    rewrite_own(&out, origin)
}

/// Non-empty scalar fields of a widget instance minus WP internals —
/// the stored-settings shape for renderer-less core types.
fn widget_scalars(inst: &Map<String, Value>) -> String {
    let mut out = Map::new();
    for (k, v) in inst {
        if k.starts_with('_') {
            continue;
        }
        if matches!(v, Value::String(_) | Value::Number(_) | Value::Bool(_)) {
            out.insert(k.clone(), v.clone());
        }
    }
    Value::Object(out).to_string()
}

/// PHP empty() on a serialized flag: absent/false/""/"0"/0 → false.
fn truthy(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => !(s.is_empty() || s == "0"),
        Some(Value::Number(n)) => n.as_i64() != Some(0),
        _ => false,
    }
}

/// The traveltour `before_widget` title markup — emitted only when the
/// instance title is non-empty, with the capture's texturize+escape
/// (` - ` → ` &#8211; `, `'` → `&#039;`, `&<>` escaped).
fn traveltour_title_block(title: &str) -> String {
    if title.is_empty() {
        return String::new();
    }
    format!(
        "<h3 class=\"traveltour-widget-title\"><span class=\"traveltour-widget-head-text\">{}</span></h3><span class=\"clear\"></span>",
        widget_title_esc(title)
    )
}

/// WP `widget_title` filter chain reduced to what the captures show:
/// esc_html + the two texturize transforms the wpcategorieswidget
/// renderer already mirrors for term names.
fn widget_title_esc(raw: &str) -> String {
    crate::render_archive::esc(raw)
        .replace(" - ", " &#8211; ")
        .replace('\'', "&#039;")
}

/// The `filter=1` paragraph pass: blank lines become paragraph breaks —
/// double newline → `</p><p>`, single newlines kept (WP leaves them; the
/// authored `<br>`s carry the visual breaks).
fn widget_autop(s: &str) -> String {
    let t = s.replace("\r\n", "\n").replace('\r', "\n");
    let chunks: Vec<&str> = t
        .split("\n\n")
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .collect();
    format!("<p>{}</p>", chunks.join("</p><p>"))
}

/// Re-anchor the site's own absolute URLs on `{{ site_url }}` so stored
/// fragments stay deploy-neutral (seed fragments use the same token).
fn rewrite_own(s: &str, origin: &str) -> String {
    if origin.is_empty() {
        return s.to_string();
    }
    let o = origin.trim_end_matches('/');
    s.replace(&format!("{o}/"), "{{ site_url }}/")
        .replace(o, "{{ site_url }}")
}

/// `<!-- wp:… -->` comments are block-editor internals — strip them from
/// imported block-widget content.
fn strip_block_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find("<!--") {
        let Some(end) = rest[i..].find("-->").map(|e| i + e + 3) else {
            out.push_str(&rest[..i]);
            return out;
        };
        out.push_str(&rest[..i]);
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// media_image instance → the wp_get_attachment_image markup the seed
/// captures froze: sized `src` from the widget's own `url` field,
/// `srcset`/`sizes` rebuilt from media.sizes_json when the attachment
/// survived import.
fn media_image_html(
    tx: &rusqlite::Transaction,
    inst_id: &str,
    inst: &Map<String, Value>,
    title: &str,
    origin: &str,
) -> Option<String> {
    let gs = |k: &str| inst.get(k).and_then(|v| v.as_str()).unwrap_or("");
    let gi = |k: &str| {
        inst.get(k)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|x| x.parse().ok())))
    };
    let size = {
        let s = gs("size");
        if s.is_empty() { "medium" } else { s }
    };
    let att_id = gi("attachment_id").unwrap_or(-1);

    // Attachment row for srcset/alt fallback; the widget still renders
    // without it (the instance `url` alone feeds <img src>).
    let media: Option<(String, String, String)> = tx
        .query_row(
            "SELECT path, alt, sizes_json FROM media WHERE wp_id=?1",
            [att_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok();

    let url = {
        let u = gs("url");
        if !u.is_empty() {
            rewrite_own(u, origin)
        } else {
            media
                .as_ref()
                .map(|(path, _, _)| format!("{{{{ site_url }}}}/wp-content/uploads/{path}"))
                .unwrap_or_default()
        }
    };
    if url.is_empty() {
        return None;
    }
    let alt = {
        let a = gs("alt");
        if !a.is_empty() {
            a.to_string()
        } else {
            media.as_ref().map(|(_, a, _)| a.clone()).unwrap_or_default()
        }
    };

    // srcset: every registered intermediate size + the full image,
    // ordered by width (WP wp_calculate_image_srcset).
    let mut srcset = String::new();
    let mut disp = (gi("width").unwrap_or(0), gi("height").unwrap_or(0));
    if let Some((path, _, sizes_raw)) = &media {
        if let Ok(Value::Object(sizes)) = serde_json::from_str::<Value>(sizes_raw) {
            let dir = path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
            let mut cand: Vec<(i64, String)> = sizes
                .values()
                .filter_map(|meta| {
                    Some((
                        meta.get("width").and_then(|v| v.as_i64())?,
                        format!(
                            "{{{{ site_url }}}}/wp-content/uploads/{dir}/{}",
                            meta.get("file").and_then(|v| v.as_str())?
                        ),
                    ))
                })
                .collect();
            cand.push((
                disp.0,
                format!("{{{{ site_url }}}}/wp-content/uploads/{path}"),
            ));
            cand.sort_by_key(|(w, _)| *w);
            cand.dedup_by(|a, b| a.0 == b.0);
            srcset = cand
                .iter()
                .map(|(w, u)| format!("{u} {w}w"))
                .collect::<Vec<_>>()
                .join(", ");
            // The instance width/height are the FULL image dims; the
            // displayed variant's dims live in sizes_json.
            if let Some(meta) = sizes.get(size) {
                disp = (
                    meta.get("width").and_then(|v| v.as_i64()).unwrap_or(disp.0),
                    meta.get("height").and_then(|v| v.as_i64()).unwrap_or(disp.1),
                );
            }
        }
    }

    let e = crate::html_decode::esc_attr;
    let mut img = String::from("<img");
    if disp.0 > 0 {
        img.push_str(&format!(" width=\"{}\"", disp.0));
    }
    if disp.1 > 0 {
        img.push_str(&format!(" height=\"{}\"", disp.1));
    }
    img.push_str(&format!(
        " src=\"{}\" class=\"image wp-image-{att_id}  attachment-{size} size-{size}\" alt=\"{}\" style=\"max-width: 100%; height: auto;\" decoding=\"async\" loading=\"lazy\"",
        e(&url),
        e(&alt),
    ));
    if !srcset.is_empty() {
        let w = if disp.0 > 0 { disp.0 } else { 300 };
        img.push_str(&format!(
            " srcset=\"{srcset}\" sizes=\"auto, (max-width: {w}px) 100vw, {w}px\""
        ));
    }
    img.push_str(" />");

    // Optional link wrapper (wp media_image link_type: custom|file|post).
    let link = match gs("link_type") {
        "custom" => gs("link_url").to_string(),
        "file" => media
            .as_ref()
            .map(|(p, _, _)| format!("{{{{ site_url }}}}/wp-content/uploads/{p}"))
            .unwrap_or_default(),
        _ => String::new(),
    };
    let mut inner = img;
    if !link.is_empty() {
        let tgt = if truthy(inst.get("link_target_blank")) {
            " target=\"_blank\""
        } else {
            ""
        };
        let cls = {
            let c = gs("link_classes");
            if c.is_empty() {
                String::new()
            } else {
                format!(" class=\"{}\"", e(c))
            }
        };
        inner = format!(
            "<a href=\"{}\"{cls}{tgt}>{inner}</a>",
            e(&rewrite_own(&link, origin))
        );
    }

    Some(format!(
        "<div id=\"{inst_id}\" class=\"widget widget_media_image traveltour-widget\">{tb}{inner}</div>",
        tb = traveltour_title_block(title),
    ))
}

// ---------------------------------------------------------------------------
// gdlr shortcode subset for widget payloads — `[gdlr_core_space]`,
// `[gdlr_core_title]`, `[gdlr_widget_list]`, `[gdlr_widget_box]`,
// `[gdlr_core_social_network]`. Markup ports the plugin's PHP emitters
// (esc_style's `"key: value ;"` spacing included — the seed captures carry
// it verbatim). Everything else falls through to expand_shortcodes.
// ---------------------------------------------------------------------------

/// gdlr_core_esc_style: `"key: value ;"` per pair (empty/"0" values
/// skipped — PHP `empty()`), wrapped as ` style="…"  ` with the capture's
/// double space before `>`.
fn gdlr_style(pairs: &[(&str, &str)]) -> String {
    let mut css = String::new();
    for (k, v) in pairs {
        if v.is_empty() || *v == "0" {
            continue;
        }
        css.push_str(&format!("{k}: {v} ;"));
    }
    if css.is_empty() {
        String::new()
    } else {
        format!(" style=\"{css}\"  ")
    }
}

/// Shortcode attribute parser: `key="v" key2='v' k3=bare` — also tolerates
/// the `attr= "v"` spacing the dumps carry.
fn widget_sc_attrs(raw: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    let b = raw.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i < n {
        while i < n && b[i].is_ascii_whitespace() {
            i += 1;
        }
        let kstart = i;
        while i < n && !b[i].is_ascii_whitespace() && b[i] != b'=' {
            i += 1;
        }
        if kstart == i {
            i += 1;
            continue;
        }
        let key = &raw[kstart..i];
        while i < n && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < n && b[i] == b'=' {
            i += 1;
            while i < n && b[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < n && (b[i] == b'"' || b[i] == b'\'') {
                let q = b[i];
                i += 1;
                let vstart = i;
                while i < n && b[i] != q {
                    i += 1;
                }
                m.insert(key.to_string(), raw[vstart..i].to_string());
                i += 1; // closing quote
            } else {
                let vstart = i;
                while i < n && !b[i].is_ascii_whitespace() && b[i] != b']' {
                    i += 1;
                }
                m.insert(key.to_string(), raw[vstart..i].to_string());
            }
        }
    }
    m
}

/// Expand the gdlr/widget shortcodes found in widget payloads. Unknown
/// tags stay verbatim for the later `expand_shortcodes` pass. Enclosing
/// shortcodes (widget_list/widget_box) recurse into their content like
/// PHP's `do_shortcode($content)`.
fn expand_widget_shortcodes(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 128);
    let mut rest = s;
    let mut list_count: u64 = 0; // $gdlr_widget_list_count, per content run
    while let Some(i) = rest.find('[') {
        out.push_str(&rest[..i]);
        let Some(end) = rest[i..].find(']').map(|e| i + e) else {
            out.push_str(&rest[i..]);
            return out;
        };
        let head = &rest[i + 1..end];
        let tag = head.split_whitespace().next().unwrap_or("");
        let attrs = widget_sc_attrs(&head[tag.len()..]);
        let g = |k: &str| attrs.get(k).map(String::as_str).unwrap_or("");
        let after = &rest[end + 1..];
        match tag {
            "gdlr_core_space" => {
                let h = g("height");
                out.push_str(&format!(
                    "<span class=\"gdlr-core-space-shortcode\"{}></span>",
                    gdlr_style(&[("margin-top", if h.is_empty() { "30px" } else { h })])
                ));
            }
            "gdlr_core_title" => {
                out.push_str(&gdlr_title_html(&attrs));
            }
            "gdlr_widget_list" => {
                let id = format!("gdlr-core-widget-list-{list_count}");
                list_count += 1;
                out.push_str(&format!(
                    "<div class=\"gdlr-core-widget-list-shortcode\" id=\"{id}\"{}>",
                    gdlr_style(&[
                        ("color", g("color")),
                        ("background-color", g("background-color")),
                        ("border-color", g("frame-border-color")),
                        ("border-width", g("frame-border-width")),
                    ])
                ));
                if !g("title").is_empty() {
                    out.push_str(&format!(
                        "<h3 class=\"gdlr-core-widget-list-shortcode-title\"{}>{}</h3>",
                        gdlr_style(&[("color", g("title-color"))]),
                        g("title")
                    ));
                }
                if let Some(close) = after.find("[/gdlr_widget_list]") {
                    out.push_str(&expand_widget_shortcodes(&after[..close]));
                    out.push_str("</div>");
                    rest = &after[close + "[/gdlr_widget_list]".len()..];
                    continue;
                }
                out.push_str("</div>");
            }
            "gdlr_widget_box" => {
                out.push_str(&gdlr_widget_box_open(&attrs));
                if let Some(close) = after.find("[/gdlr_widget_box]") {
                    out.push_str(&expand_widget_shortcodes(&after[..close]));
                    out.push_str("</div>");
                    rest = &after[close + "[/gdlr_widget_box]".len()..];
                    continue;
                }
                out.push_str("</div>");
            }
            "gdlr_core_social_network" => {
                out.push_str(&gdlr_social_html(&attrs));
            }
            _ => {
                out.push_str(&rest[i..end + 1]);
            }
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

/// gdlr_core_pb_element_title::get_content, common-attrs port: the
/// title-wrap div + h3 + divider the footer captures carry. Caption and
/// side borders are omitted (unused in widget payloads).
fn gdlr_title_html(attrs: &HashMap<String, String>) -> String {
    let g = |k: &str| attrs.get(k).map(String::as_str).unwrap_or("");
    // `attr` != WP default → emit; == default → PHP empties the style attr.
    let non_default = |k: &str, default: &str| {
        let v = g(k);
        if v.is_empty() || v == default {
            ""
        } else {
            v
        }
    };
    let esc = crate::html_decode::esc_attr;
    let align = {
        let a = g("text-align");
        if a.is_empty() { "left" } else { a }
    };
    let caption_pos = {
        let c = g("caption-position");
        if c.is_empty() { "top" } else { c }
    };
    let mut class = format!(" gdlr-core-{align}-align gdlr-core-title-item-caption-{caption_pos}");
    if !g("class").is_empty() {
        class.push(' ');
        class.push_str(g("class"));
    }
    let mut out = format!(
        "<div class=\"gdlr-core-title-item gdlr-core-item-pdb clearfix {}\"{}>",
        esc(&class),
        gdlr_style(&[
            ("padding-bottom", g("padding-bottom")),
            ("margin-left", g("item-left-margin")),
        ])
    );
    if !g("caption").is_empty() && caption_pos == "top" {
        out.push_str(&format!(
            "<span class=\"gdlr-core-title-item-caption gdlr-core-info-font gdlr-core-skin-caption\"{}>{}</span>",
            gdlr_style(&[
                ("font-size", non_default("caption-font-size", "16px")),
                ("font-weight", non_default("caption-font-weight", "400")),
                ("color", g("caption-color")),
            ]),
            g("caption")
        ));
    }
    if !g("title").is_empty() {
        let tag = {
            let t = g("heading-tag");
            if t.is_empty() { "h3" } else { t }
        };
        out.push_str("<div class=\"gdlr-core-title-item-title-wrap \"  >");
        out.push_str(&format!(
            "<{tag} class=\"gdlr-core-title-item-title gdlr-core-skin-title \"{}>",
            gdlr_style(&[
                ("font-size", non_default("title-font-size", "41px")),
                ("font-weight", non_default("title-font-weight", "800")),
                ("font-style", non_default("title-font-style", "normal")),
                ("letter-spacing", non_default("title-font-letter-spacing", "1px")),
                ("line-height", g("title-line-height")),
                (
                    "text-transform",
                    if g("title-font-uppercase") == "enable" { "" } else { "none" },
                ),
                ("color", g("title-color")),
            ])
        ));
        out.push_str(g("title"));
        out.push_str(
            "<span class=\"gdlr-core-title-item-title-divider gdlr-core-skin-divider\" ></span>",
        );
        out.push_str(&format!("</{tag}></div>"));
    }
    out.push_str("</div>");
    out
}

/// gdlr_widget_box opening markup (esc_style rules — background >7 chars
/// → `background-image: url(…)`, else `background-color`). Content and the
/// `</div>` close are handled by the caller's enclosing-tag branch.
fn gdlr_widget_box_open(attrs: &HashMap<String, String>) -> String {
    let g = |k: &str| attrs.get(k).map(String::as_str).unwrap_or("");
    let mut css = String::new();
    for (k, v) in [
        ("font-size", g("font-size")),
        ("color", g("color")),
        ("padding", g("padding")),
    ] {
        if !v.is_empty() {
            css.push_str(&format!("{k}: {v} ;"));
        }
    }
    if !g("border-radius").is_empty() {
        css.push_str(&format!(
            "border-radius: {0};-moz-border-radius: {0};-webkit-border-radius: {0};",
            g("border-radius")
        ));
    }
    // box-shadow needs a color AND a size in esc_style's branch.
    if !g("shadow-color").is_empty() && !g("shadow-size").is_empty() {
        let op = {
            let o = g("shadow-opacity");
            if o.is_empty() { "0" } else { o }
        };
        let bgs = format!(
            "{} {} {} ",
            if g("shadow-x").is_empty() { "0" } else { g("shadow-x") },
            if g("shadow-y").is_empty() { "0" } else { g("shadow-y") },
            g("shadow-size")
        );
        css.push_str(&format!(
            "box-shadow: {bgs} rgba({0},{op}); -moz-box-shadow: {bgs} rgba({0},{op}); -webkit-box-shadow: {bgs} rgba({0},{op});",
            g("shadow-color")
        ));
    }
    let bg = g("background");
    if !bg.is_empty() {
        if bg.chars().count() > 7 {
            css.push_str(&format!("background-image: url({bg}) ;"));
        } else {
            css.push_str(&format!("background-color: {bg} ;"));
        }
    }
    if !g("border-color").is_empty() {
        let w = {
            let w = g("border-width");
            if w.is_empty() { "3px" } else { w }
        };
        css.push_str(&format!("border: {w} solid {} ;", g("border-color")));
    }
    let style = if css.is_empty() {
        String::new()
    } else {
        format!(" style=\"{css}\"  ")
    };
    let extra = if g("text-align").is_empty() {
        String::new()
    } else {
        format!(" gdlr-core-{}-align", g("text-align"))
    };
    let mut out = format!("<div class=\"gdlr-core-widget-box-shortcode {extra}\"{style}>");
    if !g("title").is_empty() {
        out.push_str(&format!(
            "<h3 class=\"gdlr-core-widget-box-shortcode-title\"{}>{}</h3>",
            gdlr_style(&[("color", g("title-color"))]),
            g("title")
        ));
    }
    if !g("left-icon").is_empty() {
        out.push_str(&format!(
            "<i class=\"gdlr-core-widget-box-shortcode-icon {}\"{}></i>",
            crate::html_decode::esc_attr(g("left-icon")),
            gdlr_style(&[("color", g("left-icon-color"))])
        ));
    }
    out.push_str("<div class=\"gdlr-core-widget-box-shortcode-content\" >");
    out
}

/// gdlr_core_social_network shortcode output: shortcode mode → `<span>`
/// wrapper with `display:block`, fixed-order icon list, `_blank` targets.
fn gdlr_social_html(attrs: &HashMap<String, String>) -> String {
    let g = |k: &str| attrs.get(k).map(String::as_str).unwrap_or("");
    const SOCIALS: &[(&str, &str)] = &[
        ("delicious", "fa fa-delicious"),
        ("email", "fa fa-envelope"),
        ("deviantart", "fa fa-deviantart"),
        ("digg", "fa fa-digg"),
        ("facebook", "fa fa-facebook"),
        ("flickr", "fa fa-flickr"),
        ("lastfm", "fa fa-lastfm"),
        ("linkedin", "fa fa-linkedin"),
        ("pinterest", "fa fa-pinterest"),
        ("rss", "fa fa-rss"),
        ("skype", "fa fa-skype"),
        ("stumbleupon", "fa fa-stumbleupon"),
        ("tumblr", "fa fa-tumblr"),
        ("twitter", "fa fa-twitter"),
        ("vimeo", "fa fa-vimeo"),
        ("youtube", "fa fa-youtube"),
        ("dribbble", "fa fa-dribbble"),
        ("behance", "fa fa-behance"),
        ("instagram", "fa fa-instagram"),
        ("snapchat", "fa fa-snapchat"),
    ];
    let align = {
        let a = g("text-align");
        if a.is_empty() { "left" } else { a }
    };
    let direction = {
        let d = g("direction");
        if d.is_empty() { "horizontal" } else { d }
    };
    let e = crate::html_decode::esc_attr;
    let mut out = format!(
        "<span class=\"gdlr-core-social-network-item gdlr-core-item-pdb  gdlr-core-{align}-align gdlr-direction-{direction}\"{}>",
        gdlr_style(&[
            ("padding-bottom", g("padding-bottom")),
            ("display", "block"),
        ])
    );
    let margin_dir = if direction == "horizontal" {
        "margin-right"
    } else {
        "margin-bottom"
    };
    let icon_space = {
        let s = g("icon-space");
        if s.is_empty() || s == "20px" { "" } else { s }
    };
    for (key, icon) in SOCIALS {
        let mut link = g(key).to_string();
        if link.is_empty() {
            continue;
        }
        if *key == "email" {
            link = format!("mailto:{link}");
        }
        let tgt = {
            let t = g("link-target");
            if t.is_empty() { "_blank" } else { t }
        };
        out.push_str(&format!(
            "<a href=\"{}\" target=\"{tgt}\" class=\"gdlr-core-social-network-icon\" title=\"{key}\"{}><i class=\"{icon}\" ></i></a>",
            e(&link),
            gdlr_style(&[
                ("font-size", g("icon-size")),
                ("color", g("icon-color")),
                ("border-color", g("icon-border-color")),
                ("border-width", g("icon-border-size")),
                (margin_dir, icon_space),
                ("width", g("icon-width")),
                ("height", g("icon-width")),
                ("line-height", g("icon-width")),
                ("background-color", g("background-color")),
                ("border-radius", g("icon-border-radius")),
            ])
        ));
    }
    out.push_str("</span>");
    out
}
