---
phase: 4
title: "Import & dump correctness"
status: completed
priority: P1
effort: "3h"
dependencies: [1]
---

# Phase 4: Import & dump correctness

## Goal

Import can no longer destroy the live DB, silently drop wanted tables,
abort on one bad row, or corrupt latin1 content — the four silent-data
-loss paths the audit verified in `import.rs`/`sqldump.rs`.

## Files to Create / Modify

- Modify: `src/import.rs:15-29` — add `wp_users` to WANTED.
- Modify: `src/import.rs:97-109` — add canonicalize arms for
  `termmeta`, `rank_math_redirections`, `glsr_ratings`,
  `glsr_assigned_posts` (the `users` arm exists but is dead because
  `wp_users` is absent from WANTED — the previous item fixes that).
- Modify: `src/import.rs:113-117, 167-174, 264-283` — replace
  `v[N]` indexing with `.get()` + `bail!`/skip-with-warning.
- Modify: `src/import.rs:296, 785-795` — extend the posts dedupe
  pass to comments/menus/menu_items/media (or `INSERT OR REPLACE`
  with logged count).
- Modify: `src/import.rs:334-340` — stage to `db_path.import`,
  `rename` over `db_path` only after the import tx commits; clean up
  a stale `.import` on entry.
- Modify: `src/sqldump.rs:420-438` — `is_complete` tolerates
  trailing comments after `;`.
- Modify: `src/sqldump.rs:246` — `insert_header` skips `/* … */`
  comments like the pre-filter does.
- Modify: `src/sqldump.rs:36-40` — detect `SET NAMES latin1`/charset
  introducer-free latin1 dumps and transcode cp1252→UTF-8 (or bail
  loudly) so `s:N` lengths in php-serialized blobs stay correct.
- Fixture: `tests/fixtures/prefixed_dump.sql` (phase 1).

## Requirements

- [x] A `wp_42426_*` prefixed dump imports **all** wanted tables —
      termmeta, rank_math_redirections, glsr_ratings,
      glsr_assigned_posts — not silently discarded. Warn once per
      unmatched table so a future missing arm is loud.
- [x] `wp_users` rows stream → `users` map populated →
      `author_login` post_meta written (chrome/archive renderers at
      `render/chrome.rs:428`, `render_archive.rs:476` read it).
- [x] Import writes to `db_path.import`; on success `rename`s over
      `db_path` atomically. A killed/panicked import leaves the prior
      DB 100% intact.
- [x] Short/malformed INSERT rows produce a logged skip or a clean
      `bail!` with table+row context — never an indexing panic.
- [x] One duplicate wp_id in comments/menus/media no longer rolls
      back the whole import; dedupe parity with posts (or documented
      `INSERT OR REPLACE` + count).
- [x] `; -- comment` after a statement no longer swallows the next
      INSERT batch; `/* … */ INSERT` parses instead of hard-bailing.
- [x] cp1252 dumps without `_latin1` introducers either transcode or
      bail — no silent U+FFFD mojibake, and php-serialized `s:N`
      lengths stay byte-correct.

## Implementation Steps

1. Stage-then-rename first — it bounds the blast radius of every
   other import bug in this phase.
2. Extend the canonicalize match + WANTED; add an `unmatched` counter
   set that `eprintln!`s once per dropped table at end of import.
3. `.get(N)` guards with `bail!("{table} row too short: {} cols", v.len())`.
4. Comment-tolerant `is_complete` + `insert_header`; test with the
   mysqldump-style `/* … */ INSERT` and `; -- comment` cases.
5. Dedupe parity for comments/menus/media.
6. latin1 detection: scan the dump header for `SET NAMES latin1` or
   `character_set_client` and transcode string bodies.

## Verification

- `cargo test --test audit_remediation` — new tests:
  prefixed dump yields rows in all 4 previously-dropped tables;
  `author_login` meta present; `is_complete("; -- c")` behavior;
  killed-import leaves old DB (kill via panic injection or a staged
  file check in-process).
- Existing `sqldump.rs` (198 LOC of tests) + `import_expect.rs`
  + `widgets_import.rs` stay green.
