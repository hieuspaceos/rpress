//! `cms generate` — render every publish post into an HTML tree.

#[allow(unused_imports)]
use crate::sql;
use crate::render;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn run(db: &Path, templates: &Path, out: &Path, site_url: &str) -> Result<()> {
    let t0 = Instant::now();
    fs::create_dir_all(out)?;
    let conn = crate::db::conn(db)?;
    crate::widgets::ensure(&conn)?;
    let front: i64 = conn
        .query_row(
            sql!("generate__select_cast_value"),
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn.prepare(
        sql!("generate__select_id_slug_2"),
    )?;
    let rows: Vec<(i64, String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<std::result::Result<_, _>>()?;

    let mut written = 0u64;
    let mut failed = 0u64;
    let mut urls: Vec<(String, String, Option<String>)> = Vec::new(); // loc, kind, lastmod
    let mut mod_stmt = conn
        .prepare("SELECT modified_at FROM posts WHERE id=?1")?;

    let mut raw_redirects: Vec<(String, String)> = Vec::new(); // raw slug → canonical (tours)
    for (id, slug, lang, kind) in &rows {
        // Tours write at the canonical display_slug path — the same value
        // render_tour::page puts in canonical/og:url. When it differs from
        // the stored slug the raw URL joins redirects.map instead of 404ing.
        let mut wslug = slug.clone();
        if kind == "tour" {
            let title: String = conn
                .query_row("SELECT title FROM posts WHERE id=?1", [id], |r| r.get(0))
                .unwrap_or_default();
            let display = crate::render_tour::display_slug(&conn, *id, slug, &title);
            if display != *slug {
                let prefix = if lang == "es" { "/es" } else { "" };
                raw_redirects.push((
                    format!("{prefix}/voyage/{slug}/"),
                    format!("{site_url}{prefix}/voyage/{display}/", site_url = site_url.trim_end_matches('/')),
                ));
                wslug = display;
            }
        }
        let rel = rel_path(*id, front, lang, kind, &wslug);
        let dest = out.join(&rel);
        if let Some(p) = dest.parent() {
            fs::create_dir_all(p)?;
        }
        match render::render_slug(db, templates, slug, lang, kind, site_url) {
            Ok((html, _status)) => {
                fs::write(&dest, html)?;
                // sibling index.md — the markdown content-negotiation target
                // (/slug/index.md + Accept header). Only for post/tour/page.
                if let Some(md) = render::post_markdown(&conn, slug, lang, kind, site_url) {
                    let md_dest = dest.with_file_name("index.md");
                    let _ = fs::write(&md_dest, md);
                }
                written += 1;
                let loc_path = rel.to_string_lossy().replace("index.html", "");
                let loc = format!(
                    "{}{}",
                    site_url.trim_end_matches('/'),
                    format!("/{loc_path}").replace("//", "/")
                );
                let lastmod: Option<String> = mod_stmt
                    .query_row([*id], |r| r.get(0))
                    .ok();
                urls.push((loc, kind.clone(), lastmod));
            }
            Err(e) => {
                failed += 1;
                eprintln!("skip {slug}: {e:#}");
            }
        }
    }

    // ES front page (post 9761, `__trashed` slug) — filtered out of the
    // posts loop above; resolved via post_translations/fr fallback.
    {
        let dest = out.join("es/index.html");
        match render::render_slug(db, templates, "es", "es", "", site_url) {
            Ok((html, _status)) => {
                fs::write(&dest, html)?;
                written += 1;
                let lastmod: Option<String> = conn
                    .query_row(
                        "SELECT modified_at FROM posts WHERE slug='es' OR id=9761 LIMIT 1",
                        [],
                        |r| r.get(0),
                    )
                    .ok();
                urls.push((format!("{}/es/", site_url.trim_end_matches('/')), "page".into(), lastmod));
            }
            Err(e) => eprintln!("skip es front: {e:#}"),
        }
    }

    // Taxonomy archives: tour taxonomies at {tax}/{slug}/ (+es/), post
    // categories at their nested WP path, post tags at etiquette/{slug}/.
    {
        let mut tstmt = conn.prepare(sql!("archive__select_term_paths"))?;
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
        // term → languages of the posts carrying it (shared taxonomies like
        // category/post_tag serve both).
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
        let mut emit = |_tax: &str, _slug: &str, lang: &str, path: &str| -> Result<()> {
            let rel = PathBuf::from(format!("{path}/index.html"));
            let dest = out.join(&rel);
            if let Some(p) = dest.parent() {
                fs::create_dir_all(p)?;
            }
            let render_slug_arg = path.trim_matches('/');
            match render::render_slug(db, templates, render_slug_arg, lang, "archive", site_url) {
                Ok((html, _status)) => {
                    fs::write(&dest, html)?;
                    written += 1;
                    let loc_path = rel.to_string_lossy().replace("index.html", "");
                    urls.push((
                        format!("{}/{loc_path}", site_url.trim_end_matches('/')),
                        "archive".into(),
                        archive_lastmod.clone(),
                    ));
                }
                Err(e) => eprintln!("skip {path}: {e:#}"),
            }
            Ok(())
        };
        for (tax, tslug) in &terms {
            let tax_l = tax.to_lowercase();
            if tax_l == "category" || tax_l == "post_tag" {
                let mut langs = term_langs(tax, tslug);
                if langs.is_empty() {
                    langs.push(if crate::render_post::cat_path_is_es(tslug) {
                        "es".into()
                    } else {
                        "fr".into()
                    });
                }
                for lang in langs {
                    let es = lang == "es";
                    let prefix = if es { "es/" } else { "" };
                    let path = if tax_l == "category" {
                        format!("{prefix}{}", crate::render_post::cat_path(tslug))
                    } else {
                        format!("{prefix}etiquette/{tslug}")
                    };
                    emit(&tax_l, tslug, &lang, &path)?;
                }
            } else if crate::render_archive::is_taxonomy(&tax_l) {
                let es = matches!(tax_l.as_str(), "region-es" | "perfil" | "duracion" | "estilo");
                let lang = if es { "es" } else { "fr" };
                let path = if es {
                    format!("es/{tax_l}/{tslug}")
                } else {
                    format!("{tax_l}/{tslug}")
                };
                emit(&tax_l, tslug, lang, &path)?;
            }
        }
    }

    write_404(out, templates, &conn, site_url)?;
    // Search index must exist (or not) before the search page renders —
    // its template branches on `pagefind` to emit the real UI or an
    // honest "unavailable" notice instead of a dead div (6.5).
    run_pagefind(out);
    write_search(out, templates, &conn, site_url)?;
    write_feeds(out, &conn, site_url)?;
    write_sitemap(out, site_url, &urls)?;
    write_robots(out, site_url)?;
    write_llms_txt(out, &conn, site_url, "voyage")?;

    let dur = t0.elapsed();
    println!(
        "generate: {written} pages, {failed} failed, {} in {:.1}s → {}",
        urls.len(),
        dur.as_secs_f32(),
        out.display()
    );
    if failed > 0 {
        anyhow::bail!("{failed} pages failed to render");
    }
    // Full clean build ⇒ every pending widget edit is now published.
    conn.execute(
        "INSERT OR REPLACE INTO settings(key, value) VALUES('widgets_dirty', '0')",
        [],
    )?;
    Ok(())
}

pub(crate) fn rel_path(id: i64, front: i64, lang: &str, kind: &str, slug: &str) -> PathBuf {
    let es = lang == "es";
    if id == front {
        return if es {
            PathBuf::from("es/index.html")
        } else {
            PathBuf::from("index.html")
        };
    }
    let mut p = PathBuf::new();
    if es {
        p.push("es");
    }
    if kind == "tour" {
        p.push("voyage");
    }
    p.push(slug);
    p.push("index.html");
    p
}

/// Resolve a post's rendered output path (lang prefix, kind dir, front-page
/// special case, tour display_slug) from an existing row.
fn post_out_rel(conn: &Connection, post_id: i64) -> Result<PathBuf> {
    let (slug, lang, kind): (String, String, String) = conn
        .query_row(
            "SELECT slug, lang, kind FROM posts WHERE id=?1",
            [post_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .with_context(|| format!("post {post_id} not found"))?;
    let front: i64 = conn
        .query_row(sql!("generate__select_cast_value"), [], |r| r.get(0))
        .unwrap_or(0);
    let wslug = if kind == "tour" {
        let title: String = conn
            .query_row("SELECT title FROM posts WHERE id=?1", [post_id], |r| r.get(0))
            .unwrap_or_default();
        crate::render_tour::display_slug(conn, post_id, &slug, &title)
    } else {
        slug
    };
    Ok(rel_path(post_id, front, &lang, &kind, &wslug))
}

/// Remove a still-existing post's rendered file — call BEFORE hard-deleting
/// the row (regen_post can't resolve a deleted id). Trashes a page that was
/// never rendered is a harmless no-op.
pub fn remove_file_for(db: &Path, out: &Path, post_id: i64) -> Result<()> {
    let conn = crate::db::conn(db)?;
    let rel = post_out_rel(&conn, post_id)?;
    let dest = out.join(&rel);
    if dest.exists() {
        fs::remove_file(&dest)?;
    }
    Ok(())
}

/// Single-file regenerate for one post id — comment moderation, scheduled
/// flips and small fixes shouldn't pay for a full site build. Non-publish
/// ids delete their rendered file instead (trash/draft URLs must die).

pub fn regen_post(db: &Path, templates: &Path, out: &Path, post_id: i64) -> Result<()> {
    let conn = crate::db::conn(db)?;
    let status: String = conn
        .query_row("SELECT status FROM posts WHERE id=?1", [post_id], |r| r.get(0))
        .with_context(|| format!("regen_post: post {post_id} not found"))?;
    let rel = post_out_rel(&conn, post_id)?;
    let dest = out.join(&rel);
    if status != "publish" {
        // Draft/trash: the rendered file must go, not just stay stale.
        if dest.exists() {
            fs::remove_file(&dest)?;
        }
        return Ok(());
    }
    let site_url: String = conn
        .query_row(sql!("admin__select_value_from"), [], |r| r.get(0))
        .unwrap_or_else(|_| "/".to_string());
    let (slug, lang, kind): (String, String, String) = conn
        .query_row("SELECT slug, lang, kind FROM posts WHERE id=?1", [post_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
    let (html, _status) = render::render_slug(db, templates, &slug, &lang, &kind, &site_url)?;
    if let Some(p) = dest.parent() {
        fs::create_dir_all(p)?;
    }
    // tmp+rename: a reader mid-write must never see a half file.
    let tmp = dest.with_extension("tmp");
    fs::write(&tmp, html)?;
    fs::rename(&tmp, &dest)?;
    Ok(())
}

/// Minimal base-template context for utility pages (404, search stub) that
/// have no post row: every var base_fr/base_es reads.
fn bare_ctx(
    conn: &Connection,
    site_url: &str,
    lang: &str,
    title: &str,
    canonical: &str,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    use serde_json::json;
    let mut m = serde_json::Map::new();
    let es = lang == "es";
    m.insert("html_lang".into(), json!(if es { "es-ES" } else { "fr-FR" }));
    m.insert("og_locale".into(), json!(if es { "es_ES" } else { "fr_FR" }));
    m.insert("html_prefix".into(), json!("prefix=\"og: https://ogp.me/ns#\" class=\"no-js\""));
    m.insert("kind".into(), json!("page"));
    m.insert("body_class".into(), json!("error404 wp-theme-traveltour wp-child-theme-traveltour-child"));
    m.insert("is_front".into(), json!(false));
    m.insert("is_contact".into(), json!(false));
    m.insert("is_single".into(), json!(false));
    m.insert("robots".into(), json!("noindex, follow"));
    m.insert("seo_title".into(), json!(title));
    m.insert("seo_desc".into(), json!(""));
    m.insert("og_desc".into(), json!(""));
    m.insert("og_type".into(), json!("website"));
    m.insert("og_image_block".into(), json!(""));
    m.insert("og_image_url".into(), json!(""));
    m.insert("og_updated".into(), json!(""));
    m.insert("tw_label1".into(), json!(""));
    m.insert("tw_data1".into(), json!(""));
    m.insert("tw_label2".into(), json!(""));
    m.insert("tw_data2".into(), json!(""));
    m.insert("oembed_url".into(), json!(""));
    m.insert("rest_url".into(), json!(""));
    m.insert("shortlink".into(), json!(""));
    m.insert("canonical".into(), json!(canonical));
    m.insert("hreflang_fr".into(), json!(""));
    m.insert("hreflang_es".into(), json!(""));
    m.insert("flag_fr_url".into(), json!(site_url));
    m.insert("flag_es_url".into(), json!(format!("{}/es/", site_url.trim_end_matches('/'))));
    m.insert("navx".into(), serde_json::Value::Object(serde_json::Map::new()));
    m.insert("page_title_html".into(), json!(""));
    m.insert("head_css_extra".into(), json!(""));
    m.insert("head_css_eztoc".into(), json!(""));
    m.insert("head_css_wpdiscuz".into(), json!(""));
    m.insert("foot_scripts_eztoc".into(), json!(""));
    m.insert("foot_scripts_comment_reply".into(), json!(""));
    m.insert("foot_scripts_wpdiscuz".into(), json!(""));
    m.insert("lang".into(), json!(lang));
    m.insert("title".into(), json!(title));
    m.insert("site_url".into(), json!(site_url));
    // Footer sidebars — same registry the page chrome uses.
    for n in 1..=4 {
        let sidebar = format!("footer-{n}-{}", if es { "es" } else { "fr" });
        let html = crate::widgets::render_sidebar(conn, &sidebar, site_url, lang, "")?;
        m.insert(format!("footer_{n}"), serde_json::json!(html));
    }
    m.insert("body".into(), json!(""));
    Ok(m)
}

pub(crate) fn write_404(out: &Path, templates: &Path, conn: &Connection, site_url: &str) -> Result<()> {
    // Real 404 markup from the template — previously this re-emitted the
    // homepage as 404.html.
    let env = render::load_env(templates)?;
    let site = site_url.trim_end_matches('/');
    let ctx = bare_ctx(conn, site_url, "fr", "404 — Page introuvable", &format!("{site}/404.html"))?;
    let html = env
        .get_template("404.html")?
        .render(minijinja::Value::from_serialize(serde_json::Value::Object(ctx)))?;
    fs::write(out.join("404.html"), html)?;
    Ok(())
}

/// Run `pagefind --site <out>` when the binary is installed (6.5).
/// Pagefind builds a merged index; each page's `lang` attribute routes it
/// to the right sub-index, so FR and ES content both index with their own
/// stemming — no per-locale invocation needed. Absent binary or a failed
/// run leaves no `pagefind/` dir, and the search page falls back to its
/// "temporarily unavailable" state rather than a dead UI shell.
pub(crate) fn run_pagefind(out: &Path) {
    match std::process::Command::new("pagefind")
        .arg("--site")
        .arg(out)
        .status()
    {
        Ok(st) if st.success() => eprintln!("pagefind: indexed {}", out.display()),
        Ok(st) => eprintln!("pagefind: exited {st} — search page will show 'unavailable'"),
        Err(_) => eprintln!("pagefind: binary not found — search page will show 'unavailable' (install: see CMS-OPERATIONS.md)"),
    }
}

pub(crate) fn write_search(out: &Path, templates: &Path, conn: &Connection, site_url: &str) -> Result<()> {
    // `pagefind` tells the template whether the index bundle exists this
    // run: real pagefind-ui when present, degraded form + notice when not.
    let indexed = out.join("pagefind").join("pagefind-ui.js").is_file()
        && out.join("pagefind").join("pagefind.js").is_file();
    let env = render::load_env(templates)?;
    let site = site_url.trim_end_matches('/');
    for (lang, dir) in [("fr", "recherche"), ("es", "es/recherche")] {
        let mut ctx = bare_ctx(
            conn,
            site_url,
            lang,
            if lang == "es" { "Buscar" } else { "Recherche" },
            &format!("{site}/{dir}/"),
        )?;
        ctx.insert("pagefind".into(), serde_json::json!(indexed));
        let html = env
            .get_template("search.html")?
            .render(minijinja::Value::from_serialize(serde_json::Value::Object(ctx)))?;
        let d = out.join(dir);
        fs::create_dir_all(&d)?;
        fs::write(d.join("index.html"), html)?;
    }
    Ok(())
}

pub(crate) fn write_feeds(out: &Path, conn: &Connection, site: &str) -> Result<()> {
    let site = site.trim_end_matches('/');
    // WP "RSS feeds show the most recent / excerpt" setting (Settings →
    // Reading). 0 = full text in <content:encoded>, 1 = excerpt only.
    let use_excerpt: i64 = conn
        .query_row(
            "SELECT CAST(value AS INT) FROM settings WHERE key='rss_use_excerpt'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    for (lang, path) in [("fr", "feed/index.xml"), ("es", "es/feed/index.xml")] {
        let dest = out.join(path);
        if let Some(p) = dest.parent() {
            fs::create_dir_all(p)?;
        }
        let mut items = String::new();
        // Same row set as generate__select_title_slug plus body_html for the
        // full-text feed variant (inline SQL: src/db/sql is owned elsewhere).
        let mut stmt = conn.prepare(
            "SELECT title, slug, published_at, excerpt, body_html FROM posts
             WHERE status='publish' AND kind='post' AND lang=?1
             ORDER BY published_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map([lang], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        })?;
        for row in rows {
            let (title, slug, date, excerpt, body) = row?;
            let prefix = if lang == "es" { "/es" } else { "" };
            let link = format!("{site}{prefix}/{slug}/");
            let pub_date = rfc822(&date);
            let mut item = format!(
                "<item><title><![CDATA[{title}]]></title><link>{link}</link><guid isPermaLink=\"true\">{link}</guid><pubDate>{pub_date}</pubDate><description><![CDATA[{}]]></description>",
                excerpt.unwrap_or_default()
            );
            if use_excerpt == 0 {
                item.push_str(&format!(
                    "<content:encoded><![CDATA[{}]]></content:encoded>",
                    body.unwrap_or_default()
                ));
            }
            items.push_str(&item);
            items.push_str("</item>\n");
        }
        let channel_link = if lang == "es" {
            format!("{site}/es")
        } else {
            site.to_string()
        };
        let channel_title = conn
            .query_row(
                "SELECT value FROM settings WHERE key='site_name' LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .unwrap_or_else(|_| "pressrs".into());
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/"><channel><title>{channel_title}</title><link>{channel_link}</link><description>Feed</description>
{items}</channel></rss>"#
        );
        fs::write(dest, xml)?;
    }
    Ok(())
}

/// RFC-822 date (`Mon, 20 Oct 2026 10:24:27 +0000`) from the
/// `YYYY-MM-DD HH:MM:SS` UTC strings stored in `published_at`/`modified_at`.
/// Weekday via days→civil arithmetic, no chrono dep.
fn rfc822(dt: &str) -> String {
    const DAYS: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (y, m, d): (i64, i64, i64) = (
        dt.get(0..4).and_then(|s| s.parse().ok()).unwrap_or(0),
        dt.get(5..7).and_then(|s| s.parse().ok()).unwrap_or(0),
        dt.get(8..10).and_then(|s| s.parse().ok()).unwrap_or(0),
    );
    let time = dt.get(11..19).unwrap_or("00:00:00");
    // days since epoch (Howard Hinnant's civil algorithm, day-of-week on the
    // result: 1970-01-01 = Thursday = index 0 of DAYS).
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = if y2 >= 0 { y2 } else { y2 - 399 } / 400;
    let yoe = y2 - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    let wday = DAYS[days.rem_euclid(7) as usize];
    let mon = MONTHS.get((m - 1) as usize).copied().unwrap_or("Jan");
    format!("{wday}, {d:02} {mon} {y:04} {time} +0000")
}

/// `YYYY-MM-DD HH:MM:SS` → `YYYY-MM-DDTHH:MM:SSZ` — sitemap `<lastmod>`
/// validators want ISO-8601, sqlite stores a space separator.
fn iso_lastmod(ts: &str) -> String {
    let mut t = ts.replace(' ', "T");
    if !t.ends_with('Z') {
        t.push('Z');
    }
    t
}

pub(crate) fn write_sitemap(out: &Path, site: &str, urls: &[(String, String, Option<String>)]) -> Result<()> {
    let site = site.trim_end_matches('/');
    let mut by: std::collections::BTreeMap<String, Vec<(&str, Option<&str>)>> =
        std::collections::BTreeMap::new();
    for (loc, kind, lastmod) in urls {
        by.entry(kind.clone())
            .or_default()
            .push((loc.as_str(), lastmod.as_deref()));
    }
    let mut index = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);
    for (kind, list) in &by {
        let name = format!("sitemap-{kind}.xml");
        let mut body = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
        );
        let mut index_lastmod: Option<String> = None;
        for (loc, lastmod) in list {
            body.push_str(&format!("<url><loc>{loc}</loc>"));
            if let Some(lm) = lastmod {
                let lm = iso_lastmod(lm);
                body.push_str(&format!("<lastmod>{lm}</lastmod>"));
                if index_lastmod.as_deref().map_or(true, |cur| lm.as_str() > cur) {
                    index_lastmod = Some(lm);
                }
            }
            body.push_str("</url>");
        }
        body.push_str("</urlset>");
        fs::write(out.join(&name), body)?;
        index.push_str(&format!("<sitemap><loc>{site}/{name}</loc>"));
        if let Some(lm) = index_lastmod {
            index.push_str(&format!("<lastmod>{lm}</lastmod>"));
        }
        index.push_str("</sitemap>");
    }
    index.push_str("</sitemapindex>");
    fs::write(out.join("sitemap.xml"), index)?;
    Ok(())
}

pub(crate) fn write_robots(out: &Path, site: &str) -> Result<()> {
    fs::write(
        out.join("robots.txt"),
        format!("User-agent: *\nAllow: /\nSitemap: {site}/sitemap.xml\n"),
    )?;
    Ok(())
}

/// llms.txt — curated index of every published page + its markdown sibling,
/// for LLM crawlers. Same shape as https://llmstxt.org.
pub(crate) fn write_llms_txt(out: &Path, conn: &Connection, site: &str, tour_base: &str) -> Result<()> {
    let site = site.trim_end_matches('/');
    let mut stmt = conn.prepare(
        "SELECT title, slug, lang, kind FROM posts WHERE status='publish' ORDER BY kind, lang, slug",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .flatten()
        .collect::<Vec<_>>();
    let site_name = conn
        .query_row(
            "SELECT value FROM settings WHERE key='site_name' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "pressrs".into());
    let mut body = String::new();
    body.push_str(&format!("# {site_name}\n\n"));
    body.push_str("## Pages\n\n");
    for (title, slug, lang, kind) in &rows {
        let prefix = if lang == "es" { "/es" } else { "" };
        let dir = if kind == "tour" { format!("/{tour_base}") } else { String::new() };
        let html = format!("{site}{prefix}{dir}/{slug}/");
        let md = format!("{site}{prefix}{dir}/{slug}/index.md");
        body.push_str(&format!("- [{title}]({html}) ([md]({md}))\n"));
    }
    fs::write(out.join("llms.txt"), body)?;
    Ok(())
}

pub(crate) fn write_redirects(out: &Path, conn: &Connection, raw: &[(String, String)]) -> Result<()> {
    let mut f = fs::File::create(out.join("redirects.map"))?;
    let mut stmt = conn.prepare(sql!("generate__select_old_slug_new_url"))?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    for row in rows {
        let (old, new) = row?;
        let old = old.trim_start_matches('/');
        writeln!(f, "~^/{old}  {new};")?;
    }
    // raw slug → canonical display slug for tours whose stored slug is
    // degenerate (6416 'x' → /voyage/du-nord-au-sud-vietnam-12-jours/).
    for (old, new) in raw {
        writeln!(f, "~^{old}  {new};")?;
    }
    Ok(())
}
