# Phase 01 — Baseline visual audit

## Overview

- **Priority:** P1 — every later phase is scoped by these findings.
- **Deliverable:** `reports/baseline-ui-audit.md` + `reports/screens/<screen>.png`.

## Requirements

1. Authenticate a real browser session:
   - `T=$(sqlite3 work/aucoeur-test/rpress.db "SELECT token FROM sessions ORDER BY created_at DESC LIMIT 1")`
   - Set `rpress_session=<T>` cookie on `localhost:8410`, viewport 1440×900.
2. Screenshot + inspect every admin screen (logged in unless noted):
   `/admin`, `/admin/posts`, `/admin/pages`, `/admin/tours`, `/admin/edit/<id>` (first published post + first tour), `/admin/comments`, `/admin/media`, `/admin/media/new`, `/admin/menus`, `/admin/widgets`, `/admin/users`, `/admin/user-new`, `/admin/profile`, `/admin/settings/{general,writing,reading,discussion,media,permalink,privacy}`, `/admin/tax/category`, `/admin/tax/region`, `/admin/contact`, `/admin/contact/inbox`, `/admin/bookings`, `/admin/plugins/tourmaster/new-filter`, `/admin/login` (fresh tab, cookie cleared).
3. For each screen, record concrete findings — not vibes:
   - spacing: cramped/oversized paddings, misaligned rows, table cell rhythm;
   - buttons: divergent height/padding/variants (`.button`, `.button-primary`, raw `<a>`), missing `:hover`/`:active`;
   - color/contrast: low-contrast text, off-palette hard-coded colors;
   - states: missing `:hover` on interactive rows/links/buttons, missing `:focus-visible`;
   - transitions: none vs jarring instant changes;
   - inconsistencies vs WP chrome (adminbar, sidebar, notices, tabs, pagination).
4. Fetch `/admin/wp/atrium.css` + `/admin/wp/cms-overrides.css` text; note:
   duplicate/conflicting rules, magic numbers that repeat (candidates for a
   spacing scale), hard-coded colors that repeat (candidates for a color
   palette), missing focus styles, dead rules.
5. Grep `src/admin/*.rs` for `style="` inline attributes; list which ones are
   purely presentational and should move into a class in phase 2/3.

## Success criteria

- Report exists with findings grouped A (design-token-level), B (component-level), C (per-screen).
- Every finding cites evidence: CSS selector/rule or a DOM measurement.
- Screenshots saved for all ~20 screens for later before/after diffing.

## Risk

- Cookie auth in headless browser: set cookie on the origin before `/admin`, or reuse an authenticated tab.
- Login page needs a *separate* unauthenticated context.
