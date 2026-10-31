---
title: "Phase 6: Tests & CI"
status: todo
priority: P2
---

# Phase 6: Tests & CI

## Overview

Audit §5: the only workflow (`release.yml`, 45 lines) builds on tag — no
`cargo test`, no clippy/fmt, no `check-layers.sh` despite the gate's own
comment saying "run in CI or pre-commit". Admin HTTP surface (~13k LOC, ~25%
of codebase) has zero route-level tests.

## Requirements

- [x] New `.github/workflows/ci.yml`: on push/PR — `cargo test --locked`, `cargo clippy -- -D warnings` (only if currently clean; else `-W warnings` + issue), `cargo fmt --check` (only if repo is fmt-clean — verify first), `scripts/check-layers.sh`
- [x] Minimal admin axum smoke test (1 test): boot app on a fixture site → POST /admin/login → authed GET /admin/posts → POST a trivial CRUD (e.g. trash+restore a seeded post) — proves session + csrf + spawn_blocking wiring end-to-end
- [x] `e2e-bridge.sh` stays manual (needs PHP) — document in workflow comment, do not add to CI
- [x] `tests/cattest.rs` (29 LOC, one `cat_seq` test): fold into an existing test file or delete — scratch remnant

## Implementation Steps

1. Check `cargo fmt --check` and `clippy` cleanliness NOW; record baseline. If dirty, gate on `test` + `check-layers` only, note clippy/fmt as follow-up.
2. Write `ci.yml`: stable toolchain, `cargo test --locked`, `bash scripts/check-layers.sh`, cache `~/.cargo` + target.
3. Smoke test: look for an existing test helper that spins `serve`/axum Router on a temp db (check `tests/` for a fixture-builder pattern — reuse it; if none, seed minimal site db via `db::` schema fns used by existing integration tests).
4. Delete/merge `tests/cattest.rs`.

## Todo

- [x] clippy/fmt baseline recorded
- [x] ci.yml added, runs green on this branch
- [x] Admin login→CRUD smoke test added and passing
- [x] cattest.rs folded or removed

## Success Criteria

- CI run on the fix branch passes all enabled gates.
- Smoke test fails if session/csrf/spawn_blocking wiring breaks (deliberately break once to confirm).

## Risk

- Fixture construction for a servable site may be heavy — if no reusable fixture exists, seed via the same `import`/schema helpers tests already use; don't build new infra.
