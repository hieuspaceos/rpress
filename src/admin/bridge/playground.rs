//! `@wp-playground/cli` runtime — MVP `PhpRuntime` impl.
//!
//! Spawn: `wp-playground-cli server --port <free>` (resolved from the
//! npx cache — see `cli_bin` — so no `npm exec` wrapper lingers) with
//! `wordpressOnBoot` pointing at a bootstrap dir containing the
//! sqlite-database-integration drop-in + our mu-plugin. wp.db lives at
//! `www/bridge/<slug>/wp.db` — disposable, re-projected each spawn.
//!
//! The spawned process serves HTTP on 127.0.0.1:<port>; `request()`
//! proxies raw HTTP through `bridge::http`.

use super::{http, BridgeResponse, BridgeState, PhpRuntime};
use anyhow::{Context, Result};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

/// mu-plugin + SQLite drop-in shipped with the binary — written into the
/// bridge dir at spawn so the runtime path needs no repo checkout.
pub(crate) const MU_PLUGIN: &str = include_str!("mu-plugin/bridge.php");

/// A running `@wp-playground/cli server` child.
pub(crate) struct Playground {
    child: Child,
    port: u16,
}

impl PhpRuntime for Playground {
    fn request(&mut self, method: &str, path: &str, headers: &[(String, String)], body: &[u8]) -> Result<BridgeResponse> {
        http::request(self.port, method, path, headers, body)
    }
    fn healthy(&mut self, token: &str) -> bool {
        self.child.try_wait().map(|s| s.is_none()).unwrap_or(false) && http::probe_identity(self.port, token)
    }
    fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
    fn pid(&self) -> u32 {
        self.child.id()
    }
}

/// If the parent exits without `kill()` (restart, crash), the child must
/// not leak — ~900MB per orphaned playground. `Drop` is the last-resort
/// fence; `BridgeHandle` drops when the map entry is removed or the map
/// itself dies with the process.
impl Drop for Playground {
    fn drop(&mut self) {
        self.kill();
    }
}

/// First free port on 127.0.0.1.
pub(crate) fn free_port() -> Result<u16> {
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
    plugin_file: &str,
    templates: &Path,
) -> Result<BridgeState> {
    let dir = www.join("bridge").join(slug);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).with_context(|| format!("wipe {}", dir.display()))?;
    }
    // wp-content is mounted PER-SUBDIR, never wholesale: mounting
    // `wp-content` itself (or `plugins`/`themes`/`database` wholesale)
    // shadows the sqlite `db.php` drop-in and sqlite-database-integration
    // — wpdb falls back to mysqli, `siteurl` reads empty, WP loops the
    // installer forever. `plugins`/`themes` children mount individually;
    // `database` is reserved for the dedicated db_dir below.
    let wp_content = www.join("wp-content");
    std::fs::create_dir_all(wp_content.join("mu-plugins"))?;
    std::fs::write(wp_content.join("mu-plugins/rpress-bridge.php"), MU_PLUGIN)?;
    let mut mounts: Vec<(PathBuf, String)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&wp_content) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if matches!(name.as_str(), "plugins" | "themes") {
                if let Ok(kids) = std::fs::read_dir(e.path()) {
                    for k in kids.flatten() {
                        let kname = k.file_name().to_string_lossy().to_string();
                        mounts.push((
                            k.path(),
                            format!("/wordpress/wp-content/{name}/{kname}"),
                        ));
                    }
                }
            } else if e.path().is_dir() && name != "database" {
                mounts.push((e.path(), format!("/wordpress/wp-content/{name}")));
            }
        }
    }
    /* Bundled plugin fallback: the PB editor is format-universal — sites
       whose origin never had goodlayers-core (gate-allow: site names cited
       in comment only — blueskies/dmc/imagedmc run tevily, gate-allow) still need it inside the bridge after the Rust port's
       deletion. Ship it under templates/wp-plugins/ and mount when the
       site's own copy is absent; a site copy always wins. */
    if let Some(bundle) = plugin_file.split('/').next() {
        if !wp_content.join("plugins").join(bundle).exists() {
            let bundled = templates.join("wp-plugins").join(bundle);
            if bundled.is_dir() {
                mounts.push((bundled, format!("/wordpress/wp-content/plugins/{bundle}")));
            }
        }
    }
    // sqlite-database-integration keeps its DB under wp-content/database/;
    // mount a dedicated dir BEFORE the install or post-install mounts
    // would shadow the tables WP just created (install loop again).
    let db_dir = dir.join("db");
    std::fs::create_dir_all(&db_dir)?;
    let wp_db = db_dir.clone();
    if let Some(id) = post_id {
        project_post(rpress_db, &wp_content, id)?;
    }

    let token = format!("{:x}", rand::random::<u128>());
    std::fs::write(dir.join("bridge-token"), &token)?;
    let port = free_port()?;

    // Tokens land as PHP constants via `--define`: process env does NOT
    // propagate into PHP-WASM.
    let mut args: Vec<String> = vec![
        "server".into(),
        "--port".into(),
        port.to_string(),
        "--site-url".into(),
        format!("http://127.0.0.1:{port}"),
        "--wp".into(),
        "6.7".into(),
        "--define".into(),
        "RPRESS_BRIDGE_TOKEN".into(),
        token.clone(),
        "--define".into(),
        "RPRESS_BRIDGE_PLUGIN".into(),
        plugin_file.to_string(),
    ];
    /* The site's real theme — theme filters (skin slug, tourmaster element
       registration) only exist while it's active. Settings key is the raw
       WP `stylesheet` option (kept verbatim by import). */
    if let Ok(c) = crate::db::conn(rpress_db) {
        /* Pre-cutover DBs may lack stylesheet/active_plugins (imported
           before the allowlist grew them). Fall back to the site's
           `dump.sql` — same rows, already on disk — then seed the
           resolved values back into settings so the fallback runs once
           per site, not per spawn. */
        let dump = www.parent().map(|w| w.join("dump.sql"));
        let opt = |key: &str| -> Option<String> {
            let row: Option<String> = c
                .query_row(sql!("intake__select_setting"), [key], |r| r.get(0))
                .ok()
                .filter(|s: &String| !s.trim().is_empty());
            match row {
                Some(v) => Some(v),
                None => {
                    let v = option_from_dump(dump.as_deref(), key);
                    if let Some(v) = &v {
                        let _ = c.execute(
                            sql!("import__insert_or_replace"),
                            rusqlite::params![key, v],
                        );
                    }
                    v
                }
            }
        };
        if let Some(theme) = opt("stylesheet") {
            args.push("--define".into());
            args.push("RPRESS_BRIDGE_THEME".into());
            args.push(theme);
        }
        /* Element-provider parity: `active_plugins` re-activation is
           needed only for plugins that register page-builder elements
           (`gdlr_core_page_builder_element::add_element` — tourmaster's
           tour/room elements). Activating the full origin list brings
           admin-hijacking plugins (ml-slider wizard, cf7 welcome) that
           302 every bridge request; activating nothing loses the tour
           element class → options lightbox 500s. Scan mounted plugin
           dirs for the registration call. */
        if let Some(raw) = opt("active_plugins") {
            if let Ok(list) = crate::php_serialize::unserialize(raw.as_bytes()) {
                if let serde_json::Value::Array(items) = list {
                    let mut active: Vec<String> = items
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .filter(|p| p.as_str() != plugin_file)
                        .filter(|p| {
                            p.split('/').next().map(|d| {
                                dir_provides_pb_element(&wp_content.join("plugins").join(d))
                            }).unwrap_or(false)
                        })
                        .collect();
                    active.sort();
                    active.dedup();
                    if !active.is_empty() {
                        args.push("--define".into());
                        args.push("RPRESS_BRIDGE_PLUGINS".into());
                        args.push(serde_json::to_string(&active).unwrap());
                    }
                }
            }
        }
    }
    args.push("--mount-before-install".into());
    args.push(format!("{}:/wordpress/wp-content/database", db_dir.display()));
    for (src, dst) in &mounts {
        args.push("--mount".into());
        args.push(format!("{}:{dst}", src.display()));
    }
    let child = Command::new(cli_bin())
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn @wp-playground/cli — is Node installed?")?;

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
        spawned_at: Instant::now(),
        token,
        wp_db,
        pidfile: PathBuf::new(), // filled by mod::spawn
    })
}

/// Project ONE post + its meta as `wp-content/rpress-seed/post.json` —
/// the mu-plugin `wp_insert_post`s it on first admin request (WP must
/// create its own tables first; direct SQLite projection would fight
/// the installer). wp.db stays disposable; rpress.db is the only truth.
pub(crate) fn project_post(rpress_db: &Path, wp_content: &Path, post_id: i64) -> Result<()> {
    let src = crate::db::conn(rpress_db).context("open rpress.db for projection")?;
    let post: Option<serde_json::Value> = src
        .query_row(
            sql!("bridge_playground__select_post_projection"),
            [post_id],
            |r| {
                Ok(serde_json::json!({
                    "ID": r.get::<_, i64>(0)?,
                    "post_title": r.get::<_, String>(1)?,
                    "post_content": r.get::<_, Option<String>>(2)?,
                    "post_excerpt": r.get::<_, Option<String>>(3)?,
                    "post_status": r.get::<_, Option<String>>(4)?,
                    "post_name": r.get::<_, Option<String>>(5)?,
                    "post_type": r.get::<_, Option<String>>(6)?,
                    "post_parent": r.get::<_, i64>(7)?,
                    "menu_order": r.get::<_, i64>(8)?,
                    "post_date": r.get::<_, String>(9)?,
                    "post_modified": r.get::<_, String>(10)?,
                }))
            },
        )
        .ok();
    let Some(post) = post else { return Ok(()) };
    let mut st = src.prepare(sql!("bridge_playground__select_post_meta"))?;
    let meta: serde_json::Map<String, serde_json::Value> = st
        .query_map([post_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<(String, String)>>>()?
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(v)))
        .collect();
    let seed_dir = wp_content.join("rpress-seed");
    std::fs::create_dir_all(&seed_dir)?;
    std::fs::write(
        seed_dir.join("post.json"),
        serde_json::to_string(&serde_json::json!({
            "post": post, "meta": meta,
        }))?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_post_writes_seed_json() {
        let dir = std::env::temp_dir().join(format!("rpress-proj-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let rdb = dir.join("rpress.db");
        let c = crate::db::conn(&rdb).unwrap();
        crate::schema::migrate_conn(&c).unwrap();
        c.execute(
            sql!("bridge_playground__test_insert_post"),
            [],
        ).unwrap();
        c.execute(
            sql!("bridge_playground__test_insert_post_meta"),
            [],
        ).unwrap();
        drop(c);
        let wc = dir.join("wp-content");
        project_post(&rdb, &wc, 7).unwrap();
        let j: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(wc.join("rpress-seed/post.json")).unwrap(),
        ).unwrap();
        assert_eq!(j["post"]["post_title"], "Hi");
        assert_eq!(j["post"]["ID"], 42);
        assert_eq!(j["meta"]["gdlr-core-layout"], "wide");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// True when `dir` contains PHP that registers a page-builder element
/// (`gdlr_core_page_builder_element::add_element`). Bounded walk: element
/// files live ~2-3 levels deep (`tour/include/pb/pb-element-tour.php`).
pub(crate) fn dir_provides_pb_element(dir: &std::path::Path) -> bool {
    fn walk(d: &std::path::Path, depth: u8) -> bool {
        if depth > 4 {
            return false;
        }
        let Ok(entries) = std::fs::read_dir(d) else { return false };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if walk(&p, depth + 1) {
                    return true;
                }
            } else if p.extension().map(|x| x == "php").unwrap_or(false) {
                if let Ok(s) = std::fs::read_to_string(&p) {
                    if s.contains("add_element(") && s.contains("page_builder_element") {
                        return true;
                    }
                }
            }
        }
        false
    }
    walk(dir, 0)
}

/// Read `key`'s option value straight out of the site's `dump.sql` — used
/// as a fallback when `settings` lacks the row (DBs imported before the
/// option entered the import allowlist). Reuses `sqldump::rows`, so SQL
/// escapes are already handled. Missing dump → `None`, silent.
pub(crate) fn option_from_dump(dump: Option<&Path>, key: &str) -> Option<String> {
    let dump = dump?;
    if !dump.is_file() {
        return None;
    }
    let want = key.to_string();
    let prefix = crate::sqldump::detect_prefix(dump).unwrap_or_else(|_| "wp_".into());
    let opt_table = format!("{prefix}options");
    let mut out = None;
    let _ = crate::sqldump::rows(dump, &[opt_table.as_str()], |_, v| {
        if v.len() > 2 && v[1].to_string_lossy().as_deref() == Some(want.as_str()) {
            out = v[2].to_string_lossy();
        }
        Ok(())
    });
    out
}

#[cfg(test)]
mod tests2 {
    use super::*;
    #[test]
    fn option_from_dump_reads_row() {
        let dir = std::env::temp_dir().join(format!("rpress-od-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dump = dir.join("dump.sql");
        std::fs::write(&dump, sql!("bridge_playground__test_dump_wp_options")).unwrap();
        assert_eq!(option_from_dump(Some(&dump), "stylesheet").as_deref(), Some("tevily_child"));
        assert!(option_from_dump(Some(&dump), "active_plugins").unwrap().starts_with("a:1:"));
        assert_eq!(option_from_dump(Some(&dump), "nope"), None);
        assert_eq!(option_from_dump(None, "stylesheet"), None);
        assert_eq!(option_from_dump(Some(&dir.join("missing.sql")), "x"), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Resolve the `wp-playground-cli` executable once per process. The npx
/// cache already holds it after any prior spawn — invoking the shim
/// directly (shebang `#!/usr/bin/env node`) skips the ~50 MB `npm exec`
/// wrapper per bridge and npx's resolve pass. Order: npx cache glob →
/// `node_modules/.bin` under `npm root -g` → PATH.
fn cli_bin() -> PathBuf {
    use std::sync::OnceLock;
    static BIN: OnceLock<PathBuf> = OnceLock::new();
    BIN.get_or_init(|| {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let cache = home.join(".npm/_npx");
        let mut hits: Vec<PathBuf> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&cache) {
            for e in rd.flatten() {
                let p = e.path().join("node_modules/.bin/wp-playground-cli");
                if p.exists() {
                    hits.push(p);
                }
            }
        }
        /* Multiple npx caches → take the freshest install (version bumps
           rotate the cache-hash dir). */
        hits.sort_by_key(|p| {
            p.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        if let Some(p) = hits.pop() {
            return p;
        }
        /* Global npm install — `npm root -g` gives its node_modules. */
        if let Ok(o) = Command::new("npm").args(["root", "-g"]).output() {
            let root = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !root.is_empty() {
                let p = PathBuf::from(root).join(".bin/wp-playground-cli");
                if p.exists() {
                    return p;
                }
            }
        }
        /* PATH last — `Command::new` does the lookup; a bare name still
           produces a clear ENOENT context if nothing installed it. */
        PathBuf::from("wp-playground-cli")
    })
    .clone()
}
