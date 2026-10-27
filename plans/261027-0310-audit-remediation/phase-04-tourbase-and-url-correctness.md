---
phase: 4
title: "tour_base & URL correctness"
status: pending
priority: P1
effort: "5h"
dependencies: [1]
---

# Phase 4: tour_base & URL correctness

## Goal

`--tour-base`/`TOUR_BASE` is honored by every rendered URL (canonical,
hreflang, sitemap, redirects, llms.txt, widgets, admin invalidation);
deployed sites never carry `localhost:PORT` or stale-domain URLs; archive
aliases resolve to canonical paths; XML emitters escape.

## Findings covered

- **C2** — hardcoded `voyage` across render/generate/sitefx/widgets/admin
  (~25 emission sites, most format-embedded)
- **H11** — deploy leaks LOCAL_URL into menu links + redirects.map + vps db
- **M10** — sitemap/feed/redirects.map raw XML output
- **M19** — `e2e-site.sh` hardcodes `voyage` + VPS_HOST-only pulls
- **L** — `/page/N` 404 parity gap; `is_taxonomy` allowlist; menus
  up/down subtree order (`db/menus.rs:227-231`)

## Key insights

- `lang::page_url` already takes tour_base — the plumbing exists; the bug
  is call sites passing literals. **Corrected plumbing (red-team)**:
  `SiteCtx` exists only inside `admin::serve`; `render_slug` takes
  `(db, templates, path, lang, kind, site_url)` — no SiteCtx;
  `generate::run`/`emit_site_files` take no tour_base at all. tour_base
  persists only in `SiteEnv` (`TOUR_BASE=`) and the serve CLI flag.
  So: add `tour_base: &str` to `render_slug`, `render_slug_by_id`,
  `generate::run`, `emit_site_files`, and the shared render ctx; sources
  per caller — serve: SiteCtx/CLI flag; `site build`/`site deploy`/admin
  regen: `SiteEnv.tour_base`; bare `rpress generate`: a `--tour-base` flag
  (default `voyage`).
- Most `voyage` literals are embedded in format strings — the verification
  grep must be `grep -rn 'voyage' src/` filtered to code contexts, not
  `'voyage'` quoted-literal grep which misses them.
- LOCAL_URL deploy fix: rewrite `menu_items.url` and `redirects.new_url`
  on the STAGED db copy only — never the sandbox `work/rpress.db` (a
  rewrite there destroys the local_url key for future rewrites). Key the
  REPLACE off a list: `env.local_url` + previously-deployed domain
  (persisted in a settings row written at first deploy) + a
  `http://localhost:*` pattern. VPS push: rewrite on the pushed copy.
- XML: one `xml_esc()` helper; CDATA payloads split `]]>` →
  `]]]]><![CDATA[>`.

## Files to modify

- `src/render.rs`, `src/render_post.rs`, `src/render_tour.rs`,
  `src/render_archive.rs`, `src/widgets.rs` — replace literals with the
  threaded param
- `src/generate.rs`, `src/sitefx.rs` — `generate::run`/`emit_site_files`
  gain `tour_base`; sitemap/feed/robots/llms/redirects emitters
- `src/main.rs`, `src/site_serve.rs`, `src/site_build.rs`,
  `src/site_deploy.rs` — pass tour_base from SiteEnv/flag
- `src/admin/posts.rs` — cache-invalidation paths use `sc.tour_base`
- `src/site_deploy.rs` — stage_domain + vps db rewrite (menu_items.url,
  redirects.new_url; staged copy only; domain-keyed settings row)
- `src/db/menus.rs` — up/down moves whole subtree via `subtree_end`
- `src/render_archive.rs` — taxonomy check via DB lookup; `/page/N` →
  301 to archive root (document divergence)
- `scripts/e2e-site.sh` — `$TOUR_BASE`; SSH_TARGET remote pulls
- `docs/` — document /page/N divergence

## Implementation steps

1. Add `tour_base: &str` to the shared render ctx + the function
   signatures above; wire each caller's source (SiteCtx / SiteEnv /
   `--tour-base` flag).
2. Mechanical pass: replace every `voyage` literal (incl. format-embedded)
   with the threaded value. Known sites: posts.rs:687,1381;
   render_tour.rs:90,267; render.rs:493,497,500,1177-1178,2017,2043;
   render_post.rs:927; render_archive.rs:461; generate.rs:87-88,150,266,
   297,570,708; sitefx.rs:135-136,152; widgets.rs:1025.
3. `stage_domain`: on the staged copy REPLACE menu_items.url /
   redirects.new_url keyed on {env.local_url, previous deployed domain
   (settings row), `http://localhost:*`}; write the deployed domain to
   settings; same on the vps db-push copy.
4. `xml_esc` + CDATA-safe emit in sitemap/feed/redirects writers.
5. Taxonomy dispatch: `SELECT 1 FROM terms/post_terms WHERE taxonomy=?`
   replaces the hardcoded `is_taxonomy` list.
6. `/page/N` leaf all-digits → 301 to archive root.
7. menus up/down subtree move.
8. e2e-site.sh `$TOUR_BASE` + SSH_TARGET support.

## Verification

- `grep -rn 'voyage' src/ scripts/` → only config defaults/comments/test
  fixtures remain (format-embedded literals included).
- `cargo test`; render_slug for a tour with `tour_base='tours'` →
  canonical/hreflang/sitemap all emit `/tours/`.
- `site serve --smoke`: `/voyage/x` under tour_base=tours → 404,
  `/tours/x` → 200.
- Deploy `--dry-run` + inspect staged db: menu_items/redirects carry
  `--domain`; re-deploy to a different domain rewrites correctly
  (idempotence via settings-row domain key).
- Sitemap with a slug containing `&` → well-formed XML.

## Todo

- [ ] Thread tour_base end-to-end (ctx + fn params + caller sources)
- [ ] Zero voyage literals incl. widgets.rs and admin invalidation
- [ ] Deploy URL rewrite on staged/vps copies + domain settings row
- [ ] xml_esc + CDATA fix
- [ ] Taxonomy DB lookup; /page/N → 301; menu subtree moves; e2e-site.sh

## Risks

- tour_base signature changes touch internal APIs — cargo check gate;
  keep the param ordering consistent (append, not insert).
- Double-rewrite guard: REPLACE keyed off the list above is idempotent;
  never run it on the source sandbox db.
