# CMS operations (local)

Binary: `cms`. `cache/` = disposable HTML+md write-back (auto-warm on
miss); `www/` = persistent uploads + vendored plugin/theme assets +
`llms.txt` + `front.css`/`nav.js`. Back up `www/` + `cms.db`, never `cache/`.

```
cms user add <email> <password> --role admin|editor --db cms.db
cms user passwd <email> <password> --db cms.db
cms serve --bind 127.0.0.1:8400 --db cms.db --templates templates --out cache \
  --www www --host edit.aucoeurvietnam.com --site aucoeur --name "Au Coeur Vietnam" --tour-base voyage
# second site on same host:
# cms serve --bind 127.0.0.1:8401 --db ima.db --templates templates --out cache-ima \
#   --www www-ima --host edit.imagetraveldmc.com --site ima --name "Image Travel DMC" --tour-base tours
```

Public surface: HTML permalink → `cache/<slug>/index.html`; markdown
sibling `cache/<slug>/index.md` for `GET /{slug}.md` and for
`Accept: text/markdown`. `cms generate` writes `llms.txt` into the cache
root; copy it into `www/llms.txt` so it survives a cache purge (the
`www/` copy is what `public_page` serves).

`www/wp-content` is not self-healing: if a media/font/plugin asset is
missing on disk, `public_page` issues a 307 to the live WP host
(`test.aucoeurvietnam.com`). Images still render (`<img>` is no-CORS),
but **webfonts get CORS-blocked and icons render as tofu squares
intermittently** (cache-dependent). After restoring or reseeding `cms.db`,
resync assets from the origin so nothing needs the fallback:

```
# from pet/vps-ssh (vps.sh + .askpass/rsync-pull.sh)
.askpass/rsync-pull.sh ct4 /var/www/aucoeurtest_usr/data/www/test.aucoeurvietnam.com/wp-content/uploads/ cms/www/wp-content/uploads/
.askpass/rsync-pull.sh ct4 /var/www/aucoeurtest_usr/data/www/test.aucoeurvietnam.com/wp-content/plugins/ cms/www/wp-content/plugins/
.askpass/rsync-pull.sh ct4 /var/www/aucoeurtest_usr/data/www/test.aucoeurvietnam.com/wp-content/themes/  cms/www/wp-content/themes/
```

No password-reset email. Failed logins print `login fail email=… ip=…` on stderr.
Rate limit: 20 POSTs /api/login per IP per minute.

## nginx (public host)

Static files are served from `cache/` + `www/`; comment/review/form POSTs
and the admin must proxy to the `cms serve` process:

```
location ~ ^/(comment|review|form/|intake/|admin|api/) {
    proxy_pass http://127.0.0.1:8400;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
}
```

Comment forms post to `/comment` on the same host — without that location
block the public server returns 404 on submit.

## Search index (pagefind)

`/recherche/` and `/es/recherche/` use Pagefind. `cms generate` runs
`pagefind --site <out>` at the end of a build when the binary is on
`PATH`; without it the search page renders a "temporarily unavailable"
notice instead of a dead UI. Install on the VPS:

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
request after that flips it to `publish` and rewrites its output file
(check runs inside the session middleware — no cron needed). Editing the
date back into the past publishes immediately on save.

## Widgets (Appearance → Widgets)

Sidebars and widget instances live in the `sidebars`/`widgets` tables and
are managed at `/admin/widgets` (admin-only). Widget `html` fields are
admin-trusted **minijinja** templates — `{{ site_url }}` and `{{ lang }}`
resolve at render time; an unparseable template is rejected on save
(`msg=badsyntax`).

- Removing a widget parks it in **Inactive Widgets** (`wp_inactive_widgets`)
  with settings preserved; deleting from inactive is permanent.
- Every widget mutation sets `settings.widgets_dirty=1`; the dashboard
  shows a "pending publish" notice until a full `cms generate` clears it.
  Public cache write-back invalidates affected pages on save, so edits
  reach the site without regenerating.
- Custom sidebars can be registered/deleted from the same screen; registered
  sidebars appear in the GoodLayers/tourmaster `sidebar` comboboxes.
- `widgets::ensure` runs at boot and at generate/admin time: it migrates
  the tables, applies seed upgrades (e.g. `-fr`/`-es` sidebar split), and
  seeds only when both tables are empty — a full `TRUNCATE sidebars;
  TRUNCATE widgets` is the only way back to defaults.

