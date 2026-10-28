# Phase 2 — Atrium skin: tokens, glass chrome, dock

## Overview

- **Priority:** P1
- **Status:** Complete
- Deliver `templates/admin/wp/atrium.css` covering every admin surface:
  tokens, aurora+grain body, glass chrome (top bar + bottom dock), buttons,
  forms, list tables, postboxes, notices, media modal, TinyMCE, login.

## Key Insights

- One stylesheet loaded after `cms-overrides.css` (`html_admin`,
  `src/admin/mod.rs:960`). WP vendor CSS supplies layout; Atrium repaints —
  **no metric changes** (`width/height/top/left/position`) EXCEPT the
  enumerated nav-chrome block: `#adminmenuback`, `#adminmenuwrap`,
  `#adminmenu`, `.wp-submenu` — the dock IS a metric change there.
- `#adminmenuwrap` is pinned `position:fixed !important; top:32px; bottom:0;
  overflow-y:auto` (cms-overrides.css:6-16). Overriding needs `!important`;
  flyouts must escape its overflow clip → `.wp-submenu` positioned `fixed`.
  `common.min.js` writes inline `margin-top` on `.wp-submenu` → override via
  `margin-top:0 !important` in the dock scope (documented waiver).
- `#wpadminbar` pins at 32px: `.wp-pin` offsets `top:32px`/`top:68px`
  (cms-overrides.css:78-96). Keep the bar exactly 32px.
- `td` elements have `column-*` classes only — `data-colname` is never
  emitted; card labels come from a CSS `content` map keyed on `column-*`.
- Media modal + attachment modal are custom with ~30 inline styles
  (`#mf-frame`, `#att-modal`; admin.rs:1361-1481) — scoped `!important`
  overrides are the documented exception, not the rule.
- TinyMCE iframe: `content_css` is hardcoded at posts.rs:868, posts.rs:1043,
  widgets.rs:631 — append `/admin/wp/atrium.css` to those three strings
  (permitted diff #4).
- Login head is separate (`html_login`, auth.rs:6-17): append the link after
  `login.min.css`; login loads no cms-overrides/colors/common, so login
  rules must be self-contained; Ferris JS stays off login.
- `backdrop-filter` needs `-webkit-` prefix for older Safari; provide both.
  Browser floor unstated — degrade gracefully (opaque fallbacks).

## Requirements (numbered per component)

1. **Token block** — Hallmark stamp first line: paper/paper-2/3, ink, muted,
   rules, accent coral + softs (teal/violet/lime/amber/ok), glass colors,
   blur value, shadows, radii, durations. No literal color/font past it.
2. **Body** — aurora radial-gradient washes + faint grain (data-URI SVG,
   `opacity ≤ .04`, `pointer-events:none`).
3. **`#wpadminbar`** — glass strip, **exactly 32px** (see Key Insights).
4. **Dock** — `#adminmenuback`/`#adminmenuwrap`/`#adminmenu` repainted as a
   bottom-center rounded pill (`!important` on the wrap): icon tiles ~52px,
   hover lift, running-dot for `.current`, separators, overflow scrolls
   horizontally. `.wp-submenu` → `position:fixed` glass flyout above its
   tile; `.wp-submenu-head` hidden or repurposed. `wp-responsive-open` and
   folded states verified after restyle.
5. **Buttons** — `.button`/`.button-primary`: rounded 9px, coral gradient +
   glow on primary, press-travel `:active`. `focus-visible` instant.
6. **`.wp-list-table`** — airy rows, thead small muted labels (sentence case,
   no tracked caps per Floor 54), row hover + hover row-actions, tinted
   status pills, mono dates/comments count.
7. **`.postbox`** → glass cards; notices → glass banner + meaning chip;
   `.nav-tab`, inputs, `.tablenav` selects → rounded paper fields.
8. **Modals** — `#mf-frame`/`#att-modal`/thickbox: scoped `!important`
   overrides on inline styles ONLY here (documented waiver); TinyMCE toolbar
   chrome repainted; editor body skinned via `content_css` diff #4.
9. **Login** — self-contained Atrium rules (no reliance on cms-overrides or
   colors.min.css cascade); centered glass card + static Ferris mark; link
   appended after `login.min.css` in `html_login`.
10. **Mobile <782px** — dock → full-width bottom tab bar (top-level items;
    "More" groups the rest); `.wp-list-table` → card list via `column-*`
    content map; pill filters scroll; search pill field.
11. **Icons-in-CSS** — never interpolate runtime/DB-sourced strings into
    `url()`/`content:` — all icon and label values are build-time literals.

## Implementation Steps

1. `templates/admin/wp/atrium.css` — stamp, tokens, components in order.
2. `src/admin/mod.rs:960` — `<link rel="stylesheet" href="/admin/wp/atrium.css">`
   after `cms-overrides.css`.
3. `src/admin/auth.rs` — same link after `login.min.css`.
4. `content_css`: append `/admin/wp/atrium.css` at posts.rs:868, :1043,
   widgets.rs:631.
5. `@font-face` for the three bundled faces (`/admin/wp/fonts/*.woff2`),
   `font-display:swap`, system fallbacks listed.
6. Screenshot posts list mid-build; iterate cascade until clean.

## Todo List

- [x] `atrium.css` token block + components
- [x] `<link>` wired in `html_admin` (mod.rs:960)
- [x] `<link>` wired in `html_login` (auth.rs)
- [x] `content_css` × 3 updated
- [x] `@font-face` declarations under `/admin/wp/fonts/`
- [x] Dock + `.wp-submenu` flyout works incl. folded/responsive states
- [x] `#mf-frame`/`#att-modal` scoped `!important` overrides done
- [x] Mobile tab bar + `column-*` card list at 390px
- [x] Adminbar kept at 32px — `.wp-pin` offsets verified unbroken

## Success Criteria

- Every screen from Phase 1's list restyled; no overlap, clipping, or lost
  hit areas; submenu flyouts reachable from the dock; screenshots reviewed.

## Risk Assessment

- `#adminmenuwrap` `!important` + overflow clip → handled per Key Insights;
  verify flyouts aren't clipped inside the scroller.
- `.wp-submenu` `position:fixed` loses its tile anchor in JS-driven submenu
  logic — verify on hover/focus and on plugin submenu items.
- `backdrop-filter` inside transformed ancestors can lose blur → verify in
  real Chrome/Safari, keep opaque fallback values in tokens.
- Plugin screens (Tourmaster, GoodLayers) load their own CSS late → check
  collisions, scope fixes narrowly.
