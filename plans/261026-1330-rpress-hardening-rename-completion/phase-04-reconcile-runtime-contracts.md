---
title: "Phase 4: Reconcile Runtime Contracts"
status: todo
priority: P1
---

# Phase 4: Reconcile Runtime Contracts

## Overview

Make documentation and code agree on two core runtime contracts: session persistence and SQL ownership. The current docs say sessions are in-memory and SQL lives only in `src/db/`, while code persists sessions and has admin SQL. This phase removes that drift.

## Requirements

- [ ] Session behavior must be documented exactly as implemented.
- [ ] If persistent sessions remain, DB backup/security docs must say the DB contains live auth tokens until expiry/revocation.
- [ ] SQL ownership rule must either be enforced by moving queries into `src/db/*` helpers or rewritten to match the real convention.
- [ ] Mutating SQL from admin modules must move behind `src/db/*` helpers.
- [ ] Read-only direct SQL may remain only if explicitly justified in docs and kept minimal.

## Related Code Files

| File | Action | Notes |
|---|---|---|
| `src/schema.rs` | Review/possibly modify | Persistent `sessions` table and migrations. |
| `src/admin/mod.rs` | Review/modify | Session rehydration and direct SQL sites. |
| `src/admin/admin.rs` | Modify | Direct settings SQL during generate/cache paths. |
| `src/admin/dashboard.rs` | Modify | Direct dashboard count SQL candidates. |
| `src/admin/gdlr.rs` | Modify | Direct media SQL candidate. |
| `src/admin/rankmath.rs` | Modify | Direct SEO SQL candidate. |
| `src/db/*` | Modify/create helpers | Add focused helpers preserving current behavior. |
| `docs/architecture.md` | Modify | Session and SQL ownership contract. |
| `docs/code-standards.md` | Modify | Remove stale “no sessions table” rule or update it. |
| `CMS-OPERATIONS.md` | Modify | Backup/revocation operational note. |

## Implementation Steps

1. Confirm current session implementation:
   - table creation in `schema.rs`;
   - login insert/update path;
   - `get_sess` rehydration;
   - logout/session deletion path;
   - expiry cleanup behavior.
2. Decide contract: keep persistent sessions. Update docs to say restart does not necessarily log users out until expiry unless sessions are purged.
3. Add operator guidance: protect DB backups because they contain session tokens/hashes until expiry.
4. Inventory direct SQL outside `src/db/` with grep for `query_row`, `prepare`, `execute`, and `sql!` in `src/admin`/render modules.
5. Move mutating or reusable queries into narrow `src/db/*` helpers with names that describe behavior, not SQL shape.
6. Update admin callers to use helpers.
7. Update architecture docs to describe any intentionally allowed direct SQL exceptions.
8. Add/adjust tests from phase 3 for session persistence and DB helpers.

## Todo

- [ ] Document persistent session behavior and backup sensitivity.
- [ ] Remove or update stale “in-memory sessions only” statements.
- [ ] Inventory direct SQL outside `src/db/`.
- [ ] Move mutating direct SQL into `src/db/*` helpers.
- [ ] Update docs to match the final SQL ownership rule.

## Success Criteria

- Docs no longer claim restart logs everyone out if code rehydrates sessions.
- `docs/code-standards.md` no longer forbids a session table that exists.
- Mutating direct SQL in admin modules is removed or justified with evidence.
- `cargo check` and `cargo test` pass.

## Risk Assessment

- Moving SQL can subtly change transactions or error handling. Keep helpers thin and preserve return types/behavior.
- Session contract changes affect threat model. Document rather than silently changing behavior unless tests reveal a bug.

## Security Considerations

Persistent sessions increase DB backup sensitivity. Docs must instruct operators to treat DB copies as authentication material until tokens expire or are purged.
