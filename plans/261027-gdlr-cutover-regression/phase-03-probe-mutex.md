---
phase: 3
title: "Bridge health probe outside the global mutex + respawn rate-limit"
status: done
priority: P2
effort: "3h"
dependencies: [1]
---

# Phase 3: Probe outside mutex + respawn rate-limit

## Problem (security-reviewer RPRESS-BRIDGE-001, medium/high-confidence)

`bridge_for` (`src/admin/bridge/mod.rs:119-133`) calls `st.rt.healthy()` on
**every** request: `try_wait` + a blocking TCP connect + HTTP GET with 2s
timeouts (`http.rs` `probe`). It runs while the caller holds the global
`bridges` Mutex (`admin/mod.rs:1455`), so:

- every proxied request serializes behind a 2s-max blocking probe;
- a busy-but-alive bridge (slow WP request, cold WASM) reads dead →
  `remove` + `kill` + `remove_dir_all` + `spawn` (≤30s readiness loop) — all
  under the same lock → respawn storm + global stall.

## Design

```rust
let key = format!("{site}:{slug}");
// 1. Check liveness WITHOUT holding the lock across I/O:
//    the map only stores Arc<Mutex<BridgeState>> — or simpler:
//    probe the *port* before taking the lock.
```

Simplest correct shape (KISS, no restructuring of BridgeState ownership):

1. Read `port` + `token` under a short lock (clone cheap fields).
2. Release lock. `probe(port)` outside.
3. Re-lock; verify the entry still holds the same port/token (another
   request may have respawned already → reuse it, no double-spawn).
4. Dead or absent → `spawn` (spawn itself is blocking; keep it under lock —
   spawn storms are already bounded by step-3's identity check) but with a
   **per-key cooldown**: `HashMap<String, Instant> last_respawn` (or a field
   on BridgeState). Respawn allowed only if >60s since last attempt;
   otherwise surface "Bridge booting" 503 to the caller instead of
   double-killing.

`healthy()` itself: keep `try_wait` (free, under lock is fine); the *network*
probe is what must move out. So: lock → `try_wait` → if child alive, clone
port, unlock, `http::probe(port)` — alive: re-lock, return `&mut st`
(still requires re-borrow; adjust signature — `bridge_for` returns a
`BridgeGuard`/cloned handle or the caller splits into `lookup`+`forward`).

Since the caller (`admin/mod.rs:1454-1468`) only needs `token`, `port`-bound
`rt.request`, and `wp_db` path — alternative: change `bridge_for` to return
`(Arc<Mutex<BridgeState>>, token)` and let the caller lock the *entry* mutex
for `rt.request` — per-bridge lock, global map only brief. This removes the
global serialization entirely (the actual improvement, not just probe).

Pick whichever reads cleaner in code; requirement: no blocking I/O under
the global map lock, and no double-respawn on concurrent dead-bridge hits.

## Verify

```bash
cargo check && cargo test --lib
# concurrent dead-bridge hits → exactly one child:
pkill -f wp-playground-cli
seq 5 | xargs -P5 -I{} curl -sb ck -X POST URL/admin/plugins/goodlayers/ajax -d 'action=x' &
sleep 5; pgrep -f wp-playground-cli | wc -l   # expect ≤2 (parent+child), not 5
# busy bridge stays alive:
# while bridge is mid-request (tour options, ~10s), fire 5 parallel hits —
# all 200/302, zero respawns (pgrep count stable)
```

## Risks

- Changing `bridge_for`'s return type touches `admin/mod.rs` call sites —
  mechanical, compile-checked.
- Cooldown persistence: keep in the map (`last_spawn: Instant`), no extra
  file state.
- `wp_db`/`token` clone vs borrow lifetimes inside `spawn_blocking` —
  already `move`d; returning owned clones simplifies to `Result<(token, port-handle)>`.
