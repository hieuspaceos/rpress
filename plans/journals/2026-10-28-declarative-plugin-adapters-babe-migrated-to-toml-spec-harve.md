---
title: "Declarative plugin adapters — babe migrated to TOML spec, harvest-at-import"
date: 2026-10-28
summary: "Migrated babe to declarative plugins/specs/*.toml adapters; harvested CF7 HTML at import so generate works against dead origins; fixed kind_defs iterating ADAPTERS instead of adapters()."
---

# Declarative plugin adapters — babe migrated to TOML spec, harvest-at-import

# Declarative Plugin Adapters — Harvest-at-Import

**Date**: 2026-10-28
**Severity**: Medium
**Component**: rpress plugin system (specs, harvest, generate, import)
**Status**: Resolved

## What Happened

Completed plans/261028-1459-declarative-plugin-adapters (all 3 phases): plugin kinds are now declared in TOML specs compiled into the binary, and CF7 form HTML is harvested at import time so `generate` no longer needs a live WP origin.

- **Phase 01 — spec + SpecAdapter**: `build.rs` embeds `plugins/specs/*.toml` → `OUT_DIR/specs.rs`. `src/admin/plugins/spec.rs` parses them into `LazyLock` statics (SPECS/KIND_DEFS/TAX_TABLES/TITLES/PREFIXES/LEGACY_KEYS/CLAIMS/HARVEST/HOOKS). `SpecAdapter` indexes into the statics — no `Box::leak`, per repo rule. `adapters()` merges `ADAPTERS ∪ specs`; REGISTRY/kind_defs/migrate_all/tax_section all iterate `adapters()` so spec kinds get identical asserts. `toml` crate added.
- **Phase 02 — harvest-at-import**: `src/harvest.rs` gained `harvest_page` (moved from generate.rs) plus CF7 action rewrite to `/form/enquiry` with hidden `_page`/`tour` injection. `harvested_pages(post_id, html, fetched_at)` is created at import; publish-status spec-harvest posts are collected during the posts pass and fetched post-commit — the tx never spans HTTP. `INSERT OR REPLACE` runs on a fresh conn2 tx. Origin path = `{lang_prefix}/{origin_base|url_base}/{slug}/`; stored HTML keeps `{{ site_url }}` placeholders resolved by `rewrite_origin` at render. Generate checks `harvested_pages` → `harvest_posts` → render. Forms seed `slug='enquiry'` with `additional='harvest-seed'` provenance (delete+reseed only our rows).
- **Phase 03 — babe migration**: `plugins/specs/babe.toml` written; deleted `babe.rs`, `babe/render.rs`, `templates/single_book.html`, `elements/babe_enquiry.html`.

## The Brutal Truth

The one real bug was ours: `kind_defs()` iterated `ADAPTERS`, not `adapters()`, so booking pages rendered at site root until verification caught it. The spec adapter was silently half-registered — everything compiled, tests passed, output was wrong. Classic "looks done, isn't."

## Key Decisions

- **Port-once harvest**: WP origin only needs to be live during import; harvested HTML lives in `harvested_pages`, generate works on a dead origin. Rejected fetch-on-generate — it couples every build to origin uptime.
- **CF7 mapping (a)**: re-point form actions to the `/form/enquiry` intake endpoint rather than leaving dead forms in the output.
- **`claim_at(i)` slot table**: `claim_type` is a bare fn, so claims are dispatched through a fixed table capped at 8 specs — panics loudly on overflow instead of silently dropping.
- **`kind_defs()` fix**: switched to `adapters()` after the root-render bug surfaced.

## Verification Evidence

- `cargo check` clean; `cargo test` all pass (54 lib + suites).
- imagedmc import: `harvested_pages` = 56/56 publish to_book, 0 failed; dmc: 72/72.
- imagedmc generate: 233 pages; `/booking/1-vietnam-10-jours-7-nuits/` → 200 with tevily/wpcf7 markup, CF7 actions → `localhost:8413/form/enquiry`. dmc generate: 393 pages, `booking/{en,es}` 38+34.
- aucoeur-test: no `harvested_pages`, e2e check ok, front 200.
- code-reviewer subagent: APPROVE; fixed findings (stale `dead_code` allows, dup comment, seed refresh semantics, CF7 field parity: your-name/your-email/your-phone*/your-subject/your-message).

## Residuals

- Turnstile/quiz CF7 fields pass through unvalidated — parity with the old `babe_enquiry` template, not a regression.
- URL-encoded origin refs remain inside share-link query params only.
- `docs/architecture.md` still describes the pre-seam `EditorPlugin`/`PLUGINS` — pre-existing staleness.
- 31 draft to_book rows in imagedmc have empty slugs (pre-existing, inert).
- Harvest is serial curl, ~9s/page on this origin → minutes-scale imports for many posts.

## Lessons Learned

- A registry merge (`adapters()`) is only real if *every* consumer uses it. One straggler iterating the old constant = silent half-registration. Grep every `ADAPTERS` call site when introducing the merged view.
- Harvest-at-import is the right seam: network fragility belongs at the boundary, not in every render.

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
