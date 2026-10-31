---
title: "Audit 2026-10-31 remediation"
description: "Fix every verified finding from the 4-agent audit of 2026-10-31 (1 critical, 8 high, ~20 medium, ~15 low) across bridge security, import correctness, tourmaster, and quality debt."
status: completed
priority: P1
effort: 14h
branch: main
tags: [bugfix, security, audit]
blockedBy: []
blocks: []
created: 2026-10-31
---

# Audit 2026-10-31 remediation

## Overview

Four-agent audit (security / data-correctness / architecture-quality /
plugin subsystem) of the 48k-LOC codebase re-verified every finding in
source. Result: 1 critical (fail-open bridge mu-plugin), 8 high
(session-sweep type bug wiping all sessions, unsandboxed PHP bridge,
prefixed-dump silent drops, live-DB delete-before-import, 2 public
panics, bridge pidfile races, silent pull failure), ~20 medium, ~15
low/hygiene. This plan lands each fix with a regression gate, ordered
blast-radius first.

Baseline: `cargo check` clean, `cargo test` 109/109 green,
`bash scripts/check-layers.sh` → `layers OK`.

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | Close the fail-open auth boundary + unsandboxed-bridge holes | P1 |
| 2 | Fix the sessions wipe + every silent-data-loss path | P1 |
| 3 | Eliminate public-path panics reachable from stored data | P1 |
| 4 | Restore bridge lifecycle correctness (pidfiles, CSRF, orphans) | P2 |
| 5 | Pay quality debt: dead code, .flatten(), esc() divergence | P3 |

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [Audit baseline & regression scaffolds](./phase-01-start.md) | Pending |
| 2 | [Security hotfixes](./phase-02-security-hotfixes.md) | Pending |
| 3 | [Bridge lifecycle](./phase-03-bridge-lifecycle.md) | Pending |
| 4 | [Import & dump correctness](./phase-04-import-dump.md) | Pending |
| 5 | [Data correctness](./phase-05-data-correctness.md) | Pending |
| 6 | [Tourmaster, widgets & spec metabox](./phase-06-tourmaster-xss.md) | Pending |
| 7 | [Quality debt](./phase-07-quality-debt.md) | Pending |
| 8 | [N-language + regression gates](./phase-08-nlang-and-regression.md) | Pending |
## Deferred / accepted-risk (documented, not fixed by this plan)

| Finding | Decision |
|---|---|
| GET `/admin/media?mode=` writes a per-user view pref (`admin.rs:140`) | Keep — WP parity, SameSite=Lax applies, zero CSRF value |
| ~30 lines inline SQL in `src/db/posts.rs:247-283` | Keep — db/ is the allowed owner; statements are dynamic |
| `sites/*.env` `CLONE_SOURCE_OK=yes` on dmc/imagedmc | Phase 7 re-verifies with operator; flag stays only if docroots are confirmed clones |
| Session-sweep every serve() tick calling a per-site DELETE (`mod.rs:585`) | Keep cadence; phase 5 fixes the statement, optionally dedupes with `sitefx__delete_expired_sessions` |
| bwrap loopback-only egress | Phase 2 implements the *deployable* option — true netns isolation is unreachable without root (see phase file) |

## Success Criteria

- [ ] bridge.php refuses requests when RPRESS_BRIDGE_TOKEN is undefined (`$tok === ''` → 403)
- [ ] `sessions__delete_expired` uses `unixepoch()`; live sessions survive the minute sweep (test proves row count unchanged)
- [ ] Prefixed-dump import (`wp_42426_termmeta` et al.) imports all 4 previously-dropped tables; `wp_users` feeds `author_login`
- [ ] Import stages to `db_path.import` and renames on success; killed import leaves prior DB intact
- [ ] `weekday_of`/`hex_rgba` cannot panic: `month` filtered to 1..=12, non-ASCII hex rejected
- [ ] Non-object `tourmaster-tour-option` JSON bails instead of wiping settings
- [ ] pidfile written at `Command::spawn()` time; spawn-race loser only removes a pidfile it owns
- [ ] `pull()` failure surfaces as non-2xx to the editor
- [ ] Zero `.flatten()` row-error swallows left in `src/db/`; zero callers of the 4 dead `misc.rs` fns (deleted)
- [ ] `cargo test` stays green; new regression tests cover session sweep, prefixed import, tour save guard

<!-- slug: audit-1031-remediation -->
