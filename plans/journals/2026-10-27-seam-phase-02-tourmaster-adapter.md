---
title: "Seam phase 02: tourmaster adapter + plugin screens dispatch"
date: 2026-10-27
summary: "Tourmaster became the reference PluginAdapter — screens dispatched under /admin/plugins/<slug>, promote_tours moved into adapter.migrate(), clone_post via adapter hook; code review caught a central-auth gap now fixed at the dispatcher"
---

# Seam phase 02: tourmaster adapter

## What happened

Completed `plans/261027-scoped-plugin-seam` phase 02 (commit `204f041`),
building on phase 01's `PluginAdapter`/`KindDef`/registry substrate
(committed earlier as `e8a46d2`).

- `ScreenDef`/`ScreenHandler`/`ScreenFuture` contract: raw-request
  handlers under `/admin/plugins/{slug}/{*rest}`; dispatcher calls
  `need_auth` once and hands `&Sess`/`&SiteCtx` to the handler — a screen
  can never ship unauthenticated (review finding 4).
- Tourmaster adapter exposes `kinds()` (tour + custom taxonomies),
  `SCREENS` (save-option, new-filter), `submenu()`, `migrate()` (tours
  DDL + post_meta→tours promote; boot logs `tourmaster: N rows
  promoted`), `clone_post()` (tours row inside the caller's tx).
- `save_tour_option` + `TourOptionAjax` moved out of `admin/posts.rs`;
  `new-filter` out of `admin/tax.rs`; `tour_submenu` and
  `crud::promote_tours` deleted. Old URLs keep working via 307 shims
  (`/admin/api/tour/save-option`, `/admin/tax/new-filter`) — vendored JS
  had no hardcoded hits but shims protect cached pages.
- Menu + dashboard At-a-Glance now loop `plugins::kind_defs()`; new
  plugin kinds appear with zero dashboard edits.
- Registry insertion asserts on duplicate slug/kind/section and on the
  reserved `plugins` section (red-team R5).

## Code review (fresh-context)

12 findings; applied the 5 mediums + 3 low fixes: auth-before-parse in
save-option, documented CSRF exemption (non-migrated actions only),
`#[serde(default)]` back on csrf, central auth at dispatcher, registry
collision asserts, put_map error → 500, reserved tax slugs rejected,
dashboard label pluralizes from `noun`.

## Verified

- 68/68 tests, `cargo check` zero warnings.
- Live on aucoeur-test: boot promote 68 rows; `/admin/plugins/tourmaster/
  new-filter` 200; old paths 307; unknown slug/screen 404; unauth → 303
  to login; save-option roundtrip `{"status":"success"}`; clone 303 with
  tours row copied; dashboard `951 Posts / 49 Pages / 44 Tours`.

## Next

Phase 03 — tour extraction: render/generate/lang/widgets decoupling
(`"tour"` literals still in posts.rs/admin serve path by design).
