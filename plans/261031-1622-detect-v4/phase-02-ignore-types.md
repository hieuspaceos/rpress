---
phase: 2
title: "`ignore_types` spec field + storage-CPT decluttering"
status: pending
priority: P2
effort: "1.5h"
dependencies: []
---

# Phase 2: `ignore_types` spec field + storage-CPT decluttering

## Goal

Plugin-internal storage CPTs (dashboards, popup themes, form
submissions, video blocks) stop appearing as `UNCLAIMED` suggestions —
they're types import deliberately skips, not spec work to do.

## Findings driving this phase

Every site lists noise suggestions:
`itsec-dash-card`/`itsec-dashboard` (iThemes dashboard cards),
`popup`/`popup_theme`/`aoc_popup` (Popup Maker internals),
`pp_video_block` (Presto Player block records),
`flamingo_contact`/`flamingo_inbound` (Flamingo mail archive),
`mc4wp-form`, `wpcode`, `language_switcher`, `polylang_mo`,
`rank_math_schema`, `rm_content_editor`. Suggesting
`plugins/specs/X.toml: claim_types += [...]` for these is wrong —
import must *skip* them, not claim them.

## Files to Create / Modify

- Modify: `src/plugins/spec.rs` — accept `ignore_types = [...]` in
  spec TOMLs (sibling of `claim_types`); expose via `ImportHooks` or a
  parallel `ignore_types` accessor.
- Modify: `src/plugins/mod.rs` — registry aggregation:
  `ignored_type(name)` checking all adapters (same probe style as
  `claim_type`).

- Modify: `src/detect.rs` — `claimed()`/`run()` third bucket:
  `{t} ({n}) → ignored by spec (storage CPT — skipped at import)`.
  Excluded from the unclaimed suggestion list.
- Modify: `plugins/specs/wp-internals.toml` (new, adapter-less spec is
  the cleanest owner for cross-plugin storage) OR per-plugin specs —
  decide at implementation; `ignore_types` candidates from the
  23-site sweep (report `detect-23site-sweep`):
  itsec-dash-card, itsec-dashboard (12 sites), wpcode (10),
  ml-slide, ml-slider (6), mc4wp-form (6), polylang_mo (5),
  language_switcher (5), rm_content_editor (6), rank_math_schema (3),
  flamingo_contact, flamingo_inbound (up to 2364 rows),
  popup, popup_theme, popupbuilder, aoc_popup,
  pp_video_block, et_pb_layout, nf_sub (357 rows), mailpoet_page,
  vc_grid_item, bricks_template, elementor-hf, wpmm_theme, amn_smtp,
  pdf_lv, pdfviewer, wpgp_youtube_gallery, wpdiscuz_form,
  3d-flip-book, dflip, wpforms, custom-post-template, carousels,
  sp_wp_carousel, location_weather, uwpqsf, web-story, tahefobu_footer,
  elementskit_content.
- Decide: `tour_coupon`, `tour_service` (tourmaster registers them;
  adapter only claims `tour`) — claim or ignore, one line in the
  tourmaster adapter/spec.

## Requirements

- [ ] `ignore_types` parses from TOML and resolves at runtime
- [ ] dmc/imagedmc/imagetravel suggestions drop itsec_*, popup*,
  pp_video_block, flamingo_*, mc4wp-form, wpcode, polylang_mo,
  language_switcher, rank_math_schema, rm_content_editor
- [ ] A type in BOTH `claim_types` and `ignore_types` — claim wins
  (document choice; panic on ambiguity is acceptable, same style as
  adapter collisions elsewhere)

## Verification

- `cargo test` green incl. a spec-parse test for `ignore_types`
- detect re-run: suggestion list on dmc shrinks to real work only
