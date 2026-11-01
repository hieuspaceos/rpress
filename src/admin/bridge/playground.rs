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
    fn port(&self) -> u16 {
        self.port
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
    // rpress-bridge.php belongs only inside the bridge docroot — never the
    // persistent site tree `www/wp-content/mu-plugins/` (a stray copy there
    // would carry the token fence + uid-1 auto-login into the real site).
    let mut mounts: Vec<(PathBuf, String)> = Vec::new();
    // rpress-bridge.php mounts per-file from the disposable bridge dir —
    // site mu-plugins stay visible alongside it.
    let mu_dir = dir.join("mu-plugins");
    std::fs::create_dir_all(&mu_dir)?;
    let mu_file = mu_dir.join("rpress-bridge.php");
    std::fs::write(&mu_file, MU_PLUGIN)?;
    mounts.push((mu_file, "/wordpress/wp-content/mu-plugins/rpress-bridge.php".into()));
    if let Ok(entries) = std::fs::read_dir(&wp_content) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if matches!(name.as_str(), "plugins" | "themes" | "mu-plugins") {
                if let Ok(kids) = std::fs::read_dir(e.path()) {
                    for k in kids.flatten() {
                        let kname = k.file_name().to_string_lossy().to_string();
                        // Skip a stray rpress-bridge.php the old code left in
                        // the site tree — ours is already mounted above.
                        if name == "mu-plugins" && kname == "rpress-bridge.php" {
                            continue;
                        }
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
    let mut seeded_post = None;
    if let Some(id) = post_id {
        // Seed lands in the disposable bridge dir, then mounts into the
        // VFS explicitly — writing under www/wp-content would both be
        // unmounted here (enumeration already ran) AND world-readable via
        // the public static server. Only claim `seeded_post` when the
        // seed file actually landed.
        let seed_src = dir.join("seed");
        if project_post(rpress_db, &seed_src, id)? {
            seeded_post = Some(id);
            mounts.push((
                seed_src.join("rpress-seed/post.json"),
                "/wordpress/wp-content/rpress-seed/post.json".into(),
            ));
        }
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
                let items: Vec<serde_json::Value> = match list {
                    serde_json::Value::Array(a) => a,
                    serde_json::Value::Object(m) => m.into_values().collect(),
                    _ => Vec::new(),
                };
                {
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
    // Pidfile BEFORE the probe loop: a mid-spawn crash leaves a tracked
    // child `reap_stale_pidfile` can kill (format `pid:port`). Per-pid
    // name — a losing spawn race can never clobber the winner's.
    let pidfile = super::pidfile_path(www, slug, child.id());
    let _ = std::fs::write(&pidfile, format!("{}:{port}\n", child.id()));

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
        pidfile,
        seeded_post,
    })
}

/// Project ONE post + its meta as `wp-content/rpress-seed/post.json` —
/// the mu-plugin `wp_insert_post`s it on first admin request (WP must
/// create its own tables first; direct SQLite projection would fight
/// the installer). wp.db stays disposable; rpress.db is the only truth.
/// Returns false when the post isn't in rpress.db — no seed file then.
pub(crate) fn project_post(rpress_db: &Path, wp_content: &Path, post_id: i64) -> Result<bool> {
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
    let Some(post) = post else { return Ok(false) };
    let mut st = src.prepare(sql!("bridge_playground__select_post_meta"))?;
    let meta: serde_json::Map<String, serde_json::Value> = st
        .query_map([post_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<(String, String)>>>()?
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(v)))
        .collect();
    // Attachment posts for the media modal: rpress.db holds every
    // attachment row+meta imported from the site — project ALL of them so
    // query-attachments lists the real library. guids keep the public
    // /wp-content/uploads URLs (served by the static site, not wp.db).
    let attachments: Vec<serde_json::Value> = {
        let mut a_st = src.prepare(sql!("bridge_playground__select_attachment_posts"))?;
        let rows: rusqlite::Result<Vec<serde_json::Value>> = a_st
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "ID": r.get::<_, i64>(0)?,
                    "post_title": r.get::<_, String>(1)?,
                    "post_content": r.get::<_, Option<String>>(2)?,
                    "post_excerpt": r.get::<_, Option<String>>(3)?,
                    "post_status": r.get::<_, Option<String>>(4)?,
                    "post_name": r.get::<_, Option<String>>(5)?,
                    "post_parent": r.get::<_, i64>(6)?,
                    "menu_order": r.get::<_, i64>(7)?,
                    "post_date": r.get::<_, String>(8)?,
                    "post_modified": r.get::<_, String>(9)?,
                    "guid": r.get::<_, Option<String>>(10)?,
                    "post_mime_type": r.get::<_, Option<String>>(11)?,
                }))
            })?
            .collect();
        rows?
    };
    /* Per-attachment meta: _wp_attached_file (relative path) +
       _wp_attachment_metadata (PHP-serialized file/w/h/sizes blob WP
       core's wp_prepare_attachment_for_js decodes) + alt. `sizes_json`
       in rpress.db is already the WP shape ({size:{file,width,height}}). */
    let mut attachment_meta: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    if !attachments.is_empty() {
        let mut m_st = src.prepare(sql!("bridge_playground__select_attachment_meta"))?;
        for row in m_st.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })? {
            let (pid, path, w, h, sizes_json, alt, _caption) = row?;
            let sizes: serde_json::Value = sizes_json
                .as_deref()
                .filter(|s| !s.is_empty() && *s != "{}")
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            let meta_blob = crate::php_serialize::serialize(&serde_json::json!({
                "file": path.clone().unwrap_or_default(),
                "width": w,
                "height": h,
                "sizes": sizes,
            }));
            let mut m = serde_json::Map::new();
            m.insert("_wp_attached_file".into(), serde_json::Value::String(path.unwrap_or_default()));
            m.insert(
                "_wp_attachment_metadata".into(),
                serde_json::Value::String(String::from_utf8_lossy(&meta_blob).into_owned()),
            );
            if let Some(a) = alt.filter(|s| !s.is_empty()) {
                m.insert("_wp_attachment_image_alt".into(), serde_json::Value::String(a));
            }
            attachment_meta.insert(pid.to_string(), serde_json::Value::Object(m));
        }
    }
    let seed_dir = wp_content.join("rpress-seed");
    std::fs::create_dir_all(&seed_dir)?;
    std::fs::write(
        seed_dir.join("post.json"),
        serde_json::to_string(&serde_json::json!({
            "post": post, "meta": meta,
            "attachments": attachments,
            "attachment_meta": attachment_meta,
        }))?,
    )?;
    Ok(true)
}

/// Project the WHOLE site for the frontend bridge: every published post +
/// meta + attachment rows + nav menus + theme locations, as `site.json`.
/// Unlike `project_post` (one wp_insert_post inside WP), this runs on the
/// frontend path where WP doesn't see an `admin_init` — the mu-plugin
/// seeds wp.db directly on `init` (see bridge.php) using these exact
/// field names. Returns true when at least one post projected.
pub(crate) fn project_frontend(rpress_db: &Path, wp_content: &Path) -> Result<bool> {
    let src = crate::db::conn(rpress_db).context("open rpress.db for frontend projection")?;
    let mut posts: Vec<serde_json::Value> = {
        let mut st = src.prepare(sql!("bridge_frontend__select_posts"))?;
        let rows: rusqlite::Result<Vec<serde_json::Value>> = st
            .query_map([], |r| {
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
                    "_featured_media_id": r.get::<_, Option<i64>>(11)?,
                }))
            })?
            .collect();
        rows?
    };
    // Map rpress `featured_media_id` (media.id) → attachment wp_id.
    {
        let mut map = std::collections::HashMap::<i64, i64>::new();
        let mut st = src.prepare("SELECT id, wp_id FROM media")?;
        for r in st.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))? {
            let (id, wp) = r?;
            map.insert(id, wp);
        }
        for p in &mut posts {
            if let Some(fid) = p.get("_featured_media_id").and_then(|v| v.as_i64()) {
                if let Some(wp) = map.get(&fid) {
                    p.as_object_mut().unwrap().insert("_thumbnail_id".into(), serde_json::json!(wp));
                }
            }
            if let serde_json::Value::Object(m) = p { m.remove("_featured_media_id"); }
        }
    }
    let mut meta: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    {
        let mut st = src.prepare(sql!("bridge_frontend__select_post_meta"))?;
        for row in st.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)))? {
            let (pid, k, v) = row?;
            meta.entry(pid.to_string())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
                .as_object_mut()
                .unwrap()
                .insert(k, serde_json::Value::String(v));
        }
    }
    let attachments: Vec<serde_json::Value> = {
        let mut st = src.prepare(sql!("bridge_playground__select_attachment_posts"))?;
        let rows: rusqlite::Result<Vec<serde_json::Value>> = st
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "ID": r.get::<_, i64>(0)?,
                    "post_title": r.get::<_, String>(1)?,
                    "post_content": r.get::<_, Option<String>>(2)?,
                    "post_excerpt": r.get::<_, Option<String>>(3)?,
                    "post_status": r.get::<_, Option<String>>(4)?,
                    "post_name": r.get::<_, Option<String>>(5)?,
                    "post_parent": r.get::<_, i64>(6)?,
                    "menu_order": r.get::<_, i64>(7)?,
                    "post_date": r.get::<_, String>(8)?,
                    "post_modified": r.get::<_, String>(9)?,
                    "guid": r.get::<_, Option<String>>(10)?,
                    "post_mime_type": r.get::<_, Option<String>>(11)?,
                }))
            })?
            .collect();
        rows?
    };
    let mut attachment_meta: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    {
        let mut st = src.prepare(sql!("bridge_playground__select_attachment_meta"))?;
        for row in st.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
            ))
        })? {
            let (pid, path, w, h, sizes_json, alt) = row?;
            let sizes: serde_json::Value = sizes_json
                .as_deref()
                .filter(|s| !s.is_empty() && *s != "{}")
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            let meta_blob = crate::php_serialize::serialize(&serde_json::json!({
                "file": path.clone().unwrap_or_default(),
                "width": w,
                "height": h,
                "sizes": sizes,
            }));
            let mut m = serde_json::Map::new();
            m.insert("_wp_attached_file".into(), serde_json::Value::String(path.unwrap_or_default()));
            m.insert(
                "_wp_attachment_metadata".into(),
                serde_json::Value::String(String::from_utf8_lossy(&meta_blob).into_owned()),
            );
            if let Some(a) = alt.filter(|s| !s.is_empty()) {
                m.insert("_wp_attachment_image_alt".into(), serde_json::Value::String(a));
            }
            attachment_meta.insert(pid.to_string(), serde_json::Value::Object(m));
        }
    }
    /* nav menus → wp_terms rows (term_id = wp_term_id) + nav_menu_item posts.
       rpress `menu_items.url` stays absolute-origin; WP rewrites nothing —
       the proxy pass in public_page strips the origin prefix the same way
       it rewrites WP_HOME. */
    let menus: Vec<serde_json::Value> = {
        let mut st = src.prepare(
            "SELECT m.wp_term_id, m.name, COALESCE(m.location,''), COALESCE(m.lang,''),
                    COALESCE(tl.location,'')
             FROM menus m LEFT JOIN theme_locations tl ON tl.menu_id=m.id
             ORDER BY m.wp_term_id",
        )?;
        let rows: rusqlite::Result<Vec<serde_json::Value>> = st
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "wp_term_id": r.get::<_, i64>(0)?,
                    "name": r.get::<_, String>(1)?,
                    "location": r.get::<_, String>(2)?,
                    "lang": r.get::<_, String>(3)?,
                    "theme_location": r.get::<_, String>(4)?,
                }))
            })?
            .collect();
        rows?
    };
    /* gva__template + nav_menu_item posts live ONLY in the dump — rpress
       deliberately doesn't model plugin-internal CPTs. Parse wp_posts +
       wp_postmeta for those types straight off the dump so the theme
       builder's layout rows (header/footer/single templates) exist. */
    let mut extra_posts: Vec<serde_json::Value> = Vec::new();
    let mut extra_meta: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    // wp_content = <docroot>/wp-content → [0]=wp-content [1]=docroot
    // [2]=<slug> [3]=bridge [4]=www [5]=work/<site> — dump sits there.
    let site_dir = wp_content.ancestors().nth(5);
    if let Some(dump) = site_dir
        .map(|d| d.join("dump.sql"))
        .filter(|p| p.is_file())
    {
        let mut postmap: std::collections::HashMap<i64, serde_json::Value> = std::collections::HashMap::new();
        let res = crate::sqldump::rows(&dump, &["wp_posts", "wp_postmeta"], |table, v| {
            if table == "wp_posts" {
                let ptype = v.get(20).and_then(|x| x.as_str()).unwrap_or("");
                if matches!(ptype, "gva__template" | "nav_menu_item" | "elementor_library") {
                    let id = v.get(0).and_then(|x| x.as_i64()).unwrap_or(0);
                    if id > 0 {
                        postmap.insert(id, serde_json::json!({
                            "ID": id,
                            "post_author": v.get(1).and_then(|x| x.as_i64()).unwrap_or(1),
                            "post_date": v.get(2).and_then(|x| x.as_str()).unwrap_or(""),
                            "post_content": v.get(4).and_then(|x| x.as_str()).unwrap_or(""),
                            "post_title": v.get(5).and_then(|x| x.as_str()).unwrap_or(""),
                            "post_excerpt": v.get(6).and_then(|x| x.as_str()).unwrap_or(""),
                            "post_status": v.get(7).and_then(|x| x.as_str()).unwrap_or("publish"),
                            "comment_status": v.get(8).and_then(|x| x.as_str()).unwrap_or("closed"),
                            "ping_status": v.get(9).and_then(|x| x.as_str()).unwrap_or("closed"),
                            "post_password": v.get(10).and_then(|x| x.as_str()).unwrap_or(""),
                            "post_name": v.get(11).and_then(|x| x.as_str()).unwrap_or(""),
                            "post_modified": v.get(14).and_then(|x| x.as_str()).unwrap_or(""),
                            "post_parent": v.get(17).and_then(|x| x.as_i64()).unwrap_or(0),
                            "guid": v.get(18).and_then(|x| x.as_str()).unwrap_or(""),
                            "menu_order": v.get(19).and_then(|x| x.as_i64()).unwrap_or(0),
                            "post_type": ptype,
                            "post_mime_type": v.get(21).and_then(|x| x.as_str()).unwrap_or(""),
                        }));
                    }
                }
            } else if table == "wp_postmeta" {
                let pid = v.get(1).and_then(|x| x.as_i64()).unwrap_or(0);
                if pid > 0 {
                    if let (Some(k), Some(val)) = (v.get(2).and_then(|x| x.as_str()), v.get(3).and_then(|x| x.as_str())) {
                        extra_meta
                            .entry(pid.to_string())
                            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
                            .as_object_mut()
                            .unwrap()
                            .insert(k.to_string(), serde_json::Value::String(val.to_string()));
                    }
                }
            }
            Ok(())
        }).context("parse dump.sql for gva/nav template posts")?;
        extra_posts = postmap.into_values().collect();
    }
    let menu_items: Vec<serde_json::Value> = {
        let mut st = src.prepare(
            "SELECT mi.wp_id, mi.menu_id, m.wp_term_id, COALESCE(mi.parent_id,0),
                    COALESCE(mi.ord,0), COALESCE(mi.label,''), COALESCE(mi.url,''),
                    COALESCE(mi.object_id,0), COALESCE(mi.item_type,''),
                    COALESCE(mi.title_attr,''), COALESCE(mi.target,''),
                    COALESCE(mi.classes,''), COALESCE(mi.xfn,''), COALESCE(mi.description,'')
             FROM menu_items mi JOIN menus m ON m.id=mi.menu_id
             ORDER BY mi.wp_id",
        )?;
        let rows: rusqlite::Result<Vec<serde_json::Value>> = st
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "ID": r.get::<_, i64>(0)?,
                    "menu_term_id": r.get::<_, i64>(2)?,
                    "parent_id": r.get::<_, i64>(3)?,
                    "ord": r.get::<_, i64>(4)?,
                    "label": r.get::<_, String>(5)?,
                    "url": r.get::<_, String>(6)?,
                    "object_id": r.get::<_, i64>(7)?,
                    "item_type": r.get::<_, String>(8)?,
                    "title_attr": r.get::<_, String>(9)?,
                    "target": r.get::<_, String>(10)?,
                    "classes": r.get::<_, String>(11)?,
                    "xfn": r.get::<_, String>(12)?,
                    "description": r.get::<_, String>(13)?,
                }))
            })?
            .collect();
        rows?
    };
    // Taxonomy rows: terms + term_taxonomy + post term relationships.
    // wp_term_id/term_taxonomy_id are assigned inside the bridge at seed
    // time — carry the rpress term id + taxonomy + slug and let PHP map.
    let terms: Vec<serde_json::Value> = {
        let mut st = src.prepare(
            "SELECT t.id, t.taxonomy, t.slug, t.name, COALESCE(t.description,''),
                    COALESCE(p.slug,'') FROM terms t
             LEFT JOIN terms p ON p.id = t.parent_id ORDER BY t.id",
        )?;
        let rows: rusqlite::Result<Vec<serde_json::Value>> = st
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "id": r.get::<_, i64>(0)?,
                    "taxonomy": r.get::<_, String>(1)?,
                    "slug": r.get::<_, String>(2)?,
                    "name": r.get::<_, String>(3)?,
                    "description": r.get::<_, String>(4)?,
                    "parent_slug": r.get::<_, String>(5)?,
                }))
            })?
            .collect();
        rows?
    };
    let post_terms: Vec<serde_json::Value> = {
        let mut st = src.prepare(
            "SELECT p.wp_id, pt.taxonomy, pt.slug
             FROM post_terms pt JOIN posts p ON p.id = pt.post_id
             WHERE p.wp_id IS NOT NULL",
        )?;
        let rows: rusqlite::Result<Vec<serde_json::Value>> = st
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "post_wp_id": r.get::<_, i64>(0)?,
                    "taxonomy": r.get::<_, String>(1)?,
                    "slug": r.get::<_, String>(2)?,
                }))
            })?
            .collect();
        rows?
    };
    let mut options: serde_json::Map<String, serde_json::Value> = {
        let mut st = src.prepare("SELECT key, value FROM settings WHERE key IN ('show_on_front','page_on_front','posts_per_page','stylesheet','template','permalink_structure','nav_menu_locations','theme_mods_tevily','theme_mods_tevily_child','custom_logo','site_icon','blogname','blogdescription')")?;
        let mut m = serde_json::Map::new();
        for row in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
            let (k, v) = row?;
            m.insert(k, serde_json::Value::String(v));
        }
        m
    };
    // Merge dump-parsed meta into the rpress meta map — postmeta is
    // keyed by wp_id in both sources.
    for (pid, m) in extra_meta {
        let tgt = meta
            .entry(pid)
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if let (serde_json::Value::Object(t), serde_json::Value::Object(src_m)) = (tgt, m) {
            for (k, v) in src_m {
                t.insert(k, v);
            }
        }
    }
    // Backfill seed options missing from settings (imports predate the
    // allowlist) from the same dump.sql the extra posts came from.
    {
        let want = ["permalink_structure", "blogname", "show_on_front", "page_on_front"];
        if let Some(dump) = site_dir.map(|d| d.join("dump.sql")).filter(|p| p.is_file()) {
            for k in want {
                if !options.contains_key(k) {
                    if let Some(v) = option_from_dump(Some(dump.as_path()), k) {
                        options.insert(k.to_string(), serde_json::Value::String(v));
                    }
                }
            }
            /* Plugin/theme render state (elementor kit colors, trustindex
               connection keys, cf7 settings, redux options) lives only in
               the origin wp_options — without it the theme renders with
               default styling and plugin widgets error (observed: grey
               buttons + Trustindex "connect your business" bar). Seeded
               as a raw {name: value} map; bridge.php skips bridge-owned
               keys and unserializes before insert. */
            let mut dump_opts: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            let prefix = crate::sqldump::detect_prefix(&dump).unwrap_or_else(|_| "wp_".into());
            let opt_table = format!("{prefix}options");
            let _ = crate::sqldump::rows(&dump, &[opt_table.as_str()], |_, v| {
                if v.len() > 2 {
                    if let (Some(k), Some(val)) = (v[1].to_string_lossy(), v[2].to_string_lossy()) {
                        if !k.is_empty() && !dump_opts.contains_key(&k) {
                            dump_opts.insert(k, serde_json::Value::String(val));
                        }
                    }
                }
                Ok(())
            });
            let seed_tmp = wp_content.join("rpress-seed");
            std::fs::create_dir_all(&seed_tmp)?;
            serde_json::to_writer(
                std::fs::File::create(wp_content.join("rpress-seed").join("options.json"))?,
                &dump_opts,
            )?;
        }
    }
    let seed_dir = wp_content.join("rpress-seed");
    std::fs::create_dir_all(&seed_dir)?;
    std::fs::write(
        seed_dir.join("site.json"),
        serde_json::to_string(&serde_json::json!({
            "posts": posts,
            "extra_posts": extra_posts,
            "meta": meta,
            "attachments": attachments,
            "attachment_meta": attachment_meta,
            "menus": menus,
            "menu_items": menu_items,
            "terms": terms,
            "post_terms": post_terms,
            "options": options,
        }))?,
    )?;
    Ok(!posts.is_empty())
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
