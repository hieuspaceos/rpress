---
phase: 1
title: "Bundle goodlayers-core into rpress; make PB bridge always-on"
status: done
priority: P1
effort: "4h"
---

# Phase 1: Bundle `goodlayers-core`; make the PB bridge always-on

## Problem

`body_is_gdlr` (`src/db/misc.rs:1032`) detects the *format* — every site
(blueskies/dmc/imagedmc included) stores `body_json` as gdlr nodes and uses
the PB editor. But only aucoeur-test has `www/wp-content/plugins/goodlayers-core`
mounted + `PLUGINS_PHP=goodlayers`. Since `gdlr_pb.rs` is deleted, the other
sites' `/admin/api/gdlr/ajax` 307s into a bridge they cannot run → dead editor.

Per the seam plan premise, the builder plugin is **inherited**, like the core
— it belongs to rpress, not to the origin site. So ship it with rpress.

## Change

1. **Bundle the plugin.** Copy
   `work/aucoeur-test/www/wp-content/plugins/goodlayers-core` →
   `templates/wp-plugins/goodlayers-core` (~20 MB, includes `framework/`,
   `include/`, `goodlayers-core.php`). Add it to the templates dir that
   `serve`/`site` already ships so it travels with the binary/site tree.
   Check `.gitignore` doesn't exclude it; add a `templates/wp-plugins/README`
   one-liner noting provenance (aucoeur-test origin dump, 2026-10-27).

2. **Always mount in bridge.** In `src/admin/bridge/playground.rs`
   `spawn()`: when `slug == "goodlayers"` (adapter slug), if
   `www/wp-content/plugins/goodlayers-core` is absent, mount
   `templates/wp-plugins/goodlayers-core` as
   `/wordpress/wp-content/plugins/goodlayers-core` instead — same mechanics
   as the existing per-subdir mounts. If both exist, site wins (site
   overrides bundled, matching WP plugin semantics).

3. **Always opt-in.** `src/admin/bridge/mod.rs` `opt_in()`: return true for
   `slug == "goodlayers"` regardless of `PLUGINS_PHP` — the PB editor has no
   non-bridge path anymore, so gating it behind env made the 307 point at a
   disabled handler. Keep env opt-in for *other* plugins (tourmaster etc.).

4. **Fix stale docs.** `src/admin/plugins/goodlayers/mod.rs` head doc —
   remove the "307s here only when opted in … otherwise the legacy Rust
   dispatch answers" paragraph (fallback is gone). `src/admin/mod.rs:1449`
   comment slug `gdlr` → `goodlayers`. `wp-media-shim.js:6` endpoint comment.
   Delete `templates/admin/wp/gdlr/page-builder-var.json` (unread; verify by
   grep first).

## Verify

```bash
cargo check && cargo test --lib
# live: blueskies edit screen + ajax
rpress site serve blueskies &
# login via /admin/login, then:
curl -b ck -X POST localhost:PORT/admin/plugins/goodlayers/ajax \
  -d 'action=gdlr_core_get_skin_options'   # → {"status":"success",…}
curl -b ck -X POST localhost:PORT/admin/api/gdlr/ajax \
  -d 'action=gdlr_core_get_skin_options'   # → 307 → same success
# confirm bridge mounted bundled plugin (not absent):
pgrep -fl wp-playground-cli | grep goodlayers-core
```

Acceptance: all three legacy sites (blueskies, dmc, imagedmc) serve PB ajax
`{"status":"success"}` through both URLs; aucoeur-test unchanged.

## Risks

- **Size**: +20 MB in templates dir — acceptable (binary distribution is
  source tree, not embedded). If templates dir is meant only for admin
  assets, place under `templates/wp-plugins/` as a new first-class dir and
  resolve via existing template-dir resolution helper — check how `serve`
  locates `templates/admin/`.
- **License**: goodlayers-core is a commercial plugin; repo is private pet
  project, origin files already vendored under `www/` — same posture, but
  note it in the README.
- **Version drift**: bundled plugin frozen at aucoeur-test's version; per-site
  `www` copy wins when present, so sites with newer copies stay on theirs.
