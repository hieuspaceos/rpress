# Phase 03 — Per-screen detail polish

## Overview

- **Priority:** P2 — apply phase-02 tokens to fix every B/C finding from the baseline audit.
- **Edit surface:** `templates/admin/wp/atrium.css`, `templates/admin/wp/cms-overrides.css`; additive class attributes in `src/admin/*.rs` only where inline `style=` blocks a fix (keep identical semantics; replacing `style=` with a class of the same declarations is acceptable).

## Requirements

Work the audit's per-screen list; each item: fix → reload → screenshot → compare to baseline.

Known/predictable buckets (extend from `reports/baseline-ui-audit.md`):

1. **Dashboard** (`/admin`) — widget cards spacing, quick-draft form alignment.
2. **List screens** (`/admin/posts|pages|tours|comments|media|users|contact`) — tablenav density, column widths that jump, `.row-actions` spacing, filter selects, empty-state ("No items") centering.
3. **Editor** (`/admin/edit/{id}`) — meta-box grid gaps, slug/title input widths, publish box button stack, taxonomy metabox checkboxes.
4. **Menus** (`/admin/menus`) — menu item cards, accordion toggles, locations table.
5. **Widgets** (`/admin/widgets`) — sidebar panels, available-widget cards, drag areas.
6. **Settings** (all 7 sections) — `.form-table` row rhythm, checkbox/radio label gaps, submit bar.
7. **Media** — grid/list density, attachment editor fields.
8. **Users/Profile** — role table, app-password + 2FA sections.
9. **Taxonomies** — two-column add/edit layout, term table.
10. **Login** (`/admin/login` + `/admin/2fa`) — card centering, input focus states, button; ensure `/admin/2fa` uses the same shell (already done in a22bcb0).
11. **Notices/messages** — `?msg=` states visible, consistent placement below header.
12. **Print/narrow widths** — sanity at 1024px for `.auto-fold` sidebar behavior.

## Success criteria

- Every baseline finding has a resolution (fixed, or deliberately declined with reason recorded in the audit report).
- Before/after screenshot pairs for each touched screen in `reports/screens/`.
- No inline-`style=` removals that change computed style beyond the intended fix; grep diff of `src/admin/` shows only additive class attributes (or same-value class extractions).

## Risk

- Inline styles (~160 sites) — touching them risks churn; only convert when a CSS fix is blocked by specificity, else leave.
- Menu/widget JS positioning (jQuery UI, menu-editor.js) — never restyle dimensions JS computes at runtime (positions, drag placeholders); polish colors/spacing only.
