---
title: "Audit fixes 261031 — security, concurrency, hygiene"
description: "Fix all actionable findings from docs/audit-261031.md: 2 stored XSS, 2FA backup-code entropy, systemd root, admin spawn_blocking, swallowed DB errors, cache perf, missing CI gates, hygiene."
status: pending
priority: P1
effort: 2d
branch: main
tags: [bugfix, security, refactor, infra, critical]
blockedBy: []
blocks: []
created: 2026-10-31
---

# Audit fixes 261031

## Overview

Execute the prioritized findings from `docs/audit-261031.md` (verified against
codebase 2026-10-31). Seven sequential phases: stored XSS → auth hardening →
admin concurrency → error discipline → perf/cache → tests+CI → hygiene/refactor.

Phases are sequential, not parallel: `src/admin/posts.rs` is touched by phases
1, 3 and 4; `admin/auth.rs` by phases 2 and 3. Order = audit priority order.

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | Eliminate both confirmed stored-XSS paths + sweep same-class sites | P1 |
| 2 | Harden auth: 128-bit backup codes, systemd `User=`, remove pwd trim | P1 |
| 3 | Uniform `spawn_blocking` for all admin DB/argon2 calls + minute tick | P1 |
| 4 | Stop swallowing DB-mutation errors (`let _ =` → `?`/500/log) | P2 |
| 5 | Cache/perf: Arc site_data, hoist `kind_has_posts` conn, single-flight render-miss | P2 |
| 6 | CI runs `cargo test` + `check-layers.sh`; minimal admin axum test | P2 |
| 7 | Hygiene: empty files, misplaced html, cattest, stale allows, README | P3 |

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [Stored XSS fixes](./phase-01-stored-xss.md) | Pending |
| 2 | [Auth hardening](./phase-02-auth-hardening.md) | Pending |
| 3 | [Admin concurrency (spawn_blocking)](./phase-03-admin-concurrency.md) | Pending |
| 4 | [Error discipline](./phase-04-error-discipline.md) | Pending |
| 5 | [Perf & cache](./phase-05-perf-cache.md) | Pending |
| 6 | [Tests & CI](./phase-06-tests-ci.md) | Pending |
| 7 | [Hygiene & refactor](./phase-07-hygiene-refactor.md) | Pending |

## Source of truth

`docs/audit-261031.md` — all file:line refs verified in codebase 2026-10-31.

## Out of scope (audit findings deferred)

- `import.rs` widget/gdlr builder extraction (§4, ~1.3k LOC) — follow-up refactor.
- `site_deploy.rs` SQL → `src/db/sql/` move — bundled into phase 7 only if trivial.
- Admin-surface full test coverage — phase 6 adds a minimal smoke path only.

## Success Criteria

- [ ] `cargo test` green; `cargo clippy` no new warnings
- [ ] `scripts/check-layers.sh` passes incl. new admin-blocking rule
- [ ] XSS PoC `' onerror='alert(1)` in comment name renders inert
- [ ] SEO keyword payload `<img onerror>` renders inert in /admin/posts cell
- [ ] New backup codes are ≥128-bit; old hashes invalidated on enable/regen
- [ ] No `crud::`/`udb::`/`db::` call outside `spawn_blocking` in `src/admin/`
- [ ] `install.sh` unit has `User=` or `DynamicUser=`
