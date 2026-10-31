---
phase: 5
title: "Data correctness"
status: completed
priority: P1
effort: "2h"
dependencies: [1]
---

# Phase 5: Data correctness

## Goal

Kill the session-wipe type bug, the non-transactional term rename,
the timezone-broken scheduler, the stale-translation-cache window,
the backup brick, and the remaining `.flatten()` row swallows.

## Files to Create / Modify

- Modify: `src/db/sql/sessions__delete_expired.sql:1` — compare
  against `unixepoch()`, not `datetime('now')`.
- Modify: `src/db/terms.rs:47-89` — wrap `save_term`'s two UPDATEs in
  `c.transaction()`.
- Modify: `src/db/posts.rs:1392-1429` + `src/admin/posts.rs:1405` —
  `published_at` timezone handling.
- Modify: `src/generate.rs:449-528` — `invalidate_dependents` must
  also delete `post_file_variants` of `post_translation_members`
  siblings.
- Modify: `src/backup.rs:30-33` — remove stale `.snapshot.db` (or use
  unique name) before `VACUUM INTO`.
- Modify: `src/db/posts.rs:168,255,1077`, `users.rs:378`,
  `terms.rs:43`, `misc.rs:256`, `comments.rs:242`,
  `menus.rs:413,525,548` — replace `rows.flatten()` with
  `collect::<Result<Vec<_>>>()` or a logged `filter_map`.
- Modify: `src/db/media.rs:94-140` — `save_upload` magic-byte gate.

## Requirements

- [x] `sessions__delete_expired` uses `expires_at > 0 AND
      expires_at < unixepoch()` — live sessions survive the sweep.
      `sweep_sessions` (mod.rs:585) runs on the serve() tick and today
      deletes every session row, live or not: INTEGER `expires_at` <
      TEXT `datetime('now')` is always true under SQLite type ordering;
      `users__session_put` writes `unixepoch()+43200`. Prefer reusing
      the already-correct `sitefx__delete_expired_sessions.sql` (one
      statement, two names) if the macro allows.
- [x] `save_term` terms UPDATE + post_terms rename cascade run in one
      transaction; a failure between them is impossible.
- [x] Scheduled publish: `published_at` stored in UTC — the
      `datetime-local` site-local wall time is converted to UTC at save
      via `settings.timezone` (IANA name resolved through the host `date`
      binary, chrono-free per project convention) and back to site-local
      in the publish metabox. Deviation from WP site-local storage
      documented in `save_post_on`.
- [x] Editing post A's slug/status invalidates cached pages of every
      `post_translation_members` sibling — no stale hreflang/permalink.
- [x] `run_backup` removes `.snapshot.db` before `VACUUM INTO`; a prior
      crashed backup can't brick all later runs.
- [x] Zero `.flatten()` row-error swallows in `src/db/` — grep returns
      nothing; each site either `?`-propagates or `eprintln!`s the
      skipped row.
- [x] `save_upload` calls `mime_from_bytes` and rejects bytes whose
      sniffed mime differs from the extension's mime (stricter than
      family match — a PNG in a .jpg is rejected).

## Implementation Steps

1. One-line SQL fix + regression test: insert a live session row,
   run the sweep statement, assert the row survives.
2. `save_term`: `let mut c`, `let tx = c.transaction()?`, move both
   executes under it, `tx.commit()`.
3. Timezone: `settings.timezone` exists (`settings.rs:79`, default
   `Europe/Paris`) — store `published_at` in UTC: convert
   `datetime-local` → UTC at save using that setting. WP parity is
   site-local storage, but UTC storage + convert-at-display is the
   boring correct option here; document the choice.
4. `invalidate_dependents`: query siblings, delete their variants.
5. Backup + media + `.flatten()` sweep.

## Verification

- New test: live session survives `sessions__delete_expired`.
- `save_term` slug-rename test: crash-injection is overkill — assert
  both writes share one `tx` by code inspection + a rename
  integration test.
- Scheduled post: insert `published_at = local-now+1h` → does not
  publish until the wall time passes (unit test on `effective_status`
  with an injected `now`).
- `cargo test` green.
