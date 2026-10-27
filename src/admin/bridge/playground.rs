//! `@wp-playground/cli` runtime — MVP `PhpRuntime` impl.
//!
//! Spawn: `npx --yes @wp-playground/cli server --port <free>` with
//! `wordpressOnBoot` pointing at a bootstrap dir containing the
//! sqlite-database-integration drop-in + our mu-plugin. wp.db lives at
//! `www/bridge/<slug>/wp.db` — disposable, re-projected each spawn.
//!
//! The spawned process serves HTTP on 127.0.0.1:<port>; `request()`
//! proxies raw HTTP through `bridge::http`.

use super::{http, BridgeResponse, BridgeState, PhpRuntime};
use anyhow::{Context, Result};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

/// mu-plugin + SQLite drop-in shipped with the binary — written into the
/// bridge dir at spawn so the runtime path needs no repo checkout.
const MU_PLUGIN: &str = include_str!("mu-plugin/bridge.php");

/// A running `@wp-playground/cli server` child.
pub(crate) struct Playground {
    child: Child,
    port: u16,
}

impl PhpRuntime for Playground {
    fn request(&mut self, method: &str, path: &str, headers: &[(String, String)], body: &[u8]) -> Result<BridgeResponse> {
        http::request(self.port, method, path, headers, body)
    }
    fn healthy(&mut self) -> bool {
        self.child.try_wait().map(|s| s.is_none()).unwrap_or(false) && http::probe(self.port)
    }
    fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// First free port on 127.0.0.1.
fn free_port() -> Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

/// Spawn the sidecar; projects the single edited post (MVP T25) into
/// `wp.db` before boot. Errors if `npx`/`node` absent.
pub(crate) fn spawn(
    _site: &str,
    slug: &str,
    www: &Path,
    rpress_db: &Path,
    post_id: Option<i64>,
) -> Result<BridgeState> {
    let dir = www.join("bridge").join(slug);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).with_context(|| format!("wipe {}", dir.display()))?;
    }
    std::fs::create_dir_all(dir.join("wp-content/mu-plugins"))?;
    std::fs::write(dir.join("wp-content/mu-plugins/rpress-bridge.php"), MU_PLUGIN)?;
    let wp_db = dir.join("wp.db");
    if let Some(id) = post_id {
        project_post(rpress_db, &wp_db, id)?;
    }

    let token = format!("{:x}", rand::random::<u128>());
    std::fs::write(dir.join("bridge-token"), &token)?;
    let port = free_port()?;

    let child = Command::new("npx")
        .args([
            "--yes", "@wp-playground/cli", "server",
            "--port", &port.to_string(),
            "--mount", &format!("{}:/wordpress", dir.display()),
        ])
        .env("RPRESS_BRIDGE_TOKEN", &token)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn @wp-playground/cli — is Node/npx installed?")?;

    // Wait for the HTTP surface (cold WASM boot ~1-3s; allow 30s).
    let deadline = Instant::now() + std::time::Duration::from_secs(30);
    while Instant::now() < deadline {
        if http::probe(port) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
    anyhow::ensure!(http::probe(port), "playground never came up on 127.0.0.1:{port}");

    Ok(BridgeState {
        rt: Box::new(Playground { child, port }),
        last_used: Instant::now(),
        token,
        wp_db,
    })
}

/// Project ONE post + its meta from rpress.db into a fresh wp.db shaped
/// like WP's own tables (sqlite-database-integration reads these). The
/// bridge gets `get_post`/`get_post_meta` for the post being edited —
/// nothing else. wp.db is disposable; rpress.db stays the only truth.
fn project_post(rpress_db: &Path, wp_db: &Path, post_id: i64) -> Result<()> {
    let src = crate::db::conn(rpress_db).context("open rpress.db for projection")?;
    let dst = rusqlite::Connection::open(wp_db).context("create wp.db")?;
    dst.execute_batch(
        "CREATE TABLE IF NOT EXISTS wp_posts(
            ID INTEGER PRIMARY KEY, post_title TEXT, post_content TEXT,
            post_excerpt TEXT, post_status TEXT, post_name TEXT,
            post_type TEXT, post_parent INTEGER, menu_order INTEGER,
            post_date TEXT, post_modified TEXT);
         CREATE TABLE IF NOT EXISTS wp_postmeta(
            meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id INTEGER, meta_key TEXT, meta_value TEXT);
         CREATE TABLE IF NOT EXISTS wp_options(
            option_id INTEGER PRIMARY KEY AUTOINCREMENT,
            option_name TEXT UNIQUE, option_value TEXT);",
    )?;
    drop(dst);
    src.execute("ATTACH DATABASE ?1 AS wp", [wp_db.to_string_lossy().as_ref()])?;
    src.execute(
        "INSERT INTO wp.wp_posts(ID, post_title, post_content, post_excerpt,
             post_status, post_name, post_type, post_parent, menu_order,
             post_date, post_modified)
         SELECT COALESCE(wp_id, id), title, body_html, excerpt, status,
             slug, kind, COALESCE(parent_id, 0), COALESCE(menu_order, 0),
             COALESCE(published_at, ''), COALESCE(modified_at, '')
         FROM posts WHERE id = ?1",
        [post_id],
    )?;
    src.execute(
        "INSERT INTO wp.wp_postmeta(post_id, meta_key, meta_value)
         SELECT (SELECT COALESCE(wp_id, id) FROM posts WHERE id = ?1),
             meta_key, meta_value
         FROM post_meta WHERE post_id = ?1",
        [post_id],
    )?;
    src.execute("DETACH DATABASE wp", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_post_copies_post_and_meta() {
        let dir = std::env::temp_dir().join(format!("rpress-proj-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let rdb = dir.join("rpress.db");
        let c = crate::db::conn(&rdb).unwrap();
        crate::schema::migrate_conn(&c).unwrap();
        c.execute(
            "INSERT INTO posts(id, wp_id, slug, title, body_html, status, kind)
             VALUES(7, 42, 'hello', 'Hi', '<p>x</p>', 'publish', 'page')",
            [],
        ).unwrap();
        c.execute(
            "INSERT INTO post_meta(post_id, meta_key, meta_value)
             VALUES(7, 'gdlr-core-layout', 'wide'), (7, 'other', 'x')",
            [],
        ).unwrap();
        drop(c);
        let wdb = dir.join("wp.db");
        project_post(&rdb, &wdb, 7).unwrap();
        let w = rusqlite::Connection::open(&wdb).unwrap();
        let title: String = w.query_row("SELECT post_title FROM wp_posts WHERE ID=42", [], |r| r.get(0)).unwrap();
        assert_eq!(title, "Hi");
        let metas: i64 = w.query_row("SELECT COUNT(*) FROM wp_postmeta WHERE post_id=42", [], |r| r.get(0)).unwrap();
        assert_eq!(metas, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
