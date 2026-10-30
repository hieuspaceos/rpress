# rpress operations (local)

Binary: `rpress`. `cache/` = disposable HTML+md write-back (auto-warm on
miss); `www/` = persistent uploads + vendored plugin/theme assets +
`llms.txt` + `front.css`/`nav.js`. Back up `www/` + `rpress.db`, never
`cache/`.

The SQLite DB contains users, password hashes, application-password hashes,
and persisted admin session tokens until expiry/revocation. Treat DB backups as
authentication material.

```
rpress user add <email> <password> --role admin|editor --db rpress.db
rpress user passwd <email> <password> --db rpress.db
rpress serve --bind 127.0.0.1:8400 --db rpress.db --templates templates --out cache \
  --www www --host edit.aucoeurvietnam.com --site aucoeur --name "Au Coeur Vietnam" --tour-base voyage
# second site on same host:
# rpress serve --bind 127.0.0.1:8401 --db ima.db --templates templates --out cache-ima \
#   --www www-ima --host edit.imagetraveldmc.com --site ima --name "Image Travel DMC" --tour-base tours
```

Public surface: HTML permalink → `cache/<slug>/index.html`; markdown
sibling `cache/<slug>/index.md` for `GET /{slug}.md` and for
`Accept: text/markdown`. `rpress generate` writes utility files with atomic
sibling tmp+rename writes so live readers never see partial output.
Templates are compiled once per templates-dir (cached Environment) and
`site_data` is cached per (site_url, lang). Both bust on any admin save /
template edit — but **editing template files on disk while `serve` runs
does NOT invalidate**: restart serve or touch any admin save to pick up
disk-level edits.

`www/wp-content` is not self-healing: if a media/font/plugin asset is missing
on disk, `public_page` issues a 307 to the configured asset origin. Images
still render (`<img>` is no-CORS), but webfonts can be CORS-blocked and icons
render as tofu squares. After restoring or reseeding `rpress.db`, resync assets
from the origin so nothing needs the fallback:

```
# from pet/vps-ssh (vps.sh + .askpass/rsync-pull.sh)
.askpass/rsync-pull.sh ct4 /var/www/aucoeurtest_usr/data/www/test.aucoeurvietnam.com/wp-content/uploads/ rpress/www/wp-content/uploads/
.askpass/rsync-pull.sh ct4 /var/www/aucoeurtest_usr/data/www/test.aucoeurvietnam.com/wp-content/plugins/ rpress/www/wp-content/plugins/
.askpass/rsync-pull.sh ct4 /var/www/aucoeurtest_usr/data/www/test.aucoeurvietnam.com/wp-content/themes/  rpress/www/wp-content/themes/
```

On-disk files under `www/` are served verbatim only when the extension is a
safe asset type (`.css .js .map .mjs .woff .woff2 .ttf .otf .eot .png .jpg
.jpeg .webp .gif .ico .avif .svg .pdf .mp4 .mp3 .webm .txt .webmanifest`)
**and** the path sits under a real asset prefix (`wp-content/`,
`wp-includes/`, `uploads/`, `assets/`). A cached HTML page at the same path
wins over the raw file (`{clean}.html`, `index.html` tried first). Anything
outside that gate — `.php`, `.ini`, `.xml`, arbitrary restored artifacts —
falls through to the miss path (redirect allowlist or 404). Scriptable
restored files such as SVG, XML, HTML, JS, and CSS in legacy uploads are
forced to `application/octet-stream` downloads with `nosniff`.

No password-reset email. Failed logins print `login fail email=… ip=…` on stderr.
Rate limit: 20 POSTs `/api/login` per IP per minute. Uploads are capped at 25 MB.

When running behind nginx on localhost, start `rpress serve` with
`RPRESS_TRUSTED_PROXY=127.0.0.1` (or the proxy IP). Otherwise rpress ignores
`X-Forwarded-For` and all login/render-miss rate limits collapse to the nginx
peer IP.

## nginx (public host)

Static HTML resolves through `rpress serve` from `cache/`; shared assets
(`front.css`/`nav.js`, `llms.txt`, `wp-content/`, `wp-includes/`) resolve
from `www/` — nginx may front either root directly. Comment/review/form
POSTs and the admin must proxy to the `rpress serve` process:

```
location ~ ^/(comment|review|form/|intake/|admin|api/) {
    proxy_pass http://127.0.0.1:8400;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
}
```

Comment forms post to `/comment` on the same host — without that location block
the public server returns 404 on submit.

## Search index (pagefind)

`/recherche/` and `/es/recherche/` use Pagefind. `rpress generate` runs
`pagefind --site <out>` at the end of a build when the binary is on `PATH`;
without it the search page renders a "temporarily unavailable" notice instead
of a dead UI. Install on the VPS:

```
# any one of:
cargo install pagefind
npm install -g pagefind
# or the prebuilt binary from https://github.com/CloudCannon/pagefind/releases
```

One merged index serves both languages — pagefind routes each page by its
`lang` attribute, so FR/ES stemming work without per-locale indexes.

## Scheduled posts

Setting a future "Published on" date in the editor stores `status='future'`;
the post is excluded from `generate` until the date passes. The next admin
request after that flips it to `publish` and rewrites its output file (check
runs inside the session middleware — no cron needed). Editing the date back
into the past publishes immediately on save.

## Widgets (Appearance → Widgets)

Sidebars and widget instances live in the `sidebars`/`widgets` tables and are
managed at `/admin/widgets` (admin-only). Widget `html` fields are admin-trusted
**minijinja** templates — `{{ site_url }}` and `{{ lang }}` resolve at render
time; an unparseable template is rejected on save (`msg=badsyntax`).

- Removing a widget parks it in **Inactive Widgets** (`wp_inactive_widgets`)
  with settings preserved; deleting from inactive is permanent.
- Every widget mutation sets `settings.widgets_dirty=1`; the dashboard shows a
  "pending publish" notice until a full `rpress generate` clears it. Public
  cache write-back invalidates affected pages on save, so edits reach the site
  without regenerating.
- Custom sidebars can be registered/deleted from the same screen; registered
  sidebars appear in the GoodLayers/tourmaster `sidebar` comboboxes.
- `widgets::ensure` runs at boot and at generate/admin time: it migrates the
  tables, applies seed upgrades (e.g. `-fr`/`-es` sidebar split), and seeds
  only when both tables are empty — a full `TRUNCATE sidebars; TRUNCATE widgets`
  is the only way back to defaults.

## post_meta uniqueness (import + admin)

- `post_meta` has `UNIQUE(post_id, meta_key)`. WP allows multiple values per
  key; rpress collapses them — every reader takes `LIMIT 1`, and the kept row
  is `MIN(meta_value)` (what the covering index serves). On `rpress import`,
  extra values are `INSERT OR IGNORE`d and the count is logged in the import
  report (`post_meta: N value(s) dropped`). Admin writes go through
  `INSERT OR REPLACE`, so a second value overwrites rather than errors.
- `serve` startup runs the dedupe migration first: pre-existing duplicate
  rows are reduced to the `MIN(meta_value)` keeper before the unique index
  is created; migration failure aborts startup rather than serving on a
  wrong schema.
