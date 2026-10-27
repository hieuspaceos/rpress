---
title: "Seam phase 06: PhpBridge runtime MVP — Playground + pull-write"
date: 2026-10-27
summary: "PhpRuntime trait + spawn/TTL lifecycle; @wp-playground/cli chosen by the decision gate (no php-cgi on host); opt-in PLUGINS_PHP env per site; pull-write WriteSpec whitelist; bridge status/kill CLI. 74 tests green, smoke 7/7."
---

# Phase 06: PhpBridge infrastructure

## What happened

Runtime decision gate resolved to `@wp-playground/cli` — `php-cgi` is
absent on the dev host while Node v26 + npx are present. `PhpRuntime`
trait + `RuntimeKind` enum keep php-cgi swappable later
(`#[allow(dead_code)]` until a second impl lands).

New `src/admin/bridge/` module:

- `mod.rs` — `PhpRuntime` trait (request/healthy/kill), `BridgeState`
  (rt + last_used + one-time token + wp_db path), `Bridges` handle
  (`Arc<Mutex<HashMap>>`, sessions precedent), `opt_in()` env parser,
  `bridge_for` spawn-on-demand, `reap()` TTL kill (10min).
- `http.rs` — minimal one-shot HTTP/1.1 client over `tokio TcpStream`
  (no reqwest/hyper-client in the tree; localhost-only, connection:close).
- `playground.rs` — npx spawn, free-port discovery, 30s health probe,
  wipe-on-spawn, `project_post` copies one post + its meta from
  rpress.db into WP-shaped `wp_posts`/`wp_postmeta`/`wp_options` in
  `www/bridge/<slug>/wp.db` via ATTACH (T25, bounded projection).
- `pull.rs` — `WriteSpec{table, key_column, key_prefix}` validator
  (rejects bare `%`, `%`/`_` inside the literal part, empty) + pull
  copy wp.db → rpress.db, `INSERT OR IGNORE`, scoped to whitelisted
  key prefixes only.
- `mu-plugin/bridge.php` — `hash_equals` bearer check on
  `X-Rpress-Bridge` vs `RPRESS_BRIDGE_TOKEN` env; 403 before WP loads.

Core wiring:

- `SiteCtx.plugins_php` — per-site opt-in list (SiteEnv `PLUGINS_PHP=`,
  `rpress site init --plugins-php`, `rpress serve` reads `PLUGINS_PHP`
  env as fallback). `plugin_screen` dispatches `Backend::PhpBridge`
  adapters to `bridge_dispatch`: opt-in gate → spawn-on-demand →
  proxy with bearer header → pull-write in the same request task
  (no drift window). Opted-out or missing runtime renders an
  html_admin degrade note; rest of admin unaffected (R10).
- TTL reaper: tokio task in `serve()` sweeps every `SiteCtx`'s bridge
  map every 60s.
- `post_id` heuristic: last numeric path segment feeds projection
  (`/admin/plugins/gdlr/.../edit/42` → post 42).
- `rpress bridge <site> status|kill` — disk-level operator tooling
  over `work/<site>/www/bridge/` (status lists wp.db size+age; kill
  removes the workdir → clean rebuild next request).
- `PluginAdapter::write_specs()` — default `&[]` = no writes land.

## Verification

- `cargo check --all-targets`: clean, 0 warnings.
- `cargo test`: 74 passed / 0 failed (new: opt_in exact-match,
  project_post copies post+meta, WriteSpec shape rejection).
- `rpress site serve blueskies --smoke`: 7/7 routes, no regression.
- `rpress bridge blueskies status`: reports empty workdir dir.

Not exercised end-to-end (no PhpBridge adapter ships yet — phase 07
cuts gdlr_pb over): real Playground spawn + proxy + pull round-trip.
The pieces are each unit-tested; the seam closes when phase 07 sends
traffic through it.

## Key decisions

- Pull-write (option b) over inbound mu-plugin POST (option a): no new
  HTTP write endpoint to harden; wp.db is disposable so its writes are
  already sandboxed — pull just selects which rows survive.
- wp.db wiped + re-projected on every spawn; never a source of truth
  (two-source ban, contract R3).
- Bridge token rotates per spawn; forwarded as a header, never the
  rpress session cookie (limits session leakage, R6-adjacent).
- Bridge CLI is disk-level, not IPC — live bridges reap via TTL anyway;
  kill removes the workdir so the next spawn is clean.

## Next

Phase 07: gdlr_pb adapter cutover — `backend() = PhpBridge`, real
`write_specs()` for `gdlr-core-*` meta + `plugin:goodlayers:*`
settings, first real traffic through the bridge.
