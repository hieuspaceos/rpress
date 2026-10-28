//! Cache & regen consistency regression tests (audit remediation phase 5):
//! dependent-artifact invalidation (term archives incl. nested categories,
//! homepage, feeds, sitemaps, alias variants), full removal set on
//! trash/delete, empty_trash snapshot→delete→remove ordering, unique
//! write_cache tmp names under concurrency, and the canonical-rel 301 gate.

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

fn post(c: &Connection, id: i64, slug: &str, lang: &str, status: &str, kind: &str) {
    c.execute(
        "INSERT INTO posts(id, slug, lang, title, body_json, status, kind, published_at, modified_at) \
         VALUES(?1,?2,?3,'T','[]',?4,?5,'2025-01-01 00:00:00','2025-01-01 00:00:00')",
        rusqlite::params![id, slug, lang, status, kind],
    )
    .unwrap();
}

/// fr/es fixture: one FR post tagged into a NESTED category plus a shared
/// tag, one ES post on the same tag (so the tag archive exists per lang).
fn fixture(root: &Path) -> PathBuf {
    let db = root.join("rpress.db");
    rpress::schema::migrate(&db).unwrap();
    let c = Connection::open(&db).unwrap();
    c.execute_batch(
        "INSERT INTO settings(key,value) VALUES
         ('languages','fr,es'),('primary_lang','fr'),('page_on_front','0'),
         ('site_url','http://127.0.0.1'),('site_name','CacheTest'),('stylesheet','traveltour');",
    )
    .unwrap();
    post(&c, 1, "my-post", "fr", "publish", "post");
    post(&c, 2, "mi-post", "es", "publish", "post");
    c.execute_batch(
        "INSERT INTO post_terms(post_id, taxonomy, slug, name) VALUES
         (1,'category','culture-histoire','Culture'),
         (1,'post_tag','vietnam','Vietnam'),
         (2,'post_tag','vietnam','Vietnam');
         INSERT INTO terms(id, taxonomy, slug, name) VALUES
         (11,'category','culture-histoire','Culture'),
         (12,'post_tag','vietnam','Vietnam');",
    )
    .unwrap();
    db
}

fn put(root: &Path, rel: &str) -> PathBuf {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, "stale").unwrap();
    p
}

/// The files invalidate_dependents must delete for post 1 — everything a
/// save of that post can stale.
fn seed_dependents(out: &Path) -> Vec<PathBuf> {
    [
        "my-post/index.html",           // canonical page
        "my-post/index.md",             // markdown sibling
        "my-post.html",                 // legacy flat alias
        "fr/my-post/index.html",        // literal primary-prefix alias
        "es/my-post/index.html",        // cross-lang alias copy
        "es/my-post/index.md",
        "le-vietnam/culture-histoire/index.html", // NESTED category archive
        "etiquette/vietnam/index.html", // tag archive (fr variant)
        "es/etiquette/vietnam/index.html", // tag archive (es variant)
        "index.html",                   // homepage
        "es/index.html",                // localized front
        "feed/index.xml",
        "es/feed/index.xml",
        "sitemap.xml",
        "sitemap-post.xml",
        "sitemap-archive.xml",
    ]
    .iter()
    .map(|r| put(out, r))
    .collect()
}

#[test]
fn save_invalidates_dependents() {
    let dir = temp_dir("invdeps");
    let db = fixture(&dir);
    let out = dir.join("out");
    let conn = Connection::open(&db).unwrap();
    let doomed = seed_dependents(&out);
    let keep = put(&out, "other/index.html");

    let removed = rpress::generate::invalidate_dependents(&conn, &out, 1, "voyage", &[]);
    assert!(removed > 0);
    for p in &doomed {
        assert!(!p.exists(), "still serving stale {}", p.display());
    }
    assert!(keep.exists(), "unrelated file must survive invalidation");
    let _ = fs::remove_dir_all(&dir);
}
/// save_post slug/lang/kind change: the pre-save snapshot's variant set is
/// removed — every old-path file (index.html + index.md + {dir}.html +
/// lang copies), not just old/index.html.
#[test]
fn slug_change_removes_old_path_variants() {
    let dir = temp_dir("slugmove");
    let db = fixture(&dir);
    let out = dir.join("out");
    let conn = Connection::open(&db).unwrap();
    let files: Vec<PathBuf> = [
        "my-post/index.html",
        "my-post/index.md",
        "my-post.html",
        "fr/my-post/index.html",
        "es/my-post/index.html",
        "es/my-post/index.md",
    ]
    .iter()
    .map(|r| put(&out, r))
    .collect();

    // Same sequence the save_post/quick_edit handlers run: snapshot the
    // variant set before commit, then apply the slug change, then remove.
    let doomed = rpress::generate::file_variants_for(&conn, 1, "voyage").unwrap();
    rpress::db::quick_update(&db, 1, "T", "renamed", "publish", "fr", &[], &[]).unwrap();
    for v in &doomed {
        let p = out.join(v);
        if p.is_file() {
            fs::remove_file(&p).unwrap();
        }
    }
    for p in &files {
        assert!(!p.exists(), "old-path variant survived slug change: {}", p.display());
    }
    let _ = fs::remove_dir_all(&dir);
}

/// Term reassignment: archives the post LEFT must be invalidated too —
/// extra_terms carries the pre-commit snapshot into invalidate_dependents.
#[test]
fn term_removal_invalidates_old_archive() {
    let dir = temp_dir("termold");
    let db = fixture(&dir);
    let out = dir.join("out");
    let conn = Connection::open(&db).unwrap();
    let old_archive = put(&out, "le-vietnam/culture-histoire/index.html");

    // quick_update with empty cats replaces the term set — the old
    // category archive must still be invalidated via extra_terms.
    let extra = vec![("category".to_string(), "culture-histoire".to_string())];
    rpress::db::quick_update(&db, 1, "T", "my-post", "publish", "fr", &[], &[]).unwrap();
    // Post no longer carries the term (current post_terms wouldn't find it):
    rpress::generate::invalidate_dependents(&conn, &out, 1, "voyage", &extra);
    assert!(!old_archive.exists(), "old category archive stayed stale");
    let _ = fs::remove_dir_all(&dir);
}


#[test]
fn trash_removes_all_variants() {
    let dir = temp_dir("trashvar");
    let db = fixture(&dir);
    let out = dir.join("out");
    let variants = [
        "my-post/index.html",
        "my-post/index.md",
        "my-post.html",
        "fr/my-post/index.html",
        "fr/my-post/index.md",
        "es/my-post/index.html",
    ];
    let files: Vec<PathBuf> = variants.iter().map(|r| put(&out, r)).collect();

    // Trash the post, then regen — the non-publish branch must remove the
    // ENTIRE variant set (H5 + alias gap).
    rpress::db::set_post_status(&db, 1, "trash").unwrap();
    rpress::generate::regen_post(&db, &templates(), &out, 1, "voyage").unwrap();
    for p in &files {
        assert!(!p.exists(), "alias survived trash: {}", p.display());
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn empty_trash_snapshot_then_remove() {
    let dir = temp_dir("emptytrash");
    let db = fixture(&dir);
    let out = dir.join("out");
    rpress::db::set_post_status(&db, 1, "trash").unwrap();
    rpress::db::set_post_status(&db, 2, "trash").unwrap();
    let files: Vec<PathBuf> = [
        "my-post/index.html",
        "my-post/index.md",
        "my-post.html",
        "es/mi-post/index.html",
        "es/mi-post/index.md",
        "es/mi-post.html",
        "fr/mi-post/index.html",
    ]
    .iter()
    .map(|r| put(&out, r))
    .collect();

    // Contract ordering: snapshot id→rel list FIRST (rows must exist for
    // the path mapping), then the DB delete, then remove files.
    let conn = Connection::open(&db).unwrap();
    let ids = rpress::db::ids_matching(
        &db,
        "SELECT id FROM posts WHERE status='trash' AND kind=?1",
        ["post"],
    )
    .unwrap();
    assert_eq!(ids.len(), 2);
    let mut doomed: Vec<PathBuf> = Vec::new();
    for id in &ids {
        doomed.extend(
            rpress::generate::file_variants_for(&conn, *id, "voyage")
                .unwrap()
                .into_iter()
                .map(|r| out.join(r)),
        );
    }
    let n = rpress::db::empty_trash(&db, "post").unwrap();
    assert_eq!(n, 2);
    // After the DB delete the path mapping is gone — this is exactly why
    // the snapshot must come first.
    assert!(rpress::generate::file_variants_for(&conn, 1, "voyage").is_err());
    for p in &doomed {
        if p.is_file() {
            fs::remove_file(p).unwrap();
        }
    }
    for p in &files {
        assert!(!p.exists(), "trashed file survived: {}", p.display());
    }
    // …and no rows remain.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM posts", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn publish_regen_writes_markdown_sibling() {
    let dir = temp_dir("regenmd");
    let db = fixture(&dir);
    let out = dir.join("out");
    rpress::generate::regen_post(&db, &templates(), &out, 1, "voyage").unwrap();
    assert!(out.join("my-post/index.html").exists(), "index.html missing");
    let md = out.join("my-post/index.md");
    assert!(md.exists(), "index.md sibling missing after publish regen");
    let body = fs::read_to_string(&md).unwrap();
    assert!(body.contains("title:"), "index.md has no front matter: {body}");
    let _ = fs::remove_dir_all(&dir);
}

/// Free port via bind-drop race (fine for tests).
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

fn serve(db: &Path, out: &Path, www: &Path, port: u16) -> Server {
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
        let l = line.trim();
        if l.is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':') {
            if k.eq_ignore_ascii_case("location") {
                location = v.trim().to_string();
            }
        }
    }
    let mut body = Vec::new();
    let _ = r.read_to_end(&mut body);
    (status, location, String::from_utf8_lossy(&body).into_owned())
}

/// Every .tmp orphan anywhere under dir (atomic-write leftovers).
fn tmp_orphans(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    fn walk(d: &Path, found: &mut Vec<PathBuf>) {
        let Ok(rd) = fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, found);
            } else if p.extension().and_then(|x| x.to_str()) == Some("tmp")
                || p.file_name()
                    .and_then(|x| x.to_str())
                    .map(|n| n.contains(".tmp-"))
                    .unwrap_or(false)
            {
                found.push(p);
            }
        }
    }
    walk(dir, &mut found);
    found
}

#[test]
fn concurrent_misses_tmp_names_unique() {
    if !bin().exists() {
        eprintln!("target/debug/rpress missing — run `cargo build` first; skipping");
        return;
    }
    let dir = temp_dir("tmpnames");
    let db = fixture(&dir);
    let out = dir.join("out");
    let www = dir.join("www");
    fs::create_dir_all(&www).unwrap();
    let port = free_port();
    let srv = serve(&db, &out, &www, port);

    // Ten threads race the SAME cold path: every one renders and calls
    // write_cache — the {stem}.{pid}.{counter}.tmp names must not collide,
    // each response must be identical, and no orphan may survive.
    let mut handles = Vec::new();
    for _ in 0..10 {
        handles.push(std::thread::spawn(move || get(port, "/my-post/")));
    }
    let mut bodies = std::collections::HashSet::new();
    for h in handles {
        let (status, _loc, body) = h.join().unwrap();
        assert_eq!(status, 200);
        bodies.insert(body);
    }
    assert_eq!(bodies.len(), 1, "racing renders diverged");
    let orphans = tmp_orphans(&out);
    assert!(orphans.is_empty(), "tmp orphans: {orphans:?}");

    // Alias render 301s to canonical instead of caching a stale copy
    // (M22): /my-post.html resolves the post but must redirect to the
    // canonical permalink, not write my-post.html into the cache.
    let (status, loc, _b) = get(srv.port, "/my-post.html");
    assert_eq!(status, 301, "alias .html must 301, got {status}");
    assert_eq!(loc, "/my-post/");
    assert!(!out.join("my-post.html").exists(), "alias .html was cached");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tevily_site_renders_through_tevily_base() {
    // Phase 06: a tevily_child site must render through the tevily theme
    // dir's own base (not the traveltour fallback). The post ctx is the
    // shared-cache fixture — what changes is which base wraps it.
    let dir = temp_dir("tevilyfb");
    let db = fixture(&dir);
    let c = Connection::open(&db).unwrap();
    c.execute(
        "UPDATE settings SET value='tevily_child' WHERE key='stylesheet'",
        [],
    )
    .unwrap();
    drop(c);
    let out = dir.join("out");
    rpress::generate::regen_post(&db, &templates(), &out, 1, "voyage").unwrap();
    let html = fs::read_to_string(out.join("my-post/index.html")).unwrap();
    assert!(
        html.contains("tevily-body") || html.contains("tevily-page")
            || html.contains("gva-nav-menu"),
        "tevily site did not render through the tevily base: {}",
        &html[..html.len().min(800)]
    );
    assert!(
        !html.contains("traveltour-body") && !html.contains("wp-theme-traveltour"),
        "tevily site leaked traveltour chrome"
    );
    let _ = fs::remove_dir_all(&dir);
}
