//! `rpress generate` — render every publish post into an HTML tree.

#[allow(unused_imports)]
use crate::sql;
use crate::plugins::{self, KindBases};
use crate::render;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub fn atomic_write(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let tmp = path.with_extension(format!(
        "tmp-{}-{stamp}",
        std::process::id()
    ));
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn run(db: &Path, templates: &Path, out: &Path, site_url: &str, tour_base: &str) -> Result<()> {
    let t0 = Instant::now();
    fs::create_dir_all(out)?;
    // CLI/env boundary value → per-kind URL bases; everything below
    // resolves its own base via plugins::base_for.
    let kind_bases = plugins::resolve_bases(tour_base);
    let conn = crate::db::conn(db)?;
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
        .prepare(sql!("generate__select_modified_at"))?;

    let lang_cfg = crate::lang::config(&conn);

    // Spec harvest kinds (fetch-at-import): stored HTML wins over both
    // the elementor fetch-at-generate set below and any adapter render —
    // `harvested_pages` rows only exist for spec-owned kinds.
    // Elementor & co (fetch-at-generate): harvest the origin's rendered
    // HTML instead of re-rendering — fidelity by construction. Table is
    // written by import; absent on dbs imported before this feature.
    let mut harvested_html: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    if let Ok(mut s) = conn.prepare(sql!("generate__select_harvested_pages")) {
        if let Ok(rows) = s.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))) {
            harvested_html = rows.flatten().collect();
        }
    }
    let mut harvest: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let legacy_origin: String = conn
        .query_row(sql!("generate__select_legacy_origin"), [], |r| r.get(0))
        .unwrap_or_default();
    if let Ok(mut s) = conn.prepare(sql!("generate__select_harvest_posts")) {
        if let Ok(rows) = s.query_map([], |r| r.get(0)) {
            harvest = rows.flatten().collect();
        }
    }
    let mut harvested = 0u64;
    let mut harvest_failed = 0u64;

    let mut raw_redirects: Vec<(String, String)> = Vec::new(); // raw slug → canonical (display_slug kinds)
    for (id, slug, lang, kind) in &rows {
        // Base-prefixed kinds (the tour adapter) write at the canonical
        // display_slug path — the same value the adapter render puts in
        // canonical/og:url. When it differs from the stored slug the raw
        // URL joins redirects.map instead of 404ing.
        let mut wslug = slug.clone();
        let kind_base = plugins::base_for(&kind_bases, kind);
        if let (Some(b), Some(adapter)) = (kind_base, plugins::adapter_for_kind(kind)) {
            let title: String = conn
                .query_row(sql!("generate__select_post_title"), [id], |r| r.get(0))
                .unwrap_or_default();
            if let Some(display) = adapter.display_slug(&conn, *id, slug, &title) {
                if display != *slug {
                    let prefix = crate::lang::url_prefix(&lang_cfg, lang);
                    raw_redirects.push((
                        format!("{prefix}/{b}/{slug}/"),
                        format!("{site_url}{prefix}/{b}/{display}/", site_url = site_url.trim_end_matches('/')),
                    ));
                    wslug = display;
                }
            }
        }
        let rel = rel_path_with_cfg(&lang_cfg, *id, front, lang, &wslug, kind_base);
        let dest = out.join(&rel);
        let stored = harvested_html.remove(id);
        let rendered = if let Some(html) = stored {
            harvested += 1;
            // Stored markup carries `{{ site_url }}` placeholders —
            // resolve against this render's origin (deploy/render_slug).
            Some(Ok((crate::render_post::rewrite_origin(&html, site_url, &conn), "harvested".to_string(), String::new())))
        } else if harvest.contains(id) && !legacy_origin.is_empty() {
            // Fetch-at-generate (elementor). For a post that is ALSO a
            // spec-harvest kind, the origin URL uses the kind's
            // origin_base, not the local rel path (they diverge when
            // origin_base ≠ url_base). Spec kinds with no stored row
            // and no harvest_posts membership fall to bare-body —
            // port-once semantics, no per-generate fetch.
            let harvest_path = plugins::spec::spec_harvest_bases()
                .iter()
                .find(|(k, _)| *k == kind.as_str())
                .map(|(_, base)| {
                    let lp = crate::lang::rel_prefix(&lang_cfg, lang);
                    match (lp.is_empty(), base.is_empty()) {
                        (true, true) => format!("{wslug}/"),
                        (true, false) => format!("{base}/{wslug}/"),
                        (false, true) => format!("{lp}/{wslug}/"),
                        (false, false) => format!("{lp}/{base}/{wslug}/"),
                    }
                })
                .unwrap_or_else(|| rel.to_string_lossy().replace("index.html", ""));
            match crate::harvest::page(&legacy_origin, &harvest_path, site_url) {
                Ok(h) => {
                    harvested += 1;
                    Some(Ok((h, "harvested".to_string(), String::new())))
                }
                Err(e) => {
                    harvest_failed += 1;
                    eprintln!("[harvest] {harvest_path} failed: {e:#} — rendering normally");
                    None
                }
            }
        } else {
            None
        };
        match rendered.unwrap_or_else(|| render::render_slug_conn(&conn, templates, slug, lang, kind, site_url, tour_base)) {
            Ok((html, _status, _canonical)) => {
                atomic_write(&dest, html)?;
                // sibling index.md — the markdown content-negotiation target
                // (/slug/index.md + Accept header). Only for post/tour/page.
                if let Some(md) = render::post_markdown(&conn, slug, lang, kind, site_url, tour_base) {
                    let md_dest = dest.with_file_name("index.md");
                    let _ = atomic_write(&md_dest, md);
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

    // Localized front pages for every non-primary language. The primary
    // front is emitted by the posts loop as index.html.
    for lang in lang_cfg.langs.iter().filter(|l| *l != &lang_cfg.primary) {
        let rel = PathBuf::from(crate::lang::rel_prefix(&lang_cfg, lang)).join("index.html");
        match render::render_slug_conn(&conn, templates, lang, lang, "", site_url, tour_base) {
            Ok((html, _status, _canonical)) => {
                atomic_write(&out.join(&rel), html)?;
                written += 1;
                let lastmod: Option<String> = conn
                    .query_row(sql!("generate__select_max_modified_page_lang"), [lang], |r| r.get(0))
                    .ok();
                urls.push((format!("{}{}", site_url.trim_end_matches('/'), crate::lang::page_path(&lang_cfg, lang, "page", "", None)), "page".into(), lastmod));
            }
            Err(e) => {
                // No localized front post — harvest origin/{lang}/ when the
                // site has builder posts and a known origin.
                if !legacy_origin.is_empty() {
                    let path = rel.to_string_lossy().replace("index.html", "");
                    match crate::harvest::page(&legacy_origin, &path, site_url) {
                        Ok(h) => {
                            atomic_write(&out.join(&rel), h)?;
                            harvested += 1;
                            written += 1;
                            continue;
                        }
                        Err(he) => eprintln!("[harvest] {lang} front: {he:#}"),
                    }
                }
                eprintln!("skip {lang} front: {e:#}")
            }
        }
    }

    // Taxonomy archives: tour taxonomies at {tax}/{slug}/ (+es/), post
    // categories at their nested WP path, post tags at etiquette/{slug}/.
    // term_out_paths owns the mapping — invalidate_dependents consumes
    // the same paths so a post edit deletes exactly what generate wrote.
    {
        let mut tstmt = conn.prepare(sql!("archive__select_term_paths"))?;
        let terms: Vec<(String, String)> = tstmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .flatten()
            .collect();
        let archive_lastmod: Option<String> = conn
            .query_row(
                sql!("generate__select_max_modified_published"),
                [],
                |r| r.get(0),
            )
            .ok()
            .flatten();
        let mut emit = |_tax: &str, _slug: &str, lang: &str, path: &str| -> Result<()> {
            let rel = PathBuf::from(format!("{path}/index.html"));
            let dest = out.join(&rel);
            let render_slug_arg = path.trim_matches('/');
            match render::render_slug_conn(&conn, templates, render_slug_arg, lang, "archive", site_url, tour_base) {
                Ok((html, _status, _canonical)) => {
                    atomic_write(&dest, html)?;
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
            for (lang, dir) in term_out_paths(&conn, &lang_cfg, tax, tslug) {
                emit(tax, tslug, &lang, &dir.to_string_lossy())?;
            }
        }
    }

    write_404(out, templates, &conn, site_url, &kind_bases)?;
    // Search index must exist (or not) before the search page renders —
    // its template branches on `pagefind` to emit the real UI or an
    // honest "unavailable" notice instead of a dead div (6.5).
    run_pagefind(out);
    write_search(out, templates, &conn, site_url, &kind_bases)?;
    write_feeds(out, &conn, site_url, &kind_bases)?;
    write_sitemap(out, site_url, &urls)?;
    write_robots(out, site_url)?;
    write_llms_txt(out, &conn, site_url, &kind_bases)?;

    let dur = t0.elapsed();
    if !harvest.is_empty() {
        println!("harvest: {harvested} fetched, {harvest_failed} failed (of {} builder posts)", harvest.len());
    }
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
        sql!("generate__upsert_widgets_dirty"),
        [],
    )?;
    Ok(())
}

pub(crate) fn rel_path_with_cfg(cfg: &crate::lang::LangConfig, id: i64, front: i64, lang: &str, slug: &str, kind_base: Option<&str>) -> PathBuf {
    if id == front {
        let lp = crate::lang::rel_prefix(cfg, lang);
        return if lp.is_empty() { PathBuf::from("index.html") } else { PathBuf::from(lp).join("index.html") };
    }
    let mut p = PathBuf::new();
    let lp = crate::lang::rel_prefix(cfg, lang);
    if !lp.is_empty() { p.push(lp); }
    if let Some(b) = kind_base { p.push(b); }
    p.push(slug);
    p.push("index.html");
    p
}

/// Resolve a post's rendered output path (lang prefix, kind dir, front-page
/// special case, tour display_slug) from an existing row.
fn post_out_rel(conn: &Connection, post_id: i64, kind_bases: &KindBases) -> Result<PathBuf> {
    let (slug, lang, kind): (String, String, String) = conn
        .query_row(
            sql!("generate__select_slug_lang_kind"),
            [post_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .with_context(|| format!("post {post_id} not found"))?;
    let front: i64 = conn
        .query_row(sql!("generate__select_cast_value"), [], |r| r.get(0))
        .unwrap_or(0);
    // Adapter-owned canonical display slug (tour). Stored slug when the
    // kind has no adapter or the adapter declines a rewrite.
    let wslug = plugins::adapter_for_kind(&kind)
        .and_then(|a| {
            let title: String = conn
                .query_row(sql!("generate__select_post_title"), [post_id], |r| r.get(0))
                .unwrap_or_default();
            a.display_slug(conn, post_id, &slug, &title)
        })
        .unwrap_or(slug);
    let cfg = crate::lang::config(conn);
    Ok(rel_path_with_cfg(&cfg, post_id, front, &lang, &wslug, plugins::base_for(kind_bases, &kind)))
}

/// Every `(lang, dir)` a term archive renders at — THE shared mapping
/// `generate::run` emits and `invalidate_dependents` deletes from
/// (phase 5): category paths come from the theme's taxonomy map
/// (`theme_traveltour::cat_path`, seeded `plugin:theme-traveltour:taxonomy_map`),
/// post tags sit at `{tag_base}/{slug}`, and other
/// routable taxonomies at `{tax}/{slug}`; non-primary languages carry
/// their rel prefix. ES detection uses `cat_path_is_es` plus
/// the `-es` taxonomies list, matching the dispatch this replaces.
/// Languages come from the posts carrying the term (`term_langs`);
/// termless posts still resolve through the taxonomy-map heuristics.
/// Returns (lang, archive directory) pairs — `dir/index.html` is the file.
pub(crate) fn term_out_paths(
    conn: &Connection,
    lang_cfg: &crate::lang::LangConfig,
    tax: &str,
    slug: &str,
) -> Vec<(String, PathBuf)> {
    // term → languages of the published posts carrying it (shared
    // taxonomies like category/post_tag serve both).
    let term_langs = |tax: &str, slug: &str| -> Vec<String> {
        conn.prepare(sql!("generate__select_term_post_langs"))
        .and_then(|mut s| {
            let langs = s
                .query_map(rusqlite::params![tax, slug], |r| r.get::<_, String>(0))?
                .flatten()
                .collect::<Vec<_>>();
            Ok(langs)
        })
        .unwrap_or_default()
    };
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    let tax_l = tax.to_lowercase();
    if tax_l == "category" || tax_l == "post_tag" {
        let mut langs = term_langs(tax, slug);
        if langs.is_empty() {
            langs.push(if crate::plugins::theme_traveltour::cat_path_is_es(conn, slug) {
                "es".into()
            } else {
                "fr".into()
            });
        }
        let tag_base = crate::plugins::theme_traveltour::tag_base(conn);
        for lang in langs {
            let prefix = crate::lang::rel_prefix(lang_cfg, &lang);
            let prefix = if prefix.is_empty() { String::new() } else { format!("{prefix}/") };
            let path = if tax_l == "category" {
                format!("{prefix}{}", crate::plugins::theme_traveltour::cat_path(conn, slug))
            } else {
                format!("{prefix}{tag_base}/{slug}")
            };
            out.push((lang, PathBuf::from(path)));
        }
    } else if crate::render_archive::is_taxonomy(conn, &tax_l) {
        let langs = term_langs(tax, slug);
        let lang = langs.first().cloned().unwrap_or_else(|| {
            if matches!(tax_l.as_str(), "region-es" | "perfil" | "duracion" | "estilo") { "es".into() } else { lang_cfg.primary.clone() }
        });
        let prefix = crate::lang::rel_prefix(lang_cfg, &lang);
        let path = if prefix.is_empty() { format!("{tax_l}/{slug}") } else { format!("{prefix}/{tax_l}/{slug}") };
        out.push((lang, PathBuf::from(path)));
    }
    out
}

/// Every cache-file rel a post can occupy under `out/` — the removal set
/// for unpublish/trash/delete and the invalidation set for saves. Covers
/// the write-back contract enumerated on `public_page`: canonical
/// `{base}/index.html` + `index.md`, legacy `{base}.html` flat aliases,
/// and `/{lang}/` copies under EVERY configured prefix — the literal
/// primary-prefix dir (swept at boot but recreatable by races) plus any
/// cross-lang alias written before the phase-1 canonical redirects.
pub(crate) fn post_file_variants(
    cfg: &crate::lang::LangConfig,
    rel: &Path,
    post_lang: &str,
) -> Vec<PathBuf> {
    // Page dir = the rendered file's parent (`index.html` → "").
    let base = rel.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let mut variants = Vec::new();
    let mut push3 = |dir: &Path| {
        variants.push(dir.join("index.html"));
        variants.push(dir.join("index.md"));
        // flat {dir}.html legacy alias — only for real dirs (the front
        // page's empty base would yield ".html").
        if !dir.as_os_str().is_empty() {
            let mut flat = dir.to_path_buf();
            flat.set_extension("html");
            variants.push(flat);
        }
    };
    // Canonical copy (rel already carries the post's own lang prefix).
    push3(&base);
    // Unprefixed base: the alias dir under which every language copy sits
    // (same as `base` minus its leading own-lang component).
    let own_prefix = crate::lang::rel_prefix(cfg, post_lang);
    let unprefixed = if !own_prefix.is_empty() {
        base.strip_prefix(&own_prefix).map(|p| p.to_path_buf()).unwrap_or_else(|_| base.clone())
    } else {
        base.clone()
    };
    for l in &cfg.langs {
        // Skip the copy the canonical branch already produced (own prefix
        // on the unprefixed base == base itself).
        if l == post_lang && !own_prefix.is_empty() {
            continue;
        }
        push3(&PathBuf::from(l).join(&unprefixed));
    }
    variants.sort();
    variants.dedup();
    variants
}

/// Lazily invalidate every artifact a post mutation can stale (phase 5,
/// H4/H5): the post's own file variants, each term archive the post
/// carries (nested categories via term_out_paths), every localized
/// homepage index.html, per-lang feeds and all sitemap*.xml. DELETE only
/// — the lazy write-back cache re-renders on the next request; rendering
/// here would re-cache before dependent rows (terms, menus) settle.
/// `extra_terms` = (taxonomy, slug) pairs the post carried BEFORE the
/// mutation — a category/tag reassignment must invalidate the OLD
/// archives too; current post_terms alone can't see them.
/// Returns files removed; failures are logged, never fatal — the
/// post-regen second sweep in `regen_posts` retries stragglers.
pub fn invalidate_dependents(
    conn: &Connection,
    out: &Path,
    post_id: i64,
    tour_base: &str,
    extra_terms: &[(String, String)],
) -> usize {
    let kind_bases = plugins::resolve_bases(tour_base);
    let mut removed = 0usize;
    let mut del = |p: PathBuf| {
        if p.is_file() {
            match fs::remove_file(&p) {
                Ok(()) => removed += 1,
                Err(e) => eprintln!("invalidate {}: {e}", p.display()),
            }
        }
    };
    let lang_cfg = crate::lang::config(conn);
    // site_data materializes every published post/tour for element widgets —
    // a post-level save must drop it or cache-miss renders serve the old row.
    crate::render::site_data_bust();
    // The post's own variants (canonical + alias copies).
    if let Ok(rel) = post_out_rel(conn, post_id, &kind_bases) {
        let post_lang: String = conn
            .query_row(sql!("generate__select_post_lang"), [post_id], |r| r.get(0))
            .unwrap_or_default();
        for v in post_file_variants(&lang_cfg, &rel, &post_lang) {
            del(out.join(&v));
        }
    }
    // Term archives for every term the post carries now + carried before.
    let mut terms: Vec<(String, String)> = extra_terms.to_vec();
    if let Ok(mut s) = conn.prepare(
        sql!("generate__select_taxonomy_slug_by_post"),
    ) {
        if let Ok(rows) = s.query_map([post_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }) {
            terms.extend(rows.flatten());
        }
    }
    terms.sort();
    terms.dedup();
    for (tax, tslug) in &terms {
        for (_lang, dir) in term_out_paths(conn, &lang_cfg, tax, tslug) {
            del(out.join(&dir).join("index.html"));
        }
    }
    // Homepages (primary + every localized front).
    for lang in &lang_cfg.langs {
        let lp = crate::lang::rel_prefix(&lang_cfg, lang);
        let rel = if lp.is_empty() {
            PathBuf::from("index.html")
        } else {
            PathBuf::from(lp).join("index.html")
        };
        del(out.join(&rel));
    }
    // Feeds: {prefix}/feed/index.xml per language.
    for lang in &lang_cfg.langs {
        let lp = crate::lang::rel_prefix(&lang_cfg, lang);
        let rel = if lp.is_empty() {
            PathBuf::from("feed/index.xml")
        } else {
            PathBuf::from(format!("{lp}/feed/index.xml"))
        };
        del(out.join(&rel));
    }
    // Sitemaps: sitemap.xml + sitemap-{kind}.xml siblings.
    if let Ok(rd) = fs::read_dir(out) {
        for e in rd.flatten() {
            let name = e.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("sitemap") && name.ends_with(".xml") {
                del(e.path());
            }
        }
    }
    removed
}

/// The variant file set for a post id (same enumeration as
/// `post_file_variants`); callers that need the PATHS rather than the
/// removal itself — e.g. `posts_empty`, which must snapshot the doomed
/// files before the hard delete removes the row `post_out_rel` reads.
pub fn file_variants_for(conn: &Connection, post_id: i64, tour_base: &str) -> Result<Vec<PathBuf>> {
    let rel = post_out_rel(conn, post_id, &plugins::resolve_bases(tour_base))?;
    let lang_cfg = crate::lang::config(conn);
    let post_lang: String = conn
        .query_row(sql!("generate__select_post_lang"), [post_id], |r| r.get(0))
        .unwrap_or_default();
    Ok(post_file_variants(&lang_cfg, &rel, &post_lang))
}

/// Delete the file set `post_file_variants` enumerates — shared by
/// `remove_file_for` and `regen_post`'s non-publish branch.
fn remove_post_files(conn: &Connection, out: &Path, post_id: i64, tour_base: &str) -> Result<usize> {
    let mut removed = 0usize;
    for v in file_variants_for(conn, post_id, tour_base)? {
        let p = out.join(&v);
        if p.is_file() {
            fs::remove_file(&p).with_context(|| format!("remove {}", p.display()))?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Remove a still-existing post's rendered files — call BEFORE hard-
/// deleting the row (post_out_rel can't resolve a deleted id). Removes
/// the full variant set: index.html + index.md + {dir}.html + lang
/// copies. Trashing a page that was never rendered is a harmless no-op.
pub fn remove_file_for(db: &Path, out: &Path, post_id: i64, tour_base: &str) -> Result<()> {
    let conn = crate::db::conn(db)?;
    remove_post_files(&conn, out, post_id, tour_base)?;
    Ok(())
}

/// XML text-node escape for the raw-string emitters (sitemap `<loc>`,
/// feed `<link>`): `&<>` plus quotes so the value is attribute-safe too.
pub(crate) fn xml_esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// CDATA-safe payload: a literal `]]>` inside content would close the
/// section early — split it as `]]]]><![CDATA[>` (valid, lossless).
pub(crate) fn cdata(s: &str) -> String {
    s.replace("]]>", "]]]]><![CDATA[>")
}


/// Single-file regenerate for one post id — comment moderation, scheduled
/// flips and small fixes shouldn't pay for a full site build. Publish
/// rewrites index.html AND its index.md sibling (the two files a lazy
/// miss serves); non-publish removes the ENTIRE variant set — trash/
/// draft URLs and every alias copy must die, not just the canonical file.

pub fn regen_post(db: &Path, templates: &Path, out: &Path, post_id: i64, tour_base: &str) -> Result<()> {
    let conn = crate::db::conn(db)?;
    let kind_bases = plugins::resolve_bases(tour_base);
    let status: String = conn
        .query_row(sql!("generate__select_post_status"), [post_id], |r| r.get(0))
        .with_context(|| format!("regen_post: post {post_id} not found"))?;
    let rel = post_out_rel(&conn, post_id, &kind_bases)?;
    let dest = out.join(&rel);
    if status != "publish" {
        // Draft/trash: every rendered variant must go, not just stay stale.
        remove_post_files(&conn, out, post_id, tour_base)?;
        return Ok(());
    }
    let site_url: String = conn
        .query_row(sql!("admin__select_value_from"), [], |r| r.get(0))
        .unwrap_or_else(|_| "/".to_string());
    let (slug, lang, kind): (String, String, String) = conn
        .query_row(sql!("generate__select_slug_lang_kind"), [post_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
    let (html, _status, _canonical) = render::render_slug(db, templates, &slug, &lang, &kind, &site_url, tour_base)?;
    atomic_write(&dest, html)?;
    // index.md sibling — the markdown negotiation target. A stale .md
    // diverging from the .html was H5; regen keeps them in lock-step.
    if let Some(md) = render::post_markdown(&conn, &slug, &lang, &kind, &site_url, tour_base) {
        atomic_write(&dest.with_file_name("index.md"), md)?;
    }
    Ok(())
}

/// Minimal base-template context for utility pages (404, search stub) that
/// have no post row: every var base_fr/base_es reads.
fn bare_ctx(
    conn: &Connection,
    templates: &Path,
    site_url: &str,
    lang: &str,
    title: &str,
    canonical: &str,
    kind_base: Option<&str>,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    use serde_json::json;
    let mut m = serde_json::Map::new();
    let html_lang = match lang { "es" => "es-ES", "fr" => "fr-FR", other => other };
    let og_locale = match lang { "es" => "es_ES", "fr" => "fr_FR", other => other };
    m.insert("html_lang".into(), json!(html_lang));
    m.insert("og_locale".into(), json!(og_locale));
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
    let lang_cfg = crate::lang::config(conn);
    m.insert("flags".into(), json!(lang_cfg.langs.iter().map(|l| {
        let root = format!("{}{}/", site_url.trim_end_matches('/'), crate::lang::url_prefix(&lang_cfg, l));
        json!({"lang": l, "url": root, "flag": format!("{l}.svg")})
    }).collect::<Vec<_>>()));
    m.insert("hreflang_links".into(), json!(""));
    m.insert("navx".into(), serde_json::Value::Object(serde_json::Map::new()));
    m.insert("page_title_html".into(), json!(""));
    m.insert("head_css_extra".into(), json!(""));
    m.insert("head_css_eztoc".into(), json!(""));
    m.insert("head_css_wpdiscuz".into(), json!(""));
    m.insert("foot_scripts_eztoc".into(), json!(""));
    m.insert("foot_scripts_comment_reply".into(), json!(""));
    m.insert("foot_scripts_wpdiscuz".into(), json!(""));
    m.insert("head_extra".into(), json!(""));
    m.insert("lang".into(), json!(lang));
    m.insert("title".into(), json!(title));
    m.insert("site_url".into(), json!(site_url));
    /* Dynamic nav: base templates read `site.menus.*`. */
    if let Ok(site) = crate::render::site_data_cached(
        conn,
        lang,
        site_url,
        kind_base.unwrap_or_default(),
    ) {
        m.insert(
            "site".into(),
            serde_json::to_value(&site).unwrap_or(serde_json::Value::Null),
        );
    }
    // Footer sidebars — same registry the page chrome uses.
    for n in 1..=4 {
        let sidebar = format!("footer-{n}-{lang}");
        let html = crate::widgets::render_sidebar(conn, &sidebar, site_url, lang, "", kind_base.unwrap_or_default())?;
        m.insert(format!("footer_{n}"), serde_json::json!(html));
    }
    {
        let lang_cfg = crate::lang::config(conn);
        let theme_dir = crate::render::theme_template_dir(conn, templates);
        m.insert(
            "base".into(),
            json!(crate::lang::template_name(templates, theme_dir.as_deref(), &lang_cfg, lang)),
        );
    }
    m.insert("body".into(), json!(""));
    Ok(m)
}

pub(crate) fn write_404(out: &Path, templates: &Path, conn: &Connection, site_url: &str, kind_bases: &KindBases) -> Result<()> {
    // Real 404 markup from the template — previously this re-emitted the
    // homepage as 404.html.
    let theme_dir = render::theme_template_dir(conn, templates);
    let env = render::load_env_theme(templates, theme_dir.as_deref())?;
    let site = site_url.trim_end_matches('/');
    let ctx = bare_ctx(conn, templates, site_url, "fr", "404 — Page introuvable", &format!("{site}/404.html"), plugins::base_for(kind_bases, "tour"))?;
    let html = env
        .get_template("404.html")?
        .render(minijinja::Value::from_serialize(serde_json::Value::Object(ctx)))?;
    atomic_write(&out.join("404.html"), html)?;
    Ok(())
}

// Fetch implementation moved to `crate::harvest::page` (shared with
// import's fetch-at-import pass); signature identical.

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

pub(crate) fn write_search(out: &Path, templates: &Path, conn: &Connection, site_url: &str, kind_bases: &KindBases) -> Result<()> {
    // `pagefind` tells the template whether the index bundle exists this
    // run: real pagefind-ui when present, degraded form + notice when not.
    let indexed = out.join("pagefind").join("pagefind-ui.js").is_file()
        && out.join("pagefind").join("pagefind.js").is_file();
    let theme_dir = render::theme_template_dir(conn, templates);
    let env = render::load_env_theme(templates, theme_dir.as_deref())?;
    let site = site_url.trim_end_matches('/');
    let lang_cfg = crate::lang::config(conn);
    for lang in &lang_cfg.langs {
        let lp = crate::lang::rel_prefix(&lang_cfg, lang);
        let dir = if lp.is_empty() { "recherche".to_string() } else { format!("{lp}/recherche") };
        let mut ctx = bare_ctx(
            conn,
            templates,
            site_url,
            lang,
            if lang == "es" { "Buscar" } else { "Recherche" },
            &format!("{site}/{dir}/"),
            plugins::base_for(kind_bases, "tour"),
        )?;
        ctx.insert("pagefind".into(), serde_json::json!(indexed));
        let html = env
            .get_template("search.html")?
            .render(minijinja::Value::from_serialize(serde_json::Value::Object(ctx)))?;
        let d = out.join(dir);
        atomic_write(&d.join("index.html"), html)?;
    }
    Ok(())
}

pub(crate) fn write_feeds(out: &Path, conn: &Connection, site: &str, _kind_bases: &KindBases) -> Result<()> {
    let site = site.trim_end_matches('/');
    // WP "RSS feeds show the most recent / excerpt" setting (Settings →
    // Reading). 0 = full text in <content:encoded>, 1 = excerpt only.
    let use_excerpt: i64 = conn
        .query_row(
            sql!("generate__select_rss_use_excerpt"),
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let lang_cfg = crate::lang::config(conn);
    for lang in &lang_cfg.langs {
        let lp = crate::lang::rel_prefix(&lang_cfg, lang);
        let path = if lp.is_empty() { "feed/index.xml".to_string() } else { format!("{lp}/feed/index.xml") };
        let dest = out.join(path);
        if let Some(p) = dest.parent() {
            fs::create_dir_all(p)?;
        }
        let mut items = String::new();
        // Same row set as generate__select_title_slug plus body_html for the
        // full-text feed variant.
        let mut stmt = conn.prepare(
            sql!("generate__select_feed_posts"),
        )?;
        let rows = stmt.query_map([lang.as_str()], |r| {
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
            let prefix = crate::lang::url_prefix(&lang_cfg, lang);
            let link = xml_esc(&format!("{site}{prefix}/{slug}/"));
            let pub_date = rfc822(&date);
            let mut item = format!(
                "<item><title><![CDATA[{}]]></title><link>{link}</link><guid isPermaLink=\"true\">{link}</guid><pubDate>{pub_date}</pubDate><description><![CDATA[{}]]></description>",
                cdata(&title),
                cdata(&excerpt.unwrap_or_default()),
            );
            if use_excerpt == 0 {
                item.push_str(&format!(
                    "<content:encoded><![CDATA[{}]]></content:encoded>",
                    cdata(&body.unwrap_or_default())
                ));
            }
            items.push_str(&item);
            items.push_str("</item>\n");
        }
        let channel_link = xml_esc(&format!("{}{}", site, crate::lang::page_path(&lang_cfg, lang, "page", "", None).trim_end_matches('/')));
        let channel_title = conn
            .query_row(
                sql!("generate__select_site_name"),
                [],
                |r| r.get::<_, String>(0),
            )
            .unwrap_or_else(|_| "rpress".into());
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/"><channel><title>{}</title><link>{channel_link}</link><description>Feed</description>
{items}</channel></rss>"#,
            xml_esc(&channel_title),
        );
        atomic_write(&dest, xml)?;
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
            body.push_str(&format!("<url><loc>{}</loc>", xml_esc(loc)));
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
        atomic_write(&out.join(&name), body)?;
        index.push_str(&format!("<sitemap><loc>{site}/{name}</loc>"));
        if let Some(lm) = index_lastmod {
            index.push_str(&format!("<lastmod>{lm}</lastmod>"));
        }
        index.push_str("</sitemap>");
    }
    index.push_str("</sitemapindex>");
    atomic_write(
        &out.join("sitemap.xml"),
        index,
    )?;
    Ok(())
}

pub(crate) fn write_robots(out: &Path, site: &str) -> Result<()> {
    atomic_write(
        &out.join("robots.txt"),
        format!("User-agent: *\nAllow: /\nSitemap: {site}/sitemap.xml\n"),
    )?;
    Ok(())
}

/// llms.txt — curated index of every published page + its markdown sibling,
/// for LLM crawlers. Same shape as https://llmstxt.org.
pub(crate) fn write_llms_txt(out: &Path, conn: &Connection, site: &str, kind_bases: &KindBases) -> Result<()> {
    let site = site.trim_end_matches('/');
    let mut stmt = conn.prepare(
        sql!("generate__select_llms_pages"),
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
            sql!("generate__select_site_name"),
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "rpress".into());
    let mut body = String::new();
    body.push_str(&format!("# {site_name}\n\n"));
    body.push_str("## Pages\n\n");
    for (title, slug, lang, kind) in &rows {
        let lang_cfg = crate::lang::config(conn);
        let prefix = crate::lang::url_prefix(&lang_cfg, lang);
        let dir = plugins::base_for(kind_bases, kind).map(|b| format!("/{b}")).unwrap_or_default();
        let html = format!("{site}{prefix}{dir}/{slug}/");
        let md = format!("{site}{prefix}{dir}/{slug}/index.md");
        body.push_str(&format!("- [{title}]({html}) ([md]({md}))\n"));
    }
    atomic_write(&out.join("llms.txt"), body)?;
    Ok(())
}

/// redirects.map — nginx `map` lines `~^/{old}  {new};`. Not XML: what must
/// not leak is a `;` or newline (line injection) or whitespace (the map
/// splits on it). Rows carrying either are dropped — an unmappable redirect
/// beats a corrupt file.
pub(crate) fn write_redirects(out: &Path, conn: &Connection, raw: &[(String, String)]) -> Result<()> {
    let safe = |s: &str| -> Option<String> {
        if s.bytes().any(|b| b.is_ascii_whitespace() || b == b';') {
            None
        } else {
            Some(s.to_string())
        }
    };
    let mut body = String::new();
    let mut stmt = conn.prepare(sql!("generate__select_old_slug_new_url"))?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    for row in rows {
        let (old, new) = row?;
        let old = old.trim_start_matches('/');
        let (Some(old), Some(new)) = (safe(old), safe(&new)) else { continue };
        body.push_str(&format!("~^/{old}  {new};\n"));
    }
    // raw slug → canonical display slug for tours whose stored slug is
    // degenerate (6416 'x' → {tour_base}/du-nord-au-sud-vietnam-12-jours/).
    for (old, new) in raw {
        let (Some(old), Some(new)) = (safe(old), safe(new)) else { continue };
        body.push_str(&format!("~^{old}  {new};\n"));
    }
    atomic_write(&out.join("redirects.map"), body)?;
    Ok(())
}
