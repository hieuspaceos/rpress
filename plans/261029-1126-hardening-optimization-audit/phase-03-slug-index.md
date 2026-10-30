---
title: "Phase 3: posts.slug index"
status: todo
---

# Phase 3: posts.slug index

## Overview

`posts.slug` has no index — `render__select_id_lang` (`WHERE slug=?1 AND
lang=?2`), markdown lookup, archive desc lookup, and menu-URL resolution
all full-scan ~1200+ posts per call. This is a large share of the measured
~140ms cache-miss render and ~350ms save path.

## Evidence

- `src/schema.rs` — `idx_posts_kind_status_lang(kind,status,lang)` exists;
  nothing leading on `slug`.
- `src/db/sql/render__select_id_lang.sql`, `render__select_markdown_id.sql`,
  `archive__select_desc_page.sql`, `intake__select_id_from_9.sql` —
  all `WHERE slug=?`.

## Requirements

- [ ] Slug equality queries use an index.

## Implementation Steps

1. Add migration in `schema.rs`: `CREATE INDEX IF NOT EXISTS
   idx_posts_slug_lang ON posts(slug, lang)` (slug,lang covers the hot
   selects; status is filtered in-row).
2. Optional: `idx_posts_status_pub ON posts(status, published_at)` for
   `publish_due` (`WHERE status='future' AND published_at<=now`) — cheap,
   pairs with phase-4 memoization.
3. `EXPLAIN QUERY PLAN render__select_id_lang.sql` before/after — confirm
   index use.

## Todo

- [ ] `idx_posts_slug_lang` migration
- [ ] optional `idx_posts_status_pub`
- [ ] measure: cold miss-render + `/admin` dashboard latency before/after

## Success Criteria

- `EXPLAIN` shows index scan on slug lookups.
- Cache-miss render measurably faster on aucoeur-test (1200 posts).

## Risk Assessment

- Write amplification on save — negligible at this DB size.
