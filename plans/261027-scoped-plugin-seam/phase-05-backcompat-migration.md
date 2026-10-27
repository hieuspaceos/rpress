---
phase: 5
title: "Back-compat & migration for existing sites"
status: pending
priority: P1
effort: "1d"
dependencies: [2, 3, 4]
---

# Phase 5: existing sites keep working — migration phase

## Goal

Every site created before this plan — `kind='tour'` rows, `tours` table
rows, bare `tourmaster_*`/`gdlr-core-*` settings keys, `TOUR_BASE=` env,
`sidebar-tour-*` widget rows — continues to work with zero operator
action. One idempotent migration folds legacy state into the new
namespaces. Verification is a checklist run against a real production
rpress.db, not just unit tests.

## Findings covered

- **T19** — Existing DBs: `posts.kind='tour'` rows must resolve through
  the registry; `tours` rows already match the moved DDL
  (`CREATE TABLE IF NOT EXISTS` in core `src/schema.rs:26-36` stays —
  harmless; adapter `migrate()` is additive not exclusive).
- **T20** — Legacy settings keys (`tourmaster_general`,
  `gdlr-core-*`, `tourmaster_custom_taxonomy`, `polylang`) written by
  old imports (`src/import.rs:128-156`) must dual-read until migrated.
- **T21** — `TOUR_BASE` env + `--tour-base` CLI flag are deployed
  contract (`src/site_deploy.rs:434-447` emits it into remote env);
  rename/break is a deploy outage.
- **T22** — Widget rows referencing `sidebar-tour`/`tour_category`
  types and `tourmaster-*` option keys in `post_meta`
  (`src/widgets.rs:990` unserializes `tourmaster-tour-option`) — meta
  keys stay exactly as WP wrote them (the adapter *is* tourmaster's
  format; no rename — document this explicitly so nobody "fixes" it).

## Key insights

- **Meta keys are plugin wire format, not core schema.** `tourmaster-*`
  post_meta keys stay verbatim; the `plugin:tourmaster:` namespace
  applies only to `settings`/`options` rows. This boundary is the
  single most important line in the phase — crossing it corrupts
  tourmaster's own save/load roundtrip (`save.rs` reads the literal
  keys back).
- **Migration = one `migrate_settings` pass** at admin boot (inside
  `registry.migrate_all`): `UPDATE settings SET key='plugin:'||slug||':'||key
  WHERE key IN (<legacy list>) AND NOT EXISTS(new key)`. Legacy list is
  declared per adapter (`legacy_option_keys()`). Dual-read helpers from
  phase 01 cover the window before migration runs.
- **`TOUR_BASE` stays forever as the env key name** — it maps into
  `KindBases` for the tour adapter; renaming it gains nothing and breaks
  `site_deploy` remote env emission. Document, don't rename.
- **Rollback**: every phase before this is additive (new code paths +
  shims). Rollback = revert merge; DB changes are all `IF NOT EXISTS`/
  namespace copies — never deletes. The one destructive act,
  `gdlr_pb.rs` deletion, is phase 07 and lands *after* the bridge is
  proven per-site.

## Files to modify

- `src/admin/plugins/mod.rs` — `legacy_option_keys()` on adapter trait;
  `migrate_all` runs settings-key fold.
- `src/admin/plugins/tourmaster/mod.rs` — legacy keys list
  (`tourmaster_general`, `tourmaster_custom_taxonomy`); `url_base`
  reads `TOUR_BASE`/settings.
- `src/admin/plugins/theme_traveltour/` — legacy keys: none new
  (taxonomy_map is new); seeds only-if-absent (already idempotent).
- `src/db/settings.rs` (or wherever helpers landed) — `migrate_keys`
  helper + test.
- `docs/` (or README section) — deployment note: env keys unchanged;
  plugin settings namespaced; what an operator sees.

## Implementation steps

1. `legacy_option_keys()` trait method + `migrate_keys` boot fold.
2. Dual-read verification matrix: old DB + new binary →
   tourmaster_general read works pre-migration (fallback) and post
   (namespaced).
3. Fixture test: copy a pre-seam rpress.db → boot new binary → assert
   `plugin:tourmaster:tourmaster_general` exists and legacy row still
   intact-or-migrated per chosen policy (choose: keep legacy row —
   reads prefer namespaced; zero data loss).
4. `TOUR_BASE`/`--tour-base` regression: `site_init` +
   `site deploy --dry-run` emit unchanged env.
5. Full checklist on real DB (below).

## Verification

```bash
cargo check && cargo test
sqlite3 site/work/rpress.db \
  "select key from settings where key like '%tourmaster%' or key like 'gdlr%'"
# expect: namespaced copies present post-boot
grep -rn 'TOUR_BASE' src/site_deploy.rs src/site.rs   # unchanged emission
```

Production-DB checklist (run on a copy):
- [ ] `kind='tour'` posts listed + editable in `/admin/tours`
- [ ] `tours` row data renders on public pages (build + spot-check)
- [ ] `tourmaster_general` search-page URL honored
- [ ] `gdlr-core-*` skins/templates load in page builder
- [ ] `sidebar-tour` widgets render; recent-tours cards populated
- [ ] custom taxonomies (`tourmaster_custom_taxonomy`) editable
- [ ] `TOUR_BASE` env still emitted by deploy
- [ ] Second boot: migration is a no-op (idempotent)

## Todos

- [ ] `legacy_option_keys` + boot migration implemented
- [ ] Dual-read verified on real DB copy
- [ ] Migration idempotency test (boot twice, diff settings)
- [ ] TOUR_BASE deploy-path regression verified
- [ ] Operator-facing doc written

## Risks

- Migration racing two admins on shared DB: `UPDATE ... WHERE NOT
  EXISTS` is atomic per statement; WAL mode serializes writers — safe.
- A plugin key already namespaced by a *partial* earlier run: migration
  is prefix-aware (`key NOT LIKE 'plugin:%'`) — add that predicate.
- If a site never opens admin (deploy-only pipeline), migration never
  runs — dual-read fallback covers reads indefinitely; migration also
  hooked into `site build` boot for belt-and-braces (verify `site_build`
  path calls `migrate_all` — add at `src/site_build.rs:514` region).
