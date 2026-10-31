---
phase: 4
title: "Theme-CPT fixture + 5-site regression"
status: completed
priority: P2
effort: "1h"
dependencies: [1, 2, 3]
---

# Phase 4: Theme-CPT fixture + 5-site regression

## Goal

Prove `theme:<dir>` attribution works (untested — no production site
registers a CPT in its theme), then re-run all 5 sites and commit a
golden diff.

## Files to Create / Modify

- Create: `tests/fixtures/detect_theme_cpt/` — a minimal
  `wp-content/themes/fake-theme/functions.php` containing
  `register_post_type('theme_faq', ...)` plus a `dump.sql` fixture
  (reuse `tests/fixtures/prefixed_dump.sql` shape, add one
  `theme_faq` post row).
- Modify: `tests/…` — a detect test (or a `#[test]` in detect.rs if
  the file follows unit-test style) asserting
  `theme_faq → UNCLAIMED (registered by theme:fake-theme)`.
- Modify: `plans/reports/detect-5site-audit-2026-10-31.md` — append a
  "post-v4 re-run" section with the final outputs per site, or create
  `plans/reports/detect-v4-regression-2026-10-31.md`.

## Requirements

- [ ] Fixture proves `theme:<dir>` attribution end-to-end
- [ ] All 5 sites re-run; differences from the audit report are
  exactly the intended ones (markers quieter, notes present,
  ignored-types gone from suggestions)
- [ ] The diff is committed as evidence (report file or fixture
  outputs under `plans/reports/`)

## Verification

- `cargo test` green incl. new fixture test
- `check-layers.sh` OK
