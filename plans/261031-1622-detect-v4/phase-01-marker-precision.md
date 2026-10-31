---
phase: 1
title: "Marker precision + missing-source warning"
status: pending
priority: P2
effort: "1.5h"
dependencies: []
---

# Phase 1: Marker precision + missing-source warning

## Goal

`BUILDER_MARKERS` only fire when the evidence is real, and a dump-only
run says so instead of silently omitting attribution.

## Findings driving this phase

aucoeur-test flags four non-builder plugins as tier-3 candidates
because `Widget_Base` appears in bundled elementor-compat shims —
`code-snippets` (bundles elementor widget for its own snippets),
`easy-table-of-contents`, `presto-player`, `simply-static`. One marker
hit anywhere in a 5k-file plugin is noise.

## Files to Create / Modify

- Modify: `src/detect.rs` — `scan_markers`: return per-marker hit
  counts instead of a `BTreeSet`, and only surface a plugin as a
  builder candidate when a marker hits **≥2 distinct files** OR in a
  top-level plugin file (`<dir>/<slug>.php`). Keep the raw marker list
  for the one-off `markers:` note so the data isn't lost.
- Modify: `src/detect.rs` — `run()`: when `www` is given but
  `wp-content/plugins` doesn't exist (or is empty), print an explicit
  line after `== active plugins ==`:
  `note: wp-content/plugins not pulled — CPT/shortcode attribution unavailable; re-pull to enrich`.
  Same for `themes` missing → themes attribution unavailable note.

## Requirements

- [ ] aucoeur re-run: code-snippets, easy-table-of-contents,
  presto-player, simply-static no longer print "tier-3 candidate"
- [ ] blueskies re-run: tevily-themer + elementor still flag correctly
  (real builders — markers in many files)
- [ ] imagetravel/dmc/imagedmc print the not-pulled note
- [ ] Marker `· registers` output unchanged

## Verification

- `cargo build` + re-run detect on all 5 sites, diff against
  `/tmp/detect-*-final.txt` (re-generate first)
- `cargo test` green
