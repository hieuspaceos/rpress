---
phase: 7
title: "Performance, i18n & structural"
status: done
priority: P3
effort: "1-2d"
dependencies: [1, 4, 5]
---

# Phase 7: Performance, i18n & structural

## Goal

Cache-miss renders stop doing O(site) work; templates/regexes compile
once; blocking DB work leaves tokio worker threads; language machinery is
truly N-lang; duplicated helpers and dead weight are removed. Data decides
what gets cached — no speculative invalidation machinery.

## Findings covered

- **H7** — per-render whole-site materialization + N+1 (`render.rs:748-757,
  762-1290`)
- **H8** — per-render `Environment` rebuild + ~15 regex compiles/wpautop
- **H14** — blocking rusqlite/fs on tokio workers
- **M11** — hreflang/flag/is_front hardcoded fr/es
- **L** — `publish_due`/`session_touch` per need_auth incl. unauthed
  (moved from phase 5); Cargo `[profile.release]`; docs drift; cattest
  assertions; dead code/dup helpers; render.rs split

## Key insights

- **Scoped-down caching (red-team)**: the generation-counter design was
  rejected — it adds a second invalidation channel that post-saves never
  bump (bust_cache has 17 callers and post-save isn't one). Instead:
  (a) `media_map` → lazy lookup-by-id built only from ids the rendered
  post references; (b) cached minijinja `Environment` per templates dir
  in `OnceLock<RwLock>` invalidated on bust_cache/template edit;
  (c) posts/tours/nav data stays uncached unless the benchmark still
  shows it hot — if it does, any cache added MUST be keyed by
  (site, lang) since OnceLock caches are process-global under multi-site
  serve, and MUST be invalidated by `invalidate_dependents` too.
- Regexes: `static LazyLock<Regex>` — mechanical.
- spawn_blocking: scope to the heaviest paths (bulk actions, gdlr_pb save,
  page-builder ajax). Rewriting every handler is out of scope — WAL +
  busy_timeout makes the rest acceptable; document.
- render.rs split stays but is **the last commit of the plan, separate
  from all behavior changes** — it touches lines every earlier phase
  cites; landing it earlier would invalidate all plan line references and
  make the perf diffs unreviewable.
- i18n: replace `hreflang_fr/es` vars with `hreflang_links` generic;
  `is_front` via `front_id_for_lang`; `url_prefix` via `lang::url_prefix`.
- `publish_due` after cookie extraction (not before); `session_touch`
  only when last_seen older than 5 min.

## Files to modify

- `src/render.rs`, `src/render_post.rs`, `src/render_tour.rs`,
  `src/render_archive.rs`, `src/widgets.rs` — env caching, lazy media_map,
  LazyLock regexes, generic i18n vars; then the module split (last commit)
- `src/admin/posts.rs`, `src/admin/gdlr_pb.rs` — spawn_blocking on bulk
  loops + page-builder save
- `src/admin/mod.rs` — shared `pub(crate) fn urlenc` (kills the 7 copies
  incl. the buggy forms_editor one); publish_due/session_touch placement
- `src/admin/*.rs`, `src/db/*.rs` — dedup urlenc/slugify; delete
  `bin_mdcheck.rs`, `row_actions`, dead submissions/forms fns (verify
  unused first)
- `src/admin/dashboard.rs` — real module (delete include! hack);
  `src/lib.rs`/`src/main.rs` — single mod list
- `Cargo.toml` — `[profile.release] lto="thin", codegen-units=1, strip=true`
- `CMS-OPERATIONS.md`, `README.md` — css-serve correction, migrate --www
- `tests/cattest.rs` — real assertions

## Implementation steps

1. LazyLock regexes (wpautop + link fixups) — mechanical.
2. Cached `Environment` per templates dir; bust on bust_cache + template
   edit paths.
3. Lazy media_map (lookup by referenced ids only).
4. Benchmark: time `rpress generate` or 100 misses before/after steps
   1-3; record numbers. Only then decide if nav/posts caching is still
   needed — if yes, keyed (site, lang) + invalidated by
   invalidate_dependents.
5. spawn_blocking on the heavy paths.
6. Generic i18n vars; verify with an en/de-configured test site.
7. Dedup helpers + dead code + module hygiene (dashboard → module;
   single mod list).
8. Cargo release profile; docs fixes; cattest assertions.
9. LAST COMMIT: render.rs split along seams (mod/site_data/nodes/chrome/
   markdown) — pure moves, tests travel with code.

## Verification

- `cargo check` + `cargo test` clean.
- Benchmark delta recorded in phase notes (expect large drop from env +
  regex caching).
- en/de site: hreflang, flag URLs, is_front correct.
- `cargo build --release` size before/after.

## Todo

- [x] LazyLock regexes; cached Environment; lazy media_map
- [x] Benchmark + gated decision on nav/site_data caching
- [x] spawn_blocking heavy paths
- [x] Generic i18n vars
- [x] Dedup helpers; dead code; module hygiene
- [x] Release profile; docs; cattest assertions
- [x] render.rs split (final commit)

## Risks

- Cached Environment: must invalidate on admin template edits — wire into
  the same dirty/bust path or editors see stale templates.
- Split lands last and is pure moves — no behavior change; if it drags,
  ship the perf work without it.
