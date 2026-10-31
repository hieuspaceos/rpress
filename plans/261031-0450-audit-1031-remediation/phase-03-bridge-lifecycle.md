---
phase: 3
title: "Bridge lifecycle"
status: completed
priority: P2
effort: "2.5h"
dependencies: [2]
---

# Phase 3: Bridge lifecycle

## Goal

Make php-bridge processes impossible to orphan and impossible to
misreport: pidfile ownership is written early and verified on removal,
spawn races can't delete the winner's tracking file, pull failures
reach the editor, and the port-pick race shrinks.

## Files to Create / Modify

- Modify: `src/admin/bridge/mod.rs:157-180` — pidfile write ordering.
- Modify: `src/admin/bridge/mod.rs:257-266` — spawn-race loser cleanup.
- Modify: `src/admin/bridge/mod.rs:278-289` — `reap()` reported-kill
  correctness.
- Modify: `src/admin/mod.rs:1183-1192` — pull-failure propagation.
- Modify: `src/admin/bridge/playground.rs:56-58` — `free_port` TOCTOU.
- Modify: `src/admin/auth.rs:448-451` — `app_pass_touch` peer IP.

## Requirements

- [x] Pidfile is written **inside** each runtime's `spawn()`
      immediately after `Command::spawn()` returns (before the ~15s
      probe loop and `wp_install`), so a mid-spawn crash leaves a
      tracked child `reap_stale_pidfile` can kill.
- [x] The spawn-race loser removes the pidfile **only** when its
      contents still equal the loser's pid (read-compare-remove, or
      per-spawn unique pidfile names); a live winner's file is never
      deleted.
- [x] `reap()` does not push a key to the "killed" list when the
      second `try_lock` fails — log "reap deferred (in-flight request)"
      instead of claiming a kill that hasn't happened.
- [x] A failed `pull()` returns a non-2xx / error payload to the
      editor instead of WP's `success:1` — the comment at
      `admin/mod.rs:1185` becomes true behavior, not a warning about
      itself.
- [x] `free_port` either keeps the bound fd until `php -S` inherits it,
      retries spawn once on identity-probe failure, or the port is
      passed via `--fd`/env — a stolen port produces a retried spawn,
      not a surfaced failure.
- [x] `app_pass_sess` threads the real `ConnectInfo` peer through
      `need_auth` so `last_ip` records an address, not `"unknown"`.

## Implementation Steps

1. Move `fs::write(&pidfile, pid)` into `php_cgi::spawn` /
   `playground::spawn` right after `Command::spawn`; delete the late
   write in `bridge::spawn`.
2. In `bridge_for`'s loser arm: `fs::read_to_string(&loser.pidfile)`
   → remove only if `trim() == loser.rt.pid()`.
3. `reap()`: on `try_lock` `Err`, push the key to a `deferred` list /
   `eprintln!` and skip the `killed` push.
4. `plugin_screen`/bridge response path: match on `pull()` result;
   on `Err` return `StatusCode::BAD_GATEWAY` + a small error body the
   admin JS can show.
5. `free_port`: simplest correct fix is retry-on-probe-failure once
   (keep the drop-listener approach but loop the whole spawn).
6. `need_auth` gains an `Option<SocketAddr>` param (already has
   `headers`); `app_pass_sess` uses it.

## Verification

- Unit-style test or manual harness: two concurrent `bridge_for`
  calls → exactly one pidfile, its content == surviving pid.
- `pull()` failure path: kill the whitelist write (e.g. read-only
  file) → editor sees error, not `success:1`.
- `reap()` with a held lock reports 0 killed.
- `cargo test` green.
