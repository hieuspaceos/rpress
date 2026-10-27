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
use std::time::{Duration, Instant};

pub(crate) mod http;
pub(crate) mod playground;
pub(crate) mod pull;

/// Default idle time-to-live before the reaper kills a bridge.
pub(crate) const TTL_IDLE: Duration = Duration::from_secs(600);

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
    /// Runtime is answering (post-spawn probe + reaper liveness).
    fn healthy(&mut self) -> bool;
    /// Terminate the child process; idempotent.
    fn kill(&mut self);
}

#[allow(dead_code)]
pub(crate) struct BridgeResponse {
    pub(crate) status: u16,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

/// Runtime type picked at spawn (decision gate, phase-06 spec).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)] // PhpCgi reserved for hosts without Node
pub(crate) enum RuntimeKind {
    /// `@wp-playground/cli` — WASM PHP inside Node, our only available
    /// runtime on hosts with Node (this host: node v26, no php-cgi).
    Playground,
    /// `php-cgi` behind FastCGI — pick where Node is a worse dep than
    /// php-cgi; requires the hardening list. Not implemented in MVP.
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
    match kind {
        RuntimeKind::Playground => playground::spawn(site, slug, www, rpress_db, post_id, plugin_file, templates),
        RuntimeKind::PhpCgi => anyhow::bail!("php-cgi runtime not implemented in MVP (install Node + @wp-playground/cli)"),
    }
}

/// Per-bridge handle — the map entry. The outer `Bridges` lock guards the
/// map only; the inner mutex serializes this bridge's runtime.
pub(crate) type BridgeHandle = Arc<Mutex<BridgeState>>;

/// Get-or-spawn the bridge for `slug`; resets `last_used` on hit.
///
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
            if !st.rt.healthy() {
                if st.spawned_at.elapsed() < RESPAWN_COOLDOWN {
                    anyhow::bail!("bridge {key} died <60s after spawn — not respawning");
                }
                let _ = st.rt.kill();
                let fresh = spawn(site, slug, kind, www, rpress_db, post_id, plugin_file, templates)?;
                st.rt = fresh.rt;
                st.token = fresh.token;
                st.wp_db = fresh.wp_db;
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
        out.lock().last_used = Instant::now();
        return Ok(out);
    }
    let h = Arc::new(Mutex::new(BridgeState { spawned_at: Instant::now(), ..st }));
    map.insert(key, h.clone());
    Ok(h)
}

/// Drop bridges idle past `ttl`; returns the killed keys (for logging).
/// Kill itself runs outside the map lock — `kill()` is blocking wait.
pub(crate) fn reap(bridges: &Bridges, ttl: Duration) -> Vec<String> {
    let dead: Vec<(String, BridgeHandle)> = {
        let mut map = bridges.lock();
        let keys: Vec<String> = map
            .iter()
            .filter(|(_, st)| st.lock().last_used.elapsed() > ttl)
            .map(|(k, _)| k.clone())
            .collect();
        keys.iter()
            .filter_map(|k| map.remove(k).map(|h| (k.clone(), h)))
            .collect()
    };
    let mut out = Vec::with_capacity(dead.len());
    for (k, h) in dead {
        let mut st = h.lock();
        st.rt.kill();
        out.push(k);
    }
    out
}

/// Shared per-site map: `{site}:{slug}` → live bridge.
/// Lock order: `Bridges` (map) → entry `Mutex` — never the reverse, and
/// never blocking I/O under the map lock.
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
