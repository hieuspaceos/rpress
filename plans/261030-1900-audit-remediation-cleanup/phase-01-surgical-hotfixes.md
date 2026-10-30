---
title: "Phase 1 — Surgical hotfixes: leaks, DoS, traversal, escaping"
phase: 1
status: pending
priority: P0
effort: 3h
files:
  - src/site.rs
  - src/admin/mod.rs
  - src/render_post.rs
  - src/site_bridge.rs
  - src/admin/posts.rs
  - src/site_deploy.rs
  - tests/site_env.rs
  - tests/exposure.rs
---

# Phase 1 — Surgical hotfixes

Smallest-possible diffs on the highest-blast-radius bugs. No refactors.

## Tasks

### 1.1 `SITE_URL` persist + domain validation — `src/site.rs`, `src/site_deploy.rs`

- `SiteEnv::save()` emits `SITE_URL={q(&self.site_url)}` after
  `LOCAL_URL`. Skip when empty (pre-init envs legitimately lack it —
  but then `load()` must also treat missing vs `""` the same).
- `site_deploy.rs:55-57` — replace `starts_with("http")` with a real
  parse: `url::Url`-style manual check is fine (no new dep — validate
  `https?://<host>` with a host char check), bail otherwise.
- Regression `tests/site_env.rs`: write env via `SiteEnv::save`,
  reload via `load`, assert `site_url` round-trips; assert `deploy`'s
  domain-equals-origin bail triggers when `SITE_URL == --domain`
  (extract the check to a testable fn if needed).

### 1.2 `.md` path under the limiter — `src/admin/mod.rs:604-642`

- Move the `miss_rate` block (:633-642) **above** the `md_path.is_some()`
  branch (:604). The extension-miss/`asset_redirect` block must stay
  below the limiter? No — asset redirects are cheap 307/404s, no render
  work; keep them above the limiter **except** anything calling
  `md_for_slug`/`render`. Final order: candidate probes →
  extension-miss redirect → **limiter** → `.md` render → HTML render.
- `public_root` (:447) joins the limiter too: same
  `client_ip`/`miss_rate.allow` before `sweep()`/front render.
- Regression `tests/exposure.rs`: spawn debug bin, `GET /missing-slug-N.md`
  70× with rotating N → assert some `429`s (limiter engaged); `GET /`
  on a cold cache → render happens but second burst of unique
  uncached-but-valid slugs hits 429.

### 1.3 ez-toc heading escape — `src/render_post.rs`

- In the three `nav.push_str` sites (:1461, :1478-1479, :1489-1490)
  pass the heading text through `esc()` (the crate's HTML-escape
  helper) **before** `tex_run` — or escape inside `tex_run`'s
  `out.push(c)` default arm. Choose: add `esc()` at call sites —
  `tex_run` is also used on body text where raw `<` may be
  intentional? Verify callers; if any caller relies on raw `<>`
  passthrough, do NOT change `tex_run` itself.
- Parity check: benign headings (apostrophes, accents, `&`) must
  render byte-identical to before — `esc` is a no-op on text without
  `<>&"`. Run `visual_smoke.sh`.
- Regression: unit test `ez_toc` on body containing
  `<h2>&lt;script&gt;alert(1)&lt;/script&gt;</h2>` → output TOC anchor
  text contains `&lt;script&gt;`, never `<script>`.

### 1.4 `publish_due` on a timer — `src/admin/mod.rs`, `src/serve` entry

- Remove the `get_sess` call site (:1057). In `serve()` startup, spawn
  `tokio::task` per site: `loop { sleep(60s); publish_due(sc); }` —
  reusing the existing `publish_due_at` throttle inside `publish_due`
  as the guard (keeps semantics: no-op when nothing due).
- Keep the fn `pub(crate)`; admin path no longer triggers it (timer
  owns it). Comment at `get_sess` updated.
- Verify: scheduled post flips to publish within ~60s on a site with
  zero admin logins (manual smoke: set a post `status=future,
  published_at=now-1s` in a test db, boot server, poll the slug).

### 1.5 `bridge kill` slug validation — `src/site_bridge.rs:81-89`

- Gate `slug` through `site::valid_slug` (already exists, `site.rs:31`)
  before `base.join(slug)`; bail on invalid.
- Regression: `rpress bridge kill --slug ../..` exits non-zero, no fs
  change (assert via a canary dir).

### 1.6 `?tax=` escaping — `src/admin/posts.rs:103-120,155-205`

- `qs()` runs every interpolated component through the existing
  `urlenc` helper (admin.rs has one — `crate::admin::urlenc`); also
  escape `status`/`lang`/`paged` for symmetry (they're int/enum-typed
  today but the helper is one call).
- Check sibling list screens (comments, users, tax, contact) for the
  same `?param → href` splice pattern — fix identical instances in
  this phase (they're the same bug class).
- Regression: `GET /admin/posts?tax=" onmouseover="x` returns page
  with `tax=%22` in hrefs, no raw quote (assert via test or curl).

### 1.7 `www/bridge` deploy exclusion — `src/site_deploy.rs`

- `Dest::push` gains an `excludes: &[&str]` param; the `www/` push in
  `deploy_static`/`deploy_pages`/`deploy_vps` passes
  `["bridge/", ".secret-*"]` → `rsync --exclude`.
- `deploy_vps` is the target that NEEDS the bridge — think: vps
  deploys the full app, bridge included is legitimate there BUT the
  secrets (`bridge-token`, `.secret-*`, `wp-config.php` NONCE_KEY) must
  still not land in a web-served path. On vps they land under app
  dir not docroot — acceptable; static/pages exclude entirely.
- Regression: run `deploy --target static --dry-run`-equivalent or a
  local Dest::Local push into tmp; assert `bridge/` absent in dest.

## Acceptance

- All 7 tasks done; `cargo test` green; `visual_smoke.sh` green on
  aucoeur-test + one tevily site; grep confirms no `starts_with("http")`
  remains for domain validation.
