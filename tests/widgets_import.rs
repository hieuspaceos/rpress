//! Widget import fixture: `sidebars_widgets` + `widget_*` options land as
//! widgets-table rows. Covers the WP→rpress sidebar routing (footer-N per
//! lang, single-tour → sidebar-tour, wp_inactive_widgets skipped), theme
//! option synthesis (`traveltour_general` picks which WP sidebars feed
//! sidebar-post-*/sidebar-archive-*), `pll_lang` instance routing, and
//! the frozen-HTML / live-renderer settings shapes.

use rpress::{import, verify};
use std::path::PathBuf;

const DUMP: &[u8] = b"\
CREATE TABLE `wp_options` (
  `option_id` bigint unsigned NOT NULL,
  `option_name` varchar(191) NOT NULL,
  `option_value` longtext,
  `autoload` varchar(20),
  PRIMARY KEY (`option_id`)
) ENGINE=InnoDB;
INSERT INTO `wp_options` (`option_name`,`option_value`) VALUES
 ('siteurl','http://example.test'),
 ('polylang','a:1:{s:12:\"default_lang\";s:2:\"fr\";}'),
 ('sidebars_widgets','a:6:{s:19:\"wp_inactive_widgets\";a:1:{i:0;s:7:\"pages-2\";}s:8:\"footer-1\";a:2:{i:0;s:6:\"text-1\";i:1;s:7:\"text-20\";}s:4:\"blog\";a:3:{i:0;s:8:\"search-9\";i:1;s:6:\"text-3\";i:2;s:20:\"wpcategorieswidget-9\";}s:10:\"blog-right\";a:1:{i:0;s:6:\"text-5\";}s:24:\"gdlr-core-sidebar-preset\";a:1:{i:0;s:8:\"search-4\";}s:11:\"single-tour\";a:2:{i:0;s:6:\"text-7\";i:1;s:6:\"text-8\";}}'),
 ('traveltour_general','a:5:{s:12:\"blog-sidebar\";s:5:\"right\";s:17:\"blog-sidebar-left\";s:24:\"gdlr-core-sidebar-preset\";s:18:\"blog-sidebar-right\";s:10:\"blog-right\";s:20:\"archive-blog-sidebar\";s:5:\"right\";s:26:\"archive-blog-sidebar-right\";s:4:\"blog\";}'),
 ('widget_text','a:6:{i:1;a:4:{s:5:\"title\";s:8:\"Footer A\";s:4:\"text\";s:11:\"Hello world\";s:6:\"filter\";b:0;s:8:\"pll_lang\";s:2:\"fr\";}i:20;a:4:{s:5:\"title\";s:9:\"Footer B!\";s:4:\"text\";s:7:\"Mundo!!\";s:6:\"filter\";b:1;s:8:\"pll_lang\";s:2:\"es\";}i:3;a:3:{s:5:\"title\";s:0:\"\";s:4:\"text\";s:4:\"Body\";s:6:\"filter\";b:0;}i:5;a:3:{s:5:\"title\";s:7:\"Right w\";s:4:\"text\";s:5:\"Right\";s:6:\"filter\";b:0;}i:7;a:3:{s:5:\"title\";s:8:\"Tour tip\";s:4:\"text\";s:32:\"[gdlr_core_space height=\\\"-10px\\\"]\";s:6:\"filter\";b:0;}i:8;a:4:{s:5:\"title\";s:6:\"SoloES\";s:4:\"text\";s:6:\"EsOnly\";s:6:\"filter\";b:0;s:8:\"pll_lang\";s:2:\"es\";}}'),
 ('widget_search','a:2:{i:9;a:2:{s:5:\"title\";s:0:\"\";s:8:\"pll_lang\";s:2:\"es\";}i:4;a:2:{s:5:\"title\";s:0:\"\";s:8:\"pll_lang\";s:2:\"es\";}}'),
 ('widget_wpcategorieswidget','a:1:{i:9;a:4:{s:9:\"wcw_title\";s:10:\"Categories\";s:17:\"wcw_taxonomy_type\";s:8:\"category\";s:17:\"wcw_action_on_cat\";s:7:\"include\";s:23:\"wcw_selected_categories\";a:2:{i:0;s:2:\"10\";i:1;s:2:\"11\";}}}');
INSERT INTO `wp_terms` VALUES
 (1,'French','fr',0),(2,'Spanish','es',0),
 (10,'Vietnam','vietnam',0),(11,'Cambodge','cambodge',0);
INSERT INTO `wp_term_taxonomy` VALUES
 (1,1,'language','',0,0),(2,2,'language','',0,0),
 (10,10,'category','',0,0),(11,11,'category','',0,0);
INSERT INTO `wp_term_relationships` VALUES
 (1,1,0),(2,2,0);
INSERT INTO wp_posts VALUES
 (1,1,'2024-01-01 00:00:00','2024-01-01 00:00:00','x','Hello','','publish','open','closed','','hello','','','2024-01-02 00:00:00','2024-01-02 00:00:00','',0,'',0,'post','',0);
";

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rpress-widgets-import-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn widgets_import() {
    let d = dir("wpmini");
    let dump = d.join("dump.sql");
    std::fs::write(&dump, DUMP).unwrap();
    let db = d.join("rpress.db");
    let report = d.join("report.txt");
    import::run(&dump, &db, &report).unwrap();

    let c = rusqlite::Connection::open(&db).unwrap();
    let ids = |sidebar: &str| -> Vec<String> {
        c.prepare("SELECT id FROM widgets WHERE sidebar_id=?1 ORDER BY ord")
            .unwrap()
            .query_map([sidebar], |r| r.get(0))
            .unwrap()
            .flatten()
            .collect()
    };
    // pll_lang routing: tagged instances land on their own lang's region.
    assert_eq!(ids("footer-1-fr"), vec!["text-1"]);
    assert_eq!(ids("footer-1-es"), vec!["text-20"]);
    // Theme-option synthesis: blog-sidebar=right + blog-sidebar-right=
    // blog-right → sidebar-post-* holds only blog-right's (untagged)
    // text-5, cloned per lang (`-es` on the non-primary copy).
    assert_eq!(ids("sidebar-post-fr"), vec!["text-5"]);
    assert_eq!(ids("sidebar-post-es"), vec!["text-5-es"]);
    // archive-blog-sidebar-right=blog → sidebar-archive-* gets blog's
    // instances; `blog` keeps no verbatim rows (the position claimed it).
    assert_eq!(
        ids("sidebar-archive-fr"),
        vec!["text-3", "wpcategorieswidget-9"]
    );
    assert_eq!(
        ids("sidebar-archive-es"),
        vec!["search-9", "text-3-es", "wpcategorieswidget-9-es"]
    );
    assert!(ids("blog").is_empty());
    // Unselected sidebars keep verbatim rows under their own WP ids.
    assert_eq!(ids("gdlr-core-sidebar-preset"), vec!["search-4"]);
    // single-tour keeps its own WP ordering on the verbatim region id;
    // each row carries its pll scope — fr/es pages filter on it.
    assert_eq!(ids("sidebar-tour"), vec!["text-7", "text-8"]);
    let langs = |sidebar: &str| -> Vec<String> {
        c.prepare("SELECT lang FROM widgets WHERE sidebar_id=?1 ORDER BY ord")
            .unwrap()
            .query_map([sidebar], |r| r.get(0))
            .unwrap()
            .flatten()
            .collect()
    };
    assert_eq!(langs("sidebar-tour"), vec!["", "es"]);
    // wp_inactive_widgets / widgetless regions import nothing.
    let total: i64 = c
        .query_row("SELECT COUNT(*) FROM widgets", [], |r| r.get(0))
        .unwrap();
    assert_eq!(total, 12);

    let settings = |id: &str| -> serde_json::Value {
        let raw: String = c
            .query_row("SELECT settings_json FROM widgets WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .unwrap();
        serde_json::from_str(&raw).unwrap()
    };
    // text, filter=1 → the paragraph pass wraps the body in <p>…</p>.
    let h20 = settings("text-20")["html"].as_str().unwrap().to_string();
    assert!(h20.contains("<p>Mundo!!</p>"), "filter=1 text not autop'd: {h20}");
    assert!(h20.contains("Footer B!"));
    // text, filter=0 → body verbatim, no <p> wrap.
    let h1 = settings("text-1")["html"].as_str().unwrap().to_string();
    assert!(h1.contains("Hello world"), "{h1}");
    assert!(!h1.contains("<p>"), "filter=0 text must not autop: {h1}");
    assert!(h1.contains("widget widget_text traveltour-widget"));
    assert!(h1.contains("traveltour-widget-head-text\">Footer A<"));
    // text-7: gdlr shortcode expanded into the frozen fragment.
    let h7 = settings("text-7")["html"].as_str().unwrap().to_string();
    assert!(
        h7.contains("gdlr-core-space-shortcode") && h7.contains("margin-top: -10px"),
        "{h7}"
    );
    // es search widget: polylang-prefixed form action + ES labels.
    let h9 = settings("search-9")["html"].as_str().unwrap().to_string();
    assert!(h9.contains("widget widget_search"), "{h9}");
    assert!(h9.contains("{{ site_url }}/es/"), "{h9}");
    assert!(h9.contains("Buscar"), "{h9}");
    // wpcategorieswidget → the live `categories` type with resolved
    // selected-category slugs and counts on (no wcw_hide_count).
    let cat = settings("wpcategorieswidget-9");
    assert_eq!(cat["taxonomy"], "category");
    assert_eq!(cat["title"], "Categories");
    assert_eq!(cat["show_counts"], true);
    assert_eq!(cat["order"], serde_json::json!(["vietnam", "cambodge"]));
    // selected slugs carry explicit empty children — a flat list, never
    // the theme taxonomy map's hierarchy.
    assert_eq!(cat["children"]["vietnam"], serde_json::json!([]));
    let typ: String = c
        .query_row(
            "SELECT type FROM widgets WHERE id='wpcategorieswidget-9'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(typ, "categories");

    // The render path sees imported rows only — no theme seeds grafted.
    // sidebar-post-fr carries the theme-selected blog-right instance
    // (text-5); seed ids (wpcategorieswidget-9/search-4/media_image-4)
    // must NOT appear (widgets_schema_v stamped at import).
    let html = rpress::widgets::render_sidebar(
        &c,
        "sidebar-post",
        "http://example.test",
        "fr",
        "",
        "voyage",
    )
    .unwrap();
    assert!(html.contains("id=\"text-5\""), "{html}");
    assert!(
        !html.contains("wpcategorieswidget-9") && !html.contains("id=\"search-4\""),
        "seed widgets grafted onto imported sidebar: {html}"
    );

    // Lang filter: the es-tagged sidebar-tour instance renders only on
    // es pages; the untagged one renders on both.
    let tour_es = rpress::widgets::render_sidebar(
        &c, "sidebar-tour", "http://example.test", "es", "", "voyage",
    )
    .unwrap();
    assert!(tour_es.contains("id=\"text-8\""), "{tour_es}");
    assert!(tour_es.contains("id=\"text-7\""), "{tour_es}");
    let tour_fr = rpress::widgets::render_sidebar(
        &c, "sidebar-tour", "http://example.test", "fr", "", "voyage",
    )
    .unwrap();
    assert!(tour_fr.contains("id=\"text-7\""), "{tour_fr}");
    assert!(!tour_fr.contains("id=\"text-8\""), "{tour_fr}");

    // import_expect.widgets mirrors the landed row count — verify clean.
    verify::run(&db).unwrap();
}
