---
title: "Seam phase 05: back-compat migration - legacy_option_keys fold"
date: 2026-10-27
summary: "One idempotent settings fold at boot migrates bare plugin keys to the plugin: namespace; verified live (folded copy + intact legacy, second boot no-op); TOUR_BASE unchanged; README operator notes"
---

# Seam phase 05: back-compat & migration

## What happened

Commit `1b0469b`. Existing pre-seam sites keep working with zero
operator action:

- `PluginAdapter::legacy_option_keys()` — each adapter declares the bare
  `settings` keys it owns. `migrate_all` folds each to
  `plugin:<slug>:<key>` via `INSERT OR IGNORE ... SELECT` — atomic, no
  deletes, namespaced rows never overwritten, `plugin:`-prefixed keys
  skipped (partial earlier runs safe).
- Tourmaster declares `tourmaster_general` + `tour_custom_taxs`.
  `post_meta` `tourmaster-*` keys are the plugin's wire format —
  explicitly documented as never-folded (the adapter round-trips what
  WP wrote).
- `migrate_all` now also runs inside `site build` (check phase): a
  deploy-only pipeline that never opens admin still gets schema DDL,
  key fold, and `taxonomy_map` seed.
- `TOUR_BASE`/`--tour-base` unchanged forever (deploy emits them);
  they resolve through `KindBases`.

## Verified on live aucoeur-test DB

- Boot wrote `plugin:tourmaster:tourmaster_general` (copy of the php-
  serialized blob); bare `tourmaster_general` row intact.
- Second boot: settings count unchanged (idempotent).
- `/admin/tours` lists 40 edit links; tour edit page shows the metabox
  ajax screen; public tour page renders (117 tourmaster refs);
  `/admin/tax/region` 200; `sidebar-tour` widget rows = 3.
- Unit test `migrate_plugin_keys_fold` covers fold + keep-legacy +
  no-overwrite + idempotent rerun. 69/69 tests.

## Next

Phase 06 — PhpBridge runtime MVP (php-cgi or Playground + write
whitelist). The remaining big piece; phases 01–05 built the seam it
plugs into.
