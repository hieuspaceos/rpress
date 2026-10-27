//! End-to-end import fixture: a dump that exercises every expect-count
//! filter (publish+draft posts, attachment-with-file, menu membership,
//! orphan comments, translation members) plus the sqldump variants
//! (unquoted INSERT, column list, introducers, escapes, multi-line
//! statement, duplicate post ids). `verify::run` must see zero MISMATCH.

use rpress::{import, verify};
use std::path::PathBuf;

// wp_posts is positional — 23 columns in mysqldump order:
// ID,post_author,post_date,post_date_gmt,post_content,post_title,
// post_excerpt,post_status,comment_status,ping_status,post_password,
// post_name,to_ping,pinged,post_modified,post_modified_gmt,
// post_content_filtered,post_parent,guid,menu_order,post_type,
// post_mime_type,comment_count
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
 ('show_on_front','posts'),
 (_utf8mb4'blogname',_utf8mb4'Caf\xc3\xa9\\bX\\fY'),
 ('page_on_front','0');
INSERT INTO `wp_terms` VALUES (1,'English','en',0),(2,'Espa\xc3\xb1ol','es',0),
 (50,'Main Menu','main-menu',0),(51,'Men\xc3\xba','main-menu-es',0),
 (52,'Orphan','orphan-menu-zz',0),(99,'translations','translations',0);
INSERT INTO `wp_term_taxonomy` VALUES
 (1,1,'language','',0,0),(2,2,'language','',0,0),
 (50,50,'nav_menu','',0,0),(51,51,'nav_menu','',0,0),(52,52,'nav_menu','',0,0),
 (99,99,'post_translations','a:2:{s:2:\"en\";i:1;s:2:\"es\";i:2;}',0,0);
INSERT INTO `wp_term_relationships` VALUES
 (1,1,0),(2,2,0),(3,1,0),(20,50,0),(21,50,0),(23,51,0),(24,50,0);
INSERT INTO wp_posts VALUES
 (1,1,'2024-01-01 00:00:00','2024-01-01 00:00:00','<p>Hello;
world</p>','Hello','','publish','open','closed','','hello','','','2024-01-02 00:00:00','2024-01-02 00:00:00','',0,'',0,'post','',0),
 (1,1,'2024-01-01 00:00:00','2024-01-01 00:00:00','dup-content','Dup','','publish','open','closed','','dup','','','2024-01-02 00:00:00','2024-01-02 00:00:00','',0,'',0,'post','',0),
 (2,1,'2024-01-03 00:00:00','2024-01-03 00:00:00','<p>hola</p>','Hola','','publish','closed','closed','','hola','','','2024-01-03 00:00:00','2024-01-03 00:00:00','',0,'',0,'post','',0),
 (3,1,'2024-01-04 00:00:00','2024-01-04 00:00:00','<p>draft</p>','Draft','','draft','closed','closed','','draft','','','2024-01-04 00:00:00','2024-01-04 00:00:00','',0,'',0,'post','',0),
 (4,1,'2024-01-05 00:00:00','2024-01-05 00:00:00','trash','Trash','','trash','closed','closed','','trash','','','2024-01-05 00:00:00','2024-01-05 00:00:00','',0,'',0,'post','',0),
 (10,1,'2024-02-01 00:00:00','2024-02-01 00:00:00','','Pic','','inherit','closed','closed','','pic','','','2024-02-01 00:00:00','2024-02-01 00:00:00','',0,'',0,'attachment','image/jpeg',0),
 (11,1,'2024-02-01 00:00:00','2024-02-01 00:00:00','','DraftFile','','draft','closed','closed','','df','','','2024-02-01 00:00:00','2024-02-01 00:00:00','',0,'',0,'attachment','image/png',0),
 (12,1,'2024-02-01 00:00:00','2024-02-01 00:00:00','','NoFile','','inherit','closed','closed','','nf','','','2024-02-01 00:00:00','2024-02-01 00:00:00','',0,'',0,'attachment','image/png',0),
 (20,1,'2024-03-01 00:00:00','2024-03-01 00:00:00','','Home','','publish','closed','closed','','mi-home','','','2024-03-01 00:00:00','2024-03-01 00:00:00','',0,'',1,'nav_menu_item','',0),
 (21,1,'2024-03-01 00:00:00','2024-03-01 00:00:00','','About','','publish','closed','closed','','mi-about','','','2024-03-01 00:00:00','2024-03-01 00:00:00','',0,'',2,'nav_menu_item','',0),
 (22,1,'2024-03-01 00:00:00','2024-03-01 00:00:00','','Orphan','','publish','closed','closed','','mi-orphan','','','2024-03-01 00:00:00','2024-03-01 00:00:00','',0,'',3,'nav_menu_item','',0),
 (23,1,'2024-03-01 00:00:00','2024-03-01 00:00:00','','Casa','','publish','closed','closed','','mi-casa','','','2024-03-01 00:00:00','2024-03-01 00:00:00','',0,'',1,'nav_menu_item','',0),
 (24,1,'2024-03-01 00:00:00','2024-03-01 00:00:00','','Hidden','','draft','closed','closed','','mi-hidden','','','2024-03-01 00:00:00','2024-03-01 00:00:00','',0,'',4,'nav_menu_item','',0),
 (30,1,'2024-01-06 00:00:00','2024-01-06 00:00:00','old','Old','','publish','closed','closed','','rev','','','2024-01-06 00:00:00','2024-01-06 00:00:00','',1,'',0,'revision','',0);
INSERT INTO `wp_postmeta` VALUES
 (1,10,'_wp_attached_file','2024/02/pic.jpg'),
 (2,10,'_wp_attachment_metadata','a:2:{s:5:\"width\";i:640;s:6:\"height\";i:480;}'),
 (3,10,'_wp_attachment_image_alt','Pic alt'),
 (4,12,'_wp_attached_file',''),
 (5,1,'_thumbnail_id','10');
INSERT INTO `wp_comments` VALUES
 (1,1,'Alice','a@b.c','','','2024-01-10 00:00:00','2024-01-10 00:00:00','Nice',0,'1','ua','',0,0),
 (2,999,'Bob','b@b.c','','','2024-01-11 00:00:00','2024-01-11 00:00:00','Ghost',0,'1','ua','',0,0);
";

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rpress-import-expect-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn fixture_import_and_verify() {
    let d = dir("wpmini");
    let dump = d.join("dump.sql");
    std::fs::write(&dump, DUMP).unwrap();
    let db = d.join("rpress.db");
    let report = d.join("report.txt");
    import::run(&dump, &db, &report).unwrap();

    let c = rusqlite::Connection::open(&db).unwrap();
    let cnt = |q: &str| c.query_row(q, [], |r| r.get::<_, i64>(0)).unwrap();
    // publish+draft posts (trash excluded), deduped id
    assert_eq!(cnt("SELECT COUNT(*) FROM posts"), 3);
    // only the inherit attachment with a resolvable file
    assert_eq!(cnt("SELECT COUNT(*) FROM media"), 1);
    assert_eq!(cnt("SELECT COUNT(*) FROM menus"), 3);
    // published items with membership only (orphan + draft excluded)
    assert_eq!(cnt("SELECT COUNT(*) FROM menu_items"), 3);
    // comment on non-imported post dropped
    assert_eq!(cnt("SELECT COUNT(*) FROM comments"), 1);
    assert_eq!(cnt("SELECT COUNT(*) FROM post_translation_members"), 2);

    // menu lang: slug suffix vs detected langs, unknown → primary
    // (menus rows key by wp_term_id; the wp menu slug lands in `location`)
    let lang = |term_id: i64| {
        c.query_row("SELECT lang FROM menus WHERE wp_term_id=?1", [term_id], |r| r.get::<_, String>(0))
            .unwrap()
    };
    assert_eq!(lang(50), "en");   // main-menu → primary
    assert_eq!(lang(51), "es");   // main-menu-es → suffix match
    assert_eq!(lang(52), "en");   // orphan-menu-zz → unknown suffix → primary

    // duplicate wp id kept the first row
    let title: String = c.query_row("SELECT title FROM posts WHERE id=1", [], |r| r.get(0)).unwrap();
    assert_eq!(title, "Hello");
    // multi-line string with ';' inside survived statement buffering
    let body: String = c.query_row("SELECT body_html FROM posts WHERE id=1", [], |r| r.get(0)).unwrap();
    assert_eq!(body, "<p>Hello;\nworld</p>");
    // _utf8mb4 introducer + \b/\f escapes landed
    let name: String = c
        .query_row("SELECT value FROM settings WHERE key='site_name'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(name, "Café\u{8}X\u{c}Y");

    // report mentions the deduped id
    let rep = std::fs::read_to_string(&report).unwrap();
    assert!(rep.contains("duplicate wp_posts id"), "report missing dedupe line: {rep}");

    // the expect map mirrors insert filters: zero MISMATCH, no bail
    verify::run(&db).unwrap();
}
