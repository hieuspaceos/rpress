---
phase: 4
title: "Theme-as-data: CAT_TREE / etiquette / ES_ROOTS / seed fixtures"
status: pending
priority: P2
effort: "2d"
dependencies: [1]
---

# Phase 4: theme data becomes a `Data`-backend plugin

## Goal

The traveltour theme's hardcoded site topology — `CAT_TREE` category
paths, `etiquette/` tag base, `ES_ROOTS` Spanish roots, sidebar/footer
seed JSON — moves out of compiled Rust into a `Backend::Data` "theme"
adapter whose data lives in `settings`/`terms` rows seeded at
`site_init`/import time. Core keeps *reading* the values the same way
(via helpers, now DB-backed); the theme adapter is the *writer*. No PHP,
no editor engine — the Data backend exactly as the contract defines.

## Findings covered

- **T15** — `CAT_TREE` frozen map `src/render_post.rs:451-550`,
  `cat_path`/`cat_path_is_es` helpers `:550-589`, `ES_ROOTS` `:581-589`;
  consumers `generate.rs:289-296,338`, `render.rs:282-294`
  (`etiquette/{slug}`), `render_post.rs:324`, `render_archive.rs:315`.
- **T16** — `etiquette` literal at `src/render.rs:180,294`,
  `src/generate.rs:338`, `src/render_archive.rs:315`,
  `src/render_post.rs:324` — post_tag base is theme policy.
- **T17** — Seed fixtures: `src/seed/*.json` (33 files) referenced by
  `widgets.rs:265-490` `Seed` table + legacy widget payloads
  (`:490` custom_html-13 tour list) — sidebar layout is theme-supplied.
- **T18** — `tourmaster_general` option (`src/import.rs:137`,
  `src/render.rs:1244-1249`) is theme/plugin settings data imported as
  a raw option — proves the pattern: options already travel as data;
  CAT_TREE just becomes another seeded option.

## Key insights

- **Read path unchanged, write path moved.** `cat_path(slug)` becomes
  `term_path(conn, tax, slug)` reading a `plugin:theme:cat_paths` JSON
  map (or `terms` rows — see below); the ~10 call sites swap the
  function, not their logic. Deterministic lookups stay O(1) via a
  per-render memoized map (load once per `site_data`/`generate` pass —
  the existing code already materializes `CAT_TREE` per call, so a
  cached DB map is strictly better).
- **Store as one JSON option, not term rows.** `CAT_TREE` is a frozen
  snapshot mapping slug→URL-path; making each entry a `terms` row
  couples it to live term edits (red-team R9). Decision: single
  `plugin:theme:taxonomy_map` JSON value `{cat_paths:{}, tag_base,
  es_roots:[], taxonomy_bases:{}}`, seeded by the theme adapter at
  import/`site_init`, read-only at runtime. Term CRUD never touches it;
  theme update = reseed option (explicit admin action).
- **`etiquette`** → `taxonomy_bases["post_tag"]` in the same option;
  `render.rs:294`/`render_archive.rs:315` read the map.
- **Seed JSON files** ship inside the adapter module
  (`plugins/theme-traveltour/seed/**`, `include_str!`) — still
  compile-time embedded, but *owned* by the theme adapter; `site_init`
  asks registered `Data` adapters for `seeds()` instead of
  `widgets.rs`'s global `SEEDS` table. This keeps the seed mechanism
  (already data) and only changes ownership — do NOT build a runtime
  seed-file loader (YAGNI, KISS).
- The theme adapter is named `theme-traveltour` and registered
  unconditionally for now (rpress ships one theme); the registry
  pattern means a second theme is a new adapter module, not a fork.

## Files to modify

- `src/admin/plugins/theme_traveltour/` (new) — `mod.rs`
  (`Backend::Data` adapter: `slug="theme-traveltour"`, `migrate()` =
  seed `taxonomy_map` option + sidebar/widget seeds if absent,
  `seed_data()`), `seed/` (moved `sidebar-*.json`, `footer-*.json`,
  `wpcategorieswidget_9_legacy.json`, `custom_html_13_legacy.json`),
  `taxonomy_map.json` (CAT_TREE + etiquette + ES_ROOTS as data).
- `src/render_post.rs` — `CAT_TREE`/`ES_ROOTS` deleted; `cat_path`,
  `cat_path_is_es` → `theme::taxonomy_map(conn)` lookups.
- `src/render.rs`, `src/render_archive.rs`, `src/generate.rs`,
  `src/render_post.rs:324` — `etiquette` literal → map lookup.
- `src/widgets.rs` — `SEEDS`/`Seed` table + `include_str!` seed consts
  move to the adapter; `recent_tours` legacy payload stays tourmaster's
  (phase 03 already moved tour seeds).
- `src/site_init.rs`, `src/schema.rs` (`seed_widgets`/init path —
  locate the `Seed` application loop at implementation) — seeds applied
  via `adapter.seeds()` union.
- `src/import.rs` — after import, theme adapter `after_import` seeds
  `taxonomy_map` only if absent (idempotent).

## Implementation steps

1. Build `taxonomy_map.json` from `CAT_TREE`/`ES_ROOTS`/etiquette
   literals (script the extraction; verify count matches).
2. `theme::taxonomy_map(conn) -> &Map` with `OnceLock`-per-connection
   or explicit ctx param (follow `site_data` memoization precedent —
   check how `site_data` is cached per render at `src/render.rs:516`).
3. Swap all `cat_path`/`cat_path_is_es`/`etiquette` call sites.
4. Move seed JSON + `SEEDS` table; `site_init`/import seed via adapter.
5. `plugin:theme:taxonomy_map` written on migrate (only if absent);
   verify `settings` row via `sqlite3 rpress.db`.
6. Byte-parity check on generated output (same gate as phase 03).

## Verification

```bash
cargo check && cargo test
grep -rn 'CAT_TREE\|ES_ROOTS\|etiquette' src --include='*.rs' \
  | grep -v plugins/theme_traveltour   # expect: comment/doc refs only
sqlite3 site/work/rpress.db \
  "select key from settings where key like 'plugin:theme%'"   # seeded
```

Manual: category archive URLs identical (`/{lang}/actualites/...`
paths), tag pages at `etiquette/{slug}/`, ES category nesting correct,
sidebar/footer widgets render on fresh `site_init`.

## Todos

- [ ] `taxonomy_map.json` generated + count-verified
- [ ] `theme_traveltour` adapter (Data backend, migrate + seeds)
- [ ] `cat_path`/`cat_path_is_es`/`etiquette` call sites swapped
- [ ] Seed ownership moved; `site_init` + import apply via adapter
- [ ] Byte-parity verified; settings row inspected

## Risks

- Forgetting a `CAT_TREE` consumer produces silent wrong URLs — the
  grep in verification is the gate; all four consumer files listed in
  T15–T16.
- `plugin:theme:taxonomy_map` drift if admins edit `settings` — mitigated
  by treating it as seed data with no admin UI (R9); documented.
- `include_str!` paths change on seed move — compile-checked.
