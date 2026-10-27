//! Render posts through minijinja templates.

#[allow(unused_imports)]
use crate::sql;
use anyhow::{Context, Result};
use minijinja::{context, Environment, Value as JVal};
use minijinja::value::Rest;
use rusqlite::{Connection, OptionalExtension};

use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::cell::RefCell;

// Per-render custom CSS collected from column/wrapper elements (the PHP
// page builder's `gdlr_core_insert_style` buffer). Flushed as one <style>
// block by the top-level render_nodes call — mirrors WP's aggregated head
// style tag instead of per-element inline <style>s.
thread_local! {
    static CUSTOM_STYLES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}
fn custom_styles_take() -> Vec<String> {
    CUSTOM_STYLES.with(|c| std::mem::take(&mut *c.borrow_mut()))
}
use std::sync::Arc;

const ELEMENT_TYPES: &[&str] = &[
    "accordion",
    "alert-box",
    "audio",
    "background",
    "blockquote",
    "blog",
    "button",
    "call-to-action",
    "chart",
    "code",
    "column",
    "column-service",
    "columnize",
    "content-navigation",
    "countdown",
    "counter",
    "custom-field",
    "custom-menu",
    "divider",
    "event",
    "feature-box",
    "feature-content",
    "featured-gallery",
    "flipbox",
    "food_menu",
    "gallery",
    "hover-box",
    "hover-content",
    "icon",
    "icon-list",
    "image",
    "image-content",
    "instagram",
    "opening-hours",
    "page-list",
    "port-info",
    "post-slider",
    "price-list",
    "price-plan",
    "price-table",
    "promo-box",
    "roadmap",
    "shape-divider",
    "skill-bar",
    "skill-circle",
    "social-network",
    "social-share",
    "space",
    "stunning-text",
    "tab",
    "tab-feature",
    "tab-feature-vertical",
    "tab-feature2",
    "tab-feature3",
    "testimonial",
    "text-box",
    "text-script",
    "timeline",
    "title",
    "toggle-box",
    "tour",
    "tour_search",
    "type-animation",
    "video",
    "widget",
    "wordpress-editor-content",
];

pub fn run(db: &Path, templates: &Path, slug: &str, out: &Path, site_url: &str, tour_base: &str) -> Result<()> {
    // A leading configured language prefix (e.g. es/, de/) asks for that
    // translation; otherwise the primary language is used.
    let conn = crate::db::conn(db)?;
    let cfg = crate::lang::config(&conn);
    let clean = slug.trim_matches('/');
    let (lang, path) = crate::lang::split_path(&cfg, clean);
    let (html, _status) = render_slug(db, templates, path, &lang, "", site_url, tour_base)?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, html)?;
    println!("wrote {}", out.display());
    Ok(())
}

/// Render one permalink. `lang`/`kind` scope the posts.slug lookup (the
/// import keeps FR+ES rows sharing a slug, e.g. `bun-bo-hue`): `lang` is the
/// resolved request language — matched exactly, publish-only; `kind=""`
/// leaves the kind axis open. Archive dispatches go through the term rules
/// below.
pub fn render_slug(
    db: &Path,
    templates: &Path,
    slug: &str,
    lang: &str,
    kind: &str,
    site_url: &str,
    tour_base: &str,
) -> Result<(String, String)> {
    let conn = crate::db::conn(db)?;
    let lang_cfg = crate::lang::config(&conn);
    let row: Option<(
        i64,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
    )> = conn
        .query_row(
            sql!("render__select_id_lang"),
            rusqlite::params![slug, lang, kind],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                ))
            },
        )
        .optional()?;

    // post lookup missed → term archives + ES-front aliases.
    let Some((id, lang, title, body_json, seo_title, seo_desc, kind, status)) = row else {
        let clean = slug.trim_matches('/');
        // "{lang}" / "{lang}/" → that language's front page.
        let (alang_s, path) = crate::lang::split_path(&lang_cfg, clean);
        if path.is_empty() && alang_s != lang_cfg.primary {
            if let Some(front_id) = front_id_for_lang(&conn, &alang_s) {
                return render_slug_by_id_public(&conn, templates, front_id, site_url, tour_base);
            }
        }
        // Term/archive permalinks (the posts.slug lookup already missed):
        //   "{tax}/{slug}"                → tour-taxonomy archive
        //   "etiquette/{slug}"            → post_tag archive
        //   "{cat}[/{sub}…]" (≤4 levels)  → category archive (leaf slug;
        //     this site's categories sit at the path root, no /category/
        //     base — sidebar_post.html shows /le-vietnam/culture-histoire/)
        // A leading configured language prefix picks that variant.
        let alang = alang_s.as_str();
        let first = path.split('/').next().unwrap_or("");
        let leaf = path.rsplit('/').next().unwrap_or("");
        let term = match first {
            "etiquette" => crate::render_archive::term_by_path(&conn, "post_tag", leaf),
            "category" => crate::render_archive::term_by_path(&conn, "category", leaf),
            t if crate::render_archive::is_taxonomy(&conn, t) && !leaf.is_empty() => {
                crate::render_archive::term_by_path(&conn, t, leaf)
            }
            _ => None,
        };
        if let Some(term) = term {
            let ctx = crate::render_tour::PostCtx {
                slug: slug.to_string(),
                lang: alang.to_string(),
                title: term.name.clone(),
                id: term.wp_id,
                site_url: site_url.to_string(),
                body_html: String::new(),
                body_html_rendered: String::new(),
                kind: "archive".to_string(),
                tour_base: tour_base.to_string(),
            };
            let h = crate::render_archive::page(&conn, templates, &ctx, &term)?;
            return Ok((h, "publish".to_string()));
        }
        // Category archives can be nested or single-segment; resolve the
        // leaf slug once post and old-slug lookups have missed.
        if !leaf.is_empty() {
            if let Some(term) = crate::render_archive::term_by_path(&conn, "category", leaf) {
                let ctx = crate::render_tour::PostCtx {
                    slug: slug.to_string(),
                    lang: alang.to_string(),
                    title: term.name.clone(),
                    id: term.wp_id,
                    site_url: site_url.to_string(),
                    body_html: String::new(),
                    body_html_rendered: String::new(),
                    kind: "archive".to_string(),
                    tour_base: tour_base.to_string(),
                };
                let h = crate::render_archive::page(&conn, templates, &ctx, &term)?;
                return Ok((h, "publish".to_string()));
            }
        }
        // canonical slug of a post whose stored slug is degenerate
        // ('x' on tour 6416): the import-time permalink lives in the
        // redirect row written from that post's own _wp_old_slug set.
        if !leaf.is_empty() {
            if let Ok(Some(pid)) = conn
                .query_row(
                    "SELECT pm.post_id FROM post_meta pm \
                     JOIN redirects r ON r.old_slug=pm.meta_value AND r.source='old-slug' \
                     WHERE pm.meta_key='_wp_old_slug' AND r.new_url=?1 LIMIT 1",
                    [format!("/{leaf}/")],
                    |r| r.get::<_, i64>(0),
                )
                .optional()
            {
                return render_slug_by_id_public(&conn, templates, pid, site_url, tour_base);
            }
        }
        anyhow::bail!("slug not found: {slug}");
    };
    let h = render_row(&conn, templates, id, slug, lang, title, body_json, seo_title, seo_desc, kind, site_url, tour_base)?;
    // Final-stage origin rewrite: element option values, menus, chrome and
    // any other imported markup may carry the source domain — normalize it
    // here so no render path can leak it (filter_content alone missed them).
    let h = crate::render_post::rewrite_origin(&h, site_url, &conn);
    Ok((h, status))
}


/// Apply `navx` menu classes produced by `nav_map` onto the rendered page:
/// each `{tax}/{slug}`-style key maps to the nav <a> whose href is the
/// matching permalink. Tour term keys mark just the term's <li> (WP's
/// current-tour-* classes sit on the item itself); post/page/es-home keys
/// mark the item plus its <li> ancestors like the archive backstop does.
pub(crate) fn apply_navx(html: &str, site_url: &str, navx: &Map<String, Value>) -> String {
    let site = site_url.trim_end_matches('/');
    let mut out = html.to_string();
    for (key, classes) in navx {
        let cls = classes.as_str().unwrap_or("").trim().to_string();
        if cls.is_empty() {
            continue;
        }
        let (href, item_only) = match key.as_str() {
            "es-home" => (format!("{site}/es/"), false),
            _ => match key.split_once('/') {
                Some(("post", s)) => (format!("{site}/{s}/"), false),
                Some(("page", s)) => {
                    // the front page's menu object is the site root, not
                    // its literal slug permalink.
                    let p = if s == "homepage" { String::new() } else { format!("{s}/") };
                    (format!("{site}/{p}"), false)
                }
                Some((tax, s)) => (format!("{site}/{tax}/{s}/"), true),
                None => continue,
            },
        };
        out = mark_nav_item(&out, &href, &cls, item_only);
    }
    out
}

/// Byte ranges of the nav menu <ul>s (`sf-menu` desktop, `m-menu` mobile).
/// Flag links, canonical <link>s and sidebar cat-item anchors share the
/// same hrefs as menu items, so nav marking must stay inside these spans.
pub(crate) fn nav_menu_ranges(html: &str) -> Vec<(usize, usize)> {
    let b = html.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'<' && html[i..].starts_with("<ul") {
            let end = html[i..].find('>').map(|e| i + e).unwrap_or(html.len());
            let head = &html[i..end];
            if head.contains("sf-menu") || head.contains("m-menu") {
                // find the matching </ul> past any nested <ul>s
                let mut depth = 1usize;
                let mut j = end + 1;
                let mut close = html.len();
                while j < b.len() {
                    if b[j] == b'<' {
                        if html[j..].starts_with("<ul") {
                            depth += 1;
                            j += 3;
                            continue;
                        }
                        if html[j..].starts_with("</ul") {
                            depth -= 1;
                            if depth == 0 {
                                close = j;
                                break;
                            }
                            j += 4;
                            continue;
                        }
                    }
                    j += 1;
                }
                out.push((i, close));
                i = close + 5;
                continue;
            }
            i = end + 1;
            continue;
        }
        i += 1;
    }
    out
}

/// `pos` sits inside one of the nav menu spans.
pub(crate) fn in_nav(ranges: &[(usize, usize)], pos: usize) -> bool {
    ranges.iter().any(|(s, e)| pos >= *s && pos < *e)
}

/// Mark the nav <li> enclosing <a href="{href}"> with `item_cls`; when
/// `item_only` is false the enclosing <li> chain gets WP ancestor classes
/// and the anchor gains aria-current="page" (same shape as the term-archive
/// marker). Byte offsets applied right-to-left stay valid.
fn mark_nav_item(html: &str, href: &str, item_cls: &str, item_only: bool) -> String {
    let needle = format!("href=\"{href}\"");
    let ranges = nav_menu_ranges(html);
    // same href shows up on flag links, canonical <link>s, sidebar
    // cat-items — only <a> tags inside the nav menu <ul>s get marked
    let is_nav_anchor = |a: usize| -> bool {
        in_nav(&ranges, a)
            && html[..a]
                .rsplit_once('<')
                .is_some_and(|(_, t)| t.starts_with("a ") && !t.contains('>'))
    };
    let mut hits: Vec<usize> = Vec::new();
    let mut scan = 0usize;
    while let Some(a) = html[scan..].find(&needle).map(|p| p + scan) {
        if is_nav_anchor(a) {
            hits.push(a);
        }
        scan = a + needle.len();
    }
    let mut inserts: Vec<(usize, String)> = Vec::new();
    for &a in &hits {
        let mut stack: Vec<(usize, usize)> = Vec::new();
        let mut i = 0usize;
        let b = html.as_bytes();
        while i < a {
            if b[i] == b'<' {
                if html[i..].starts_with("<li") && matches!(b.get(i + 3), Some(b' ') | Some(b'>')) {
                    let end = html[i..].find('>').map(|e| i + e).unwrap_or(a);
                    stack.push((i, end));
                    i = end + 1;
                    continue;
                }
                if html[i..].starts_with("</li") {
                    stack.pop();
                    i += 4;
                    continue;
                }
            }
            i += 1;
        }
        let n = stack.len();
        for (depth, &(s, e)) in stack.iter().enumerate() {
            let cls = if depth + 1 == n {
                item_cls
            } else if item_only {
                continue;
            } else if depth + 2 == n {
                "current-menu-ancestor current-menu-parent"
            } else {
                "current-menu-ancestor"
            };
            let head = &html[s..e];
            if !head.contains("menu-item") || head.contains("current-menu") {
                continue;
            }
            if let Some(c) = head.find("class=\"") {
                inserts.push((s + c + 7, format!("{cls} ")));
            }
        }
    }
    if !item_only {
        for &a in &hits {
            if let Some(gt) = html[a..].find('>').map(|e| a + e) {
                if !html[a..gt].contains("aria-current") {
                    inserts.push((gt, " aria-current=\"page\"".to_string()));
                }
            }
        }
    }
    inserts.sort_by(|x, y| y.0.cmp(&x.0));
    inserts.dedup_by(|x, y| x.0 == y.0);
    let mut out = html.to_string();
    for (at, text) in inserts {
        out.insert_str(at, &text);
    }
    out
}



/// Posts row → rendered page: the body of render_slug after slug lookup.
#[allow(clippy::too_many_arguments)]
fn render_row(
    conn: &Connection,
    templates: &Path,
    id: i64,
    slug: &str,
    lang: String,
    title: String,
    body_json: String,
    seo_title: Option<String>,
    seo_desc: Option<String>,
    kind: String,
    site_url: &str,
    tour_base: &str,
) -> Result<String> {
    let media = media_map(conn)?;

    // current-post context for elements that read post content/meta
    let mut post = Map::new();
    post.insert("id".into(), json!(id));
    post.insert("title".into(), json!(title));
    let body_html: String = conn
        .query_row("SELECT COALESCE(body_html,'') FROM posts WHERE id=?1", [id], |r| {
            r.get(0)
        })
        .unwrap_or_default();
    post.insert("body_html".into(), json!(body_html));
    {
        let mut mstmt = conn.prepare(
            "SELECT DISTINCT meta_key, meta_value FROM post_meta WHERE post_id=?1 AND meta_key NOT LIKE '\\_%' ESCAPE '\\'",
        )?;
        let mrows = mstmt.query_map([id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        let mut meta = Map::new();
        for row in mrows.flatten() {
            meta.entry(row.0).or_insert(Value::String(row.1));
        }
        post.insert("meta".into(), Value::Object(meta));
    }
    let post_v = JVal::from_serialize(Value::Object(post));

    let tree: Value = serde_json::from_str(&body_json).context("body_json")?;
    let mut env = load_env(templates)?;
    register_helpers(&mut env, &media, site_url);
    let site = site_data(&conn, &lang, site_url, &media, tour_base)?;
    let body = render_nodes(&mut env, &tree, &media, site_url, &site, &lang, &post_v, Some(&conn), true, tour_base)?;

    // tour/post kinds get their full single-page scaffolding from dedicated
    // modules (banner, booking bar, breadcrumbs, comments, sidebar…).
    match kind.as_str() {
        "tour" => {
            let ctx = crate::render_tour::PostCtx {
                slug: slug.to_string(),
                lang: lang.clone(),
                title: title.clone(),
                id,
                site_url: site_url.to_string(),
                body_html: body_html.clone(),
                body_html_rendered: body.clone(),
                kind: kind.clone(),
                tour_base: tour_base.to_string(),
            };
            return crate::render_tour::page(&conn, templates, &ctx);
        }
        "post" => {
            let ctx = crate::render_post::PostCtx {
                slug: slug.to_string(),
                lang: lang.clone(),
                title: title.clone(),
                id,
                site_url: site_url.to_string(),
                body_html: body_html.clone(),
                body_html_rendered: body.clone(),
                kind: kind.clone(),
                tour_base: tour_base.to_string(),
            };
            return crate::render_post::page(&conn, templates, &ctx);
        }
        _ => {}
    }

    let lang_cfg = crate::lang::config(conn);
    let mut alternates = translation_urls(conn, id, site_url, tour_base)?;
    let mut canonical = crate::lang::page_url(&lang_cfg, site_url, &lang, kind.as_str(), slug, tour_base);
    // Front pages: WP permalinks them at the language root regardless of
    // stored slug; generate one alternate root per configured language.
    if lang_cfg.langs.iter().any(|l| front_id_for_lang(conn, l) == Some(id)) {
        canonical = crate::lang::page_url(&lang_cfg, site_url, &lang, "page", "", tour_base);
        alternates = lang_cfg.langs.iter()
            .filter(|l| front_id_for_lang(conn, l).is_some())
            .map(|l| (l.clone(), crate::lang::page_url(&lang_cfg, site_url, l, "page", "", tour_base)))
            .collect();
    }
    let hreflang_fr = alternates.iter().find(|(l, _)| l == "fr").map(|(_, u)| u.clone()).unwrap_or_default();
    let hreflang_es = alternates.iter().find(|(l, _)| l == "es").map(|(_, u)| u.clone()).unwrap_or_default();
    let hreflang_links = hreflang_links(&alternates);
    let base = crate::lang::template_name(templates, &lang_cfg, &lang);
    // page templates just wrap body; we inject into base content block directly
    let site_name = conn
        .query_row(
            "SELECT value FROM settings WHERE key='site_name' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "rpress".into());
    let seo_title = seo_title
        .filter(|s| !s.is_empty())
        .map(|t| crate::db::resolve_seo(&t, &title, &site_name))
        .filter(|s| !s.is_empty())
        .unwrap_or(title.clone());
    let seo_desc = seo_desc
        .map(|d| crate::db::resolve_seo(&d, &title, &site_name))
        .unwrap_or_default();

    // shared <head>/chrome vars (meta block, links, body class, nav, flags)
    let mut ctx = chrome_vars(
        &conn,
        site_url,
        id,
        &kind,
        slug,
        &lang,
        &title,
        &canonical,
        &seo_desc,
        &body_html,
        &hreflang_fr,
        &hreflang_es,
        tour_base,
    )?;
    // Templates use `seo_title` inside <title> and meta content attributes:
    // pre-escape for attribute position (esc_attr covers text too — the
    // base has AutoEscape::None). `title` is escaped for the same reason.
    ctx.insert("seo_title".into(), json!(esc_attr(&seo_title)));
    ctx.insert("seo_desc".into(), json!(seo_desc));
    ctx.insert("canonical".into(), json!(canonical));
    ctx.insert("hreflang_fr".into(), json!(hreflang_fr));
    ctx.insert("hreflang_es".into(), json!(hreflang_es));
    ctx.insert("hreflang_links".into(), json!(hreflang_links));
    ctx.insert("lang".into(), json!(lang));
    ctx.insert("title".into(), json!(esc(&title)));
    ctx.insert("site_url".into(), json!(site_url));
    ctx.insert("body".into(), json!(body));

    let navx = match ctx.get("navx") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };
    let html = env
        .get_template(&base)?
        .render(JVal::from_serialize(Value::Object(ctx)))
        .map_err(|e| anyhow::anyhow!("render {base}: {e}"))?;
    Ok(apply_navx(&html, site_url, &navx))
}

/// FR + ES front-page post ids. `page_on_front` gives the FR post (publish-
/// gated by `generate__select_cast_value`); its translation lives in
/// post_translation_members (or the legacy post_translations pair table),
/// else the lone trashed ES page WP kept as the localized front.
pub(crate) fn front_ids(conn: &Connection) -> (Option<i64>, Option<i64>) {
    let fr: Option<i64> = conn
        .query_row(
            sql!("generate__select_cast_value"),
            [],
            |r| r.get(0),
        )
        .ok();
    (fr, es_front_id(conn))
}

/// ES front post id — see `front_id_for_lang`.
fn es_front_id(conn: &Connection) -> Option<i64> {
    front_id_for_lang(conn, "es")
}

/// Localized front page for `lang`, deterministic and publish-only:
///   1. the primary `page_on_front`'s translation-group member in `lang`;
///   2. the legacy post_translations pair member in `lang` (admin writes
///      land here on sites imported before members sync);
///   3. Polylang's `__trashed` localized front — `ORDER BY id LIMIT 1`
///      pins the pick instead of an arbitrary row.
fn front_id_for_lang(conn: &Connection, lang: &str) -> Option<i64> {
    let cfg = crate::lang::config(conn);
    let fr: Option<i64> = conn
        .query_row(
            sql!("generate__select_cast_value"),
            [],
            |r| r.get(0),
        )
        .ok();
    if lang == cfg.primary {
        return fr;
    }
    if let Some(front) = fr {
        // Translation-group member of the primary front in the target lang.
        let grouped: Option<i64> = conn
            .query_row(
                "SELECT m2.post_id FROM post_translation_members m \
                 JOIN post_translation_members m2 ON m2.group_id=m.group_id \
                 JOIN posts p ON p.id=m2.post_id AND p.status='publish' \
                 WHERE m.post_id=?1 AND m2.lang=?2 LIMIT 1",
                rusqlite::params![front, lang],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        if grouped.is_some() {
            return grouped;
        }
        // Legacy fr/es pair table — admin translation links land here too.
        let paired: Option<i64> = conn
            .query_row(
                "SELECT p.id FROM post_translations t \
                 JOIN posts p ON p.id=CASE WHEN t.fr_post_id=?1 THEN t.es_post_id ELSE t.fr_post_id END \
                 WHERE p.lang=?2 AND p.status='publish' \
                   AND (t.fr_post_id=?1 OR t.es_post_id=?1) LIMIT 1",
                rusqlite::params![front, lang],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        if paired.is_some() {
            return paired;
        }
    }
    // Generic fallback: Polylang often keeps the localized static front as
    // a publish page whose slug is a trashed artifact.
    conn.query_row(
        "SELECT id FROM posts WHERE lang=?1 AND kind='page' AND slug LIKE '%__trashed%' AND status='publish' ORDER BY id LIMIT 1",
        [lang],
        |r| r.get::<_, i64>(0),
    )
    .optional()
    .ok()
    .flatten()
}

/// Render a post addressed by id (the ES front alias — its slug carries a
/// `__trashed` suffix WP strips at the permalink level). `publish_only`
/// picks the publish-gated query for public paths; preview passes false so
/// drafts stay previewable. Returns (html, real status) — callers must not
/// fabricate "publish".
fn render_by_id(
    conn: &Connection,
    templates: &Path,
    id: i64,
    site_url: &str,
    publish_only: bool,
    tour_base: &str,
) -> Result<(String, String)> {
    let (lang, title, body_json, seo_title, seo_desc, kind, status): (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
    ) = conn
        .query_row(
            if publish_only {
                sql!("render__select_by_id_public")
            } else {
                sql!("render__select_by_id")
            },
            [id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            },
        )
        .with_context(|| format!("post id not found: {id}"))?;
    // WP permalinks the front translation at /es/ — the real slug is a
    // trashed artifact; pass it through for the old-slug machinery and let
    // render_row's front-page special-case fix the canonical.
    let slug: String = conn
        .query_row("SELECT slug FROM posts WHERE id=?1", [id], |r| r.get(0))
        .unwrap_or_default();
    let h = render_row(conn, templates, id, &slug, lang, title, body_json, seo_title, seo_desc, kind, site_url, tour_base)?;
    Ok((crate::render_post::rewrite_origin(&h, site_url, conn), status))
}

/// Public by-id render (front-page aliases, old-slug fallback): publish-
/// gated — a draft/trash id resolves to Err, which the handler turns into
/// a 404 *without* a cache write.
pub(crate) fn render_slug_by_id_public(
    conn: &Connection,
    templates: &Path,
    id: i64,
    site_url: &str,
    tour_base: &str,
) -> Result<(String, String)> {
    render_by_id(conn, templates, id, site_url, true, tour_base)
}

/// Canonical URL of a post's translation group for a missed `/{lang}/{slug}`:
/// the request asked for a `lang` rendering of `slug` that doesn't exist —
/// when a published sibling holds the slug (or a group member carries the
/// primary lang), Polylang redirects to the existing variant instead of a
/// 404. Returns the canonical path (no scheme/host).
pub(crate) fn translation_canonical(
    conn: &Connection,
    cfg: &crate::lang::LangConfig,
    lang: &str,
    slug: &str,
    kind: &str,
    tour_base: &str,
) -> Option<String> {
    if lang == cfg.primary || slug.is_empty() {
        return None;
    }
    // A published sibling in another language holding this exact slug.
    let (id, plang, pkind, pslug): (i64, String, String, String) = conn
        .query_row(
            "SELECT id, lang, kind, slug FROM posts \
             WHERE slug=?1 AND lang<>?2 AND (?3='' OR kind=?3) \
             AND status='publish' LIMIT 1",
            rusqlite::params![slug, lang, kind],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()
        .ok()
        .flatten()?;
    // Prefer the group's primary-lang member when the post is grouped —
    // its permalink is the canonical one. Tours permalink under their
    // display slug (meta-derived), not the stored slug.
    let target = conn
        .query_row(
            "SELECT m2.post_id, p.lang, p.kind, p.slug FROM post_translation_members m \
             JOIN post_translation_members m2 ON m2.group_id=m.group_id AND m2.lang=?2 \
             JOIN posts p ON p.id=m2.post_id AND p.status='publish' \
             WHERE m.post_id=?1 LIMIT 1",
            rusqlite::params![id, cfg.primary],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?)),
        )
        .optional()
        .ok()
        .flatten()
        .map(|(_pid, l, k, s)| (l, k, s))
        .unwrap_or((plang, pkind, pslug));
    let (tlang, tkind, tslug) = target;
    let url_slug = if tkind == "tour" {
        let tid: i64 = conn
            .query_row("SELECT id FROM posts WHERE slug=?1 AND lang=?2 AND kind='tour' AND status='publish' LIMIT 1",
                rusqlite::params![tslug, tlang], |r| r.get(0))
            .unwrap_or(id);
        let title: String = conn
            .query_row("SELECT title FROM posts WHERE id=?1", [tid], |r| r.get(0))
            .unwrap_or_default();
        crate::render_tour::display_slug(conn, tid, &tslug, &title)
    } else {
        tslug
    };
    Some(crate::lang::page_url(cfg, "", &tlang, &tkind, &url_slug, tour_base))
}

/// Admin draft preview (phase 6.1): render any post — including drafts,
/// pending, scheduled — through the same pipeline the generator uses,
/// then force robots to noindex so a leaked preview URL can't be crawled.
/// The route also sets `X-Robots-Tag: noindex`.
pub fn render_preview(
    db: &Path,
    templates: &Path,
    post_id: i64,
    site_url: &str,
    tour_base: &str,
) -> Result<String> {
    let conn = crate::db::conn(db)?;
    let (html, _status) = render_by_id(&conn, templates, post_id, site_url, false, tour_base)?;
    Ok(force_noindex(&html))
}


/// Set every `<meta name="robots" content="…">` to `noindex, nofollow`
/// (published posts carry index directives that must not survive into a
/// preview response).
fn force_noindex(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html;
    while let Some(pos) = rest.find("name=\"robots\"") {
        let Some(tag_start) = rest[..pos].rfind('<') else { break };
        let Some(tag_end) = rest[pos..].find('>').map(|e| pos + e + 1) else { break };
        out.push_str(&rest[..tag_start]);
        out.push_str("<meta name=\"robots\" content=\"noindex, nofollow\"/>");
        rest = &rest[tag_end..];
    }
    out.push_str(rest);
    out
}

pub fn coverage(db: &Path) -> Result<()> {
    let conn = crate::db::conn(db)?;
    let mut stmt = conn.prepare(sql!("render__select_body_json_from"))?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut total = 0u64;
    let mut covered = 0u64;
    let mut by: HashMap<String, u64> = HashMap::new();
    for row in rows {
        let bj = row?;
        if let Ok(v) = serde_json::from_str::<Value>(&bj) {
            count_types(&v, &mut total, &mut covered, &mut by);
        }
    }
    println!("== element coverage ==");
    let mut keys: Vec<_> = by.keys().cloned().collect();
    keys.sort();
    for k in keys {
        let n = by[&k];
        let ok = ELEMENT_TYPES.contains(&k.as_str())
            || k == "normal"
            || k == "wrapper"
            || k == "column";
        println!("{:<24} {:>6} {}", k, n, if ok { "OK" } else { "fallback" });
    }
    let pct = if total == 0 { 0 } else { 100 * covered / total };
    println!("covered {covered}/{total} ({pct}%)");
    Ok(())
}

fn count_types(v: &Value, total: &mut u64, covered: &mut u64, by: &mut HashMap<String, u64>) {
    match v {
        Value::Array(a) => {
            for x in a {
                count_types(x, total, covered, by);
            }
        }
        Value::Object(m) => {
            if let Some(Value::String(t)) = m.get("type") {
                *total += 1;
                *by.entry(t.clone()).or_default() += 1;
                if ELEMENT_TYPES.contains(&t.as_str()) || t == "normal" {
                    *covered += 1;
                }
            }
            for x in m.values() {
                count_types(x, total, covered, by);
            }
        }
        _ => {}
    }
}

pub(crate) fn load_env(dir: &Path) -> Result<Environment<'static>> {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader(dir));
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    env.add_filter("tojson", |v: minijinja::Value| {
        serde_json::to_string(&serde_json::to_value(v).unwrap_or(Value::Null))
            .unwrap_or_else(|_| "null".into())
    });
    Ok(env)
}

/// Media id -> (uploads path, alt) map used by image elements.
pub(crate) fn media_map(conn: &Connection) -> Result<HashMap<i64, (String, String)>> {
    let mut media: HashMap<i64, (String, String)> = HashMap::new();
    let mut stmt = conn.prepare(sql!("render__select_id_path_2"))?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)))?;
    for row in rows {
        let (mid, path, alt) = row?;
        media.insert(mid, (path, alt));
    }
    Ok(media)
}

/// Site-wide data exposed to element templates as `site` so dynamic
/// elements (blog, tour, testimonial, tour_search, custom-menu…) can render
/// without issuing their own queries. Built once per render call.
pub(crate) fn site_data(
    conn: &Connection,
    lang: &str,
    site_url: &str,
    media: &HashMap<i64, (String, String)>,
    tour_base: &str,
) -> Result<JVal> {
    let prefix = if lang == "es" { "/es" } else { "" };
    let url = |kind: &str, slug: &str| page_url(site_url, lang, kind, slug, tour_base);
    let img = |mid: Option<i64>| -> Value {
        mid.and_then(|m| media.get(&m))
            .map(|(p, _)| json!(format!("{site_url}/wp-content/uploads/{p}")))
            .unwrap_or(Value::Null)
    };

    // posts (kind='post') with category slugs
    let mut posts = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, slug, title, COALESCE(excerpt,''), featured_media_id, \
             COALESCE(published_at,'') FROM posts \
             WHERE kind='post' AND lang=?1 AND status='publish' \
             ORDER BY published_at DESC",
        )?;
        let rows = stmt.query_map([lang], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;
        for row in rows {
            let (id, slug, title, excerpt, fid, date) = row?;
            let mut cats = Vec::new();
            let mut cstmt = conn.prepare(
                "SELECT slug, name FROM post_terms WHERE post_id=?1 AND taxonomy='category'",
            )?;
            let crows = cstmt.query_map([id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
            for c in crows.flatten() {
                cats.push(json!({"slug": c.0, "name": c.1}));
            }
            posts.push(json!({
                "id": id, "slug": slug, "title": title, "excerpt": excerpt,
                "date": date, "url": url("post", &slug), "image": img(fid),
                "categories": cats,
            }));
        }
    }
    // tours (kind='tour' joined to tours table) with term slugs per taxonomy.
    // '-copy' slugs are WP duplicated drafts published post-capture — the
    // reference predates them, so exclude to match live selection.
    let mut tours = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT p.id, p.slug, p.title, COALESCE(p.excerpt,''), p.featured_media_id, \
             t.duration_days, COALESCE(t.price_range,''), COALESCE(t.rating,0), t.max_people, \
             COALESCE(p.published_at,'') \
             FROM posts p JOIN tours t ON t.post_id=p.id \
             WHERE p.kind='tour' AND p.lang=?1 AND p.status='publish' \
             AND p.slug NOT LIKE '%-copy' \
             ORDER BY p.menu_order, p.published_at",
        )?;
        let rows = stmt.query_map([lang], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, f64>(7)?,
                r.get::<_, Option<i64>>(8)?,
                r.get::<_, String>(9)?,
            ))
        })?;
        for row in rows {
            let (id, slug, title, excerpt, fid, days, price, rating, max_people, pub_at) = row?;
            let mut terms = Vec::new();
            let mut tstmt = conn.prepare(
                "SELECT taxonomy, slug, name FROM post_terms WHERE post_id=?1",
            )?;
            let trows = tstmt.query_map([id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
            })?;
            for t in trows.flatten() {
                terms.push(json!({"taxonomy": t.0, "slug": t.1, "name": t.2}));
            }
            // tour option meta: ribbon, duration text, departure, price, rating
            let opt = conn
                .query_row(
                    "SELECT meta_value FROM post_meta WHERE post_id=?1 \
                     AND meta_key='tourmaster-tour-option' LIMIT 1",
                    [id],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .and_then(|raw| {
                    crate::php_serialize::unserialize(raw.as_bytes()).ok()
                });
            let opt_s = |k: &str| -> String {
                opt.as_ref()
                    .and_then(|v| v.get(k))
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            // promo ribbon falls back to style term names
            let mut promo_text = opt_s("promo-text");
            if promo_text.is_empty() {
                promo_text = terms
                    .iter()
                    .filter(|t| {
                        t.get("taxonomy").and_then(|x| x.as_str())
                            == Some(if lang == "es" { "estilo" } else { "style" })
                    })
                    .filter_map(|t| t.get("name").and_then(|x| x.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ");
            }
            let promo_bg = opt_s("promo-text-ribbon-background");
            // rating + reviewer count live in tourmaster-tour-rating meta
            let mut review_count: i64 = 0;
            let mut rating = rating;
            if let Ok(raw) = conn.query_row::<String, _, _>(
                "SELECT meta_value FROM post_meta WHERE post_id=?1 \
                 AND meta_key='tourmaster-tour-rating' LIMIT 1",
                [id],
                |r| r.get(0),
            ) {
                if let Ok(v) = crate::php_serialize::unserialize(raw.as_bytes()) {
                    if let Some(sc) = v
                        .get("score")
                        .and_then(|x| x.as_f64().or_else(|| x.as_i64().map(|i| i as f64)))
                    {
                        rating = sc / 2.0;
                    }
                    review_count = v
                        .get("reviewer")
                        .and_then(|x| x.as_i64())
                        .unwrap_or(0);
                }
            }
            // display price: tour-price-text > tourmaster-tour-price > price_range
            let raw_price = {
                let t = opt_s("tour-price-text");
                if !t.is_empty() {
                    t
                } else {
                    conn.query_row::<String, _, _>(
                        "SELECT meta_value FROM post_meta WHERE post_id=?1 \
                         AND meta_key='tourmaster-tour-price' LIMIT 1",
                        [id],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| price.clone())
                }
            };
            // tour permalinks use display_slug (raw slug can be degenerate,
            // e.g. 'x' on 6416) — the value file path + canonical agree on.
            let tslug = crate::render_tour::display_slug(conn, id, &slug, &title);
            tours.push(json!({
                "id": id, "slug": slug, "title": title, "excerpt": excerpt,
                "url": url("tour", &tslug), "image": img(fid),
                "alt": fid.and_then(|f| media.get(&f)).map(|(_, a)| a.clone()),
                "duration_days": days, "price": price, "rating": rating,
                "max_people": max_people, "terms": terms,
                "duration_text": opt_s("duration-text"),
                "departure": opt_s("departure-location"),
                "promo_text": promo_text,
                "promo_color": opt_s("promo-text-ribbon-text-color"),
                "promo_bg": promo_bg,
                "promo_bg_alpha": hex_rgba(&promo_bg, 0.5),
                "price_display": fmt_tour_price(&raw_price),
                "review_count": review_count, "date": pub_at,
            }));
        }
    }

    // testimonials: site-review posts; reviewer name from _submitted meta
    let mut reviews = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, title, COALESCE(body_json,''), COALESCE(excerpt,''), featured_media_id \
             FROM posts WHERE kind='site-review' AND status='publish' \
             ORDER BY published_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
            ))
        })?;
        for row in rows {
            let (id, title, body_json, excerpt, fid) = row?;
            let mut name = title.clone();
            let mut rating: Option<i64> = None;
            let submitted: Option<String> = conn
                .query_row(
                    "SELECT meta_value FROM post_meta WHERE post_id=?1 AND meta_key='_submitted' LIMIT 1",
                    [id],
                    |r| r.get(0),
                )
                .ok();
            if let Some(raw) = submitted {
                if let Ok(v) = crate::php_serialize::unserialize(raw.as_bytes()) {
                    if let Some(n) = v.get("name").and_then(|x| x.as_str()) {
                        if !n.is_empty() {
                            name = n.to_string();
                        }
                    }
                    rating = v
                        .get("rating")
                        .and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())));
                }
            }
            // review text: body_json is a fragment of text-box elements
            let mut content = String::new();
            if let Ok(bj) = serde_json::from_str::<Value>(&body_json) {
                collect_textbox(&bj, &mut content);
            }
            reviews.push(json!({
                "id": id, "name": name, "content": content, "excerpt": excerpt,
                "rating": rating, "image": img(fid),
            }));
        }
    }

    // terms grouped by taxonomy
    let mut terms: Map<String, Value> = Map::new();
    {
        let mut stmt = conn.prepare(
            "SELECT taxonomy, slug, name FROM terms ORDER BY taxonomy, name",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        })?;
        for row in rows {
            let (tax, slug, name) = row?;
            terms
                .entry(tax)
                .or_insert_with(|| Value::Array(vec![]))
                .as_array_mut()
                .unwrap()
                .push(json!({"slug": slug, "name": name}));
        }
    }

    // menus keyed by name, id and wp_term_id; items parented into trees
    let mut menus: Map<String, Value> = Map::new();
    {
        let mut istmt = conn.prepare(
            "SELECT m.id, m.wp_term_id, m.name, m.lang, i.id, i.parent_id, i.ord, \
             i.label, COALESCE(i.url,'') FROM menus m \
             JOIN menu_items i ON i.menu_id=m.id ORDER BY i.ord",
        )?;
        let rows = istmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<i64>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, String>(8)?,
            ))
        })?;
        let mut by_menu: HashMap<i64, (i64, String, Option<String>, Vec<Value>)> = HashMap::new();
        for row in rows {
            let (mid, wpid, name, mlang, iid, parent, ord, label, iurl) = row?;
            let entry = by_menu
                .entry(mid)
                .or_insert_with(|| (wpid, name, mlang, Vec::new()));
            entry.3.push(json!({
                "id": iid, "parent_id": parent.unwrap_or(0), "ord": ord,
                "title": label.unwrap_or_default(), "url": iurl, "children": [],
            }));
        }
        for (mid, (wpid, name, _mlang, flat)) in by_menu {
            let tree = build_menu_tree(flat);
            let v = json!(tree);
            menus.insert(name.clone(), v.clone());
            menus.insert(mid.to_string(), v.clone());
            menus.insert(format!("wp:{wpid}"), v);
        }
    }

    // tourmaster_general options: search page url + month filter list, both
    // consumed by elements/tour_search.html.
    let general: Option<Value> = conn
        .query_row(
            sql!("render__select_value_from_settings"),
            ["tourmaster_general"],
            |r| r.get::<_, String>(0),
        )
        .ok()
        .and_then(|raw| crate::php_serialize::unserialize(raw.as_bytes()).ok());
    let gen_opt = |key: &str| -> Option<String> {
        general
            .as_ref()
            .and_then(|g| g.get(key))
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            .filter(|s| !s.is_empty())
    };

    // tourmaster_get_template_url('search'): permalink of the 'search-page'
    // option resolved to the current language; home url when unset.
    let search_url = {
        let mut sp = gen_opt("search-page").and_then(|s| s.parse::<i64>().ok());
        if lang != "fr" {
            if let Some(sid) = sp {
                let pair: Option<(Option<i64>, Option<i64>)> = conn
                    .query_row(
                        sql!("render__select_fr_post_id_es_post_id"),
                        [sid],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .ok();
                if let Some((f, e)) = pair {
                    if f == Some(sid) {
                        if let Some(e) = e {
                            sp = Some(e);
                        }
                    }
                }
            }
        }
        match sp.and_then(|sid| slug_of(conn, sid)) {
            Some((slug, kind)) => page_url(site_url, lang, &kind, &slug, tour_base),
            None => format!("{site_url}{prefix}/"),
        }
    };

    // 'month' combobox: N months starting this month (PHP
    // search-month-amount + date_i18n('F Y')); days→civil arithmetic, no chrono.
    const MONTHS_FR: [&str; 12] = [
        "Janvier", "Février", "Mars", "Avril", "Mai", "Juin",
        "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre",
    ];
    const MONTHS_ES: [&str; 12] = [
        "Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio",
        "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre",
    ];
    let month_names = if lang == "es" { &MONTHS_ES } else { &MONTHS_FR };
    let month_amount: i64 = gen_opt("search-month-amount")
        .and_then(|s| s.parse().ok())
        .unwrap_or(12)
        .clamp(0, 36);
    let months: Vec<Value> = {
        let days = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() / 86400)
            .unwrap_or(0)) as i64;
        let z = days + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let (mut y, m) = (if mp >= 10 { y + 1 } else { y }, mp + 3 - (if mp >= 10 { 12 } else { 0 }));
        (0..month_amount)
            .map(|i| {
                let mi = m + i;
                y += (mi - 1) / 12;
                let mm = (mi - 1) % 12 + 1;
                json!({"slug": format!("{y:04}-{mm:02}"),
                       "name": format!("{} {}", month_names[(mm - 1) as usize], y)})
            })
            .collect()
    };

    // pages for page-list
    let mut pages = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, slug, title, COALESCE(parent_id,0), COALESCE(menu_order,0) \
             FROM posts WHERE kind='page' AND lang=?1 AND status='publish' \
             ORDER BY menu_order, title",
        )?;
        let rows = stmt.query_map([lang], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })?;
        for row in rows {
            let (id, slug, title, parent_id, ord) = row?;
            pages.push(json!({
                "id": id, "slug": slug, "title": title,
                "url": url("page", &slug), "parent_id": parent_id, "order": ord,
            }));
        }
    }

    Ok(JVal::from_serialize(json!({
        "posts": posts,
        "tours": tours,
        "reviews": reviews,
        "terms": terms,
        "menus": menus,
        "pages": pages,
        "lang": lang,
        "url_prefix": prefix,
        "site_url": site_url,
        "tour_base": tour_base,
        "tour_base_url": format!("{site_url}{prefix}/{tour_base}/"),
        "search_url": search_url,
        "search_months": months,
    })))
}

/// Pull text-box contents out of a body_json fragment (for review posts).
fn collect_textbox(v: &Value, out: &mut String) {
    match v {
        Value::Array(a) => a.iter().for_each(|x| collect_textbox(x, out)),
        Value::Object(m) => {
            if m.get("type").and_then(|t| t.as_str()) == Some("text-box") {
                if let Some(c) = m
                    .get("value")
                    .and_then(|v| v.get("content"))
                    .and_then(|c| c.as_str())
                {
                    if !out.is_empty() {
                        out.push_str("<br/>");
                    }
                    out.push_str(c);
                }
            }
            for x in m.values() {
                collect_textbox(x, out);
            }
        }
        _ => {}
    }
}

/// "#10873b" + alpha -> "rgba(16, 135, 59, 0.5)" for the ribbon corner.
fn hex_rgba(hex: &str, alpha: f64) -> String {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return String::new();
    }
    let byte = |i: usize| -> Option<u8> { u8::from_str_radix(&h[i..i + 2], 16).ok() };
    match (byte(0), byte(2), byte(4)) {
        (Some(r), Some(g), Some(b)) => format!("rgba({r}, {g}, {b}, {alpha})"),
        _ => String::new(),
    }
}

/// Format a tour price string: bare digits -> "1,825 USD", else raw text.
fn fmt_tour_price(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return String::new();
    }
    if t.chars().all(|c| c.is_ascii_digit()) {
        let mut out = String::new();
        for (i, c) in t.chars().enumerate() {
            if i > 0 && (t.len() - i) % 3 == 0 {
                out.push(',');
            }
            out.push(c);
        }
        format!("{out} USD")
    } else {
        t.to_string()
    }
}

/// Nest flat menu items (with parent_id) into a children tree.
fn build_menu_tree(flat: Vec<Value>) -> Vec<Value> {
    let mut nodes: HashMap<i64, Value> = HashMap::new();
    let mut order = Vec::new();
    for it in flat {
        let id = it.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
        order.push(id);
        nodes.insert(id, it);
    }
    fn attach(id: i64, nodes: &mut HashMap<i64, Value>, seen: &mut Vec<i64>) -> Option<Value> {
        if seen.contains(&id) {
            return None;
        }
        seen.push(id);
        let mut node = nodes.get(&id)?.clone();
        let mut children = Vec::new();
        let child_ids: Vec<i64> = nodes
            .iter()
            .filter(|(_, n)| n.get("parent_id").and_then(|p| p.as_i64()) == Some(id))
            .map(|(cid, _)| *cid)
            .collect();
        for cid in child_ids {
            if let Some(c) = attach(cid, nodes, seen) {
                children.push(c);
            }
        }
        node.as_object_mut()
            .unwrap()
            .insert("children".into(), Value::Array(children));
        Some(node)
    }
    let mut seen = Vec::new();
    let mut roots = Vec::new();
    for id in order {
        let is_root = nodes
            .get(&id)
            .and_then(|n| n.get("parent_id").and_then(|p| p.as_i64()))
            .map(|p| p == 0 || !nodes.contains_key(&p))
            .unwrap_or(true);
        if is_root {
            if let Some(n) = attach(id, &mut nodes, &mut seen) {
                roots.push(n);
            }
        }
    }
    roots
}

/// Register per-render template functions: media_url/media_alt/style/pad4.
fn register_helpers(
    env: &mut Environment,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
) {
    let media = Arc::new(media.clone());
    let site = Arc::new(site_url.to_string());
    // Reset the per-render custom-style buffer: a fresh Environment means a
    // fresh page render, so nothing collected by a previous render leaks in.
    let _ = custom_styles_take();
    {
        let media = media.clone();
        let site = site.clone();
        env.add_function("media_url", move |v: JVal| -> JVal {
            let out = val_to_i64(&v)
                .and_then(|i| media.get(&i))
                .map(|(p, _)| format!("{}/wp-content/uploads/{}", site, p))
                .unwrap_or_default();
            JVal::from(out)
        });
    }
    {
        let media = media.clone();
        env.add_function("media_alt", move |v: JVal| -> JVal {
            let out = val_to_i64(&v)
                .and_then(|i| media.get(&i))
                .map(|(_, a)| a.clone())
                .unwrap_or_default();
            JVal::from(out)
        });
    }
    // css_url(v): media id or URL -> url("…") suitable as a style() value.
    {
        let media = media.clone();
        let site = site.clone();
        env.add_function("css_url", move |v: JVal| -> String {
            let s = v.to_string();
            if s.is_empty() {
                return String::new();
            }
            let url = val_to_i64(&v)
                .and_then(|i| media.get(&i))
                .map(|(p, _)| format!("{}/wp-content/uploads/{}", site, p))
                .unwrap_or_else(|| s.clone());
            format!("url(\"{url}\")")
        });
    }
    // style('prop', value, 'prop2', value2, …) -> ' style="prop: v; prop2: v2;"'
    env.add_function("style", move |args: Rest<JVal>| -> String {
        let mut css = String::new();
        let mut i = 0;
        while i + 1 < args.len() {
            let prop = args[i].to_string();
            let val = css_value(&args[i + 1]);
            if !prop.is_empty() && !val.is_empty() {
                css.push_str(&format!("{}: {};", prop, val));
            }
            i += 2;
        }
        if css.is_empty() {
            String::new()
        } else {
            format!(" style=\"{css}\"")
        }
    });
    // pad4({"top":..,"right":..,"bottom":..,"left":..}) -> "t r b l" or ""
    env.add_function("pad4", move |v: JVal| -> String {
        let get = |k: &str| -> String {
            v.get_attr(k)
                .ok()
                .filter(|x| !x.is_undefined())
                .map(|x| x.to_string())
                .unwrap_or_default()
        };
        let t = get("top");
        let r = get("right");
        let b = get("bottom");
        let l = get("left");
        if [t.as_str(), r.as_str(), b.as_str(), l.as_str()]
            .iter()
            .all(|s| s.is_empty() || *s == "0px" || *s == "0")
        {
            String::new()
        } else {
            format!("{t} {r} {b} {l}")
        }
    });
    // uid('gdlr-core-column-') -> 'gdlr-core-column-1', sequential per prefix.
    // Mirrors PHP globals like $gdlr_core_column_id / $gdlr_core_wrapper_id so
    // generated wrapper ids stay deterministic in render order.
    {
        let counters = Arc::new(parking_lot::Mutex::new(HashMap::<String, u64>::new()));
        env.add_function("uid", move |v: JVal| -> String {
            let prefix = v.to_string();
            let mut m = counters.lock();
            let n = m.entry(prefix.clone()).or_insert(0);
            *n += 1;
            format!("{prefix}{n}")
        });
    }
    // custom_style(css): buffer element custom CSS (column/wrapper custom
    // styles) for the single aggregated <style> block — WP's
    // gdlr_core_insert_style behavior. Templates call it inside {% if %}; it
    // renders nothing inline.
    env.add_function("custom_style", move |v: JVal| -> String {
        let s = v.to_string();
        if !s.is_empty() {
            CUSTOM_STYLES.with(|c| c.borrow_mut().push(s));
        }
        String::new()
    });
    // gdlr_css('prop', value, ...) -> CSS declarations emulating PHP
    // gdlr_core_esc_style: 4-side {top,right,bottom,left,settings} maps emit
    // shorthand or per-side declarations; corner radius maps emit
    // (moz/webkit-prefixed) border-radius; [color,opacity] pairs emit rgba();
    // gradient props emit linear-gradient(). gdlr_css_imp appends !important.
    env.add_function("gdlr_css", move |args: Rest<JVal>| -> String {
        gdlr_css_impl(&args, false)
    });
    env.add_function("gdlr_css_imp", move |args: Rest<JVal>| -> String {
        gdlr_css_impl(&args, true)
    });
    // gdlr_rgba(color, opacity) -> 'rgba(r,g,b,a)' for hex colors; raw color
    // or '' otherwise.
    env.add_function("gdlr_rgba", move |c: JVal, o: JVal| -> String {
        let color = c.to_string();
        if color.is_empty() {
            return String::new();
        }
        let a: f64 = o.to_string().parse().unwrap_or(1.0);
        match parse_hex(&color) {
            Some((r, g, b)) => format!("rgba({r},{g},{b},{})", fmt_alpha(a)),
            None => color,
        }
    });
}

fn fmt_alpha(a: f64) -> String {
    if a.fract() == 0.0 {
        format!("{}", a as i64)
    } else {
        format!("{a}")
    }
}

fn parse_hex(color: &str) -> Option<(u8, u8, u8)> {
    let h = color.strip_prefix('#')?;
    let parse = |s: &str| u8::from_str_radix(s, 16).ok();
    match h.len() {
        6 => Some((parse(&h[0..2])?, parse(&h[2..4])?, parse(&h[4..6])?)),
        3 => Some((
            parse(&h[0..1].repeat(2))?,
            parse(&h[1..2].repeat(2))?,
            parse(&h[2..3].repeat(2))?,
        )),
        _ => None,
    }
}

fn s_empty(x: &Value) -> bool {
    match x {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        _ => false,
    }
}

fn s_str(x: &Value) -> String {
    match x {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

fn rgba_of(c: Option<&Value>, op: Option<&Value>) -> String {
    let color = c.map(s_str).unwrap_or_default();
    if color.is_empty() {
        return String::new();
    }
    let a: f64 = op.and_then(|v| s_str(v).parse().ok()).unwrap_or(1.0);
    match parse_hex(&color) {
        Some((r, g, b)) => format!("rgba({r},{g},{b},{})", fmt_alpha(a)),
        None => color,
    }
}

fn radius_shorthand(vals: &[String], bang: &str) -> String {
    let j = vals.join(" ");
    format!("border-radius: {j}{bang};-moz-border-radius: {j}{bang};-webkit-border-radius: {j}{bang};")
}

/// Emit CSS declaration(s) for one esc_style-style (prop, value) pair.
fn gdlr_decl(prop: &str, v: &Value, bang: &str) -> String {
    match v {
        Value::Object(m) => {
            let get = |k: &str| m.get(k).map(s_str).unwrap_or_default();
            let settings = get("settings");
            if prop == "border-radius" && m.contains_key("t-left") {
                // corner-map shape: t-left/t-right/b-right/b-left
                if settings == "link" {
                    let t = get("t-left");
                    if t.is_empty() {
                        return String::new();
                    }
                    return radius_shorthand(&[t.clone(), t.clone(), t.clone(), t], bang);
                }
                let cs: Vec<String> = ["t-left", "t-right", "b-right", "b-left"]
                    .iter()
                    .map(|k| get(k))
                    .collect();
                if cs.iter().all(|s| s.is_empty()) {
                    return String::new();
                }
                let vals: Vec<String> = cs
                    .iter()
                    .map(|s| if s.is_empty() { "0px".into() } else { s.clone() })
                    .collect();
                return radius_shorthand(&vals, bang);
            }
            let sides: Vec<String> = ["top", "right", "bottom", "left"].iter().map(|k| get(k)).collect();
            if sides.iter().all(|s| s.is_empty()) {
                return String::new();
            }
            if settings == "link" {
                let t = &sides[0];
                if t.is_empty() {
                    return String::new();
                }
                if prop == "border-radius" {
                    return radius_shorthand(&[t.clone(), t.clone(), t.clone(), t.clone()], bang);
                }
                return format!("{prop}: {t} {t} {t} {t}{bang};");
            }
            if sides.iter().all(|s| !s.is_empty()) {
                if prop == "border-radius" {
                    return radius_shorthand(&sides, bang);
                }
                return format!("{prop}: {}{bang};", sides.join(" "));
            }
            let mut out = String::new();
            for (k, val) in ["top", "right", "bottom", "left"].iter().zip(sides.iter()) {
                if !val.is_empty() {
                    out.push_str(&format!("{prop}-{k}: {val}{bang};"));
                }
            }
            out
        }
        Value::Array(a) => {
            if (prop == "gradient" || prop == "gradient-v") && a.iter().all(|x| x.is_array()) {
                let stops: Vec<String> = a.iter().map(|p| rgba_of(p.get(0), p.get(1))).collect();
                if stops.iter().all(|s| s.is_empty()) {
                    return String::new();
                }
                let deg = if prop == "gradient-v" { "180deg" } else { "90deg" };
                return format!("background: linear-gradient({deg},{}){bang};", stops.join(","));
            }
            if a.len() == 2 {
                let c = rgba_of(a.get(0), a.get(1));
                if c.is_empty() {
                    return String::new();
                }
                return format!("{prop}: {c}{bang};");
            }
            String::new()
        }
        // PHP esc_style prints scalar values with a trailing space ('prop: v ;')
        Value::String(s) => {
            if s.is_empty() {
                String::new()
            } else if bang.is_empty() {
                format!("{prop}: {s} ;")
            } else {
                format!("{prop}: {s}{bang};")
            }
        }
        Value::Number(n) => {
            if bang.is_empty() {
                format!("{prop}: {n} ;")
            } else {
                format!("{prop}: {n}{bang};")
            }
        }
        _ => String::new(),
    }
}

fn gdlr_css_impl(args: &[JVal], important: bool) -> String {
    let bang = if important { " !important" } else { "" };
    let mut css = String::new();
    let mut i = 0;
    while i + 1 < args.len() {
        let prop = args[i].to_string();
        let v = serde_json::to_value(&args[i + 1]).unwrap_or(Value::Null);
        if !s_empty(&v) {
            css.push_str(&gdlr_decl(&prop, &v, bang));
        }
        i += 2;
    }
    css
}

fn val_to_i64(v: &JVal) -> Option<i64> {
    if let Ok(n) = i64::try_from(v.clone()) {
        Some(n)
    } else {
        v.to_string().parse().ok()
    }
}

/// Stringify a style value; 4-side maps become "t r b l".
fn css_value(v: &JVal) -> String {
    if v.is_undefined() || v.is_none() {
        return String::new();
    }
    if v.as_object().is_some() {
        let mut parts = Vec::new();
        for k in ["top", "right", "bottom", "left"] {
            parts.push(
                v.get_attr(k)
                    .ok()
                    .filter(|x| !x.is_undefined())
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| "0px".into()),
            );
        }
        return parts.join(" ");
    }
    v.to_string()
}


#[allow(dead_code)]
/// Render a body_json fragment (array or single node) to element HTML.
/// Used by both the static generator and the builder preview endpoint.
pub(crate) fn render_fragment(
    templates: &Path,
    v: &Value,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    tour_base: &str,
) -> Result<String> {
    let mut env = load_env(templates)?;
    register_helpers(&mut env, media, site_url);
    let site = JVal::from_serialize(json!({}));
    let post = JVal::from_serialize(json!({}));
    render_nodes(&mut env, v, media, site_url, &site, "fr", &post, None, false, tour_base)
}

/// render_fragment with a live DB so dynamic elements (blog, tour,
/// testimonial, tour_search, custom-menu…) get a populated `site` context.
pub(crate) fn render_fragment_db(
    conn: &Connection,
    templates: &Path,
    v: &Value,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    lang: &str,
    tour_base: &str,
) -> Result<String> {
    let mut env = load_env(templates)?;
    register_helpers(&mut env, media, site_url);
    let site = site_data(conn, lang, site_url, media, tour_base)?;
    let post = JVal::from_serialize(json!({}));
    render_nodes(&mut env, v, media, site_url, &site, lang, &post, Some(conn), false, tour_base)
}

thread_local! {
    static RENDER_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// render_nodes wrapper: tracks recursion depth so collected element custom
/// styles are emitted once, as a single leading <style> block, by the
/// outermost call (WP prints them in <head>; we keep them ahead of the body
/// markup since this function only returns the body fragment).
fn render_nodes(
    env: &mut Environment,
    v: &Value,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    site: &JVal,
    lang: &str,
    post: &JVal,
    conn: Option<&Connection>,
    top_level: bool,
    tour_base: &str,
) -> Result<String> {
    RENDER_DEPTH.with(|d| d.set(d.get() + 1));
    let result = render_nodes_inner(env, v, media, site_url, site, lang, post, conn, top_level, tour_base);
    let depth = RENDER_DEPTH.with(|d| {
        d.set(d.get() - 1);
        d.get()
    });
    match result {
        Ok(html) if depth == 0 => {
            let styles = custom_styles_take();
            if styles.is_empty() {
                Ok(html)
            } else {
                Ok(format!("<style>{}</style>{html}", styles.concat()))
            }
        }
        r => r,
    }
}

fn render_nodes_inner(
    env: &mut Environment,
    v: &Value,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    site: &JVal,
    lang: &str,
    post: &JVal,
    conn: Option<&Connection>,
    top_level: bool,
    tour_base: &str,
) -> Result<String> {
    match v {
        Value::Array(a) => {
            let mut out = String::new();
            // PHP page-builder.php: column widths accumulate within an item
            // stream; a column is 'first' when it opens a 60-wide row.
            let mut col_count: i64 = 0;
            // Non-wrapper runs (leaf elements and column wrappers) are grouped
            // into <div class="gdlr-core-pbf-section"><div class="gdlr-core-
            // pbf-section-container gdlr-core-container clearfix">…</div></div>
            // exactly like PHP page-builder.php emits for sections.
            let mut section_open = false;
            let close_section = |out: &mut String, open: &mut bool| {
                if *open {
                    out.push_str("</div></div>");
                    *open = false;
                }
            };
            for x in a {
                match x {
                    Value::Object(m) => {
                        let typ = m.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        let is_wrapper = matches!(typ, "background" | "normal" | "sidebar");
                        if is_wrapper {
                            close_section(&mut out, &mut section_open);
                            out.push_str(&render_node(env, m, media, site_url, site, lang, post, false, conn, tour_base)?);
                            col_count = 0;
                            continue;
                        }
                        if top_level && !section_open {
                            out.push_str("<div class=\"gdlr-core-pbf-section\" ><div class=\"gdlr-core-pbf-section-container gdlr-core-container clearfix\" >");
                            section_open = true;
                        }
                        let mut first = false;
                        if typ == "column" || typ == "flexbox" {
                            let w: i64 = m
                                .get("column")
                                .and_then(|c| {
                                    c.as_str()
                                        .and_then(|s| s.parse().ok())
                                        .or_else(|| c.as_i64())
                                })
                                .unwrap_or(0);
                            if col_count == 0 {
                                first = true;
                                col_count = w;
                            } else {
                                col_count += w;
                                if col_count > 60 {
                                    first = true;
                                    col_count = w;
                                }
                            }
                        }
                        let html = render_node(env, m, media, site_url, site, lang, post, first, conn, tour_base)?;
                        // every leaf item rides in <div class="gdlr-core-pbf-element">;
                        // columns carry their own wrapper.
                        if typ == "column" || typ == "flexbox" {
                            out.push_str(&html);
                        } else {
                            out.push_str(&format!("<div class=\"gdlr-core-pbf-element\" >{html}</div>"));
                        }
                    }
                    _ => out.push_str(&render_nodes(env, x, media, site_url, site, lang, post, conn, false, tour_base)?),
                }
            }
            close_section(&mut out, &mut section_open);
            Ok(out)
        }
        Value::Object(m) => render_node(env, m, media, site_url, site, lang, post, false, conn, tour_base),
        _ => Ok(String::new()),
    }
}
fn render_node(
    env: &mut Environment,
    m: &Map<String, Value>,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    site: &JVal,
    lang: &str,
    post: &JVal,
    first_col: bool,
    conn: Option<&Connection>,
    tour_base: &str,
) -> Result<String> {
    let typ = m.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
    let children_val = m.get("items").cloned().unwrap_or(Value::Array(vec![]));
    let children = render_nodes(env, &children_val, media, site_url, site, lang, post, conn, false, tour_base)?;
    let mut opts = match m.get("value") {
        Some(Value::Object(o)) => o.clone(),
        _ => Map::new(),
    };
    if let Some(c) = m.get("column") {
        opts.insert("column".into(), c.clone());
    }
    if first_col {
        opts.insert("first-column".into(), json!(true));
    }
    let column = opts
        .get("column")
        .and_then(|c| c.as_str().map(|s| s.to_string()).or_else(|| c.as_i64().map(|n| n.to_string())))
        .unwrap_or_else(|| m.get("column").and_then(|c| c.as_str()).unwrap_or("60").to_string());

    if typ == "image" {
        if let Some(id) = opts.get("image").and_then(|x| match x {
            Value::Number(n) => n.as_i64(),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }) {
            if let Some((path, alt)) = media.get(&id) {
                opts.insert("src".into(), json!(format!("{site_url}/wp-content/uploads/{path}")));
                if opts.get("alt").and_then(|a| a.as_str()).unwrap_or("").is_empty() {
                    opts.insert("alt".into(), json!(alt));
                }
            }
        }
    }
    if typ == "text-box" {
        if let Some(Value::String(s)) = opts.get_mut("content") {
            // Saved via TinyMCE: content may be entity-encoded — WP decodes on
            // output; mirror that so `<h1>` isn't printed as `&lt;h1&gt;`.
            if s.contains("&lt;") || s.contains("&gt;") {
                *s = crate::admin::gdlr_pb::html_decode_pub(s);
            }
            // gdlr_core_content_filter → do_shortcode + oembed for the small
            // set this site actually uses.
            if s.contains('[') {
                *s = expand_shortcodes(s, site_url);
            }
            // gdlr_core_content_filter also unwraps <p> that encloses a
            // single block-level heading (TinyMCE artifact).
            for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
                for (pre, post) in [
                    (format!("<p><{tag} "), format!("<{tag} ")),
                    (format!("<p><{tag}>"), format!("<{tag}>")),
                ] {
                    if s.contains(&pre) {
                        // strip wrapper only when </hN></p> matches symmetric
                        let close = format!("</{tag}></p>");
                        if s.contains(&close) {
                            *s = s.replace(&pre, &post).replace(&close, &format!("</{tag}>"));
                        }
                    }
                }
            }
        }
    }
    if typ == "sidebar" {
        // Sidebar wrapper: 'sidebar' is positional (left|right|both); the
        // registered sidebar ids live in 'sidebar-left'/'sidebar-right'
        // comboboxes (wrappers.json). Each emitted side column renders
        // the assigned sidebar's widgets via the registry — custom
        // sidebars carry no wrapper (phase-3 decision, documented).
        let sidebar = {
            let v = opts
                .get("sidebar")
                .and_then(|v| v.as_str())
                .unwrap_or("right");
            if v.is_empty() { "right" } else { v }
        };
        let width: i64 = opts
            .get("sidebar-width")
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            .and_then(|s| s.trim_end_matches("px").parse().ok())
            .unwrap_or(20);
        let mut content_w = 60;
        if sidebar == "left" || sidebar == "right" {
            content_w -= width;
        }
        if sidebar == "both" {
            content_w -= 2 * width;
        }
        let mut o = format!(
            "<div class=\"gdlr-core-pbf-sidebar-container clearfix gdlr-core-line-height-0 gdlr-core-js\" >"
        );
        for side in ["left", "right"] {
            if sidebar != side && sidebar != "both" {
                continue;
            }
            let sb_id = opts
                .get(&format!("sidebar-{side}"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let sb_html = if sb_id.is_empty() {
                String::new()
            } else {
                conn.map(|c| {
                    crate::widgets::render_sidebar(c, sb_id, site_url, lang, "", tour_base)
                        .unwrap_or_default()
                })
                .unwrap_or_default()
            };
            o.push_str(&format!(
                "<div class=\"gdlr-core-pbf-sidebar-{side} gdlr-core-column-extend-{side}  gdlr-core-column-{width} gdlr-core-pbf-sidebar-padding  gdlr-core-line-height\" ><div class=\"gdlr-core-sidebar-inner-bg gdlr-core-item-mglr\"></div>{sb_html}</div>"
            ));
        }
        o.push_str(&format!(
            "<div class=\" gdlr-core-column-{content_w} gdlr-core-pbf-sidebar-padding gdlr-core-line-height\" ><div class=\"gdlr-core-page-builder-wrapper-sidebar-content-inner clearfix\" >{children}</div></div></div>"
        ));
        return Ok(o);
    }
    if typ == "blog" {
        // WP_Query equivalent: posts in the chosen category (or tag) for the
        // page language; WP tax_query also covers child terms — we have no
        // parent column, so an empty match falls back to latest posts, which
        // is what the reference homepage shows anyway.
        let posts = conn.and_then(|c| blog_posts(c, &opts, lang, media, site_url, tour_base).ok()).unwrap_or_default();
        let children = posts;
        let opts_v = JVal::from_serialize(&opts);
        return env
            .get_template("elements/blog.html")?
            .render(context! {
                opts => opts_v,
                children => children,
                type => typ,
                column => column,
                site => site.clone(),
                site_url => site_url,
                lang => lang,
                pdb => "40px",
                post => post.clone(),
            })
            .map_err(|e| anyhow::anyhow!("element {typ}: {e}"));
    }
    if matches!(typ, "accordion" | "toggle-box") {
        // gdlr_core_content_filter also runs on tab contents — expand the
        // same shortcode subset so [caption] becomes a <figure>.
        if let Some(Value::Array(tabs)) = opts.get_mut("tabs") {
            for tab in tabs.iter_mut() {
                if let Some(Value::String(s)) = tab.get_mut("content") {
                    if s.contains('[') {
                        *s = expand_shortcodes(s, site_url);
                    }
                }
            }
        }
    }
    let mut gallery_items = JVal::from_serialize(json!([]));
    if typ == "gallery" {
        // gallery ids -> {path, alt, caption, title, w, h} for the template's
        // fitrows grid (grid-no-space: lightgallery link + zoom overlay icon).
        let ids: Vec<i64> = opts
            .get("gallery")
            .and_then(|g| g.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.get("id"))
                    .filter_map(|x| match x {
                        Value::Number(n) => n.as_i64(),
                        Value::String(s) => s.parse().ok(),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut items = Vec::new();
        for id in &ids {
            items.push(
                conn.and_then(|c| media_info(c, *id))
                    .or_else(|| {
                        media.get(id).map(|(p, alt)| {
                            json!({"path": p, "alt": alt, "caption": "", "title": "", "w": 0, "h": 0})
                        })
                    })
                    .unwrap_or_else(|| json!({})),
            );
        }
        gallery_items = JVal::from_serialize(items);
    }

    let tpl_file = if ELEMENT_TYPES.contains(&typ) {
        format!("elements/{typ}.html")
    } else if typ == "normal" {
        "elements/background.html".into()
    } else {
        "elements/fallback.html".into()
    };
    let opts_v = JVal::from_serialize(&opts);
    env.get_template(&tpl_file)?
        .render(context! {
            opts => opts_v,
            children => children,
            type => typ,
            column => column,
            site => site.clone(),
            site_url => site_url,
            lang => lang,
            pdb => "40px",
            post => post.clone(),
            gallery_items => gallery_items,
        })
        .map_err(|e| anyhow::anyhow!("element {typ}: {e}"))
}

fn translation_urls(conn: &Connection, id: i64, site: &str, tour_base: &str) -> Result<Vec<(String, String)>> {
    let cfg = crate::lang::config(conn);
    let group: Option<String> = conn
        .query_row(
            "SELECT group_id FROM post_translation_members WHERE post_id=?1 LIMIT 1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(group) = group else { return Ok(Vec::new()); };
    let mut stmt = conn.prepare(
        "SELECT lang, post_id FROM post_translation_members WHERE group_id=?1 ORDER BY lang",
    )?;
    let rows = stmt
        .query_map([group], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut out = Vec::new();
    for (lang, pid) in rows {
        if let Some((slug, kind)) = slug_of(conn, pid) {
            out.push((lang.clone(), crate::lang::page_url(&cfg, site, &lang, &kind, &slug, tour_base)));
        }
    }
    Ok(out)
}

fn hreflang_links(alternates: &[(String, String)]) -> String {
    alternates
        .iter()
        .map(|(lang, url)| format!("<link rel=\"alternate\" href=\"{}\" hreflang=\"{}\" />", esc_attr(url), esc_attr(lang)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn slug_of(conn: &Connection, id: i64) -> Option<(String, String)> {
    conn.query_row(
        sql!("render__select_slug_kind"),
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .ok()
}

pub(crate) fn page_url(site: &str, lang: &str, kind: &str, slug: &str, tour_base: &str) -> String {
    let prefix = if lang == "es" { "/es" } else { "" };
    match kind {
        "tour" => format!("{site}{prefix}/{tour_base}/{slug}/"),
        _ => format!("{site}{prefix}/{slug}/"),
    }
}

/* ================= shared head/chrome context (Phase 2) ================= */

/// Full media row for an attachment id (featured/gallery): (path, alt,
/// mime, width, height). Used by the og:image block and the gallery element.
pub(crate) fn media_detail(
    conn: &Connection,
    id: i64,
) -> Result<Option<(String, String, String, i64, i64)>> {
    Ok(conn
        .query_row(sql!("render__select_media_og"), [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .ok())
}

/// RankMath-style robots meta. `rank_math_robots` post_meta holds a PHP
/// array like `a:2:{i:0;s:5:"index";i:1;s:8:"nofollow";}`; WP renders it
/// follow/nofollow first, then index, then the fixed snippet suffix.
fn robots_meta(ser: &str) -> String {
    let mut items: Vec<String> = Vec::new();
    if let Ok(Value::Array(a)) = crate::php_serialize::unserialize(ser.as_bytes()) {
        items = a
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
    }
    let has = |n: &str| items.iter().any(|x| x == n);
    let follow = if has("nofollow") { "nofollow" } else { "follow" };
    let index = if has("noindex") {
        "noindex"
    } else {
        "index"
    };
    format!(
        "{follow}, {index}, max-snippet:-1, max-video-preview:-1, max-image-preview:large"
    )
}

/// RankMath `rank_math_schema_{Type}` post_meta — PHP-serialized schema.org
/// template with `%placeholder%` tokens. Substitute the known tokens with
/// post fields, emit as JSON-LD. Unknown tokens resolve to empty strings;
/// the `metadata` block is RankMath-internal and dropped.
fn post_jsonld(
    conn: &Connection,
    id: i64,
    title: &str,
    desc: &str,
    published_at: &str,
    modified_at: &str,
    img_url: &str,
    site_url: &str,
) -> Option<String> {
    let types = ["Article", "BlogPosting", "VideoObject", "Product", "FAQPage"];
    let mut raw = String::new();
    for t in types {
        let v: String = conn
            .query_row(
                "SELECT COALESCE(meta_value,'') FROM post_meta WHERE post_id=?1 AND meta_key=?2 LIMIT 1",
                rusqlite::params![id, format!("rank_math_schema_{t}")],
                |r| r.get(0),
            )
            .unwrap_or_default();
        if !v.is_empty() {
            raw = v;
            break;
        }
    }
    if raw.is_empty() {
        return None;
    }
    let value = crate::php_serialize::unserialize(raw.as_bytes()).ok()?;
    let author_name = conn
        .query_row(
            "SELECT value FROM settings WHERE key='site_name' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "rpress".into());
    let json = schema_to_json(&value, &|tok: &str| -> String {
        match tok {
            "seo_title" | "title" => title.to_string(),
            "seo_description" | "description" | "excerpt" => desc.to_string(),
            t if t.starts_with("date") => iso8601(published_at),
            t if t.starts_with("modified") => iso8601(modified_at),
            "post_thumbnail" | "image" => img_url.to_string(),
            "name" | "author" => author_name.clone(),
            "url" | "site_url" => site_url.to_string(),
            "keywords" | "categories" | "tags" => String::new(),
            _ => String::new(),
        }
    });
    serde_json::to_string(&json).ok()
}

/// PHP Value → serde_json::Value, walking `%token%` substitutions. Drops
/// the RankMath `metadata` block — it's bookkeeping, not schema output.
fn schema_to_json(v: &Value, sub: &dyn Fn(&str) -> String) -> Value {
    use serde_json::Map;
    match v {
        Value::Object(m) => {
            let mut out = Map::new();
            for (k, val) in m {
                if k == "metadata" {
                    continue;
                }
                out.insert(k.clone(), schema_to_json(val, sub));
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| schema_to_json(x, sub)).collect()),
        Value::String(s) => {
            let mut out = s.clone();
            let mut i = 0usize;
            while let Some(start) = out[i..].find('%') {
                let s0 = i + start;
                match out[s0 + 1..].find('%') {
                    Some(e0) => {
                        let e = s0 + 1 + e0;
                        let tok = out[s0 + 1..e].to_string();
                        let rep = sub(&tok);
                        out = format!("{}{}{}", &out[..s0], rep, &out[e + 1..]);
                        i = s0 + rep.len();
                    }
                    None => break,
                }
            }
            Value::String(out)
        }
        other => other.clone(),
    }
}

/// Strip HTML tags, collapse whitespace; entities left as-is (WP excerpt
/// behaves the same for our purposes since &amp; etc. are rare here).
fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Auto description fallback (RankMath when no seo_desc/excerpt): first
/// paragraph after the first heading, else the first text block; capped at
/// 30 words (the observed RankMath excerpt length).
fn auto_desc(body_html: &str) -> String {
    let h_end = body_html.find("</h").map(|i| {
        body_html[i..]
            .find('>')
            .map(|o| i + o + 1)
            .unwrap_or(i)
    });
    let hay = h_end.map(|i| &body_html[i..]).unwrap_or(body_html);
    // first <p>…</p> if present, else the first non-empty line
    let text = if let Some(ps) = hay.find("<p") {
        let rest = &hay[ps..];
        let start = rest.find('>').map(|o| o + 1).unwrap_or(0);
        let end = rest[start..].find("</p").map(|o| start + o).unwrap_or(rest.len());
        strip_tags(&rest[start..end])
    } else {
        let line = hay.split('\n').find(|l| !strip_tags(l).is_empty()).unwrap_or("");
        strip_tags(line)
    };
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() > 30 {
        words[..30].join(" ")
    } else {
        words.join(" ")
    }
}

/// Rough word count of rendered content for the reading-time label.
fn word_count(html: &str) -> usize {
    strip_tags(html).split_whitespace().count()
}
/// Reduced HTML→markdown for the `.md` content-negotiation surface
/// (`/{slug}/index.md` + Accept negotiation in `public_page`). Covers the
/// block shapes the post body produces (headings, paragraphs, lists,
/// quotes, tables, links, images); anything else degrades to plain text.
/// This is a one-pass converter, not an HTML parser — malformed input can
/// only leak stray tags, never panic.
pub fn html_to_markdown(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    let mut buf = String::new();
    fn take_until<'a>(hay: &'a str, needle: &str) -> (String, &'a str) {
        match hay.find(needle) {
            Some(i) => (hay[..i].to_string(), &hay[i + needle.len()..]),
            None => (hay.to_string(), ""),
        }
    }
    let mut cur_text = String::new();
    macro_rules! flush_text {
        () => {
            let t = inline_md(&cur_text);
            if !t.is_empty() {
                buf.push_str(&t);
                buf.push_str("\n\n");
            }
            cur_text.clear();
        };
    }
    while !rest.is_empty() {
        if let Some(i) = rest.find('<') {
            cur_text.push_str(&rest[..i]);
            rest = &rest[i..];
            // tag name
            let name_end = rest.find(|c: char| c.is_whitespace() || c == '>' || c == '/').unwrap_or(rest.len());
            let tag = rest[1..name_end.min(rest.len())].to_lowercase();
            // attrs sit between name_end and the tag's '>' — skip to it
            // before grabbing the inner span so they can't leak into text.
            let gt_rel = rest[name_end..].find('>').map(|i| name_end + i + 1).unwrap_or(rest.len());
            match tag.as_str() {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    flush_text!();
                    let level: usize = tag[1..].parse().unwrap_or(2);
                    let (inner, r) = take_until(&rest[gt_rel..], &format!("</{tag}>"));
                    rest = r;
                    let hashes = "#".repeat(level.min(6));
                    buf.push_str(&format!("{hashes} {}\n\n", inline_md(&inner)));
                }
                "p" => {
                    let (inner, r) = take_until(&rest[gt_rel..], "</p>");
                    rest = r;
                    cur_text.push_str(&inner);
                    flush_text!();
                }
                "br" => {
                    cur_text.push(' ');
                    let (_, r) = take_until(rest, ">");
                    rest = r;
                }
                "ul" | "ol" => {
                    flush_text!();
                    let ordered = tag == "ol";
                    let (inner, r) = take_until(&rest[gt_rel..], &format!("</{tag}>"));
                    rest = r;
                    let mut n = 0usize;
                    let mut items = inner.as_str();
                    while let Some(li) = items.find("<li") {
                        let (body, r2) = take_until(&items[li..], "</li>");
                        let body = body.split_once('>').map(|(_, b)| b).unwrap_or("");
                        n += 1;
                        let marker = if ordered { format!("{n}.") } else { "-".into() };
                        buf.push_str(&format!("{marker} {}\n", inline_md(body)));
                        items = r2;
                    }
                    buf.push('\n');
                }
                "blockquote" => {
                    flush_text!();
                    let (inner, r) = take_until(&rest[gt_rel..], "</blockquote>");
                    rest = r;
                    for line in html_to_markdown(&inner).lines() {
                        buf.push_str(&format!("> {line}\n"));
                    }
                    buf.push('\n');
                }
                "table" => {
                    flush_text!();
                    let (inner, r) = take_until(&rest[gt_rel..], "</table>");
                    rest = r;
                    let mut rows: Vec<Vec<String>> = Vec::new();
                    let mut rrest = inner.as_str();
                    while let Some(tr) = rrest.find("<tr") {
                        let (row_html, r2) = take_until(&rrest[tr..], "</tr>");
                        rrest = r2;
                        let mut cells = Vec::new();
                        let mut crest = row_html.as_str();
                        while let Some(td) = crest.find("<td").into_iter().chain(crest.find("<th")).min() {
                            let (cell, r3) = take_until(&crest[td..], if crest[td..].starts_with("<th") { "</th>" } else { "</td>" });
                            let cell = cell.split_once('>').map(|(_, c)| c).unwrap_or("");
                            cells.push(inline_md(cell).replace('|', "\\|").replace('\n', " "));
                            crest = r3;
                        }
                        if !cells.is_empty() { rows.push(cells); }
                    }
                    if !rows.is_empty() {
                        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0);
                        for (i, row) in rows.iter().enumerate() {
                            let mut padded = row.clone();
                            padded.resize(w, String::new());
                            buf.push_str(&format!("| {} |\n", padded.join(" | ")));
                            if i == 0 {
                                buf.push_str(&format!("|{}|\n", "|".repeat(w + 1).replace("||", "| --- ")));
                            }
                        }
                        buf.push('\n');
                    }
                }
                "figure" | "div" | "section" | "article" | "header" | "footer" | "main" | "aside" => {
                    let (inner, r) = take_until(&rest[gt_rel..], &format!("</{tag}>"));
                    rest = r;
                    cur_text.push_str(&inner);
                    flush_text!();
                }
                "script" | "style" | "noscript" | "iframe" | "form" | "button" | "select" | "input" | "textarea" => {
                    // skip entire element — not content
                    let (_, r) = take_until(&rest[gt_rel..], &format!("</{tag}>"));
                    rest = r;
                }
                "a" => {
                    // keep inner text; inline_md handles <a> if we left it
                    let gt = rest.find('>').map(|i| i + 1).unwrap_or(rest.len());
                    rest = &rest[gt..];
                }
                "img" => {
                    let (attrs, r) = take_until(rest, ">");
                    rest = r;
                    if let Some(alt) = extract_attr(&attrs, "alt") {
                        if !alt.is_empty() { cur_text.push_str(&alt); }
                    }
                }
                _ => {
                    // unknown/inline tag → drop the tag only
                    let gt = rest.find('>').map(|i| i + 1).unwrap_or(rest.len());
                    rest = &rest[gt..];
                }
            }
        } else {
            cur_text.push_str(rest);
            break;
        }
    }
    flush_text!();
    out.push_str(&buf);
    // collapse 3+ blank lines
    let mut collapsed = String::with_capacity(out.len());
    let mut blank = 0;
    for line in out.lines() {
        if line.trim().is_empty() {
            blank += 1;
            if blank <= 1 { collapsed.push('\n'); }
        } else {
            blank = 0;
            collapsed.push_str(line);
            collapsed.push('\n');
        }
    }
    collapsed.trim().to_string()
}

/// Inline-level conversion inside a block: <strong>/<b> → **, <em>/<i> → *,
/// <a href>text</a> → [text](href), <img> → ![alt](src), <code> → `code`,
/// entities decoded minimally.
fn inline_md(s: &str) -> String {
    let mut t = s.to_string();
    // links before generic tag-strip so we keep the href
    while let Some(a) = t.find("<a ") {
        let gt = t[a..].find('>').map(|i| a + i + 1).unwrap_or(t.len());
        let attrs = t[a + 3..gt.saturating_sub(1)].to_string();
        let (inner, rest) = take_once(&t[gt..], "</a>");
        let href = extract_attr(&attrs, "href").unwrap_or_default();
        let label = strip_tags(&inner);
        t = format!("{}{}", &t[..a], if href.is_empty() { label } else { format!("[{label}]({href})") }) + rest;
    }
    while let Some(i) = t.find("<img") {
        let (attrs, rest) = take_once(&t[i..], ">");
        let alt = extract_attr(&attrs, "alt").unwrap_or_default();
        let src = extract_attr(&attrs, "src").unwrap_or_default();
        t = format!("{}{}{}", &t[..i], if src.is_empty() { alt } else { format!("![{alt}]({src})") }, rest);
    }
    for (tag, rep) in [("strong", "**"), ("b", "**"), ("em", "*"), ("i", "*"), ("code", "`"), ("mark", "==")] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        while let Some(a) = t.find(&open) {
            let gt = t[a..].find('>').map(|i| a + i + 1).unwrap_or(t.len());
            let (inner, rest) = take_once(&t[gt..], &close);
            let inner_txt = strip_tags(inner);
            t = format!("{}{}{}{}", &t[..a], rep, inner_txt, rep) + rest;
        }
    }
    // drop residual tags, decode common entities
    strip_tags(&t)
        .replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
        .replace("&quot;", "\"").replace("&#039;", "'").replace("&nbsp;", " ")
        .replace("&eacute;", "é").replace("&egrave;", "è").replace("&agrave;", "à")
        .replace("&ccedil;", "ç").replace("&ocirc;", "ô").replace("&icirc;", "î")
        .replace("&ecirc;", "ê").replace("&ucirc;", "û").replace("&euml;", "ë")
        .replace("&iuml;", "ï").replace("&ugrave;", "ù")
        .split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Take until `needle`, returning (before, after).
fn take_once<'a>(hay: &'a str, needle: &str) -> (&'a str, &'a str) {
    match hay.find(needle) {
        Some(i) => (&hay[..i], &hay[i + needle.len()..]),
        None => (hay, ""),
    }
}

/// Extract an HTML attribute value from a tag-attr fragment.
fn extract_attr(attrs: &str, name: &str) -> Option<String> {
    let pat = format!("{name}=");
    let i = attrs.find(&pat)? + pat.len();
    let q = attrs.as_bytes().get(i).copied()?;
    if q != b'"' && q != b'\'' { return None; }
    let rest = &attrs[i + 1..];
    rest.find(q as char).map(|e| rest[..e].to_string())
}

/// Markdown page for one published post (`.md` sibling of the HTML page).
/// Front-matter + converted body; links/images keep absolute site URLs.
pub fn post_markdown(conn: &Connection, slug: &str, lang: &str, kind: &str, site_url: &str, tour_base: &str) -> Option<String> {
    let (title, body_html, excerpt, published_at, kind, slug, body_json): (String, String, String, String, String, String, String) = conn
        .query_row(sql!("render__select_markdown"), rusqlite::params![slug, lang, kind], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
        })
        .ok()?;
    // tours carry the page-builder tree in body_json (body_html is NULL):
    // walk it and pull every "content"/"text" field so the .md still has
    // the real copy, not an empty body.
    let body_html = if body_html.trim().is_empty() {
        body_json_text(&body_json)
    } else {
        body_html
    };
    let mut md = String::new();
    md.push_str("---\n");
    md.push_str(&format!("title: \"{}\"\n", title.replace('"', "\\\"")));
    md.push_str(&format!("url: {}\n", page_url(site_url.trim_end_matches('/'), lang, &kind, &slug, tour_base)));
    if !published_at.is_empty() { md.push_str(&format!("date: {}\n", iso8601(&published_at))); }
    md.push_str(&format!("type: {kind}\n---\n\n# {title}\n\n"));
    if !excerpt.is_empty() {
        md.push_str(&format!("> {}\n\n", excerpt.trim()));
    }
    md.push_str(&html_to_markdown(&body_html));
    md.push('\n');
    Some(md)
}

/// Collect every text-ish leaf out of a page-builder body_json tree —
/// element "content" fields hold HTML fragments; other string leaves are
/// titles/labels. Returns concatenated HTML for html_to_markdown.
fn body_json_text(json: &str) -> String {
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let mut out = String::new();
    fn walk(v: &serde_json::Value, out: &mut String) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, val) in m {
                    if matches!(k.as_str(), "content" | "text" | "excerpt" | "description") {
                        if let Some(s) = val.as_str() {
                            out.push_str(s);
                            out.push_str("\n\n");
                        }
                    }
                    walk(val, out);
                }
            }
            serde_json::Value::Array(a) => {
                for x in a {
                    walk(x, out);
                }
            }
            _ => {}
        }
    }
    walk(&v, &mut out);
    out
}

/// WP `_n` reading time: <200 words → "Moins d’une minute", else N minutes
/// at 200 wpm (matches the reference post: 2033 words → "10 minutes").
fn reading_time(words: usize, lang: &str) -> String {
    let es = lang == "es";
    if words < 200 {
        return if es {
            "Menos de un minuto".into()
        } else {
            "Moins d’une minute".into()
        };
    }
    let n = ((words as f64) / 200.0).round().max(1.0) as u64;
    if es {
        format!("{n} minutos")
    } else {
        format!("{n} minutes")
    }
}

/// Percent-encode the canonical URL for the oEmbed link hrefs (WP encodes
/// the scheme + path, upper-case hex).
fn urlenc_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `posts.modified_at` "2025-08-21 11:39:05" → "2025-08-21T11:39:05+00:00".
fn iso8601(dt: &str) -> String {
    let t = dt.trim().replace(' ', "T");
    if t.is_empty() {
        String::new()
    } else {
        format!("{t}+00:00")
    }
}

/// body_class per WP body_class(): the traveltour suffix is constant
/// (front and single pages share it verbatim in the captures).
fn body_class(_lang: &str, kind: &str, id: i64, is_front: bool) -> String {
    const SUFFIX: &str = "wp-theme-traveltour wp-child-theme-traveltour-child gdlr-core-body tourmaster-body metaslider-plugin traveltour-body traveltour-body-front traveltour-full  traveltour-with-sticky-navigation gdlr-core-link-to-lightbox";
    let core = if is_front {
        format!("home wp-singular page-template-default page page-id-{id}")
    } else {
        match kind {
            "post" => format!(
                "wp-singular post-template-default single single-post postid-{id} single-format-standard"
            ),
            "tour" => format!(
                "wp-singular tour-template-default single single-tour postid-{id} wp-theme-traveltour wp-child-theme-traveltour-child gdlr-core-body tourmaster-bottom-booking-bar tourmaster-mobile-read-more tourmaster-body metaslider-plugin traveltour-body traveltour-body-front traveltour-full  traveltour-with-sticky-navigation gdlr-core-link-to-lightbox"
            ),
            _ => format!("wp-singular page-template-default page page-id-{id}"),
        }
    };
    if kind == "tour" && !is_front {
        core
    } else {
        format!("{core} {SUFFIX}")
    }
}

/// Header navigation `current-*` classes keyed by `{tax-or-type}/{slug}`
/// (the base templates annotate each taxonomy/post-type menu <li> with a
/// navx lookup on the slug inside its hardcoded href — menu_items.object_id
/// does not line up with terms.id, so the template's own href is the join).
///
/// WP adds for a single tour: `current-tour-ancestor current-menu-parent
/// current-tour-parent` on taxonomy items pointing at one of the tour's
/// terms; for a post, `current-menu-item` on the post object item (and the
/// FR reference shows nothing on the post page — only filled when a menu
/// item actually matches).
pub(crate) fn nav_map(
    conn: &Connection,
    id: i64,
    kind: &str,
    slug: &str,
    is_front: bool,
) -> Result<Map<String, Value>> {
    let mut m = Map::new();
    match kind {
        "tour" => {
            let mut st = conn.prepare(
                "SELECT taxonomy, slug FROM post_terms WHERE post_id=?1 \
                 AND taxonomy IN ('region','duree','style','profil','Pays','Activites','duracion','estilo','perfil','region-es')",
            )?;
            let rows = st.query_map([id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?;
            for r in rows.flatten() {
                m.insert(
                    format!("{}/{}", r.0, r.1),
                    json!(" current-tour-ancestor current-menu-parent current-tour-parent"),
                );
            }
        }
        "post" => {
            m.insert(format!("post/{slug}"), json!(" current-menu-item"));
        }
        _ => {
            m.insert(
                format!("page/{slug}"),
                json!(" current-menu-item current_page_item"),
            );
        }
    }
    if is_front {
        // ES home nav marks its home root item; FR front does not.
        m.insert(
            "es-home".into(),
            json!(" current-menu-item current_page_item menu-item-home"),
        );
    }
    Ok(m)
}

/// Shared <head>/chrome template variables for base_fr/base_es. `seo_desc`
/// arrives already resolved by the caller (it differs subtly per kind);
/// everything else is computed here from the post row + post_meta so the
/// three render paths stay consistent.
///
/// Emitted keys (see templates/base_fr.html): html_lang, html_prefix,
/// body_class, is_front, is_contact, robots, og_type, og_desc,
/// og_image_block, og_updated, tw_label1/data1/label2/data2, og_feed_title
///(unused), feed_title, oembed_url, rest_url, shortlink, flag_fr_url,
/// flag_es_url, navx, head_css_extra/foot_scripts_extra placeholders.
pub(crate) fn chrome_vars(
    conn: &Connection,
    site_url: &str,
    id: i64,
    kind: &str,
    slug: &str,
    lang: &str,
    title: &str,
    canonical: &str,
    seo_desc: &str,
    body_html: &str,
    hreflang_fr: &str,
    hreflang_es: &str,
    tour_base: &str,
) -> Result<Map<String, Value>> {
    let (modified_at, featured_id, excerpt, published_at): (String, Option<i64>, String, String) = conn
        .query_row(sql!("render__select_chrome"), [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .unwrap_or_default();
    let meta = |key: &str| -> String {
        conn.query_row(
            "SELECT COALESCE(meta_value,'') FROM post_meta WHERE post_id=?1 AND meta_key=?2 LIMIT 1",
            rusqlite::params![id, key],
            |r| r.get(0),
        )
        .unwrap_or_default()
    };

    let es = lang == "es";
    let is_front = canonical == format!("{site_url}/")
        || canonical == format!("{site_url}/es/");
    let is_contact = slug == "rappel-gratuit" || slug == "devolucion-de-llamada-gratuita";
    let is_single = !is_front;

    // --- meta description / og:description -------------------------------
    // seo_desc already resolved by caller; else rank_math_description, then
    // excerpt, then the auto excerpt RankMath generates from post_content.
    let desc = if !seo_desc.is_empty() {
        seo_desc.to_string()
    } else {
        let rm = meta("rank_math_description");
        if !rm.is_empty() {
            rm
        } else if !excerpt.is_empty() {
            excerpt.clone()
        } else {
            auto_desc(body_html)
        }
    };

    // --- og:image block ---------------------------------------------------
    // featured_media_id → _thumbnail_id → 2000×1010 PNG logo (the RankMath
    // default for pages without an image; the home page sets it explicitly).
    let img_id = featured_id
        .or_else(|| meta("_thumbnail_id").parse::<i64>().ok())
        .unwrap_or(6444);
    let (path, alt, mime, w, h) = media_detail(conn, img_id)
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            (
                "2023/12/logo-agence-de-voyage-au-vietnam.png".into(),
                "Agence de voyage locale au Vietnam, depuis 2009. Voyage au Vietnam sur mesure, authentique, immersif au coeur de la culture, de la population.".into(),
                "image/png".into(),
                2000,
                1010,
            )
        });
    let img_url = format!("{site_url}/wp-content/uploads/{path}");
    let og_image_block = format!(
        "<meta property=\"og:image\" content=\"{u}\" />\n\
         <meta property=\"og:image:secure_url\" content=\"{u}\" />\n\
         <meta property=\"og:image:width\" content=\"{w}\" />\n\
         <meta property=\"og:image:height\" content=\"{h}\" />\n\
         <meta property=\"og:image:alt\" content=\"{a}\" />\n\
         <meta property=\"og:image:type\" content=\"{m}\" />",
        u = esc_attr(&img_url),
        w = w,
        h = h,
        a = esc_attr(&alt),
        m = mime,
    );

    // --- twitter label/data pairs -----------------------------------------
    // post: Écrit par + Temps de lecture; page: label1=lecture only; tour &
    // front: none (front emits the static author block in the template).
    let words = word_count(body_html);
    let (tw_label1, tw_data1, tw_label2, tw_data2) = match kind {
        "post" => (
            if es { "Escrito por" } else { "Écrit par" }.to_string(),
            "hieutuyen".to_string(),
            if es { "Tiempo de lectura" } else { "Temps de lecture" }.to_string(),
            reading_time(words, lang),
        ),
        "page" => (
            if es { "Tiempo de lectura" } else { "Temps de lecture" }.to_string(),
            reading_time(words, lang),
            String::new(),
            String::new(),
        ),
        _ => (String::new(), String::new(), String::new(), String::new()),
    };

    // --- flags ------------------------------------------------------------
    // FR flag points at the FR version (self on FR pages, the FR twin on ES
    // pages); ES flag at the ES version, omitted entirely when no pairing.
    let flag_fr_url = if !hreflang_fr.is_empty() {
        hreflang_fr.to_string()
    } else {
        canonical.to_string()
    };
    let flag_es_url = if !is_front && hreflang_es == format!("{site_url}/es/") {
        String::new()
    } else if es {
        canonical.to_string()
    } else {
        hreflang_es.to_string()
    };

    // --- per-page link tags -------------------------------------------------
    let rest_type = match kind {
        "post" => "posts",
        "tour" => "tour",
        _ => "pages",
    };
    let rest_url = format!("{site_url}/wp-json/wp/v2/{rest_type}/{id}");
    let oembed_url = urlenc_path(canonical);
    let shortlink = if is_front {
        canonical.to_string()
    } else {
        format!("{site_url}/?p={id}")
    };

    let mut m = Map::new();
    m.insert("html_lang".into(), json!(if es { "es-ES" } else { "fr-FR" }));
    m.insert("og_locale".into(), json!(if es { "es_ES" } else { "fr_FR" }));
    m.insert("kind".into(), json!(kind));
    m.insert(
        "html_prefix".into(),
        json!("prefix=\"og: https://ogp.me/ns#\" class=\"no-js\""),
    );
    m.insert("body_class".into(), json!(body_class(lang, kind, id, is_front)));
    m.insert("is_front".into(), json!(is_front));
    m.insert("is_contact".into(), json!(is_contact));
    m.insert("is_single".into(), json!(is_single));
    m.insert("robots".into(), json!(robots_meta(&meta("rank_math_robots"))));
    m.insert(
        "og_type".into(),
        json!(if is_front { "website" } else { "article" }),
    );
    m.insert("og_desc".into(), json!(esc_attr(&desc)));
    m.insert("og_image_block".into(), json!(og_image_block));
    m.insert("og_image_url".into(), json!(img_url));
    m.insert("og_updated".into(), json!(iso8601(&modified_at)));
    m.insert("tw_label1".into(), json!(tw_label1));
    m.insert("tw_data1".into(), json!(tw_data1));
    m.insert("tw_label2".into(), json!(tw_label2));
    m.insert("tw_data2".into(), json!(tw_data2));
    m.insert("oembed_url".into(), json!(oembed_url));
    m.insert("rest_url".into(), json!(rest_url));
    m.insert("shortlink".into(), json!(shortlink));
    m.insert("flag_fr_url".into(), json!(flag_fr_url));
    m.insert("flag_es_url".into(), json!(flag_es_url));
    m.insert("navx".into(), Value::Object(nav_map(conn, id, kind, slug, is_front)?));
    let page_title_html = if kind == "page" && !is_front {
        let bg = if slug == "rappel-gratuit" {
            format!(" style=\"background-image: url({site_url}/wp-content/uploads/2024/02/slider-1.jpg) ;\"")
        } else {
            String::new()
        };
        format!("<div class=\"traveltour-page-title-wrap  traveltour-style-large traveltour-center-align\"{bg} ><div class=\"traveltour-header-transparent-substitute\" ></div><div class=\"traveltour-page-title-overlay\"  ></div><div class=\"traveltour-page-title-container traveltour-container\" ><div class=\"traveltour-page-title-content traveltour-item-pdlr\"  ><h1 class=\"traveltour-page-title\"  >{}</h1></div></div></div>", esc(title))
    } else {
        String::new()
    };
    m.insert("page_title_html".into(), json!(page_title_html));
    // hooks filled by other render paths/phases; defined so the base
    // template can reference them unconditionally.
    // settings.custom_css = WP "Additional CSS" carried by the importer —
    // emitted inline like wp_custom_css_cb does.
    let custom_css: String = conn
        .query_row("SELECT value FROM settings WHERE key='custom_css'", [], |r| r.get(0))
        .unwrap_or_default();
    let custom_css_tag = if custom_css.trim().is_empty() {
        String::new()
    } else {
        format!(
            "<style id='wp-custom-css' type='text/css'>{}</style>",
            custom_css.replace("</style", "<\\/style")
        )
    };
    m.insert("head_css_extra".into(), json!(custom_css_tag));
    m.insert("head_css_eztoc".into(), json!(""));
    m.insert("head_css_wpdiscuz".into(), json!(""));
    m.insert("foot_scripts_eztoc".into(), json!(""));
    m.insert("foot_scripts_comment_reply".into(), json!(""));
    m.insert("foot_scripts_wpdiscuz".into(), json!(""));
    // RankMath schema.org JSON-LD (rank_math_schema_{Type} post_meta):
    // unserialize the stored template, substitute %token% fields, emit the
    // <script> tag. Empty when the post carries no schema meta.
    let jsonld = post_jsonld(conn, id, title, &desc, &published_at, &modified_at, &img_url, site_url)
        .map(|j| format!("<script type=\"application/ld+json\">{j}</script>"))
        .unwrap_or_default();
    m.insert("post_jsonld".into(), json!(jsonld));
    // Widget registry: the four traveltour-footer-column contents come from
    // the footer-{1..4}-{lang} sidebars; base_fr/base_es keep the column
    // wrapper divs.
    for n in 1..=4 {
        let sidebar = format!("footer-{n}-{}", if es { "es" } else { "fr" });
        let html = crate::widgets::render_sidebar(conn, &sidebar, site_url, lang, slug, tour_base)?;
        m.insert(format!("footer_{n}"), json!(html));
    }
    Ok(m)
}

/// Dummy site payload so template smoke tests exercise dynamic branches.
#[cfg(test)]
fn dummy_site() -> JVal {
    JVal::from_serialize(json!({
        "posts": [{"id":1,"slug":"hello","title":"Hello post","excerpt":"Ex",
            "date":"2026-01-01","url":"https://x.test/hello/","image":"https://x.test/i.jpg",
            "categories":[{"slug":"news","name":"News"}]}],
        "tours": [{"id":2,"slug":"tour-a","title":"Tour A","excerpt":"TEx",
            "url":"https://x.test/voyage/tour-a/","image":"https://x.test/t.jpg",
            "alt":"ta","duration_days":12,"price":"$1000","rating":4.5,"max_people":10,
            "duration_text":"12 jours 11 nuits","departure":"Hanoi",
            "promo_text":"Promo","promo_color":"#fff","promo_bg":"#10873b",
            "promo_bg_alpha":"rgba(16, 135, 59, 0.5)","price_display":"1,000 USD",
            "review_count":2,
            "terms":[{"taxonomy":"duree","slug":"x","name":"12 days"},
                     {"taxonomy":"style","slug":"style-1","name":"S1"}]}],
        "reviews": [{"id":3,"name":"Jane","content":"<p>Great!</p>","excerpt":"",
            "rating":5,"image":"https://x.test/r.jpg"}],
        "terms": {"duracion":[{"slug":"d1","name":"D1"}],"region-es":[{"slug":"r1","name":"R1"}],
            "estilo":[{"slug":"e1","name":"E1"}],"perfil":[{"slug":"p1","name":"P1"}],
            "category":[{"slug":"news","name":"News"}]},
        "menus": {"main":[{"id":1,"parent_id":0,"ord":1,"title":"Home","url":"https://x.test/",
            "children":[{"id":2,"parent_id":1,"ord":2,"title":"Sub","url":"https://x.test/s/","children":[]}]}]},
        "pages": [{"id":4,"slug":"about","title":"About","url":"https://x.test/about/","parent_id":0,"order":1}],
        "lang":"fr","url_prefix":"","site_url":"https://x.test",
        "tour_base":"voyage","tour_base_url":"https://x.test/voyage/"
    }))
}

/// Broad opts payload so most conditional branches render.
#[cfg(test)]
fn dummy_opts() -> Value {
    serde_json::from_str(r##"{
        "id":"el-1","class":"extra","title":"A Title","content":"<p>Some <b>content</b></p>",
        "caption":"Caption","link":"https://x.test/","link-target":"_blank","icon":"fa fa-gear",
        "image":"1","gallery":[{"id":"1","title":"g","caption":"c"},{"id":"1","title":"g2"}],
        "tabs":[{"title":"Tab 1","content":"<p>c1</p>","active":"enable","icon":"fa fa-star","image":"1"},
                {"title":"Tab 2","content":"<p>c2</p>","active":"disable"}],
        "items":[{"title":"It","content":"<p>i</p>"}],
        "slides":[{"image":"1","title":"s"}],"icons":[{"icon":"fa fa-a","title":"i"}],
        "lists":[{"title":"L","content":"c"}],"hours":[{"day":"Mon","time":"9-5"}],
        "column":"3","num-fetch":"3","category":"news","style":"style-1","align":"center",
        "text-align":"center","alignment":"center","type":"normal","icon-type":"icon",
        "padding-bottom":"30px","margin-top":"10px","margin-bottom":"10px",
        "margin-left":"5px","margin-right":"5px","padding":{"top":"1px","right":"2px","bottom":"3px","left":"4px","settings":"unlink"},
        "margin":{"top":"1px","right":"2px","bottom":"3px","left":"4px","settings":"unlink"},
        "border-width":{"top":"1px","right":"1px","bottom":"1px","left":"1px","settings":"link"},
        "background-color":"#ff0000","text-color":"#00ff00","divider-color":"#0000ff",
        "title-color":"#111111","content-color":"#222222","icon-color":"#333333",
        "url":"https://www.youtube.com/watch?v=x","mp4":"v.mp4","webm":"v.webm","ogv":"v.ogv",
        "video-url":"https://vimeo.com/1","audio-url":"a.mp3","code":"<html>x</html>",
        "script":"console.log(1)","menu":"main","menu-id":"1","fields":["duracion","region-es","estilo","perfil"],
        "fields-title":[{"title":"Dur","name":"duracion"}],
        "item-style":"style-1","with-frame":"disable","enable-read-more":"enable",
        "media-type":"icon","heading-tag":"h3","title-tag":"h3","date":"2030-01-01",
        "day":"1","month":"Jan","year":"2030","time":"10:00","location":"Hanoi",
        "number":"100","counter-start":"0","counter-end":"100","percent":"80","size":"100",
        "front-title":"Front","back-title":"Back","front-content":"<p>f</p>","back-content":"<p>b</p>",
        "price":"$99","currency":"$","period":"/mo","features":[{"title":"F1","content":"C1"}],
        "button-text":"Go","button-link":"#","social":"facebook","username":"user",
        "taxonomy":"duree","order":"asc","orderby":"title","num":"5","height":"100px",
        "svg":"<svg></svg>","data":"1,2,3","labels":"a,b,c","embed":"<iframe></iframe>",
        "thumbnail-size":"full","quote":"A quote","author":"Auth","position":"Dev",
        "left-title":"L","right-title":"R","duration":"1000","text":"Animated",
        "words":["one","two"],"speed":"100","opening-hours":[{"day":"Mon","time":"9-5"}],
        "social-networks":[{"network":"facebook","url":"https://fb.com/x","title":"FB"}],
        "networks":[{"network":"twitter","url":"https://tw.com/x"}],
        "posts":[{"title":"P"}],"events":[{"title":"E"}],"roads":[{"title":"R","content":"C"}],
        "food_items":[{"name":"Dish","price":"$5","content":"desc"}],
        "tab":[{"title":"T","content":"C"}],"slides-title":[{"title":"S"}],
        "port-infos":[{"title":"Client","content":"ACV"}],
        "infos":[{"title":"Client","content":"ACV"}],
        "timeline":[{"date":"2020","title":"T","content":"C"}],
        "title-size":"30px","title-font-weight":"600","title-text-transform":"uppercase",
        "content-size":"15px","icon-size":"30px","width":"100","thumbnail":"1",
        "enable-title":"enable","enable-caption":"enable","enable-overlay":"enable",
        "enable-icon":"enable","enable-border":"enable","enable-shadow":"enable",
        "disable":"disable","hide-title":"disable","carousel":"enable","carousel-navigation":"none",
        "background":"none","background-image":"1","background-type":"color",
        "media":"1","src":"1","file":"1","poster":"1","quote-author":"Auth",
        "head-text":"Head","price-symbol":"$","currency-symbol":"$",
        "sub-title":"Sub","max-width":"500px","font-size":"16px",
        "social-share":"facebook","services":"facebook,twitter"
    }"##).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Renders every elements/*.html against a broad dummy context; fails on
    /// any template syntax/render error. New element templates are picked up
    /// automatically.
    #[test]
    fn element_templates_render() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
        let eldir = dir.join("elements");
        let media: HashMap<i64, (String, String)> =
            [(1i64, ("2024/01/img.jpg".to_string(), "alt".to_string()))]
                .into_iter()
                .collect();
        let site_url = "https://x.test";
        let mut failures = Vec::new();
        let mut entries: Vec<_> = fs::read_dir(&eldir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".html"))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let name = e.file_name().to_string_lossy().to_string();
            let typ = name.trim_end_matches(".html").to_string();
            let node = json!({
                "template": "element",
                "type": typ,
                "column": "60",
                "value": dummy_opts(),
                "items": [{"template": "element", "type": "text-box",
                           "value": {"content": "child"}}]
            });
            let mut env = match load_env(&dir) {
                Ok(v) => v,
                Err(err) => {
                    failures.push(format!("env: {err}"));
                    break;
                }
            };
            register_helpers(&mut env, &media, site_url);
            let post = JVal::from_serialize(json!({"title": "Post", "body_html": "<p>BH</p>", "meta": {"cf":"v"}}));
            match render_nodes(&mut env, &node, &media, site_url, &dummy_site(), "fr", &post, None, false, "voyage") {
                Ok(_) => {}
                Err(err) => {
                    if name == "fallback.html" {
                        // unknown type 'fallback' → renders itself fine anyway
                    }
                    failures.push(format!("{name}: {err}"));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}

#[cfg(test)]
mod exposure_tests {
    use super::*;

    /// In-memory DB with the migration DDL applied.
    fn memdb() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(crate::schema::DDL).unwrap();
        c
    }

    fn post(c: &Connection, id: i64, slug: &str, lang: &str, status: &str, kind: &str) {
        c.execute(
            "INSERT INTO posts(id, slug, lang, title, body_json, status, kind) VALUES(?1,?2,?3,'t','[]',?4,?5)",
            rusqlite::params![id, slug, lang, status, kind],
        )
        .unwrap();
    }

    #[test]
    fn front_id_prefers_group_member_and_is_deterministic() {
        let c = memdb();
        c.execute_batch(
            "INSERT INTO settings(key,value) VALUES
             ('languages','fr,es'),('primary_lang','fr'),('page_on_front','10');",
        )
        .unwrap();
        post(&c, 10, "accueil", "fr", "publish", "page");
        post(&c, 11, "inicio", "es", "publish", "page");
        // Translation group → ES member wins over the __trashed fallback.
        c.execute_batch(
            "INSERT INTO post_translation_members(group_id,lang,post_id) VALUES('g','fr',10),('g','es',11);
             INSERT INTO posts(id,slug,lang,title,body_json,status,kind) VALUES(12,'x__trashed','es','t','[]','publish','page'),(13,'y__trashed','es','t','[]','publish','page');",
        )
        .unwrap();
        assert_eq!(front_id_for_lang(&c, "es"), Some(11));
        assert_eq!(front_id_for_lang(&c, "fr"), Some(10));
        // Group gone → legacy pair table → 11.
        c.execute_batch("DELETE FROM post_translation_members; INSERT INTO post_translations(fr_post_id,es_post_id) VALUES(10,11);").unwrap();
        assert_eq!(front_id_for_lang(&c, "es"), Some(11));
        // Neither → lowest-id published __trashed page, deterministic.
        c.execute("DELETE FROM post_translations", []).unwrap();
        assert_eq!(front_id_for_lang(&c, "es"), Some(12));
    }

    #[test]
    fn front_id_skips_unpublished() {
        let c = memdb();
        c.execute_batch(
            "INSERT INTO settings(key,value) VALUES
             ('languages','fr,es'),('primary_lang','fr'),('page_on_front','10');",
        )
        .unwrap();
        post(&c, 10, "accueil", "fr", "draft", "page");
        post(&c, 11, "inicio", "es", "draft", "page");
        c.execute("INSERT INTO post_translation_members(group_id,lang,post_id) VALUES('g','fr',10),('g','es',11)", []).unwrap();
        assert_eq!(front_id_for_lang(&c, "fr"), None);
        // Draft group member must not leak; no publish __trashed page → None.
        assert_eq!(front_id_for_lang(&c, "es"), None);
        post(&c, 12, "x__trashed", "es", "draft", "page");
        post(&c, 13, "y__trashed", "es", "publish", "page");
        assert_eq!(front_id_for_lang(&c, "es"), Some(13));
    }
}

/* ================= gdlr_core_content_filter equivalent ================= */

/// `[tag attr="v" attr2='v']` attribute parser for the shortcode subset.
fn sc_attrs(s: &str) -> Map<String, Value> {
    let mut m = Map::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // key=
        let key_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
            i += 1;
        }
        if i == key_start {
            i += 1;
            continue;
        }
        let key = s[key_start..i].to_string();
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let vstart = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                m.insert(key, json!(s[vstart..i]));
                i += 1;
            } else {
                let vstart = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b']' {
                    i += 1;
                }
                m.insert(key, json!(s[vstart..i]));
            }
        }
    }
    m
}

/// Expand the shortcodes this site's text-boxes actually contain:
/// `[gdlr_core_button]`, `[embed]url[/embed]` (YouTube/Vimeo → iframe),
/// `[caption]…[/caption]` (WP figure markup), `[gdlr_core_icon]`.
/// Unknown tags are left verbatim (WP shows them as text too).
pub(crate) fn expand_shortcodes(s: &str, site_url: &str) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(s.len() + 256);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            // find matching ] for the tag header
            if let Some(end) = s[i..].find(']') {
                let head = &s[i + 1..i + end];
                let tag = head.split_whitespace().next().unwrap_or("");
                match tag {
                    "site_reviews" => {
                        let html = include_str!("../templates/elements/site_reviews_static.html")
                            .replace("{{ site_url }}", site_url);
                        out.extend_from_slice(html.as_bytes());
                        i += end + 1;
                        continue;
                    }
                    "site_reviews_form" => {
                        let a = sc_attrs(head);
                        let id = a.get("id").and_then(|v| v.as_str()).unwrap_or("lte68mso");
                        let html = include_str!("../templates/elements/site_reviews_form.html")
                            .replace("{{ form_id }}", id)
                            .replace("{{ post_id }}", "6190")
                            .replace("{{ site_url }}", site_url);
                        out.extend_from_slice(html.as_bytes());
                        i += end + 1;
                        continue;
                    }
                    "contact-form-7" => {
                        let html = include_str!("../templates/elements/cf7_78de6b9.html")
                            .replace("{{ site_url }}", site_url);
                        out.extend_from_slice(html.as_bytes());
                        i += end + 1;
                        continue;
                    }
                    "gdlr_core_button" => {
                        let a = sc_attrs(head);
                        let gs = |k: &str| a.get(k).and_then(|v| v.as_str()).unwrap_or("");
                        let text = gs("button-text");
                        let link = gs("button-link");
                        let target = gs("button-link-target");
                        let mut style = String::new();
                        for (k, css) in [
                            ("margin-right", "margin-right"),
                            ("margin-left", "margin-left"),
                            ("margin-bottom", "margin-bottom"),
                            ("padding", "padding"),
                        ] {
                            let v = gs(k);
                            if !v.is_empty() && v != "0px" {
                                style.push_str(&format!("{css}: {v};"));
                            }
                        }
                        let tgt = if target.is_empty() || target == "_self" {
                            String::new()
                        } else {
                            format!(" target=\"{}\"", esc_attr(target))
                        };
                        let st = if style.is_empty() { String::new() } else { format!(" style=\"{style}\"") };
                        out.extend_from_slice(format!(
                            "<a class=\"gdlr-core-button gdlr-core-button-shortcode  gdlr-core-button-gradient gdlr-core-button-no-border\" href=\"{}\"{tgt} {st}  ><span class=\"gdlr-core-content\"  >{}</span></a>",
                            esc_attr(link),
                            esc(text)
                        ).as_bytes());
                        i += end + 1;
                        continue;
                    }
                    "embed" => {
                        if let Some(close) = s[i + end + 1..].find("[/embed]") {
                            let inner = &s[i + end + 1..i + end + 1 + close];
                            out.extend_from_slice(embed_html(inner, site_url).as_bytes());
                            i += end + 1 + close + "[/embed]".len();
                            continue;
                        }
                    }
                    "caption" => {
                        if let Some(close) = s[i + end + 1..].find("[/caption]") {
                            let inner = &s[i + end + 1..i + end + 1 + close];
                            // inner: <img …/> followed by the caption text
                            let img_end = inner.find("/>").map(|p| p + 2).unwrap_or(0);
                            let img = &inner[..img_end];
                            let cap = inner[img_end..].trim();
                            let a = sc_attrs(head);
                            let gs = |k: &str| a.get(k).and_then(|v| v.as_str()).unwrap_or("");
                            let att_id = gs("id");
                            let align = {
                                let al = gs("align");
                                if al.is_empty() { "alignnone" } else { al }
                            };
                            let width = gs("width");
                            // img_caption_shortcode: figure + figcaption
                            let wstyle = if width.is_empty() {
                                String::new()
                            } else {
                                format!(" style=\"width: {}px\"", esc_attr(width))
                            };
                            let att_id = esc_attr(att_id);
                            let align = esc_attr(align);
                            out.extend_from_slice(format!(
                                "<figure id=\"{att_id}\" aria-describedby=\"caption-{att_id}\"{wstyle} class=\"wp-caption {align}\">{img}<figcaption id=\"caption-{att_id}\" class=\"wp-caption-text\">{cap}</figcaption></figure>",
                            ).as_bytes());
                            i += end + 1 + close + "[/caption]".len();
                            continue;
                        }
                    }
                    "gdlr_core_icon" => {
                        let a = sc_attrs(head);
                        let gs = |k: &str| a.get(k).and_then(|v| v.as_str()).unwrap_or("");
                        let icon = gs("icon");
                        let mut style = String::new();
                        for (k, css) in [("color", "color"), ("size", "font-size"), ("margin-left", "margin-left"), ("margin-right", "margin-right")] {
                            let v = gs(k);
                            if !v.is_empty() {
                                style.push_str(&format!("{css}: {v};"));
                            }
                        }
                        out.extend_from_slice(format!(
                            "<i class=\"{}\" style=\"{}\" ></i>",
                            esc_attr(icon),
                            esc_attr(&style)
                        ).as_bytes());
                        i += end + 1;
                        continue;
                    }
                    _ => {}
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `[embed]` — YouTube/Vimeo/Spotify URL → iframe, matching wp_oembed output
/// shape (1150×647 like the reference homepage).
fn embed_html(url: &str, _site_url: &str) -> String {
    let u = url.trim();
    let video_id = if let Some(p) = u.split("v=").nth(1) {
        Some(p.split('&').next().unwrap_or(""))
    } else if u.contains("youtu.be/") {
        u.split("youtu.be/").nth(1).map(|x| x.split('?').next().unwrap_or(""))
    } else {
        None
    };
    if let Some(id) = video_id {
        if !id.is_empty() {
            let t = u.split("t=").nth(1).and_then(|x| x.trim_end_matches('s').split('&').next()).and_then(|x| x.parse::<i64>().ok()).unwrap_or(0);
            let start = if t > 0 { format!("?start={t}&amp;feature=oembed") } else { "?feature=oembed".into() };
            return format!(
                "<iframe title=\"YouTube video\" width=\"1150\" height=\"647\" src=\"https://www.youtube.com/embed/{id}{start}\" frameborder=\"0\" allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share\" referrerpolicy=\"strict-origin-when-cross-origin\" allowfullscreen></iframe>",
            );
        }
    }
    u.to_string()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}

/// Latest published posts in `lang`, optionally narrowed to the element's
/// category/tag slugs (WP `tax_query field=slug`); empty match → latest posts.
/// Returns HTML for `blog.html`'s `{{ children|safe }}`.
fn blog_posts(
    conn: &Connection,
    opts: &Map<String, Value>,
    lang: &str,
    media: &HashMap<i64, (String, String)>,
    site_url: &str,
    tour_base: &str,
) -> Result<String> {
    let num: i64 = opts
        .get("num-fetch")
        .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_i64()))
        .unwrap_or(10)
        .clamp(1, 24);
    let cats: Vec<String> = opts
        .get("category")
        .map(|v| match v {
            Value::Array(a) => a.iter().filter_map(|x| x.as_str().map(String::from)).collect(),
            Value::String(s) => s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect(),
            _ => vec![],
        })
        .unwrap_or_default();
    let mut rows = query_blog(conn, &cats, lang, num).unwrap_or_default();
    if rows.is_empty() {
        rows = query_blog(conn, &[], lang, num).unwrap_or_default();
    }
    let bstyle = opts
        .get("blog-style")
        .and_then(|v| v.as_str())
        .unwrap_or("blog-column-with-frame");
    if bstyle == "blog-column" {
        // Non-carousel column grid (pb-element-blog.php blog-column branch):
        // item-list > blog-grid > thumbnail + grid-content-wrap; cards carry
        // no frame and the content is empty when excerpt-number is 0.
        let col_size: i64 = opts
            .get("column-size")
            .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_i64()))
            .unwrap_or(60)
            .clamp(1, 60);
        let ncols = (60 / col_size).max(1);
        let grid_style = opts
            .get("blog-column-style")
            .and_then(|v| v.as_str())
            .unwrap_or("style-1");
        let title_size = opts
            .get("blog-title-font-size")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let title_fstyle = opts
            .get("blog-title-font-style")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let excerpt_n: usize = opts
            .get("excerpt-number")
            .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_i64()))
            .unwrap_or(0)
            .max(0) as usize;
        let mut out = String::new();
        for (i, (_id, slug, title, excerpt, feat_id, _published)) in rows.iter().enumerate() {
            let url = page_url(site_url.trim_end_matches('/'), lang, "post", slug, tour_base);
            let img = feat_id
                .and_then(|fid| media_info(conn, fid).map(|d| (fid, d)))
                .map(|(fid, d)| {
                    let p = d.get("path").and_then(|x| x.as_str()).unwrap_or("");
                    let alt = d.get("alt").and_then(|x| x.as_str()).unwrap_or("");
                    let w = d.get("w").and_then(|x| x.as_i64()).unwrap_or(0);
                    let h = d.get("h").and_then(|x| x.as_i64()).unwrap_or(0);
                    format!(
                        "<img src=\"{}/wp-content/uploads/{}\" alt=\"{}\"{} title=\"{}\"{} />",
                        site_url.trim_end_matches('/'),
                        esc_attr(p),
                        esc_attr(alt),
                        if w > 0 { format!(" width=\"{w}\" height=\"{h}\"") } else { String::new() },
                        esc_attr(&wp_title(alt)),
                        media_img_attrs(conn, fid, site_url),
                    )
                })
                .unwrap_or_default();
            let exc = if excerpt_n == 0 {
                String::new()
            } else {
                let t = excerpt.split_whitespace().take(excerpt_n).collect::<Vec<_>>().join(" ");
                if t.is_empty() { String::new() } else { format!("{t}...") }
            };
            let mut title_style = String::new();
            if !title_size.is_empty() {
                title_style.push_str(&format!("font-size: {title_size} ;"));
            }
            if !title_fstyle.is_empty() {
                title_style.push_str(&format!("font-style: {title_fstyle} ;"));
            }
            out.push_str(&format!(
                "<div class=\"gdlr-core-item-list  gdlr-core-item-pdlr gdlr-core-column-{col_size}{}\" >\
<div class=\"gdlr-core-blog-grid gdlr-core-style-{grid_style} \"  >\
<div class=\"gdlr-core-blog-thumbnail gdlr-core-media-image  gdlr-core-opacity-on-hover gdlr-core-zoom-on-hover\"  ><a href=\"{url}\" >{img}</a></div>\
<div class=\"gdlr-core-blog-grid-content-wrap\">\
<h3 class=\"gdlr-core-blog-title gdlr-core-skin-title\" style=\"{title_style}\"  ><a href=\"{url}\" >{}</a></h3>\
<div class=\"gdlr-core-blog-content clearfix\" >{exc}</div></div></div></div>",
                if i as i64 % ncols == 0 { " gdlr-core-column-first" } else { "" },
                esc(title)
            ));
        }
        return Ok(out);
    }
    // pb-element-blog.php (blog-column-with-frame): cards ride a flexslider
    // carousel — <ul class="slides"><li> per post.
    let mut out = String::from(
        "<div class=\"gdlr-core-flexslider flexslider gdlr-core-js-2\" data-type=\"carousel\" data-column=\"4\" data-move=\"3\" data-nav=\"none\" data-nav-parent=\"gdlr-core-blog-item\" ><ul class=\"slides\" >",
    );
    for (id, slug, title, excerpt, feat_id, _published) in rows {
        let url = page_url(site_url.trim_end_matches('/'), lang, "post", &slug, tour_base);
        let img = feat_id
            .and_then(|fid| media.get(&fid).map(|(p, alt)| (fid, p.clone(), alt.clone())))
            .map(|(fid, p, alt)| {
                format!(
                    "<img src=\"{}/wp-content/uploads/{}\" alt=\"{}\"{} />",
                    site_url.trim_end_matches('/'),
                    esc_attr(&p),
                    esc_attr(&alt),
                    media_img_attrs(conn, fid, site_url),
                )
            })
            .unwrap_or_default();
        let exc = excerpt.chars().take(140).collect::<String>();
        let exc = if exc.is_empty() { String::new() } else { format!("{exc}...") };
        out.push_str(&format!(
            "<li class=\"gdlr-core-item-mglr\" ><div class=\"gdlr-core-blog-grid gdlr-core-style-2 gdlr-core-js gdlr-core-blog-grid-with-frame gdlr-core-item-mgb gdlr-core-skin-e-background\" data-sync-height=\"blog-item-{id}\" >\
<div class=\"gdlr-core-blog-thumbnail gdlr-core-media-image gdlr-core-opacity-on-hover gdlr-core-zoom-on-hover\" ><a href=\"{url}\" >{img}</a></div>\
<div class=\"gdlr-core-blog-grid-frame gdlr-core-sync-height-space-position\" >\
<h3 class=\"gdlr-core-blog-title gdlr-core-skin-title\" style=\"font-size: 17px ;font-style: normal ;\" ><a href=\"{url}\" >{}</a></h3>\
<div class=\"gdlr-core-blog-content clearfix\" >{exc}<div class=\"clear\"></div></div></div></div></li>",
            esc(&title)
        ));
    }
    out.push_str("</ul></div>");
    Ok(out)
}
/// WP `wp_get_attachment_image_srcset`+`sizes` as a ready-made attribute
/// fragment (` srcset="…" sizes="…"`, leading space; empty string when the
/// attachment has no usable sizes). Candidate sizes follow WP's rules:
/// aspect ratio within 1% of the full image — this filters out the cropped
/// `thumbnail` automatically, as WP does.
pub(crate) fn media_img_attrs(conn: &Connection, id: i64, site_url: &str) -> String {
    let row = conn
        .query_row(
            "SELECT path, COALESCE(sizes_json,''), COALESCE(w,0), COALESCE(h,0) FROM media WHERE id=?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            },
        )
        .ok();
    let Some((path, sizes_json, w, h)) = row else { return String::new() };
    if w <= 0 || h <= 0 {
        return String::new();
    }
    let site = site_url.trim_end_matches('/');
    let dir = path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let ratio = w as f64 / h as f64;
    // (url, width) candidates, ascending — WP sorts srcset by width.
    let mut entries: Vec<(String, i64)> = Vec::new();
    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&sizes_json) {
        for sz in map.values() {
            let f = sz.get("file").and_then(|v| v.as_str()).unwrap_or("");
            let (sw, sh) = (
                sz.get("width").and_then(|v| v.as_i64()).unwrap_or(0),
                sz.get("height").and_then(|v| v.as_i64()).unwrap_or(0),
            );
            if f.is_empty() || sw <= 0 || sh <= 0 || f.contains("..") || f.contains('/') {
                continue;
            }
            if ((sw as f64 / sh as f64) - ratio).abs() / ratio > 0.01 {
                continue;
            }
            entries.push((format!("{site}/wp-content/uploads/{dir}/{f}"), sw));
        }
    }
    if entries.is_empty() {
        return String::new();
    }
    entries.push((format!("{site}/wp-content/uploads/{path}"), w));
    entries.sort_by_key(|(_, ew)| *ew);
    let srcset = entries
        .iter()
        .map(|(u, ew)| format!("{} {}w", esc_attr(u), ew))
        .collect::<Vec<_>>()
        .join(", ");
    format!(" srcset=\"{srcset}\" sizes=\"auto, (max-width: {w}px) 100vw, {w}px\"")
}

/// One media row as a JSON object: {path, alt, caption, title, w, h}.
/// (Distinct from Phase-2 `media_detail`, which returns the og:image tuple.)
fn media_info(conn: &Connection, id: i64) -> Option<Value> {
    conn.query_row(
        "SELECT path, COALESCE(alt,''), COALESCE(caption,''), COALESCE(title,''), \
         COALESCE(w,0), COALESCE(h,0) FROM media WHERE id=?1",
        [id],
        |r| {
            Ok(json!({
                "path": r.get::<_, String>(0)?,
                "alt": r.get::<_, String>(1)?,
                "caption": r.get::<_, String>(2)?,
                "title": r.get::<_, String>(3)?,
                "w": r.get::<_, i64>(4)?,
                "h": r.get::<_, i64>(5)?,
            }))
        },
    )
    .ok()
}

/// WP wptexturize subset for attribute text: ' -> &#8217;, " - " -> &#8211;.
fn wp_title(s: &str) -> String {
    s.replace('\'', "&#8217;").replace(" - ", " &#8211; ").replace('—', "&#8212;")
}

fn query_blog(
    conn: &Connection,
    cats: &[String],
    lang: &str,
    num: i64,
) -> Result<Vec<(i64, String, String, String, Option<i64>, String)>> {
    let base = "SELECT p.id, p.slug, p.title, COALESCE(p.excerpt,''), p.featured_media_id, COALESCE(p.published_at,'') FROM posts p";
    let (sql, params): (String, Vec<String>) = if cats.is_empty() {
        (
            format!("{base} WHERE p.status='publish' AND p.lang=?1 AND p.kind='post' ORDER BY p.published_at DESC LIMIT ?2"),
            vec![lang.to_string(), num.to_string()],
        )
    } else {
        let ph = cats.iter().enumerate().map(|(i, _)| format!("?{}", i + 3)).collect::<Vec<_>>().join(",");
        (
            format!("{base} JOIN post_terms pt ON pt.post_id=p.id WHERE p.status='publish' AND p.lang=?1 AND p.kind='post' AND pt.taxonomy='category' AND pt.slug IN ({ph}) ORDER BY p.published_at DESC LIMIT ?2"),
            [vec![lang.to_string(), num.to_string()], cats.to_vec()].concat(),
        )
    };
    let mut stmt = conn.prepare(&sql)?;
    let params_ref: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let rows = stmt.query_map(params_ref.as_slice(), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<i64>>(4)?,
            r.get::<_, String>(5)?,
        ))
    })?;
    Ok(rows.flatten().collect())
}
