//! Site-level file side-effects that used to live inside `cms generate`.
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
fn collect_urls(conn: &Connection, site: &str) -> Result<Vec<(String, String, Option<String>)>> {
    let site = site.trim_end_matches('/');
    let mut urls: Vec<(String, String, Option<String>)> = Vec::new();

    let front: i64 = conn
        .query_row(crate::sql!("generate__select_cast_value"), [], |r| r.get(0))
        .unwrap_or(0);

    let mut stmt = conn.prepare(crate::sql!("generate__select_id_slug_2"))?;
    let rows: Vec<(i64, String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .flatten()
        .collect();
    let mut mod_stmt = conn.prepare("SELECT modified_at FROM posts WHERE id=?1")?;
    for (id, slug, lang, kind) in &rows {
        let wslug = if kind == "tour" {
            let title: String = conn
                .query_row("SELECT title FROM posts WHERE id=?1", [id], |r| r.get(0))
                .unwrap_or_default();
            crate::render_tour::display_slug(conn, *id, slug, &title)
        } else {
            slug.clone()
        };
        let rel = crate::generate::rel_path(*id, front, lang, kind, &wslug);
        let loc_path = rel.to_string_lossy().replace("index.html", "");
        let loc = format!(
            "{}{}",
            site,
            format!("/{loc_path}").replace("//", "/")
        );
        let lastmod: Option<String> = mod_stmt.query_row([*id], |r| r.get(0)).ok();
        urls.push((loc, kind.clone(), lastmod));
    }

    // ES front page (post 9761, `__trashed` slug — filtered out of the loop).
    {
        let lastmod: Option<String> = conn
            .query_row(
                "SELECT modified_at FROM posts WHERE slug='es' OR id=9761 LIMIT 1",
                [],
                |r| r.get(0),
            )
            .ok();
        urls.push((format!("{site}/es/"), "page".into(), lastmod));
    }

    // Taxonomy archives — same set generate emitted.
    {
        let mut tstmt = conn.prepare(crate::sql!("archive__select_term_paths"))?;
        let terms: Vec<(String, String)> = tstmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .flatten()
            .collect();
        let archive_lastmod: Option<String> = conn
            .query_row(
                "SELECT MAX(modified_at) FROM posts WHERE status='publish'",
                [],
                |r| r.get(0),
            )
            .ok()
            .flatten();
        let term_langs = |tax: &str, slug: &str| -> Vec<String> {
            conn.prepare(
                "SELECT DISTINCT p.lang FROM posts p JOIN post_terms pt \
                 ON pt.post_id=p.id WHERE pt.taxonomy=?1 COLLATE NOCASE \
                 AND pt.slug=?2 AND p.status='publish'",
            )
            .and_then(|mut s| {
                let langs = s
                    .query_map(rusqlite::params![tax, slug], |r| r.get::<_, String>(0))?
                    .flatten()
                    .collect::<Vec<_>>();
                Ok(langs)
            })
            .unwrap_or_default()
        };
        for (tax, tslug) in &terms {
            let tax_l = tax.to_lowercase();
            if tax_l == "category" || tax_l == "post_tag" {
                let mut langs = term_langs(&tax_l, tslug);
                if langs.is_empty() {
                    langs.push(if crate::render_post::cat_path_is_es(tslug) {
                        "es".into()
                    } else {
                        "fr".into()
                    });
                }
                for lang in langs {
                    let prefix = if lang == "es" { "es/" } else { "" };
                    let path = if tax_l == "category" {
                        format!("{prefix}{}", crate::render_post::cat_path(tslug))
                    } else {
                        format!("{prefix}etiquette/{tslug}")
                    };
                    urls.push((
                        format!("{site}/{path}/"),
                        "archive".into(),
                        archive_lastmod.clone(),
                    ));
                }
            } else if crate::render_archive::is_taxonomy(&tax_l) {
                let es = matches!(tax_l.as_str(), "region-es" | "perfil" | "duracion" | "estilo");
                let path = if es {
                    format!("es/{tax_l}/{tslug}")
                } else {
                    format!("{tax_l}/{tslug}")
                };
                urls.push((
                    format!("{site}/{path}/"),
                    "archive".into(),
                    archive_lastmod.clone(),
                ));
            }
        }
    }

    Ok(urls)
}

/// Raw-slug → canonical display-slug redirect rows for degenerate tour
/// slugs (6416 'x'). Mirrors the raw_redirects collection in `generate::run`.
fn collect_raw_redirects(conn: &Connection, site: &str) -> Result<Vec<(String, String)>> {
    let site = site.trim_end_matches('/');
    let mut raw = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, slug, lang, title FROM posts WHERE status='publish' AND kind='tour'",
    )?;
    let rows: Vec<(i64, String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .flatten()
        .collect();
    for (id, slug, lang, title) in &rows {
        let display = crate::render_tour::display_slug(conn, *id, slug, title);
        if &display != slug {
            let prefix = if lang == "es" { "/es" } else { "" };
            raw.push((
                format!("{prefix}/voyage/{slug}/"),
                format!("{site}{prefix}/voyage/{display}/"),
            ));
        }
    }
    Ok(raw)
}

/// One pass over everything `cms generate` used to emit besides post HTML.
/// Cheap enough to run at `serve()` boot: pure SQL + string writes, no
/// renders. Called once on startup so a cold cache still has sitemap,
/// feeds, robots, redirects and the search page from minute zero.
pub fn emit_site_files(db: &Path, templates: &Path, out: &Path, site: &str) -> Result<()> {
    let conn = crate::db::conn(db)?;
    crate::generate::write_404(out, templates, &conn, site)?;
    crate::generate::write_search(out, templates, &conn, site)?;
    crate::generate::write_feeds(out, &conn, site)?;
    let urls = collect_urls(&conn, site)?;
    crate::generate::write_sitemap(out, site, &urls)?;
    crate::generate::write_robots(out, site)?;
    let raw = collect_raw_redirects(&conn, site)?;
    crate::generate::write_redirects(out, &conn, &raw)?;
    Ok(())
}

/// Scheduled-duties that used to ride inside `api_generate`'s spawn:
/// WP `_wp_scheduled_delete` equivalent — posts/comments trashed > 30 days
/// die here. Called at `serve()` boot so a long-running site never needs
/// a manual generate to collect its trash.
pub fn purge_duties(db: &Path, out: &Path) {
    if let Ok(ids) = crate::db::ids_matching(
        db,
        "SELECT id FROM posts WHERE status='trash' AND modified_at < datetime('now','-30 days')",
        [],
    ) {
        for id in ids {
            let _ = crate::generate::remove_file_for(db, out, id);
        }
    }
    if let Err(e) = crate::db::purge_old_trash(db) {
        eprintln!("purge_old_trash: {e:#}");
    }
}
