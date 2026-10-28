# Phase 1 — Audit and baseline

## Overview

- **Priority:** P1
- **Status:** Complete
- Inventory everything the reskin must cover before writing CSS/JS: the icon
  map, the submenu structure the dock must fly out, the CSS load order, mobile
  breakpoints, and a visual baseline.

## Key Insights

- Admin skin is vendored WordPress 7.1.3 CSS under `templates/admin/wp/`, loaded
- Markup is generated inside `src/admin/*.rs`. JS depends on class names and
  inline sizes (112px media thumbs) — none of it changes.
- `#adminmenu` markup: top-level `li` + `ul.wp-submenu.wp-submenu-wrap`
  (src/admin/mod.rs:1181, with `.wp-submenu-head`/`.wp-first-item`) — NOT
  `ul.submenu`. `common.min.js` sets inline `margin-top` on `.wp-submenu`;
  `#adminmenuwrap` is pinned `position:fixed !important` with overflow-y:auto
  (cms-overrides.css:6-16) — both must be handled by the dock repaint.
- `td` elements carry `column-*` classes only — `data-colname` is never
  emitted (list.rs:156) — mobile card labels come from a CSS class map.
- `templates/admin/wp-ref/NOTES.md` documents load order + Modern scheme.
- Assets MUST live under `templates/admin/wp/` — the only admin asset route
  is `/admin/wp/{*path}` reading `templates/admin/wp` (mod.rs:209, 1553).
  `templates/admin/wp/fonts/` already exists.
- Login is a separate head (`html_login`, src/admin/auth.rs:6-17): 7 fixed
  stylesheets, no cms-overrides, no common.min.js.
- `rpress site serve` is the sandbox runner; a site env under `sites/`
  (e.g. `imagetravel.env`) provides real content.
- Font MIME exists inside the `/admin/wp/*` handler (mod.rs:~1600 CT map),
  not at :751 — place fonts under `templates/admin/wp/fonts/`.
- `templates/admin/wp-ref/NOTES.md` documents load order + Modern scheme tokens.
- `rpress site serve` is the sandbox runner; a site env under `sites/`
  (e.g. `imagetravel.env`) provides real content.

## Requirements

1. Enumerate every admin screen: login, dashboard, posts, post edit (TinyMCE),
   pages, media library + upload modal, menus, comments, users, settings,
   widgets, revisions, taxonomies, contact/Flamingo, forms editor, Rank Math,
   plugin screens (Tourmaster, GoodLayers).
2. `grep -rn "dashicons-" src/admin templates/admin | sort -u` → `reports/icons.md`
   (class, where used, tint, custom glyph needed Y/N). Include JS-toggled
   classes (`classList`, template literals). Note dashicons.woff remains as
   fallback for unmapped classes.
3. Map `#adminmenu` top-level items + `.wp-submenu` contents → dock flyout plan.
4. Serve a site env, log in, screenshot each screen desktop + mobile into
   `reports/baseline/`. Verify `rpress site serve --help` for env/bind flags
   (do not assume `--smoke` exists).
5. Catalogue existing `cms-overrides.css` rules — esp. the `#adminmenuwrap`
   `!important` pin and `.wp-pin` offsets (top:32px/68px).
6. Record WP mobile breakpoints (auto-fold <960px, collapse <782px).

## Implementation Steps

1. `grep -rn "dashicons-" src/admin templates/admin | sort -u` → icon list
   (class, where used, tint, custom glyph needed Y/N).
2. Serve a site env, log in, screenshot each screen desktop + mobile into
   `reports/baseline/`.
3. Download OFL woff2: Space Grotesk, Lora (incl. italic), IBM Plex Mono →
   `templates/admin/wp/fonts/`.

## Todo List

- [x] Icon inventory in `reports/icons.md`
- [x] Baseline screenshots (desktop + 390px) in `reports/baseline/`
- [x] `#adminmenu` DOM/submenu map written down
- [x] `cms-overrides.css` existing rules catalogued
- [x] Fonts downloaded locally

## Success Criteria

- Complete icon map; submenu→flyout map; baseline for every screen at both
  widths.

## Risk Assessment

- JS-toggled icon classes missed by static grep → grep `classList`, template
  literals too.
- A site env may not serve → try another `sites/*.env` or check
  `rpress site serve --help` for flags.
