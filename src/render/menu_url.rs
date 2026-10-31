//! Resolve `menu_items` rows to `{site_url}`-absolute URLs.
//!
//! `object_id` is a WP id: join `posts.wp_id` for `post_type` items and
//! `terms.wp_term_id` for `taxonomy` items. `custom` items carry the raw
//! URL — own-origin/alias hosts re-anchor on `site_url`
//! (`url_util::strip_origin`); foreign hosts pass through. Resolution
//! failures return `None` — the caller drops the item and its subtree
//! (WP hides dead menu items; emitting `href="#"` would leak them and
//! inflate the nav count vs origin).
use rusqlite::Connection;
use std::collections::HashMap;

/// Per-menu-build resolution context (audit P3): `lang::config` +
/// `page_on_front` + post/term lookups were per-ITEM — a 50-item menu
/// burned ~250 queries. Build once per menu resolve, batch the wp_id
/// lookups into two `IN` queries.
pub(crate) struct MenuCtx {
    cfg: crate::lang::LangConfig,
    front: i64,
    /// wp_id → (posts.id, slug, lang, kind, title)
    posts: HashMap<i64, (i64, String, String, String, String)>,
    /// terms.id → (taxonomy, slug)
    terms: HashMap<i64, (String, String)>,
}

impl MenuCtx {
    /// Prefetch everything `items` needs. `wp_ids`/`term_ids` come from a
    /// scan of the menu's item rows (item_type determines which map).
    pub(crate) fn new(conn: &Connection, wp_ids: &[i64], term_ids: &[i64]) -> MenuCtx {
        let posts = crate::db::menus::posts_by_wp_ids(conn, wp_ids);
        let terms = crate::db::menus::terms_by_ids(conn, term_ids);
        let front: i64 = conn
            .query_row(sql!("menu_url__select_page_on_front"), [], |r| r.get(0))
            .unwrap_or(0);
        MenuCtx { cfg: crate::lang::config(conn), front, posts, terms }
    }
}

/// One resolved menu item field on `site.menus` nodes.
/// Returns the absolute URL, or `None` when the item is dead.
pub(crate) fn resolve_menu_url_ctx(
    ctx: &MenuCtx,
    conn: &Connection,
    item_type: &str,
    object_id: Option<i64>,
    raw_url: &str,
    site_url: &str,
    tour_base: &str,
    lang: &str,
    aliases: &str,
) -> Option<String> {
    let site = site_url.trim_end_matches('/');
    match item_type {
        "post_type" | "page" | "post" => {
            let id = object_id?;
            let (post_id, slug, plang, kind, title) = ctx.posts.get(&id)?.clone();
            /* page_on_front stores posts.id — front page links to the
               lang root, not /{slug}/. */
            if post_id == ctx.front {
                return Some(format!("{site}{}/", crate::lang::url_prefix(&ctx.cfg, &plang)));
            }
            let wslug = crate::plugins::adapter_for_kind(&kind)
                .and_then(|a| a.display_slug(conn, post_id, &slug, &title))
                .unwrap_or(slug);
            let base = crate::plugins::kind_url_base(tour_base, &kind);
            Some(format!(
                "{site}{}",
                crate::lang::page_url(&ctx.cfg, "", &plang, &kind, &wslug, base.as_deref())
            ))
        }
        "taxonomy" => {
            let id = object_id?;
            let (tax, tslug) = ctx.terms.get(&id)?.clone();
            let t = tax.to_lowercase();
            let theme = crate::plugins::theme_for_site(conn);
            let base = if t == "category" {
                crate::plugins::theme_cat_path(conn, theme, &tslug)
            } else if t == "post_tag" {
                format!("{}/{}", crate::plugins::theme_tag_base(conn, theme), tslug)
            } else {
                format!("{t}/{tslug}")
            };
            let prefix = crate::lang::url_prefix(&ctx.cfg, lang);
            Some(format!("{site}{prefix}/{base}/"))
        }
        "custom" => {
            /* WP menu containers use url `#` and render as a bare
               `<a>` (no href) — a dead item would be dropped by WP,
               a container is not. Sentinel `none` = render without href. */
            if raw_url.is_empty() || raw_url == "#" {
                return Some(String::new());
            }
            // Stored custom URL: scheme-gated at save AND again here —
            // older DB rows and WP imports predate the save-time check
            // (audit M2); `javascript:` must never reach the anchor.
            Some(crate::url_util::safe_href_or_hash(&crate::url_util::strip_origin(raw_url, site_url, aliases)))
        }
        /* lang-switcher items render from the `flags` ctx var (chrome_vars
           emits it), not as menu links — None drops them from the tree. */
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_menu_url_ctx, MenuCtx};
    use rusqlite::Connection;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            sql!("menu_url__test_ddl"),
        )
        .unwrap();
        c
    }
    const SITE: &str = "http://localhost:8414";
    fn ctx(c: &Connection) -> MenuCtx {
        // Known fixture ids: post wp_id 101 (draft 102), term 7.
        MenuCtx::new(c, &[101, 102, 999], &[7, 999])
    }

    #[test]
    fn post_type_resolves() {
        let c = db();
        let m = ctx(&c);
        let u = resolve_menu_url_ctx(&m, &c, "post_type", Some(101), "", SITE, "voyage", "fr", "origin.test").unwrap();
        assert_eq!(u, "http://localhost:8414/hello/");
    }
    #[test]
    fn draft_drops() {
        let c = db();
        let m = ctx(&c);
        assert!(resolve_menu_url_ctx(&m, &c, "post_type", Some(102), "", SITE, "voyage", "fr", "origin.test").is_none());
        assert!(resolve_menu_url_ctx(&m, &c, "post_type", Some(999), "", SITE, "voyage", "fr", "origin.test").is_none());
    }
    #[test]
    fn taxonomy_resolves() {
        let c = db();
        let m = ctx(&c);
        let u = resolve_menu_url_ctx(&m, &c, "taxonomy", Some(7), "", SITE, "voyage", "fr", "origin.test").unwrap();
        assert!(u.ends_with("/voyage-au-vietnam/"), "{u}");
        assert!(u.starts_with(SITE), "{u}");
    }
    #[test]
    fn taxonomy_missing_drops() {
        let c = db();
        let m = ctx(&c);
        assert!(resolve_menu_url_ctx(&m, &c, "taxonomy", Some(999), "", SITE, "voyage", "fr", "origin.test").is_none());
    }
    #[test]
    fn custom_origin_reanchors_foreign_passes() {
        let c = db();
        let m = ctx(&c);
        let u = resolve_menu_url_ctx(&m, &c, "custom", None, "https://origin.test/x/", SITE, "voyage", "fr", "origin.test").unwrap();
        assert_eq!(u, "http://localhost:8414/x/");
        let u = resolve_menu_url_ctx(&m, &c, "custom", None, "https://fb.com/x", SITE, "voyage", "fr", "origin.test").unwrap();
        assert_eq!(u, "https://fb.com/x");
    }
    #[test]
    fn custom_hash_is_nohref_not_dead() {
        let c = db();
        let m = ctx(&c);
        let u = resolve_menu_url_ctx(&m, &c, "custom", None, "#", SITE, "voyage", "fr", "").unwrap();
        assert_eq!(u, "");
    }
    #[test]
    fn lang_switcher_drops() {
        let c = db();
        let m = ctx(&c);
        assert!(resolve_menu_url_ctx(&m, &c, "lang-switcher", None, "", SITE, "voyage", "fr", "").is_none());
    }
}
