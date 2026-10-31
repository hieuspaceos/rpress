---
title: "detect v3 — full-fleet sweep (23 sites, ct1/ct2/ct4)"
date: 2026-10-31
status: complete
command: "rpress detect --dump work/detect-all/<host>--<domain>/dump.sql   # dump-only, no --www"
dumps: "fresh mysqldump per site pulled 2026-10-31; reports at work/detect-all/*.report.txt"
---

# detect v3 — full-fleet sweep

All 23 WP sites found on ct1/ct2/ct4 were dumped fresh and run through
`rpress detect` **dump-only** (no `--www` — plugin source isn't pulled
for these, so attribution arms stay off; results are listing-only but
structurally complete).

## Per-site verdict

| site | host | theme → ThemeDef | adapters matched | PLUGINS_PHP | unclaimed | note |
|---|---|---|---|---|---|---|
| ananasfamilyhotel.com | ct1 | jnews-child ✗ | — | — | 5 | jnews + js_composer stack; retire candidate anyway |
| blueskiestour.com | ct1 | tevily ✓ | — | — | 2 | babe booking site; same family as vinhhyblueskies |
| chuyenchauau.com | ct1 | traveltour-child ✓ | goodlayers(PhpBridge), tourmaster | `goodlayers` | 9 | elementor + tourmaster — same shape as imagetravel |
| companytrip-image.com | ct1 | astra ✗ | head-footer-code | — | 4 | elementor site on astra — theme spec missing |
| dulichcotrachnhiem.org | ct1 | charityhub ✗ | head-footer-code | — | 5 | gdlr-* plugins w/o goodlayers-core adapter hit; `cause`/`trip` CPTs = charityhub niche |
| gitenamhien-mekong.com | ct1 | Divi ✗ | — | — | 5 | Divi + flamingo_inbound 2364 rows; retire anyway (WP 6.4 EOL) |
| hieuspace.com | ct1 | blocksy ✗ | head-footer-code | — | 0 | cleanest site — zero unclaimed |
| mekonglandservices.com | ct1 | astra ✗ | — | — | 3 | elementor-pro |
| vinhhyblueskies.com | ct1 | tevily_child ✓ | — | — | 10 | babe + tribe_* — the blueskies clone's parent |
| wafort.org | ct1 | astra ✗ | — | — | 1 | simplest site (2 plugins) — pilot candidate confirmed |
| imagetravel.vn | ct2 | traveltour-child ✓ | goodlayers(PhpBridge), head-footer-code, tourmaster | `goodlayers` | 8 | matches local imagetravel env |
| aucoeurvietnam.com | ct4 | traveltour-child ✓ | 6 adapters incl. wpdiscuz, site-reviews | `goodlayers` | 13 | fullest stack |
| brochure.aucoeurvietnam.com | ct4 | twentytwentyfour ✗ | — | — | 1 | 2 plugins — near-empty |
| brochure.imagetraveldmc.com | ct4 | twentytwentyfour ✗ | — | — | 1 | 2 plugins — near-empty |
| de.dmcmekongimage.com | ct4 | tevily_child ✓ | chaty, head-footer-code | — | 13 | babe family + flamingo |
| dmcmekongimage.com | ct4 | tevily_child ✓ | chaty, head-footer-code | — | 18 | busiest unclaimed list |
| imagetraveldmc.com | ct4 | tevily_child ✓ | chaty, head-footer-code | — | 14 | babe family |
| imaquantri2.hieuspace.com | ct4 | twentytwentyfour ✗ | — | — | 0 | internal tool, 1 plugin, retire |
| it.aucoeurvietnam.com | ct4 | traveltour-child ✓ | 6 adapters | `goodlayers` | 13 | aucoeur it-locale sibling |
| it.dmcmekongimage.com | ct4 | tevily_child ✓ | chaty, head-footer-code | — | 14 | dmc it-locale sibling |
| mekongvillages.com | ct4 | traveltour-child ✓ | goodlayers, chaty, head-footer-code, tourmaster | `goodlayers` | 13 | + nf_sub (ninja-forms submissions) 357 rows |
| reise.aucoeurvietnam.com | ct4 | traveltour-child ✓ | 6 adapters | `goodlayers` | 12 | aucoeur de-locale sibling |
| test.aucoeurvietnam.com | ct4 | traveltour-child ✓ | 6 adapters | `goodlayers` | 13 | = aucoeur-test local site |

## Correctness checks

- **Theme matching**: 13/23 match a ThemeDef — all traveltour-child,
  tevily, tevily_child. The 10 misses are exactly the themes with no
  spec written yet (astra ×3, twentytwentyfour ×3, blocksy,
  charityhub, Divi, jnews-child) — correct "consider a theme spec"
  guidance.
- **Adapter hits**: every goodlayers-core site → PhpBridge; every
  tourmaster site → Rust adapter; chaty/hfc/site-reviews/wpdiscuz hit
  only where active_plugins contains them. Zero false adapter hits.
- **PLUGINS_PHP** suggests `goodlayers` on exactly the 6 sites that
  have goodlayers-core active — no other suggestions printed.
- **Unclaimed CPTs**: 54 distinct types across the fleet; the top
  noise cluster is `itsec-*` (12 sites), `wpcode` (10), `ml-slide*`,
  `mc4wp-form`, `polylang_mo`, `language_switcher` — all storage
  types, validating the `ignore_types` plan (phase-02).
- **New CPTs surfaced**: `et_pb_layout` (Divi layouts),
  `nf_sub` (ninja-forms submissions — mekongvillages has 357),
  `flamingo_inbound` up to 2364 rows on gitenamhien, `trip`/`cause`
  (charityhub), `mailpoet_page`, `vc_grid_item`, `bricks_template`,
  `elementor-hf`, `wpmm_theme`, `amn_smtp`, `pdf_lv`/`pdfviewer`.
- **tourmaster claims only `tour`**: `tour_coupon`/`tour_service`
  still report UNCLAIMED on mekongvillages — either an
  `ignore_types` entry or a real claim decision, flagged for v4.

## What the sweep adds over the 5-site report

1. Fleet-wide `ignore_types` candidate list is now data-backed
   (frequency table above, not 1-site anecdote).
2. Theme-spec gap quantified: 7 stylesheet families need
   `themes/specs/*.toml` (or stay DEFAULT_THEME): astra,
   twentytwentyfour, blocksy, charityhub, Divi, jnews, and any
   Divi-family child.
3. babe-booking cluster (blueskiestour, vinhhyblueskies, dmc-family
   ×4, blueskies local) consistently produces `faq/service/order/
   to_book/portfolio` — one `babe` spec improvement covers 7 sites.
4. The missing-source warning (phase-01) is confirmed necessary:
   every report here ran dump-only and silently omitted
   `· registers` — exactly the confusing gap the phase warns about.

## Known limits this run

- No `--www` anywhere → no `· registers`, no attribution, no marker
  scan. To attribute, each site's `wp-content/plugins` must be pulled
  (~300MB/site — doable via the same vps.sh pipe used for dumps).
- `active_plugins` serialized-option parse worked on all 23 dumps;
  zero `option_from_dump` failures observed.
- `imaquantri2.hieuspace.com` and both `brochure.*` sites produce
  near-empty reports — correct, they're 1-2 plugin sites.
