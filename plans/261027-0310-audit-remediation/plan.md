---
title: "Audit Remediation"
description: "Fix all defects found in the 2026-10-27 full-repo audit of rpress (2 critical, 15 high, 22 medium, ~15 low)"
status: in_progress
priority: P1
effort: "3-5d"
tags: [audit, security, correctness, cache, import]
created: 2026-10-27
---

# Audit Remediation

## Overview

Full-repo audit (`reports/audit-2026-10-27.md`) found 2 critical, 15 high,
22 medium and ~15 low findings. This plan fixes all of them in dependency
order: serve-path exposure first (public-facing correctness), then injection
sinks in the ops pipeline, then importer determinism, then URL/cache
consistency, then admin/DB integrity, then perf + structural cleanup.

Baseline before starting: `cargo check` clean, 10/10 tests pass, uncommitted
diff in `src/import.rs` (partial `import_expect` fix — Phase 4 completes it;
commit it as part of that phase).

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | No unpublished content is publicly servable; URL space is canonical | P1 |
| 2 | No shell/path injection sinks in site init/deploy/pull | P1 |
| 3 | Import is deterministic: verify passes on real dumps, .sql.gz works | P1 |
| 4 | tour_base honored everywhere; no stale alias/localhost URLs | P1 |
| 5 | Cache coherence: edits/comments invalidate all affected artifacts | P2 |
| 6 | Admin deletes/saves are atomic and complete; no silent data loss | P2 |
| 7 | Perf wins (env/regex caching, spawn_blocking, indexes) + hygiene | P3 |

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [Public content exposure](./phase-01-public-content-exposure.md) | Done |
| 2 | [Shell & path injection](./phase-02-shell-and-path-injection.md) | Done |
| 3 | [Import & dump correctness](./phase-03-import-and-dump-correctness.md) | Pending |
| 4 | [tour_base & URL correctness](./phase-04-tourbase-and-url-correctness.md) | Pending |
| 5 | [Cache & regen consistency](./phase-05-cache-and-regen-consistency.md) | Pending |
| 6 | [Admin & DB integrity](./phase-06-admin-db-integrity.md) | Pending |
| 7 | [Performance, i18n & structural](./phase-07-perf-i18n-structural.md) | Pending |

| 8 | [Core/plugin seam](../261027-scoped-plugin-seam/plan.md) | Follow-up plan (not part of this audit) |

## Dependencies

- Phases 1–3 are independent of each other. 4 depends on 1 (lang/status
  plumbing in render_slug). 5 depends on 4 (tour_base literals inside the
  same invalidation paths) and phase 6 depends on 5 (both edit
  quick_update/bulk_apply — phase 5 owns handler-level invalidation,
  phase 6 owns the crud layer they call). 7 depends on 1, 4, 5 (its
  refactors must land on stable, correctly-threaded code).

## Open questions (resolved — Validation Session 1)

- [x] `sites/{dmc,blueskies,imagedmc}.env` are real clones → keep
      `CLONE_SOURCE_OK`, add provenance comments (phase 2 task).
- [x] `src/bin_mdcheck.rs` → delete (phase 7 task).
- [x] `/page/N` → 301 to archive root (phase 4, already specified).
- [x] `intake.rs` MAX(id)+1 → fix all 7 sites, no exemption (phase 6).

## Validation Log

### Session 1 — 2026-10-27

**Verification pass (Full tier):** key red-team claims re-verified in code:
`render__select_by_id` status!='trash' (draft leak confirmed),
`shell_quote` private at site_build.rs:419, CAT_TREE/ES_ROOTS real
(render_post.rs:448-578), 7 `coalesce_max` sites incl. intake, widgets.rs
voyage literal. Claims checked: ~40 | Failed: 0 | Unverified: 0.

**Interview answers:**
1. sites/{dmc,blueskies,imagedmc}.env → real clones; keep
   CLONE_SOURCE_OK + add provenance comments.
2. src/bin_mdcheck.rs → delete.
3. /page/N → 301 to archive root.
4. intake.rs MAX(id)+1 → fix all 7 sites.

**Propagation:** phase-02 (env annotation resolved), phase-04 (/page/N
already specified), phase-06 (intake sites now in scope), phase-07
(bin_mdcheck delete already listed).

### Whole-Plan Consistency Sweep (validation)
- Files reread: plan.md + all 7 phase files after propagation.
- Stale claims fixed: intake "exempt" text in phase-06 replaced by
  fix-all-7; file list updated.
- Unresolved contradictions: 0.

## Red Team Review

### Session — 2026-10-27
**Findings:** 30 total across 4 lenses (Security Adversary, Failure Mode
Analyst, Assumption Destroyer, Scope Critic). 27 accepted, 2 accepted
modified, 1 rejected.

| # | Finding | Severity | Disposition | Applied To |
|---|---------|----------|-------------|------------|
| 1 | select_id_lang not shared w/ preview; real hole is render_slug_by_id (select_by_id leaks drafts via public_root, old-slug, front fallbacks; status fabricated) | Critical | Accept | Phase 1 rewritten |
| 2 | post_meta unique index runs in DDL before dedupe can exist → migration crash on existing dbs | Critical | Accept | Phase 6: pre-DDL dedupe hook |
| 3 | site_build check() `sh -c` interpolates settings.asset_origin — unlisted injection sink | High | Accept | Phase 2 |
| 4 | shell_quote is private at site_build.rs:419 (not 388); move to site.rs | High | Accept | Phase 2 |
| 5 | tour_base plumbing: SiteCtx is serve-only; generate/sitefx have no source — per-caller sources specified | High | Accept | Phase 4 |
| 6 | invalidate_dependents can't compute archive paths (CAT_TREE/etiquette/ES_ROOTS) — extract generate.rs:224-258 shared helper | High | Accept | Phase 5 |
| 7 | invalidate via regen_posts funnel covers all 20+ callers incl. contact.rs | Medium | Accept | Phase 5 |
| 8 | M17 reparent loop dual-claimed by phases 5+6, TWO loop sites exist | High | Accept | Phase 6 sole owner, both sites |
| 9 | empty_trash DB-first leaves files serving → snapshot rels → delete → remove+retry+bust | Medium | Accept | Phase 5 |
| 10 | .sql.gz contract conflict phases 2/3 → phase 3 owns (flate2 reader); pull untouched | High | Accept | Phases 2, 3 |
| 11 | deploy rewrite non-idempotent on domain change; must rewrite staged copy only + domain settings row + localhost:* | High | Accept | Phase 4 |
| 12 | delete-vs-late-write race: invalidation after commit + post-regen sweep + documented window | High | Accept-modified | Phase 5 (documented residual, no lock) |
| 13 | MAX(id)+1 count wrong: 7 sites incl. intake.rs:284,390,408 → exempt intake, fix 4 | Medium | Accept | Phase 6 |
| 14 | phase-4 voyage grep misses format-embedded literals; widgets.rs:1025 missing | High | Accept | Phase 4 |
| 15 | generation-counter site_data cache over-engineered → lazy media + env cache first, benchmark decides | Medium | Accept | Phase 7 scoped down |
| 16 | phase 5 grab-bag: publish_due/session_touch → phase 7; reparent → phase 6 | Medium | Accept | Phases 5-7 |
| 17 | render.rs split conflicts w/ earlier phases citing its lines → last commit | High | Accept | Phase 7 |
| 18 | phase 2 crams M9/M21/M2 → M9→phase1, M21→phase6, M2→open questions | Medium | Accept | Reassigned |
| 19 | site_for returns &SiteCtx — can't 421; router-level host_known instead | Medium | Accept | Phase 1 |
| 20 | admin writes post_translations not members — members fixes degrade; add admin upsert | Medium | Accept | Phase 1 |
| 21 | csrf persist needs write-path enumeration + app-password transient Sess carve-out (HMAC) | Medium | Accept | Phase 6 |
| 22 | creds regex partial-match hazard: anchor whole value + charset validate | High | Accept | Phase 2 |
| 23 | asset_origin open-redirect primitive + redirect before is_junk_prefix + uploads 307 | Medium | Accept | Phase 1 |
| 24 | "zero sh -c" verification unsatisfiable (command -v, stream_pull legit) → "zero interpolated non-const" | Medium | Accept | Phase 2 |
| 25 | LOCAL_URL rewrite keyed only on env.local_url misses old-domain/localhost:* rows | Medium | Accept | Phase 4 |
| 26 | regen_post non-publish removes only index.html — md + alias variants survive unpublish | Medium | Accept | Phase 5 |
| 27 | phases 5+6 both edit quick_update/bulk_apply — declared dependency + ownership split | Medium | Accept | plan.md + phase 6 |
| 28 | new caches must key (site, lang) under multi-site | Medium | Accept-modified | Phase 7 (only if cache added) |
| 29 | front_id_for_lang/page_on_front reads lack status predicate | Medium | Accept | Phase 1 |
| 30 | "15 regexes" count unverified (cosmetic, direction correct) | Low | Reject | No plan change |

### Whole-Plan Consistency Sweep
- Files reread: plan.md, phase-01..07
- Deltas applied: premise correction (by_id leak), gz-contract single
  owner, dependency table (6 on 5; 7 on 1,4,5), M17 single-owner,
  funnel-wired invalidation, term_out_paths shared helper, staged-copy
  deploy rewrite, cache-scope reduction, phase 2 scope fix, open
  questions block
- Unresolved contradictions: 0

## Success Criteria

- [ ] Every finding in `reports/audit-2026-10-27.md` either fixed or
      explicitly documented as wontfix in the phase file.
- [ ] `cargo check` + `cargo test` clean; new regression tests for the
      critical/high fixes.
- [ ] `rpress site serve --smoke` on an existing site (e.g. blueskies env)
      still passes end-to-end.

<!-- slug: audit-remediation -->
