---
title: "Core/plugin seam — extract site-specific features"
description: "Make rpress core consistent for every site type (post/page/media/comments/menus/taxonomy); move tour, Tourmaster, GoodLayers, CF7/Flamingo intake, and reference-theme hardcodes behind an extension boundary. Runs AFTER 261027-0310-audit-remediation."
status: pending
priority: P2
effort: "1-2w"
tags: [architecture, plugins, core-boundary]
created: 2026-10-27
---

# Core/plugin seam

## Premise (user-confirmed 2026-10-27)

Core must be identical for every kind of site. Anything that exists
because a *specific* site's theme/plugins exist is an extension — even if
it ships in this repo.

- **Core**: posts (kind `post`/`page`), pages, media, comments, menus,
  widgets, taxonomy, users/sessions, import pipeline, generate/serve.
- **Extensions (per-site)**: `kind='tour'` + `tours` table + tour_base,
  `admin/plugins/tourmaster/`, `admin/gdlr_pb.rs` (2404 lines),
  `admin/contact.rs` + `intake.rs` + `submissions` (CF7/Flamingo parity),
  `admin/rankmath.rs`, `widgets` recent_tours, `seed/sidebar-tour-*`,
  `CAT_TREE`/`ES_ROOTS`/`etiquette` (reference-theme taxonomy map),
  `posts__insert_into_tours*.sql`, `tour__*.sql`, `archive__select_tours`.

## Depends on

`261027-0310-audit-remediation` — phase 4's tour_base threading collapses
~25 `voyage` literals into one config source; do the seam work on top of
that. Landing the seam first would re-create the same literals problem.

## Direction (to be shaped in brainstorm before phases)

- Kind registry: core knows `post|page` only; extensions register
  `{kind, url_base, renderer, importer hooks, admin screens, widget
  providers, seed data}`.
- `src/admin/plugins/` already exists → the boundary precedent is set;
  extend it to cover render/import/serve, not just admin.
- Open: same-binary static registry vs feature flags vs dynamic loading.
  Static registry + per-site `PLUGINS=` env line is likely sufficient
  (KISS — one binary, config chooses).
- `voyage`/`tour_base` become an extension-provided config key, not a
  core `SiteEnv` field.

## Scope notes

- CAT_TREE/ES_ROOTS/etiquette: audit phase 5 already extracts
  `term_out_paths` as a shared helper — this plan later makes the map
  itself site-supplied data.
- import.rs: importer must dispatch post-kind inserts through the
  registry instead of hardcoding `tours` table writes.
- This file is a placeholder contract, not a validated plan. Phase
  files come out of a brainstorm + scout pass on the post-audit tree.
