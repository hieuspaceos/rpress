---
title: "Phase 1: PHP bridge lifecycle hardening"
status: todo
---

# Phase 1: PHP bridge lifecycle hardening

## Overview

Highest-priority ops fix for multi-site systemd deployment. Today
`rpress serve` only listens for `ctrl_c` (SIGINT); under `systemctl restart`
(SIGTERM) every live `php -S` / playground bridge is orphaned (~100MB-1.2GB
each) **and keeps its port**. Port selection is also a TOCTOU: the probe
listener is dropped before the child binds, and `healthy()` only checks "any
bytes back" — an orphan of another site answers the probe and the admin sees
`403 bridge token mismatch`.

## Evidence

- `src/admin/mod.rs:291` — `tokio::signal::ctrl_c()` only; no SIGTERM.
- `src/admin/bridge/playground.rs:53-55` — `free_port()` binds `:0`, drops
  the listener, hands the port to the child ~hundreds of ms later.
- `src/admin/bridge/http.rs` `probe()` — liveness only, no identity check.
- `src/admin/bridge/mod.rs:199-215` — `reap()` holds the `Bridges` map lock
  while acquiring each entry mutex (a 120s HTTP read blocks ALL sites).
- No pidfile/port tracking under `www/bridge/<slug>/`.

## Requirements

- [ ] `serve` shuts down cleanly on SIGTERM **and** SIGINT.
- [ ] Shutdown kills every live bridge child (existing Drop paths run).
- [ ] Startup sweeps stale bridge children (pidfiles or cmdline match on
      `www/bridge/` docroots) so orphans can never hold a port across
      restarts.
- [ ] Spawn verifies bridge **identity** (X-Rpress-Bridge token → require a
      2xx response), retrying on a fresh port up to 3 times.
- [ ] `reap()` never blocks the whole bridge map on a slow entry
      (try_lock or atomic `last_used`).

## Implementation Steps

1. `admin/mod.rs` — replace `ctrl_c()` wait with
   `tokio::signal::unix::signal(SignalKind::terminate())` raced against
   `ctrl_c()`; on either, run the same bridge reap used today, then exit.
2. `bridge/mod.rs` — write `<www>/bridge/<slug>.pid` on spawn; delete on
   clean kill. At `serve` boot, scan `www/bridge/*.pid`: if pid alive and
   cmdline matches the recorded docroot → `kill(TERM)` then `KILL`; remove
   stale files.
3. `bridge/php_cgi.rs` + `playground.rs` — after spawn, probe `GET /` with
   the `X-Rpress-Bridge: <token>` header; require `2xx`. On mismatch or bind
   failure, pick another port and retry ≤3.
4. `bridge/mod.rs::reap` — switch entry access to `try_lock`; entries busy
   with an in-flight request are skipped this sweep (they'll be reaped on
   the next tick); keep the map lock held only for iteration.

## Todo

- [ ] SIGTERM handled + bridge reap on shutdown
- [ ] pidfile write/cleanup + startup stale-child sweep
- [ ] identity-probing spawn with port retry
- [ ] non-blocking reap
- [ ] rebuild release binary; retest: `systemctl restart`-equivalent
      (SIGTERM) leaves 0 `php -S` processes

## Success Criteria

- `kill -TERM <rpress pid>` → process exits, `pgrep -f "php -S"` empty.
- Spawn under parallel pressure never surfaces a foreign bridge (no 403
  token mismatch in admin).
- `reap()` worst-case holds map lock <1ms.
- e2e `check` still passes on aucoeur-test.

## Risk Assessment

- Sweep matching too broad → kills an unrelated process: mitigate by
  matching BOTH pidfile and `www/bridge/` docroot in cmdline.
- Identity probe adds ~1 RTT to spawn (already ~200-300ms) — negligible.

## Security Considerations

- Token identity probe prevents a foreign bridge being marked healthy —
  keeps the existing cross-site token fence meaningful.
