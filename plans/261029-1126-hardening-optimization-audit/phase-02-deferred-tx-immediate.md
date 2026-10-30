---
title: "Phase 2: All write transactions BEGIN IMMEDIATE"
status: todo
---

# Phase 2: All write transactions BEGIN IMMEDIATE

## Overview

`save_post` was fixed to `conn_write()` (BEGIN IMMEDIATE) after concurrent
admin saves returned `database is locked` 400s. The same bug still exists in
~20 other write transactions: `transaction()`/`unchecked_transaction()` on
`conn()` issues BEGIN DEFERRED; a mid-transaction read→write upgrade racing
a peer writer returns `SQLITE_BUSY`/`BUSY_SNAPSHOT` **immediately** — the 3s
`busy_timeout` does not cover it.

## Evidence (ranked by exposure)

| Rank | Site | Why |
|------|------|-----|
| 1 | `db/posts.rs:1349 publish_due` | runs inside `get_sess` → **every authed admin request**; collides with any in-flight save |
| 2 | `admin/bridge/pull.rs:109` | runs after every proxied bridge request while editors save |
| 3 | `db/misc.rs:1080` body_json save | the goodlayers builder save path — same editor flow as save_post |
| 4 | `db/posts.rs:360,399,429` quick/bulk/status | bulk tx is long-lived → widest collision window |
| 5 | `intake.rs:413` | public-facing write, driven by traffic not editors |
| 6 | `tourmaster/screens.rs:77`, `widgets.rs:1005,1038`, `db/menus.rs:139,152,182,202,420`, `db/terms.rs:114`, `db/comments.rs:191,204,224` | editor-vs-editor on the same 3s timeout |
| 7 | `db/posts.rs:549,565` empty_trash/purge_old_trash | boot + scheduled duty; read-then-write worst shape |
| 8 | `db/posts.rs:1429` restore_revision, `import.rs` txs (offline CLI — acceptable) | low |

## Requirements

- [ ] Every write transaction in `src/` takes the write lock up front.
- [ ] No new code path can regress to DEFERRED by habit.

## Implementation Steps

1. `src/db/mod.rs` — move `set_transaction_behavior(TransactionBehavior::Immediate)`
   **into `conn()` itself** so every tx is IMMEDIATE; delete or alias
   `conn_write` to `conn` (keep the name for readability at call sites).
   Readers are unaffected in WAL; a tx that only reads simply commits.
2. `admin/bridge/pull.rs:108` — the raw `Connection::open_with_flags(READ_ONLY)`
   gets `busy_timeout(3000)` too (reads wp.db while php -S writes → currently
   a silently-dropped BUSY at `let _ =` in admin/mod.rs:1496).
3. Verify no `.transaction()` call site depends on DEFERRED semantics
   (none should — Immediate only changes when the lock is taken).

## Todo

- [ ] `conn()` = IMMEDIATE for all txs; `conn_write` kept as alias or removed
- [ ] `pull.rs` read-conn busy_timeout
- [ ] rebuild; retest: 15 concurrent saves 303/303 + concurrent
      quick-edit/bulk/menu-save script → 0 `database is locked`

## Success Criteria

- Stress script: parallel `save_post` + `quick_update` + menu save +
  widget reorder → no `database is locked` in any response or stderr.
- e2e `check` passes.

## Risk Assessment

- `publish_due` now takes the write lock up front per admin request —
  trivial (one tiny tx); Phase 4 also memoizes it.
- `import.rs` keeps its own pragmas — offline path, unchanged.
