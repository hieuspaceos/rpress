# Phase 04 — Browser functional + visual regression gate

## Overview

- **Priority:** P1 — the "must not break features" contract. CSS broke a click? Catch it here.

## Functional regression (live browser, not curl where interaction matters)

Re-exercise the October audit's action matrix on the restyled admin. Use a real browser tab for anything JS-driven (menus drag, widgets, metabox toggles); `curl` is acceptable for pure form POSTs.

Must all pass:

| Area | Action | Expect |
|---|---|---|
| Auth | login page loads; login → dashboard | 200 + session works |
| Posts | list render; quickedit open+save; bulk-trash `ids=<id>`; trash→restore | 303 + DB row status changes |
| Editor | `/admin/edit/{id}` loads metaboxes; save returns 303 | no JS errors in console |
| Comments | approve/unapprove cycle on a reply | status flips in DB |
| Media | `/admin/media` grid renders thumbnails | 200, images not 404 |
| Menus | `/admin/menus` loads; locations save (`assign-main_menu=156`) | 303, `theme_locations` unchanged if same values |
| Widgets | widget save (title change on scratch widget then revert) | 303 + value round-trips |
| Settings | discussion save with all fields | 303 + values persist |
| Search/filters | `/admin/posts?s=...` filters | rows shrink, no 500 |
| Console | capture `tab` console errors on 6 main screens | zero errors |

## Visual regression

- Re-screenshot the full screen list from phase 01 into `reports/screens-after/`; diff-review each pair.
- Viewports: 1440×900 (primary) + 1024×768 spot-check for `.auto-fold`.
- Check hover states visually: hover a row-action cell, a button, a sidebar item; screenshot the state (evaluate `dispatchEvent` mouseover or use `tab` hover API).
- Check `:focus-visible` rings via keyboard Tab on login + posts filter bar.

## Non-regression diff

- `git diff` review: only `*.css` + (optionally) additive `class=` attributes in `src/admin/*.rs`.
- `cargo build --release` green; restart server once before the browser run so the served CSS is current.

## Success criteria

- Functional table: all rows pass.
- Visual: no screen regresses vs baseline intent; every audit finding is closed or declined with reason.
- `git status` clean after commit; plan archived after user accepts.
