---
phase: 5
title: "Cache & regen consistency"
status: done
effort: "6h"
dependencies: [4]
---

# Phase 5: Cache & regen consistency

## Goal

Every mutation that changes public output invalidates/regenerates all
affected artifacts: post file, `.md` mirror, every alias-path variant,
term archives (incl. nested), homepage, feeds, sitemap — lazily, without
whole-cache busts.

## Findings covered

- **H4** — post save regenerates only own file; archives/homepage/feeds/
  sitemap stale indefinitely
- **H5** — `index.md` mirror never regenerated/removed → divergence AND
  public exposure of trashed content (file-first serve path)
- **H6** — deterministic `index.tmp` races; tmp orphans survive bust
- **M22** — cache keyed on request path not canonical rel → stale alias
  copies forever
- **NEW (red-team)** — regen_post non-publish branch removes only
  index.html (generate.rs:350-357); `remove_file_for` same gap; alias
  variants (`{clean}.html`, `{lang}/` dirs, `index.md`) all survive
  unpublish
- **L** — `empty_trash` ordering (redo correctly: snapshot rels → DB
  delete → remove files w/ error propagation + retry + bust fallback);
  `restore_revision` TOCTOU; `media_replace_upload` fs-before-DB

## Key insights

- **Funnel wiring (red-team)**: don't enumerate regen callers —
  `regen_posts` itself (admin/admin.rs:1070) is the single funnel; wire
  invalidate_dependents INSIDE it so all 20+ callers (posts, comments,
  contact.rs:723/767/799, future) are covered by construction.
- **Archive path mapping is NOT derivable from post_terms+terms**: category
  paths come from `CAT_TREE` (hardcoded static map, render_post.rs:525-548)
  via `cat_path()`; `post_tag` renders at `etiquette/{slug}`; other
  taxonomies at `{prefix}/{tax}/{slug}`; ES detection uses `ES_ROOTS` +
  `-es` heuristics (render_post.rs:577-583). The real dispatch lives in
  generate.rs:224-258 — extract it into a shared
  `term_out_paths(conn, cfg, tax, slug) -> Vec<PathBuf>` used by BOTH
  generate and invalidate_dependents. Flag CAT_TREE/ES_ROOTS as
  site-specific debt to document.
- **Delete-vs-late-write race**: a request that began rendering before
  invalidation completes re-caches stale output after the delete.
  Mitigate: run invalidation AFTER the save tx commits; add a second
  invalidate sweep after regen completes; document the residual window
  (bounded by render duration).
- **Cacheable candidate enumeration** (from phase 1 step 10): for a post
  at canonical rel R, public_page can have written `R/index.html`,
  `R/index.md`, `{R}.html` (clean.html variant), plus alias-dir copies
  under every `/{lang}/` prefix. Invalidation/removal must cover all of
  them.
- empty_trash redo: `remove_file_for` needs DB rows for path mapping —
  so: snapshot doomed id→rel list FIRST, then `empty_trash` DB delete,
  then remove files with real error propagation (not `let _ =`), retry
  once, and `bust_cache` fallback if files remain.

## Files to modify

- `src/generate.rs` — `regen_post`: non-publish branch removes index.md
  too; publish branch rewrites index.md; `remove_file_for` removes all
  alias variants; extract `term_out_paths` from lines 224-258 shared by
  generate + invalidate_dependents
- `src/admin/admin.rs` — `invalidate_dependents` wired inside
  `regen_posts` (the funnel); bust_cache_dir ext list + `tmp`
- `src/admin/posts.rs` — post-commit invalidation ordering; empty_trash
  snapshot→delete→remove(+retry)+bust-fallback; quick_update/bulk_apply
  call sites (coordination: phase 6 edits the same functions' DB layer —
  phase 5 owns the invalidation wiring INSIDE the handlers)
- `src/admin/comments.rs` — comment transitions go through regen_posts
  (already do — verify dependents fire)
- `src/admin/mod.rs` — `write_cache` unique tmp name + rename errors
  logged; cache-write only when path == canonical rel (else 301)
- `src/db/posts.rs` — restore_revision single-connection SELECT
- `src/db/misc.rs` — media_replace: sizes computed before fs::write;
  stale sizes unlinked

## Implementation steps

1. `term_out_paths` shared helper extracted from generate.rs:224-258 —
   same taxonomy/lang/nesting rules render_archive produces (CAT_TREE,
   etiquette, ES_ROOTS, per-lang prefixes).
2. `invalidate_dependents(conn, sc, post_id)`: term archives via
   `term_out_paths` for each post_terms row + homepage `index.html` +
   feeds + `sitemap*.xml` + post alias variants (index.md, {clean}.html,
   `/{lang}/` copies). Delete, don't render.
3. Wire into `regen_posts` (the funnel) — post-commit ordering.
4. `regen_post`: publish → write index.html + index.md (post_markdown);
   non-publish → remove index.html AND index.md AND alias variants.
   `remove_file_for`: same removal set.
5. `write_cache`: tmp name `{stem}.{pid}.{counter}.tmp`; log rename
   errors; write only for canonical rel else 301.
6. `bust_cache_dir` ext list + `tmp`.
7. empty_trash ordering per Key insights.
8. restore_revision: SELECT inside the tx. media_replace ordering.
9. Post-regen second invalidate sweep + residual-window doc comment.

## Verification

- `cargo test`; manual: edit post → its term archive index (incl. a
  NESTED category) deleted → fresh on next hit; comment approve → post +
  .md regenerated; trash post → /slug/, /slug.md, /{lang}/slug/, 
  {slug}.html all 404.
- Concurrency: `ab -n 50 -c 5` on one missing slug → consistent output,
  no `.tmp` orphans; file removed then written by a racing miss doesn't
  resurrect stale content past the documented window.

## Todo

- [x] term_out_paths shared helper (CAT_TREE/etiquette/ES_ROOTS aware)
- [x] invalidate_dependents inside regen_posts funnel (covers contact.rs)
- [x] regen_post + remove_file_for full removal set (md + alias variants)
- [x] write_cache unique tmp + canonical-rel gate + 301 aliases
- [x] bust ext list; empty_trash ordering; restore_revision/media ordering
- [x] Post-commit invalidation + post-regen sweep + window doc

## Risks

- term_out_paths reuse: CAT_TREE/ES_ROOTS are reference-theme-specific —
  document them; a theme swap must update the map (same as render today).
- The delete-vs-late-write window is bounded but real; the second sweep
  shrinks it; eliminating it entirely needs a per-path render lock —
  deferred (noted, not built).
