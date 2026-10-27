//! PhpBridge runtime — lifecycle + proxy seam for `Backend::PhpBridge`
//! plugins. A bridge is an isolated WP runtime (MVP: `@wp-playground/cli`
//! sidecar over HTTP; php-cgi / php-wasm are future `PhpRuntime` impls)
//! that owns ITS OWN `wp.db` and never sees `rpress.db`.
//!
//! Hard rules (contract §bridge):
//! - opt-in per site (`PLUGINS_PHP=` env list) — no adapter spawns
//!   unprompted;
//! - admin-only: only `/admin/plugins/<slug>/…` requests reach it;
//! - localhost-only: runtimes bind 127.0.0.1, never a LAN interface;
//! - spawn-on-demand + idle TTL kill (default 10 min);
//! - writes return via whitelisted pull (`pull.rs`), never inbound HTTP.
//!
//! `wp.db` is disposable: wiped and re-projected on every spawn — never
//! a source of truth (two-source ban).

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Instant;

pub(crate) mod http;
pub(crate) mod playground;
pub(crate) mod pull;

/// Default idle time-to-live before the reaper kills a bridge.
pub(crate) const TTL_IDLE: std::time::Duration = std::time::Duration::from_secs(600);

/// One live bridge process keyed `{site}:{slug}`.
pub(crate) struct BridgeState {
    /// The spawned runtime (transport owner).
    pub(crate) rt: Box<dyn PhpRuntime>,
    /// Last proxied request — the reaper kills when now-last_used > TTL.
    pub(crate) last_used: Instant,
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
/// Empty = bridge fully off (default, zero runtime surface).
pub(crate) fn opt_in(env_list: &str, slug: &str) -> bool {
    env_list.split(',').map(str::trim).any(|s| !s.is_empty() && s == slug)
}

/// Spawn the runtime for `slug` at `site`, projecting `post_id`'s context
/// into a fresh `wp.db` under `www/bridge/<slug>/`. Errors when the
/// runtime binary is missing — caller renders the degrade screen.
pub(crate) fn spawn(
    site: &str,
    slug: &str,
    kind: RuntimeKind,
    www: &std::path::Path,
    rpress_db: &std::path::Path,
    post_id: Option<i64>,
    plugin_file: &str,
) -> Result<BridgeState> {
    match kind {
        RuntimeKind::Playground => playground::spawn(site, slug, www, rpress_db, post_id, plugin_file),
        RuntimeKind::PhpCgi => anyhow::bail!("php-cgi runtime not implemented in MVP (install Node + @wp-playground/cli)"),
    }
}

/// Get-or-spawn the bridge for `slug`; resets `last_used` on hit. If the
/// cached child's port is dead (killed externally), drop and respawn once —
/// without this the map served a stale port forever and every proxied
/// request degraded to the "Bridge unavailable" page until rpress restart.
pub(crate) fn bridge_for<'a>(
    bridges: &'a mut HashMap<String, BridgeState>,
    site: &str,
    slug: &str,
    kind: RuntimeKind,
    www: &std::path::Path,
    rpress_db: &std::path::Path,
    post_id: Option<i64>,
    plugin_file: &str,
) -> Result<&'a mut BridgeState> {
    let key = format!("{site}:{slug}");
    if let Some(st) = bridges.get_mut(&key) {
        if !st.rt.healthy() {
            let mut st = bridges.remove(&key).unwrap();
            let _ = st.rt.kill();
        }
    }
    if !bridges.contains_key(&key) {
        let st = spawn(site, slug, kind, www, rpress_db, post_id, plugin_file)?;
        bridges.insert(key.clone(), st);
    }
    let st = bridges.get_mut(&key).unwrap();
    st.last_used = Instant::now();
    Ok(st)
}

/// Drop bridges idle past `ttl`; returns the killed keys (for logging).
pub(crate) fn reap(bridges: &mut HashMap<String, BridgeState>, ttl: std::time::Duration) -> Vec<String> {
    let dead: Vec<String> = bridges
        .iter()
        .filter(|(_, st)| st.last_used.elapsed() > ttl)
        .map(|(k, _)| k.clone())
        .collect();
    for k in &dead {
        if let Some(mut st) = bridges.remove(k) {
            st.rt.kill();
        }
    }
    dead
}

/// Handle table shared on `App` — `Mutex<HashMap>` like sessions.
pub(crate) type Bridges = Arc<Mutex<HashMap<String, BridgeState>>>;

#[cfg(test)]
mod tests {
    #[test]
    fn opt_in_matches_exact_slug() {
        assert!(super::opt_in("goodlayers,tourmaster", "goodlayers"));
        assert!(!super::opt_in("goodlayers,tourmaster", "gdlr"));
        assert!(!super::opt_in("", "goodlayers"));
        assert!(super::opt_in(" goodlayers ", "goodlayers"));
    }
}
