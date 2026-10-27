//! Single post page renderer (kind='post').
//!
//! Reproduces the traveltour single.php scaffold around the page-builder
//! body: hero banner with featured image + date/category head, RDFa
//! breadcrumbs, right-sidebar layout (article content + page-builder),
//! tags / prev-next nav, and the wpDiscuz comments block — all extracted
//! from templates/reference/post.html (post 34896 capture).
//!
//! `page()` is handed the post row plus `body_html_rendered` (body_json
//! already run through render_nodes with the wpdiscuz token below already
//! substituted) by the caller.

#[allow(unused_imports)]
use crate::sql;
use anyhow::Result;
use minijinja::context;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::path::Path;

use crate::render::load_env;

/// Placeholder the caller plants in place of the `[wpdiscuz_comments]`
/// shortcode inside body_json text-boxes before rendering; `page()`
/// replaces it with the real comments block.
pub const WPDISCUZ_TOKEN: &str = "\u{0}WPDISCUZ_COMMENTS\u{0}";

/// Substitute the wpdiscuz comment shortcode in the rendered page-builder
/// body. Callers pass `body_html_rendered` straight from render_nodes;
/// `[wpdiscuz_comments]` becomes this token which `page()` swaps for the
/// real comments block.
pub fn substitute_comments_token(rendered: &str) -> String {
    // The wpDiscuz output replaces the whole <p>[wpdiscuz_comments]</p>
    // (gdlr strips the empty p) with a clearing div + comments markup.
    let t = rendered.replace(
        "<p>[wpdiscuz_comments]</p>",
        &format!("    <div class=\"wpdiscuz_top_clearing\"></div>\n    {WPDISCUZ_TOKEN}"),
    );
    t.replace("[wpdiscuz_comments]", &format!("<div class=\"wpdiscuz_top_clearing\"></div>\n{WPDISCUZ_TOKEN}"))
}

/// Context handed in by the caller: the post row plus its already-rendered
/// page-builder body (`body_html_rendered`).
pub struct PostCtx {
    pub slug: String,
    pub lang: String,
    pub title: String,
    pub id: i64,
    pub site_url: String,
    /// Raw WP post_content (already the_content-filtered by `filter_content`
    /// here — pass it raw, this module applies wpautop/shortcodes/ez-toc).
    pub body_html: String,
    /// body_json rendered to element HTML by render_nodes (top_level).
    pub body_html_rendered: String,
    pub kind: String,
    /// URL segment for the tour CPT (threaded from SiteEnv/CLI).
    pub tour_base: String,
}

/// Render one post single page.
pub(crate) fn page(conn: &Connection, templates: &Path, ctx: &PostCtx) -> Result<String> {
    let site_url = ctx.site_url.trim_end_matches('/').to_string();
    let site = site_url.as_str();
    let lang_cfg = crate::lang::config(conn);

    // --- post row bits we still need -------------------------------------
    let (published, featured_id, seo_title, seo_desc): (
        String,
        Option<i64>,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(sql!("render_post__select_post_row"), [ctx.id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .unwrap_or_default();

    // Lazy media_map: only the featured image (+ _thumbnail_id fallback)
    // this post references — never the whole media table.
    let featured_img = featured_id
        .and_then(|mid| crate::render::media_lookup(conn, mid))
        .map(|(p, _)| format!("{site}/wp-content/uploads/{p}"))
        .unwrap_or_default();
    // Theme falls back to the post-thumbnail metabox image even when
    // featured_media_id is null; _thumbnail_id mirrors it.
    let featured_img = if featured_img.is_empty() {
        conn.query_row(
            sql!("render_post__select_thumbnail_id"),
            [ctx.id],
            |r| r.get::<_, i64>(0),
        )
        .ok()
        .and_then(|mid| crate::render::media_lookup(conn, mid))
        .map(|(p, _)| format!("{site}/wp-content/uploads/{p}"))
        .unwrap_or_default()
    } else {
        featured_img
    };

    // --- terms ------------------------------------------------------------
    let mut cats: Vec<(String, String)> = Vec::new(); // (slug, name)
    let mut tags: Vec<(String, String)> = Vec::new();
    {
        let mut st = conn.prepare(sql!("render_post__select_post_terms"))?;
        let rows = st.query_map([ctx.id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        for r in rows.flatten() {
            match r.0.as_str() {
                "category" => cats.push((r.1, r.2)),
                "post_tag" => tags.push((r.1, r.2)),
                _ => {}
            }
        }
    }
    cats.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

    // Category links like the theme: alphabetical, comma-separated, each
    // pointing at its hierarchical archive path.
    let cat_links: Vec<serde_json::Value> = cats
        .iter()
        .map(|(slug, name)| {
            json!({
                "url": format!("{site}{}/{}/", crate::lang::url_prefix(&lang_cfg, &ctx.lang), crate::admin::plugins::theme_traveltour::cat_path(conn, slug)),
                "name": esc(name),
            })
        })
        .collect();
    let cat_classes: String = cats
        .iter()
        .map(|(slug, _)| format!(" category-{slug}"))
        .collect::<Vec<_>>()
        .join("");

    // Breadcrumb chain: deepest category → walk parents to root.
    let bc_chain = breadcrumb_chain(conn, &cats);
    let post_url = format!("{site}{}/{}/", crate::lang::url_prefix(&lang_cfg, &ctx.lang), ctx.slug);
    let site_name = conn
        .query_row(
            "SELECT value FROM settings WHERE key='site_name' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "rpress".into());
    let mut crumbs: Vec<Value> = vec![json!({
        "url": site,
        "class": "home",
        "title": format!("Go to {}.", site_name),
        "name": site_name,
    })];
    for (depth, (_slug, name)) in bc_chain.iter().enumerate() {
        // path prefix: root cat has no parent dir; children nest under parents
        let path = bc_chain
            .iter()
            .take(depth + 1)
            .map(|(s, _)| s.clone())
            .collect::<Vec<_>>()
            .join("/");
        crumbs.push(json!({
            "url": format!("{site}{}/{path}/", crate::lang::url_prefix(&lang_cfg, &ctx.lang)),
            "class": "taxonomy category",
            "title": esc_attr(&format!("Go to the {name} Catégorie archives.")),
            "name": esc(name),
        }));
    }

    // --- dates ------------------------------------------------------------
    let (day, month, year, date_url, date_label) = fr_date_parts(&published);

    // --- prev / next -------------------------------------------------------
    let prev_slug: Option<String> = conn
        .query_row(
            sql!("render_post__select_prev"),
            rusqlite::params![ctx.kind, ctx.lang, published],
            |r| r.get(0),
        )
        .ok();
    let next_slug: Option<String> = conn
        .query_row(
            sql!("render_post__select_next"),
            rusqlite::params![ctx.kind, ctx.lang, published],
            |r| r.get(0),
        )
        .ok();
    let prev_url = prev_slug.map(|s| format!("{site}{}/{s}/", crate::lang::url_prefix(&lang_cfg, &ctx.lang)));
    let next_url = next_slug.map(|s| format!("{site}{}/{s}/", crate::lang::url_prefix(&lang_cfg, &ctx.lang)));

    // --- comments -----------------------------------------------------------
    let comments = load_comments(conn, ctx.id)?;
    // wpDiscuz Ajax-hydrates its thread on the WP site; on the static build
    // the whole approved thread renders server-side (no pagination — emit
    // all, it costs bytes not queries). wpdiscuz DOM shape, ul/li nested by
    // parent_id, oldest-first per level (the reference default sort).
    let comment_page = format!("{}/{}/", crate::lang::url_prefix(&lang_cfg, &ctx.lang), ctx.slug);
    let comments_open = conn
        .query_row(
            sql!("intake__select_post_published"),
            [ctx.id],
            |r| r.get::<_, String>(0),
        )
        .map(|published| {
            let dcs = crate::intake::setting(conn, "default_comment_status", "open");
            if dcs == "closed" || dcs == "0" {
                return false;
            }
            let days: i64 = crate::intake::setting(conn, "close_comments_days_old", "14")
                .parse()
                .unwrap_or(14);
            !(crate::intake::setting(conn, "close_comments", "0") == "1"
                && days > 0
                && crate::intake::days_old(&published) > days)
        })
        .unwrap_or(true);
    let comments_html = render_thread(&comments, conn, ctx.id, &comment_page, comments_open);
    let rating: String = conn
        .query_row(
            sql!("render_post__select_rating"),
            rusqlite::params![ctx.id],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "0".into());
    let rating = rating.parse::<f64>().unwrap_or(0.0);
    let rating_count: i64 = conn
        .query_row(
            sql!("render_post__select_rating_count"),
            rusqlite::params![ctx.id],
            |r| r.get::<_, String>(0),
        )
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let env = load_env(templates)?;
    let comments_block = env
        .get_template("comments_wpdiscuz.html")?
        .render(context! {
            site_url => site,
            post_id => ctx.id,
            post_url => format!("{}/{}/", crate::lang::url_prefix(&lang_cfg, &ctx.lang), ctx.slug),
            post_url_enc => urlenc(&post_url),
            comments_open => comments_open,
            rating_value => fmt_rating(rating),
            rating_count => rating_count,
            comments_count => comments.len(),
            comments_html => comments_html,
        })
        .map_err(|e| anyhow::anyhow!("render comments_wpdiscuz.html: {e}"))?;

    // --- article content (the_content pipeline) ------------------------------
    let article_html = filter_content(&ctx.body_html, site, ctx.id, conn);

    // --- seo / translations ---------------------------------------------------
    let canonical = post_url.clone();
    let alternates = translation_urls(conn, ctx.id, site, &ctx.tour_base);
    let hreflang_links = hreflang_links(&alternates);
    let seo_title = seo_title
        .filter(|s| !s.is_empty())
        .map(|t| crate::db::resolve_seo(&t, &ctx.title, &site_name))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ctx.title.clone());
    let seo_desc = seo_desc
        .map(|d| crate::db::resolve_seo(&d, &ctx.title, &site_name))
        .unwrap_or_default();

    let tokenized = substitute_comments_token(&ctx.body_html_rendered);
    let body_rendered = tokenized.replace(WPDISCUZ_TOKEN, &comments_block);
    // single.php emits the comments template a second time after the nav
    // (the in-content copy comes from the [wpdiscuz_comments] text-box);
    // the site's CSS hides the first #comments, so both stay in the markup.
    let after_nav = format!(
        "    <div class=\"wpdiscuz_top_clearing\"></div>\n{comments_block}"
    );

    // --- shared head/chrome vars (Phase 2: meta block, links, nav, flags) --
    let mut tctx = crate::render::chrome_vars(
        conn,
        site,
        ctx.id,
        &ctx.kind,
        &ctx.slug,
        &ctx.lang,
        &ctx.title,
        &canonical,
        &seo_desc,
        &ctx.body_html,
        &alternates,
        &ctx.tour_base,
    )?;
    // `seo_title`/`title`/term names go into <title>, meta content="…"
    // attributes and text nodes — pre-escape at the boundary (templates run
    // AutoEscape::None; `|safe` on names marks them intentionally raw).
    let sidebar_post = crate::widgets::render_sidebar(conn, "sidebar-post", site, &ctx.lang, &ctx.slug, &ctx.tour_base)?;
    let tag_base = crate::admin::plugins::theme_traveltour::tag_base(conn);
    for (k, v) in serde_json::json!({
        "seo_title": esc_attr(&seo_title),
        "seo_desc": seo_desc,
        "canonical": canonical,
        "hreflang_links": hreflang_links,
        "lang": ctx.lang,
        "site_url": site,
        "id": ctx.id,
        "title": esc(&ctx.title),
        "slug": ctx.slug,
        "post_url": post_url,
        "featured_img": featured_img,
        "day": day,
        "month": month,
        "year": year,
        "date_url": date_url,
        "date_label": date_label,
        "cat_links": cat_links,
        "cat_classes": cat_classes,
        "crumbs": crumbs,
        "article_html": article_html,
        "body": body_rendered,
        "tags": tags.iter().map(|(slug, name)| json!({
            "url": format!("{site}{}/{tag_base}/{slug}/", crate::lang::url_prefix(&lang_cfg, &ctx.lang)),
            "name": esc(name),
        })).collect::<Vec<_>>(),
        "prev_url": prev_url,
        "next_url": next_url,
        "comments_after_nav": after_nav,
        "sidebar_post": sidebar_post,
    })
    .as_object()
    .into_iter()
    .flatten()
    {
        tctx.insert(k.clone(), v.clone());
    }

    let navx = match tctx.get("navx") {
        Some(Value::Object(m)) => m.clone(),
        _ => serde_json::Map::new(),
    };
    let html = env
        .get_template("single_post.html")?
        .render(minijinja::Value::from_serialize(Value::Object(tctx)))
        .map_err(|e| anyhow::anyhow!("render single_post.html: {e}"))
        .map(|html| inject_post_assets(&html, site, ctx.id))?;
    Ok(crate::render::apply_navx(&html, site, &navx, conn, &ctx.lang))
}

/// Post-kind-only assets the frozen base templates lack, extracted verbatim
/// from the reference single-post capture (templates/post_assets.txt):
/// ez-toc + wpDiscuz head CSS, their footer scripts, comment-reply.js, and
/// the hidden wpDiscuz tail markup (wpdUserContentInfo et al.).
fn inject_post_assets(html: &str, site: &str, post_id: i64) -> String {
    const ASSETS: &str = include_str!("../templates/post_assets.txt");
    let mut sections = std::collections::HashMap::new();
    let parts: Vec<&str> = ASSETS.split("===").collect();
    // `===NAME===` markers alternate name/body around the === delimiter.
    // split("===") yields a leading "" before the first marker, so names
    // live at odd indices and bodies at even ones — skip the empty head.
    for pair in parts[1..].chunks(2) {
        if pair.len() == 2 {
            sections.insert(pair[0].trim().to_string(), pair[1].trim_start_matches('\n'));
        }
    }
    let mut out = html.to_string();
    let fill = |name: &str| -> String {
        sections
            .get(name)
            .copied()
            .unwrap_or("")
            .replace("{SITE}", site)
            .replace("{POST_ID}", &post_id.to_string())
    };
    // head CSS: eztoc before the theme core css, wpdiscuz before
    // tourmaster-custom (reference enqueue order).
    let head_ez = fill("HEAD_EZ");
    if !head_ez.is_empty() {
        if let Some(i) = out.find("<link rel='stylesheet' id='traveltour-style-core-css'") {
            out.insert_str(i, &head_ez);
        }
    }
    let head_wpd = fill("HEAD_WPD");
    if !head_wpd.is_empty() {
        if let Some(i) = out.find("<link rel='stylesheet' id='tourmaster-custom-style-css'") {
            out.insert_str(i, &head_wpd);
        }
    }
    // footer scripts, in reference order.
    let foot_ez = fill("FOOT_EZ");
    if !foot_ez.is_empty() {
        if let Some(i) = out.find(r#"<script id="traveltour-script-core-js-extra">"#) {
            out.insert_str(i, &foot_ez);
        }
    }
    let foot_reply = fill("FOOT_REPLY");
    if !foot_reply.is_empty() {
        if let Some(i) = out.find(r#"<script id="chaty-front-end-js-extra">"#) {
            out.insert_str(i, &foot_reply);
        }
    }
    let foot_wpd = fill("FOOT_WPD");
    if !foot_wpd.is_empty() {
        if let Some(i) = out.find(r#"<script id="tourmaster-tour-script-js""#) {
            out.insert_str(i, &foot_wpd);
        }
    }
    // hidden wpDiscuz tail markup before the footer script block.
    let tail = fill("TAIL");
    if !tail.is_empty() {
        if let Some(i) = out.find(r#"<script id="wp-hooks-js""#) {
            out.insert_str(i, &tail);
        }
    }
    out
}


/// `2025-08-21 11:38:47` → (21, "Août", 2025, /2025/08/21/, "août 21, 2025")
pub(crate) fn fr_date_parts(published: &str) -> (String, String, String, String, String) {
    const MONTHS_CAP: [&str; 12] = [
        "Janvier", "Février", "Mars", "Avril", "Mai", "Juin", "Juillet",
        "Août", "Septembre", "Octobre", "Novembre", "Décembre",
    ];
    const MONTHS_LOW: [&str; 12] = [
        "janvier", "février", "mars", "avril", "mai", "juin", "juillet",
        "août", "septembre", "octobre", "novembre", "décembre",
    ];
    let y = published.get(0..4).unwrap_or("");
    let m: usize = published.get(5..7).unwrap_or("1").parse().unwrap_or(1);
    let d = published.get(8..10).unwrap_or("01");
    let day = d.trim_start_matches('0');
    let day = if day.is_empty() { "0" } else { day };
    let m = m.clamp(1, 12);
    (
        day.to_string(),
        MONTHS_CAP[m - 1].to_string(),
        y.to_string(),
        format!("/{y}/{:02}/{d}/", m),
        format!("{} {}, {}", MONTHS_LOW[m - 1], day, y),
    )
}

/// Breadcrumb chain = full root→deepest path of the post's deepest category
/// (WP picks the primary/longest path). Segment list comes from the theme's
/// taxonomy map via `theme_traveltour::cat_path`.
fn breadcrumb_chain(conn: &Connection, cats: &[(String, String)]) -> Vec<(String, String)> {
    let mut best: Vec<(String, String)> = Vec::new();
    for (slug, name) in cats {
        let mut chain: Vec<(String, String)> = crate::admin::plugins::theme_traveltour::cat_path(conn, slug)
            .split('/')
            .map(|s| {
                let n = cats
                    .iter()
                    .find(|(cs, _)| cs == s)
                    .map(|(_, n)| n.clone())
                    .unwrap_or_else(|| cat_name(s));
                (s.to_string(), n)
            })
            .collect();
        // Deepest segment carries the post's own term name (cat_path only
        // knows slugs).
        if let Some(last) = chain.last_mut() {
            last.1 = name.clone();
        }
        if chain.len() > best.len() {
            best = chain;
        }
    }
    best
}

/// FR display names for parent categories not attached to the post itself.
pub(crate) fn cat_name(slug: &str) -> String {
    const N: &[(&str, &str)] = &[
        ("guide-de-voyage-vietnam", "Guide de voyage Vietnam"),
        ("le-vietnam", "Le Vietnam"),
        ("sites-a-explorer", "Sites à explorer"),
        ("idees-de-circuit", "Idées de circuit"),
        ("cuisine-vietnamienne", "Cuisine vietnamienne"),
        ("plats-vietnamiens", "Plats vietnamiens"),
        ("montagnes-du-nord", "Montagnes du Nord"),
        ("station-balneaire-vietnam", "Station balnéaire Vietnam"),
        ("vietnam-du-centre", "Vietnam du Centre"),
        ("vietnam-du-nord", "Vietnam du Nord"),
        ("vietnam-du-sud", "Vietnam du Sud"),
        ("voyages-au-cambodge", "Voyages au Cambodge"),
        ("guia-de-viaje-a-vietnam", "Guía de viaje a Vietnam"),
        ("viaje-a-vietnam", "Viaje a Vietnam"),
        ("sitios-para-explorar", "Lugares para explorar"),
        ("la-gastronomia-vietnamita", "La gastronomía vietnamita"),
        ("platos-vietnamitas", "Platos vietnamitas"),
        ("ideas-de-viajes", "Ideas de viajes"),
        ("norte-de-vietnam", "Norte de Vietnam"),
        ("resort-de-playa-vietnam", "Resort de playa Vietnam"),
        ("centro-de-vietnam", "Centro de Vietnam"),
        ("el-sur-de-vietnam", "El sur de Vietnam."),
    ];
    N.iter()
        .find(|(s, _)| *s == slug)
        .map(|(_, n)| n.to_string())
        .unwrap_or_else(|| slug.replace('-', " "))
}

struct Comment {
    id: i64,
    parent: i64,
    author: String,
    author_url: String,
    body: String,
    created: String,
    rating: i64,
    email: String,
}

fn load_comments(conn: &Connection, post_id: i64) -> Result<Vec<Comment>> {
    let mut st = conn.prepare(sql!("render_post__select_comments"))?;
    let rows = st.query_map([post_id], |r| {
        Ok(Comment {
            id: r.get(0)?,
            parent: r.get(1)?,
            author: r.get(2)?,
            author_url: r.get(3)?,
            body: r.get(4)?,
            created: r.get(5)?,
            rating: r.get(6)?,
            email: r.get(7)?,
        })
    })?;
    Ok(rows.flatten().collect())
}


/// Whole approved thread as nested markup: `<ul class="wpd-comment-list">`
/// of `<li>` items; each `<li>` holds the `.wpd-comment` div plus a nested
/// `<ul>` of its children (wpdiscuz's own nesting is div-in-div; the ul/li
/// wrappers carry the same descendant-class relationship so its CSS
/// selectors still apply). Ordered oldest-first like the reference sort;
/// orphans (parent not in the set) render at top level.
fn render_thread(
    comments: &[Comment],
    conn: &Connection,
    post_id: i64,
    page: &str,
    open: bool,
) -> String {
    use std::collections::HashMap;
    let mut kids: HashMap<i64, Vec<&Comment>> = HashMap::new();
    let ids: std::collections::HashSet<i64> = comments.iter().map(|c| c.id).collect();
    for c in comments {
        let p = if c.parent != 0 && ids.contains(&c.parent) {
            c.parent
        } else {
            0
        };
        kids.entry(p).or_default().push(c);
    }
    fn level(
        kids: &std::collections::HashMap<i64, Vec<&Comment>>,
        parent: i64,
        depth: u32,
        conn: &Connection,
        post_id: i64,
        page: &str,
        open: bool,
        out: &mut String,
    ) {
        let Some(list) = kids.get(&parent) else {
            return;
        };
        out.push_str("<ul class=\"wpd-comment-list\">");
        for c in list {
            out.push_str("<li class=\"wpd-comment-item\">");
            out.push_str(&comment_html(conn, c, depth));
            if open {
                out.push_str(&reply_form(c, post_id, page));
            }
            level(kids, c.id, depth + 1, conn, post_id, page, open, out);
            out.push_str("</li>");
        }
        out.push_str("</ul>");
    }
    let mut out = String::new();
    level(&kids, 0, 1, conn, post_id, page, open, &mut out);
    out
}

/// JS-free reply affordance: a `<details>`-wrapped mini form under every
/// comment. Posts the same intake contract as the main form, just with
/// `parent` set. `<details>` needs no script to open.
fn reply_form(c: &Comment, post_id: i64, page: &str) -> String {
    format!(
        "<details class=\"wpd-reply-details\">\
<summary class=\"wpd-reply-toggle\">Répondre</summary>\
<form class=\"wpd_comm_form wpd-reply-form\" method=\"post\" action=\"/comment\">\
<div class=\"wpdiscuz-item wc-field-textarea\"><textarea name=\"wc_comment\" \
class=\"wc_comment wpd-field\" required rows=\"3\" placeholder=\"Commentaire*\"></textarea></div>\
<div class=\"wpd-form-row\"><input type=\"text\" name=\"wc_name\" class=\"wc_name wpd-field\" \
required maxlength=\"50\" placeholder=\"Nom*\"/> \
<input type=\"email\" name=\"wc_email\" class=\"wc_email wpd-field\" placeholder=\"Email\"/></div>\
<input type=\"hidden\" name=\"post_id\" value=\"{post_id}\"/>\
<input type=\"hidden\" name=\"_page\" value=\"{page}\"/>\
<input type=\"hidden\" name=\"parent\" value=\"{parent}\"/>\
<input type=\"text\" name=\"_gotcha\" class=\"wpd-gotcha\" value=\"\" tabindex=\"-1\" \
autocomplete=\"off\" aria-hidden=\"true\"/>\
<div class=\"wc-field-submit\"><input type=\"submit\" class=\"wc_comm_submit wpd-prim-button\" \
name=\"submit\" value=\"Poster\"/></div>\
</form></details>",
        post_id = post_id,
        page = esc_attr(page),
        parent = c.id,
    )
}

/// One wpDiscuz thread item (layout-1). Reference pages capture zero
/// rendered comments (Ajax-loaded), so this is structural best-effort.
fn comment_html(conn: &Connection, c: &Comment, depth: u32) -> String {
    let is_admin = is_staff(conn, &c.author);
    let label = if is_admin { "wpd-blog-administrator" } else { "wpd-blog-guest" };
    let depth_class = if c.parent != 0 { " wpd-reply" } else { "" };
    let role_label = if is_admin {
        "<span class=\"wpd-comment-label\" >Administrateur</span>"
    } else {
        ""
    };
    // author URLs link only for http(s) — anything else (javascript:,
    // mailto, garbage) renders the name without the anchor (R13-b).
    let author_html = if c.author_url.starts_with("http://") || c.author_url.starts_with("https://") {
        format!("<a href=\"{}\" target=\"_blank\" rel=\"nofollow ugc\">{}</a>", esc_attr(&c.author_url), esc(&c.author))
    } else {
        esc(&c.author)
    };
    let rating_html = if c.rating > 0 {
        format!("<span class=\"wpd-comment-rating\" title=\"Note : {}/5\">{}/5</span>", c.rating, c.rating)
    } else {
        String::new()
    };
    let date = fr_comment_date(&c.created);
    format!(
        "<div id=\"comment-{id}\" class=\"wpd-comment wpd-thread-level-{depth}{depth_cls}\">\
<div class=\"wpd-comment-wrap {label}\">\
<div class=\"wpd-avatar\"><img alt='{author}' src='https://secure.gravatar.com/avatar/{grav}?s=56&amp;d=mm&amp;r=g' srcset='https://secure.gravatar.com/avatar/{grav}?s=112&amp;d=mm&amp;r=g 2x' class='avatar avatar-56 photo' height='56' width='56' loading='lazy' decoding='async'/></div>\
<div class=\"wpd-comment-right\">\
<div class=\"wpd-comment-header\">\
<div class=\"wpd-comment-author\">{author_html}{role_label}{rating}</div>\
<div class=\"wpd-comment-date\">{date}</div>\
</div>\
<div class=\"wpd-comment-content\">\
<div class=\"wpd-comment-text\">{body}</div>\
</div>\
<div class=\"wpd-comment-footer\"></div>\
</div>\
<div class=\"wpd-clear\"></div>\
</div></div>",
        id = c.id,
        depth = depth.min(10),
        depth_cls = depth_class,
        label = label,
        author = esc(&c.author),
        grav = gravatar_hash(&c.email, &c.author),
        author_html = author_html,
        role_label = role_label,
        rating = rating_html,
        date = date,
        body = wpautop_inline(&c.body),
    )
}

fn is_staff(conn: &Connection, author: &str) -> bool {
    let a = author.to_lowercase();
    if a.contains("admin") {
        return true;
    }
    conn.query_row(
        "SELECT 1 FROM users WHERE lower(display_name)=?1 OR lower(email)=?1 LIMIT 1",
        [a],
        |r| r.get::<_, i64>(0),
    )
    .is_ok()
}

/// Gravatar hash: the captured wpDiscuz markup carries 64-hex SHA256
/// digests (WP's modern `wp_get_avatar` gravatar path), so hash
/// `email.trim().lowercase()` with sha2 — NOT md5. Falls back to the
/// author name when the email column is empty.
fn gravatar_hash(email: &str, fallback: &str) -> String {
    use sha2::{Digest, Sha256};
    let seed = if email.trim().is_empty() {
        fallback.trim().to_lowercase()
    } else {
        email.trim().to_lowercase()
    };
    let mut h = Sha256::new();
    h.update(seed.as_bytes());
    format!("{:x}", h.finalize())
}

/// "2025-08-21 21:05:43" → "21 août 2025 21:05"
fn fr_comment_date(created: &str) -> String {
    const M: [&str; 12] = [
        "janvier", "février", "mars", "avril", "mai", "juin", "juillet",
        "août", "septembre", "octobre", "novembre", "décembre",
    ];
    let m: usize = created.get(5..7).unwrap_or("1").parse().unwrap_or(1);
    format!(
        "{} {} {} {}",
        created.get(8..10).unwrap_or("").trim_start_matches('0'),
        M[m.clamp(1, 12) - 1],
        created.get(0..4).unwrap_or(""),
        created.get(11..16).unwrap_or(""),
    )
}

/// Minimal wpautop for a comment body: \n\n → </p><p>, \n → <br/>.
fn wpautop_inline(s: &str) -> String {
    let mut out = String::new();
    for (i, para) in s.split("\n\n").enumerate() {
        if i > 0 {
            out.push_str("</p><p>");
        }
        out.push_str(&esc(para).replace('\n', "<br />"));
    }
    format!("<p>{out}</p>")
}

fn fmt_rating(r: f64) -> String {
    if r.fract().abs() < f64::EPSILON {
        format!("{}", r as i64)
    } else {
        format!("{r:.1}")
    }
}

fn urlenc(s: &str) -> String {
    crate::admin::urlenc(s)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;").replace('\'', "&#039;")
}

fn translation_urls(conn: &Connection, id: i64, site: &str, tour_base: &str) -> Vec<(String, String)> {
    let cfg = crate::lang::config(conn);
    let group: Option<String> = conn
        .query_row("SELECT group_id FROM post_translation_members WHERE post_id=?1 LIMIT 1", [id], |r| r.get(0))
        .ok();
    let Some(group) = group else { return Vec::new(); };
    let mut stmt = match conn.prepare("SELECT lang, post_id FROM post_translation_members WHERE group_id=?1 ORDER BY lang") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map([group], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|(lang, pid)| {
            conn.query_row(sql!("render__select_slug_kind"), [pid], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .ok()
                .map(|(slug, kind)| (lang.clone(), crate::lang::page_url(&cfg, site, &lang, &kind, &slug, crate::admin::plugins::kind_url_base(tour_base, &kind).as_deref())))
        })
        .collect()
}

fn hreflang_links(alternates: &[(String, String)]) -> String {
    alternates
        .iter()
        .map(|(lang, url)| format!("<link rel=\"alternate\" href=\"{}\" hreflang=\"{}\" />", esc_attr(url), esc_attr(lang)))
        .collect::<Vec<_>>()
        .join("\n")
}

/* ================= the_content equivalent ================= */

/// post_content → article HTML: entity-preserve, wpautop, wptexturize(FR),
/// caption/button/embed/icon shortcodes, then ez-toc anchors + container.
fn filter_content(raw: &str, site: &str, post_id: i64, conn: &Connection) -> String {
    let mut s = rewrite_origin(raw, site, conn);
    s = expand_body_shortcodes(&s, site);
    s = wpautop(&s);
    s = texturize(&s);
    s = link_rel_fixup(&s, site);
    s = rmwr_wrap(&s, post_id);
    ez_toc(&s)
}

/// Rewrite the imported site origin to the configured one. The source
/// domain lives in `settings.legacy_origin` (set at import time); an empty
/// value means "no rewrite" so rpress ships domain-free.
pub(crate) fn rewrite_origin(raw: &str, site: &str, conn: &Connection) -> String {
    let s = raw.replace("{{ site_url }}", site);
    let origin = crate::intake::setting(conn, "legacy_origin", "");
    if origin.is_empty() {
        s
    } else {
        let site = site.trim_end_matches('/');
        let origin = origin.trim_end_matches('/');
        s.replace(origin, site)
            .replace(&origin.replacen("https://", "http://", 1), site)
    }
}

/// `[read more]` → RMWR (Read More WPTabs) toggle: everything after the
/// tag is wrapped in a hidden `read_div` until the button is clicked.
/// WP plugin semantics: the tag is a delimiter, not a pair — no `[/]`.
fn rmwr_wrap(s: &str, post_id: i64) -> String {
    let Some(i) = s.find("[read more]") else {
        return s.to_string();
    };
    let (head, tail) = s.split_at(i);
    let tail = &tail["[read more]".len()..];
    let id = format!("rmwr-p{post_id}-1");
    format!(
        "{head}<div class=\"rmwr-wrapper\" data-id=\"{id}\" data-key=\"p{post_id}-1\" data-post=\"{post_id}\" data-animation=\"none\" data-duration=\"300\" data-mode=\"normal\" data-lazy=\"false\" data-smooth-scroll=\"true\" data-scroll-offset=\"0\">\
        <button type=\"button\" class=\"read-link\" id=\"readlink{id}\" data-open-text=\"Lire la suite ...\" data-close-text=\"Lire moins\" aria-expanded=\"false\" aria-controls=\"read{id}\" onclick=\"(function(b){{var d=document.getElementById(b.getAttribute('aria-controls')),o=b.getAttribute('aria-expanded')==='true';b.setAttribute('aria-expanded',!o);d.style.display=o?'none':'block';d.setAttribute('aria-hidden',o);b.querySelector('.rmwr-text').textContent=o?b.dataset.openText:b.dataset.closeText;}})(this)\">\
        <span class=\"rmwr-text\">Lire la suite ...</span></button>\
        <div class=\"read_div\" id=\"read{id}\" aria-hidden=\"true\" style=\"display:none;\">{tail}</div></div>"
    )
}

/// Shortcode expansion for the post-body subset. Unknown shortcodes are
/// stripped (unregistered in WP output); known ones emit the WP markup the
/// reference shows verbatim.
fn expand_body_shortcodes(s: &str, site: &str) -> String {
    let mut out = String::with_capacity(s.len() + 256);
    let b = s.as_bytes();
    let mut i = 0usize;
    let mut first_img = true;
    while i < b.len() {
        if b[i] == b'[' {
            if let Some(end) = s[i..].find(']') {
                let head = &s[i + 1..i + end];
                let tag = head.split_whitespace().next().unwrap_or("");
                match tag {
                    "caption" => {
                        if let Some(close) = s[i + end + 1..].find("[/caption]") {
                            let inner = &s[i + end + 1..i + end + 1 + close];
                            let a = sc_attrs(head);
                            let id = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            let num = id.trim_start_matches("attachment_");
                            let align = a.get("align").and_then(|v| v.as_str()).unwrap_or("aligncenter");
                            let w = a.get("width").and_then(|v| v.as_str()).unwrap_or("0");
                            let (img, cap) = split_caption(inner);
                            let img = if first_img {
                                first_img = false;
                                img.replace("<img ", "<img fetchpriority=\"high\" decoding=\"async\" ")
                            } else {
                                img.replace("<img ", "<img loading=\"lazy\" decoding=\"async\" ")
                            };
                            out.push_str(&format!(
                                "<figure id=\"{}\" aria-describedby=\"caption-attachment-{}\" style=\"width: {}px\" class=\"wp-caption {}\">{img}<figcaption id=\"caption-attachment-{}\" class=\"wp-caption-text\">{cap}</figcaption></figure>",
                                esc_attr(id), esc_attr(num), esc_attr(w), esc_attr(align), esc_attr(num)
                            ));
                            i += end + 1 + close + "[/caption]".len();
                            continue;
                        }
                    }
                    "gdlr_core_button" => {
                        let a = sc_attrs(head);
                        let gs = |k: &str| a.get(k).and_then(|v| v.as_str()).unwrap_or("");
                        let text = gs("button-text");
                        let link = gs("button-link").replace("{{ site_url }}", site);
                        let target = gs("button-link-target").replace("{{ site_url }}", site);
                        let mut style = String::new();
                        for (k, css) in [
                            ("margin-right", "margin-right"),
                            ("margin-left", "margin-left"),
                            ("margin-bottom", "margin-bottom"),
                            ("margin-center", "margin-center"),
                            ("padding", "padding"),
                        ] {
                            let v = gs(k);
                            if !v.is_empty() && v != "0px" {
                                style.push_str(&format!("{css}: {v};"));
                            }
                        }
                        let tgt = if target.is_empty() {
                            String::new()
                        } else {
                            format!(" target=\"{}\"", esc_attr(&target))
                        };
                        let st = if style.is_empty() {
                            String::new()
                        } else {
                            format!(" style=\"{style}\"")
                        };
                        out.push_str(&format!(
                            "<a class=\"gdlr-core-button gdlr-core-button-shortcode  gdlr-core-button-gradient gdlr-core-button-no-border\" href=\"{}\"{tgt}{st}><span class=\"gdlr-core-content\"  >{}</span></a>",
                            esc_attr(&link),
                            esc(text)
                        ));
                        i += end + 1;
                        continue;
                    }
                    "embed" => {
                        if let Some(close) = s[i + end + 1..].find("[/embed]") {
                            let inner = &s[i + end + 1..i + end + 1 + close];
                            out.push_str(&embed_html(inner));
                            i += end + 1 + close + "[/embed]".len();
                            continue;
                        }
                    }
                    "gdlr_core_icon" | "gdlr_widget_box" => {
                        // widget_box wraps nested shortcodes — expand them
                        // recursively rather than passing the brackets
                        // through as literal text.
                        if tag == "gdlr_widget_box" {
                            if let Some(close) = s[i + end + 1..].find("[/gdlr_widget_box]") {
                                let inner = &s[i + end + 1..i + end + 1 + close];
                                out.push_str(&expand_body_shortcodes(inner, site));
                                i += end + 1 + close + "[/gdlr_widget_box]".len();
                                continue;
                            }
                        }
                        i += end + 1;
                        continue;
                    }
                    _ => {
                        // Unknown shortcode: WP leaves unregistered tokens as
                        // literal text — keep the bracket verbatim (Tailwind
                        // snippets like `min-h-[20px]` must survive).
                        out.push_str(&s[i..i + end + 1]);
                        i += end + 1;
                        continue;
                    }
                }
            }
        }
        // Multi-byte safe copy: `b[i] as char` would decode UTF-8
        // continuation bytes as Latin-1 → mojibake.
        out.push_str(&s[i..i + utf8_len(b[i])]);
        i += utf8_len(b[i]);
    }
    out
}

/// `[caption]` inner: "<img …> caption text" → (img, caption).
fn split_caption(inner: &str) -> (String, String) {
    let t = inner.trim();
    if let Some(end) = t.find("/>") {
        let (img, rest) = t.split_at(end + 2);
        (img.to_string(), rest.trim().to_string())
    } else {
        (t.to_string(), String::new())
    }
}

fn embed_html(url: &str) -> String {
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
            return format!(
                "<iframe title=\"YouTube video\" width=\"1150\" height=\"647\" src=\"https://www.youtube.com/embed/{id}?feature=oembed\" frameborder=\"0\" allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share\" referrerpolicy=\"strict-origin-when-cross-origin\" allowfullscreen></iframe>"
            );
        }
    }
    u.to_string()
}

/// `[tag attr="v" attr2='v']` attribute parser (same shape as render.rs).
fn sc_attrs(s: &str) -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        while i < b.len() && (b[i].is_ascii_whitespace() || b[i] == b'[') {
            i += 1;
        }
        let ks = i;
        while i < b.len() && b[i] != b'=' && !b[i].is_ascii_whitespace() && b[i] != b']' {
            i += 1;
        }
        let key = &s[ks..i];
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < b.len() && b[i] == b'=' {
            i += 1;
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < b.len() && (b[i] == b'"' || b[i] == b'\'') {
                let q = b[i];
                i += 1;
                let vs = i;
                while i < b.len() && b[i] != q {
                    i += 1;
                }
                m.insert(key.to_string(), json!(&s[vs..i]));
                i += 1;
            } else {
                let vs = i;
                while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b']' {
                    i += 1;
                }
                m.insert(key.to_string(), json!(&s[vs..i]));
            }
        } else if !key.is_empty() {
            m.insert(key.to_string(), json!(""));
        }
        while i < b.len() && b[i] != b' ' && b[i] != b'\t' && b[i] != b'\n' && b[i] != b']' {
            // no-op; loop continues naturally
            break;
        }
    }
    m
}

/// UTF-8 length of the char whose first byte is `b` (leading byte → 1-4;
/// continuation or stray byte → 1 so byte loops never split a code point).
fn utf8_len(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

/// Faithful port of WordPress `wpautop()` (formatting.php): autopad block
/// tags with \n\n, wrap every paragraph in <p>, unwrap block tags back out,
/// convert remaining single newlines to <br/>. Matches the reference
/// markup: bare <img> gets `<p><img/></p>`, text after `</h3>` gets its own
/// <p>, blockquote internals keep `<p>` paragraphs.
fn wpautop(s: &str) -> String {
    if s.trim().is_empty() {
        return String::new();
    }
    const BLOCKS: &str = "table|thead|tfoot|caption|col|colgroup|tbody|tr|td|th|div|dl|dd|dt|ul|ol|li|pre|form|map|area|blockquote|address|style|p|h[1-6]|hr|fieldset|legend|section|article|aside|hgroup|header|footer|nav|figure|figcaption|details|menu|summary";
    // All wpautop/link-fixup regexes compile once per process — each render
    // used to build ~15 of them.
    use std::sync::LazyLock;
    static RE_BR_BR: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"<br\s*/?>\s*<br\s*/?>").unwrap());
    static RE_OPEN: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(&format!(r"(<(?:{BLOCKS})[\s/>])")).unwrap());
    static RE_CLOSE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(&format!(r"(</(?:{BLOCKS})>)")).unwrap());
    static RE_HR: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"(<hr\s*?/?>)").unwrap());
    static RE_NN: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\n\n+").unwrap());
    static RE_SPLIT: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\n\s*\n").unwrap());
    static RE_EMPTY_P: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"<p>\s*</p>").unwrap());
    static RE_P_BLOCK_CLOSE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"<p>([^<]+)</(div|address|form)>").unwrap());
    static RE_P_BLOCK_FULL: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(&format!(r"<p>\s*(</?(?:{BLOCKS})[^>]*>)\s*</p>")).unwrap());
    static RE_P_LI: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"<p>(<li.+?)</p>").unwrap());
    static RE_P_BQ: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r#"<p><blockquote((?:[^>"']|"[^"]*"|'[^']*')*)>"#).unwrap());
    static RE_P_BLOCK_PRE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(&format!(r"<p>\s*(</?(?:{BLOCKS})[^>]*>)")).unwrap());
    static RE_BLOCK_P_POST: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(&format!(r"(</?(?:{BLOCKS})[^>]*>)\s*</p>")).unwrap());
    static RE_BLOCK_BR: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(&format!(r"(</?(?:{BLOCKS})[^>]*>)\s*<br />")).unwrap());
    static RE_BR_BLOCK: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"<br />(\s*</?(?:p|li|div|dl|dd|dt|th|pre|td|ul|ol)[^>]*>)").unwrap());
    static RE_N_ENDP: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\n</p>$").unwrap());

    // pad end; no <pre> handling needed (article body has none).
    let mut text = format!("{s}\n");
    // <br><br> → paragraph break.
    text = RE_BR_BR
        .replace_all(&text, "\n\n")
        .into_owned();
    // (?:{BLOCKS}) is REQUIRED - without the non-capturing group the
    // alternation splits (`(<table|thead|...` matches bare 'thead' etc.),
    // which is how 'a.jp' + 'g' inside an img src used to get \n\n inserted.
    let open = &*RE_OPEN;
    let close = &*RE_CLOSE;
    text = open.replace_all(&text, "\n\n$1").into_owned();
    text = close.replace_all(&text, "$1\n\n").into_owned();
    text = RE_HR.replace_all(&text, "$1\n\n").into_owned();
    text = text.replace("\r\n", "\n").replace('\r', "\n");
    text = wp_replace_in_html_tags(&text);
    text = RE_NN.replace_all(&text, "\n\n").into_owned();

    // Wrap every \n\n-separated chunk in <p>.
    let mut out = String::with_capacity(text.len() + 512);
    for para in RE_SPLIT.split(&text) {
        let t = para.trim_matches('\n');
        if !t.is_empty() {
            out.push_str("<p>");
            out.push_str(t);
            out.push_str("</p>\n");
        }
    }
    let mut text = out;
    text = RE_EMPTY_P.replace_all(&text, "").into_owned();
    text = RE_P_BLOCK_CLOSE
        .replace_all(&text, "<p>$1</p></$2>")
        .into_owned();
    // unwrap <p>-wrapped block tags (three cleanup passes like WP).
    text = RE_P_BLOCK_FULL
        .replace_all(&text, "$1")
        .into_owned();
    text = RE_P_LI.replace_all(&text, "$1").into_owned();
    text = RE_P_BQ
        .replace_all(&text, "<blockquote$1><p>")
        .into_owned();
    text = text.replace("</blockquote></p>", "</p></blockquote>");
    text = RE_P_BLOCK_PRE
        .replace_all(&text, "$1")
        .into_owned();
    text = RE_BLOCK_P_POST
        .replace_all(&text, "$1")
        .into_owned();

    // $br pass: protect newlines inside <script>/<style>/<svg>/<math>,
    // normalize <br>, then newline (not after <br />) → <br />\n.
    text = preserve_block_newlines(&text);
    text = text.replace("<br>", "<br />").replace("<br/>", "<br />");
    text = newline_to_br(&text);
    text = text.replace("<WPPreserveNewline />", "\n");
    text = RE_BLOCK_BR
        .replace_all(&text, "$1")
        .into_owned();
    text = RE_BR_BLOCK
        .replace_all(&text, "$1")
        .into_owned();
    text = RE_N_ENDP.replace_all(&text, "</p>").into_owned();
    // restore newlines inside attributes
    text.replace(" <!-- wpnl --> ", "\n")
        .replace("<!-- wpnl -->", "\n")
}

/// `wp_replace_in_html_tags`: newline inside tag definitions →
/// ` <!-- wpnl --> ` so paragraph splitting never splits a tag.
fn wp_replace_in_html_tags(s: &str) -> String {
    static RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"(?s)<!--.*?-->|<[^>]*>").unwrap());
    RE.replace_all(s, |c: &regex::Captures| {
        c[0].replace('\n', " <!-- wpnl --> ")
    })
    .into_owned()
}

/// PHP `(?<!<br />)\s*\n` → "<br />\n": each maximal whitespace run that
/// ends on a newline becomes one <br />, unless the run itself begins right
/// after a literal `<br />`.
fn newline_to_br(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len() + 64);
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_whitespace() {
            // scan maximal whitespace run
            let mut j = i;
            let mut last_nl = usize::MAX;
            while j < b.len() && b[j].is_ascii_whitespace() {
                if b[j] == b'\n' {
                    last_nl = j;
                }
                j += 1;
            }
            if last_nl == usize::MAX {
                out.push_str(&s[i..j]);
            } else {
                // PHP `(?<!<br />)\s*\n`: the match starts at the first
                // position inside the run whose 6 preceding bytes are NOT
                // literally "<br />" (lookbehind retries per position).
                // From there through the run's last newline collapses to a
                // single "<br />\n"; leading ws before the start position
                // and trailing ws after the last \n survive.
                let mut m = i;
                while m <= last_nl && m >= 6 && b[m].is_ascii_whitespace()
                    && &b[m - 6..m] == b"<br />"
                {
                    m += 1;
                }
                if m <= last_nl {
                    out.push_str(&s[i..m]);
                    out.push_str("<br />\n");
                    out.push_str(&s[last_nl + 1..j]);
                } else {
                    out.push_str(&s[i..j]);
                }
            }
            i = j;
        } else {
            let l = utf8_len(b[i]);
            out.push_str(&s[i..i + l]);
            i += l;
        }
    }
    out
}

/// `_autop_newline_preservation_helper`: replace \n inside
/// <script|style|svg|math>…</same> bodies with <WPPreserveNewline />.
fn preserve_block_newlines(s: &str) -> String {
    let mut out = s.to_string();
    for tag in ["script", "style", "svg", "math"] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let mut search_from = 0;
        loop {
            let Some(start) = out[search_from..].find(&open).map(|p| p + search_from)
            else {
                break;
            };
            let Some(gt) = out[start..].find('>').map(|p| p + start) else {
                break;
            };
            // Skip unclosed/nested-in-strange-ways tags the same way PHP's
            // non-greedy `.*?` would: nearest closing tag.
            if let Some(ce) = out[gt..].find(&close) {
                let body_start = gt + 1;
                let body_end = gt + ce;
                let body = out[body_start..body_end]
                    .replace('\n', "<WPPreserveNewline />");
                out.replace_range(body_start..body_end, &body);
                search_from = body_end + close.len();
            } else {
                search_from = gt + 1;
            }
        }
    }
    out
}

/// WP parity for the captured article: external links (off-site hrefs)
/// get `target="_blank" rel="noopener"`; existing target=_blank links
/// missing rel gain `rel="noopener"`.
fn link_rel_fixup(s: &str, site: &str) -> String {
    static RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r#"<a\s[^>]*href="([^"]+)"[^>]*>"#).unwrap());
    RE.replace_all(s, |c: &regex::Captures| {
        let tag = &c[0];
        let href = &c[1];
        if tag.contains("target=") {
            if tag.contains("rel=") {
                tag.to_string()
            } else {
                tag.replacen('>', " rel=\"noopener\">", 1)
            }
        } else if href.starts_with("http") && !href.starts_with(site) {
            tag.replacen('>', " target=\"_blank\" rel=\"noopener\">", 1)
        } else {
            tag.to_string()
        }
    })
    .into_owned()
}

/// wptexturize subset observed in the reference: curly quotes → entities,
/// spaced hyphens → en-dash. Operates outside tags only.
fn texturize(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 64);
    let mut rest = s;
    // Tag bytes pass through untouched; only text runs between tags are
    // texturized. Slicing whole tag spans keeps multi-byte attr values intact.
    while let Some(lt) = rest.find('<') {
        let (run, tail) = rest.split_at(lt);
        out.push_str(&tex_run(run));
        match tail.find('>') {
            Some(gt) => {
                out.push_str(&tail[..=gt]);
                rest = &tail[gt + 1..];
            }
            None => {
                // Dangling '<' — treat as text.
                out.push_str(&tex_run(tail));
                rest = "";
            }
        }
    }
    out.push_str(&tex_run(rest));
    out
}

fn tex_run(t: &str) -> String {
    let mut r = String::with_capacity(t.len() + 8);
    let chars: Vec<char> = t.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            // straight or curly apostrophe between letters → &rsquo;
            '\'' | '\u{2019}' => {
                let prev_alpha = i > 0 && chars[i - 1].is_alphanumeric();
                let next_alpha = i + 1 < chars.len() && chars[i + 1].is_alphanumeric();
                if prev_alpha || next_alpha {
                    r.push_str("&rsquo;");
                } else {
                    r.push(c);
                }
            }
            // " - " spaced hyphen → en dash entity
            '-' => {
                let prev_sp = i > 0 && (chars[i - 1] == ' ' || chars[i - 1] == '\n');
                let next_sp = i + 1 < chars.len() && (chars[i + 1] == ' ' || chars[i + 1] == '\n');
                if prev_sp && next_sp {
                    r.push_str("&#8211;");
                } else {
                    r.push(c);
                }
            }
            _ => r.push(c),
        }
        i += 1;
    }
    r
}

/* ---------------- ez-toc ---------------- */

/// Fold accented chars to ASCII base (subset needed by FR/ES headings).
fn fold(c: char, out: &mut String) {
    match c {
        'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' | 'À' | 'Á' | 'Â' | 'Ä' => out.push('a'),
        'è' | 'é' | 'ê' | 'ë' | 'È' | 'É' | 'Ê' | 'Ë' => out.push('e'),
        'ì' | 'í' | 'î' | 'ï' | 'Ì' | 'Í' | 'Î' | 'Ï' => out.push('i'),
        'ò' | 'ó' | 'ô' | 'ö' | 'õ' | 'Ò' | 'Ó' | 'Ô' | 'Ö' => out.push('o'),
        'ù' | 'ú' | 'û' | 'ü' | 'Ù' | 'Ú' | 'Û' | 'Ü' => out.push('u'),
        'ý' | 'ÿ' => out.push('y'),
        'ç' | 'Ç' => out.push('c'),
        'ñ' | 'Ñ' => out.push('n'),
        'đ' | 'Đ' => out.push('d'),
        'ơ' | 'ở' | 'ờ' | 'ớ' | 'ỡ' | 'ợ' | 'ố' | 'ồ' | 'ỗ' | 'ộ' => out.push('o'),
        'ă' | 'ắ' | 'ằ' | 'ẵ' | 'ặ' | 'ấ' | 'ầ' | 'ẫ' | 'ậ' => out.push('a'),
        'ế' | 'ề' | 'ễ' | 'ệ' => out.push('e'),
        'ư' | 'ứ' | 'ừ' | 'ữ' | 'ự' => out.push('u'),
        _ => {
            if c.is_uppercase() {
                for lc in c.to_lowercase() {
                    fold(lc, out);
                    // keep original case
                }
                out.pop();
                out.push(c);
            } else {
                out.push(c);
            }
        }
    }
}

/// ez-toc anchor: fold accents, spaces→'_', drop apostrophes/punct,
/// percent-encode the rest (multi-byte UTF-8 → %XX).
fn toc_anchor(text: &str) -> String {
    let mut folded = String::new();
    for c in text.chars() {
        fold(c, &mut folded);
    }
    let mut out = String::new();
    for c in folded.chars() {
        match c {
            ' ' => out.push('_'),
            '\'' | '’' | '?' | '!' | '‘' | ',' | '.' | ':' | ';' | '(' | ')' | '«' | '»' | '"' => {}
            c if c.is_ascii() => out.push(c),
            c => {
                let mut buf = [0u8; 4];
                for b in c.encode_utf8(&mut buf).as_bytes() {
                    out.push_str(&format!("%{b:02X}"));
                }
            }
        }
    }
    out
}

/// Strip tags + decode the handful of entities headings carry.
fn heading_text(inner: &str) -> String {
    let mut t = String::new();
    let mut in_tag = false;
    for c in inner.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => t.push(c),
            _ => {}
        }
    }
    t.replace("&rsquo;", "’")
        .replace("&#8211;", "–")
        .replace("&rsquo;", "’")
        .replace("&#039;", "'")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#8217;", "’")
        .replace("&#8216;", "‘")
        .replace("&nbsp;", " ")
}

/// Inject ez-toc anchors into h2/h3 and the TOC container before the first h2.
fn ez_toc(s: &str) -> String {
    // collect headings (level 2/3 only — the site's ez-toc setting)
    let mut heads: Vec<(usize, usize, u8, String)> = Vec::new(); // (start,end,level,anchor)
    let b = s.as_bytes();
    let mut i = 0;
    while i + 4 < b.len() {
        if b[i] == b'<' && b[i + 1] == b'h' && (b[i + 2] == b'2' || b[i + 2] == b'3') {
            let level = b[i + 2] - b'0';
            // find '>' of open tag
            if let Some(gt) = s[i..].find('>') {
                let gt = i + gt;
                let close = format!("</h{level}>");
                if let Some(ce) = s[gt..].find(&close) {
                    let ce = gt + ce + close.len();
                    let inner = heading_text(&s[gt + 1..ce - close.len()]);
                    heads.push((gt + 1, ce - close.len(), level, inner));
                    i = ce;
                    continue;
                }
            }
        }
        i += 1;
    }
    if heads.is_empty() {
        return s.to_string();
    }
    // assign anchors (dedupe)
    let mut anchors: Vec<String> = Vec::with_capacity(heads.len());
    for (_, _, _, text) in &heads {
        let mut a = toc_anchor(text);
        let mut n = 1;
        while anchors.contains(&a) {
            n += 1;
            a = format!("{}_{}", toc_anchor(text), n);
        }
        anchors.push(a);
    }
    // rebuild with spans (walk positions in reverse order to keep offsets)
    let mut out = s.to_string();
    for (idx, (hs, he, _, _)) in heads.iter().enumerate().rev() {
        let inner = &out[*hs..*he];
        let wrapped = format!(
            "<span class=\"ez-toc-section\" id=\"{}\"></span>{inner}<span class=\"ez-toc-section-end\"></span>",
            anchors[idx]
        );
        out.replace_range(*hs..*he, &wrapped);
    }
    // TOC nav (level-2 items with level-3 children nested)
    let mut nav = String::from("<nav><ul class='ez-toc-list ez-toc-list-level-1 ' >");
    let mut h2_count = 0usize;
    let mut i = 0;
    while i < heads.len() {
        let (_, _, level, _) = heads[i];
        if level == 2 {
            h2_count += 1;
            if i > 0 {
                // close previous h3 list if the previous item opened one
            }
            nav.push_str(&format!(
                "<li class='ez-toc-page-1 ez-toc-heading-level-2'><a class=\"ez-toc-link ez-toc-heading-{}\" href=\"#{}\" >{}</a>",
                h2_count, anchors[i], tex_run(&heads[i].3)
            ));
            // following level-3 children
            let mut j = i + 1;
            let mut kids: Vec<usize> = Vec::new();
            while j < heads.len() && heads[j].2 == 3 {
                kids.push(j);
                j += 1;
            }
            if !kids.is_empty() {
                nav.push_str("<ul class='ez-toc-list-level-3' >");
                for (k, idx) in kids.iter().enumerate() {
                    // ez-toc quirk in the reference capture: the first
                    // level-3 item of a nested list carries no
                    // ez-toc-page-1 class; later siblings do.
                    let page = if k == 0 { "" } else { "ez-toc-page-1 " };
                    nav.push_str(&format!(
                        "<li class='{page}ez-toc-heading-level-3'><a class=\"ez-toc-link ez-toc-heading-{}\" href=\"#{}\" >{}</a></li>",
                        idx + 1, anchors[*idx], tex_run(&heads[*idx].3)
                    ));
                }
                nav.push_str("</ul>");
            }
            nav.push_str("</li>");
            i = j;
        } else {
            // level-3 without a preceding h2: still emit at top level
            nav.push_str(&format!(
                "<li class='ez-toc-page-1 ez-toc-heading-level-3'><a class=\"ez-toc-link ez-toc-heading-{}\" href=\"#{}\" >{}</a></li>",
                i + 1, anchors[i], tex_run(&heads[i].3)
            ));
            i += 1;
        }
    }
    nav.push_str("</ul></nav>");
    let toc = format!(
        "<div id=\"ez-toc-container\" class=\"ez-toc-v2_0_89 counter-hierarchy ez-toc-counter ez-toc-grey ez-toc-container-direction\">\n<div class=\"ez-toc-title-container\">\n<p class=\"ez-toc-title\" style=\"cursor:inherit\">Sommaire</p>\n<span class=\"ez-toc-title-toggle\"><a href=\"#\" class=\"ez-toc-pull-right ez-toc-btn ez-toc-btn-xs ez-toc-btn-default ez-toc-toggle\" aria-label=\"Toggle Table of Content\"><span class=\"ez-toc-js-icon-con\"><span class=\"\"><span class=\"eztoc-hide\" style=\"display:none;\">Toggle</span><span class=\"ez-toc-icon-toggle-span\">{SVG1}{SVG2}</span></span></span></a></span></div>\n{nav}</div>\n",
        SVG1 = r#"<svg style="fill: #999;color:#999" xmlns="http://www.w3.org/2000/svg" class="list-377408" width="20px" height="20px" viewBox="0 0 24 24" fill="none"><path d="M6 6H4v2h2V6zm14 0H8v2h12V6zM4 11h2v2H4v-2zm16 0H8v2h12v-2zM4 16h2v2H4v-2zm16 0H8v2h12v-2z" fill="currentColor"></path></svg>"#,
        SVG2 = r#"<svg style="fill: #999;color:#999" class="arrow-unsorted-368013" xmlns="http://www.w3.org/2000/svg" width="10px" height="10px" viewBox="0 0 24 24" version="1.2" baseProfile="tiny"><path d="M18.2 9.3l-6.2-6.3-6.2 6.3c-.2.2-.3.4-.3.7s.1.5.3.7c.2.2.4.3.7.3h11c.3 0 .5-.1.7-.3.2-.2.3-.5.3-.7s-.1-.5-.3-.7zM5.8 14.7l6.2 6.3 6.2-6.3c.2-.2.3-.5.3-.7s-.1-.5-.3-.7c-.2-.2-.4-.3-.7-.3h-11c-.3 0-.5.1-.7.3-.2.2-.3.5-.3.7s.1.5.3.7z"/></svg>"#,
    );
    // insert before the first heading element of any level (h2–h6) —
    // previously rfind("<h2") could land inside the <h3> that precedes the
    // first h2, or inside an unrelated tag containing the literal.
    {
        let b = out.as_bytes();
        let mut open = out.len();
        let mut i = 0usize;
        while i + 3 < b.len() {
            if b[i] == b'<' && b[i + 1] == b'h' && (b'2'..=b'6').contains(&b[i + 2])
                && matches!(b[i + 3], b'>' | b' ')
            {
                open = i;
                break;
            }
            i += 1;
        }
        if open == out.len() {
            // no h2–h6 boundary found — heads[0].0 is the inner-text start
            // of the first wrapped heading; its '<' is the last '<' before it.
            open = heads
                .first()
                .and_then(|h| out[..h.0].rfind('<'))
                .unwrap_or(0);
        }
        out.insert_str(open, &toc);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_shortcodes_pass_through_verbatim() {
        // Tailwind-style literals (min-h-[20px]) must not be eaten.
        let src = "<p>x</p> min-h-[20px] [bogus attr=\"v\"] <p>y</p>";
        let out = expand_body_shortcodes(src, "https://x.test");
        assert!(out.contains("min-h-[20px]"), "{out}");
        assert!(out.contains("[bogus attr=\"v\"]"), "{out}");
        // known shortcode still expands (icons are decorative → dropped,
        // so check a button's anchor output instead)
        let out2 = expand_body_shortcodes("[gdlr_core_button button-text=\"Go\" button-link=\"https://x.test/y\"]", "https://x.test");
        assert!(out2.contains(">Go<"), "{out2}");
    }

    #[test]
    fn texturize_preserves_utf8() {
        // Regression: byte-as-char copying produced mojibake ('Cái'→'CÃ¡i').
        let src = "<p>Cái Bè, au Vietnam — très joli été là où l’âme</p><a title=\"é è\">x</a>";
        let out = texturize(src);
        assert!(out.contains("Cái Bè"));
        assert!(out.contains("très joli été là où"));
        assert!(out.contains("title=\"é è\""));
        assert!(!out.contains('Ã'));
        // apostrophe still entity-ized between letters
        assert!(out.contains("l&rsquo;âme"));
    }

    #[test]
    fn wpautop_wraps_img_and_post_heading_text() {
        // bare <img> between paragraphs → <p><img/></p> (WP behaviour)
        let src = "one.\n\n<img class=\"x\" src=\"a.jpg\" />\n\ntwo.";
        let out = wpautop(src);
        assert!(out.contains("<p><img class=\"x\" src=\"a.jpg\" /></p>"), "{out}");
        // text on the line after a block close gets its own <p>
        let src = "<h3>Titre</h3>\nsuite du texte\n\nfin.";
        let out = wpautop(src);
        assert!(out.contains("</h3>\n<p>suite du texte</p>"), "{out}");
        // blockquote keeps inner paragraph, no stray br before </p>+
        let src = "<blockquote>Vous pouvez lire plus:\n\n<a href=\"/x\">lien</a></blockquote>\n<p>Nos derniers mots,</p>\n<p>fin</p>";
        let out = wpautop(src);
        assert!(out.contains("<a href=\"/x\">lien</a></p></blockquote>"), "{out}");
        assert!(out.contains("<p>Nos derniers mots,</p>\n<p>fin</p>"), "{out}");
    }
}
