---
title: "Site-agnostic chrome: all 6 phases landed"
date: 2026-10-28
summary: "261028-site-agnostic-chrome executed end-to-end: theme-separated template dirs, per-site custom css carry, tevily bases ported from blueskies origin, structural smoke gate green on all four sites."
---

# Site-agnostic chrome: 6 phases, all committed

## What landed

- **Phase 03 (4a073a2)** — `ThemeDef` + `theme()` on `PluginAdapter`,
  `theme_for_stylesheet()`; `load_env_theme` probes
  `templates/theme/<dir>` then shared root (env cache keyed
  `(dir, theme_dir)`); all 10 templates switched to `{% extends base %}`
  with `base` injected per ctx; `base_fr/base_es` +
  `site_reviews_static` moved to `templates/theme/traveltour/`;
  `theme_traveltour::migrate` stylesheet-gated; `theme_tevily` stub
  registered (dash+underscore separators).
- **Phase 04 (430fb94)** — `carry_custom_css` in theme migrate: strips
  own-origin `url()` refs to site-relative (incl. quoted `url("…")`
  fix), emits `.scoped.css` twin via new `css_scope.rs` (Rust port of
  `scripts/scope-css.py`, parity-verified), seeds
  `plugin:theme:css_upload_path`; base link guarded by
  `site.chrome.theme_css`; `front_css` route serves per-site uploads
  twin before repo copy; repo `traveltour-style-custom*` deleted;
  non-traveltour sites get `plugin:theme-traveltour:*` rows purged.
- **Phase 06 (ed70e12)** — `templates/theme/tevily/base_en.html` ported
  from blueskies origin markup (real header/nav/footer); wpfc-minified
  bundles swapped for the real tevily stylesheet set;
  `menu_li_tevily` (`.submenu-inner` + caret + `.menu-title`);
  `template_name` theme-dir authoritative (no cross-theme base
  fallback); `body_class` suffix resolves via `ThemeDef.body_suffix`;
  tevily migrate seeds `site_body_suffix`; `base-minimal.html` deleted.
- **Phase 05 (464eaf5)** — `scripts/visual_smoke.sh` structural gate:
  per-site render + chrome asserts (no jinja leak, no cross-theme
  chrome, menu presence, no foreign-origin links); aucoeur strict gate
  = nav `<li>` sequence + footer byte-parity vs
  `tests/fixtures/aucoeur-*-baseline.txt`. All sites PASS.
- Plan marked done (821287d). Commits this session: `a2a32f3` (phase
  02, earlier), `4a073a2`, `430fb94`, `ed70e12`, `464eaf5`, `821287d`.

## Verification

- `cargo check` clean (0 warnings), `cargo test` ~90 tests all green.
- Aucoeur full regen: 1239 pages, 0 failed; content byte-identical to
  previous `www/` (only pagefind internals + mirrored statics differ).
- blueskies `shop`/VI/FR pages render through `base_en.html` — tevily
  markup, its own menus (EN/FR/VI per `theme_locations` backfill), no
  traveltour strings, no origin links (trustpilot widget excepted).
- imagetravel renders own chrome (281 nav items), no aucoeur chrome.

## Surprises worth remembering

- **blueskies DB was stale**: imported before `theme_mods_*` filter and
  `theme_locations` lang rebuild existed — `theme_locations` was empty
  with old schema. Backfilled `primary→70 (en) / 2017 (vi) / 111254
  (fr)` and `menus.lang` by name convention. Other sites imported in
  the same era may need the same backfill.
- **`grep -c` counts lines**, not matches — rendered HTML is one giant
  line; smoke asserts must `grep -o | wc -l`.
- **Test fixtures need `stylesheet`**: three suites silently relied on
  root `base_fr.html`; they now seed `stylesheet='traveltour'`.
- **`template_name` must not fall ACROSS theme dirs**: tevily's missing
  `base_fr` correctly maps to its own `base_en`, not traveltour's
  `base_fr`.
