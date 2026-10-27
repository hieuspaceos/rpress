---
title: "Seam phase 04: theme-as-data - taxonomy map + seeds moved to Data adapter"
date: 2026-10-27
summary: "CAT_TREE/etiquette/ES_ROOTS and all seed fixtures moved into theme-traveltour (Backend::Data) adapter; taxonomy_map seeds as plugin setting; byte-parity verified — zero diffs vs pre-04 output"
---

# Seam phase 04: theme-as-data

## What happened

`theme-traveltour` is now a registered `Backend::Data` adapter — the
traveltour theme's frozen topology lives in `settings`, not Rust.

- `taxonomy_map.json` generated from `CAT_TREE` (93 edges — verified
  count) + `es_roots` + `taxonomy_bases{post_tag: "etiquette"}`;
  `migrate()` seeds `plugin:theme-traveltour:taxonomy_map` with
  `INSERT OR IGNORE` (idempotent, runtime read-only — R9).
- `render_post.rs`: `CAT_TREE`, `cat_parent`, `cat_children`,
  `cat_path`, `cat_path_is_es`, `ES_ROOTS` deleted; readers moved to
  `theme_traveltour::{cat_path,cat_children,cat_path_is_es,tag_base,
  taxonomy_base_in}` reading the settings row with a compile-time
  fallback to the embedded map (identical content → behavior safe
  pre-seed). `breadcrumb_chain` + `term_canonical_path` gained `conn`.
- `"etiquette"` literals replaced by `tag_base(conn)` in render.rs,
  generate.rs, render_archive.rs, render_post.rs.
- Seed ownership: 29 JSON payloads + `SIDEBARS` + `SEED_WIDGETS` moved
  to `theme_traveltour/{seed/,widgets_theme.rs}`; core `SIDEBARS`/
  `SEED_WIDGETS` emptied — sidebar layout is theme-supplied. The
  tourmaster `tour_list` payloads (`custom_html-13/14`) moved into
  `tourmaster/seed/`.
- `import.rs` seeds the map post-commit via `adapter.migrate()`.
- Legacy row-rewrite UPDATEs (phase-2→3 upgrade guard) intentionally
  stay in `widgets.rs` — history, not theme data.

## Byte-parity

Baseline at `042f61b` (pre-04): `diff -r` over 1239 generated pages +
`redirects.map` = **identical**.

## Verified

- 68/68 tests, `cargo check --all-targets` 0 warnings.
- `plugin:theme-traveltour:taxonomy_map` row present (4452B).
- Live: `/guide-de-voyage-vietnam/idees-de-circuit/` 200,
  `/etiquette/art-martial/` 200, ES `/es/guia-de-viaje-a-vietnam/
  ideas-de-viajes/` 200, unknown tag 301→root (pre-existing semantics).

## Next

Phase 05 — back-compat & migration for existing sites (TOUR_BASE key
naming, settings migration path).
