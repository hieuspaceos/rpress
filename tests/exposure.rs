//! Public-content-exposure regression tests (audit remediation phase 1):
//! draft/old-slug 404s, primary-prefix + missing-translation 301s, junk
//! prefixes never become origin redirects, asset-origin 307 gating, and
//! the primary-alias cache purge.

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

/// Minimal serving fixture: fr/es site, one published + one draft post,
/// an old-slug redirect aimed at the draft, and a draft page_on_front.
fn fixture(root: &Path) -> PathBuf {
    let db = root.join("rpress.db");
    rpress::schema::migrate(&db).unwrap();
    let c = Connection::open(&db).unwrap();
    c.execute_batch(
        "INSERT INTO settings(key,value) VALUES
         ('languages','fr,es'),('primary_lang','fr'),('page_on_front','30'),
         ('site_url','http://127.0.0.1'),('site_name','ExposureTest'),('stylesheet','traveltour'),
         ('asset_origin','http://wp-src.example.com');",
    )
    .unwrap();
    post(&c, 1, "hello", "fr", "publish", "page");
    post(&c, 2, "secret", "fr", "draft", "page");
    post(&c, 30, "accueil", "fr", "draft", "page");
    // Old-slug machinery pointing at the draft.
    c.execute_batch(
        "INSERT INTO post_meta(post_id, meta_key, meta_value) VALUES(2,'_wp_old_slug','old-secret');
         INSERT INTO redirects(old_slug, new_url, source) VALUES('old-secret','/old-secret/','old-slug');",
    )
    .unwrap();
    db
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

fn serve(db: &Path, out: &Path, www: &Path, port: u16, envs: &[(&str, &str)]) -> Server {
    let mut cmd = Command::new(bin());
    cmd.arg("serve")
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
        .stderr(Stdio::null());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let child = cmd.spawn().expect("spawn rpress serve");
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

/// Bare GET; returns (status code, Location header).
fn get(port: u16, path: &str) -> (u16, String) {
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
    // Drain body so the server isn't left writing to a dead socket.
    let mut sink = Vec::new();
    let _ = r.read_to_end(&mut sink);
    (status, location)
}

#[test]
fn public_exposure() {
    if !bin().exists() {
        eprintln!("target/debug/rpress missing — run `cargo build` first; skipping");
        return;
    }
    let root = temp_dir("exposure");
    let db = fixture(&root);
    let out = root.join("cache");
    let www = root.join("www");
    fs::create_dir_all(&out).unwrap();
    fs::create_dir_all(&www).unwrap();
    // Pre-existing primary-prefix alias cache — the boot sweep must remove
    // it or the 301 is dead (cache hit precedes split_path).
    fs::create_dir_all(out.join("fr/hello")).unwrap();
    fs::write(out.join("fr/hello/index.html"), "stale-alias").unwrap();
    // Legacy flat alias file a regen never removes.
    fs::write(out.join("stale.html"), "old-write-back").unwrap();

    let port = free_port();
    // Allowlist the fixture origin so the positive-307 cases exercise the
    // full gate; an unlisted origin must 404.
    let srv = serve(&db, &out, &www, port, &[("RPRESS_ASSET_ORIGINS", "wp-src.example.com")]);

    // Alias cache purge ran at boot.
    assert!(!out.join("fr/hello/index.html").exists(), "stale /fr/ alias survived boot purge");
    assert!(!out.join("stale.html").exists(), "legacy flat .html alias survived boot purge");

    // C1: draft slug → 404, and no cache write-back.
    let (st, _) = get(srv.port, "/secret/");
    assert_eq!(st, 404, "draft slug served");
    assert!(!out.join("secret/index.html").exists(), "draft was cached");
    // Published slug renders (sanity that 404s aren't just broken render).
    let (st, _) = get(srv.port, "/hello/");
    assert_eq!(st, 200, "published page should render");

    // Red-team: draft reachable by old-slug → 404, not a rendered draft.
    let (st, _) = get(srv.port, "/old-secret/");
    assert_eq!(st, 404, "draft served via old-slug");

    // Draft page_on_front → / is a 404, never a rendered draft.
    let (st, _) = get(srv.port, "/");
    assert_eq!(st, 404, "draft front page served");

    // H3: primary-lang prefix → 301 to canonical, alias cache gone.
    let (st, loc) = get(srv.port, "/fr/hello/");
    assert_eq!(st, 301, "/fr/ alias should redirect, got {st}");
    assert_eq!(loc, "/hello");
    let (st, _) = get(srv.port, "/fr/hello/");
    assert_eq!(st, 301, "/fr/ alias file was re-served from cache");

    // M8: missing ES translation of an FR-only slug → 301 to canonical,
    // never a 200 of the FR page under /es/.
    let (st, loc) = get(srv.port, "/es/hello/");
    assert_eq!(st, 301, "missing translation should redirect, got {st}");
    assert_eq!(loc, "/hello/");
    // A slug with no translation group at all → 404.
    let (st, _) = get(srv.port, "/es/no-such-post/");
    assert_eq!(st, 404);

    // M9: junk prefixes 404 — they must never reach the asset redirect.
    for p in ["/.env", "/wp-admin/x.php", "/wp-config.php", "/xmlrpc.php", "/secret.php"] {
        let (st, loc) = get(srv.port, p);
        assert_eq!(st, 404, "{p} should 404, got {st} → {loc}");
    }

    // M9: allowlisted WP asset trees DO redirect to an allowlisted origin…
    let (st, loc) = get(srv.port, "/wp-content/uploads/2024/01/x.jpg");
    assert_eq!(st, 307, "uploads-miss should 307 to origin, got {st}");
    assert_eq!(loc, "http://wp-src.example.com/wp-content/uploads/2024/01/x.jpg");
    // …but only when settings.asset_origin is itself allowlisted: poison
    // the setting to a non-allowlisted host → the same miss now 404s.
    {
        let c = Connection::open(&db).unwrap();
        c.execute("UPDATE settings SET value='http://evil.example.net' WHERE key='asset_origin'", []).ok();
        c.execute("INSERT INTO settings(key,value) VALUES('asset_origin','http://evil.example.net')", []).ok();
    }
    let (st, _) = get(srv.port, "/wp-content/uploads/2024/01/x.jpg");
    assert_eq!(st, 404, "non-allowlisted asset_origin must not redirect");
    let (st, _) = get(srv.port, "/wp-content/themes/t/style.css");
    assert_eq!(st, 404);

    let _ = fs::remove_dir_all(&root);
}
