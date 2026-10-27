---
title: "gdlr cutover regression fixes — 4 phases, all audit findings closed"
date: 2026-10-28
summary: "Post-audit remediation for 413906b: bundled goodlayers-core + always-on bridge, defs URL strip, probe moved under entry mutex, dump.sql settings fallback. blueskies/dmc PB editor verified live."
---

# gdlr cutover regression fixes

## What happened

Two-persona audit of the `gdlr_pb.rs` deletion (413906b) surfaced one
critical and three medium findings; executed
`plans/261027-gdlr-cutover-regression` (4 phases) to close them.

- **C1 (critical)**: `body_is_gdlr` is format-universal — blueskies/dmc/
  imagedmc store gdlr `body_json` and use the PB editor but carry no
  `goodlayers-core` plugin. The 307 cutover left them dead. Fix:
  goodlayers-core (20 MB, aucoeur-test copy) bundled under
  `templates/wp-plugins/`; `playground::spawn` mounts it when the site's
  own dir is absent (site copy wins); `opt_in("goodlayers")` → always
  true since no non-bridge path exists anymore.
- **M1**: defs JSON dumps carried 139 `http://127.0.0.1:64123` URLs →
  stripped at the source + `strip_origin_urls()` defense in `defs()`
  (rewrites `http(s)://<host>/wp-content/plugins/goodlayers-core` tails
  to root-relative).
- **M2**: `bridge_for` probed `healthy()` (blocking 2s HTTP) under the
  global `bridges` Mutex. Redesigned: map stores
  `Arc<Mutex<BridgeState>>` entries; the map lock only clones handles —
  probe/request/spawn/kill run under the entry mutex (per-bridge
  serialization, no fleet stall). In-place respawn → concurrent
  dead-bridge hits queue on the entry mutex, exactly one child spawned.
  `spawned_at` cooldown (60s) converts dying-fast bridges into errors
  instead of npx storms.
- **M3**: pre-cutover DBs lacked `stylesheet`/`active_plugins`.
  `option_from_dump` falls back to `<site>/dump.sql` via `sqldump::rows`
  (proper escape handling) and seeds resolved values back into
  `settings` — runs once per site.

## Verified live

- blueskies: login → bridge spawn mounts bundled goodlayers-core →
  `gdlr_core_get_skin_options` 200 `{"status":"success"}`;
  `stylesheet=tevily_child` + `active_plugins` seeded from dump.sql;
  `RPRESS_BRIDGE_THEME=tevily_child` in spawn args.
- dmc: same result — 200 success, `tevily_child`.
- Concurrency: killed blueskies bridge, 6 parallel ajax hits → exactly
  one new child (in-place respawn under entry mutex).
- `/admin/api/gdlr/ajax` → 307 → `/admin/plugins/goodlayers/ajax` ✓.
- `cargo check` clean, `cargo test` 37/37 (new `option_from_dump` unit
  test + updated `opt_in` expectations).
- grep `http://` in defs JSONs → only external docs links remain.

## Files

- `src/admin/bridge/mod.rs` — rewritten: `BridgeHandle` type, in-place
  respawn, cooldown, reap kills outside map lock.
- `src/admin/bridge/playground.rs` — `templates` param, bundled mount,
  `option_from_dump`, element-provider filter kept.
- `src/admin/bridge/mu-plugin/bridge.php` — unchanged this pass.
- `src/admin/mod.rs` — caller uses entry mutex; stale slug comment.
- `templates/wp-plugins/goodlayers-core/` — new (20 MB vendored plugin).
- `templates/admin/wp/gdlr/{elements,wrappers}.json` — URLs stripped;
  `page-builder-var.json` deleted (unread).
- `src/admin/plugins/goodlayers/boot.rs` — `strip_origin_urls` in `defs()`.
- `sites/aucoeur-test.env` keeps `PLUGINS_PHP=goodlayers` (harmless now;
  gate no longer consults it for goodlayers).

## Watchpoints

- First bridge request per site still pays WP install (~90-100s); a
  second 400-cold-response is normal — subsequent requests 200.
- Element-provider activation on blueskies/dmc is empty by design (no
  `add_element` registers in their plugin sets) — tevily uses a
  different builder family for theme elements.
- Accepted lows from the audit stay open: free_port TOCTOU, CRLF in
  `rest`, `rpress_seed_done` ordering.
