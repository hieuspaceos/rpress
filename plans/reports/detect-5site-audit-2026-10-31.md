---
title: "detect v3 — 5-site verification"
date: 2026-10-31
status: complete
command: "rpress detect --dump work/<site>/dump.sql --www work/<site>/www"
binary: e921f49 + shortcode-filter patch (uncommitted at report time)
---

# detect v3 — 5-site verification

Ran the new detect (dynamic CPT resolution, theme attribution,
registered-shortcode filtering) against every pulled site. This file
records what it got right, what it got wrong, and the concrete gaps
that remain.

## Pull coverage (the limit that shapes every result)

| site | VPS | `wp-content/plugins` | `wp-content/themes` | attribution possible |
|---|---|---|---|---|
| blueskies | ct1 | partial (4 plugin dirs) | yes | partial |
| aucoeur-test | ct4 | full | yes | full |
| imagetravel | ct2 | **no** (uploads + mu-plugins only) | no | none |
| dmc | ct4 | **no** (uploads only) | no | none |
| imagedmc | ct4 | **no** (uploads only) | no | none |

Sites without pulled source still get theme/plugin/CPT listing from the
dump alone — the `· registers` and `(registered by X)` notes simply go
absent instead of being wrong.

## Correct per site

| site | theme | adapter hits | attributed CPTs | notes |
|---|---|---|---|---|
| blueskies | tevily_child → ThemeDef ✓ | chaty, head-footer-code | elementor → e-landing-page/e_default_style/e_global_class/elementor_library; tevily-themer → gva__template/gva_megamenu/portfolio; ba-book-everything (inactive) → coupon/faq/fee/order/places/service/to_book | `babe_*` options whitelisted; `wp-core/misc` folds 183 core keys |
| aucoeur-test | traveltour-child → ThemeDef ✓ | chaty, head-footer-code, site-reviews, tourmaster, wpdiscuz, goodlayers(PhpBridge) | polylang → polylang_mo; CF7 → wpcf7_contact_form; goodlayers → gdlr_core_header; tourmaster → tour + room/coupon/service family; ml-slider → ml-slide/ml-slider; rank-math → rm_content_editor; better-wp-security (inactive) → itsec-dash-card/dashboard | tier-3 marker false-positives on code-snippets/ez-toc/presto/simply-static — see findings |
| imagetravel | traveltour-child → ThemeDef ✓ | goodlayers(PhpBridge), head-footer-code, tourmaster | none (no source pulled) | dump-only mode behaves: listing is complete, suggestions just say `<plugin-slug>` |
| dmc | tevily_child → ThemeDef ✓ | chaty, head-footer-code | none (no source pulled) | `to_book (80)` correctly claimed by babe adapter even without source |
| imagedmc | tevily_child → ThemeDef ✓ | chaty, head-footer-code | none (no source pulled) | `to_book (92)` claimed; cleanest report of the five |

## Bugs found during this test (all fixed in the same pass)

1. **Shortcode section was ~90% noise.** Raw `\[tag\]` matching caught
   CSS attribute selectors (`[type]` ×13671, `[class]`, `[src]`,
   `[data-*]`), prose brackets in Vietnamese text (`[nh]`, `[sg]`,
   `[read]`), and `[cdata]`. Fix: cross-reference against
   `add_shortcode` registrations in pulled source + WP core tags +
   rpress-native renders; unregistered patterns collapse to one line.
2. **Core tags mislabeled.** `[gallery]`/`[video]` reported as
   "needs render support" — they're WP core, now a third arm.
3. **Dynamic registrations confirmed resolving.** aucoeur shows
   `self::$faq_post_type`-style slugs attributed correctly to
   ba-book-everything (blueskies) and tevily-themer's
   `self::$post_type` → `gva__template`.

## Remaining gaps (feed into the improvement plan)

- **Builder markers false-positive.** `code-snippets`, `ez-toc`,
  `presto-player`, `simply-static` on aucoeur all hit "elementor
  widgets" merely because bundled elementor-compat files contain
  `Widget_Base`. Needs a marker→plugin coupling check (e.g. marker in
  the plugin's own top-level files or a minimum marker count).
- **No `--www` → no attribution warning.** On imagetravel/dmc/imagedmc
  the report silently omits `· registers`; it should say
  "plugins dir not pulled — attribution unavailable" so a reader
  doesn't assume the plugins register nothing.
- **`itsec-dash-card`/`popup`/`pp_video_block` unclaimed** on all sites
  — those are storage CPTs (dashboards, popup themes, video blocks)
  that probably should be skipped at import; detect can't tell yet.
  Needs an adapter `claim_types` spec or an explicit "ignore-types"
  convention.
- **`claimed()` mirrors `kind_of` by hand** — documented drift risk.
- **`register_post_type(get_option(...))` / DB-held slugs** are still
  unresolvable (none observed yet, low priority).
- **Theme-CPT attribution untested**: no site here has a theme that
  registers a CPT, so the `theme:<dir>` path is only code-reviewed.

## Suggested next steps quality check

- aucoeur suggestions name real spec files
  (`plugins/specs/ba-book-everything.toml` style) where owner known ✓
- dump-only sites correctly degrade to `<plugin-slug>` placeholders ✓
- `PLUGINS_PHP=goodlayers` on imagetravel matches the actual
  configuration in `sites/imagetravel.env` ✓

## Verdict

Detection itself is accurate on pulled-source sites — every claim
spot-checked against PHP source held up. The two real product issues:
shortcode noise (fixed) and marker false-positives (open). The bigger
systemic gap is *pull coverage*: 3/5 sites can't do attribution at all
because plugin source was never fetched.
