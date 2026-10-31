---
phase: 1
title: "Pull engine: literal prefix matching"
status: done
priority: P1
effort: "2h"
dependencies: []
---

# Phase 1: Pull engine — literal prefix matching

## Goal

Bridge pull-back matches `meta_key`/`option_name` by literal prefix
(`instr(col, ?1) = 1`) instead of SQL `LIKE`, so `_` no longer acts as
a wildcard and `_elementor*` keys can be whitelisted.

## Call-site census (verified — all current spec literals)

- `src/plugins/goodlayers/mod.rs:36` `"gdlr-core-%"` (PostMeta),
  :40 `"gdlr-core-%"` (Settings), :43 `"gdlr-core-page-builder"` (BodyJson, no `%` — already exact).
- `src/plugins/theme_traveltour/mod.rs:142` `"traveltour%"`.
- `pull.rs` tests :227-233 use `"gdlr-core-%"` and reject cases.
- `key_map` literals: `plugin:goodlayers:`-style prefixes (grep
  `key_map:` when editing).

## Files to Modify

- `src/db/sql/bridge_pull__select_postmeta_like.sql` →
  `SELECT post_id, meta_key, meta_value FROM wp_postmeta WHERE instr(meta_key, ?1) = 1`
- `src/db/sql/bridge_pull__select_options_like.sql` →
  `SELECT option_name, option_value FROM wp_options WHERE instr(option_name, ?1) = 1`
  (filenames keep `_like` — churn over accuracy; note in doc comment.)
- `src/admin/bridge/pull.rs`:
  - `key_prefix` becomes a **literal prefix**; update doc comment.
  - `validate()`: PostMeta/Settings — require non-empty, forbid `%`
    (under `instr` a literal `%` would silently match nothing or
    wrong-key — config bug); `_` legal. BodyJson — exact `=` match
    stays; forbid `%`, allow `_` (enables `_elementor_data` — though
    Phase 2 uses PostFields instead). `key_map` unchanged.
  - Call sites above: strip trailing `%` from the four literals.
- `pull.rs` tests: update `"gdlr-core-%"` → `"gdlr-core"`, rejection
  cases to `%`-containing prefixes; add a `_`-containing prefix case
  (`_elementor`) proving it matches and a same-prefix-different-key
  case proving literal semantics (`_elementor` must NOT match
  `Xelementor`).

## Tasks & Steps

1. Update the two SQL files to `instr(...)=1`.
2. Rewrite `validate()` + doc comments.
3. Strip `%` at the four call sites.
4. Update/extend tests per above.

## Verification

- `cargo test --lib bridge::pull` green.
- Sanity vs. data: `sqlite3 work/blueskies/rpress.db` — the old LIKE
  `"gdlr-core-%"` and new `instr` on `gdlr-core` match the same row set
  (`_` never appeared mid-prefix in existing specs, so equivalence holds).
