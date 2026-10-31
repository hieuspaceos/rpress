//! Admin surface smoke (audit-261031 §5): the ~13k LOC `/admin/*` HTTP layer
//! had zero route-level tests. One end-to-end path proves the wiring that
//! unit tests can't — login → session cookie → csrf → authed list → POST a
//! trash mutation → row state actually changed in SQLite.
//!
//! Runs the real binary (`rpress serve`) like exposure/cache_coherence do —
//! the point is the full axum stack, not a stubbed Router.

use rusqlite::Connection;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/rpress")
}

fn templates() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rpress-admin-smoke-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Minimal servable site: schema + settings + one published post + one
/// admin user with a known password.
fn fixture(root: &Path) -> PathBuf {
    let db = root.join("rpress.db");
    rpress::schema::migrate(&db).unwrap();
    let c = Connection::open(&db).unwrap();
    c.execute_batch(
        "INSERT INTO settings(key,value) VALUES
         ('languages','en'),('primary_lang','en'),('page_on_front','0'),
         ('site_url','http://127.0.0.1'),('site_name','Smoke'),('stylesheet','traveltour');",
    )
    .unwrap();
    c.execute(
        "INSERT INTO posts(id, slug, lang, title, body_json, status, kind, published_at, modified_at) \
         VALUES(1,'smoke-post','en','T','[]','publish','post','2025-01-01 00:00:00','2025-01-01 00:00:00')",
        [],
    )
    .unwrap();
    drop(c);
    rpress::auth::add_user(&db, "smoke@test.local", "smoke-password-123", "admin").unwrap();
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

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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

/// Raw request; returns (status, headers-joined, body). Keep-alive off.
fn req(port: u16, method: &str, path: &str, headers: &str, body: &str) -> (u16, String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: example.test\r\nContent-Length: {}\r\n{headers}Connection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut r = BufReader::new(s);
    let mut status_line = String::new();
    r.read_line(&mut status_line).unwrap();
    let status: u16 = status_line.split_whitespace().nth(1).unwrap().parse().unwrap();
    let mut hdrs = String::new();
    loop {
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        if line.trim().is_empty() {
            break;
        }
        hdrs.push_str(&line);
    }
    let mut body_out = Vec::new();
    let _ = r.read_to_end(&mut body_out);
    (status, hdrs, String::from_utf8_lossy(&body_out).into_owned())
}

fn header<'a>(hdrs: &'a str, name: &str) -> Option<&'a str> {
    hdrs.lines().find_map(|l| {
        l.split_once(':')
            .filter(|(k, _)| k.trim().eq_ignore_ascii_case(name))
            .map(|(_, v)| v.trim())
    })
}

#[test]
fn admin_login_to_trash() {
    if !bin().exists() {
        eprintln!("target/debug/rpress missing — run `cargo build` first; skipping");
        return;
    }
    let root = temp_dir("login-trash");
    let db = fixture(&root);
    let out = root.join("cache");
    let www = root.join("www");
    fs::create_dir_all(&out).unwrap();
    fs::create_dir_all(&www).unwrap();
    let port = free_port();
    let _srv = serve(&db, &out, &www, port);

    // 1. Login — POST /api/login with form creds → 303 + session cookie.
    let (status, hdrs, _b) = req(
        port,
        "POST",
        "/api/login",
        "Content-Type: application/x-www-form-urlencoded\r\n",
        "log=smoke%40test.local&pwd=smoke-password-123",
    );
    assert_eq!(status, 303, "login must 303 to /admin");
    let cookie = header(&hdrs, "set-cookie")
        .expect("login must set rpress_session")
        .split(';')
        .next()
        .unwrap()
        .to_string();
    assert!(cookie.starts_with("rpress_session="), "unexpected cookie {cookie}");

    // 2. Authed GET /admin/posts — 200 and our post slug is listed.
    let auth = format!("Cookie: {cookie}\r\n");
    let (status, _h, body) = req(port, "GET", "/admin/posts", &auth, "");
    assert_eq!(status, 200, "authed posts list failed");
    assert!(body.contains("smoke-post"), "posts list missing fixture post");

    // 3. CSRF token rides the persisted session row (decision B) — read it
    // back rather than scraping admin HTML.
    let csrf: String = Connection::open(&db)
        .unwrap()
        .query_row("SELECT csrf FROM sessions ORDER BY rowid DESC LIMIT 1", [], |r| r.get(0))
        .expect("session row with csrf must exist after login");
    assert!(!csrf.is_empty());

    // 4. POST /admin/trash id=1 → the mutation lands in SQLite (this is the
    //    exact path audit §2.1/§2.2 flagged for blocking + error swallowing).
    let (status, _h, _b) = req(
        port,
        "POST",
        "/admin/trash",
        &format!("{auth}Content-Type: application/x-www-form-urlencoded\r\n"),
        &format!("csrf={csrf}&id=1&status=trash"),
    );
    assert!((303..=399).contains(&status) || status == 200, "trash failed: {status}");
    let st: String = Connection::open(&db)
        .unwrap()
        .query_row("SELECT status FROM posts WHERE id=1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(st, "trash", "trash mutation did not persist");

    // 5. CSRF check actually gates — same POST without csrf → 403.
    let (status, _h, _b) = req(
        port,
        "POST",
        "/admin/trash",
        &format!("{auth}Content-Type: application/x-www-form-urlencoded\r\n"),
        "id=1&status=publish",
    );
    assert_eq!(status, 403, "missing csrf must 403");

    let _ = fs::remove_dir_all(&root);
}
