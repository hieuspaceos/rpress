---
phase: 3
title: "Import & dump correctness"
status: done
effort: "6h"
dependencies: []
---

# Phase 3: Import & dump correctness

## Goal

`import verify` passes on real dumps; `.sql.gz` end-to-end works; the SQL
dump lexer handles mysqldump/phpMyAdmin variants or fails loudly; the
uncommitted `import.rs` diff is completed and committed.

## Findings covered

- **H9** — `.sql.gz` copied/imported raw → empty DB (`site_build.rs:192-194`,
  `sqldump.rs:40-54`)
- **H10** — `import_expect` undercounts posts/media/menu_items
  (`import.rs:360,611,615-616,557-561,700-703`)
- **M3** — sqldump parser variants (unquoted INSERT, column lists, charset
  introducers, `\b`/`\f` escapes, latin1) (`sqldump.rs:54,132-136,158`)
- **M4** — `choose()` auto-picks option 1 on non-TTY (`site.rs:236-256`)
- **L** — verify `page_on_front` vs `show_on_front=posts` (`verify.rs:243`)
- **L** — menu lang fr/es hardcode (`import.rs:537,247-262`)
- **L** — whole-dump in-memory, dup post-ID abort (`import.rs:60-72,196-199`)
- `.sql.gz` contract (resolved): **phase 3 owns it entirely** — the import
  reader detects `.gz` magic bytes and decompresses via flate2. Phase 2
  does NOT touch pull-time decompression; `site_init`/`site_build` need
  no decompression code at all. This removes the cross-phase helper
  dependency and makes every caller correct by construction.

## Key insights

- Existing uncommitted diff already fixes `comments_total` and
  `translation_members` expect-counting; this phase extends the same
  "count only what can land" rule to posts/media/menu_items. Commit the
  completed work as one coherent change.
- The expect rules must mirror the INSERT filters — the clean fix is to
  compute expect *from* the same filtered iterators, not parallel logic.
- Charset: detect BOM/`SET NAMES`/CREATE TABLE charset header; if latin1,
  transcode per-column or bail with a clear message — silently importing
  mojibake is worse than refusing.

## Files to modify

- `src/import.rs` — expect-count filters mirror insert filters; menu lang
  inference from detected `languages`; dup-ID dedupe in idx
- `src/sqldump.rs` — accept unquoted `INSERT INTO ident`; parse+honor
  column lists (or loud bail); strip `_[a-z0-9]+`/`N'` introducers; correct
  `\b`/`\f` escape mapping; quote-aware `is_complete`; gzip magic-detect
  reader (`.sql.gz` transparent decompress via flate2 — this phase OWNS
  the contract; site_init/site_build need no gz code)
- `src/site.rs` — `choose()` non-TTY + >1 item → bail naming the flag
- `src/verify.rs` — `page_on_front` check conditional on `show_on_front`
- `Cargo.toml` — add `flate2` dep
- `tests/` — new `tests/sqldump.rs` + `tests/import_expect.rs` fixture tests

## Implementation steps

1. Commit-worthy cleanup: complete `import_expect` — posts counts over
   `publish|draft`; `media` counts only `status=='inherit'` with a file;
   `menu_items` counts only items with menu membership.
2. sqldump lexer: `INSERT INTO` table-name scan accepts ident with or
   without backticks; optional `(col,…)` list parsed → rows map by name,
   missing columns → NULL; unknown columns → bail.
3. Value introducers: `_charset'x'`/`N'x'` stripped at value start; escapes
   `\\b`→0x08, `\\f`→0x0C (not literal letters); `is_complete` tracks
   in-string state across buffered lines.
4. `.sql.gz` contract: `import::run` (or the reader) detects `.gz` magic
   bytes and decompresses via flate2 — then every caller is correct by
   construction. (Decide flate2 dep vs piping `gunzip`; flate2 is safer
   and kills a shell path entirely — also helps phase 2.)
5. `choose()`: `!tty() && items.len() > 1` → bail naming the disambiguating
   flag per call site (pass flag-name hint param).
6. verify: skip `page_on_front` check when `show_on_front == 'posts'`.
7. Menu lang: infer from slug suffix against `cfg.langs`; unknown → primary.
8. posts idx: dedupe by id (keep first, log skipped dup count to report).
9. Tests: fixture dump covering publish+draft+attachment counts, unquoted
   INSERT, column list, introducer, escapes, multi-line statement.

## Verification

- `cargo test` — new tests pass.
- Real-dump gate: `rpress import parse` + `import verify` on an existing
  dump (work/blueskies/rpress.db or a known dump) → zero MISMATCH.
- `rpress site build blueskies --only pull,import` succeeds against the
  existing env (verify no longer hard-fails).

## Todo

- [x] Complete import_expect parity (posts/media/menu_items)
- [x] sqldump: unquoted INSERT + column lists + introducers + escapes
- [x] quote-aware multi-line statement completion
- [x] .sql.gz transparent decompress in import reader
- [x] choose() non-TTY bail
- [x] verify page_on_front conditional
- [x] menu-lang inference; post-ID dedupe
- [x] tests/sqldump.rs + import fixture test

## Risks

- Column-list INSERTs: wp dumps normally don't emit them, but supporting
  them removes a whole silent-misimport class; if complexity balloons, loud
  bail is acceptable — silent positional mapping is not.
- flate2 adds a dep; alternative `gunzip -c` subprocess reintroduces the
  quoting problem — prefer flate2.
- Real-dump verification needs a dump on hand; blueskies work dir has one.
