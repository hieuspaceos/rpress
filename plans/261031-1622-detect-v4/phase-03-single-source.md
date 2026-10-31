---
phase: 3
title: "Single source for claimed() and rendered shortcodes"
status: pending
priority: P2
effort: "2h"
dependencies: []
---

# Phase 3: Single source for `claimed()` and rendered shortcodes

## Goal

Two hand-kept mirrors become derived data: `detect::claimed()` stops
duplicating `import::kind_of` logic, and `RENDERED_SHORTCODES` stops
mirroring `expand_shortcodes` match arms by hand.

## Findings driving this phase

- `claimed()` builds its set by calling every adapter's `claim_type`
  over the observed types — correct today, but it *re-derives* what
  import already knows; if `kind_of` gains a hard-coded arm
  (it already special-cases attachment/nav_menu_item/revision/
  wpcf7_contact_form/custom_css/to_book), detect drifts silently.
- `RENDERED_SHORTCODES` is a const copy of the arms in
  `render/shortcode.rs`. A new arm lands, the mirror lies.

## Files to Create / Modify

- Modify: `src/import.rs` — split `kind_of` into a public-ish
  `fn imported_post_type(name) -> ImportKind` enum
  (`Adapter | BuiltinStorage | Skipped`) that both the posts pass and
  detect consult. `attachment`/`nav_menu_item`/`revision`/
  `wpcf7_contact_form`/`custom_css`/`to_book` become enum arms, not
  if-chains.
- Modify: `src/detect.rs` — `claimed()` consumes that enum: report
  shows `claimed` vs `built-in (storage/media/forms) — no adapter
  needed` as distinct rows.
- Modify: `src/render/shortcode.rs` — replace the match-arm list with
  a `const RENDERED_TAGS: &[&str]` the match dispatches through, OR
  expose the tag list via the adapter that owns each tag
  (`PluginAdapter::rendered_shortcodes()` default `&[]`) — prefer the
  adapter route if goodlayers/site-reviews handlers already live
  adapter-side; detect then unions `adapter.rendered_shortcodes()`
  instead of mirroring. Decide at implementation time: fewer call
  sites wins.
- Modify: `src/detect.rs` — drop `RENDERED_SHORTCODES` const, use the
  derived set.

## Requirements

- [ ] One definition of "import claims this type" — detect and import
  cannot disagree
- [ ] Adding a new render shortcode arm updates detect automatically
  (or fails compile — either is fine, silent drift is not)
- [ ] `wpcf7_contact_form`/`custom_css`/`to_book` still print claimed
- [ ] Output rows distinguish `claimed (adapter X)` from
  `built-in` where applicable

## Verification

- `cargo test` green; grep shows single authority for both
- detect output unchanged except the built-in wording where intended
