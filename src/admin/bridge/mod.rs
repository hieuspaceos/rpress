//! PhpBridge runtime — lifecycle + proxy seam for `Backend::PhpBridge`
//! plugins. A bridge is an isolated WP runtime (MVP: `@wp-playground/cli`
//! sidecar over HTTP; php-cgi / php-wasm are future `PhpRuntime` impls)
//! that owns ITS OWN `wp.db` and never sees `rpress.db`.
//!
//! Hard rules (contract §bridge):
//! - opt-in per site (`PLUGINS_PHP=` env list) — no adapter spawns
//!   unprompted (except `goodlayers`, always-on since the Rust port was
//!   deleted — see `opt_in`);
//! - admin-only: only `/admin/plugins/<slug>/…` requests reach it;
//! - localhost-only: runtimes bind 127.0.0.1, never a LAN interface;
//! - spawn-on-demand + idle TTL kill (default 10 min);
//! - writes return via whitelisted pull (`pull.rs`), never inbound HTTP.
//!
//! `wp.db` is disposable: wiped and re-projected on every spawn — never
//! a source of truth (two-source ban).
//!
//! Concurrency: the `Bridges` map lock is only ever held for cheap map
//! reads/writes — never across `healthy()`, `request()`, `spawn()`, or
//! `kill()` (all blocking I/O). Each bridge's own `Mutex` serializes its
//! requests. A network probe saying "dead" therefore cannot stall other
//! sites' traffic, and concurrent dead-bridge hits cannot double-spawn
//! (identity re-check under the map lock + per-key respawn cooldown).

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use std::process::Command;
use std::time::{Duration, Instant};

pub(crate) mod http;
pub(crate) mod php_cgi;
pub(crate) mod playground;
pub(crate) mod pull;

/// Default idle time-to-live before the reaper kills a bridge.
pub(crate) const TTL_IDLE: Duration = Duration::from_secs(300);

/// Minimum gap between respawns of the same key. A bridge that dies
/// faster than this is a boot failure — surface errors instead of a
/// kill/spawn storm on every request.
const RESPAWN_COOLDOWN: Duration = Duration::from_secs(60);

/// One live bridge process keyed `{site}:{slug}`.
pub(crate) struct BridgeState {
    /// The spawned runtime (transport owner).
    pub(crate) rt: Box<dyn PhpRuntime>,
    /// Last proxied request — the reaper kills when now-last_used > TTL.
    pub(crate) last_used: Instant,
    /// When this instance spawned — respawn cooldown gate.
    pub(crate) spawned_at: Instant,
    /// One-time bearer the proxy forwards; the mu-plugin validates it.
    /// Rotated per spawn.
    pub(crate) token: String,
    /// The runtime's disposable WP database (pull source).
    pub(crate) wp_db: PathBuf,
    /// `<www>/bridge/<slug>.pid` — records the live child pid + port so a
    /// later `serve` process can reap orphans a crash left behind.
    pub(crate) pidfile: PathBuf,
}

/// A spawned PHP runtime. Owns transport details; the manager owns
/// scheduling + lifecycle.
#[allow(dead_code)]
pub(crate) trait PhpRuntime: Send {
    /// Forward one request; returns (status, headers, body).
    fn request(
        &mut self,
        method: &str,
        path: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Result<BridgeResponse>;
    /// Runtime is answering (post-spawn probe + reaper liveness). Must
    /// verify identity with the spawn-time `token` — a bare TCP reply only
    /// proves *something* listens, not that it is our child.
    fn healthy(&mut self, token: &str) -> bool;
    /// Terminate the child process; idempotent.
    fn kill(&mut self);
    /// Child pid (the wrapper's pid for sandboxed/npx spawns — best-effort
    /// for pidfile orphan sweep).
    fn pid(&self) -> u32;
}

#[allow(dead_code)]
pub(crate) struct BridgeResponse {
    pub(crate) status: u16,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

/// Runtime type picked at spawn. `PhpCgi` is preferred when `php` is on
/// PATH (~60-70MB RSS, <200ms spawn, sandbox-exec confined); `Playground`
/// stays the fallback on hosts without php-cli.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub(crate) enum RuntimeKind {
    /// `@wp-playground/cli` — WASM PHP inside Node (~1.2GB RSS, multi-second
    /// cold start). Fallback when php-cli is absent.
    Playground,
    /// `php -S` builtin server behind HTTP on loopback, wrapped in
    /// `sandbox-exec` on macOS (deny-default FS + loopback-only net).
    PhpCgi,
}

/// Per-site opt-in list parsed from `PLUGINS_PHP=slug1,slug2`.
/// The `goodlayers` bridge is NOT opt-in: since `gdlr_pb.rs` was deleted,
/// the page-builder editor on every gdlr-format site has no non-bridge
/// path — env gating would leave the editor dead. Other plugins stay
/// env-gated.
pub(crate) fn opt_in(env_list: &str, slug: &str) -> bool {
    if slug == "goodlayers" {
        return true;
    }
    env_list.split(',').map(str::trim).any(|s| !s.is_empty() && s == slug)
}

/// Spawn the runtime for `slug` at `site`, projecting `post_id`'s context
/// into a fresh `wp.db` under `www/bridge/<slug>/`. Errors when the
/// runtime binary is missing — caller renders the degrade screen.
/// `<www>/bridge/<slug>.<pid>.pid` — per-spawn-attempt name so a losing
/// racer's cleanup can never delete the winner's record, and the file
/// sits one level up, out of the wiped runtime dir.
pub(crate) fn pidfile_path(www: &std::path::Path, slug: &str, pid: u32) -> PathBuf {
    www.join("bridge").join(format!("{slug}.{pid}.pid"))
}

/// Reap bridge processes a *previous* serve left behind (crash before
/// Drop/SIGTERM). `<slug>.<pid>.pid` files carry `pid:port`; each pid must
/// still be alive to die — stale files are just removed. Command-line
/// check keeps a PID-recycled process safe from a stray kill.
fn reap_stale_pidfile(www: &std::path::Path, slug: &str) {
    let dir = www.join("bridge");
    let prefix = format!("{slug}.");
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    for e in entries.filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().to_string();
        if !(name.starts_with(&prefix) && name.ends_with(".pid")) {
            continue;
        }
        let pf = e.path();
        let Ok(raw) = std::fs::read_to_string(&pf) else { continue };
        let pid: Option<u32> = raw.split(':').next().and_then(|s| s.trim().parse().ok());
        if let Some(pid) = pid {
            // Match the child's actual argv signature, not just the image
            // name: a PID recycled into an unrelated php/node process
            // must never be killed. Expected argv contains the bridge
            // docroot/port or the playground workdir.
            let ps = Command::new("ps").args(["-p", &pid.to_string(), "-o", "args="]).output();
            if let Ok(o) = ps {
                let args = String::from_utf8_lossy(&o.stdout);
                let ours = o.status.success()
                    && (args.contains(&format!("bridge/{slug}"))
                        || args.contains("wp-playground"));
                if ours {
                    let _ = Command::new("kill").arg(pid.to_string()).status();
                    eprintln!("bridge: reaped orphan {slug} pid {pid}");
                }
            }
        }
        let _ = std::fs::remove_file(&pf);
    }
}

fn spawn(
    site: &str,
    slug: &str,
    kind: RuntimeKind,
    www: &std::path::Path,
    rpress_db: &std::path::Path,
    post_id: Option<i64>,
    plugin_file: &str,
    templates: &std::path::Path,
) -> Result<BridgeState> {
    reap_stale_pidfile(www, slug);
    // Runtimes write their own per-pid pidfile right after Command::spawn.
    // The port picked by free_port() can be stolen between the probe
    // close and `php -S`/cli's bind; the identity probe catches it
    // (foreign listener can't answer our token), so retry the spawn
    // once with a fresh port instead of surfacing a flaky failure.
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..2 {
        let st = match kind {
            RuntimeKind::Playground => playground::spawn(site, slug, www, rpress_db, post_id, plugin_file, templates),
            RuntimeKind::PhpCgi => php_cgi::spawn(site, slug, www, rpress_db, post_id, plugin_file, templates),
        };
        let mut st = match st {
            Ok(st) => st,
            // Setup errors (missing binary, template failure) aren't
            // port races — fail fast, no retry.
            Err(e) => return Err(e),
        };
        // Post-spawn identity check: the port must answer OUR token, not just
        // reply — a recycled port behind a foreign listener fails here and the
        // half-built runtime is killed rather than adopted.
        let mut rt = std::mem::replace(&mut st.rt, Box::new(DeadRuntime));
        if !rt.healthy(&st.token) {
            rt.kill();
            if attempt == 1 {
                anyhow::bail!("bridge {slug}: identity probe failed after spawn");
            }
            eprintln!("bridge {slug}: identity probe failed (stolen port?) — retrying spawn");
            last_err = Some(anyhow::anyhow!("identity probe failed"));
            continue;
        }
        st.rt = rt;
        return Ok(st);
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("bridge {slug}: spawn failed")))
}

/// Placeholder runtime swapped in while `healthy()` verifies a spawn —
/// `BridgeState.rt` can't be empty mid-check.
struct DeadRuntime;
impl PhpRuntime for DeadRuntime {
    fn request(&mut self, _m: &str, _p: &str, _h: &[(String, String)], _b: &[u8]) -> Result<BridgeResponse> {
        anyhow::bail!("dead runtime")
    }
    fn healthy(&mut self, _t: &str) -> bool { false }
    fn kill(&mut self) {}
    fn pid(&self) -> u32 { 0 }
}

/// Per-bridge handle — the map entry. The outer `Bridges` lock guards the
/// map only; the inner mutex serializes this bridge's runtime.
pub(crate) type BridgeHandle = Arc<Mutex<BridgeState>>;
/// No blocking I/O under the map lock — the map only hands out/clones
/// entry handles. All slow work (probe, request, spawn, kill) runs under
/// the *entry's own* mutex, so a busy or dead bridge stalls only its own
/// callers, never other sites'. In-place respawn: concurrent dead-bridge
/// hits queue on the entry mutex; the first waiter spawns, the rest see
/// `healthy() == true` and proceed — exactly one child per death.
/// `spawned_at` enforces a per-key cooldown: a bridge dying <60s after
/// spawn is a boot failure — error out instead of storming npx.
pub(crate) fn bridge_for(
    bridges: &Bridges,
    site: &str,
    slug: &str,
    kind: RuntimeKind,
    www: &std::path::Path,
    rpress_db: &std::path::Path,
    post_id: Option<i64>,
    plugin_file: &str,
    templates: &std::path::Path,
) -> Result<BridgeHandle> {
    let key = format!("{site}:{slug}");

    let handle = {
        let map = bridges.lock();
        map.get(&key).cloned()
    };
    if let Some(h) = handle {
        {
            let mut st = h.lock();
            let tok = st.token.clone();
            if !st.rt.healthy(&tok) {
                if st.spawned_at.elapsed() < RESPAWN_COOLDOWN {
                    anyhow::bail!("bridge {key} died <60s after spawn — not respawning");
                }
                let _ = st.rt.kill();
                let _ = std::fs::remove_file(&st.pidfile);
                let fresh = spawn(site, slug, kind, www, rpress_db, post_id, plugin_file, templates)?;
                st.rt = fresh.rt;
                st.token = fresh.token;
                st.wp_db = fresh.wp_db;
                st.pidfile = fresh.pidfile;
                st.spawned_at = Instant::now();
            }
            st.last_used = Instant::now();
        }
        return Ok(h);
    }

    // First-ever spawn for this key. Spawn outside the map lock; on the
    // rare first-hit race the loser is killed — bounded waste, no stall.
    let st = spawn(site, slug, kind, www, rpress_db, post_id, plugin_file, templates)?;
    let mut map = bridges.lock();
    if let Some(cur) = map.get(&key) {
        let out = cur.clone();
        drop(map);
        let mut loser = st;
        let _ = loser.rt.kill();
        // Per-pid pidfiles: the loser's file is uniquely its own — remove
        // it unconditionally; the winner's record is untouchable.
        let _ = std::fs::remove_file(&loser.pidfile);
        out.lock().last_used = Instant::now();
        return Ok(out);
    }
    let h = Arc::new(Mutex::new(BridgeState { spawned_at: Instant::now(), ..st }));
    map.insert(key, h.clone());
    Ok(h)
}

/// Drop bridges idle past `ttl`; returns the killed keys (for logging).
/// Non-blocking: `try_lock` skips a busy entry (in-flight proxy traffic
/// proves it alive anyway) so the periodic sweep never queues behind a
/// long request, and the map lock is never held across a blocking lock.
/// Kill runs outside the map lock — `kill()` is blocking wait.
pub(crate) fn reap(bridges: &Bridges, ttl: Duration) -> Vec<String> {
    let dead: Vec<(String, BridgeHandle)> = {
        let mut map = bridges.lock();
        let mut out = Vec::new();
        map.retain(|k, st| {
            let idle = st.try_lock().map(|g| g.last_used.elapsed() > ttl).unwrap_or(false);
            if idle {
                out.push((k.clone(), st.clone()));
            }
            !idle
        });
        out
    };
    let mut out = Vec::with_capacity(dead.len());
    for (k, h) in dead {
        match h.try_lock() {
            Some(mut st) => {
                st.rt.kill();
                let _ = std::fs::remove_file(&st.pidfile);
                out.push(k);
            }
            // Race with the eligibility pass — a request arrived between
            // drain and now. No kill happened here: defer instead of
            // claiming one. (The entry was dropped from the map; the
            // bridge's own Drop kills the child when the handle dies.)
            None => eprintln!("bridge: reap deferred (in-flight request) {k}"),
        }
    }
    out
}
pub(crate) type Bridges = Arc<Mutex<HashMap<String, BridgeHandle>>>;

#[cfg(test)]
mod tests {
    #[test]
    fn opt_in_matches_exact_slug() {
        assert!(super::opt_in("goodlayers,tourmaster", "goodlayers"));
        assert!(super::opt_in("", "goodlayers")); // always-on
        assert!(!super::opt_in("", "tourmaster"));
        assert!(!super::opt_in("tourmaster", "x"));
        assert!(!super::opt_in("goodlayersx", "tourmaster"));
    }
}
