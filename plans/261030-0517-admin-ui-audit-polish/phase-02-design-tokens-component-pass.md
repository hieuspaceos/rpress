# Phase 02 — Design tokens + shared components

## Overview

- **Priority:** P1 — the "smooth & cohesive" lever; one pass over shared surfaces.
- **Edit surface:** `templates/admin/wp/atrium.css` primary; `templates/admin/wp/cms-overrides.css` for surgical WP-core overrides. **Never edit vendored `.min.css`.**

## Design tokens (add at top of `atrium.css` under `:root`)

- Spacing scale: `--rp-sp-1:4px; --rp-sp-2:8px; --rp-sp-3:12px; --rp-sp-4:16px; --rp-sp-5:24px; --rp-sp-6:32px` (8-pt rhythm; WP uses 8/12/20 inconsistently — normalize inside our components only).
- Color: reuse `admin-schemes.min.css` (`--menu-*`, `--body-*`, `--link-*` vars if exposed); else define `--rp-accent`, `--rp-accent-hover`, `--rp-text`, `--rp-muted`, `--rp-border`, `--rp-surface`, `--rp-surface-2` mapped to the active scheme. Respect `body.{scheme}` color classes — tokens must not fight user color schemes.
- Radius: `--rp-radius:3px` (WP-parity) / `--rp-radius-lg:6px`.
- Motion: `--rp-motion:150ms ease`; apply to hover/background/box-shadow transitions. Add `@media (prefers-reduced-motion: reduce) { * { transition: none !important } }`.
- Shadow: `--rp-shadow-1` subtle card/elevation (kept minimal — WP is flat).

## Shared components to normalize

1. **Buttons** — unify `.button`, `.button-primary`, `.button-small`, `.button-large`, `.btn-*`, link-buttons:
   - one height rhythm (32px default, 24px small), padding `0 var(--rp-sp-3)`;
   - `:hover` darken + transition, `:active` translateY(0)/darker, `:focus-visible` 2px outline ring, `:disabled` reduced opacity + `cursor:default`.
2. **List tables** (`.wp-list-table`) — zebra rows, row `:hover` background, `.row-actions` reveal on hover/focus-within, cell padding `var(--rp-sp-2) var(--rp-sp-3)`, sticky header optional.
3. **Forms** (`.form-table`, inputs/selects/textarea) — 32px control height, focus ring, label weight/spacing consistency.
4. **Notices** (`.notice*`) — consistent border-left, padding, margins between notices.
5. **Pagination / tablenav** (`.tablenav`, `.pagination-links`) — button parity, hover states.
6. **Tabs** (`.nav-tab-wrapper`) — active/inactive hover polish.
7. **Adminbar + sidebar submenu hover/flyout** — transition + alignment checks.
8. **Focus-visible** — global `:focus-visible { outline: 2px solid var(--rp-accent); outline-offset:1px }` inside admin scope.

## Rules

- CSS only; no markup edits in this phase unless a rule is physically impossible to target (then an additive `class` attribute on an existing element — never renaming existing classes/ids used by JS).
- Keep specificity under WP core; no `!important` unless overriding an existing `!important`.
- Verify each block against a live page before moving on (reload + screenshot the affected component).

## Success criteria

- Buttons across dashboard, list tables, editor, settings look and behave identically (heights, hover, transitions) in screenshots.
- Tables show row hover + zebra on posts/comments/media/users screens.
- No regression: phase-4 functional spot-check list still passes (smoke subset inline: posts list renders, editor loads, settings save returns 303).

## Baseline-driven targets (from reports/baseline-ui-audit.md)

A-block fixes, in priority order:

1. **A1/VA — `a.page-title-action` WP-blue leak.** `colors.min.css` `.wrap .page-title-action` (border `--wp-admin-theme-color`, radius 2px) beats atrium on specificity → "Add Post/Add New User/Add New" render stock blue. Fix: add `.wrap a.page-title-action` to the atrium button block (paper bg, `--rule-2` border, `--r-sm`).
2. **A2/A9 — one focus ring.** Zero `a:focus-visible` rules exist; inputs use 35% wash, buttons full `var(--focus)`, checkboxes a third variant. Fix: single `--focus-ring` token; global `a:focus-visible` inside `.wp-admin`; convert `:focus` → `:focus-visible` on checkbox/radio so mouse clicks stop flashing outlines.
3. **A3 — drop `font-style:italic` on row-title hover** (reflows glyphs); keep color change.
4. **A4 — hide literal `|` separators inside `.subsubsub` pills** (verify DOM first; likely `li::after` or text node — hide via font-size/color trick without markup change).
5. **A5 — `transition: filter var(--dur-micro) var(--ease-out)` on `.button-primary`** (hover `brightness(1.05)` currently snaps).
6. **A7 — replace off-palette literals in `cms-overrides.css`**: `#f0f0f0`, `#e5e5e5`, `#fff` (pinned editor toolbars), `rgba(0,0,0,.4)` (modal shadow), `#f1f1f1`/`#444` (captions) → `--paper`/`--paper-2`/`--rule`/`--shadow-float`.
7. **A8 — remove the naked `transition-duration` list in the "MOTION pass" block** (positional misalignment vs each element's property list).
8. **A9 — `input:disabled, select:disabled, [readonly]`** → `background:var(--paper-2); color:var(--muted); border-color:var(--rule)` (settings general fields currently look like plain text).
9. **B17 — adminbar 32 vs 33px seam**: `#wpadminbar{box-sizing:border-box;height:32px}` or pin offsets `top:33px` — affects editor pinned toolbar + pinned sidebar.
10. **B18 — sidebar flyouts animate**: replace `display:none→block` with `visibility`/`opacity`/`translate` (pure CSS).
11. **B11 — checkbox affordance**: darken unchecked border to `--muted`; drop invisible `rotate:90deg` on `:checked`.
12. **B8 — pin `.wp-list-table .check-column{width:2.2em}`** (comments all-hidden case becomes sane card instead of 1368px checkbox column).
13. **B5 — clamp `td.column-description`** (`-webkit-line-clamp:6` or max-height) — kills the 2600px region row.
14. **B9 — `.row-actions a` color `--ink-2`** (muted on `--paper-2` hover is low-contrast).
15. **B22 — `.nav-tab-active` bg → `var(--glass)`** instead of solid `--paper`.
16. **B3 — `.tablenav-pages .button:disabled` lower opacity + `cursor:default`**.
17. **B14 — `.notice::before` chip `vertical-align` fix** (~1px high).
18. **B19 — `#collapse-button:focus-visible` ring**.
19. **B20 — `#wpfooter` left padding align with `.wrap` gutter (24px)**.
20. **B21 — pager mono text 11→12px; `#current-page-selector` height 30px**.

Markup flags (NOT CSS-fixable — record in the report, do not silently touch):
- `/admin/posts` emits `.current` on both "All" and "Mine" pills (template logic in `posts.rs`).
- `/admin/contact` emits a header-only table — no `tr.no-items` row for zero forms.
- `/admin/comments` screen-options can hide every data column → checkbox-only card; a "columns hidden" hint needs markup.
- Taxonomy copy typo "Categorys can have a hierarchy"; search placeholder missing.

## Risk
