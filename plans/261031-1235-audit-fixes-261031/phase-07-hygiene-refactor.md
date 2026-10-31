---
title: "Phase 7: Hygiene & small refactors"
status: todo
priority: P3
---

# Phase 7: Hygiene & small refactors

## Overview

Audit §4–§5 hygiene items + the low-risk refactors. Excludes the big
`import.rs` split (deferred to its own plan).

## Requirements

- [x] Delete 3 empty tracked files at repo root: `aucoeur-test`, `blueskies`, `imagetravel` (0 bytes confirmed) — check git history/usage references first (`grep -rn "aucoeur-test\|blueskies\|imagetravel" --exclude-dir=target` and README/sites/*.env)
- [x] Move `bridge-architecture.html` (12KB) → `docs/`
- [x] `tests/cattest.rs` — handled in phase 6 (list here for tracking only)
- [x] `sites/*.env` (5 files): audit for credentials/SSH data; if clean → keep; if secrets → rotate + gitignore + scrub. Report finding, do not commit secrets either way
- [x] `plans/` (~12MB/192 files): leave for now or archive old plan dirs via `ak plan archive`/`cleanup` — user decision not needed if we just stop committing new agent artifacts going forward; minimal action: no change beyond this plan
- [x] README:397 scripts list — add `diff-render.sh`, `scope-css.py`
- [x] Remove stale `#[allow(dead_code)]` on `db/mod.rs:77` `conn_write` (function IS called by `db/posts.rs:1049` save_post — attribute only, never delete the fn)
- [x] Review remaining `#[allow(dead_code)]` fields: `plugins/mod.rs` (~10), `tourmaster/schema.rs` ×3, `bridge/mod.rs` ×3, `bridge/pull.rs` ×1 — delete if truly dead, keep+comment if reserved for wire/API parity; also `#[allow(unused_imports)]` at `posts.rs:1`, `render.rs:3`, `generate.rs:3`, `auth.rs:1`, `util.rs:8` — remove allow + fix import
- [x] `admin/comments.rs:21` rename local `esc_attr` → `esc_attr_lines`
- [x] `posts.rs` CSRF convention split (`check_csrf` vs hand-rolled `ct_eq` at 596-598): unify on `check_csrf`
- [x] `main.rs` flag defaults duplicated across 3 `#[arg]` blocks → clap flatten struct or single `Cli` defaults const — pick boring option
- [x] H6 SQL literals outside `src/db/sql/`: `site_deploy.rs:296,306-310`, `backup.rs:37` `VACUUM INTO` — move to `src/db/sql/` only if the module has a fitting pattern (read it first); `VACUUM INTO` with `?1` param is arguably fine inline — document decision
- [x] `render.rs:990` `traveltour` fallback literal — leave (intentional `// gate-allow: legacy fallback`); no action, tracking only

## Todo

- [x] Empty files deleted (after ref check)
- [x] bridge-architecture.html → docs/
- [x] sites/*.env audit reported
- [x] README scripts list updated
- [x] `conn_write` allow removed
- [x] dead_code/unused_imports review done (list kept-vs-removed)
- [x] `esc_attr` rename in comments.rs
- [x] CSRF convention unified in posts.rs
- [x] clap defaults deduped
- [x] H6 decision documented

## Success Criteria

- `cargo test` + `scripts/check-layers.sh` green.
- No `#[allow(dead_code)]`/`#[allow(unused_imports)]` remaining unless commented with reason.
- Report: sites/*.env contents verdict (clean/rotated).

## Risk

- `sites/*.env` may contain real secrets → rotation affects running sites; surface before touching.
- sites/*.env empty-file names may be referenced by sites configs — grep first.
