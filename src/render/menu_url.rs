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

/// One resolved menu item field on `site.menus` nodes.
/// Returns the absolute URL, or `None` when the item is dead.
pub(crate) fn resolve_menu_url(
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
            let (slug, plang, kind, title): (String, String, String, String) = conn
                .query_row(
                    sql!("menu_url__select_post_by_wp_id"),
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .ok()?;
            let cfg = crate::lang::config(conn);
            /* page_on_front stores posts.id — front page links to the
               lang root, not /{slug}/. */
            let post_id: i64 = conn
                .query_row(sql!("menu_url__select_id_by_wp_id"), [id], |r| r.get(0))
                .ok()?;
            let front: i64 = conn
                .query_row(sql!("menu_url__select_page_on_front"), [], |r| r.get(0))
                .unwrap_or(0);
            if post_id == front {
                return Some(format!("{site}{}/", crate::lang::url_prefix(&cfg, &plang)));
            }
            let wslug = crate::plugins::adapter_for_kind(&kind)
                .and_then(|a| a.display_slug(conn, post_id, &slug, &title))
                .unwrap_or(slug);
            let base = crate::plugins::kind_url_base(tour_base, &kind);
            Some(format!(
                "{site}{}",
                crate::lang::page_url(&cfg, "", &plang, &kind, &wslug, base.as_deref())
            ))
        }
        "taxonomy" => {
            let id = object_id?;
            let (tax, tslug): (String, String) = conn
                .query_row(
                    /* terms.id holds the WP term_id by import design
                       (cat-item-N classes and translation maps key it). */
                    sql!("menu_url__select_term"),
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .ok()?;
            let cfg = crate::lang::config(conn);
            let t = tax.to_lowercase();
            let base = if t == "category" {
                crate::plugins::theme_traveltour::cat_path(conn, &tslug)
            } else if t == "post_tag" {
                format!("{}/{}", crate::plugins::theme_traveltour::tag_base(conn), tslug)
            } else {
                format!("{t}/{tslug}")
            };
            let prefix = crate::lang::url_prefix(&cfg, lang);
            Some(format!("{site}{prefix}/{base}/"))
        }
        "custom" => {
            /* WP menu containers use url `#` and render as a bare
               `<a>` (no href) — a dead item would be dropped by WP,
               a container is not. Sentinel `none` = render without href. */
            if raw_url.is_empty() || raw_url == "#" {
                return Some(String::new());
            }
            Some(crate::url_util::strip_origin(raw_url, site_url, aliases))
        }
        /* lang-switcher items render from the `flags` ctx var (chrome_vars
           emits it), not as menu links — None drops them from the tree. */
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_menu_url;
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

    #[test]
    fn post_type_resolves() {
        let c = db();
        let u = resolve_menu_url(&c, "post_type", Some(101), "", SITE, "voyage", "fr", "origin.test").unwrap();
        assert_eq!(u, "http://localhost:8414/hello/");
    }
    #[test]
    fn draft_drops() {
        let c = db();
        assert!(resolve_menu_url(&c, "post_type", Some(102), "", SITE, "voyage", "fr", "origin.test").is_none());
        assert!(resolve_menu_url(&c, "post_type", Some(999), "", SITE, "voyage", "fr", "origin.test").is_none());
    }
    #[test]
    fn taxonomy_resolves() {
        let c = db();
        let u = resolve_menu_url(&c, "taxonomy", Some(7), "", SITE, "voyage", "fr", "origin.test").unwrap();
        assert!(u.ends_with("/voyage-au-vietnam/"), "{u}");
        assert!(u.starts_with(SITE), "{u}");
    }
    #[test]
    fn taxonomy_missing_drops() {
        let c = db();
        assert!(resolve_menu_url(&c, "taxonomy", Some(999), "", SITE, "voyage", "fr", "origin.test").is_none());
    }
    #[test]
    fn custom_origin_reanchors_foreign_passes() {
        let c = db();
        let u = resolve_menu_url(&c, "custom", None, "https://origin.test/x/", SITE, "voyage", "fr", "origin.test").unwrap();
        assert_eq!(u, "http://localhost:8414/x/");
        let u = resolve_menu_url(&c, "custom", None, "https://fb.com/x", SITE, "voyage", "fr", "origin.test").unwrap();
        assert_eq!(u, "https://fb.com/x");
    }
    #[test]
    fn custom_hash_is_nohref_not_dead() {
        let c = db();
        let u = resolve_menu_url(&c, "custom", None, "#", SITE, "voyage", "fr", "").unwrap();
        assert_eq!(u, "");
    }
    #[test]
    fn lang_switcher_drops() {
        let c = db();
        assert!(resolve_menu_url(&c, "lang-switcher", None, "", SITE, "voyage", "fr", "").is_none());
    }
}
