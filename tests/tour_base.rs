//! tour_base & URL-correctness regression tests (audit remediation phase 4):
//! a non-default `--tour-base` reaches canonical/hreflang/sitemap/llms,
//! sitemap+feed escape `&`/CDATA `]]>`, `/…/page/{N}` 301s to the archive
//! root, menu up/down move whole subtrees, and the deploy URL rewrite is
//! idempotent across domain changes.

use rusqlite::Connection;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rpress-{name}-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/rpress")
}

fn templates() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn post(c: &Connection, id: i64, slug: &str, lang: &str, title: &str, status: &str, kind: &str) {
    c.execute(
        "INSERT INTO posts(id, slug, lang, title, body_json, status, kind, published_at, modified_at) \
         VALUES(?1,?2,?3,?4,'[]',?5,?6,'2025-01-01 00:00:00','2025-01-01 00:00:00')",
        rusqlite::params![id, slug, lang, title, status, kind],
    )
    .unwrap();
}

/// FR/ES fixture: front page, a tour, an ampersand slug, a post with a
/// CDATA-terminating excerpt — enough to exercise every emitter.
fn fixture(root: &Path) -> PathBuf {
    let db = root.join("rpress.db");
    rpress::schema::migrate(&db).unwrap();
    let c = Connection::open(&db).unwrap();
    c.execute_batch(
        "INSERT INTO settings(key,value) VALUES
         ('languages','fr,es'),('primary_lang','fr'),('page_on_front','30'),
         ('site_url','http://127.0.0.1'),('site_name','TourBaseTest');",
    )
    .unwrap();
    post(&c, 30, "accueil", "fr", "Accueil", "publish", "page");
    post(&c, 31, "accueil-es", "es", "Portada", "publish", "page");
    post(&c, 40, "du-nord", "fr", "Du Nord Tour", "publish", "tour");
    post(&c, 41, "du-norte", "es", "Tour del Norte", "publish", "tour");
    post(&c, 50, "a&b", "fr", "Amp Post", "publish", "post");
    post(&c, 51, "cdata", "fr", "Cdata Post", "publish", "post");
    c.execute(
        "UPDATE posts SET excerpt='x]]>y' WHERE id=51",
        [],
    )
    .unwrap();
    // group the tour translations so hreflang alternates exist
    c.execute_batch(
        "INSERT INTO post_translation_members(group_id, post_id, lang) VALUES
         ('g1', 40, 'fr'), ('g1', 41, 'es');",
    )
    .unwrap();
    db
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

struct Server {
    child: Child,
    port: u16,
}

fn serve(db: &Path, out: &Path, www: &Path, port: u16, tour_base: &str) -> Server {
    let child = Command::new(bin())
        .arg("serve")
        .arg("--bind")
        .arg(format!("127.0.0.1:{port}"))
        .arg("--db")
        .arg(db)
        .arg("--templates")
        .arg(templates())
        .arg("--out")
        .arg(out)
        .arg("--www")
        .arg(www)
        .arg("--host")
        .arg("example.test")
        .arg("--tour-base")
        .arg(tour_base)
        .env("RPRESS_TRUSTED_PROXY", "")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn rpress serve");
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        assert!(Instant::now() < deadline, "server did not listen on {port}");
        std::thread::sleep(Duration::from_millis(50));
    }
    Server { child, port }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Bare GET; returns (status code, Location header, body).
fn get(port: u16, path: &str) -> (u16, String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    write!(
        s,
        "GET {path} HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut r = BufReader::new(s);
    let mut status_line = String::new();
    r.read_line(&mut status_line).unwrap();
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let mut location = String::new();
    loop {
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        if line == "\r\n" {
            break;
        }
        if let Some(v) = line.strip_prefix("Location: ").or_else(|| line.strip_prefix("location: ")) {
            location = v.trim().to_string();
        }
    }
    let mut body = String::new();
    let _ = r.read_to_string(&mut body);
    (status, location, body)
}

/// tour_base='tours': the tour URL space moves wholesale — canonical,
/// hreflang, sitemap and llms.txt all emit /tours/…, /voyage/ is dead.
#[test]
fn tour_base_honored_in_urls() {
    let dir = temp_dir("tourbase");
    let db = fixture(&dir);
    let out = dir.join("cache");
    let www = dir.join("www");

    // Rendered page: canonical + hreflang carry /tours/.
    let (html, status) = rpress::render::render_slug(
        &db, &templates(), "du-nord", "fr", "tour", "http://127.0.0.1", "tours",
    )
    .unwrap();
    assert_eq!(status, "publish");
    assert!(
        html.contains("rel=\"canonical\" href=\"http://127.0.0.1/tours/du-nord/\""),
        "canonical missing /tours/: {}",
        &html[..html.len().min(2000)]
    );
    assert!(
        html.contains("href=\"http://127.0.0.1/es/tours/du-norte/\""),
        "hreflang es missing /tours/"
    );
    assert!(!html.contains("/voyage/"), "voyage leaked into render");

    // Utility files: sitemap + llms.txt via emit_site_files.
    fs::create_dir_all(&out).unwrap();
    rpress::sitefx::emit_site_files(&db, &templates(), &out, "http://127.0.0.1", "tours").unwrap();
    let sm = fs::read_to_string(out.join("sitemap-tour.xml")).unwrap();
    assert!(sm.contains("<loc>http://127.0.0.1/tours/du-nord/</loc>"), "sitemap: {sm}");
    assert!(!sm.contains("/voyage/"));
    let llms = fs::read_to_string(out.join("llms.txt")).unwrap();
    assert!(llms.contains("/tours/du-nord/"), "llms: {llms}");
    assert!(!llms.contains("/voyage/"));
    let redir = fs::read_to_string(out.join("redirects.map")).unwrap();
    assert!(!redir.contains("/voyage/"), "redirects.map: {redir}");

    // Live serve: /tours/x → 200, /voyage/x → 404.
    let port = free_port();
    let srv = serve(&db, &out, &www, port, "tours");
    let (code, _, _) = get(srv.port, "/tours/du-nord/");
    assert_eq!(code, 200, "/tours/ must serve");
    let (code, _, _) = get(srv.port, "/voyage/du-nord/");
    assert_eq!(code, 404, "/voyage/ must 404 under tour_base=tours");
}

/// `&`/`<`/`]]>` in data must not corrupt the XML emitters.
#[test]
fn xml_emitters_escape() {
    let dir = temp_dir("xmlesc");
    let db = fixture(&dir);
    let out = dir.join("cache");
    fs::create_dir_all(&out).unwrap();
    rpress::sitefx::emit_site_files(&db, &templates(), &out, "http://127.0.0.1", "voyage").unwrap();

    // & slug → &amp; in sitemap loc.
    let sm = fs::read_to_string(out.join("sitemap-post.xml")).unwrap();
    assert!(sm.contains("a&amp;b"), "sitemap not escaped: {sm}");
    assert!(!sm.contains("<loc>http://127.0.0.1/a&b/"));
    // Rough well-formedness probe: no bare '&' outside an entity.
    for m in sm.match_indices('&') {
        let tail = &sm[m.0..];
        assert!(
            tail.starts_with("&amp;") || tail.starts_with("&lt;") || tail.starts_with("&gt;"),
            "bare & at {} in sitemap",
            m.0
        );
    }

    // feed: title/description are CDATA — ']]>' inside content must split.
    let feed = fs::read_to_string(out.join("feed/index.xml")).unwrap();
    assert!(feed.contains("x]]]]><![CDATA[>y"), "CDATA not split: {feed}");
    // the amp slug is a text node, not CDATA → escaped
    assert!(feed.contains("a&amp;b"), "feed link not escaped: {feed}");
}

/// `/…/page/{N}` (all digits) → 301 to the archive root.
#[test]
fn page_n_redirects_to_archive_root() {
    let dir = temp_dir("pagen");
    let db = fixture(&dir);
    let out = dir.join("cache");
    fs::create_dir_all(&out).unwrap();
    let www = dir.join("www");
    let port = free_port();
    let srv = serve(&db, &out, &www, port, "voyage");

    let (code, loc, _) = get(srv.port, "/page/2/");
    assert_eq!(code, 301, "status {code}");
    assert_eq!(loc, "/", "loc {loc}");

    let (code, loc, _) = get(srv.port, "/voyage/page/3");
    assert_eq!(code, 301);
    assert_eq!(loc, "/voyage/");

    // non-digit leaves are not the pagination alias → still 404.
    let (code, _, _) = get(srv.port, "/page/two/");
    assert_eq!(code, 404);
}

/// Up/down move the whole subtree — DFS contiguity and parent links hold.
#[test]
fn menu_subtree_moves() {
    let dir = temp_dir("menus");
    let db = dir.join("rpress.db");
    rpress::schema::migrate(&db).unwrap();
    let c = Connection::open(&db).unwrap();
    c.execute("INSERT INTO menus(id, name) VALUES(1, 'm')", []).unwrap();
    // flat DFS: A(1) → A1(11) → A2(12), B(2) → B1(21)
    for (id, parent, ord) in [(1, None, 1), (11, Some(1), 2), (12, Some(1), 3), (2, None, 4), (21, Some(2), 5)] {
        c.execute(
            "INSERT INTO menu_items(id, menu_id, parent_id, ord, label) VALUES(?1,1,?2,?3,?4)",
            rusqlite::params![id, parent, ord, format!("i{id}")],
        )
        .unwrap();
    }
    let flat = |c: &Connection| -> Vec<i64> {
        c.prepare("SELECT id FROM menu_items WHERE menu_id=1 ORDER BY ord")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .flatten()
            .collect()
    };

    // B up → whole B subtree before A's.
    assert!(rpress::db::menus::move_item(&db, 1, 2, "up").unwrap());
    assert_eq!(flat(&c), vec![2, 21, 1, 11, 12]);

    // B down → back past A's subtree.
    assert!(rpress::db::menus::move_item(&db, 1, 2, "down").unwrap());
    assert_eq!(flat(&c), vec![1, 11, 12, 2, 21]);

    // A1 (child) down past sibling A2's subtree.
    assert!(rpress::db::menus::move_item(&db, 1, 11, "down").unwrap());
    assert_eq!(flat(&c), vec![1, 12, 11, 2, 21]);

    // Last child cannot slide past the next ancestor-level subtree.
    assert!(!rpress::db::menus::move_item(&db, 1, 11, "down").unwrap());
    assert_eq!(flat(&c), vec![1, 12, 11, 2, 21]);

    // A2 is now first child → up is a no-op (can't pass its parent).
    assert!(!rpress::db::menus::move_item(&db, 1, 12, "up").unwrap());
    assert_eq!(flat(&c), vec![1, 12, 11, 2, 21]);

    // A1 back up → swaps with sibling subtree A2.
    assert!(rpress::db::menus::move_item(&db, 1, 11, "up").unwrap());
    assert_eq!(flat(&c), vec![1, 11, 12, 2, 21]);
}


/// Staged-db deploy rewrite: localhost + previous domains → new domain,
/// idempotent when the domain changes between deploys.
#[test]
fn deploy_rewrite_idempotent() {
    let dir = temp_dir("deploy");
    let db = dir.join("rpress.db");
    rpress::schema::migrate(&db).unwrap();
    let c = Connection::open(&db).unwrap();
    c.execute_batch(
        "INSERT INTO menus(id, name) VALUES(1, 'm');
         INSERT INTO menu_items(menu_id, ord, label, url) VALUES
           (1,1,'a','http://localhost:8400/tour/x/'),
           (1,2,'b','http://localhost:9191/other/'),
           (1,3,'c','/relative/path/'),
           (1,4,'d','http://localhost:8400');
         INSERT INTO redirects(old_slug, new_url, source) VALUES
           ('old','http://localhost:8400/new/','old-slug');",
    )
    .unwrap();

    // First deploy: localhost → A.
    rpress::site_deploy::rewrite_deploy_urls(&c, "http://localhost:8400", "https://a.example.com").unwrap();
    let urls: Vec<String> = {
        let mut s = c.prepare("SELECT url FROM menu_items ORDER BY ord").unwrap();
        s.query_map([], |r| r.get(0)).unwrap().flatten().collect()
    };
    assert_eq!(
        urls,
        vec![
            "https://a.example.com/tour/x/",
            "https://a.example.com/other/",
            "/relative/path/",
            "https://a.example.com", // bare localhost origin → domain
        ]
    );
    let new_url: String = c
        .query_row("SELECT new_url FROM redirects WHERE old_slug='old'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(new_url, "https://a.example.com/new/");
    let dd: String = c
        .query_row("SELECT value FROM settings WHERE key='deployed_domain'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(dd, "https://a.example.com");

    // Re-deploy to B: the A-keyed rows must follow, not stay pinned to A.
    rpress::site_deploy::rewrite_deploy_urls(&c, "http://localhost:8400", "https://b.example.com").unwrap();
    let urls: Vec<String> = {
        let mut s = c.prepare("SELECT url FROM menu_items ORDER BY ord").unwrap();
        s.query_map([], |r| r.get(0)).unwrap().flatten().collect()
    };
    assert_eq!(
        urls,
        vec![
            "https://b.example.com/tour/x/",
            "https://b.example.com/other/",
            "/relative/path/",
            "https://b.example.com",
        ]
    );
    let new_url: String = c
        .query_row("SELECT new_url FROM redirects WHERE old_slug='old'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(new_url, "https://b.example.com/new/");
}

/// Fresh-copy lifecycle: work db keeps local URLs; stage → deploy A marks
/// the work db, then stage → deploy B rewrites via the A key.
#[test]
fn deploy_work_db_lifecycle() {
    let dir = temp_dir("lifecycle");
    let work = dir.join("rpress.db");
    rpress::schema::migrate(&work).unwrap();
    {
        let c = Connection::open(&work).unwrap();
        c.execute_batch(
            "INSERT INTO menus(id, name) VALUES(1, 'm');
             INSERT INTO menu_items(menu_id, ord, label, url) VALUES
               (1,1,'a','http://localhost:8400/tour/x/');",
        )
        .unwrap();
    }
    let staged = dir.join("deploy.db");

    // Deploy A: stage, rewrite the copy, mark the work db.
    fs::copy(&work, &staged).unwrap();
    {
        let c = Connection::open(&staged).unwrap();
        rpress::site_deploy::rewrite_deploy_urls(&c, "http://localhost:8400", "https://a.example.com").unwrap();
    }
    rpress::site_deploy::mark_deployed(&work, "https://a.example.com").unwrap();

    // Deploy B from the same work db: fresh stage must carry the A key so
    // the rewrite can re-key — and the staged rows end at B.
    fs::copy(&work, &staged).unwrap();
    let c = Connection::open(&staged).unwrap();
    rpress::site_deploy::rewrite_deploy_urls(&c, "http://localhost:8400", "https://b.example.com").unwrap();
    let url: String = c
        .query_row("SELECT url FROM menu_items WHERE ord=1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(url, "https://b.example.com/tour/x/");

    // Work db is untouched except the bookkeeping row.
    let w = Connection::open(&work).unwrap();
    let url: String = w
        .query_row("SELECT url FROM menu_items WHERE ord=1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(url, "http://localhost:8400/tour/x/");
    let dd: String = w
        .query_row("SELECT value FROM settings WHERE key='deployed_domain'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(dd, "https://a.example.com");
}

