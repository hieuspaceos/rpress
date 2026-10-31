---
title: "Audit remediation + layering cleanup"
description: "Fix every verified audit finding (security, correctness, dead code) then restore the documented layering: plugin registry out of admin/, serve path out of mod.rs, SQL back into db/. Every fix lands with a regression gate."
status: completed
priority: P0
effort: 16h
branch: main
tags: [bugfix, security, refactor, audit]
blockedBy: []
blocks: []
created: 2026-10-30
---

# Audit remediation + layering cleanup

## Overview

Three-agent audit of the codebase (48.7k LOC) verified 6 high, ~15
medium findings plus systemic layering drift. This plan fixes **all
verified findings** and then removes the structural causes so the next
audit finds fewer of them.

Ordering principle: **blast radius first** — deploy/backup secret leaks
and DoS holes before code moves; structural refactor last, after tests
exist to prove behavior didn't change.

Rule for every fix: **behavioral fix + one regression test or script
gate proving it**, then cleanup. No fix ships as "looks right".

## Current state (verified evidence)

Every item below was re-read in source, not trusted from agent output.

### High

- `src/site.rs` `SiteEnv::save()` (~L115-140) never writes `SITE_URL`
  though `load()` reads it at :103 → `site_deploy.rs:61`
  `domain == env.site_url` compares against `""` → the
  "refuses domain == source origin" check is dead for every env written
  by `site init`.
- `src/admin/bridge/php_cgi.rs:236-239,458-469` — `sandboxed()` returns
  `None` without `/usr/bin/sandbox-exec` (Linux = the vps deploy
  target) → PHP spawns unconfined (full fs + network). The comment
  "confined to loopback + token" is wrong: nothing restricts network.
- `src/site_deploy.rs:343-349,358-378` — `dest.push(&work.join("www"))`
  rsyncs `www/` verbatim (no `--exclude`); `www/bridge/` on disk holds
  `bridge-token`, `docroot/wp-config.php` (NONCE_KEY), `.secret-<slug>`
  → secrets land on the public static docroot. Confirmed artifacts exist
  under `work/*/www/bridge/`.
- `src/admin/mod.rs:604-621` — `.md` miss path runs `md_for_slug` (new
  db conn + full markdown conversion + `write_cache`) **before** the
  `miss_rate` limiter at :633-642 → unthrottled render DoS.
- `src/render_post.rs:1393-1394` `heading_text()` decodes `&lt;`/`&gt;`
  then `tex_run` (:1300-1310) passes `<`/`>` through raw into TOC
  `<a>` text (:1461,1478,1489) → stored XSS via crafted heading.
- `src/admin/mod.rs:1057,1127` — `publish_due` only runs inside
  `get_sess`; public paths never call it → scheduled posts never
  publish without admin traffic (self-acknowledged in comment, still a
  bug for near-static sites).

### Medium (spot-verified)

- `src/admin/posts.rs:107-115` `qs()` splices `?tax=` into `href="{}"
  unescaped → reflected XSS.
- `src/admin/bridge/pull.rs:134-192` + `mod.rs:1734` — `let _ = pull()`
  swallows errors; `COALESCE(wp_id→id, ?1)` fallback can stamp meta on
  wrong post; DELETE+INSERT (not OR REPLACE) crashes pull tx on
  multi-value meta.
- `src/site_bridge.rs:81-89` — `kill` joins unvalidated `slug` into
  `remove_dir_all` → `rpress bridge kill --slug ../..` deletes outside
  `www/bridge`.
- `src/site_deploy.rs:240` copies `rpress.db` without `-wal`;
  `import.rs:308` deletes db but not `-wal/-shm` → stale WAL replays
  into a fresh DB / deployed DB misses uncommitted pages.
- `src/import.rs:308` — unconditional `remove_file(db_path)` destroys
  users/sessions/submissions on re-import, no guard.
- `src/admin/bridge/php_cgi.rs:484+` — `install_php` format!-splices
  `{theme}/{token}/{docroot}/{fqdb}` into PHP single-quoted literals
  unescaped (wp_config() uses `php_str()` correctly); `theme` is
  dump-derived → PHP code injection in trusted install step.
- `src/backup.rs:29-54` — `tar -chzf` dereferences bridge symlink farm
  → backups duplicate plugins/themes/uploads (~40MB WP core) and carry
  `.secret-*`; admin page doesn't warn that rpress.db holds live
  session tokens/TOTP secrets.
- `src/admin/mod.rs:1639` — `plugin_screen` gates `role=="admin"` for
  every adapter screen, but tourmaster's own metabox ajax
  (`/admin/plugins/tourmaster/save-option`) posts from the tour *edit*
  screen served to editors → 403, broken editor UX.
- `src/schema.rs:352-404` — migrations run `execute_batch` outside a
  tx; `PRAGMA foreign_keys` never enabled → the only declared FK
  (`theme_locations.menu_id`) is inert → dangling rows.
- `src/schema.rs:214,268` — `post_meta` has PK `(post_id,meta_key,
  meta_value)` **and** UNIQUE `(post_id,meta_key)` → WP multi-value
  metas collapse silently at import; pull DELETE+INSERT fails mid-tx.
- `src/render/site_data.rs:118,330-385` — `site_reviews` N+1 payload
  computed per cold render, zero consumers;
  `src/render/shortcode.rs:81` hardcodes `post_id=6190`.
- `src/db/sql/render__select_id_lang.sql` — `?3='' OR kind=?3` with
  `kind=""` (mod.rs:654) matches ANY published kind → `ml-slide`,
  `nav_menu_item` render publicly on a guessable slug.
- `src/css_scope.rs:29` — `out.push(b[i] as char)` mangles non-ASCII
  CSS to mojibake; `;`-terminated `@import` never reaches the drop arm
  (:74-75 precedes :97-113).
- `src/db/posts.rs:1132-1204` — `sync_translation_members` hardcodes
  `'fr'`; write path assumes primary-lang is fr.

### Layering drift (doc vs code)

- Doc: "SQL lives only in src/db/". Reality: ~30 inline statements in
  `admin/`, ~50 across render*/generate/sitefx/intake; `db::ids_matching`
  takes a raw SQL fragment from callers.
- Doc: "db may import nothing". Reality: `db/posts.rs` calls
  `admin::rankmath::save_conn`, `admin::plugins::{editor_for,
  adapter_for_kind}` inside its tx; `db/misc.rs` (1.2k) is a media
  service (fs, image decode).
- Doc: "render may import db". Reality: `render*.rs` call
  `crate::admin::plugins::*` ~15 sites — the plugin registry is a
  cross-layer kernel misplaced under `admin/`.
- `admin/mod.rs` ~2k LOC carries the entire public serve path
  (~700 LOC) + session engine (~300) + chrome (~200) — public serving
  isn't admin.

### Low debt

`AutoEscape::None` global + copy-pasted `esc`; mutating-GET screen
options; session map never reaps; unreachable multi-site map in `App`;
password via CLI argv + `.trim()` mismatch CLI vs login;
`verify.rs` format! SQL; `generate::run` opens a conn per post.

## Design

### Phase order

| Phase | Name | Status |
|-------|------|--------|
| 1 | [Surgical hotfixes — leaks, DoS, traversal, escaping](./phase-01-surgical-hotfixes.md) | Pending |
| 2 | [Bridge + deploy trust-boundary fixes](./phase-02-bridge-deploy-trust.md) | Pending |
| 3 | [Data-layer correctness — schema, tx, WAL, pull](./phase-03-data-correctness.md) | Pending |
| 4 | [Render/serve hygiene — kind filter, dead code, escaping discipline](./phase-04-render-serve-hygiene.md) | Pending |
| 5 | [Layering restore — plugins/ → root, serve.rs, SQL sweep](./phase-05-layering-restore.md) | Pending |
| 6 | [Regression gates + doc resync](./phase-06-regression-docs.md) | Pending |
| 7 | [Zero-install — web wizard + Cloudflare Tunnel](./phase-07-zero-install.md) | Pending |

Phases 1–4 are independent of each other's file sets and may run in
parallel *except* phase 2 touches `site_deploy.rs` also touched by 1 —
sequence 1→2, parallel (1,3,4). Phase 5 must come last (it moves what
1–4 patched). Phase 6 gates everything.

### Fix contracts (summary — details in phase files)

- `SiteEnv::save` emits `SITE_URL`; `deploy` re-checks after
  `configure` rewrites it; add `--domain` URL validation beyond
  `starts_with("http")`.
- `.md` miss path moves below `miss_rate`; `public_root` joins the
  limiter; per-slug single-flight optional (defer unless cheap).
- `tex_run` gains HTML escaping for `<`/`>`/`&` in text it emits, or
  callers pass `esc_attr`-ed heading text — verify against reference
  captures (ez-toc parity is a stated goal; escaped text is identical
  for benign input).
- `publish_due` runs on a timer (spawn a per-site interval task in
  `serve`) — not per-request, removes the admin-traffic dependency
  entirely.
- `Dest::push` gains `--exclude bridge/` + `--exclude .secret-*` for
  the `www/` push; `backup.rs` excludes `www/bridge/**` and warns re:
  live tokens on the admin page.
- `pull()` errors log to stderr (not `let _ =`); id resolution refuses
  the rowid fallback when `wp_id` lookup misses (skip row + count
  skipped); post_meta pulls use `INSERT OR REPLACE`.
- `bridge kill --slug` goes through `valid_slug`.
- `install_php` routes every interpolated value through `php_str()`.
- Deploy copies `rpress.db` via `VACUUM INTO` (consistent, WAL-free)
  instead of `fs::copy`; `import` removes `db` + `-wal` + `-shm` and
  refuses to clobber a DB with >0 users unless `--force`.
- `render__select_id_lang` gains an explicit kind allowlist for the
  `?3=''` branch (`kind IN ('post','page')` + registered kind bases).
- `plugin_screen` gate becomes per-screen: `ScreenDef` gains a
  `min_role` field defaulting to `admin`; tourmaster ajax save-option
  declares `editor`.
- `theme_locations` rebuild wraps in a tx; `PRAGMA foreign_keys=ON` in
  `conn()` after auditing existing write paths for violations (audit
  first — enable only if no path relies on dangling rows).
- `post_meta` schema keeps the strict UNIQUE `(post_id,meta_key)`;
  importer collapses multi-value metas deterministically (last-wins,
  matching WP get_post_meta scalar reads) and counts them; pull uses
  OR REPLACE. Remove the redundant wide PK column at next schema
  rebuild — or keep for compat; decide in phase 3.
- `site_reviews` ctx payload removed; `site_reviews_form` resolves the
  enclosing post id instead of `6190`.
- `qs()` + any query-param→href path goes through `urlenc`.
- `css_scope::strip_comments` iterates `char_indices`; at-rule dispatch
  handles `;`-terminated rules.

### Structural (phase 5)

- Move `src/admin/plugins/` registry core (`PluginAdapter`,
  `RenderCtx`, `adapter_for_*`, `kind_url_base`, `screen_for`,
  `PLUGINS`/`ADAPTERS`) → `src/plugins/`; adapters' screen handlers may
  keep importing `admin/` helpers — direction becomes
  `admin → plugins ← render`.
- Move public serve path (`public_page`, `public_root`, `read_out`,
  `write_cache`, `md_for_slug`, `asset_redirect_target`, `is_junk`,
  `publish_due` wiring) → `src/serve.rs`; `admin/mod.rs` keeps the
  `/admin/*` router + session/CSRF engine.
- SQL sweep: inline statements move to `db/sql/*.sql` via `sql!()`;
  `db::ids_matching` deleted (callers get typed helpers).
- `db/misc.rs` splits: SQL helpers stay, media fs/image ops →
  `src/media.rs`.
- Docs: architecture.md layer table updated to the real dependency
  graph after the moves.

## Non-goals

- No new features; no WP-parity additions.
- No Linux sandbox implementation (bubblewrap) — phase 2 makes the
  fail-open **explicit** (warn + env opt-out `RPRESS_BRIDGE_UNSAFE=1`
  or refuse to spawn unconfined; decide by deploy reality) rather than
  silently unsandboxed.
- No multi-site-in-one-process work — `App.sites` stays a 1-entry map;
  dead branch removed, not extended.
- `esc`/`AutoEscape::None` global policy stays (huge blast radius);
  phase 4 fixes the two proven misses and adds a test seam only.

## Verification

Per-phase gates (each phase file carries exact commands):

```sh
cargo build && cargo test                    # unit/integration suite
sh scripts/e2e-site.sh sites/aucoeur-test.env import generate check serve
sh scripts/visual_smoke.sh                   # chrome parity gate
```

New regression tests land in `tests/hardening.rs` (public path),
`tests/exposure.rs` (kind filter, .md limiter), and a new
`tests/site_env.rs` for `SITE_URL` round-trip + slug validation.

Final gate (phase 6): full suite + one site rebuilt end-to-end from a
fresh `site build` + `grep -rn "SELECT\|INSERT\|UPDATE\|DELETE"
src/{admin,render,intake}` returns only `sql!()` references —
that grep is the layering proof, committed into CI-adjacent script
form.
