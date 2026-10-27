---
phase: 6
title: "PhpBridge runtime MVP + write whitelist"
status: done
priority: P1
effort: "3d"
dependencies: [1]
---

# Phase 6: PhpBridge infrastructure (php-cgi or Playground MVP)

## Goal

A `PhpBridge` backend exists and passes the contract's hard rules:
opt-in per site/plugin, admin-only, localhost-only, spawn-on-demand +
TTL kill, adapter-side write whitelist, documented blast radius. MVP
runtime = whichever is cheaper to stand up *per host*: `php-cgi` behind
a FastCGI client, or `@wp-playground/cli` sidecar (decision gate below).
php-wasm-in-process remains the documented target runtime, not MVP.

## Runtime decision gate (evaluate at phase start)

| Option | Host dep | Isolation | Effort | Verdict rule |
|---|---|---|---|---|
| `@wp-playground/cli` | Node + npx | WASM PHP, Node host surface | small | **default** — pick unless Node absent on deploy target |
| `php-cgi` | system php-cgi binary | OS-level (open_basedir, disable_functions, uid) | small | pick when Node is a worse dep than php-cgi; MUST apply the hardening list (contract §php-cgi) |
| php-wasm in-process | none (crate) | strongest | largest | future; trait is designed to not block it |

The bridge is a `PhpRuntime` trait (`request(method, path, headers,
body) -> Response`) with Playground and php-cgi impls behind it, so
swapping runtimes later is a config change.

**Verdict (2026-10-27):** `@wp-playground/cli` — `php-cgi` absent on
dev host; Node v26 + npx present. `RuntimeKind::PhpCgi` stays behind
`#[allow(dead_code)]` for hosts without Node.

## Findings covered

- **T23** — Bridge process lifecycle: spawn on first
  `/admin/plugins/<slug>` request, TTL kill after idle (default 10min),
  bind 127.0.0.1 only, health probe before proxying.
- **T24** — Write path (R3): bridge WP runs its OWN SQLite
  (`bridge/<slug>/wp.db` via sqlite-database-integration); it never
  sees rpress.db. Writes land by one of two channels — (a) mu-plugin
  POSTs writes back to `/admin/plugins/<slug>/write` which the adapter
  validates then executes on rpress.db; (b) rpress pulls declared
  option/meta keys out of wp.db read-only after each request. MVP picks
  **(b) pull**: simpler, no inbound HTTP from PHP, no write endpoint to
  harden. `WriteSpec` on the adapter declares allowed
  `{table, key_prefix}` (e.g. `{post_meta, "gdlr-core-%"}`,
  `{settings, "plugin:goodlayers:%"}`); the pull copies only matching
  rows.
- **T25** — Seed/projection: bridge WP needs posts/media context the
  plugin reads (`get_post`, `get_post_meta`, media list). MVP projects
  the single post being edited + its meta into wp.db on spawn
  (per-post ephemeral bridge), not the whole DB — bounded cost, tight
  blast radius.
- **T26** — Adapter availability degradation (R10): `backend() ==
  PhpBridge` + runtime missing → screen renders install instructions,
  admin otherwise unaffected.

## Key insights

- `/admin/plugins/{slug}/{*path}` dispatch (phase 02) is the natural
  proxy point: adapter `handle()` gets the request; `PhpBridge`
  adapters return `ProxyTo(port)` and core streams the response —
  the same handler signature serves `Rust` and `PhpBridge` backends.
- Auth: rpress session stays in rpress; the bridge URL is never
  reachable directly except via localhost, and the proxy forwards a
  one-time bridge token (mu-plugin validates it) rather than the rpress
  cookie — limits session leakage if WP is compromised (R6-adjacent).
- TTL kill: `tokio::time::sleep` reaper over a `Mutex<BridgeState>`
  in `App` (sessions map precedent, `src/admin/mod.rs:57`); each
  proxied request resets `last_used`.
- Pull-write ordering: pull runs *after* the response is collected, in
  the same request task — no async drift window where rpress.db is
  behind what the editor just saved.
- wp.db is disposable: delete+rebuild on every spawn; projection cost
  ~1 post + meta ≈ ms. Never a source of truth (two-source ban).

## Files to modify

- `src/admin/bridge/` (new) — `mod.rs` (`PhpRuntime` trait,
  `BridgeState`, spawn/kill/probe, TTL reaper), `playground.rs` (npx
  spawn + port discovery + SQLite wp.db path), `php_cgi.rs` (FastCGI
  over TCP + hardening env/ini flags), `pull.rs` (WriteSpec validation
  + copy into rpress.db), `mu-plugin/bridge.php` (projected context
  shims: auth token check, option get/set landing in wp.db tables the
  pull can read — write the PHP in `www/` vendored dir, not `src/`).
- `src/admin/mod.rs` — `App` gains `bridges: Arc<Mutex<HashMap<String,
  BridgeState>>>`; plugin-screen handler routes `PhpBridge` backends
  through proxy.
- `src/admin/plugins/mod.rs` — `WriteSpec` type; `Adapter.handle()`
  may return proxy intent.
- `src/site_init.rs`/`src/site_deploy.rs` — `plugins:` config stanza
  (opt-in list in env, e.g. `PLUGINS_PHP=goodlayers`) — opt-in gate.
- `src/main.rs` — `rpress bridge status/kill` debug subcommand
  (operator tooling).

## Implementation steps

1. `PhpRuntime` trait + `BridgeState` + spawn/probe/TTL reaper (tests
   with a stub `php-cgi`-equivalent `nc`-style fake, or a tiny
   `php -S` fixture).
2. wp.db projection + sqlite-database-integration bootstrap script.
3. `pull.rs` WriteSpec validator + unit tests (rejects `posts`,
   `users`, `sessions`, non-matching prefixes, other plugins' keys).
4. Proxy path through `/admin/plugins/{slug}/{*path}`; token issuance.
5. Opt-in env gate; missing-runtime degrade screen.
6. php-cgi hardening preset (ini flags, read-only mounts list) if the
   php-cgi impl is chosen — else document Playground choice.
7. `bridge status`/`kill` subcommand.

## Verification

```bash
cargo check && cargo test
cargo test bridge   # WriteSpec unit tests; spawn/TTL fake-runtime tests
```

Manual (with real runtime installed):
- `/admin/plugins/echo-test` → first request spawns, page loads;
  10min idle → process gone (`ps`); rpress.db `settings` untouched by
  writes outside whitelist (attempt a forbidden write via test hook →
  rejected + logged).
- `lsof -i` confirms 127.0.0.1 bind only.
- Degrade: rename runtime binary → plugin screen shows install note,
  `/admin` unaffected.

## Todos

- [x] `PhpRuntime` trait + lifecycle manager + TTL reaper
- [x] wp.db projection + bootstrap
- [x] Pull-write with WriteSpec validation + tests
- [x] Proxy dispatch + bridge token
- [x] Opt-in env gate + degrade screen
- [x] Runtime choice documented (Playground vs php-cgi) + hardening
- [x] `bridge status/kill` CLI

## Risks

- Node/php-cgi presence on deploy hosts — degrade screen covers it;
  deploy docs updated.
- Pull-write race if two admins edit simultaneously — pull is
  per-request, last-write-wins per key (documented semantics, same as
  WP).
- Playground startup latency (~1-3s) on first request — acceptable;
  show a "starting editor…" interstitial via the proxy on cold start.
- wp.db grows if plugin writes unbounded tables — pull only copies
  whitelisted rows; wp.db wiped on respawn (TTL) — self-healing.
