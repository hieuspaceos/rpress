---
phase: 7
title: "Quality debt"
status: completed
priority: P3
effort: "1.5h"
dependencies: [1]
---

# Phase 7: Quality debt

## Goal

Delete verified dead code, consolidate the divergent `esc()` copies,
fix the remaining doc/literal drift, and align `plan.md` bookkeeping
with the commits that already landed.

## Files to Create / Modify

- Delete: `src/db/misc.rs` —
  `setting_json_value_conn` (:23), `setting_json_map` (:199),
  `setting_json_put_map` (:210), `save_body_json` (:377). All four
  verified zero callers in src/ and tests/.
- Consolidate: `esc()` at `src/plugins/tourmaster/render.rs:167`,
  `src/admin/mod.rs:1265`, `src/render/shortcode.rs:546`,
  `src/render_post.rs:742`, `src/render_archive.rs:878` → one shared
  helper. Decide the contract: single `esc()` escaping `& < > "`
  everywhere (attribute-safe superset) — the admin copy already
  escapes `"`; making render copies match is the safe direction.
- Modify: `src/render_archive.rs:47-55` — `"viet"` taxonomy
  exclusion → move to theme/plugin spec data or add a gate-allow
  comment explaining it as a curated-site internal.
- Modify: `docs/architecture.md:8-12` — drop `migrate.rs` node
  (no `src/migrate.rs` exists) or point it at the real command.
- Modify: `sites/dmc.env:8-14`, `sites/imagedmc.env:25` — re-verify
  `CLONE_SOURCE_OK=yes` really points at clone docroots, or remove
  the flag for production-adjacent hosts.
- Modify: `plans/261030-1900-audit-remediation-cleanup/plan.md` —
  `status: pending` → `completed` (commits `4028d2c`, `c219e2a`
  landed the remediation).
- Modify: `src/admin/posts.rs:218-232, 279-282` — switch posts list
  to `list::Pager`/`empty_row` shared helpers like comments/users.
- Modify: `src/plugins/goodlayers/boot.rs:441-453` — `PB_TOUR_BASE`
  thread_local default "voyage" — document or make it impossible
  to read before `boot()`.
- Modify: `src/db/posts.rs:247-283` — note: inline SQL text in db/
  is allowed by convention; either extract the lang-counts statements
  to `db/sql/` or leave with a comment. Low priority — last.

## Requirements

- [x] `cargo check` stays clean with the 4 fns deleted.
- [x] One `esc()` definition; the other four call sites use it.
      The escaping contract is uniform (`& < > "` all escaped) —
      document the superset choice in `docs/code-standards.md`.
- [x] `grep -rn '"viet"' src/` returns nothing without a spec/gate
      comment.
- [x] `architecture.md` module map matches `src/` reality.
- [x] `check-layers.sh` still prints `layers OK`.

## Verification

- `cargo check && cargo test` green.
- `grep -rn "fn esc(" src/` → exactly 1 definition.
- `ak plan validate plans/261030-1900-audit-remediation-cleanup`
  passes with `status: completed`.
