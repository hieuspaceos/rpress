---
title: "Phase 4: SQLite runtime pragmas"
status: todo
---

# Phase 4: SQLite runtime pragmas

## Overview

`conn()` only sets `busy_timeout`. WAL is on via DDL, but `synchronous`
defaults to FULL → an fsync per commit on `save_post`, `setting_put`,
`session_touch`. Runtime pragmas are free wins for save latency and read
cost on a ~100MB DB.

## Implementation Steps

1. In `src/db/mod.rs::conn()` apply via `pragma_update`/`execute_batch`:
   - `PRAGMA synchronous=NORMAL` — safe under WAL (corruption-proof; only
     last-commit loss on power fail).
   - `PRAGMA temp_store=MEMORY`.
   - `PRAGMA cache_size=-20000` (20MB page cache per conn).
   - `PRAGMA mmap_size=268435456` (256MB — read-heavy path avoids page
     copies through page cache).
   - `PRAGMA wal_autocheckpoint=4000` — amortize checkpoint I/O during
     regen bursts.
2. `busy_timeout` stays 3000ms (consider 10s only if stress shows waits;
   phase 2's IMMEDIATE makes waits serialize, not fail).

## Todo

- [ ] pragmas applied in `conn()` so every path inherits them
- [ ] rebuild; time `save_post` before/after; e2e `check` passes

## Success Criteria

- Save p50 improves measurably on 15-save burst.
- No durability regression vs documented WP-level expectations (NORMAL is
  the standard WAL recommendation).

## Risk Assessment

- `mmap_size` per conn on a 4GB VPS × N conns is virtual, not resident —
  safe; page cache still shared at OS level.
- `synchronous=OFF` already used deliberately for bulk import — untouched.
