# E2E workflow — WP clone → running rpress site

Generic, per-site. One env file + one script; nothing hardcodes a site.

```sh
cp sites/_template.env sites/mysite.env   # edit: SITE, BIND, SITE_URL, pull source
scripts/e2e-site.sh sites/mysite.env all  # pull → import → configure → generate → check
scripts/e2e-site.sh sites/mysite.env serve
scripts/e2e-site.sh sites/mysite.env parity   # quality gate vs origin (optional)
```

State: `work/<SITE>/{dump.sql, rpress.db, cache/, www/, credentials.txt}`.
`work/` is gitignored, fully disposable — rerun `all` any time.

## What each step guarantees

| Step | Guarantee |
|---|---|
| pull | dump + `wp-content/{uploads,themes,plugins}` on disk, from the declared clone source. Refuses to run unless the env opts in with `CLONE_SOURCE_OK=yes` — pulls never touch production by default. |
| import | `rpress.db` with content counts verified (`import verify` exits non-zero on mismatch). Importer carries the WP options the runtime reads: `siteurl`, `blogname`→`site_name`, front-page, discussion/moderation keys, media sizes, `tourmaster_general`. |
| configure | `site_url` → `LOCAL_URL`; `asset_origin` ← the dump's real origin (kept for the 307 fallback on missing assets). First admin created only when `users` is empty; password in `work/<SITE>/credentials.txt`. |
| generate | full `cache/` render + utility files (sitemap, feeds, robots, 404, llms.txt); `llms.txt` copied to `www/` so it survives cache purges. |
| check | sampled posts/pages/tours exist in `cache/`; utility files present; warns on internal links still pointing at `asset_origin`. Fails hard on missing pages. |
| parity | fetches the same paths on the live origin and compares `gdlr/tourmaster/traveltour` class sets. This is the honest quality gate — run it, don't trust "generate: N pages". |
| serve | `rpress serve` with env bindings; admin at `http://<BIND>/admin`. |

## Known parity gap (measured, not assumed)

`parity` on aucoeur-test shows ~65–75% class overlap on list/home pages —
the tour-grid and some sidebar markup are thinner than origin. That is a
renderer gap in `src/render*.rs` / `templates/elements/`, not a workflow
gap. `vps-ssh/cms/plans/2026-10-26-render-parity-100/` is the reference
for closing it.

## Notes

- `pagefind` must be on PATH at generate time or `/recherche` degrades to a
  notice (`cargo install pagefind` or `npm i -g pagefind`).
- `www/` inside `work/<SITE>/` is the only dir worth backing up (plus the
  DB); `cache/` is disposable write-back.
- Serve behind nginx: proxy `/admin /api /comment /review /form/ /intake/`
  to the process; let static files hit `cache/` + `www/` directly.
