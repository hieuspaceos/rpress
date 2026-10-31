---
title: "Phase 4: Error discipline — stop swallowing DB errors"
status: todo
priority: P2
---

# Phase 4: Error discipline — stop swallowing DB errors

## Overview

Audit §2.2 (C2): `let _ =` on DB mutations makes the UI report success on
failure — including a security-logic hazard (silent TOTP no-op). 78 `let _ =`
in `src/admin/`; only DB-mutation sites need fixing, file-cleanup ones are
acceptable.

## Confirmed sites

- `admin/posts.rs:587` — trash fail still regens + redirects success (also deletes cache)
- `admin/users.rs:1092-1096` — delete-user fail still deletes sessions/app-passes
- `admin/users.rs:1173-1175, 1242, 1313, 1334, 1345, 1354` — role change,
  TOTP enable/disable, app-pass revoke: UI says success on DB fail
- `admin/mod.rs:620` — `sessions__delete_expired` fail → tokens live in backups

## Requirements

- [x] Every `let _ =` on a DB mutation becomes `?` + 500 response, or at minimum `eprintln!` + visible failure — `?` preferred, handler returns `Result<Response>` or use existing error pattern (check how other handlers signal 500)
- [x] TOTP enable/disable/revoke paths MUST return an error to UI on failure — silent no-op is a security-logic hazard
- [x] `mod.rs:620` sweep failure: log + surface in admin status or eprintln (no HTTP context — background sweep)
- [x] Non-DB `let _ =` (file cleanup, best-effort cache removal): leave, but grep-review the list once to confirm classification

## Implementation Steps

1. `grep -rn "let _ =" src/admin/` → full list; classify DB-mutation vs cleanup.
2. For each DB-mutation site: propagate with `?` where handler returns `Result`; where it returns `Response`, match existing error-response convention (check `check_csrf` error path — it returns `Err(r)` with `r` a Response; mirror that).
3. `posts.rs:587`: `crud::set_post_status(...)?`-equivalent; on Err return 500 page instead of regen+redirect.
4. `users.rs` sites: same pattern; TOTP paths get explicit error flash/redirect.
5. `mod.rs:620`: log `eprintln!` on sweep failure.

## Todo

- [x] `let _ =` inventory + classification
- [x] posts.rs:587 fixed
- [x] users.rs 7 sites fixed
- [x] mod.rs:620 logging
- [x] Cleanup-class sites reviewed, list documented

## Success Criteria

- Simulated DB failure (e.g. read-only db file) on trash → 500/error, not success redirect + cache loss.
- `cargo test` green.

## Risk

- Interacts with phase 3 (posts.rs/users.rs touched by both) — do after phase 3 merges, or same PR.
