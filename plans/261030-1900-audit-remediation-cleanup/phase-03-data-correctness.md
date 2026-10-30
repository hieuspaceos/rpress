---
title: "Phase 3 — Data-layer correctness: schema, tx, WAL, import safety"
phase: 3
status: pending
priority: P1
effort: 3h
files:
  - src/schema.rs
  - src/db/mod.rs
  - src/db/posts.rs
  - src/import.rs
  - src/verify.rs
  - tests/import_expect.rs
  - tests/hardening.rs
---

# Phase 3 — Data-layer correctness

Schema-level bugs that silently corrupt or lose data.

## Tasks

### 3.1 `post_meta` dual key — `schema.rs:214,268`, `import.rs`, `pull.rs`

- Decide and document: keep `UNIQUE(post_id,meta_key)` (matches
  WP's `get_post_meta` scalar semantics) — the wide PK
  `(post_id,meta_key,meta_value)` column list is redundant; at the
  next table rebuild drop to `PRIMARY KEY(post_id,meta_key)`.
  New DBs get the clean DDL; existing DBs keep both (harmless —
  strict index wins) — record the choice in schema comments.
- Import: multi-value meta rows collapse deterministically —
  last-value-wins per `(post_id,meta_key)` (WP's `get_post_meta($id,
  $key, true)` returns the first row, but our consumers treat meta as
  scalar map; WP write order makes *last insert* the value WP shows
  for singular reads — verify against a real dump which matches
  parity), counted into `dropped_meta` report as today.
- `verify.rs` gains a check: `post_meta` has no `(post_id,meta_key)`
  duplicates (trivially true with UNIQUE index — make the check the
  import-time collapse count instead: assert report.dropped_meta ==
  actual collapsed rows).

### 3.2 Import destructiveness — `import.rs:308`

- `remove_file(db_path)` → remove `db`, `db-wal`, `db-shm` together.
- Add a guard: if the existing DB has `users` with >0 rows
  (a lived-in DB — users/sessions/submissions are runtime state),
  `rpress import parse`/`migrate` refuse unless `--force`; print
  exactly what will be destroyed. `site_build`'s import phase already
  deletes db intentionally — it passes `--force` internally.
- Regression `tests/import_expect.rs` (or hardening): seeded db with
  a user → `rpress import` without `--force` exits non-zero, db
  untouched; with `--force` succeeds.

### 3.3 FK pragma — `src/db/mod.rs`, `schema.rs`

- Audit first: `grep` every DELETE path for tables with declared FK
  (only `theme_locations.menu_id` today). `delete_menu` must
  explicitly `UPDATE theme_locations SET menu_id=NULL` (or rely on
  ON DELETE SET NULL once pragma is on — it will then work).
- Enable `PRAGMA foreign_keys = ON` in `conn()` **after** the audit;
  add an integrity-check row in `verify.rs` for orphan
  `theme_locations.menu_id`.
- Risk: pragma is per-connection — `conn()` is the only opener, so
  coverage is total. Any write path caught violating gets fixed, not
  the pragma off.

### 3.4 Transactional migrations — `schema.rs:352-404`

- Wrap `theme_locations` rebuild + the `MIGRATIONS` ALTER loop in a
  single tx (`conn.transaction()`); DDL in `execute_batch(DDL)` stays
  outside (it's idempotent CREATE IF NOT EXISTS).
- A failed mid-rebuild must leave the DB in either old or new shape —
  tx guarantees it.

### 3.5 Translation model — `db/posts.rs:1132-1204`

- `sync_translation_members` hardcodes `'fr'`: parameterize by
  `f.lang` (the post being saved) instead of assuming fr; the "evict
  only fr slot" logic becomes "evict this post's lang slot + partner".
- Save-side `post_translations` write gated on `f.lang == "fr"` stays
  (legacy two-lang pairing is fr-anchored by design — README says
  admin edit screen is legacy two-language) **but** add a comment
  naming the invariant and a `debug_assert` on primary lang == "fr"
  so a non-fr-primary site fails loudly at dev time, not by silently
  writing wrong rows.
- `verify.rs`: add orphan check for `post_translation_members`
  (member → posts id must resolve) — currently only
  `post_translations` is checked.

### 3.6 `verify.rs` format! SQL — `verify.rs:194-197`

- `format!("SELECT COUNT(*) ... kind='{kind}' AND lang='{lang}'")` →
  bound params. Also audit the same file for sibling splices.

## Acceptance

- `cargo test` green; import of a real dump re-run produces identical
  `import_expect` report (deterministic collapse proves no behavior
  drift on normal data).
- `rpress verify` passes on the reference DB with new checks.
- `PRAGMA foreign_keys` on, no test/regression failures.
