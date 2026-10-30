---
title: "rpress hardening + optimization (post-audit)"
description: "Fix audit findings: bridge lifecycle leaks, deferred-tx SQLITE_BUSY across write paths, slug index, SQLite pragmas, per-site settings cache, tail perf and data pruning."
status: pending
priority: P1
effort: 2-3d
tags: [backend, database, infra, tech-debt]
created: 2026-10-29
---

# rpress hardening + optimization (post-audit)

## Overview

Follow-up plan from the 2026-10-29 architecture audit of rpress (measured on
aucoeur-test: 1240 pages, 104MB DB, 19-site / 1-VPS deployment target).
Phases are ordered by blast radius: ops-critical leaks first, then the
remaining SQLITE_BUSY exposure, then measurable perf wins, then lifecycle
pruning.

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | No orphaned PHP bridges / port leaks under systemd | P1 |
| 2 | Zero `database is locked` under concurrent editors | P1 |
| 3 | Cut /admin dashboard (~390ms) and miss-render (~140ms) latency | P2 |
| 4 | Bound DB growth (revisions, sessions, submissions) | P3 |

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [PHP bridge lifecycle hardening](./phase-01-php-bridge-lifecycle.md) | Pending |
| 2 | [All write transactions BEGIN IMMEDIATE](./phase-02-deferred-tx-immediate.md) | Pending |
| 3 | [posts.slug index](./phase-03-slug-index.md) | Pending |
| 4 | [SQLite runtime pragmas](./phase-04-pragmas.md) | Pending |
| 5 | [Per-site cached settings + publish_due throttle](./phase-05-settings-cache.md) | Pending |
| 6 | [Tail perf, lifecycle prune, spawn_blocking](./phase-06-tail-perf-and-prune.md) | Pending |

## Success Criteria

- [ ] `kill -TERM` leaves zero `php -S`/playground orphans; port collisions gone
- [ ] Parallel save/quick-edit/bulk/menu stress: no `database is locked`
- [ ] `EXPLAIN` uses `idx_posts_slug_lang`; dashboard p50 measurably lower
- [ ] e2e `check` passes on aucoeur-test after every phase

## Key evidence

- Audit findings: deferred-tx sites ranked in `phase-02` (publish_due is
  highest-frequency — runs per authed request).
- `save_post` already fixed via `conn_write` (BEGIN IMMEDIATE); phase 2
  generalizes to `conn()`.
- PHP bridge: spawn-on-demand + 600s TTL already; missing SIGTERM reap +
  identity probing.

<!-- slug: hardening-optimization-audit -->
