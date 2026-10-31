//! Render posts through minijinja templates.

use anyhow::{Context, Result};
use minijinja::{context, Environment, Value as JVal};
use minijinja::value::Rest;
use rusqlite::{Connection, OptionalExtension};

use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::cell::RefCell;

// Submodules peeled off this file (pure moves — no behavior change).
pub(crate) mod markdown;
pub(crate) mod nodes;
pub(crate) use nodes::{register_helpers, render_fragment_db, render_nodes, ELEMENT_TYPES};
pub(crate) mod shortcode;
pub(crate) use shortcode::{expand_shortcodes, media_img_attrs, blog_posts, media_info};
pub(crate) use crate::html_decode::{esc, esc_attr};
pub(crate) mod chrome;
pub(crate) use chrome::{chrome_vars, hreflang_links, iso8601, page_url, translation_urls};
pub(crate) mod menu_url;
pub(crate) mod site_data;
pub(crate) use site_data::{site_data_cached, site_data_bust};
pub use markdown::{html_to_markdown, post_markdown};
pub(crate) use markdown::post_markdown_id;


pub fn run(db: &Path, templates: &Path, slug: &str, out: &Path, site_url: &str, tour_base: &str) -> Result<()> {
    // A leading configured language prefix (e.g. es/, de/) asks for that
    // translation; otherwise the primary language is used.
    let conn = crate::db::conn(db)?;
    let cfg = crate::lang::config(&conn);
    let clean = slug.trim_matches('/');
    let (lang, path) = crate::lang::split_path(&cfg, clean);
    let (html, _status, _canonical) = render_slug(db, templates, path, &lang, "", site_url, tour_base)?;
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
/// below. `render_slug` opens its own connection; bulk callers
/// (generate/verify) use `render_slug_conn` on a shared one.
pub fn render_slug(
    db: &Path,
    templates: &Path,
    slug: &str,
    lang: &str,
    kind: &str,
    site_url: &str,
    tour_base: &str,
) -> Result<(String, String, String)> {
    let conn = crate::db::conn(db)?;
    render_slug_conn(&conn, templates, slug, lang, kind, site_url, tour_base)
}

pub fn render_slug_conn(
    conn: &rusqlite::Connection,
    templates: &Path,
    slug: &str,
    lang: &str,
    kind: &str,
    site_url: &str,
    tour_base: &str,
) -> Result<(String, String, String)> {
    let lang_cfg = crate::lang::config(conn);
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
    let Some((id, lang, title, body_json, seo_title, seo_desc, row_kind, status)) = row else {
        let clean = slug.trim_matches('/');
        // "{lang}" / "{lang}/" → that language's front page.
        let (alang_s, path) = if clean.is_empty() { (lang.to_string(), "") } else { crate::lang::split_path(&lang_cfg, clean) };
        if path.is_empty() && alang_s != lang_cfg.primary {
            if let Some(front_id) = front_id_for_lang(&conn, &alang_s) {
                let (h, s) = render_slug_by_id_public(&conn, templates, front_id, site_url, tour_base)?;
                let canonical = crate::lang::page_path(&lang_cfg, &alang_s, "page", "", None);
                return Ok((h, s, canonical));
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
        let tag_base = crate::plugins::theme_tag_base(&conn, crate::plugins::theme_for_site(&conn));
        let term = match first {
            f if f == tag_base => crate::render_archive::term_by_path(&conn, "post_tag", leaf),
            "category" => crate::render_archive::term_by_path(&conn, "category", leaf),
            t if crate::render_archive::is_taxonomy(&conn, t) && !leaf.is_empty() => {
                crate::render_archive::term_by_path(&conn, t, leaf)
            }
            _ => None,
        };
        if let Some(term) = term {
            let canonical = term_canonical_path(&conn, &lang_cfg, &term, alang);
            let ctx = crate::plugins::RenderCtx {
                slug: slug.to_string(),
                lang: alang.to_string(),
                title: term.name.clone(),
                id: term.wp_id,
                site_url: site_url.to_string(),
                body_html: String::new(),
                body_html_rendered: String::new(),
                kind: "archive".to_string(),
                kind_base: tour_base.to_string(),
            };
            let h = crate::render_archive::page(&conn, templates, &ctx, &term)?;
            return Ok((crate::render_post::rewrite_origin(&h, site_url, &conn), "publish".to_string(), canonical));
        }
        // Category archives can be nested or single-segment; resolve the
        // leaf slug once post and old-slug lookups have missed.
        if !leaf.is_empty() {
            if let Some(term) = crate::render_archive::term_by_path(&conn, "category", leaf) {
                let canonical = term_canonical_path(&conn, &lang_cfg, &term, alang);
                let ctx = crate::plugins::RenderCtx {
                    slug: slug.to_string(),
                    lang: alang.to_string(),
                    title: term.name.clone(),
                    id: term.wp_id,
                    site_url: site_url.to_string(),
                    body_html: String::new(),
                    body_html_rendered: String::new(),
                    kind: "archive".to_string(),
                    kind_base: tour_base.to_string(),
                };
                let h = crate::render_archive::page(&conn, templates, &ctx, &term)?;
                return Ok((crate::render_post::rewrite_origin(&h, site_url, &conn), "publish".to_string(), canonical));
            }
        }
        // WP old-slug redirects: `/voyage/{old}/` etc must resolve to the
        // post's CURRENT slug. The `redirects` rows hold the full WP
        // history (post_meta's (post_id,meta_key) unique index collapses
        // multi-value _wp_old_slug to one row, so meta can't be the
        // source). Post lookup stays scoped to the request lang when
        // possible — a non-primary lang keeps the first published target.
        if !leaf.is_empty() {
            if let Ok(Some(pid)) = conn
                .query_row(
                    sql!("render__select_by_old_slug"),
                    [leaf.to_string()],
                    |r| r.get::<_, i64>(0),
                )
                .optional()
            {
                let (h, s) = render_slug_by_id_public(&conn, templates, pid, site_url, tour_base)?;
                let canonical = post_canonical_path(&conn, pid, tour_base)
                    .unwrap_or_else(|| format!("/{leaf}/"));
                return Ok((h, s, canonical));
            }
            // canonical slug of a post whose stored slug is degenerate
            // ('x' on tour 6416): the import-time permalink lives in the
            // redirect row written from that post's own _wp_old_slug set.
            if let Ok(Some(pid)) = conn
                .query_row(
                    sql!("render__select_old_slug_post"),
                    [format!("/{leaf}/")],
                    |r| r.get::<_, i64>(0),
                )
                .optional()
            {
                let (h, s) = render_slug_by_id_public(&conn, templates, pid, site_url, tour_base)?;
                let canonical = post_canonical_path(&conn, pid, tour_base)
                    .unwrap_or_else(|| format!("/{leaf}/"));
                return Ok((h, s, canonical));
            }
        }
        anyhow::bail!("slug not found: {slug}");
    };
    // kind="" on the miss path means "any kind" — but only kinds with a
    // registered KindDef are publicly routable; data-only kinds
    // (site-review, ml-slider/ml-slide) must not become pages.
    if kind.is_empty() && crate::plugins::kind_def(&row_kind).is_none() {
        anyhow::bail!("kind not publicly routable: {row_kind}");
    }
    let h = render_row(&conn, templates, id, slug, lang, title, body_json, seo_title, seo_desc, row_kind, site_url, tour_base)?;
    // Final-stage origin rewrite: element option values, menus, chrome and
    // any other imported markup may carry the source domain — normalize it
    // here so no render path can leak it (filter_content alone missed them).
    let h = crate::render_post::rewrite_origin(&h, site_url, &conn);
    let canonical = post_canonical_path(&conn, id, tour_base)
        .unwrap_or_else(|| format!("/{}/", slug.trim_matches('/')));
    Ok((h, status, canonical))
}

/// Canonical permalink path (leading+trailing `/`, no host) for a post id:
/// front pages resolve to their language root (`/` or `/{lang}/`), tours
/// to `/{lang-prefix}{tour_base}/{display_slug}/`. Drives the public_page
/// canonical-rel gate (M22): writes only land under this path, everything
/// else 301s here.
pub(crate) fn post_canonical_path(conn: &Connection, id: i64, tour_base: &str) -> Option<String> {
    let (slug, lang, kind, title): (String, String, String, String) = conn
        .query_row(
            sql!("render__select_slug_lang_kind_title"),
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .ok()?;
    let cfg = crate::lang::config(conn);
    if cfg.langs.iter().any(|l| front_id_for_lang(conn, l) == Some(id)) {
        return Some(crate::lang::page_path(&cfg, &lang, "page", "", None));
    }
    // Adapter-owned kinds canonicalize their slug (display_slug); core
    // kinds use the stored slug verbatim.
    let wslug = crate::plugins::adapter_for_kind(&kind)
        .and_then(|a| a.display_slug(conn, id, &slug, &title))
        .unwrap_or(slug);
    let base = crate::plugins::kind_url_base(tour_base, &kind);
    Some(crate::lang::page_path(&cfg, &lang, &kind, &wslug, base.as_deref()))
}

/// Canonical archive path for a resolved term under request language
/// `lang` — mirrors the emit mapping in `generate::term_out_paths`
/// (categories nest via the theme's cat_path, post_tag under the theme's
/// tag base, other taxonomies at {tax}/{slug}, lang rel-prefix). Using the
/// term's own taxonomy (not the request segment) keeps `/category/x/` → `/x/`.
fn term_canonical_path(
    conn: &Connection,
    cfg: &crate::lang::LangConfig,
    term: &crate::render_archive::Term,
    lang: &str,
) -> String {
    let t = term.taxonomy.to_lowercase();
    let theme = crate::plugins::theme_for_site(conn);
    let base = if t == "category" {
        crate::plugins::theme_cat_path(conn, theme, &term.slug)
    } else if t == "post_tag" {
        format!("{}/{}", crate::plugins::theme_tag_base(conn, theme), term.slug)
    } else {
        format!("{t}/{}", term.slug)
    };
    format!("{}/{base}/", crate::lang::url_prefix(cfg, lang))
}


/// Apply `navx` menu classes produced by `nav_map` onto the rendered page:
/// each `{tax}/{slug}`-style key maps to the nav <a> whose href is the
/// matching permalink. Tour term keys mark just the term's <li> (WP's
/// current-tour-* classes sit on the item itself); post/page/es-home keys
/// mark the item plus its <li> ancestors like the archive backstop does.
pub(crate) fn apply_navx(
    html: &str,
    site_url: &str,
    navx: &Map<String, Value>,
    conn: &Connection,
    lang: &str,
) -> String {
    let site = site_url.trim_end_matches('/');
    let mut out = html.to_string();
    for (key, classes) in navx {
        let cls = classes.as_str().unwrap_or("").trim().to_string();
        if cls.is_empty() {
            continue;
        }
        let (href, item_only) = match key.as_str() {
            _ if key.starts_with("home/") => (format!("{site}/{}/", &key[5..]), false),
            _ => match key.split_once('/') {
                Some(("post", s)) => (format!("{site}/{s}/"), false),
                Some(("page", s)) => {
                    // the front page's menu object is the site root, not
                    // its literal slug permalink.
                    let p = if s == "homepage" { String::new() } else { format!("{s}/") };
                    (format!("{site}/{p}"), false)
                }
                Some((tax, s)) => {
                    /* Menu hrefs are canonical now — resolve the term the
                       same way term_canonical_path does (category nests,
                       post_tag lives under the theme tag base, other
                       taxonomies stay {tax}/{slug}). */
                    let t = tax.to_lowercase();
                    let theme = crate::plugins::theme_for_site(conn);
                    let base = if t == "category" {
                        crate::plugins::theme_cat_path(conn, theme, s)
                    } else if t == "post_tag" {
                        format!("{}/{}", crate::plugins::theme_tag_base(conn, theme), s)
                    } else {
                        format!("{t}/{s}")
                    };
                    let cfg = crate::lang::config(conn);
                    let prefix = crate::lang::url_prefix(&cfg, lang);
                    (format!("{site}{prefix}/{base}/"), true)
                },
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
    let tree: Value = serde_json::from_str(&body_json).context("body_json")?;
    // Lazy media_map: fetch only the attachment ids this post's body_json
    // references (featured image resolves per-query in chrome_vars).
    let media = {
        let mut ids = std::collections::HashSet::new();
        collect_media_ids(&tree, &mut ids);
        media_map_for(conn, &ids)?
    };

    // current-post context for elements that read post content/meta
    let mut post = Map::new();
    post.insert("id".into(), json!(id));
    post.insert("title".into(), json!(title));
    let body_html: String = conn
        .query_row(sql!("render__select_body_html"), [id], |r| {
            r.get(0)
        })
        .unwrap_or_default();
    post.insert("body_html".into(), json!(body_html));
    {
        let mut mstmt = conn.prepare(sql!("render__select_public_meta"))?;
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
    let theme_dir = theme_template_dir(&conn, templates);
    let mut env = load_env_theme(templates, theme_dir.as_deref())?;
    register_helpers(&mut env, &media, site_url);
    let site = site_data_cached(&conn, &lang, site_url, tour_base)?;
    let body = render_nodes(&mut env, &tree, &media, site_url, &site, &lang, &post_v, Some(&conn), true, tour_base)?;

    // Adapter-owned kinds get their full single-page scaffolding from the
    // owning plugin; built-in kinds (post) keep dedicated core modules.
    if let Some(adapter) = crate::plugins::adapter_for_kind(&kind) {
        let ctx = crate::plugins::RenderCtx {
            slug: slug.to_string(),
            lang: lang.clone(),
            title: title.clone(),
            id,
            site_url: site_url.to_string(),
            body_html: body_html.clone(),
            body_html_rendered: body.clone(),
            kind: kind.clone(),
            kind_base: tour_base.to_string(),
        };
        if let Some(html) = adapter.render(&conn, templates, &ctx)? {
            return Ok(html);
        }
    }
    match kind.as_str() {
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

    // Non-adapter kinds rendered by this generic path = builder pages.
    // WP `traveltour_page_options`: `show-content` prepends the_content
    // (post_content through the core filter) above the page-builder body;
    // `sidebar` right/left wraps both in the theme's content-container +
    // sidebar-wrap scaffolding that `traveltour_content_with_sidebar`
    // emits. Pages without these options keep the bare builder body.
    let mut body = body;
    if kind == "page" {
        let page_opts_raw: String = conn
            .query_row(
                sql!("render__select_page_option"),
                [id],
                |r| r.get(0),
            )
            .unwrap_or_default();
        let page_opts = crate::php_serialize::unserialize(page_opts_raw.as_bytes()).ok();
        let opt = |k: &str| {
            page_opts
                .as_ref()
                .and_then(|v| v.get(k))
                .and_then(|x| x.as_str().map(str::to_string))
                .unwrap_or_default()
        };
        if opt("show-content") == "enable" && !body_html.trim().is_empty() {
            let article = crate::render_post::filter_content(&body_html, site_url, id, &conn);
            body = format!("{article}{body}");
        }
        let sidebar_pos = opt("sidebar");
        if sidebar_pos == "right" {
            // Page sidebars use the same blog-family regions as the post
            // template (traveltour `{pos}-sidebar-{left,right}` option or
            // the page's own `sidebar-{left,right}` override); WP sidebar
            // names map to imported region ids verbatim except the
            // theme-mapped blog family, which lives on sidebar-post-*.
            let sidebar_id = |k: &str| match opt(k).as_str() {
                "" | "blog" | "blog-left" | "blog-right" => format!("sidebar-post-{lang}"),
                s => s.to_string(),
            };
            if sidebar_pos == "right" {
                let sid = sidebar_id("sidebar-right");
                let sidebar_html = crate::widgets::render_sidebar(&conn, &sid, site_url, &lang, slug, "")?;
                // Theme-owned page scaffold: ThemeDef carries the exact
                // wrapper markup (traveltour's gdlr chain, default's
                // neutral divs) — no theme literals here.
                let theme = crate::plugins::theme_for_site(&conn);
                let area = (theme.wrap_page_sidebar)(theme, &body, &sidebar_html, "right");
                body = area;
            }
        }
    }

    let lang_cfg = crate::lang::config(conn);
    let mut alternates = translation_urls(conn, id, site_url, tour_base)?;
    let mut canonical = crate::lang::page_url(&lang_cfg, site_url, &lang, kind.as_str(), slug, crate::plugins::kind_url_base(tour_base, &kind).as_deref());
    // Front pages: WP permalinks them at the language root regardless of
    // stored slug; generate one alternate root per configured language.
    if lang_cfg.langs.iter().any(|l| front_id_for_lang(conn, l) == Some(id)) {
        canonical = crate::lang::page_url(&lang_cfg, site_url, &lang, "page", "", None);
        alternates = lang_cfg.langs.iter()
            .filter(|l| front_id_for_lang(conn, l).is_some())
            .map(|l| (l.clone(), crate::lang::page_url(&lang_cfg, site_url, l, "page", "", None)))
            .collect();
    }
    let hreflang_links = hreflang_links(&alternates);
    let base = crate::lang::template_name(templates, theme_dir.as_deref(), &lang_cfg, &lang);
    // page templates just wrap body; we inject into base content block directly
    let site_name = conn
        .query_row(
            sql!("render__select_site_name"),
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
        &alternates,
        tour_base,
    )?;
    // Templates use `seo_title` inside <title> and meta content attributes:
    // pre-escape for attribute position (esc_attr covers text too — the
    // base has AutoEscape::None). `title` is escaped for the same reason.
    ctx.insert("seo_title".into(), json!(esc_attr(&seo_title)));
    ctx.insert("seo_desc".into(), json!(seo_desc));
    ctx.insert("canonical".into(), json!(canonical));
    ctx.insert("hreflang_links".into(), json!(hreflang_links));
    ctx.insert("lang".into(), json!(lang));
    ctx.insert("title".into(), json!(esc(&title)));
    ctx.insert("site_url".into(), json!(site_url));
    ctx.insert("body".into(), json!(body));
    ctx.insert("base".into(), json!(base));

    let navx = match ctx.get("navx") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };
    let html = env
        .get_template(&base)?
        .render(JVal::from_serialize(Value::Object(ctx)))
        .map_err(|e| anyhow::anyhow!("render {base}: {e}"))?;
    Ok(apply_navx(&html, site_url, &navx, conn, &lang))
}

/// Front-page post id for the site's primary language (`page_on_front`,
/// publish-gated by `generate__select_cast_value`).
pub(crate) fn front_id(conn: &Connection) -> Option<i64> {
    front_id_for_lang(conn, &crate::lang::config(conn).primary)
}


/// Localized front page for `lang`, deterministic and publish-only:
///   1. the primary `page_on_front`'s translation-group member in `lang`;
///   2. Polylang's `__trashed` localized front — `ORDER BY id LIMIT 1`
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
                sql!("render__select_front_group_member"),
                rusqlite::params![front, lang],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        if grouped.is_some() {
            return grouped;
        }
        // post_translations retired (phase 4) — the group arm above is the
        // only pairing left.
    }
    // Generic fallback: Polylang often keeps the localized static front as
    // a publish page whose slug is a trashed artifact.
    conn.query_row(
        sql!("render__select_trashed_front"),
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
        .query_row(sql!("render__select_slug"), [id], |r| r.get(0))
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
            sql!("render__select_sibling_slug"),
            rusqlite::params![slug, lang, kind],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()
        .ok()
        .flatten()?;
    // Prefer the group's primary-lang member when the post is grouped —
    // its permalink is the canonical one. Adapter-owned kinds permalink
    // under their display slug (meta-derived), not the stored slug.
    let target = conn
        .query_row(
            sql!("render__select_group_member"),
            rusqlite::params![id, cfg.primary],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?)),
        )
        .optional()
        .ok()
        .flatten()
        .map(|(_pid, l, k, s)| (l, k, s))
        .unwrap_or((plang, pkind, pslug));
    let (tlang, tkind, tslug) = target;
    let url_slug = match crate::plugins::adapter_for_kind(&tkind) {
        Some(adapter) => {
            // Adapter-owned kinds permalink under a canonical display slug,
            // not the stored slug — resolve the row first.
            let tid: i64 = conn
                .query_row(sql!("render__select_id_by_slug_lang_kind"),
                    rusqlite::params![tslug, tlang, tkind], |r| r.get(0))
                .unwrap_or(id);
            let title: String = conn
                .query_row(sql!("render__select_title"), [tid], |r| r.get(0))
                .unwrap_or_default();
            adapter.display_slug(conn, tid, &tslug, &title).unwrap_or(tslug)
        }
        None => tslug,
    };
    Some(crate::lang::page_url(cfg, "", &tlang, &tkind, &url_slug, crate::plugins::kind_url_base(tour_base, &tkind).as_deref()))
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

/// Template environments cached per templates-dir. `Environment::clone`
/// shares the compiled-template MemoMap, so one compile per template per
/// process; per-render helpers still register on the clone. Busted by
/// `env_cache_bust` (admin bust_cache/template edit paths) — a stale
/// Environment would serve pre-edit template output.
static ENVS: std::sync::LazyLock<parking_lot::RwLock<std::collections::HashMap<(std::path::PathBuf, Option<std::path::PathBuf>), Environment<'static>>>> =
    std::sync::LazyLock::new(parking_lot::RwLock::default);

/// Drop all cached environments — call from every cache-bust path.
pub(crate) fn env_cache_bust() {
    ENVS.write().clear();
}

/// Templates dir probed before the shared `templates/` root for the
/// given site — theme adapter's `template_dir` under `templates/theme/`
/// when `settings.stylesheet` matches a registered theme; `None`
/// otherwise (or no adapter match — falls back to shared only).
pub(crate) fn theme_template_dir(conn: &Connection, templates: &Path) -> Option<std::path::PathBuf> {
    let stylesheet = conn
        .query_row(sql!("render__select_stylesheet"), [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap_or_default();
    // Same resolution as `theme_for_site` minus the DB re-read: adapter
    // themes first, TOML theme specs second, neutral default last.
    let def = crate::plugins::theme_for_stylesheet(&stylesheet)
        .or_else(|| crate::plugins::themes::theme_for_stylesheet(&stylesheet))
        .unwrap_or(&crate::plugins::DEFAULT_THEME);
    // The matched theme's dir only wins when it actually ships base
    // templates — an empty dir (tevily stub until phase 06) falls back
    // to the default theme's bases (traveltour), matching the interim
    // behavior the phase-03 plan calls for.
    // Any base_* in the theme dir counts — theme dirs are per-theme and
    // may ship only the langs their sites use (tevily: base_en.html).
    let has_bases = |d: &Path| {
        std::fs::read_dir(d)
            .map(|rd| {
                rd.flatten().any(|e| {
                    e.file_name().to_string_lossy().starts_with("base_")
                        && e.path().is_file()
                })
            })
            .unwrap_or(false)
    };
    let themed = templates.join("theme").join(def.template_dir);
    if has_bases(&themed) {
        return Some(themed);
    }
    // Neutral default first (the shipped fallback theme), then the
    // traveltour dir as legacy safety for pre-phase-5 sites whose data
    // predates the neutral theme shipping.
    for dir in [crate::plugins::DEFAULT_THEME.template_dir, "traveltour"] { // gate-allow: legacy fallback for pre-phase-5 sites
        let d = templates.join("theme").join(dir);
        if has_bases(&d) {
            return Some(d);
        }
    }
    None
}

pub(crate) fn load_env(dir: &Path) -> Result<Environment<'static>> {
    load_env_theme(dir, None)
}

/// `load_env` with a theme probe dir: `theme/<dir>` first, shared root
/// second. The env cache keys on (dir, theme_dir) — two sites sharing a
/// templates root but different themes get distinct environments.
pub(crate) fn load_env_theme(dir: &Path, theme_dir: Option<&Path>) -> Result<Environment<'static>> {
    let key = (dir.to_path_buf(), theme_dir.map(|p| p.to_path_buf()));
    if let Some(env) = ENVS.read().get(&key) {
        return Ok(env.clone());
    }
    let mut env = Environment::new();
    let theme = theme_dir.map(|p| p.to_path_buf());
    let shared = dir.to_path_buf();
    env.set_loader(move |name: &str| {
        if let Some(t) = &theme {
            let p = t.join(name);
            if let Ok(src) = std::fs::read_to_string(&p) {
                return Ok(Some(src));
            }
        }
        match std::fs::read_to_string(shared.join(name)) {
            Ok(src) => Ok(Some(src)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("read {name}: {e}"),
            )),
        }
    });
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    env.add_filter("tojson", |v: minijinja::Value| {
        serde_json::to_string(&serde_json::to_value(v).unwrap_or(Value::Null))
            .unwrap_or_else(|_| "null".into())
    });
    ENVS.write().entry(key).or_insert_with(|| env.clone());
    Ok(env)
}



/// `media_map` for a bounded id set — one `IN` query instead of a whole-
/// table materialization (the lazy media_map: cache-miss renders fetch only
/// the media rows the rendered post references).
pub(crate) fn media_map_for(conn: &Connection, ids: &std::collections::HashSet<i64>) -> Result<HashMap<i64, (String, String)>> {
    let mut media: HashMap<i64, (String, String)> = HashMap::new();
    if ids.is_empty() {
        return Ok(media);
    }
    // Chunk under SQLite's SQLITE_MAX_VARIABLE_NUMBER.
    for chunk in ids.iter().copied().collect::<Vec<_>>().chunks(400) {
        let qs = std::iter::repeat_n("?", chunk.len()).collect::<Vec<_>>().join(",");
        let q = format!("{}WHERE id IN ({qs})", sql!("render__select_id_path_2"));
        let mut stmt = conn.prepare(&q)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(chunk.iter()),
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)),
        )?;
        for row in rows {
            let (mid, path, alt) = row?;
            media.insert(mid, (path, alt));
        }
    }
    Ok(media)
}

/// Single media row lookup — (path, alt). For callers that already know
/// the id and would otherwise materialize a whole map (lazy media_map).
pub(crate) fn media_lookup(conn: &Connection, id: i64) -> Option<(String, String)> {
    conn.query_row(
        sql!("render__select_media_path_alt"),
        [id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )
    .ok()
}

/// Node-value keys that carry media attachment ids (templates feed them to
/// `media_url`/`media_alt`/`css_url`; `render_node` resolves `image`).
const MEDIA_VALUE_KEYS: &[&str] = &[
    "image",
    "icon",
    "custom-image",
    "background-image",
    "b-image",
    "b-background-image",
    "left-image",
    "mobile-background-image",
    "container-background-image",
    "background-image-overlay",
    "background-video-image",
];

/// Walk a page-builder body_json tree and collect every media attachment id
/// it references — numeric values under `MEDIA_VALUE_KEYS` plus `{"id":N}`
/// entries inside `"gallery"`/`"featured-gallery"` item arrays (the shape
/// featured-gallery.html feeds to `media_url(image_id)`).
pub(crate) fn collect_media_ids(v: &Value, out: &mut std::collections::HashSet<i64>) {
    let num_id = |x: &Value| -> Option<i64> {
        match x {
            Value::Number(n) => n.as_i64(),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }
    };
    match v {
        Value::Array(a) => a.iter().for_each(|x| collect_media_ids(x, out)),
        Value::Object(m) => {
            for (k, x) in m {
                if MEDIA_VALUE_KEYS.contains(&k.as_str()) {
                    if let Some(id) = num_id(x) {
                        out.insert(id);
                    }
                }
                if k == "gallery" || k == "featured-gallery" {
                    if let Value::Array(items) = x {
                        for it in items {
                            if let Some(id) = it.get("id").and_then(&num_id) {
                                out.insert(id);
                            }
                        }
                    }
                }
                collect_media_ids(x, out);
            }
        }
        _ => {}
    }
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
            sql!("render__test_insert_post"),
            rusqlite::params![id, slug, lang, status, kind],
        )
        .unwrap();
    }

    #[test]
    fn front_id_prefers_group_member_and_is_deterministic() {
        let c = memdb();
        c.execute_batch(
            sql!("render__test_seed_settings"),
        )
        .unwrap();
        post(&c, 10, "accueil", "fr", "publish", "page");
        post(&c, 11, "inicio", "es", "publish", "page");
        // Translation group → ES member wins over the __trashed fallback.
        c.execute_batch(
            sql!("render__test_seed_group_trashed"),
        )
        .unwrap();
        assert_eq!(front_id_for_lang(&c, "es"), Some(11));
        assert_eq!(front_id_for_lang(&c, "fr"), Some(10));
        // Group gone → the __trashed fallback picks the lowest publish id.
        c.execute_batch(sql!("render__test_delete_group_members")).unwrap();
        assert_eq!(front_id_for_lang(&c, "es"), Some(12));
    }

    #[test]
    fn front_id_skips_unpublished() {
        let c = memdb();
        c.execute_batch(
            sql!("render__test_seed_settings"),
        )
        .unwrap();
        post(&c, 10, "accueil", "fr", "draft", "page");
        post(&c, 11, "inicio", "es", "draft", "page");
        c.execute(sql!("render__test_insert_group"), []).unwrap();
        assert_eq!(front_id_for_lang(&c, "fr"), None);
        // Draft group member must not leak; no publish __trashed page → None.
        assert_eq!(front_id_for_lang(&c, "es"), None);
        post(&c, 12, "x__trashed", "es", "draft", "page");
        post(&c, 13, "y__trashed", "es", "publish", "page");
        assert_eq!(front_id_for_lang(&c, "es"), Some(13));
    }
}
