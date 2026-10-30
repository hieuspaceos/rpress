---
title: "Phase 4 — Render/serve hygiene: kind filter, dead code, escaping"
phase: 4
status: pending
priority: P1
effort: 3h
files:
  - src/db/sql/render__select_id_lang.sql
  - src/admin/mod.rs
  - src/render/site_data.rs
  - src/render/shortcode.rs
  - src/css_scope.rs
  - src/intake.rs
  - src/generate.rs
  - src/admin/posts.rs
  - tests/exposure.rs
---

# Phase 4 — Render/serve hygiene

Public-surface correctness: what can render, what escapes, dead work.

## Tasks

### 4.1 Open-kind render — `render__select_id_lang.sql`, `mod.rs:654`

- The `kind=""` branch must not match every kind. Fix in SQL:
  `(?3='' AND kind IN ('post','page')) OR kind=?3` — plus any
  kind registered with an **empty** `url_base` in the adapter registry
  (those legitimately render at root). Implement: the SQL gains a
  generated IN-list, or simpler — `mod.rs` resolves `kind=""` to a
  real allowlist and passes a concrete kind or a sentinel the SQL
  rejects. Cleanest: pass `kind='post'` + fallback second query for
  `page`? No — keep one query: expand `?3=''` to
  `kind IN (SELECT kind FROM posts WHERE kind IN (<root_kinds>))`.
  Decide in implementation: build the IN-list string once per SiteCtx
  from `kind_bases` (empty-base kinds + post/page) and splice into the
  statement via `sql!` variant with a `?` list.
- Regression `tests/exposure.rs`: published `nav_menu_item`-kind row
  with slug → `GET /<slug>` returns 404 (not rendered page);
  `ml-slide` likewise.

### 4.2 `site_reviews` dead payload — `render/site_data.rs:118,330-385`

- Remove the `reviews` ctx insert + `site_reviews()` query block —
  zero consumers (verified: no template/`{{ reviews` reader).
- `[site_reviews_form]` in `shortcode.rs:81` — resolve the enclosing
  post id at expansion time (the shortcode expander knows the post);
  replace `6190` literal.
- Regression: `sig()`-level render diff on a page containing
  `site_reviews_form` shows only the `_post_id` value change; cold
  render perf marginally faster (assert no `site_reviews` query via
  a debug counter or grep — template-level assert is enough).

### 4.3 `css_scope` bugs — `css_scope.rs:18-37,74-113`

- `strip_comments` byte→char mojibake → iterate `css.char_indices()`
  or slice-copy non-comment spans (no per-byte cast).
- At-rule dispatch: `;`-terminated rules (`@import 'x';`,
  `@charset`) currently bypass the drop arm at :97-113 — restructure
  so at-rule name dispatch happens before the `;` early-return; the
  test asserting "@import passes verbatim (parity)" gets revisited —
  if the parity intent is "scope keeps working", the correct parity is
  @import *dropped* (external fetch in live-preview iframe is the bug
  being scoped away). Confirm against actual origin CSS before
  deciding; keep verbatim ONLY if the reference sheet needs it.
- Regression: css with `content:"café"` survives scoping un-mangled;
  `@import` handling matches decided behavior (test both arms).

### 4.4 `plugin_screen` role gate — `admin/mod.rs:1639`, plugins registry

- `ScreenDef` gains `min_role: Role` (default `admin`); the dispatcher
  checks `s.role` rank vs `screen.min_role`.
- tourmaster's ajax `save-option` (posted from the editor) declares
  `min_role: editor` — editors can save metabox settings.
- Audit each adapter's `screens()`: list screens stay `admin` except
  the ones embedded into post-edit UI (tourmaster metabox save).
- Regression: editor-role session POSTs save-option → 200; hits
  `/admin/plugins/tourmaster/settings` → 403.

### 4.5 `generate` conn reuse — `generate.rs:31-283`, `render.rs:61`

- `render_slug` gains a `&Connection` param variant (`render_slug_conn`)
  used by `generate::run`; the HTTP miss path keeps opening per-call
  (correct there). One conn for a 1240-post build.
- Same for the per-post `modified_at` N+1 in sitefx — prepare once
  outside the loop (already prepared once? verify — finding says
  per-row call into a prepared stmt is fine; the conn open per render
  is the real cost).

### 4.6 Intake link count — `intake.rs:309`

- `body.matches("http")` → count `http://`/`https://`/`www.`/`//`
  occurrences case-insensitively; document the heuristic.

### 4.7 Mutating-GET screen options — `admin/{tax,users,admin}.rs`

- `/admin/tax?screen_options=1`, users, media mode: convert to POST
  routes with CSRF (matching `comments::screen_options` which is
  already POST+CSRF). The UI JS posts instead of linking.
- If WP-parity demands GET links: at minimum add CSRF token to the
  link (still a mutating GET — prefer POST).

### 4.8 `verify.rs`-adjacent escapes — `admin/posts.rs:813,1130`

- `post_data` `format!("{:?}")` → `serde_json::json!`.
- tag-cloud `onclick` custom escape → `esc_js` (exists at
  admin.rs:978).

## Acceptance

- `cargo test` + `visual_smoke.sh` green.
- `GET /<nav-menu-item-slug>` on a live db → 404.
- Editor can save tour metabox options; admin gates unchanged
  elsewhere.
