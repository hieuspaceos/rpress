---
title: "Phase 5: Harden Cache And Upload Operations"
status: todo
priority: P2
---

# Phase 5: Harden Cache And Upload Operations

## Overview

Align operational behavior for uploads and generated files: make upload-size limits truthful and make all live-served generation writes atomic, or document and enforce an offline-only generation workflow. Prefer atomic writes because the server already uses tmp+rename in lazy write-back and `regen_post`.

## Requirements

- [ ] Admin UI upload limit, Axum request body limit, and DB/media helper limit must agree.
- [ ] The agreed upload limit must be documented in ops/API docs.
- [ ] Bulk generation must not expose partial files to live readers.
- [ ] Existing tmp+rename behavior should be reused rather than duplicated.
- [ ] Cache bust must keep `www/` persistent uploads untouched.

## Related Code Files

| File | Action | Notes |
|---|---|---|
| `src/admin/mod.rs` | Modify | Axum body limit and cache write helper references. |
| `src/admin/admin.rs` | Modify | Upload page text currently says 64 MB; cache bust behavior. |
| `src/db/misc.rs` | Modify | `save_upload` limit currently rejects >25 MB. |
| `src/generate.rs` | Modify | Bulk generation currently has direct `fs::write` paths. |
| `src/sitefx.rs` | Review/modify | Utility-file emits may write live-served files. |
| `CMS-OPERATIONS.md` | Modify | Document upload limit and generation/cache safety. |
| `docs/api-reference.md` | Modify | Upload route limit/status contract. |
| `tests/` | Modify/create | Test atomic write helper and limit consistency where practical. |

## Implementation Steps

1. Define one upload limit constant in one owner module, ideally bytes plus display string.
2. Wire the constant into:
   - Axum body limit;
   - `save_upload` validation;
   - admin UI text;
   - docs/API route description.
3. Identify all generation writes that target files served from `cache/`/`out`.
4. Extract or reuse an `atomic_write(path, bytes/string)` helper that writes sibling temp file then renames.
5. Replace direct `fs::write` for generated HTML/MD/XML/TXT/JSON files that can be read by visitors or nginx.
6. Preserve existing permissions/error behavior as much as possible.
7. Add tests from phase 3 for the helper and for limit consistency.

## Todo

- [ ] Introduce shared upload limit constant.
- [ ] Update request body limit, DB helper validation, UI text, and docs.
- [ ] Inventory direct generation `fs::write` calls.
- [ ] Replace live-served writes with tmp+rename helper.
- [ ] Verify cache bust does not touch `www/`.

## Success Criteria

- UI no longer says 64 MB unless backend accepts 64 MB.
- API docs state the same upload limit as code.
- Bulk generation cannot leave a partially written live-served file under normal interruption/error paths.
- `cargo test` includes coverage for upload limit and atomic write helper.
- `cargo check` passes.

## Risk Assessment

- Atomic rename must use a temp file on the same filesystem as the target. Use sibling temp paths, not system temp dirs.
- Concurrent generation may collide on temp names. Include PID/thread/random suffix or use a robust unique name.

## Security Considerations

Upload limits protect memory/disk pressure. Atomic writes protect visitors from malformed partial files and reduce cache corruption during deploy/generate operations.
