---
title: "Admin reskin: 'Atrium' — Notion×glass admin for rpress"
description: "CSS/JS-only reskin of the whole wp-admin clone: warm Notion-style content, floating glass surfaces, bottom dock nav, custom icon set, Ferris mascot. No markup or Rust changes."
status: complete
priority: P1
effort: 8h
tags: [frontend, feature]
blockedBy: []
blocks: []
created: 2026-10-28
---

# Admin reskin: "Atrium"

## Overview

Reskin the entire rpress admin (the wp-admin clone) as **Atrium**: Notion-style
warm paper + Apple-glass chrome, with a macOS-style dock at the bottom edge on
desktop and an iOS-style bottom tab bar on mobile — plus **Ferris**, a custom
rust-crab mascot (Rust × Press) that roams the page, hunts a parked cursor
Tom-&-Jerry style, and dodges a cursor that pokes it.

Design language (prototyped in `assets/atrium-mock-v4.html` and
`assets/atrium-mock-v4-mobile.html`):

- **Surfaces**: warm paper `oklch(97% .008 90)` under a 3-stop aurora wash
  (coral/violet/teal) + faint film grain; cards and chrome are translucent
  `backdrop-blur` panels with hairline light borders and soft float shadows.
- **Type**: Space Grotesk UI, Lora italic serif for screen titles ("Posts."),
  IBM Plex Mono for labels/figures; tilted `// section` kickers.
- **Color**: coral `oklch(60% .2 38)` is the signal; status pills take teal /
  violet / lime / amber by meaning; menu icons carry section tints.
- **Nav**: bottom glass dock — a repaint of `#adminmenu` DOM (`li` +
  `ul.wp-submenu.wp-submenu-wrap`): rounded pill, icon tiles, tooltips,
  running-dot, badges, a "+" tile; `.wp-submenu` → flyout above the tile.
  On <782px it becomes a full-width tab bar with a "More" item. Requires
  `!important` on `#adminmenuwrap`'s existing `!important` pin
  (cms-overrides.css:6-16) and neutralizing `common.min.js`'s inline
  `margin-top` on `.wp-submenu`.
- **Ferris**: JS-driven crab (entrance scuttle, edge patrol, cursor hunt
  with rev-and-charge, <80px dodge, speech lines on click, speed streaks +
  dust puffs). All animation gated by `prefers-reduced-motion`; the
  `#ferris` node itself is script-injected so no-JS renders clean.
- **Mobile**: posts table becomes a card list — labels derived from WP
  `column-*` classes via a CSS map (`data-colname` is never emitted).

Hard constraint: **no markup changes**; permitted code changes are exactly
four small diffs —
1. `<link href="/admin/wp/atrium.css">` in `html_admin` head
   (`src/admin/mod.rs:960`, after `cms-overrides.css`),
2. same link appended after `login.min.css` in `html_login`
   (`src/admin/auth.rs:15` — login does not load cms-overrides),
3. `<script src="/admin/wp/atrium.js" defer>` at `chrome_end`
   (`src/admin/mod.rs:1360`, singleton-guarded against htmx fragment swaps),
4. `/admin/wp/atrium.css` appended to the three `content_css` strings
   (`src/admin/posts.rs:868`, `src/admin/posts.rs:1043`,
   `src/admin/widgets.rs:631`) so the TinyMCE body gets the paper skin.

Assets live INSIDE the served tree `templates/admin/wp/` — the only admin
asset route is `/admin/wp/{*path}` (mod.rs:209, 1553); `/admin/atrium.*` or
`/admin/fonts/*` would 404.

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | Admin looks designed (not WP) across every screen, desktop + mobile | P1 |
| 2 | Zero functional regressions — paint + additive JS only | P1 |
| 3 | Passes Hallmark Floor + `sloplint --scope app` | P2 |

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [Audit and baseline](./phase-01-audit-and-baseline.md) | Pending |
| 2 | [Atrium skin: tokens, glass chrome, dock](./phase-02-tokens-and-chrome-paper-skin.md) | Pending |
| 3 | [Custom icons, Ferris mascot, motion](./phase-03-custom-icons-and-motion.md) | Pending |
| 4 | [Verification sweep](./phase-04-verification-sweep.md) | Pending |

## Success Criteria

- [ ] `atrium.css` + `atrium.js` served under `/admin/wp/`; every admin screen restyled
- [ ] Dock nav + submenu flyouts work; mobile tab bar + card lists work at 390px
- [ ] All used `dashicons-*` mapped to custom tinted glyphs or left intact
- [ ] Ferris runs, hunts, dodges, chats; absent under reduced-motion/no-JS
- [ ] Baseline vs reskin screenshots for all screens reviewed
- [ ] Media modal, menu drag, quick-edit, metabox collapse still work
- [ ] `sloplint --render --scope app` clean

## Red Team Review

### Session — 2026-10-28
**Findings:** 12 (12 accepted, 0 rejected)
**Severity breakdown:** 4 Critical, 4 High, 4 Medium
**Reviewers:** Security Adversary + Fact Checker; Failure Mode Analyst + Flow
Tracer; Assumption Destroyer + Scope Auditor

| # | Finding | Severity | Disposition | Applied To |
|---|---------|----------|-------------|------------|
| 1 | `/admin/atrium.*` + `/admin/fonts/*` have no route — only `/admin/wp/{*path}` serves `templates/admin/wp` | Critical | Accept | plan.md, Ph 1, 2, 3, 4 — assets under `templates/admin/wp/` |
| 2 | `ul.submenu` doesn't exist — real selector `ul.wp-submenu.wp-submenu-wrap` | Critical | Accept | plan.md, Ph 1, 2 |
| 3 | `data-colname` never emitted — mobile card labels from `column-*` class map | Critical | Accept | plan.md, Ph 1, 2 |
| 4 | Dock needs `!important` on `#adminmenuwrap` pin + flyout escapes overflow clip + neutralize inline margin-top | Critical | Accept | plan.md, Ph 1, 2 |
| 5 | Media/attachment modals custom with inline styles — scoped `!important` waiver | High | Accept | Ph 1, 2 |
| 6 | TinyMCE `content_css` hardcoded ×3 — 4th permitted diff appends atrium.css | High | Accept | plan.md, Ph 2 |
| 7 | Login has separate head, no cms-overrides — self-contained rules + link after `login.min.css` | High | Accept | plan.md, Ph 1, 2 |
| 8 | Adminbar 30–36px would break `.wp-pin` 32/68px offsets | Medium | Accept | Ph 2 — bar pinned at 32px |
| 9 | No admin-footer insertion; htmx swaps re-exec scripts — `chrome_end` + `__ferrisInit` guard | High | Accept | plan.md, Ph 3 |
| 10 | sloplint path wrong; dashicons.woff kept as fallback | Medium | Accept | Ph 4 absolute path; Ph 1/3 fallback noted |
| 11 | `--smoke` flag unverified | Medium | Accept | Ph 1 — verify `--help` first |
| 12 | Never interpolate runtime/DB strings into `url()`/`content:` | Medium | Accept | Ph 2 req 11, Ph 3 icons |

### Whole-Plan Consistency Sweep
- Files reread: plan.md, phase-01, phase-02, phase-03, phase-04
- Decision deltas checked: asset paths, submenu selectors, data-colname,
  permitted-diff count, login head, injection point, `--smoke`
- Reconciled stale references: `data-colname`/`ul.submenu`/`/admin/fonts`/old
  constraint prose removed or marked "never emitted"
- Unresolved contradictions: 0

<!-- slug: admin-desk-reskin -->
