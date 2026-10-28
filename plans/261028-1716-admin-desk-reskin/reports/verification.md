# Phase 4 — Verification report

Date: 2026-10-29. Site: imagetravel (`rpress site serve`, 127.0.0.1:8414).
Screenshots: `after/` (25 shots), `baseline/` (vendor skin, atrium.css disabled, 6 shots).

## Screens captured and reviewed

| Screen | 1280px | 390px | Notes |
|---|---|---|---|
| Login | ✓ | ✓ | Glass card, Ferris mark, aurora backdrop |
| Dashboard | ✓ | ✓ | Metaboxes stack on mobile |
| Posts list | ✓ | ✓ | Mobile cards labeled (author/date/cats/lang/cmts/seo) |
| Post edit | ✓ | — | TinyMCE toolbar skinned, iframe body unsinned (expected) |
| Media library | ✓ | ✓ | Card list with labels |
| Media modal | ✓ | — | Open/close verified, glass frame |
| Pages | ✓ | — | |
| Comments | ✓ | ✓ | |
| Users | ✓ | — | |
| Settings | ✓ | — | `/admin/settings/general` |
| Menus | ✓ | — | |
| Widgets | ✓ | — | |
| Contact forms | ✓ | — | |
| Site Reviews | ✓ | — | |
| Tours | ✓ | — | |
| Categories | ✓ | — | `/admin/tax/category` |
| Revisions | ✓ | — | Empty-state; diff route exists but no revisions on this post |
| Flyout | ✓ | — | 14px clear of dock, clickable links |
| Quick Edit | ✓ | — | Inline row opens under skin |

## Interactive flows

- Dock hover → glass flyout above tile: pass (clickable, `elementFromPoint` = `A`).
- Mobile tab bar: 5 tiles + More sheet; `more-tile` click reveals hidden items: pass.
- Media modal open/close: pass.
- Quick Edit open: pass.
- Collapse tile → `/admin/new/post`: pass.
- Ferris: spawns, walks, hunts parked cursor (moved 136px in 4s), click chatter: pass. Hidden under 782px and under `prefers-reduced-motion`.
- Singleton: re-executing atrium.js post-load adds no second `#ferris` (`__atriumInit` guard).

## Reduced-motion / no-JS / network

- `prefers-reduced-motion: reduce`: page renders, Ferris absent, transitions 0s.
- JS disabled (`page.setJavaScriptEnabled(false)`): full render, tiles are links, icons paint, no #ferris. Screen usable.
- External resources: none (all requests stay on 127.0.0.1; OFL fonts local). 4xx: none.
- Fonts blocked (woff2 requests aborted): page renders on system fallbacks; no FOUT collapse.
- Motion budget: every transition/animation uses --dur-micro (100ms) or --dur-short (160ms); no literal ms elsewhere. Ferris leg animation .12s/.18s delays only.
- Offline: N/A — admin has no service worker; offline means server unreachable, same as before the skin.

## Console

- `TypeError: p.find(...).hoverIntent is not a function` — fires in `common.min.js` on every screen **with atrium.css disabled too**. Pre-existing vendor issue (hoverIntent plugin never enqueued); not a skin regression. Waived.
- `ReferenceError: wp is not defined` in `user-profile.min.js` — seen once on `/admin/new/post`; vendor enqueue ordering, unrelated to skin.

## Known limitations / waived

- sloplint: binary not installed in this environment; gate not run.
- Dock has no fold control on desktop (collapse tile reused as "+ New Post" link per design).
- Postboxes: this build ships no collapse handler (`.hndle` click does nothing, no `.handlediv`); not a regression.
- TinyMCE iframe content area keeps its own theme — acceptable per plan risk note.
- Mobile row "More" sheet lists items as plain tiles; labels truncated with ellipsis when >64px.

## Fixes applied during sweep

- `#adminmenuwrap` `display:block !important` under 782px (vendor hides it).
- `#adminmenu` `position:static !important` under 782px (vendor `top:46px` absolute).
- `.wp-menu-image` `position:static !important` + 26px `!important` (vendor auto-fold pins).
- `.wp-menu-name` `padding:0 !important` (vendor mobile padding).
- `.dashicons::before` `display:inline-block` + `background:currentColor !important`, `color:inherit !important` (vendor colors.min glyph colors).
- `#adminmenumain #adminmenu` tile `height/min-height !important` (vendor line-height collapse).
- More-mode tiles `max-width:78px`; squeeze rule scoped to 783–960px.
- `.tablenav` mobile `flex-wrap:wrap !important`; `.tablenav-pages` un-floated; search full-width.
- Mobile table cells `width:100% !important`, `overflow-wrap:anywhere`; label map `!important` (beats vendor `attr(data-colname)`); labels added for `categories/comments/pll/rm-seo-col`.
- Postbox containers and post-body columns stack under 782px.
- Flyout gap 12→18px; background hardened to 94% + `--glass-strong`.
- `#ferris { display:none }` under 782px.

## Code review pass (post-verification)

code-reviewer found 14 items; fixes applied and re-verified live:

- **High** — base `.wp-menu-image::before` painted `background:currentColor` on every icon → unmapped dashicons rendered as colored squares. Removed; verified `dashicons-smiley` (unmapped) now renders its woff glyph (`content:"\uf328"`, transparent bg).
- **Med** — 18px flyout gap had no hover bridge → `.wp-submenu::before` bridge added; gap point hit-tests to the flyout and it stays open through the crossing.
- **Med** — `.show-all` more-sheet tiles overflowed off-screen with no scroll → switched to `flex-wrap:wrap` tile grid (scroll would clip flyouts). All 13 tiles verified reachable.
- **Med** — rAF loop leaked on htmx detach → `tick()` bails when `!ferris.isConnected`, also idles when `mobileQ.matches`.
- **Low** — `.puff`/`.streak` hidden under 782px; squash transform reset on the >560px flee exit; Ferris `x` initialized off-screen so the enter slide-in actually plays; x/y clamped each tick (resize safety); label gutter `min-width:88px` scoped to labeled columns only; dead rules removed (`td[class=""]` label, `::before --icon-tint` pseudo variant, duplicate show-all scrollbar rule).
- **Info** — vendor-contract risks documented; `!important` use is counter-important vs cms-overrides/colors.min.
