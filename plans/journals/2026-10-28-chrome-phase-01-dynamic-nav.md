---
title: "Chrome phase 01: dynamic header nav"
date: 2026-10-28
summary: "Hardcoded aucoeur nav replaced by menus/menu_items-driven trees: theme_locations gains lang, import writes per-lang location bindings, url/menu resolution modules, base_fr/base_es menu blocks cut over, nav_map taxonomy literal removed."
---

# Chrome phase 01: dynamic header nav

## What happened

Implemented `plans/261028-site-agnostic-chrome/phase-01`:

- `theme_locations` rebuilt PK `(location,lang)` with migration rebuild
  (old PK `location` cannot hold Polylang's per-lang bindings).
- Import: `theme_mods_*` allowlisted; `nav_menu_locations` deserialized
  from BOTH `theme_mods_{template}` and `theme_mods_{stylesheet}` and
  merged (parent + child maps differ per lang); menu_id=0 skips; lang
  comes from menus' `-{lang}` suffix via `menus.wp_term_id` join.
- `src/url_util.rs::strip_origin` — own-origin + alias re-anchor;
  goodlayers `boot.rs` delegates its JSON walker to it.
- `src/render/menu_url.rs` — `resolve_menu_url`: post_type→posts.wp_id
  join (publish-gated, front→lang root, display_slug/kind_base aware);
  taxonomy→terms.id (= WP term_id) join + canonical cat/tag bases;
  custom→strip_origin (foreign hosts pass through, `#`→nohref sentinel);
  lang-switcher→None. 7 unit tests.
- `site_data`: menu_items query extended (location, object_id,
  item_type, classes, title_attr); `build_menu_tree` orphan-drop +
  ord-sorted children; `resolve_tree` resolves URLs, drops dead
  subtrees, emits WP `menu-item-*` classes incl. `has-children`;
  `menu_main`/`menu_footer` (+ `_html` pre-rendered li markup) picked
  via theme_locations(location,lang) → menus.lang → first.
- `chrome_vars` + `generate::bare_ctx` inject `site` ctx so every base
  render (post/page/tour/archive/404) gets menus.
- `nav_map` tour arm: taxonomy literal replaced by menu-item
  intersection (post_terms ∩ taxonomy-typed menu_items).
- `apply_navx` signature gains conn+lang; tax keys resolve canonical
  paths (cat_path/tag_base) matching the new resolved hrefs.
- `base_fr`/`base_es`: hardcoded ~150-item nav replaced by
  `site.menus.menu_main_html` in both mobile (m-menu) and desktop
  (sf-menu) uls; center-logo li preserved.

## Verification

- `cargo test`: 89 pass / 0 fail (incl. new url_util + menu_url units).
- Live serve on aucoeur-test DB: `/` renders 124 menu li across
  mobile+desktop navs — roots 6963/9324/7685/7686/6964/8850 in ord
  order, bare `<a>` containers, image items intact; ES page renders
  menu 579 under `/es/` URLs. `/es/` root 404 is the missing es front
  page (page_on_front is fr-only in this DB) — pre-existing, not nav.

## Open / phase-02+

- `bare_ctx` body_class still hardcodes traveltour classes (phase 02).
- Mega-menu image furniture deferred to phase 03 theme adapter per plan.
- imagetravel no-theme_mods fallback not exercised (no dump on hand).
