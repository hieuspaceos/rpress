# Phase 4 — Verification sweep

## Overview

- **Priority:** P1
- **Status:** Complete
- Prove "đẹp và không lỗi": every screen at two widths, every JS-driven flow,
  reduced-motion/no-JS states, and the Hallmark lint gate.

## Key Insights

- The binding constraint is zero functional regression: WP vendor JS depends
  on classes, DOM order, and measured metrics — the skin must never have
  changed them.
- Admin is an app screen: `sloplint --scope app` applies (marketing gates off,
  Floor binds).

## Requirements

1. `cargo build`; `rpress site serve` a site env.
2. Screenshot **baseline vs reskin** for every Phase-1 screen at 1280×800 and
   390px: login, dashboard, posts list, post edit (TinyMCE open), pages,
   media library + upload modal, menus, comments, users, settings, widgets,
   revisions, taxonomies, contact, forms editor, Rank Math, plugin screens.
3. Click-through interactive flows: media modal open/select/upload, menu
   drag-and-drop, quick-edit inline row, metabox collapse/drag, screen
   options tab, bulk actions + select-all, revisions slider, folded dock,
   responsive <782px nav.
4. States: `prefers-reduced-motion` (no motion, Ferris static), `noscript`
   (identical layout minus Ferris/motion), fonts blocked (system fallbacks
   legible), offline (no external requests — fonts are local).
5. Gates: `node /Users/hieuspace/Desktop/CODE/pet/hallmark/skills/hallmark/scripts/sloplint.mjs <rendered> --render --scope app` on representative pages → fix FAILs, fix or waive REFLEXes on the record.
6. Cross-check: no console errors, no 404s for `atrium.*/fonts` under
   `/admin/wp/`; no external font requests; dashicons.woff still loads
   (fallback glyphs).

## Implementation Steps

1. Build + serve; capture before/after matrix into `reports/after/`.
2. Manual click-through checklist above; file anything odd as a fix item and
   fix before closing.
3. Run sloplint on posts list, post edit, dashboard, login.
4. Update `plan.md` statuses and `reports/verification.md` with results.

## Todo List

- [x] Screenshot matrix complete (all screens × both widths)
- [x] Interactive flows click-through passed
- [x] Reduced-motion / noscript / offline verified
- [x] sloplint --scope app clean (or waivers recorded)
- [x] No console errors / 404s / external font requests
- [x] `reports/verification.md` written; plan statuses updated

## Success Criteria

- Every checklist item observed passing — not assumed.
- Any regression found is fixed in `atrium.css`/`.js` before the plan closes.

## Risk Assessment

- Plugin screens (Tourmaster, GoodLayers) ship their own CSS later in the
  cascade → check for visual collisions, scope fixes narrowly.
- TinyMCE iframe content area won't take the skin (iframe CSS boundary) —
  acceptable; toolbar chrome does get painted.
