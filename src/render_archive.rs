//! Taxonomy archive renderer — `/{taxonomy}/{slug}/` term pages.
//!
//! Reproduces the tourmaster archive template captured in
//! `templates/reference/archive.html`: traveltour page-title banner
//! (taxonomy label caption + term name + featured background), taxonomy
//! description, tour grid (grid-style-2, 3 columns, publish tours only),
//! and the right sidebar of per-taxonomy category widgets.
//!
//! `page()` renders templates/archive.html through base_{fr,es}.html like
//! the other page renderers, then post-processes the output:
//!   - splices the page-title banner before `.traveltour-page-wrapper`
//!     (the base opens that wrapper around `{% block content %}` while WP
//!     prints the banner between `</header>` and it),
//!   - replaces the frozen home body class with the archive class (until
//!     Phase 2 parameterizes `<body class>`),
//!   - marks nav <li>s whose URL matches the term permalink with
//!     current-menu-item / current-menu-ancestor classes (Phase 2 owns the
//!     generic mechanism — this is the term-archive backstop).

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};
use serde_json::{json, Value};
use std::path::Path;

#[allow(unused_imports)] // used by the binary target's copy of this module
use crate::sql;

use crate::render::load_env_theme;

/// Resolved term for dispatch: `render_slug` calls `term_by_path` first so
/// an unknown `tax/slug` still reports "slug not found".
pub(crate) struct Term {
    pub id: i64,
    pub wp_id: i64,
    pub taxonomy: String,
    pub slug: String,
    pub name: String,
    pub description: String,
}


/// Taxonomies routable as `/{tax}/{slug}/`: any taxonomy that actually
/// exists in the data (DB lookup replaces the frozen allowlist so renamed
/// or newly-imported taxonomies route too), minus WP/rpress internals that
/// can never own a public archive. Category and post_tag still dispatch
/// through their own bases (bare paths, `etiquette/`).
pub(crate) fn is_taxonomy(conn: &Connection, t: &str) -> bool {
    const INTERNAL: &[&str] = &[
        "nav_menu", "term_language", "post_format", "link_category",
        "ml-slider", "elementor_library_type", "wp_theme", "wp_template",
        "wp_template_part", "wp_navigation", "wp_font_family", "wp_font_face",
        "wpcode_location", "wpcode_tags", "wpcode_type",
        "rank_math_redirection_category", "flamingo_inbound_channel",
        "taxonomies_list", "viet",
    ];
    let tl = t.to_lowercase();
    if tl == "category" || tl == "post_tag" || INTERNAL.iter().any(|x| *x == tl) {
        return false;
    }
    conn.query_row(
        "SELECT 1 FROM terms WHERE taxonomy=?1 COLLATE NOCASE \
         UNION SELECT 1 FROM post_terms WHERE taxonomy=?1 COLLATE NOCASE LIMIT 1",
        [t],
        |r| r.get::<_, i64>(0),
    )
    .optional()
    .ok()
    .flatten()
    .is_some()
}

/// Look up `terms(taxonomy, slug)`; fall back to a post_terms-only term
/// (a pair that was never registered in `terms`).
pub(crate) fn term_by_path(conn: &Connection, taxonomy: &str, slug: &str) -> Option<Term> {
    let make = |id: i64, name: String, description: String| Term {
        id,
        wp_id: wp_term_id(taxonomy, slug).unwrap_or(id),
        taxonomy: taxonomy.to_string(),
        slug: slug.to_string(),
        name,
        description,
    };
    if let Ok((id, name, description)) = conn.query_row(
        sql!("archive__select_term"),
        rusqlite::params![taxonomy, slug],
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        },
    ) {
        return Some(make(id, name, description));
    }
    conn.query_row(
        sql!("archive__select_post_term"),
        rusqlite::params![taxonomy, slug],
        |r| r.get::<_, String>(0),
    )
    .ok()
    .map(|name| make(0, name, String::new()))
}

/// Original WP `term_id`. The dump loader never stored wp term ids on
/// `terms` rows (they auto-increment), so `term-N`/`cat-item-N` ids for
/// the four tour taxonomies in the reference chrome come from this frozen
/// map. Unknown terms fall back to the local `terms.id`.
pub(crate) fn wp_term_id(taxonomy: &str, slug: &str) -> Option<i64> {
    const IDS: &[(&str, &str, i64)] = &[
        ("region", "voyage-au-cambodge", 170),
        ("region", "voyage-au-vietnam", 172),
        ("region", "voyage-combine-vietnam-cambodge-laos", 173),
        ("region", "voyage-au-vietnam-du-nord", 213),
        ("region", "voyage-au-vietnam-du-sud", 214),
        ("region", "voyage-au-centre-du-vietnam", 217),
        ("profil", "rejoindre-un-groupe", 187),
        ("profil", "voyage-en-couple", 188),
        ("profil", "voyage-en-famille", 189),
        ("profil", "voyage-entre-amis", 190),
        ("duree", "circuit-cambodge-une-semaine", 723),
        ("duree", "circuit-vietnam-17-jours-22-jours", 1006),
        ("duree", "circuit-vietnam-cambodge-16-jours", 1015),
        ("duree", "circuit-vietnam-cambodge-20-jours", 1017),
        ("duree", "circuit-vietnam-14-jours", 1021),
        ("duree", "circuit-vietnam-cambodge-19-jours", 1024),
        ("duree", "circuit-vietnam-10-jours-12-jours", 166),
        ("duree", "circuit-vietnam-15-jours", 167),
        ("duree", "circuit-vietnam-3-semaines", 168),
        ("duree", "excursion-1-2-3-4-jours-au-vietnam", 209),
        ("style", "bien-etre", 178),
        ("style", "bike-tour", 179),
        ("style", "charme-chic", 180),
        ("style", "croisiere", 181),
        ("style", "gastronomie", 182),
        ("style", "incontournables", 183),
        ("style", "rencontre-ethnique", 184),
        ("style", "trek-randonnee", 185),
        ("style", "voyage-moto", 186),
        ("style", "tous-styles", 191),
        ("style", "circuit-extension-plage", 210),
        ("style", "vietnam-en-liberte", 212),
        // taxonomy='category': extracted from the wpcategorieswidget-9
        // capture (cat-item-N ↔ slug anchors) — the wp term ids WP printed
        // on the blog sidebar.
        ("category", "guide-de-voyage-vietnam", 120),
        ("category", "conseils-pratiques-vietnam", 121),
        ("category", "idees-de-circuit", 126),
        ("category", "circuit-au-vietnam", 127),
        ("category", "croisiere", 713),
        ("category", "randonnee-au-vietnam", 128),
        ("category", "top-choses-a-faire-a-voir", 129),
        ("category", "vietnam-en-famille", 687),
        ("category", "voyage-de-luxe-vietnam", 130),
        ("category", "voyage-en-couple-de-noce-vietnam", 131),
        ("category", "ou-dormir", 124),
        ("category", "quand-partir-au-vietnam", 122),
        ("category", "tout-sur-le-visa-vietnam", 123),
        ("category", "voyage-en-asie-du-sud-est", 125),
        ("category", "le-vietnam", 112),
        ("category", "actualites-du-tourisme", 113),
        ("category", "cuisine-vietnamienne", 115),
        ("category", "boisson-vietnamienne", 117),
        ("category", "plats-vietnamiens", 116),
        ("category", "adresses-recommandees", 118),
        ("category", "recettes-faciles", 119),
        ("category", "culture-histoire", 114),
        ("category", "sites-a-explorer", 174),
        ("category", "montagnes-du-nord", 715),
        ("category", "bac-ha", 695),
        ("category", "ha-giang", 640),
        ("category", "mai-chau", 689),
        ("category", "mu-cang-chai", 208),
        ("category", "sapa", 623),
        ("category", "station-balneaire-vietnam", 195),
        ("category", "ile-de-con-dao", 707),
        ("category", "nha-trang", 203),
        ("category", "phan-thiet", 709),
        ("category", "phu-quoc", 202),
        ("category", "vung-tau", 769),
        ("category", "vietnam-du-centre", 193),
        ("category", "da-nang", 644),
        ("category", "hauts-plateaux-vietnam", 215),
        ("category", "hoian", 200),
        ("category", "hue", 204),
        ("category", "quang-binh", 642),
        ("category", "vietnam-du-nord", 192),
        ("category", "baie-dhalong", 206),
        ("category", "dien-bien-phu", 711),
        ("category", "hanoi", 199),
        ("category", "ninh-binh", 196),
        ("category", "parc-national-de-cat-ba", 207),
        ("category", "vietnam-du-sud", 194),
        ("category", "can-tho", 1036),
        ("category", "chau-doc", 705),
        ("category", "delta-du-mekong", 201),
        ("category", "dong-nai", 736),
        ("category", "saigon-ho-chi-minh", 205),
        ("category", "voyages-au-cambodge", 198),
        ("category", "angkor-siem-reap", 197),
        ("category", "battambang", 721),
        ("category", "kampot", 957),
        ("category", "phnom-penh", 693),
    ];
    IDS.iter()
        .find(|(t, s, _)| *t == taxonomy && *s == slug)
        .map(|(_, _, id)| *id)
}

/// WP taxonomy label shown as `traveltour-page-caption`.
fn tax_label(taxonomy: &str, lang: &str) -> &'static str {
    match taxonomy {
        "profil" | "perfil" => "Profil",
        "region" | "region-es" => "Région",
        "duree" | "duracion" => "Durée",
        "style" | "estilo" => "Style",
        "category" => crate::lang::t(lang, "category_label"),
        "post_tag" => crate::lang::t(lang, "tag_label"),
        _ => crate::lang::t(lang, "term_label"),
    }
}


/// Term description HTML. The import drops `wp_term_taxonomy.description`;
/// this site fills term descriptions from dedicated `page` posts instead
/// (the famille term reads "Voyage en famille au Vietnam"). Resolution:
/// frozen alias map → `{slug}-au-vietnam` → `{slug}`.
fn term_description(conn: &Connection, term: &Term) -> String {
    const DESC_PAGES: &[(&str, &str)] =
        &[("rejoindre-un-groupe", "voyage-en-petit-groupe-au-vietnam")];
    let mut candidates: Vec<String> = DESC_PAGES
        .iter()
        .filter(|(t, _)| *t == term.slug)
        .map(|(_, p)| p.to_string())
        .collect();
    candidates.push(format!("{}-au-vietnam", term.slug));
    candidates.push(term.slug.clone());
    for slug in &candidates {
        if let Ok(body) = conn.query_row(
            sql!("archive__select_desc_page"),
            [slug.as_str()],
            |r| r.get::<_, String>(0),
        ) {
            if !body.trim().is_empty() {
                return wpautop(&crate::render::expand_shortcodes(&body, "", Some(conn)));
            }
        }
    }
    String::new()
}

/// Term banner (page-title background). `wp_termmeta.thumbnail_id` was not
/// imported; match `media.path` on the term slug, preferring the edited
/// `-e<timestamp>` upload variants WP uses for taxonomy headers.
fn term_banner(conn: &Connection, slug: &str) -> String {
    let slug = slug.to_lowercase();
    let edited = |file: &str| -> bool {
        file.split('-')
            .last()
            .is_some_and(|p| p.starts_with('e') && p[1..].chars().take_while(|c| c.is_ascii_digit()).count() >= 8)
    };
    // Lazy media_map: only candidate rows matching the slug, never the
    // whole table (the old code scanned a materialized media_map).
    let mut stmt = match conn.prepare("SELECT path FROM media WHERE LOWER(path) LIKE '%' || ?1 || '%'") {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let paths: Vec<String> = stmt
        .query_map([&slug], |r| r.get::<_, String>(0))
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default();
    let mut best: Option<String> = None;
    for path in paths {
        let Some(file) = path.rsplit('/').next() else { continue };
        if !file.to_lowercase().contains(&slug) {
            continue;
        }
        let better = match &best {
            None => true,
            Some(b) => {
                let be = edited(b.rsplit('/').next().unwrap_or(b));
                (edited(file) && !be) || (edited(file) == be && file < b.rsplit('/').next().unwrap_or(b))
            }
        };
        if better {
            best = Some(path);
        }
    }
    best.unwrap_or_default()
}

/// Render one taxonomy archive page. `ctx` carries the dispatch context
/// (site_url); the URL derives from taxonomy+slug.
/// Render one taxonomy archive page. `ctx.lang` disambiguates shared
/// taxonomies (category/post_tag serve both languages at different paths);
/// tour taxonomies derive the language from the taxonomy itself.
pub(crate) fn page(
    conn: &Connection,
    templates: &Path,
    ctx: &crate::admin::plugins::RenderCtx,
    term: &Term,
) -> Result<String> {
    let site_url = ctx.site_url.trim_end_matches('/').to_string();
    let is_post_tax = term.taxonomy == "category" || term.taxonomy == "post_tag";
    let lang = ctx.lang.as_str();
    // The taxonomy's owning adapter (its kind's KindDef.taxonomies claims
    // the name) supplies member kind, slug canonicalization + URL bases.
    let kind_defs = crate::admin::plugins::kind_defs();
    let tax_kind_def = kind_defs
        .iter()
        .find(|d| {
            d.taxonomies
                .iter()
                .any(|t| t.eq_ignore_ascii_case(&term.taxonomy))
        });
    let tax_adapter = tax_kind_def
        .and_then(|d| crate::admin::plugins::adapter_for_kind(d.kind));
    // Member posts carry the owning kind (`tour`, `to_book`); post-tax
    // archives keep the post arm below instead. `member_base` is the
    // kind's own permalink base (`booking` for to_book) — never the
    // caller's tour_base, which belongs to `tour`.
    let member_kind = tax_kind_def.map(|d| d.kind).unwrap_or("tour");
    let member_base = tax_kind_def
        .and_then(|d| d.url_base)
        .unwrap_or(&ctx.kind_base);
    let es = lang == "es" || (!is_post_tax && matches!(term.taxonomy.as_str(), "region-es" | "perfil" | "duracion" | "estilo"));
    let meta1 = |id: i64, key: &str| -> String {
        conn.query_row(
            sql!("archive__select_meta_value"),
            rusqlite::params![id, key],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_default()
    };

    let lang_cfg = crate::lang::config(conn);
    let prefix = crate::lang::url_prefix(&lang_cfg, lang);
    let term_url_path = |tax: &str, slug: &str, prefix: &str| -> String {
        match tax {
            "category" => format!("{prefix}/{}/", crate::admin::plugins::theme_traveltour::cat_path(conn, slug)),
            "post_tag" => format!("{prefix}/{}/{slug}/", crate::admin::plugins::theme_traveltour::tag_base(conn)),
            t => {
                let base = tax_adapter
                    .and_then(|a| a.taxonomy_base(t))
                    .map(|b| b.to_lowercase())
                    .unwrap_or_else(|| t.to_lowercase());
                format!("{prefix}/{base}/{slug}/")
            }
        }
    };
    let term_path = term_url_path(&term.taxonomy, &term.slug, &prefix);
    let canonical = format!("{site_url}{term_path}");

    // fr/es alternate via term_translations (the twin keeps its own
    // taxonomy so region→region-es pairings resolve).
    let mut alternates: Vec<(String, String)> = Vec::new();
    if let Some((tslug, ttax)) = conn
        .query_row(
            sql!("archive__select_term_twin"),
            [term.id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .ok()
    {
        // Twin language: `-{lang}` taxonomy suffix when configured, else
        // the legacy ES taxonomies, else the non-current lang (2-lang
        // sites — the pair table only knows primary↔other anyway).
        let suffixed = lang_cfg
            .langs
            .iter()
            .find(|l| *l != &lang_cfg.primary && ttax.ends_with(&format!("-{l}")));
        let twin_lang: &str = if let Some(l) = suffixed {
            l
        } else if is_post_tax {
            lang_cfg.langs.iter().find(|l| *l != lang).map(|s| s.as_str()).unwrap_or(&lang_cfg.primary)
        } else if ttax == "estilo" || ttax == "duracion" || ttax == "perfil" {
            "es"
        } else {
            &lang_cfg.primary
        };
        let twin_prefix = crate::lang::url_prefix(&lang_cfg, twin_lang);
        let twin_url = format!("{site_url}{}", term_url_path(&ttax, &tslug, &twin_prefix));
        alternates.push((lang.to_string(), canonical.clone()));
        alternates.push((twin_lang.to_string(), twin_url));
    }

    let hreflang_links = alternates
        .iter()
        .filter(|(_, u)| !u.is_empty())
        .map(|(l, u)| format!("<link rel=\"alternate\" href=\"{}\" hreflang=\"{}\" />", esc_attr(u), esc_attr(l)))
        .collect::<Vec<_>>()
        .join("\n");

    // ---- cards: tours carry tourmaster data; posts use the blog grid ----
    let mut cards: Vec<Value> = Vec::new();
    let mut post_cards: Vec<Value> = Vec::new();
    if is_post_tax {
        let mut stmt = conn.prepare(sql!("archive__select_posts"))?;
        let rows = stmt
            .query_map(rusqlite::params![term.taxonomy, term.slug, lang], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<i64>>(4)?,
                    r.get::<_, String>(5)?,
                ))
            })?
            .flatten()
            .collect::<Vec<_>>();
        for (post_id, slug, title, excerpt, fid, published) in &rows {
            let url = format!("{site_url}{prefix}/{slug}/");
            let image = fid
                .and_then(|m| crate::render::media_lookup(conn, m))
                .map(|(p, _)| format!("{site_url}/wp-content/uploads/{p}"))
                .unwrap_or_default();
            let img_attrs = fid
                .map(|m| crate::render::media_img_attrs(conn, m, &site_url))
                .unwrap_or_default();
            let alt = fid
                .and_then(|m| crate::render::media_lookup(conn, m))
                .map(|(_, a)| a.clone())
                .unwrap_or_default();
            let (day, month, _y, date_url, date_label) =
                crate::render_post::fr_date_parts(published);
            // Card taxonomy links — same alphabetical/comma shape the
            // single-post hero uses (term.slug → cat_path for 'category').
            let mut pcats: Vec<(String, String)> = Vec::new();
            {
                let mut st = conn
                    .prepare(sql!("render_post__select_post_terms"))
                    .ok();
                if let Some(st) = st.as_mut() {
                    let rws = st.query_map([*post_id], |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, String>(2)?,
                        ))
                    });
                    if let Ok(rws) = rws {
                        for r in rws.flatten() {
                            if r.0 == "category" {
                                pcats.push((r.1, r.2));
                            }
                        }
                    }
                }
            }
            pcats.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
            let cat_links: Vec<Value> = pcats
                .iter()
                .map(|(s, n)| json!({
                    "url": format!("{site_url}{prefix}/{}/", crate::admin::plugins::theme_traveltour::cat_path(conn, s)),
                    "name": esc(n),
                }))
                .collect();
            post_cards.push(json!({
                "url": url,
                "title": wp_title(&title.replace('<', "&lt;").replace('>', "&gt;")),
                "image": image, "img_attrs": img_attrs, "alt": alt,
                "excerpt": esc(excerpt),
                "day": day, "month": month,
                "date_url": format!("{site_url}{prefix}{date_url}"),
                "date_label": date_label,
                "author": post_meta1(conn, *post_id, "author_login"),
                "cats": cat_links,
            }));
        }
    } else {
        let mut stmt = conn.prepare(sql!("archive__select_tours"))?;
        let rows = stmt
            .query_map(rusqlite::params![term.taxonomy, term.slug, member_kind, lang], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<i64>>(4)?,
                ))
            })?
            .flatten()
            .collect::<Vec<_>>();
        for (id, slug, title, excerpt, fid) in &rows {
            // WP draft "— Copy" rows: duplicates created after the
            // reference was captured (documented drift exclusion).
            if slug.ends_with("-copy") {
                continue;
            }
            let raw = meta1(*id, "tourmaster-tour-option");
            let opt = crate::php_serialize::unserialize(raw.as_bytes()).unwrap_or(Value::Null);
            let os = |k: &str| -> String {
                opt.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string()
            };
            let mut promo_text = os("promo-text");
            if promo_text.is_empty() {
                // ribbon text falls back to the tour's style term names
                let style_tax = if es { "estilo" } else { "style" };
                let names: Vec<String> = {
                    let mut t = conn.prepare(sql!("archive__select_post_term_names"))?;
                    let names: Vec<String> = t
                        .query_map(rusqlite::params![id, style_tax], |r| r.get::<_, String>(0))?
                        .flatten()
                        .collect();
                    names
                };
                promo_text = names.join(", ");
            }
            let raw_price = {
                let t = os("tour-price-text");
                if !t.is_empty() {
                    t
                } else {
                    meta1(*id, "tourmaster-tour-price")
                }
            };
            // Adapter-owned kinds canonicalize the member slug via their
            // display_slug hook; unowned taxonomies keep the raw slug.
            let display = tax_adapter
                .and_then(|a| a.display_slug(conn, *id, slug, title))
                .unwrap_or_else(|| slug.clone());
            let url = format!("{site_url}{prefix}/{}/{display}/", member_base);
            let image = fid
                .and_then(|m| crate::render::media_lookup(conn, m))
                .map(|(p, _)| format!("{site_url}/wp-content/uploads/{p}"))
                .unwrap_or_default();
            let alt = fid
                .and_then(|m| crate::render::media_lookup(conn, m))
                .map(|(_, a)| a.clone())
                .unwrap_or_default();
            let promo_bg = os("promo-text-ribbon-background");
            let promo_color = os("promo-text-ribbon-text-color");
            let custom_excerpt = os("custom-excerpt");
            cards.push(json!({
                "url": url,
                "title": wp_title(&title.replace('<', "&lt;").replace('>', "&gt;")),
                "image": image, "alt": esc_attr(&alt),

                "promo_color": if promo_color.is_empty() { "#ffffff".into() } else { esc_attr(&promo_color) },
                "promo_bg": esc_attr(&promo_bg),
                "promo_bg_alpha": rgba(&promo_bg, 0.5),
                "promo_text": esc(&promo_text),
                "price": fmt_price(&raw_price),
                "duration_text": esc(&os("duration-text")),
                "departure": esc(&os("departure-location")),
                "excerpt": esc(if custom_excerpt.is_empty() { excerpt } else { &custom_excerpt }),
            }));
        }
    }

    // Post terms carry a real `description` column; tour taxonomies keep
    // the description-page heuristic.
    let description = if is_post_tax && !term.description.is_empty() {
        wpautop(&term.description)
    } else {
        term_description(conn, term)
    };
    let banner_path = term_banner(conn, &term.slug);
    let banner = if banner_path.is_empty() {
        String::new()
    } else {
        format!("{site_url}/wp-content/uploads/{banner_path}")
    };

    let body_class = format!(
        "archive tax-{} term-{} term-{} wp-theme-traveltour wp-child-theme-traveltour-child gdlr-core-body tourmaster-template-archive tourmaster-body metaslider-plugin traveltour-body traveltour-body-front traveltour-full  traveltour-with-sticky-navigation gdlr-core-link-to-lightbox",
        term.taxonomy.to_lowercase(), term.slug, term.wp_id
    );
    let seo_desc = if description.is_empty() {
        String::new()
    } else {
        strip_tags(&description)
    };

    // Full chrome context — same head/nav/flag machinery posts use
    // og:updated_time — newest modification among member posts (or the
    // term archive's own freshness via published_at when none matches).
    let og_updated: Option<String> = conn
        .query_row(
            "SELECT MAX(p.modified_at) FROM posts p JOIN post_terms pt \
             ON pt.post_id=p.id WHERE pt.taxonomy=?1 COLLATE NOCASE \
             AND pt.slug=?2 AND p.status='publish'",
            rusqlite::params![term.taxonomy, term.slug],
            |r| r.get(0),
        )
        .ok()
        .flatten();
    // (id=0: no post row; the term's own values override below).
    let mut tctx = crate::render::chrome_vars(
        conn,
        &site_url,
        0,
        "archive",
        &term.slug,
        lang,
        &term.name,
        &canonical,
        &seo_desc,
        "",
        &alternates,
        &ctx.kind_base,
    )?;
    let sidebar_post = crate::widgets::render_sidebar(conn, "sidebar-post", &site_url, lang, &term.slug, &ctx.kind_base)?;
    for (k, v) in serde_json::json!({
        "seo_title": esc_attr(&term.name),
        "seo_desc": seo_desc,
        "canonical": canonical,
        "hreflang_links": hreflang_links,
        "lang": lang,
        "site_url": &site_url,
        "title": esc(&term.name),
        "og_type": "website",
        "og_updated": og_updated.map(|t| format!("{}+00:00", t.replace(' ', "T"))).unwrap_or_default(),
        "body_class": body_class,
        "tax_label": tax_label(&term.taxonomy, lang),
        "banner": &banner,
        "description": description,
        "tours": cards,
        "posts": post_cards,
        "current": &term.slug,
        "term_count": if post_cards.is_empty() { cards.len() } else { post_cards.len() },
        "widgets": if is_post_tax { Vec::<Value>::new() } else { crate::widgets::sidebar_widgets(conn, "sidebar-archive", &site_url, lang, &ctx.kind_base)? },
        "sidebar_post": sidebar_post,
        "body": "",
    })
    .as_object()
    .into_iter()
    .flatten()
    {
        tctx.insert(k.clone(), v.clone());
    }

    let theme_dir = crate::render::theme_template_dir(conn, templates);
    let env = load_env_theme(templates, theme_dir.as_deref())?;
    tctx.insert(
        "base".into(),
        serde_json::json!(crate::lang::template_name(templates, theme_dir.as_deref(), &crate::lang::config(conn), lang)),
    );
    let tpl = if is_post_tax { "archive_post.html" } else { "archive.html" };
    let mut html = env
        .get_template(tpl)?
        .render(minijinja::Value::from_serialize(Value::Object(tctx)))
        .with_context(|| format!("render {tpl}"))?;

    // Splice the page-title banner before .traveltour-page-wrapper.
    const WRAPPER: &str = "<div class=\"traveltour-page-wrapper\" id=\"traveltour-page-wrapper\" >";
    let bg_attr = if banner.is_empty() {
        String::new()
    } else {
        format!(" style=\"background-image: url({banner}) ;\"")
    };
    let title_wrap = format!(
        "<div class=\"traveltour-page-title-wrap  traveltour-style-large traveltour-center-align\"{bg_attr} ><div class=\"traveltour-header-transparent-substitute\" ></div><div class=\"traveltour-page-title-overlay\"  ></div><div class=\"traveltour-page-title-container traveltour-container\" ><div class=\"traveltour-page-title-content traveltour-item-pdlr\"  ><h3 class=\"traveltour-page-caption\"  >{}</h3><h1 class=\"traveltour-page-title\"  >{}</h1></div></div></div>",
        esc(&tax_label(&term.taxonomy, lang)),
        esc(&term.name)
    );
    if let Some(p) = html.find(WRAPPER) {
        html = format!("{}{}\t{}", &html[..p], title_wrap, &html[p..]);
    }

    // Nav current classes for the term permalink (desktop + mobile nav).
    html = mark_current_menu(&html, &site_url, &term_path);

    Ok(html)
}


fn esc_attr(s: &str) -> String {
    esc_attr_pub(s)
}

/// pub(crate) view for the widget renderers (tour_list alt/style attrs).
pub(crate) fn esc_attr_pub(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}


/// Add WP nav current classes for `term_path` (site-relative like
/// `/profil/voyage-en-famille/`): the <li> enclosing a matching <a> gets
/// `current-menu-item` (+ `aria-current="page"` on the link, matching WP),
/// enclosing <li>s get `current-menu-ancestor`, the direct parent also
/// `current-menu-parent`.
fn mark_current_menu(html: &str, site_url: &str, term_path: &str) -> String {
    let needle = format!("href=\"{site_url}{term_path}\"");
    // every (insert_pos, text) collected; applied right-to-left so byte
    // offsets stay valid. The same href appears on flag links, the
    // canonical <link> and sidebar cat-item anchors — only <a>s inside
    // the nav menu <ul>s (sf-menu/m-menu) get marked.
    let ranges = crate::render::nav_menu_ranges(html);
    let mut hits: Vec<usize> = Vec::new();
    {
        let mut scan = 0usize;
        while let Some(a) = html[scan..].find(&needle).map(|p| p + scan) {
            if crate::render::in_nav(&ranges, a)
                && html[..a]
                    .rsplit_once('<')
                    .is_some_and(|(_, t)| t.starts_with("a ") && !t.contains('>'))
            {
                hits.push(a);
            }
            scan = a + needle.len();
        }
    }
    let mut inserts: Vec<(usize, String)> = Vec::new();
    for &a in &hits {
        // enclosing <li> chain: scan <li>/</li> boundaries up to `a`
        let mut stack: Vec<(usize, usize)> = Vec::new(); // (open_start, tag_end)
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
                "current-menu-item"
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
    // aria-current on the nav anchors (right before their open tag's >)
    for &a in &hits {
        if let Some(gt) = html[a..].find('>').map(|e| a + e) {
            if !html[a..gt].contains("aria-current") {
                inserts.push((gt, " aria-current=\"page\"".to_string()));
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

/// Minimal wpautop for taxonomy descriptions: "\n\n" chunks → <p>. Unlike
/// the post pipeline, bare <img>/<iframe> lines ARE wrapped (WP behavior
/// for inline-only chunks — the reference shows <p><img/></p>).
/// Single post_meta value — bylines + archive card fields.
fn post_meta1(conn: &Connection, post_id: i64, key: &str) -> String {
    conn.query_row(
        "SELECT COALESCE(meta_value,'') FROM post_meta WHERE post_id=?1 AND meta_key=?2 LIMIT 1",
        rusqlite::params![post_id, key],
        |r| r.get(0),
    )
    .unwrap_or_default()
}

fn wpautop(s: &str) -> String {
    let s = s.replace("\r\n", "\n").replace('\r', "\n");
    const BLOCKS: &[&str] = &[
        "p", "div", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "figure",
        "figcaption", "ul", "ol", "li", "table", "thead", "tbody", "tr", "td",
        "th", "form", "fieldset", "pre", "hr", "section", "article", "header",
        "footer", "nav", "aside", "video", "audio", "script", "style", "dl",
        "dt", "dd",
    ];
    let mut out = String::with_capacity(s.len() + 128);
    for chunk in s.split("\n\n") {
        let t = chunk.trim_matches('\n');
        if t.trim().is_empty() {
            continue;
        }
        let tag = t
            .trim_start()
            .strip_prefix('<')
            .and_then(|x| x.split(|c: char| c == ' ' || c == '>' || c == '/').next())
            .map(|x| x.trim_end_matches('/').to_lowercase())
            .unwrap_or_default();
        if BLOCKS.contains(&tag.as_str()) {
            out.push_str(t);
            out.push('\n');
        } else {
            out.push_str("<p>");
            out.push_str(&t.replace('\n', "<br />\n"));
            out.push_str("</p>\n");
        }
    }
    out
}

/// Strip tags for the meta-description fallback.
fn strip_tags(h: &str) -> String {
    let mut out = String::with_capacity(h.len());
    let mut in_tag = false;
    for c in h.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// "1825" → "1,825 USD"; passthrough when already formatted or empty.
fn fmt_price(raw: &str) -> String {
    fmt_price_pub(raw)
}

/// pub(crate) view for the widget renderers (tour_list price tails).
pub(crate) fn fmt_price_pub(raw: &str) -> String {
    let r = raw.trim();
    if !r.is_empty() && r.chars().all(|c| c.is_ascii_digit()) {
        let mut s = String::new();
        for (i, c) in r.chars().enumerate() {
            if i > 0 && (r.len() - i) % 3 == 0 {
                s.push(',');
            }
            s.push(c);
        }
        format!("{s} USD")
    } else {
        r.to_string()
    }
}

/// "#10873b" + 0.5 → "rgba(16, 135, 59, 0.5)" (ribbon corner color).
fn rgba(hex: &str, alpha: f64) -> String {
    rgba_pub(hex, alpha)
}

/// pub(crate) view for the widget renderers (tour_list ribbon corner).
pub(crate) fn rgba_pub(hex: &str, alpha: f64) -> String {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return String::new();
    }
    let (Ok(r), Ok(g), Ok(b)) = (
        u8::from_str_radix(&h[0..2], 16),
        u8::from_str_radix(&h[2..4], 16),
        u8::from_str_radix(&h[4..6], 16),
    ) else {
        return String::new();
    };
    format!("rgba({r}, {g}, {b}, {alpha})")
}

pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// WP `the_title` texturize subset for tour-card titles: ampersands then
/// typographic punctuation → the entities WP prints.
pub(crate) fn wp_title_pub(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('’', "&rsquo;")
        .replace('‘', "&lsquo;")
        .replace('–', "&#8211;")
        .replace('—', "&#8212;")
        .replace('…', "&#8230;")
}

/// Internal alias — callers predate the pub rename.
fn wp_title(s: &str) -> String {
    wp_title_pub(s)
}
