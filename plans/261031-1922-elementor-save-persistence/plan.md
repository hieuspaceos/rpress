---
title: "elementor-save-persistence"
description: "Make Elementor edits survive bridge respawns; fix seeded_post truthfulness and an escaped-URL collapse miss."
status: done
priority: P1
effort: "1d"
tags: [bridge, elementor, audit-fix]
created: 2026-10-31
---

# elementor-save-persistence

## Overview

Audit follow-up on the Elementor bridge (`src/admin/bridge/*`,
`src/plugins/elementor.rs`, `src/admin/plugin_dispatch.rs`):

1. **Elementor saves are silently dropped.** `WRITE_SPECS = &[]` in
   `elementor.rs` — the `_` in `_elementor_data` is a SQL LIKE wildcard,
   which `WriteSpec::validate` forbids, so the spike shipped no specs.
   Pull-back copies nothing; every edit dies with the disposable `wp.db`.
2. **`seeded_post` lies on failed/absent seeds.** `php_cgi.rs:754` /
   `playground.rs:274` set `seeded_post: post_id` regardless of
   `project_post`'s `Ok(false)`; `reseed_post` returns `Ok(true)` when
   `pull::locate` finds no `wp.db` (flag never cleared). Both poison the
   skip-resseed check → post switch silently broken for that bridge.
3. **Escaped mount-collapse misses one case.** `plugin_dispatch.rs:363`
   checks `after.starts_with("\\\\/wp-")` (literal `\\/`) but the
   rewritten escaped form is `\/admin\/plugins\/elementor\/wp-admin\/…`
   (single `\/`) → doubled-mount URLs leak un-collapsed in JSON bodies.

Root-cause fix for (1): drop SQL `LIKE` from pull matching — compare
`instr(key, 'prefix') = 1` (prefix match) and `key = 'k'`
(exact, BodyJson). `_` then carries zero wildcard semantics, validate's
`_` ban becomes unnecessary, and `_elementor*` is a plain literal prefix.

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | Pull matching uses literal substr/equality — `_` allowed in keys | P1 |
| 2 | Elementor write_specs: `_elementor` meta pull + `post_content`/`post_status` → `posts.body_html`/`status` via new `PullTable::PostFields` (never `body_json` — `render_nodes` can't consume Elementor JSON) | P1 |
| 3 | `seeded_post` only set on real seeds (spawn + reseed paths) | P1 |
| 4 | Escaped-collapse matches `\/wp-` (single backslash) | P2 |

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [Pull engine: literal matching](./phase-01-literal-pull-match.md) | Done |
| 2 | [Elementor write_specs + posts fields](./phase-02-elementor-write-specs.md) | Done |
| 3 | [seeded_post truthfulness + collapse fix](./phase-03-seeded-truth-collapse.md) | Done |
| 4 | [End-to-end persistence smoke](./phase-04-persistence-smoke.md) | Done |

## Success Criteria

- [x] Edit inside Elementor editor → respawn bridge (or idle-TTL kill) →
      reopened editor shows the saved layout.
- [x] `cargo test --lib` passes incl. new pull-match + collapse tests.
- [x] `seeded_post` never set when `project_post`/`reseed_post` reported
      no seed.
- [x] No doubled `/admin/plugins/elementor/admin/plugins/elementor/` in
      proxied JSON bodies (escaped form included).

<!-- slug: elementor-save-persistence -->
