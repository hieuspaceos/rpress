---
title: "detect v4 — accuracy + coverage gaps"
description: "Close the remaining gaps found by the 5-site detect audit: marker false-positives, missing-source warning, ignore-types convention, kind_of drift, adapter-declared shortcodes"
status: completed
priority: P2
effort: "6h (actual ~3h)"
branch: main
tags: [detect, audit, quality]
blockedBy: []
blocks: []
created: 2026-10-31 16:22
---
# detect v4 — accuracy + coverage gaps

## Overview

The 5-site verification (`plans/reports/detect-5site-audit-2026-10-31.md`)
proved detection is accurate wherever plugin/theme source is pulled,
and surfaced a shortlist of real gaps. This plan fixes the ones that
are code gaps (not pull-coverage gaps — those need `pull.sh` work, out
of scope here).

Baseline: `cargo check` clean · `cargo test` green · `check-layers.sh` →
`layers OK` · detect verified on blueskies/aucoeur-test (source-pulled)
and imagetravel/dmc/imagedmc (dump-only).

## Goals

- [ ] Builder markers stop false-positiving on plugins that merely
  bundle elementor-compat files (aucoeur: code-snippets, ez-toc,
  presto-player, simply-static all flagged)
- [ ] Dump-only sites print an explicit "attribution unavailable —
  plugins dir not pulled" line instead of silently omitting
  `· registers`
- [ ] Storage CPTs (itsec-dash-card, popup_theme, pp_video_block,
  flamingo_inbound…) can be marked "ignore" via an adapter spec field
  so suggestions stop proposing specs for types import skips anyway
- [ ] `claimed()` no longer mirrors `kind_of` by hand — one source of
  truth
- [ ] `theme:<dir>` CPT attribution verified on a real or fixture theme
- [ ] Adapter-owned shortcode tags (the `RENDERED_SHORTCODES` constant)
  derived from adapter metadata, not a hand-kept mirror

## Phases

| # | Phase | Effort | Deps |
|---|---|---|---|
| 1 | Marker precision + missing-source warning | 1.5h | — |
| 2 | `ignore_types` spec field + storage-CPT decluttering | 1.5h | — |
| 3 | `claimed()` ↔ `kind_of` unification + `RENDERED_SHORTCODES` from adapter metadata | 2h | — |
| 4 | Theme-CPT fixture + full 5-site regression re-run | 1h | 1–3 |

## Non-goals

- Pulling `wp-content/plugins` for imagetravel/dmc/imagedmc (a
  `pull.sh` concern, not detect)
- Resolving `register_post_type(get_option(...))` / DB-held slugs —
  no site observed using one
- Machine-readable output (`--json`) — worth a separate plan if needed

## Verification

- `cargo check` / `cargo test` green, `check-layers.sh` OK
- Re-run detect on all 5 sites: aucoeur marker list shrinks to real
  builder plugins only; dump-only sites print the attribution
  unavailable line; itsec/flamingo/pp_* types stop appearing in
  suggested specs; report file diff committed beside this plan
