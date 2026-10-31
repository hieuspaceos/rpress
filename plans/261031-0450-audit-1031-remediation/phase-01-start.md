---
phase: 1
title: "Audit baseline & regression scaffolds"
status: pending
priority: P1
effort: "1h"
dependencies: []
---

# Phase 1: Audit baseline & regression scaffolds

## Goal

Freeze the audit evidence into the repo and stand up the two regression
harnesses every later phase hangs tests on, so each fix lands with a
proving artifact.

## Context Links

- Audit report: the four-agent audit of 2026-10-31 (conversation
  deliverable); distill it into `plans/reports/audit-2026-10-31.md` so
  the evidence survives outside chat.
- Prior audits for diff-baseline:
  `plans/261027-0310-audit-remediation/reports/audit-2026-10-27.md`.

## Files to Create / Modify

- Create: `plans/reports/audit-2026-10-31.md` — full finding list with
  file:line + verification status (verified vs inferred) per finding.
- Modify: `scripts/check-layers.sh` — add `#!/usr/bin/env bash`
  shebang guarantee: the script uses `< <(…)` process substitution and
  currently dies under `sh` (`syntax error near unexpected token '<'`).
  Either exec bash when `$BASH_VERSION` is unset or add the shebang +
  document invocation as `bash scripts/check-layers.sh`.
- Create: `tests/audit_remediation.rs` — shared home for the phase
  regression tests (session sweep, tour-option guard, metabox marker).
- Create: `tests/fixtures/prefixed_dump.sql` — minimal `wp_42426_*`
  prefixed dump covering `posts`, `postmeta`, `termmeta`,
  `rank_math_redirections`, `glsr_ratings`, `glsr_assigned_posts`,
  `users` (1–2 rows each) — used by phase 4.

## Tasks & Steps

1. Write `plans/reports/audit-2026-10-31.md` from the agent findings;
   mark each item verified/inferred.
2. Fix `check-layers.sh` to be sh-safe or bash-pinned; confirm
   `sh scripts/check-layers.sh` either runs correctly or fails with a
   clear "requires bash" message instead of a syntax error.
3. Scaffold `tests/audit_remediation.rs` with the tempdir+DB helpers
   the later phases need (init schema via `schema.rs`, insert session
   row, run one statement).
4. Build the prefixed dump fixture.

## Verification

- `sh scripts/check-layers.sh` exits 0 or fails loudly with a bash
  requirement (not a syntax error); `bash scripts/check-layers.sh`
  still prints `layers OK`.
- `cargo test --test audit_remediation` compiles and runs 0–1 tests.
- Fixture parses under `sqldump::dump_rows` (compile-time check only).

## Risk

Low — no production code touched except the script shebang.
