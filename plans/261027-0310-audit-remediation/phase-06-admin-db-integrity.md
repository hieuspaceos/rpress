---
phase: 6
title: "Admin & DB integrity"
status: pending
priority: P2
effort: "7h"
dependencies: [5]
---

# Phase 6: Admin & DB integrity

## Goal

Deletes are complete (no orphan credentials/terms/sessions); saves are
transactional; decode errors surface instead of silently dropping rows;
ids can't race; session/CSRF contract is explicit.

## Findings covered

- **H12** — user delete orphans app_passwords + in-memory sessions +
  misleading "delete all content" (`db/users.rs:106-109,350`,
  `admin/users.rs:1092-1097,1038`, `admin/mod.rs:562-600`)
- **H13** — delete_term leaves post_terms (`db/terms.rs:105-110`)
- **H15** — term+tax filter binds wrong column (`db/posts.rs:284-294`)
- **M7** — php_serialize negative-length abort + trailing bytes
- **M12** — `MAX(id)+1` id races — **7 sites enumerated (red-team)**:
  `posts.rs:1095`, `misc.rs:275,961`, `comments.rs:206`, plus
  `intake.rs:284,390,408` (intake is single-process offline import —
  decision: exempt intake, fix the 4 concurrent-capable sites, document
- **M12** — `MAX(id)+1` id races — **7 sites enumerated (red-team)**:
  `posts.rs:1095`, `misc.rs:275,961`, `comments.rs:206`, plus
  `intake.rs:284,390,408`. Validation decision: **fix all 7** — same
  `last_insert_rowid` pattern everywhere, removes the dangerous
  copy-pasteable pattern entirely.
- **M16** — forms_editor CSRF rehydration bypass + urlenc codepoint bug +
  slug unvalidated
- **M17** — cyclic reparent loop, **SOLE OWNER = this phase, TWO sites**:
  `db/comments.rs:221-226` (empty_status) AND `db/posts.rs:545-560`
  (30-day purge). Bound both (`for _ in 0..1000` + force-NULL fallback +
  `parent_id != id` guard).
- **M18** — missing indexes / post_meta dup keys — **migration-order
  fix (red-team Critical)**: `migrate_conn` runs `execute_batch(DDL)` at
  schema.rs:327-328 BEFORE any migration hook — a new unique index would
  fail on existing dbs with dup rows. Fix: add a pre-DDL dedupe step (or
  dedupe-then-index in one guarded tx); pick dup-keeper deterministically
  (get_meta is `LIMIT 1` no ORDER BY — match its likely winner: keep
  `MIN(rowid)`; document the behavior change). Choose unique index vs PK
  rebuild — prefer `CREATE UNIQUE INDEX` (no table rebuild).
- **M20** — session rotation + CSRF stability — **contract specified
  (red-team)**: persist csrf in the sessions row (additive column via the
  MIGRATIONS list, precedent `sessions.expires_at`) AND enumerate write
  path: login INSERT populates it, `users__session_get.sql` returns it,
  `get_sess` rehydrates it (mod.rs:565-583), `check_csrf` compares
  constant-time. **App-password trap**: transient Sess for Basic-auth is
  never inserted into app.sessions (auth.rs:249-252) → derive csrf =
  HMAC(server_key, user_id + app_pass_hash) for transient sessions, OR
  document a scoped exemption. Decide: HMAC derive for transient, persisted
  random for cookie sessions.
- **M21** — `gdlr_pb::url_decode` byte-slice panic + `parse_form_nested`
  unbounded recursion (moved here from phase 2)
- **L** — comment edit date unvalidated; widgets_order swallowed errors;
  Pager clamp; dead fns (row_actions, submissions set_status/purge,
  first_textbox_html, fdb::get — verify unused first)

## Key insights

- `app_pass_delete_user` exists but is `#[allow(dead_code)]` — wiring, not
  writing.
- `MAX(id)+1` → omit `id`, read `last_insert_rowid()`; `create_menu`
  already does this.
- `.flatten()` → `collect::<Result<Vec<_>,_>>()?` — mechanical; log any
  intentional skip.
- post_meta dedupe must run BEFORE `execute_batch(DDL)` — add a pre-DDL
  hook in `migrate_conn` (existing .db files may hold dups; work/*.db
  verify before deploy).
- Coordination with phase 5: phase 5 owns invalidation wiring INSIDE
  quick_update/bulk_apply handlers; this phase owns the DB-layer changes
  those handlers call (revision snapshot in crud fns, id assignment).

## Files to modify

- `src/admin/users.rs`, `src/db/users.rs` — delete cascade:
  drop_sessions_for + app_pass_delete_user; content handling per confirm
  screen (check what the form submits — reassign or fix copy)
- `src/db/terms.rs` — delete_term cascades post_terms in one tx
- `src/db/posts.rs`, `src/db/misc.rs`, `src/db/comments.rs`,
  `src/intake.rs` — tm.taxonomy fix; last_insert_rowid (7 sites);
  body_html COALESCE; flatten→collect; bounded reparent loops (2 sites)
- `src/admin/posts.rs` — quick_update/bulk revision snapshot at the crud
  layer; save_tour_option `unchecked_transaction`
- `src/php_serialize.rs` — length guards + trailing check
- `src/admin/forms_editor.rs` — need_auth csrf; shared byte-wise urlenc;
  slug validation
- `src/schema.rs` — pre-DDL dedupe hook + `CREATE UNIQUE INDEX` post_meta
  + `CREATE INDEX` comments(post_id), comments(parent_id),
  revisions(post_id), posts(kind,status)
- `src/admin/mod.rs`, `src/admin/auth.rs`, `src/db/sql/` — sessions csrf
  column + write/read path + constant-time compare + transient HMAC
- `src/admin/gdlr_pb.rs` — url_decode on bytes + recursion cap (8)
- `src/admin/comments.rs` — norm_datetime; `src/admin/widgets.rs` — `?`;
  `src/admin/list.rs` — Pager clamp; dead fns deletion

## Implementation steps

1. users_delete: `drop_sessions_for` + `app_pass_delete_user`; content:
   check confirm screen fields — implement reassign or honest copy.
2. delete_term: capture (taxonomy, slug), `DELETE FROM post_terms` in tx.
3. `AND t.taxonomy=?` → `AND tm.taxonomy=?`.
4. 4 inserters → `last_insert_rowid()`; intake exemption comment.
5. php_serialize: `n < 0 → bail`; `with_capacity` cap 1M; trailing-byte
   check.
6. body_html `COALESCE(NULLIF(?,''),body_html)`; revision snapshots in
   quick_update + bulk_apply crud fns.
7. `.flatten()` sweep → `collect::<Result<Vec<_>>>()?`.
8. save_tour_option tx wrap.
9. forms_editor 3 fixes.
10. schema: pre-DDL `DELETE` dup post_meta (keep MIN(rowid)) → then DDL
    adds unique + lookup indexes; verify on work/*.db.
11. Sessions: csrf column (MIGRATIONS), login INSERT, session_get cols,
    get_sess rehydrate, check_csrf constant-time; transient Sess →
    HMAC(server_key, user_id+pass_hash); login invalidates prior cookie
    session row.
12. gdlr_pb url_decode bytes + depth cap.
13. Small wins batch.

## Verification

- `cargo test` — php_serialize adversarial (`a:-1:`, `O:-1:`, trailing
  junk) → errors not aborts; reparent cycle fixture terminates.
- Schema migration on a copy of work/*.db with injected dup post_meta →
  migrates clean; `get_meta` stable.
- Login → restart → submit stored-csrf form → 200 (csrf persisted);
  app-password ajax POST → passes via derived csrf.
- Manual: delete user with app password → credential dead; delete term →
  post loses it; `?term=N&tax=mismatch` behaves; two parallel saves → no
  PK collision.

## Todo

- [ ] users_delete full cascade + honest content handling
- [ ] delete_term post_terms cascade; tm.taxonomy fix
- [ ] last_insert_rowid (4 sites) + intake exemption note
- [ ] php_serialize guards; reparent loops bounded (both sites)
- [ ] body_html COALESCE + quick_update/bulk revisions (crud layer)
- [ ] flatten→collect sweep; save_tour_option tx
- [ ] forms_editor 3 fixes; gdlr_pb decode + depth cap
- [ ] Schema: pre-DDL dedupe + unique post_meta + lookup indexes
- [ ] Sessions: persisted csrf + transient HMAC + rotation + const-time
- [ ] Small wins (dates, widgets, pager, dead code)

## Risks

- post_meta dedupe picks MIN(rowid) — may differ from the arbitrary
  LIMIT-1 winner some pages rendered before; acceptable, document.
- csrf column on existing sessions rows: NULL → rehydrate mints a fresh
  one once (same as today) then persists — no forced logout.
- `id == wp_id` assumptions: grep for `wp_id` usage post-import before
  dropping explicit id binding.
