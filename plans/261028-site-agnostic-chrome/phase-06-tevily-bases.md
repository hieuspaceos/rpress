---
phase: 6
title: "Tevily base templates authored from harvest/origin markup"
status: pending
priority: P1
effort: "1.5d"
dependencies: [3]
---

# Phase 6: tevily base templates

## Goal

Author real `templates/theme/tevily/base_{lang}.html` so blueskies and
dmc/imagedmc inner pages render actual tevily markup — not a
base-minimal skeleton, not traveltour fallback. Source of truth is the
sites' own harvested origin HTML: blueskies pages were partially
harvested, so its header/footer chrome exists as frozen markup to
parameterize.

## Findings covered

- Red-team: phase 03 originally scoped tevily bases as "start from
  base-minimal skeleton" — rejected; blueskies deserves real markup
  ported from its own harvested HTML (nav structure, footer layout,
  tevily class names its CSS expects). Split into this phase (~1.5d).
- `templates/theme/tevily/` exists EMPTY from phase 03; the loader
  already probes it first.
- `templates/theme/base-minimal.html` — orphaned (zero refs); delete
  here once tevily bases exist.
- blueskies runs `tevily_child`; dmc/imagedmc also `tevily_child` —
  both get these bases for non-harvested pages.
- Menu/footer/body ctx vars already exist post-phases 01–03:
  `site.menu_main`, `site.footer_widgets`, `{{ body }}`, `flags`,
  `body_class`, `base`.

## Key insights

- **Port, don't invent.** Take a harvested blueskies page's
  header/nav/footer HTML, replace its concrete links/text with
  `{% for item in site.menu_main %}` (same macro shape phase 01
  built for traveltour), `{{ site.* }}`, `{{ body | safe }}` in the
  content slot. Keep tevily wrapper classes verbatim — the theme CSS
  keys off them.
- **CSS**: tevily pages link tevily stylesheets, not traveltour's —
  link set comes from the harvest head (strip origin-absolute URLs to
  `{site_url}`-relative, same rule as menu urls). Phase 04's
  `site.theme_css` mechanism covers the custom-css link if tevily has
  an uploads-generated twin (check the dump; likely none — link only
  what the harvest shows).
- **Lang coverage**: blueskies is EN+VI. Author `base_en.html` first;
  `base_vi.html` only if the harvest shows differing markup — else the
  lang fallback (template_name probes `base_{lang}` → primary) serves
  EN structure, which is correct if VI chrome differs only in strings
  (VI translations deferred anyway per master plan).
- **Harvest pages stay frozen**: harvested pages bypass templates —
  this phase only affects pages that render through jinja.
- **base-minimal deletion** happens in this phase (after tevily bases
  land and render), per red-team "author tevily base before deleting
  it".

## Files to modify

- `templates/theme/tevily/base_en.html` (new) — ported header/nav/
  footer/content-slot markup.
- `templates/theme/tevily/base_vi.html` (new, optional — only on real
  markup divergence).
- `templates/theme/base-minimal.html` — delete.
- `src/admin/plugins/theme_tevily/` — if porting reveals seed gaps
  (e.g. tevily-specific `plugin:theme:*` keys for topbar/social), add
  to adapter seeds.
- `templates/theme/tevily/` partials if the port factors nav/footer
  into `{% include %}`s — keep flat unless duplication forces it (KISS).

## Implementation steps

1. Pick a representative harvested blueskies page (inner, non-home —
  home is often harvest anyway); extract its chrome skeleton.
2. Parameterize: menu → `site.menu_main` macro; logo/site_name →
  `site.*`; footer columns → `site.footer_widgets`; head links →
  site-relative hrefs + `site.theme_css` guard.
3. `base_en.html` in place; generate blueskies inner page; compare
  structure vs origin harvest (nav order, footer blocks).
4. VI check: generate a VI page — if markup identical modulo strings,
  skip `base_vi.html`.
5. Delete `templates/theme/base-minimal.html`.
6. `cargo check && cargo test`; phase-05 blueskies asserts.

## Verification

```bash
cargo check && cargo test
ls templates/theme/tevily/        # base_en.html (+ base_vi.html?)
test ! -f templates/theme/base-minimal.html
# blueskies generated inner page: tevily classes present,
# traveltour-body absent, menu = its own menu items, css links 200
```

## Todos
- [ ] Harvest source page selected; chrome skeleton extracted
- [ ] `base_en.html` parameterized + renders
- [ ] VI coverage decided (fallback vs base_vi.html)
- [ ] base-minimal.html deleted
- [ ] Blueskies inner page renders tevily chrome + own menu

## Risks

- Harvested chrome may embed origin-absolute or JS-driven structures
  that don't survive parameterization — port conservatively: structure
  and classes over JS behavior (menu hover scripts live in tevily JS
  already linked).
- dmc/imagedmc uses tevily_child too — its non-harvested pages get
  these bases; if its harvest shows a different chrome variant than
  blueskies, flag it rather than forking the adapter (one tevily base
  serves both; per-site differences come through `site.*` data).
