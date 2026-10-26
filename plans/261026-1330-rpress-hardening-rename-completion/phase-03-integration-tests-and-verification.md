---
title: "Phase 3: Integration Tests And Verification Harness"
status: todo
priority: P1
---

# Phase 3: Integration Tests And Verification Harness

## Overview

Add durable tests before and during the remaining refactors so security and contract fixes do not regress. This phase may be implemented before phases 4-5 or alongside them, but final completion happens only after all new tests pass against the final code.

## Requirements

- [ ] Tests must be behavior-oriented, not source-text assertions.
- [ ] Tests must use isolated temporary SQLite DBs, cache dirs, and www dirs.
- [ ] Tests must cover upload MIME safety, logout CSRF, session persistence, generation atomicity or offline-only contract, and CLI/help rename contracts.
- [ ] Tests must follow existing Rust test conventions and run under `cargo test`.

## Related Code Files

| File | Action | Notes |
|---|---|---|
| `tests/` | Create/modify | Add integration tests or helper-level tests. |
| `src/admin/mod.rs` | Modify if needed | Expose `pub(crate)` helpers for testable MIME/cache decisions. |
| `src/generate.rs` | Modify if needed | Expose atomic write helper or use existing helper consistently. |
| `src/schema.rs` | Use in tests | Temporary DB setup. |
| `Cargo.toml` | Modify only if necessary | Add dev-dependency only if stdlib/tempdir alternatives are insufficient. |

## Test Matrix

| Behavior | Test type | Pass condition |
|---|---|---|
| Scriptable uploads | Helper or route integration | SVG/XML/JS/CSS are not served inline. |
| Safe uploads | Helper or route integration | PNG/JPEG/PDF keep safe content type behavior. |
| Logout CSRF | Route/handler integration | Missing/invalid token returns forbidden; valid token logs out. |
| Session persistence | DB/helper integration | Docs-chosen contract matches code: persisted session rehydrates or restart logs out. |
| Rename contract | CLI/doc smoke | `rpress --help` and operator docs do not point to `cms` binary. |
| Generation atomicity | File behavior test | Writer uses tmp+rename or documented offline-only path is enforced. |

## Implementation Steps

1. Inspect current test patterns in `tests/php_serialize.rs` and `tests/cattest.rs`.
2. Prefer extracting pure helpers from production code over building brittle full HTTP server tests.
3. Add temp path setup using standard library or a minimal dev-dependency if needed.
4. Add tests adjacent to the behavior they protect:
   - upload MIME decision helper;
   - logout CSRF validation path;
   - session DB rehydration behavior;
   - atomic write helper used by generation;
   - CLI help smoke via `std::process::Command` only if stable enough in Cargo integration tests.
5. Run `cargo test` after each test group is made green.

## Todo

- [ ] Add isolated test utilities for temp DB/cache/www paths.
- [ ] Add upload MIME policy tests.
- [ ] Add logout CSRF tests.
- [ ] Add session persistence contract tests.
- [ ] Add generation write safety tests.
- [ ] Add rename/CLI smoke where practical.

## Success Criteria

- `cargo test` passes with the new tests included.
- Tests fail on the pre-fix behavior for upload MIME and logout CSRF.
- Tests do not require network, a real WordPress dump, or persistent local state.
- No tests assert incidental source text or comments.

## Risk Assessment

- Full Axum route tests may require more setup than value. Prefer extracted deterministic helpers when they cover the same consumer-visible behavior.
- Do not add broad dependencies just for temp directories unless necessary.

## Security Considerations

Security tests must prove browser-executable content is not served inline and mutating logout requires CSRF.
