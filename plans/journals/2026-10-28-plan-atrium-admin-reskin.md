---
title: "Plan: Atrium admin reskin"
date: 2026-10-28
summary: "Brainstormed and planned a Notion×glass reskin of the wp-admin clone with bottom dock nav and a Ferris mascot; CSS/JS-only, no markup changes"
---

# Plan: Atrium admin reskin

## What happened

User cloned Nutlope/hallmark into pet/ to reuse as design input for rpress.
Brainstormed admin redesign directions across several mocks: Desk (newsprint
+ graphite), Blueprint (prussian technical sheet), Atrium v1–v4 (Notion-warm
paper + Apple glass). Iterated on layout (sidebar → top nav → bottom dock)
and a mascot: "Ferris", a rust-crab pressman (Rust × Press) that roams the
page, hunts a parked cursor Tom-&-Jerry style, dodges pokes, and chats on
click. Final direction settled: Atrium + bottom dock + Ferris.

## Decision

- Skin = `templates/admin/atrium.css` + `templates/admin/atrium.js` loaded
  after `cms-overrides.css` (src/admin/mod.rs:960). No markup or Rust
  changes — repaint + additive JS only, per the "không bị lỗi" constraint.
- Bottom dock on desktop (repaint of `#adminmenu`, `ul.submenu` → flyout),
  iOS tab bar under 782px, WP `data-colname` card list on mobile.
- Coral accent + teal/violet/lime/amber status pills; Space Grotesk /
  Lora italic / Plex Mono, bundled woff2.
- Ferris: JS engine with streaks/puffs; reduced-motion and no-JS degrade
  cleanly.

## Next steps

Plan: plans/261028-1716-admin-desk-reskin (pinned). 4 phases:
audit/baseline → atrium.css skin → icons+Ferris+motion → verification
sweep (screenshots, click-through, sloplint --scope app). Implement via
/ak:cook when ready.

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
