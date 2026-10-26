---
title: "Phase 2: Complete Rename And Docs"
status: todo
priority: P1
---

# Phase 2: Complete Rename And Docs

## Overview

Finish the `rpress` rename across operator docs, API docs, scripts, logs, admin-visible footer text, cookie naming, DB defaults, and source comments. Keep only intentionally compatible names, and document each retained compatibility name explicitly.

## Requirements

- [ ] Operator-facing docs must use `rpress`, not `cms`, for binary commands.
- [ ] Scripts must reference `target/release/rpress`, not `target/release/cms`.
- [ ] Server logs and admin footer must brand as `rpress`.
- [ ] CLI defaults should use `rpress.db` unless compatibility with existing `cms.db` is explicitly documented.
- [ ] Cookie name should become `rpress_session` unless an explicit compatibility decision retains `cms_session`.
- [ ] `docs/architecture.md` must have valid Markdown fences.
- [ ] README remains the canonical quick-start overview.

## Related Code Files

| File | Action | Notes |
|---|---|---|
| `CMS-OPERATIONS.md` | Modify | Replace binary examples and deployment text. |
| `docs/api-reference.md` | Modify | Title and cookie contract. |
| `docs/architecture.md` | Modify | Close broken Mermaid fence and update session/name text if needed. |
| `docs/code-standards.md` | Modify | Align naming/session rules. |
| `scripts/diff-render.sh` | Modify | Use `target/release/rpress`. |
| `src/main.rs` | Modify | CLI default DB names and help strings. |
| `src/admin/mod.rs` | Modify | Server log and cookie lookup if renamed. |
| `src/admin/auth.rs` | Modify | Cookie set/delete names if renamed. |
| `src/admin/dashboard.rs` | Modify | Footer/version string. |
| `src/*.rs` | Modify | Source comments that still say `cms` when they mean product name. |

## Implementation Steps

1. Search `cms|pressrs|target/release/cms|cms_session|cms.db|cms listening|cms v` across `rpress`.
2. Classify each match:
   - product-name leftover → rename to `rpress`;
   - compatibility artifact → keep and document;
   - generic concept (“CMS”) → leave if semantically correct.
3. Rename binary references in `CMS-OPERATIONS.md` and `scripts/diff-render.sh`.
4. Update API docs and code if cookie name changes from `cms_session` to `rpress_session`. Migrate all cookie read/write sites in one cutover; no dual-cookie shim unless explicitly required.
5. Update CLI defaults from `cms.db` to `rpress.db`, including help strings and README/ops examples.
6. Fix the open Mermaid fence in `docs/architecture.md` immediately after the multisite diagram.
7. Re-run grep to prove only intentional `cms` matches remain.

## Todo

- [ ] Replace stale binary/script references.
- [ ] Update admin-visible branding and logs.
- [ ] Decide and apply cookie/default DB rename.
- [ ] Fix Markdown Mermaid fence.
- [ ] Add a short compatibility note for any retained `cms` names.

## Success Criteria

- `cargo run -- --help` shows `rpress` and `rpress.db` defaults if DB defaults are renamed.
- `scripts/diff-render.sh` points to `target/release/rpress`.
- `CMS-OPERATIONS.md` commands are runnable with `rpress`.
- `docs/api-reference.md` title says `rpress` and auth cookie docs match code.
- `docs/architecture.md` has paired code fences.
- `grep -R "pressrs\|target/release/cms\|cms listening\|cms v"` returns no unintended matches.

## Risk Assessment

- Renaming `cms_session` logs out existing users. Acceptable for clean cutover unless production has a stated compatibility constraint.
- Renaming default DB from `cms.db` to `rpress.db` changes local default behavior. Existing deployments using `--db` are unaffected.

## Security Considerations

Cookie rename must preserve `HttpOnly; Secure; SameSite=Lax; Path=/` attributes and session scoping behavior.
