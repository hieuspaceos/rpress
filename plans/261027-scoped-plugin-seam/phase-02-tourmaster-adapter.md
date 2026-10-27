---
phase: 2
title: "Tourmaster adapter registration"
status: done
priority: P1
effort: "1.5d"
dependencies: [1]
---

# Phase 2: Tourmaster becomes the reference adapter

## Goal

Tourmaster registers as a `Backend::Rust` `PluginAdapter` owning kind
`tour`, its admin screens (list head, "Add New Tour", taxonomies, menu
entry, dashboard count, `save_tour_option` endpoint), the `tours` table
DDL, `promote_tours`, and the `sidebar-tour`/`widgets__recent_tours`
widget surface — proving the contract end-to-end while nothing user-visible
changes. Tour *render/generate* decoupling is phase 03; this phase moves
ownership only.

## Findings covered

- **T5** — `tourmaster` plugin already conforms: `PLUGIN` static
  (`src/admin/plugins/tourmaster/mod.rs:18-24`) with load/metabox/
  assets/save; `save.rs` syncs `tours` table
  (`src/admin/plugins/tourmaster/save.rs:330-337`).
- **T6** — `KindSpec` tour entry is core-owned
  (`src/admin/posts.rs:33`, HEAD_TOUR French column heads at
  `src/admin/posts.rs:22`); `/admin/tours` menu hardcoded
  (`src/admin/mod.rs:1096-1101`), dashboard count literal
  (`src/admin/dashboard.rs:99,111`).
- **T7** — `/admin/api/tour/save-option` route is a plugin endpoint in
  core router (`src/admin/mod.rs:163` → `src/admin/posts.rs:1491`).
- **T8** — `promote_tours` runs unconditionally at admin boot
  (`src/admin/mod.rs:87` → `src/db/posts.rs:1194-1226`); tour taxonomies
  land in `/admin/tax/*` via `tax_menu` catch-all
  (`src/admin/tax.rs:472-477`) and "Add New Filter" is a tourmaster
  feature in core tax screen (`src/admin/tax.rs:483-486`).

## Key insights

- `save_tour_option` (`src/admin/posts.rs:1491`) is the first real
  `admin_screens()` consumer — moves to
  `POST /admin/plugins/tourmaster/save-option`; keep a redirect shim at
  the old path for one release (JS lives in vendored tourmaster assets —
  check `www/wp-content/plugins/tourmaster/**` for the literal URL before
  deleting the shim; grep the vendored JS at implementation time).
- `HEAD_TOUR` is a site-specific label set — acceptable inside the
  adapter now; phase 04 makes column labels data-driven.
- `tour_submenu` (`src/admin/mod.rs` near :1096) injects taxonomy links;
  `ScreenDef` needs a `submenu(db) -> Vec<(url,label)>` hook — reuse the
  existing fn signature.
- `dashboard.rs` count (`src/admin/dashboard.rs:99`) generalizes to
  "one card per registered non-core kind" — the adapter supplies
  `noun`/`section`; the SQL stays `COUNT(*) ... WHERE kind=?`.
- `custom_taxs` (`src/admin/tax.rs` "Add New Filter") is tourmaster's
  `tourmaster_custom_taxonomy` option — screen moves under the adapter;
  `tax_page` checks `adapter.taxonomies()` for existence before render.
- `promote_tours` becomes `adapter.migrate()/activate()` — runs at boot
  only when the tour adapter is registered (red-team R7), same cost.

## Files to modify

- `src/admin/plugins/tourmaster/mod.rs` — `impl PluginAdapter for
  Tourmaster`; `kinds()` returns `KindDef{kind:"tour", section:"tours",
  head: HEAD_TOUR, url_base_key: Some("tour_base"), taxonomies:
  ["tour_category","tour_tag","tour_service", +custom], editor:
  Some(existing hooks)}`; `admin_screens()` returns save-option +
  new-filter; `import_hooks()` declared (wired in phase 03); `migrate()`
  = `CREATE TABLE IF NOT EXISTS tours(...)` + `promote_tours` body.
- `src/admin/plugins/mod.rs` — registry gains `screens()`,
  `migrate_all(db)`, `taxonomies()` lookups; `ADAPTERS = [&TOURMASTER]`.
- `src/admin/mod.rs` — route `/admin/plugins/{slug}/{*path}` BEFORE
  `/admin/{section}` (`src/admin/mod.rs:191`) — screen dispatch:
  `need_auth` → `adapter_for_slug` → `screen.handle(sc, sess, rest)`;
  menu loop emits one section per registered kind (`kind_def.section`);
  boot calls `registry.migrate_all(&db)` replacing the bare
  `promote_tours` call at `src/admin/mod.rs:87`; legacy
  `/admin/api/tour/save-option` shim → 307 to new path.
- `src/admin/posts.rs` — delete `tours` KindSpec entry + `HEAD_TOUR`;
  `save_tour_option` moves to the adapter module.
- `src/admin/dashboard.rs` — counts loop over registered kinds.
- `src/admin/tax.rs` — `tax_menu` via `kind_def`/adapter taxonomies;
  "Add New Filter" link renders only when the owning adapter exposes
  that screen.
- `src/db/posts.rs` — `promote_tours` + `posts__insert_into_tours*` move
  under `plugins/tourmaster/` (keep `pub(crate)` paths via adapter);
  clone path (`src/db/posts.rs:437`) calls
  `adapter.clone_post(tx, new_id, id)` — add optional hook defaulting
  to copying `post_meta` only.

## Implementation steps

1. Implement `PluginAdapter` for tourmaster; move `HEAD_TOUR`,
   `tour_submenu`, taxonomies list, `save_tour_option`, `new_filter`
   into it. Compile-driven: every moved symbol fixes its import site.
2. Router: add `/admin/plugins/{slug}/{*path}` before
   `/admin/{section}`; reserve `"plugins"` as a section name
   (reject in `spec_for_section`) — red-team R5.
3. Menu/dashboard loops emit registered-kind sections generically.
4. `migrate_all` at boot; `promote_tours` invocation moves inside
   tourmaster `migrate()`.
5. Legacy save-option shim (307) + grep vendored JS for
   `api/tour/save-option`; keep shim until vendored asset patched.
6. `clone_post` hook on the adapter for the `tours` row copy.

## Verification

```bash
cargo check && cargo test
grep -rn '"tour"' src/admin/posts.rs src/admin/mod.rs src/admin/dashboard.rs  # only via registry
grep -rn 'promote_tours\|insert_into_tours' src/db src/admin/mod.rs           # moved under plugins/
```

Manual smoke: admin boot logs `tourmaster: N rows promoted`;
`/admin/tours` lists with Durée/Región/Style/Profil columns; edit a tour
→ metabox + date picker + save-option roundtrip via
`/admin/plugins/tourmaster/save-option`; clone a tour → `tours` row
copied; taxonomy screens under `/admin/tax/tour_category` show Tours
chrome; dashboard Tour count unchanged.

## Todos

- [x] `Tourmaster` adapter impl + `ADAPTERS` registration
- [x] `/admin/plugins/{slug}/{*path}` dispatch + reserved `plugins` section
- [x] Menu/dashboard/tax generic over registry
- [x] `migrate_all` boot hook owns `promote_tours` + `tours` DDL
- [x] save-option endpoint moved + 307 shim + vendored-JS grep
- [x] `clone_post` adapter hook; tours row copy inside adapter

## Risks

- Vendored JS may hardcode `/admin/api/tour/save-option` — shim covers
  it; removal gated on asset patch (step 5).
- `custom_taxs`/`new_filter` option key stays `tourmaster_custom_taxonomy`
  but under `plugin:tourmaster:` namespace (phase 01 helper) — dual-read.
- Moving `promote_tours` out of `db/posts.rs` leaves `sql!()` paths
  referenced from a new module — `sql!` macro resolves relative to
  `src/db/sql/`; confirm macro isn't path-sensitive before moving
  (`src/db/mod.rs` sql! definition — verify at implementation).
