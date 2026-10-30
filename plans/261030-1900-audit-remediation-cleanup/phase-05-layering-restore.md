---
title: "Phase 5 — Layering restore: plugins/ → root, serve.rs, SQL sweep"
phase: 5
status: pending
priority: P2
effort: 4h
files:
  - src/plugins/ (new — moved from src/admin/plugins/)
  - src/serve.rs (new)
  - src/admin/mod.rs
  - src/media.rs (new)
  - src/db/misc.rs
  - src/db/sql/*.sql
  - src/lib.rs
  - docs/architecture.md
---

# Phase 5 — Layering restore

Structural fix so the documented layer rules become true. Runs last —
phases 1–4 patched the code in its current location; this phase moves
it.

## Tasks

### 5.1 Plugin registry → `src/plugins/`

- `git mv src/admin/plugins src/plugins` (keep `plugins/specs/` at
  repo root untouched — that's the TOML input dir).
- The registry kernel (`PluginAdapter`, `RenderCtx`, `ADAPTERS`,
  `adapter_for_kind/slug`, `kind_url_base`, `screen_for`,
  `kind/section` duplicate asserts, `Backend`, `ScreenDef`) is
  cross-layer: admin screens, render, import, generate, widgets all
  consume it. After the move, dependency direction becomes
  `admin → plugins` and `render/intake/generate → plugins` — both
  legal.
- Adapter internals that use admin helpers (`chrome`, `esc`,
  `html_admin`) keep importing `crate::admin::*` — that's allowed:
  the rule governs *modules' layer membership*, and adapter code is
  explicitly an admin-surface plugin. The kernel itself must not
  import `admin::`.
- Mechanical: fix ~all `use crate::admin::plugins::` paths →
  `crate::plugins::` (grep count first — expect 30-50 sites).

### 5.2 Public serve path → `src/serve.rs`

Move out of `admin/mod.rs` (~700 LOC):

- `public_root`, `public_page`, `read_out`/`read_out_async`,
  `write_cache`, `md_for_slug`, `asset_redirect_target`,
  `redirect_prefix`, `is_junk_prefix`, `front_css`, `wp_asset`,
  `uploads` serving, `publish_due` + timer task from phase 1.
- `admin/mod.rs` keeps: `serve()` router (imports the serve fns for
  the public routes), `App`/`Sess`/`SiteCtx`, session engine
  (`get_sess`, `need_auth`, CSRF), chrome/menu helpers.
- `need_auth` stays in `admin/mod.rs` or moves to a small
  `src/session.rs` — decide by what `serve.rs` needs (preview
  session-gating uses sessions). Prefer `session.rs` if it cleanly
  holds `Sess` + `get_sess` + `need_auth` + CSRF helpers; otherwise
  leave in `admin/mod.rs` and let `serve.rs` import it.

### 5.3 SQL sweep → `db/sql/*.sql`

- Move every inline statement outside `src/db/` into named
  `db/sql/*.sql` files + `sql!()` calls. ~80 sites total:
  `admin/` ~30 (dashboard counts, rankmath, widgets, posts helpers,
  auth session_touch…), render*/generate/sitefx/intake ~50.
- Handler-facing reads become `db::` functions where they're more
  than a one-off lookup (dashboard count queries →
  `db::dashboard_counts`).
- Delete `db::ids_matching(db, ids_sql, params)` — callers get typed
  helpers (`ids_of_kind_status`).
- Gate: `grep -rn '"\s*SELECT\|"\s*INSERT\|"\s*UPDATE\|"\s*DELETE'
  src/ --include='*.rs' | grep -v 'src/db/'` returns only
  schema/migration files and tests. Commit this grep as
  `scripts/check-layers.sh` (also asserts no `crate::admin::plugins::`
  remains outside `src/plugins/` adapters).

### 5.4 `db/misc.rs` → `src/media.rs`

- fs/image code (`save_upload`, `media_replace_upload`,
  `media_regenerate_sizes`, `write_image_sizes`, `served_upload_mime`,
  filename sanitize) moves to `src/media.rs`; SQL-only helpers stay in
  `db/misc.rs`. Callers flip imports.

### 5.5 Dead-weight prune

- `App.sites` is always 1-entry: keep the map (cheap) but delete the
  unreachable `host_known` multi-site 421 branch + comment.
- `comments.rs:1186` stale `#[allow(dead_code)]` — remove.
- `import.rs` ~1.3k LOC widget conversion → `src/import_widgets.rs`
  (mechanical move; `import.rs` calls it).
- `verify.rs` unused legacy checks if any surface during 3.x.

### 5.6 Doc resync — `docs/architecture.md`

- Update the layer table to the real graph:
  `admin → {db, plugins, intake, sitefx, render, serve, session}`;
  `serve/render/intake → {db, plugins, session?}`; `plugins → db`
  (+ `admin` for adapter screen helpers — document the exception);
  `db → nothing`; `media → db`.
- Fix the `/api/v1/posts` doc (returns `{"count":N}`, not a list).

## Acceptance

- `cargo build && cargo test` green; `e2e-site.sh … all` green;
  `visual_smoke.sh` green — pure refactor must produce byte-identical
  renders (the smoke script's baseline asserts this).
- `scripts/check-layers.sh` passes and is the committed proof the
  invariants hold.
