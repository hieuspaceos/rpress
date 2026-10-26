# Architecture — pressrs

One site-agnostic binary, three layers, WP-shaped names on purpose.

```mermaid
flowchart LR
    CLI[main.rs CLI] --> SRV[admin/ axum server]
    CLI --> IMP[import.rs WP dump → SQLite]
    CLI --> GEN[generate.rs → cache/ warm]
    CLI --> MIG[migrate.rs WP → running site]
    SRV --> DB[(db/ rusqlite → pressrs.db)]
    SRV --> TPL[templates/ minijinja]
    SRV --> STATIC[cache/ + www/]
```

## Layers

| Layer | Directory | Owns | May import |
|---|---|---|---|
| Admin (HTTP) | `src/admin/` | routes, HTML screens, auth guards, forms | `db`, `intake`, `sitefx`, `render` |
| Data | `src/db/` | every SQL statement, term/post mutation helpers | nothing (rusqlite only) |
| Site output | `src/render*.rs`, `src/generate.rs`, `src/intake.rs`, `src/sitefx.rs` | page render + cache write-back + public intake + utility-file emit | `db` |
| Support | `src/schema.rs`, `src/auth.rs`, `src/import.rs`, `src/sqldump.rs`, `src/php_serialize.rs`, `src/verify.rs`, `src/widgets.rs` | schema DDL, password hash, WP-dump import, sidebar/widget store | `db` where needed |

Rule: **SQL lives only in `src/db/`**. Admin handlers never write SQL; they
call a `db::` function. Rendering helpers (`esc`, `chrome`, `html_admin`,
`need_auth`) live in `src/admin/mod.rs` and are `pub(crate)` for siblings.

## Sites — multisite by process

```mermaid
flowchart TB
    nginx --> A[pressrs serve :8400 --site site-a --host a.example.com]
    nginx --> B[pressrs serve :8401 --site site-b --host b.example.com]
    A --> DB1[(site-a.db)]
    B --> DB2[(site-b.db)]

The binary is site-agnostic; the *process* is bound to one site via CLI args.
`App.sites` is a `Host → SiteCtx` map so one process CAN serve several hosts,
but the normal layout is **one process per site, one `.db` per site** —
the WP-multisite equivalent with stronger isolation (a crash or bad import on
one site never touches another's data). nginx maps each host to its port.

`SiteCtx` carries everything site-local:

| Field | Source |
|---|---|
| `slug` | `--site` — session scoping key |
| `name` | `--name` — admin chrome brand |
| `db`, `templates`, `out`, `www` | `--db`/`--templates`/`--out`/`--www` — `out` is the disposable cache; `www` holds persistent uploads + vendored assets |
| `tour_base` | `--tour-base` — CPT URL segment |
| `generating` | per-site generate flag |

**`need_auth(&app, &headers) → (Sess, &SiteCtx)`** is the single entry:
handlers never call `site_for` separately and never touch `app.db` directly —
there is no `app.db`. Unknown `Host` falls back to `default_site` so
single-site dev needs no flags beyond `--site default`.

Site-shared in `App` only: `sessions` (keyed per-site via `Sess.site` — a
token minted on site A is rejected on site B), `login_hits`, `intake_rate`,
`miss_rate` (60/min/IP cap on public render-miss work).

**Nothing in `src/` names a domain.** The public base URL for permalinks and
CORS origins comes from `settings.site_url` in that site's own DB
(`site_url(sc)`, `allowed_origins(sc)`). A second origin rewrite is
`settings.legacy_origin`: when non-empty, `render_post::rewrite_origin`
replaces it with `settings.site_url` in post content (set it to the
imported WP domain; empty = no rewrite).

## `src/admin/` module map (wp-admin equivalents)

| File | WP analog | Contents |
|---|---|---|
| `mod.rs` | `wp-admin/admin.php` + `admin-header.php` | `serve()` router, `App`/`Sess`, sessions, `need_auth`, `chrome*` sidebar, `menu_li`/`menu_li_badge`, `esc`, `html_admin` |
| `auth.rs` | `wp-login.php` | login page/handler, logout, dashboard |
| `list.rs` | `class-wp-list-table.php` | shared `subsubsub` views, `tablenav` bulk+pager, `row_actions`, `thead`, `empty_row`, `back_or` |
| `posts.rs` | `edit.php`, `post.php`, `post-new.php` | list screen (`render_list`), row actions (trash/clone/quick-edit/bulk), post editor (`edit_html`), save |
| `comments.rs` | `edit-comments.php`, `comment.php` | views w/ counts, per-status row actions, reply, quick-edit, edit (status radios + date), bulk, Empty Spam/Trash |
| `users.rs` | `users.php`, `user-new.php`, `user-edit.php`, `profile.php` | role views, create/edit/delete confirm screen, change-role bulk, self/last-admin guards, `drop_sessions_for` |
| `contact.rs` | CF7 list + Flamingo inbox | `forms_page` (FORM_SLUGS registry), `inbox_page` (Inbox/Spam/Trash, sortable), `detail_page`, moderation via `intake::moderate`, `/admin/submissions` 301s |
| `menus.rs` | `nav-menus.php` | selector+create/delete, add-items metaboxes (Pages/Posts/Tours/Custom/Categories), item blocks (Original:, move links, Remove), save_tree tx |
| `tax.rs` | `edit-tags.php` | Categories/Tags two-column screen, save/delete/bulk |
| `admin.rs` | `upload.php`, APIs | media, static-generate + status, `api/v1/posts` JSON, intake handlers |
| `plugins/` | plugin metaboxes (`add_meta_box`) | per-kind editor plugins: `EditorPlugin{kind, load, metabox, assets, save}` registry (`PLUGINS`), `tourmaster/` — ported WP metabox |

## `src/db/` module map (wp-includes equivalents)

| File | WP analog | Contents |
|---|---|---|
| `mod.rs` | — | `slugify`, `attach_term`, `term_name`; re-exports all submodules |
| `posts.rs` | `post.php` | `PostForm`, `load_post`, `save_post`, `list_admin`, `clone_post`, `quick_update`, `bulk_apply`, `set_post_status`, admin-support queries (`tag_names_for`, `lang_of`, `media_choices`, `media_path`, `published_at`) |
| `comments.rs` | `comment.php` | `counts`, `list(Filter)`, `get`, `set_status`, `update_fields`, `delete_permanent` (re-parents children in same tx), `insert_reply`, `empty_status` |
| `users.rs` | `user.php` | `counts_by_role`, `list`, `get`, `create`, `update`, `delete`, `count_admins`, `change_role` |
| `submissions.rs` | — | `counts`, `form_counts`, `list(Filter)` (sortable), `get`, `set_status`, `purge` |
| `menus.rs` | `nav-menu.php` | `list_menus`, `get_menu`, `list_items`, `create_menu`, `rename_menu`, `delete_menu`, `save_tree`, `add_items`, `move_item`, panel queries |
| `terms.rs` | `taxonomy.php` | `TermRow`, `list_terms`, `save_term`, `delete_term`, `term_choices`, `term_slugs_for`, `terms_for`, `set_terms`, `taxonomy_exists` |
| `misc.rs` | `media.php` | `save_upload`, media file helpers, `body_json` (GoodLayers) helpers |

**`src/db/sql/`** — one `.sql` file per statement (`{module}__{verb}_{noun}.sql`),
loaded at compile time via `sql!("name")` (`include_str!`). The `db/`
modules are the only SQL writers; `admin/` goes through `db::` functions.

## Data model (owner: `src/schema.rs`)

`posts` (all kinds: post/page/tour via `kind`), `tours` (tour-only columns),
`post_terms` (assignments), `terms` (term registry — needed so
Categories/Tags screens can create terms with no post yet),
`term_translations`, `post_translations` (FR↔ES pairing), `post_meta`,
`media`, `menus`/`menu_items`, `comments`, `submissions`,
`settings` (site_url/site_name/legacy_origin/asset_origin/widgets_dirty
  + misc options), `sidebars`/`widgets` (widget instances — WP-parity model),
`sessions` (in-memory only — restart logs everyone out, intentional).

Language model: `posts.lang` ∈ {`fr`,`es`}; FR/ES pairing via
`post_translations`. ES taxonomies use `-es`-suffixed slugs or separate
taxonomies (`duracion`, `region-es`, `estilo`, `perfil`) — imported that way
from WP/Polylang; preserved, not normalized.

## Decisions worth knowing

- **Sessions in-memory, CSRF per-session, scoped per-site.** Restart =
  relogin; chosen over a sessions table to keep the DB append-only-ish. CSRF
  token lives on `Sess`; `Sess.site` pins the token to one site's `slug` so
  credentialed access never leaks across vhosts.
- **`form=` attribute pattern.** `<form>` inside `<tbody>` is dropped by the
  HTML parser, which silently broke Quick Edit submits. Rule: quick-edit and
  bulk forms declare the form *outside* the table; controls inside rows carry
  `form="id"` attributes.
- **Multi-value form keys use `p-{id}` / `d-{id}` / `bcat-{slug}` prefixes**
  (`menus_save` pattern) because axum's `Form<HashMap>` collapses repeated
  keys. Handlers strip prefixes server-side.
- **Bulk edit is additive** (WP behavior): cats/tags are only added, never
  removed; status changes only when a non-empty value is chosen.
- **Clone** duplicates post + terms + meta + tour row + translation link in
  one transaction; "New Draft" = clone with `status=draft`.
- **`INSERT OR IGNORE`** everywhere for terms/assignments — idempotent, no
  race surface for the single-user admin.
- **Quick Edit writes through `quick_update`/`bulk_apply`**, not `save_post`:
  they touch only title/slug/status/lang + cat/tag terms and preserve
  tour taxonomies (`duree`, `region`, …) untouched.
- **No domain literals in `src/`.** Any string naming a host, brand, or URL
  segment comes from `SiteCtx` (CLI), `settings.*` (DB), or `src/seed/`
  (reference-theme payloads). Adding a site must never require a code change.
  `src/widgets.rs` SEED_WIDGETS + `src/seed/*.json` are the reference theme's
  default widget set — replace both for your own theme.
- **Builder Live Mode loads scoped frontend CSS.** `templates/front-css/*.scoped.css`
  are the theme/plugin stylesheets fetched once from the live site, prefixed to
  `.gdlr-core-pb-livemode` by `scripts/scope-css.py` (re-run it after refreshing
  the raw `.css` files). The pb body also carries `traveltour-body(-front)` so
  `.traveltour-body …` theme rules resolve. Canvas is zoomed to the 1350px
  frontend width via `gdlrLiveScale()` in `asset_tags()` — keep it, else
  %-padded wrappers collapse in the ~860px admin column.
- **text-box content is entity-decoded at render** (`render.rs::render_node`).
  TinyMCE saves store `&lt;h1&gt;`; WP decodes entities on output, so we mirror
  it for the static build and the live-mode preview.

## What this is NOT

- No plugin/theme hooks — features are code, deployed with the binary.
- No REST JSON admin API beyond `/api/v1/posts` (list) and `/api/generate*`.
- Public pages are **lazy render-cache, not a baked tree**: `/{*public}`
  serves `cache/<slug>/index.html` when present, else renders on demand
  and writes back (publish-only, atomic tmp+rename). `pressrs generate` still
  exists as a bulk warm/refresh tool but is no longer the serving path —
  a cold `cache/` warms itself from traffic. `www/` is the persistent
  sibling (uploads, vendored plugin/theme assets, `llms.txt`, `front.css`,
  `nav.js`) and is what nginx should back up + serve directly.
- Markdown siblings: `cache/<slug>/index.md` serves `GET /<slug>.md` and
  `Accept: text/markdown` requests; `llms.txt` indexes every page + md.
  Source: `render::post_markdown` + `html_to_markdown`.

## Front-end runtime (`www/nav.js`, `www/front.css`)

Every public page ends with `<script defer src="/nav.js">` inside
`base_*.html`, *inside* the swapped content region. `nav.js` is therefore
re-executed on every SPA-style navigation — the file opens with a
`window.__navSwapInit` idempotence guard, and every side effect must be
safe to re-run.

`navigate()` swaps `.traveltour-body-outer-wrapper` (the element that
contains both `<header>` and `<footer>`), merges `<style id>`/`<script id>`
in `<head>` by id (`mergeHeadTags`), revives `<script>`s in the new
content, then re-runs theme JS init that bound to the *old* nodes —
`rebuildMegaMenus()` and `$.fn.superfish`. Anchors `#id` and same-path
`?…#…` URLs bypass swap entirely; `popstate` skips same-path hash hops.
Ownership: `www/nav.js` — see its leading comment for the full contract.

`templates/post_assets.txt` carries post-only head/footer assets
(ez-toc CSS, wpDiscuz, comment-reply) in `===NAME===` sections;
`render_post::inject_post_assets` splices them into the rendered page.
`filter_content` owns the_content-equivalent pipeline: `expand_body_
shortcodes` (caption/button/embed/icon/widget_box), `wpautop`,
`texturize`, `rmwr_wrap` (`[read more]` → hidden div + toggle), `ez_toc`
(heading anchors + Sommaire container).
