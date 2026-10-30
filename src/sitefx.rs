//! Site-level file side-effects that used to live inside `rpress generate`.
//! Under the render-first + lazy write-back model these are produced
//! on-demand (the first public hit renders, then writes back) or warmed
//! once at `serve()` boot — never by a full-site build.
//!
//! Everything here is cheap: sitemap/feed/robot/redirect writes are pure
//! SQL → string ops, and the post-file loop only computes output paths —
//! no render happens in this module.

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

/// Rebuild the `(loc, kind, lastmod)` vec `write_sitemap` consumes —
/// the same set `generate::run` collected while writing files, computed
/// from `posts`/`post_terms` without rendering anything.
fn collect_urls(conn: &Connection, site: &str, tour_base: &str) -> Result<Vec<(String, String, Option<String>)>> {
    let site = site.trim_end_matches('/');
    let mut urls: Vec<(String, String, Option<String>)> = Vec::new();

    let front: i64 = conn
        .query_row(crate::sql!("generate__select_cast_value"), [], |r| r.get(0))
        .unwrap_or(0);

    let lang_cfg = crate::lang::config(conn);
    let mut stmt = conn.prepare(crate::sql!("generate__select_id_slug_2"))?;
    let rows: Vec<(i64, String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .flatten()
        .collect();
    let mut mod_stmt = conn.prepare(crate::sql!("generate__select_modified_at"))?;
    for (id, slug, lang, kind) in &rows {
        // Adapter-owned kinds canonicalize their slug (display_slug).
        let wslug = crate::plugins::adapter_for_kind(kind)
            .and_then(|a| {
                let title: String = conn
                    .query_row(crate::sql!("generate__select_post_title"), [id], |r| r.get(0))
                    .unwrap_or_default();
                a.display_slug(conn, *id, slug, &title)
            })
            .unwrap_or_else(|| slug.clone());
        let rel = crate::generate::rel_path_with_cfg(&lang_cfg, *id, front, lang, &wslug, crate::plugins::kind_url_base(tour_base, kind).as_deref());
        let loc_path = rel.to_string_lossy().replace("index.html", "");
        let loc = format!(
            "{}{}",
            site,
            format!("/{loc_path}").replace("//", "/")
        );
        let lastmod: Option<String> = mod_stmt.query_row([*id], |r| r.get(0)).ok();
        urls.push((loc, kind.clone(), lastmod));
    }

    // Localized front pages for every non-primary language.
    for lang in lang_cfg.langs.iter().filter(|l| *l != &lang_cfg.primary) {
        let lastmod: Option<String> = conn
            .query_row(crate::sql!("generate__select_max_modified_page_lang"), [lang], |r| r.get(0))
            .ok();
        urls.push((format!("{}{}", site, crate::lang::page_path(&lang_cfg, lang, "page", "", None)), "page".into(), lastmod));
    }

    // Taxonomy archives — same set generate emitted, same shared mapping.
    {
        let mut tstmt = conn.prepare(crate::sql!("archive__select_term_paths"))?;
        let terms: Vec<(String, String)> = tstmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .flatten()
            .collect();
        let archive_lastmod: Option<String> = conn
            .query_row(
                crate::sql!("generate__select_max_modified_published"),
                [],
                |r| r.get(0),
            )
            .ok()
            .flatten();
        for (tax, tslug) in &terms {
            for (_lang, dir) in crate::generate::term_out_paths(conn, &lang_cfg, tax, tslug) {
                urls.push((
                    format!("{site}/{}/", dir.to_string_lossy()),
                    "archive".into(),
                    archive_lastmod.clone(),
                ));
            }
        }
    }

    Ok(urls)
}

/// Raw-slug → canonical display-slug redirect rows for adapter-owned kinds
/// with degenerate slugs (6416 'x'). Mirrors the raw_redirects collection
/// in `generate::run`.
fn collect_raw_redirects(conn: &Connection, site: &str, tour_base: &str) -> Result<Vec<(String, String)>> {
    let site = site.trim_end_matches('/');
    let lang_cfg = crate::lang::config(conn);
    let mut raw = Vec::new();
    // Plugin kinds with a display_slug hook: only those can diverge from
    // the stored slug.
    let owned: Vec<&'static str> = crate::plugins::kind_defs()
        .iter()
        .filter(|d| crate::plugins::adapter_for_kind(d.kind).is_some())
        .map(|d| d.kind)
        .collect();
    if owned.is_empty() {
        return Ok(raw);
    }
    let qs = owned.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let mut stmt = conn.prepare(&format!(
        "{}{qs})", crate::sql!("sitefx__select_published_kinds")
    ))?;
    let rows: Vec<(i64, String, String, String, String)> = stmt
        .query_map(rusqlite::params_from_iter(owned.iter()), |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .flatten()
        .collect();
    for (id, slug, lang, title, kind) in &rows {
        let display = crate::plugins::adapter_for_kind(kind)
            .and_then(|a| a.display_slug(conn, *id, slug, title))
            .unwrap_or_else(|| slug.clone());
        if &display != slug {
            let prefix = crate::lang::url_prefix(&lang_cfg, lang);
            let base = crate::plugins::kind_url_base(tour_base, kind).unwrap_or_default();
            raw.push((
                format!("{prefix}/{base}/{slug}/"),
                format!("{site}{prefix}/{base}/{display}/"),
            ));
        }
    }
    Ok(raw)
}

/// One pass over everything `rpress generate` used to emit besides post HTML.
/// Cheap enough to run at `serve()` boot: pure SQL + string writes, no
/// renders. Called once on startup so a cold cache still has sitemap,
/// feeds, robots, redirects and the search page from minute zero.
pub fn emit_site_files(db: &Path, templates: &Path, out: &Path, site: &str, tour_base: &str) -> Result<()> {
    let conn = crate::db::conn(db)?;
    let bases = crate::plugins::resolve_bases(tour_base);
    crate::generate::write_404(out, templates, &conn, site, &bases)?;
    crate::generate::write_search(out, templates, &conn, site, &bases)?;
    crate::generate::write_feeds(out, &conn, site, &bases)?;
    let urls = collect_urls(&conn, site, tour_base)?;
    crate::generate::write_sitemap(out, site, &urls)?;
    crate::generate::write_robots(out, site)?;
    crate::generate::write_llms_txt(out, &conn, site, &bases)?;
    let raw = collect_raw_redirects(&conn, site, tour_base)?;
    crate::generate::write_redirects(out, &conn, &raw)?;
    Ok(())
}

/// Scheduled-duties that used to ride inside `api_generate`'s spawn:
/// WP `_wp_scheduled_delete` equivalent — posts/comments trashed > 30 days
/// die here. Called at `serve()` boot so a long-running site never needs
/// a manual generate to collect its trash.
pub fn purge_duties(db: &Path, out: &Path, tour_base: &str) {
    if let Ok(ids) = crate::db::ids_matching(
        db,
        crate::sql!("sitefx__select_expired_trash"),
        [],
    ) {
        for id in ids {
            let _ = crate::generate::remove_file_for(db, out, id, tour_base);
        }
    }
    if let Err(e) = crate::db::purge_old_trash(db) {
        eprintln!("purge_old_trash: {e:#}");
    }
    // Data-lifecycle prune (phase 6): the rows below only grow — every save
    // writes a revision blob, every login a session row, every spam catch a
    // comment/submission. Caps keep the db bounded on a long-running box.
    if let Ok(c) = crate::db::conn(db) {
        // Revisions: latest 50 per post survive (body_json snapshots are the
        // top growth vector on builder pages — hundreds of KB per save).
        let _ = c.execute_batch(crate::sql!("sitefx__delete_old_revisions"));
        // Expired admin sessions — dead rows linger forever otherwise.
        let _ = c.execute(
            crate::sql!("sitefx__delete_expired_sessions"),
            [],
        );
        // Spam comments + spam/trash submissions past 30 days (same horizon
        // as post/comment trash above).
        let _ = c.execute(
            crate::sql!("sitefx__delete_old_spam_comments"),
            [],
        );
        let _ = c.execute(
            crate::sql!("sitefx__delete_old_spam_submissions"),
            [],
        );
    }
}
