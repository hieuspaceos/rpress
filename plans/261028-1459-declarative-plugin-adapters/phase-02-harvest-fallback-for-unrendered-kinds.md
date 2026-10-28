---
phase: 2
title: "Harvest-at-import for unrendered spec kinds"
status: pending
priority: P1
effort: "1d"
dependencies: [1]
---

# Phase 2: Harvest-at-import for unrendered spec kinds

## Goal

A spec kind with `harvest = true` gets origin-rendered HTML fetched
**once at import** and stored in the DB — generate emits it verbatim,
no per-generate fetch, no Rust render, no template.

## Current state (verified)

- `harvest_posts(post_id)` exists (`src/import.rs:490`); populated only
  when `p.meta` has `_elementor_data` (`src/import.rs:557`).
- Generate harvests `harvest_posts` ids from `settings.legacy_origin`
  (`src/generate.rs:64-116`); failure falls back to `render_slug`
  (bare body — never blank).
- `harvest_page` rewrites origin asset URLs to `site_url` already.

## Changes — harvest at import, emit at generate

Design decision (user, 2026-10-28): the WP origin must be live **only
during `import`** — rpress ports once, then the old site can go away.
Harvested HTML therefore lives in the DB, not fetched per-generate.

1. **New table `harvested_pages(post_id INTEGER PRIMARY KEY, html TEXT,
   fetched_at TEXT)`** created in `src/import.rs` next to the existing
   `harvest_posts` batch (`src/import.rs:490`). Distinct semantics:
   `harvest_posts` = fetch-at-generate ids (elementor path, unchanged);
   `harvested_pages` = fetch-at-import stored HTML.
2. **Import-side fetch**: `legacy_origin` is written inside the tx at
   `src/import.rs:867-872` from the already-loaded `options["siteurl"]`
   map — so resolve the origin from `options` directly, not the
   `settings` table. Fetch loop runs **after `tx.commit()`
   (`src/import.rs:1096`)**: collect `(post_id, origin_url)` pairs for
   spec-harvest kinds during the posts pass, curl each, then a second
   short `INSERT OR REPLACE` tx writes `harvested_pages`. Never hold
   the write tx across HTTP (dozens of 30s-timeout fetches).
   Reuse `harvest_page` (`src/generate.rs:698`) — move it (scheme
   default + origin→local rewrite) to `src/harvest.rs` imported by both
   generate and import.
   - Fetch failures: count + log like `harvest_failed`; post still
     imports (falls to bare-body render later).
   - Origin URL shape: `{origin}/{url_base}/{wp-slug}/` — `url_base`
     from spec (`to_book` → `booking`); if a future stack's rpress
     `url_base` differs from the WP CPT slug, spec gains an explicit
     `origin_base` field (default = `url_base`).
3. **`src/generate.rs`**: at the existing harvest-set build
   (`src/generate.rs:68`), also `SELECT post_id FROM harvested_pages`;
   in the render branch (`src/generate.rs:101`) emit stored `html`
   directly — no fetch for those ids. `harvested_pages` takes
   precedence over `harvest_posts` if a post were in both.
4. **`src/admin/plugins/mod.rs`**: `spec_harvest_kinds()` — derived from
   the same parsed specs (no second parse pass).
5. **`slug_from_title: bool`** spec field: default false; to_book keeps
   default (stored slugs are real). Don't port tourmaster's
   title-derived slug logic into specs until needed (YAGNI).
6. **Origin absent at import** (`options["siteurl"]` missing/empty or
   site already dead): no fetch → no `harvested_pages` rows → generate
   emits bare body via `render_slug_by_id_public`. Rows from a prior
   successful import survive (`INSERT OR REPLACE` only on success).

## Edge cases

- Post has `harvested_pages` row AND adapter `render()` returns `Some`:
  stored harvest wins (generate checks `harvested_pages` first) — spec
  load rejects `harvest=true` on a kind a Rust adapter already owns
  (contradictory config, fail at startup).
- Origin 404s a single page at import: post still imports; row absent;
  generate emits bare body — acceptable degradation, counted in import
  report.
- Draft posts: only publish rows reach the outer generate loop —
  `harvested_pages` rows for drafts are inert.
- Re-import on a live origin refreshes rows (INSERT OR REPLACE);
  re-import with dead origin leaves stale-but-valid markup — matches
  "port once" semantics.

## Verification

- Re-import imagedmc; `sqlite3 work/imagedmc/rpress.db "SELECT count(*) FROM harvested_pages"` ≥ to_book count;
  `scripts/e2e-site.sh sites/imagedmc.env generate` (origin can be
  unreachable — stub `legacy_origin` in a test site to prove it):
  `booking/` pages still emit full markup (grep `tevily`/`gsc-booking`).
- Diff harvested `booking/<slug>/index.html` vs
  `curl https://imagetraveldmc.com/booking/<slug>/` normalized —
  class-overlap parity gate passes.
- aucoeur: `harvested_pages` empty (no spec kind); generate log
  unchanged vs baseline.