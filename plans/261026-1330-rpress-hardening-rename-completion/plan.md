---
title: "rpress Hardening and Rename Completion"
description: "Complete the rpress rename and close the security, contract, operational, and test gaps found in the post-rename assessment."
status: pending
priority: P1
effort: 16h
issue: null
branch: main
tags: [security, refactor, docs, auth, backend, tech-debt]
blockedBy: []
blocks: []
created: 2026-10-26
---

# rpress Hardening and Rename Completion

## Overview

Complete the post-rename cleanup for `rpress` and harden the production contract before the project is treated as release-ready. The plan covers every finding from the reassessment: upload MIME safety, logout CSRF, remaining `cms`/old-brand references, broken docs rendering, session contract drift, SQL ownership drift, upload-size mismatch, non-atomic generation writes, and missing integration coverage.

## Decisions

- The project name is `rpress` everywhere user/operator-facing.
- Clean cutover: rename `cms_session` to `rpress_session` and CLI default DB names from `cms.db` to `rpress.db` unless a phase discovers an explicit compatibility requirement in code/docs.
- Persistent sessions are treated as intentional because current code already stores and rehydrates sessions; documentation and operational guidance must match that runtime behavior.
- The architecture rule stays strict: SQL belongs in `src/db/`; admin/render helpers should call database helper functions instead of embedding SQL.
- Upload safety wins over WordPress compatibility for scriptable legacy files; block or force-download scriptable uploads rather than serving them inline.

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | Remove security risks in upload serving and logout | P1 |
| 2 | Complete the `rpress` rename across docs, scripts, logs, UI, cookie, and DB defaults | P1 |
| 3 | Reconcile runtime contracts for sessions and SQL ownership | P1 |
| 4 | Make cache/generation operations safe and operator-visible limits accurate | P2 |
| 5 | Add integration coverage and verification gates for the touched behavior | P1 |

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [Security Hotfixes](./phase-01-start.md) | Pending |
| 2 | [Complete Rename And Docs](./phase-02-complete-rename-and-docs.md) | Pending |
| 3 | [Integration Tests And Verification](./phase-03-integration-tests-and-verification.md) | Pending |
| 4 | [Reconcile Runtime Contracts](./phase-04-reconcile-runtime-contracts.md) | Pending |
| 5 | [Harden Cache And Upload Operations](./phase-05-harden-cache-and-upload-operations.md) | Pending |

## Success Criteria

- [ ] `/wp-content/uploads/*` never serves SVG/XML/HTML/JS/CSS inline from local legacy uploads.
- [ ] `POST /api/logout` requires the session CSRF token and the admin logout form supplies it.
- [ ] `grep -R "pressrs\|target/release/cms\|cms listening\|cms v"` has no unintended matches.
- [ ] `docs/architecture.md` renders valid Markdown with closed Mermaid fences.
- [ ] Session persistence docs match the code path that stores and rehydrates `sessions` rows.
- [ ] Direct SQL in `src/admin/*` is moved into `src/db/*` helpers or explicitly justified where read-only utility code remains.
- [ ] Upload-size UI, Axum body limit, and DB/helper validation agree on one documented limit.
- [ ] Bulk generation writes use tmp+rename or are documented and tested as offline-only.
- [ ] New integration tests cover upload MIME policy, logout CSRF, session persistence contract, atomic generation behavior, and rename-visible CLI/docs contracts.
- [ ] `cargo test`, `cargo check`, and available formatting/lint checks pass; if `rustfmt` is unavailable, the exact toolchain blocker is documented.

## Verification Commands

```bash
$HOME/.cargo/bin/cargo test
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo run -- --help
$HOME/.cargo/bin/cargo fmt --check  # if rustfmt is installed
```

## Known Current Evidence

- `cargo test` passed: 18 tests, 0 failures.
- `cargo check` passed.
- CLI help shows `Usage: rpress <COMMAND>`.
- `cargo fmt --check` could not run because `cargo-fmt` is not installed for `stable-aarch64-apple-darwin`.
- Latest pushed commit before this plan: `dc6ed7f chore: rename project to rpress`.

## Dependencies

- No external service dependency.
- Rust toolchain must include `cargo`; formatting verification additionally requires `rustup component add rustfmt` or equivalent installed component.
- Some test fixtures may need temporary SQLite DBs and temporary `www/`/`cache/` directories.

## Handoff

Implement with:

```bash
/ak:cook /Users/hieuspace/Desktop/CODE/pet/rpress/plans/261026-1330-rpress-hardening-rename-completion/plan.md
```

<!-- slug: rpress-hardening-rename-completion -->
