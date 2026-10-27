---
title: "Plan drafted: site-agnostic chrome (6 phases, red-teamed)"
date: 2026-10-28
summary: "Visual check exposed that every non-harvested page on all 4 sites renders aucoeur's hardcoded chrome. 2-scout sweep found the full blast radius; planner drafted 261028-site-agnostic-chrome (8-9d); red-team found 2 blockers + 6 concerns, all amended."
---

# Site-agnostic chrome plan

## What happened

User reported broken UI on imagetravel.vn → visual screenshots of all
sites revealed a long-standing architectural debt: `templates/base_fr`
+ `base_es` hardcode aucoeurvietnam.com chrome (logo, ~150 menu items,
footer, flags, chaty, JSON-LD). Harvested pages (dmc/imagedmc front)
look fine only because they bypass templates; blueskies (tevily_child)
renders unstyled — traveltour CSS links 404; imagetravel wears
aucoeur's nav everywhere.

Scout findings beyond templates: `chrome.rs` literals (body SUFFIX,
og fallback, rappel-gratuit, author), `render_archive.rs` frozen
aucoeur wp_term_id map, `theme_traveltour` adapter seeds aucoeur
taxonomy+28 widget JSONs unconditionally, `intake.rs` FORM_SLUGS,
front-css absolute aucoeur URLs, orphaned `theme/base-minimal.html`.

Red-team caught 2 real blockers (per-lang `nav_menu_locations` vs
single-menu `theme_locations` PK; literal `{% extends "base_fr.html" %}`
in ~10 templates defeating theme-dir resolution) + 6 concerns —
plan amended: `theme_locations` gets lang column, extends becomes
ctx `base` var, custom multi-dir set_loader, host-guarded
strip_origin keeping absolute URLs for apply_navx, dead/orphan menu
items dropped not promoted, phase-06 added for real tevily bases
(blueskies harvest port).

## Output

`plans/261028-site-agnostic-chrome/` — plan.md + 6 phases, 8-9d:
01 dynamic nav · 02 chrome ctx · 03 theme separation · 04 site css ·
05 visual smoke gate · 06 tevily bases.

## Deferred (documented in plan)

templates/reference/* (verify fixtures), site_reviews_static content,
schema.rs fresh-site seeds, lang.rs VI translation table.
