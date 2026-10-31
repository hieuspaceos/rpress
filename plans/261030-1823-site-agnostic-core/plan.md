---
title: "Site-Agnostic Core — Audit Remediation + N-Language + Theme/Plugin Contracts"
description: "Fix all 2026-10-30 audit findings and remove the three extensibility blockers so any site can run on any theme and any plugin spec."
status: pending
priority: P1
effort: multi-day
tags: [refactor, security, performance, i18n, theming]
created: 2026-10-30
---

# Site-Agnostic Core

## Overview

Remediates every finding from the 2026-10-30 three-axis audit (security /
performance / extensibility) and completes the strategic goal: **any site,
any theme, any plugin — no edits to `src/` core required**.

Source audit artifacts: findings cited inline per phase with file:line.
User decisions locked: retire `post_translations` entirely (single N-lang
model on `post_translation_members`); hybrid theme contract (TOML spec +
Rust fallback adapter); full audit scope.

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | Zero CRITICAL/HIGH security findings; MEDIUM M1 (bridge cred leak) + M2 (menu stored XSS) closed | P0 |
| 2 | No blocking I/O on the async executor anywhere in the serve path; N+1s in cold render batched | P1 |
| 3 | Language #3+ editable/pairable in admin; `post_translations` table dropped | P0 |
| 4 | New theme = TOML spec + templates dir (Rust adapter only as escape hatch); zero traveltour literals in core | P0 |
| 5 | TOML spec expresses metabox editor + settings screen + render hooks for "dumb CPT" coverage | P1 |
| 6 | `schema_migrations` versioned runner + a check-layers gate enforcing site-agnosticism | P1 |

## Phases

| # | Phase | Status | Owns (exclusive files) |
|---|-------|--------|------------------------|
| 1 | [Security Hardening](./phase-01-security-hardening.md) | Pending | admin/bridge/*, admin/menus.rs, render/menu_url.rs, render/site_data.rs, intake.rs |
| 2 | [Serve-Path Performance](./phase-02-perf-serve-path.md) | Pending | admin/serve.rs, admin/mod.rs, admin/admin.rs, admin/auth.rs, db/mod.rs |
| 3 | [Render Batching](./phase-03-perf-render-batching.md) | Pending | generate.rs, render/site_data.rs (img batch only), render/menu_url.rs (hoist only) |
| 4 | [N-Language Admin](./phase-04-n-language-admin.md) | Pending | admin/posts.rs, admin/render lang UI, db/posts.rs, lang.rs, import.rs, render.rs, plugins/tourmaster/render.rs, schema.rs (drop table) |
| 5 | [Theme Contract](./phase-05-theme-contract.md) | Pending | plugins/mod.rs (ThemeDef), plugins/theme_*, render.rs (literals), admin/mod.rs (tax_label, chrome), admin/widgets.rs, admin/rankmath.rs, generate.rs |
| 6 | [Plugin Spec Extension](./phase-06-plugin-spec-extension.md) | Pending | plugins/spec.rs, plugins/specs/*.toml, build.rs, docs |
| 7 | [Schema Migrations + Agnostic Gate](./phase-07-schema-migrations-gate.md) | Pending | schema.rs, db/sql/schema__*, scripts/check-layers.sh |

## Execution Order

Phases are **serial** — file overlaps (serve.rs, mod.rs, site_data.rs,
render.rs, schema.rs) make parallel ownership unsafe. Phase 6 is the only
one with disjoint ownership and MAY overlap phase 5 in a worktree.

Order matters: 1→2→3 clears correctness/latency debt before the big
refactors; 4 before 5 because both touch render.rs literals; the
`schema_migrations` runner (first half of phase 7) lands **between**
phases 3 and 4 so the `post_translations` drop rides it; the agnostic
gate (second half of phase 7) runs last to validate the final tree.

Effective cook order: 1 → 2 → 3 → 7a (migrations runner) → 4 → 5 →
(6 may overlap 5 in a worktree) → 7b (agnostic gate + docs).

## Success Criteria

- [ ] `cargo clippy --all-targets` 0 warnings; `cargo test` all green; `./scripts/check-layers.sh` passes including the new agnostic gate.
- [ ] `./scripts/visual_smoke.sh` passes on all reference sites (byte-parity fixtures still match — no output drift unless intentionally recaptured).
- [ ] A `fr,es,de` site renders + admin-edits German posts, pairs them, and links hreflang correctly — proven by a throwaway seeded test.
- [ ] A second theme onboarded via TOML+templates without touching `src/render*.rs` or `src/admin/*.rs`.
- [ ] `grep -rn "traveltour\|duree\|duracion\|region-es\|estilo\|perfil" src/ | grep -v plugins/theme_traveltour | grep -v spec` returns only gate-annotated exceptions.
- [ ] `post_translations` absent from `schema__ddl.sql`; existing DBs migrated via `schema_migrations`.

<!-- slug: site-agnostic-core -->
