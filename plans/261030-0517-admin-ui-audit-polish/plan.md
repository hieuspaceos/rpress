---
title: "Admin UI audit & visual polish"
description: "Full visual audit and design-token-driven polish of every rpress admin screen — spacing, buttons, colors, hover, transitions — with browser-verified regression gates and zero behavior changes."
status: pending
priority: P2
effort: 10h
branch: main
tags: [frontend, refactor]
blockedBy: []
blocks: []
created: 2026-10-30
---

# Admin UI audit & visual polish

## Overview

Make the WP-parity admin (`/admin`) feel smooth and cohesive: consistent
spacing rhythm, unified button system, sensible color/contrast, proper
`:hover` / `:focus-visible` / `:active` states and short transitions across
all screens — dashboard, list tables, editor, comments, media, menus,
widgets, users, settings, taxonomies, contact, login.

**Hard constraint: no behavior changes.** CSS and presentational class work
only. No handler, route, template-structure, or JS-contract changes. Every
phase ends with a browser gate: real screenshots plus a functional spot-check
of the actions exercised in the October admin audit (create/edit/delete,
bulk, quickedit, menus save, widget save, settings save).

## Non-goals

- Redesigning the WP-parity layout (sidebar, adminbar stay).
- Touching the public frontend render.
- New features or copy changes.

## Phases

| Phase | Name | Status |
|-------|------|--------|
| 1 | [Baseline visual audit](./phase-01-baseline-visual-audit.md) | Pending |
| 2 | [Design tokens + shared components](./phase-02-design-tokens-component-pass.md) | Pending |
| 3 | [Per-screen detail polish](./phase-03-screen-detail-polish.md) | Pending |
| 4 | [Browser functional + visual regression](./phase-04-browser-functional-regression.md) | Pending |

## Key files

- `templates/admin/wp/atrium.css` (1592 lines — rpress's admin layer) — primary edit surface.
- `templates/admin/wp/cms-overrides.css` (142 lines) — targeted overrides.
- `templates/admin/wp/*.min.css` — vendored WP core: **do not edit**; override in atrium.
- `src/admin/*.rs` — inline `style=` attributes (≈160 sites): convert to classes only where they block token polish; markup must keep identical semantics.

## Dependencies

- Live `rpress serve` on `127.0.0.1:8410` (`scripts/e2e-site.sh sites/aucoeur-test.env serve`).
- Playwright/Chromium via the `browser` tool for screenshots and interaction.
- `sqlite3` for session tokens + DB verification.
