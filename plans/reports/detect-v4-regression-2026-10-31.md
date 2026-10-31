---
title: "detect v4 — post-change regression (23 sites)"
date: 2026-10-31
status: complete
command: "rpress detect --dump work/detect-all/<site>/dump.sql   # dump-only, new binary"
outputs: "work/detect-v4-reports/*.txt (23 files, 2992 lines)"
---

# detect v4 regression — what changed in the reports

Same 23 dumps, new detect. Differences vs the v3 sweep are exactly the
intended ones:

| metric | v3 sweep | v4 | note |
|---|---|---|---|
| `tier-3 candidate` lines | ~10 (incl. code-snippets, ez-toc, simply-static) | **0** dump-only; elementor + tevily-themer still flag on blueskies (--www) | bundled-shim noise gone; real builders survive via ≥3-file / main-file / CPT+marker evidence |
| `UNCLAIMED` lines | ~150 | **45** | storage CPTs now `ignored by spec` (120 lines across fleet) |
| `ignored by spec` | n/a | **120** | wp-internals.toml + tourmaster's tour_coupon/tour_service |
| "not pulled" note | absent | **23/23** sites | dump-only runs now say attribution is unavailable instead of implying zero registrations |
| "suggested next steps" | free-text list | **routing board** (THEME/BRIDGE/RENDER/CLAIM/IMPORT/PULL) | each finding names the rpress file that owns the fix |

## CLAIM rows that remain (real work)

- dmc-family ×4 + vinhhyblueskies + blueskiestour: `faq`, `service`,
  `order`, `portfolio`, `gva__template`, `tribe_*` — all attributed to
  ba-book-everything / tevily-themer / the-events-calendar on
  pulled-source sites (blueskies/aucoeur confirm)
- `brtpmj_review`, `wpgp_youtube_gallery`… — small per-site leftovers,
  route to spec claim or wp-internals ignore as decided per site

## New behavior

- `theme:<dir>` attribution tested by unit fixture (no production site
  registers a CPT in its theme)
- `rendered_shortcodes()` on adapters drives "rpress renders natively"
  rows; adding a render arm auto-updates detect — no mirror
- `kind_for_post_type` in `import.rs` is the single claim authority —
  detect and import cannot disagree
- `ignore_types` spec field: claim∩ignore asserted disjoint at init;
  core-claimed kinds (ml-slider/ml-slide) excluded by a unit test

## Verification

- `cargo test --workspace` — all green incl. 3 new detect tests
- `check-layers.sh` — layers OK
- `cargo check` — clean
