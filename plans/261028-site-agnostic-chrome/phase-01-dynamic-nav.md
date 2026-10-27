---
phase: 1
title: "Dynamic header nav from menus/menu_items + theme_locations"
status: done
priority: P1
effort: "1.5d"
dependencies: []
---

# Phase 1: dynamic header nav

## Goal

Base templates stop shipping a hardcoded ~150-item aucoeur menu. The
header nav renders from the site's own `menus`/`menu_items` rows —
imported for every site already — with location resolution via
`theme_locations`, per-lang menu pick, and URL resolution per
`item_type`. `apply_navx` keeps annotating `current-*` classes unchanged.

## Findings covered

- `templates/base_fr.html` / `base_es.html` — hardcoded menu <ul>/<li>
  blocks (desktop + mobile + mega-menu "Voyage sur mesure" with images,
  INFOS & IDÉES nested menu, taxonomy menus Par région/durée/profil/
  style). base_es desktop block is verbatim French.
- `src/import.rs:591-654` — nav_menu terms → `menus`, nav_menu_item
  posts → `menu_items` (parent_id, menu_order, label, url for custom,
  object_id, item_type incl. `lang-switcher`). `menus.location` bound to
  the term slug (`:606`), lang derived from `-<lang>` suffix (`:600-605`).
- `src/schema.rs:108-111` + `src/db/menus.rs:366-400` — `theme_locations`
  table + `theme_locations_available` setting + admin UI exist but are
  never populated by import and never read by render.
- `src/render/site_data.rs:318-363` — `build_menu_tree` already nests
  flat items; `site.menus` is built but only handed to element
## Key insights

- **WP location map lives in `theme_mods_{slug}.nav_menu_locations`**
  (PHP-serialized `{location_slug: term_id}`) keyed by the **parent**
  theme slug — `theme_mods_traveltour` for stylesheet
  `traveltour-child`. Import must probe `settings.template` first, then
  `settings.stylesheet`; imagetravel's dump contains ZERO theme_mods
  rows, so the fallback chain is load-bearing, not decorative.
- **The map is per-lang**: aucoeur's `main_menu` binds menu 156 (FR)
  and 579 (ES) inside one option — `theme_locations`' current
  `location→menu_id` shape (`schema.rs:108-111`) cannot hold that.
  **Schema change**: add a `lang` column; PK becomes
  `(location, lang)`. On import, each bound menu's lang comes from the
  `-{lang}` suffix rule already at `import.rs:600-605`. Existing DBs
  get `ALTER TABLE theme_locations ADD COLUMN lang` + backfill from
  `menus.lang`. Update `save_locations`/`list_locations`
  (`db/menus.rs:377-400`), `menus__location_set`/`menus__locations_list`
  SQL, and admin signatures (`admin/menus.rs:443-475` render,
  `:921-925` save) — admin assigns per declared location; the lang
  filter is the assigned menu's own `menus.lang`.
- **Skip `menu_id=0` entries** — aucoeur's dump carries
  `right_menu;i:0` (explicitly unassigned); writing a 0 row would
  shadow a valid fallback.
- **Location slugs are theme-owned**: traveltour's are
  `main_menu`/`mobile_menu`/`right_menu` — NOT `main`/`primary`. The
  render pick chain: adapter `preferred_locations` (phase-03
  `ThemeDef`; tevily's list read from the blueskies dump at
  implementation) → `theme_locations` row matching `(location, lang)`
  → `menus.lang == render lang` → first `menus` row.
- **`object_id` is a WP id, not a local id** — join `menu_items.object_id`
  to `posts.wp_id` for post_type items and `terms.wp_term_id` for
  taxonomy items (the `nav_map` doc at `chrome.rs:234-237` already warns
  about the mismatch).
- **Resolved URLs stay `{site_url}`-absolute.** `apply_navx`
  (`render.rs:251-265`) rebuilds absolute hrefs from navx keys and
  string-matches them inside `<a href>` — a site-relative URL silently
  kills every `current-*` class and fails the aucoeur regression.
- **`custom` items: strip only own origin + alias hosts.** The raw
  `menu_items.url` may point at the site's origin OR an alias
  (aucoeur base_fr references `test.aucoeurvietnam.com`). Strip to
  absolute `{site_url}` form only when the host matches site origin
  or a `plugin:theme:url_aliases` list; foreign hosts
  (facebook/youtube/…) pass through unchanged. Generalize
  `strip_origin_urls` (`src/admin/plugins/goodlayers/boot.rs:332`)
  into `src/url_util.rs::strip_origin(url, site_url, aliases)` —
  boot.rs keeps its Value-tree wrapper calling it.
- **Dead items drop, orphans drop.** WP hides a menu item whose object
  is gone: `menu_url` returning None → drop the item AND its subtree —
  never emit `href="#"` (the aucoeur parity diff counts `<li>`). Orphan
  `parent_id` (parent absent from the menu): `build_menu_tree`
  (`site_data.rs:351-355`) currently treats it as root — change to
  drop-subtree.
- **`lang-switcher` items** (item_type synthesized at `import.rs:625-629`):
  template renders them as flag links using the existing `flags` ctx
  var (already emitted by `chrome_vars:300`).
- **Aucoeur nav parity is the gate**: its DB menus must render the same
  top-level structure WP produced. The mega-menu images inside "Voyage
  sur mesure" are menu-item-adjacent template furniture, NOT menu rows —
  keep them as a per-theme template block (traveltour adapter owns it
  in phase 03).

## Files to modify

- `src/schema.rs` — `theme_locations` gains `lang TEXT`; PK becomes
  `(location, lang)`; `CREATE TABLE IF NOT EXISTS` for new DBs +
  `ALTER TABLE`/backfill step for existing ones (lang from
  `menus.lang` via menu_id join).
- `src/db/menus.rs` (`:377-400`), `src/db/sql/menus__location_set.sql`,
  `src/db/sql/menus__locations_list.sql` — signatures + SQL carry lang.
- `src/admin/menus.rs` (`:443-475` render, `:921-925` save) — Manage
  Locations assigns per location; rows carry the menu's lang.
- `src/import.rs` — options loop (`:162-166` allowlist region, `:720-726`
  insert): accept `theme_mods_*`; deserialize `nav_menu_locations` from
  BOTH `theme_mods_{template}` and `theme_mods_{stylesheet}` (parent-
  theme key wins — `theme_mods_traveltour` for `traveltour-child`);
  write `theme_locations(location, menu_id, lang)` skipping `menu_id=0`
  entries; write `theme_locations_available` from the map keys.
  `php_serialize.rs` already provides deserialization.
- `src/render/menu_url.rs` (new) — `resolve_menu_url(conn, item,
  site_url, lang, tour_base, aliases) -> Option<String>`; returns
  `{site_url}`-ABSOLUTE urls (apply_navx matches absolute hrefs);
  `None` = dead item → caller drops item+subtree. Dispatch on
  `item_type`: `post_type`/`page`/`post` → posts row via `wp_id` join →
  rel_path rules incl. lang prefix + kind bases; `taxonomy` →
  `theme::taxonomy_map` term path via `wp_term_id` join; `custom` →
  `url_util::strip_origin` (own origin + aliases only); `lang-switcher`
  → None (rendered from `flags`, not the tree).
- `src/url_util.rs` (new) — `strip_origin(url, site_url, aliases)`
  single-string helper; `src/admin/plugins/goodlayers/boot.rs:332`
  delegates.
- `src/render/site_data.rs` — `build_menu_tree` (`:351-355`): orphan
  `parent_id` → drop subtree, never promote to root; extend
  `site_data` with `menu_main`/`menu_footer` (resolved URLs baked into
  the tree, cached under the existing `(site_url, lang)` SITE_DATA
  key — resolution is per-lang so the cache key already fits).
- `src/render/chrome.rs` — `chrome_vars`/`nav_map`: inject `site` map
  (incl. menus) into base ctx; move nav_map taxonomy literal to
  `taxonomy_map` keys (or all `post_terms` taxonomies minus language —
  decide: all distinct taxonomies on the post, simpler and site-true).
- `templates/base_fr.html`, `templates/base_es.html` — replace the
  hardcoded menu <ul> trees with `{% for item in site.menu_main %}`
  recursion (macro for nested <ul>); keep the traveltour wrapper
  classes (`traveltour-main-menu`, `dl-submenu`, etc.) since CSS keys
  off them. Mobile menu block reuses the same macro.
- `src/generate.rs` `bare_ctx` (`:596`) — same `site` injection so
  archive/404 renders get nav.

## Implementation steps

1. Schema: `lang` column on `theme_locations` + migration backfill;
   update db/menus fns + SQL + admin signatures.
2. Import: accept `theme_mods_*` (probe `settings.template` then
   `settings.stylesheet`); deserialize; write per-lang
   `theme_locations` skipping 0-assignments +
   `theme_locations_available`. Re-import aucoeur + imagetravel
   dev copies; `sqlite3` verify — **imagetravel is the no-theme_mods
   case**: assert zero rows + fallback path exercised, not an error.
3. `url_util::strip_origin` (origin + aliases) + boot.rs delegation.
4. `menu_url.rs` resolution + unit tests (post_type, taxonomy, custom
   own-origin, custom foreign-host passthrough, dangling object_id →
   None, lang-switcher → None).
5. `build_menu_tree` orphan-drop; `site_data`: pick menu
   (adapter `preferred_locations` → `(location, lang)` row → same-lang
   → first), resolve tree URLs, expose `menu_main`/`menu_footer`.
6. Template macro for nested menu; replace base_fr/base_es blocks;
   render aucoeur home → structural diff of nav <li> sequence vs
   current generated output (count + order must match — no `#` items).
7. nav_map off the literal taxonomy list.
8. `cargo check && cargo test`.

## Verification

```bash
cargo check && cargo test
sqlite3 site/aucoeur-test/work/rpress.db \
  "select location, menu_id, lang from theme_locations"   # per-lang rows
sqlite3 site/imagetravel/work/rpress.db \
  "select count(*) from theme_locations"                  # 0 = fallback path
# aucoeur regression: generate home, extract nav <li> text sequence,
# diff vs previous generated home — top-level order must match
```

Manual: imagetravel page shows its VI/EN menu labels; a menu item
pointing at a taxonomy term resolves to `/{lang}/{tax-base}/{slug}/`.

## Todos
- [ ] `theme_locations.lang` column + migration + admin signatures
- [ ] `theme_mods` import (parent+stylesheet keys) → per-lang rows,
      0-assignments skipped
- [ ] `url_util::strip_origin` (origin+aliases); boot.rs delegates
- [ ] `menu_url.rs` + tests; absolute URLs; None→drop-subtree
- [ ] `build_menu_tree` orphan → drop, not promote
- [ ] `site.menus` resolved into base ctx (`chrome_vars`, `bare_ctx`)
- [ ] base_fr/base_es menu macro; aucoeur structural diff clean
- [ ] nav_map taxonomy literal removed
- [ ] no-theme_mods site (imagetravel) verified on fallback chain

## Risks

- Menus whose import produced wrong `lang` (no `-<lang>` suffix in slug)
  fall back to the first-menu heuristic — acceptable, logged at render
  debug level.
- Theme-mods map may assign a menu to a location the theme doesn't
  declare — keep the row anyway; render only queries declared ones.
- Mega-menu images: aucoeur regression diff will flag them; they stay
  traveltour-template furniture until phase 03 relocates them — do not
  try to data-drive them here (scope guard).
