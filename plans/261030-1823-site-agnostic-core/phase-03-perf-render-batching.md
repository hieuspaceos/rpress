---
title: "Phase 3: Render Batching + Generate Parallelism"
status: todo
priority: P1
---

# Phase 3: Render Batching + Generate Parallelism

## Overview

Kill the remaining N+1 query patterns on the cold-render path and make
`rpress generate` parallel + batched. Targets HIGH findings P3/P4 and the
generate-loop MEDIUMs.

## File Ownership

- `src/render/menu_url.rs` (hoist-only changes — deeper refactors in phase 5 stay out)
- `src/render/site_data.rs` (`img(fid)` batching only)
- `src/render.rs` (`media_map_for` reuse; only if needed for the batch call site)
- `src/generate.rs`
- `src/db/sql/` new batched query files (per layering rule)

## Requirements

- [ ] P3: `resolve_menu_url`/`resolve_tree` (`menu_url.rs:34-62`) — hoist `lang::config` and `page_on_front` out of per-item calls (compute once per menu build, pass in). Batch `post_by_wp_id`/`select_id_by_wp_id` lookups: collect all wp_ids first, one `WHERE wp_id IN (...)` query → HashMap, then resolve. 50-item menu: ~250 queries → ~4.
- [ ] P4: `img(fid)`→`media_lookup` N+1 (`site_data.rs:51-102`, also reviews ~295-300, pages feed) — collect fids across posts/reviews/pages, one `IN` query (`media_map_for` already implements this shape in render.rs:1040 — extract/reuse it), then attach.
- [ ] G1: generate loop (`generate.rs:87-92`) — batch `select_post_title` + `display_slug` upfront (one query returning all rows); batch `mod_stmt` lastmod fetches (generate.rs:162-165) similarly.
- [ ] G2: parallelize renders over a small worker pool (N=4 default or `min(4, cpus)`): each worker owns its own `Connection` (rusqlite `!Sync`); `SITE_DATA` cache is read-shared already — verify the cache key/locking holds under concurrency (it's behind a global map — check Mutex granularity). Harvest fetches parallelize too but cap concurrency to be polite to origin.
- [ ] L1: `rewrite_origin` (`render_post.rs:793-815`) — skip all 5 passes when none of the needle origins appears in the haystack (single `contains` check per needle first, or one `memchr`-style scan). Cache the `settings` lookup per render (it's a per-call query today).
- [ ] L2 (optional): `tex_run` `Vec<char>` alloc (render_post.rs:1281-1308) → char-index iteration; `toc_anchor` per-byte `format!` → pre-sized String + `write!`. Only if trivially safe.

## Implementation Steps

1. `menu_url.rs`: change `resolve_menu_url(conn, item)` signature to `resolve_menu_url(ctx: &MenuResolveCtx, item)` where ctx holds `{lang_cfg, front_page_id, wp_map}`. Update `resolve_tree` to build ctx once. New SQL: `menu_url__select_posts_by_wp_ids.sql` (IN query).
2. `site_data.rs`: two-pass — pass A collects `featured_id`s (posts, reviews, pages); pass B one `media_map_for`-style query; attach during emit. Verify zero behavior change (same alt/title fields).
3. `generate.rs`: upfront batch queries; then `std::thread::scope` + work queue (`std::sync::mpsc`) over per-worker conns, OR `rayon` (new dep — prefer no new dep; std threads suffice). Respect ordering: `harvested_pages` writes are per-post so any order works; `widgets_dirty` upsert at end.
4. `rewrite_origin`: read the 5 needles once (settings, cached on the conn/render ctx); `if !needles.iter().any(|n| html.contains(n)) { return html; }`.
5. Bench: `time ./target/release/rpress generate` on the largest reference site DB before/after; record in phase notes.

## Success Criteria

- Cold render of the heaviest reference page: menu build issues ≤5 queries total (log-count or `EXPLAIN` instrumentation — a debug `RUST_LOG`-style counter or temporary `rusqlite::trace` hook is fine, remove before merge).
- `generate` on the biggest fixture DB: measurable wall-clock improvement ≥2× on multi-core (or document if harvest-bound).
- `visual_smoke.sh` byte-parity holds — menu HTML and featured images identical.
- `cargo test` green.

## Risk Assessment

- Parallel generate + shared `SITE_DATA` cache: if the cache is a `Mutex<HashMap>` keyed (site_url,lang), contention is fine; if it mutates during render, serialize access — verify first, keep workers read-only on cache.
- IN-clause batch sizes: SQLite default `SQLITE_MAX_VARIABLE_NUMBER`=999 (bundled may differ) — chunk wp_id/fid lists at 500 to be safe.
- Ordering-sensitive output (feed/sitemap order) must not depend on worker completion order — collect + sort by the original post order.
