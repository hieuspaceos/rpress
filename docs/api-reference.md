# API reference — rpress

Route authority is `src/admin/mod.rs::serve()` — one `Router::new()` chain.
This file documents intent + contract, not mechanics. If a row drifts from
the router, the router wins.

Auth: every `/admin/*` route needs the `rpress_session` cookie scoped to the
current site's `SiteCtx.slug` — a token minted on one host is rejected on
another. Every mutating route, including logout, needs the session CSRF token (`name="csrf"`).
## Admin screens (HTML)

| Route | Handler (`admin/`) | Purpose |
|---|---|---|
| `GET /admin/login`, `POST /api/login`, `POST /api/logout` | `auth.rs` | login/logout (rate-limited 20/min/IP; logout is POST-only) |
| `GET /admin`, `POST /admin` (screen options) | `auth.rs::dashboard`, `::screen_options_save` | WP dashboard |
| `POST /admin/draft`, `GET /admin/health` | `dashboard.rs` | editor autosave draft, admin health page |
| `GET /admin/{section}` | `posts.rs::section` | list screen — `posts`, `pages`, `tours`; views All/Published/Pending/Drafts/Trash, `?status=`, `?lang=`, `?tax=`, `?cat=`, `?s=`, `?paged=` |
| `GET /admin/new/{kind}`, `GET /admin/edit/{id}`, `POST /admin/save` | `posts.rs` | post editor |
| `POST /admin/trash`, `POST /admin/clone`, `POST /admin/quickedit`, `POST /admin/bulk` | `posts.rs` | row/bulk actions; statuses `publish\|future\|draft\|pending\|trash` |
| `POST /admin/posts/empty?kind=post` | `posts.rs::posts_empty` | Empty Trash on the Trash view — hard-deletes trashed posts + dependents |
| `GET /admin/posts/{id}/preview` | `posts.rs::preview_post` | session-gated render of any non-trash status through the generate pipeline; `X-Robots-Tag: noindex` + forced robots meta |
| `GET /admin/posts/{id}/revisions` | `revisions.rs` | revision list (one snapshot per save) |
| `GET /admin/revisions/{rid}` | `revisions.rs` | title/body line+word diff vs current content |
| `POST /admin/revisions/{rid}/restore` | `revisions.rs` | snapshot current, copy revision title/excerpt/body back (WP `_wp_restore_post_revision`) |
| `GET /admin/postdata/{id}` | `posts.rs` | JSON `{cats,tags,lang}` for Quick Edit prefill |
| `POST /admin/api/tour/save-option` | `posts.rs::save_tour_option` | tourmaster metabox ajax save (`csrf`,`post_id`,`name`,`value` → `{status,head,message}`) |
| `GET /admin/tax/new-filter`, `POST /admin/tax/new-filter`, `GET /admin/tax/{tax}` (any taxonomy registered in `terms`/`post_terms`) | `tax.rs` | term list + add form, `?s=` search |
| `GET /admin/tax/edit/{id}`, `POST /admin/tax/save`, `POST /admin/tax/delete`, `POST /admin/tax/bulk` | `tax.rs` | term mutations |
| `GET /admin/media` | `admin.rs::media_page` | library — `?mode=list|grid`, `?mime=`, `?m=YYYY/MM`, `?s=`, `?sort=`, `?paged=` |
| `GET /admin/media/new`, `POST /admin/media/upload` | `admin.rs` | Add Media (drag-drop multi-file). Upload: extension allowlist → **415** on scriptable/disallowed types (svg/html/xml), **400** empty/failed, 25MB body cap (axum 413) |
| `GET /admin/media/edit/{id}`, `POST /admin/media/save` | `admin.rs` | attachment editor (title/alt/caption/description) |
| `POST /admin/media/delete`, `POST /admin/media/bulk`, `POST /admin/media/detach`, `GET /admin/media/download/{id}` | `admin.rs` | row/bulk actions; download forces `Content-Disposition` |
| `GET /admin/media/json/{id}?ids=…` | `admin.rs::media_json` | attachment detail JSON for the grid modal (incl. prev/next) |
| `GET /admin/media/items?mime=&m=&s=&page=` | `admin.rs::media_items` | picker grid JSON (`items,total,months`) for the wp.media frame |
| `POST /admin/media/replace/{id}` | `admin.rs::media_replace` | canvas image-editor save — overwrites bytes, refreshes w/h/mime |
| `POST /admin/media/regenerate` | `admin.rs::media_regenerate` | rebuild size files + `sizes_json` for all local images from Settings→Media |
| `GET /admin/comments` | `comments.rs` | `edit-comments.php` — views All/Pending/Approved/Spam/Trash, `?s=`, `?post_id=`, `?paged=` |
| `POST /admin/comments/action`, `/bulk`, `/empty`, `/quickedit`, `/reply`, `/save`, `/screen` | `comments.rs` | row/bulk moderation, Empty Spam/Trash, quick edit, reply, edit save, screen options |
| `GET /admin/comments/reply/{id}`, `GET /admin/comments/edit/{id}` | `comments.rs` | reply form, edit form (status radios + date) |
| `GET /admin/users`, `GET /admin/user-new`, `GET /admin/user-edit/{id}`, `GET /admin/profile` | `users.rs` | `users.php`/`user-new`/`user-edit`/`profile` — admin role only except profile |
| `POST /admin/users/create`, `/save`, `/delete`, `/bulk`, `/logout-others`; `GET /admin/users/delete?ids=` (confirm) | `users.rs` | user mutations; self/last-admin guards |
| `POST /admin/users/app-pass/add`, `/app-pass/revoke` | `users.rs` | application passwords (mint/revoke) |
| `GET /admin/contact` | `contact.rs` | CF7 Contact Forms registry (`FORM_SLUGS` + counts) |
| `GET /admin/contact/form/{slug}`, `GET /admin/contact/form/{slug}/edit`, `POST /admin/contact/form/{slug}/save`, `GET /admin/contact/new`, `POST /admin/contact/create`, `POST /admin/contact/form/delete` | `contact.rs`, `forms_editor.rs` | form detail + editor CRUD |
| `GET /admin/contact/inbox`, `GET /admin/contact/inbox/{id}`, `GET /admin/contact/inbox/export` | `contact.rs` | Flamingo Inbound Messages — Inbox/Spam/Trash, `?form=`, `?s=`, `?sort=`, `?paged=`; export CSV |
| `POST /admin/contact/inbox/action`, `/bulk` | `contact.rs` | moderation via `intake::moderate` (approve/spam/trash/restore/purge) |
| `GET /admin/reviews` | `contact.rs` | review moderation list |
| `GET /admin/submissions`, `POST /admin/submissions/mod` | `contact.rs` | 301 → `/admin/contact/inbox*` (legacy aliases) |
| `GET /admin/menus?menu=` | `menus.rs` | `nav-menus.php` — selector, add-items panels, item blocks, move links |
| `POST /admin/menus/create`, `/delete`, `/add-items`, `/move`, `/save` | `menus.rs` | menu CRUD + structure tx |
| `POST /admin/menus/locations`, `POST /admin/menus/screen` | `menus.rs` | Manage Locations save (`assign-{location}` selects) + Screen Options save (`show-{field}`, `sec-{section}` checkboxes) |
| `GET /admin/widgets` | `widgets.rs` | `widgets.php` — two-column editor (Available Widgets + per-sidebar postboxes + `wp_inactive_widgets` pseudo-sidebar); **admin-only** (`edit_theme_options` parity) |
| `POST /admin/widgets/add` | `widgets.rs` | `{type, sidebar, title}` → new `{type}-{n}` instance appended to the sidebar |
| `POST /admin/widgets/save` | `widgets.rs` | `{id, title, sidebar, f_{key}…}` → schema-validated settings merge (sidebar select = move/restore); stored `html` is admin-trusted **jinja** (`{{ site_url }}`/`{{ lang }}`), dry-run rendered and rejected (`msg=badsyntax`) before write |
| `POST /admin/widgets/delete` | `widgets.rs` | `{id}` → parks in `wp_inactive_widgets` preserving settings; delete from inactive = hard delete |
| `POST /admin/widgets/order` | `widgets.rs` | `{id, dir=up\|down}` adjacent swap, or `{sidebar, ids_csv}` full re-sequence |
| widget mutations | `widgets.rs` | all set `settings.widgets_dirty=1`; dashboard shows "pending publish" + Publish until the next emit/bust clears it |
| `GET|POST /admin/settings/{section}` | `settings.rs` | `general|writing|reading|discussion|media|permalink|privacy`; permalink/category/tag base render disabled (fixed structure on static site); email controls unsupported — no mail transport |
| `GET /admin/wp/{*path}`, `GET /admin/front-css/{*path}`, `GET /admin/vendor/htmx.min.js` | `admin.rs` | vendored WP admin assets |
| `GET /wp-content/uploads/{*path}` | `mod.rs::uploads` | local copy under `www/wp-content/uploads` else redirect to `asset_origin`; nosniff; SVG/XML/HTML/JS/CSS/unknown local legacy files force download as octet-stream |

## JSON / machine routes

| Route | Purpose |
|---|---|
| `GET /health` | liveness probe, always 200 |
| `POST /api/generate`, `GET /api/generate/status` | utility-files rebuild (`sitefx::emit_site_files`: sitemap/feeds/robots/404/search/redirects.map) + trashed-row purge >30 days (WP `_wp_scheduled_delete`) + `cache/` bust — NOT a page bake; cache re-warms from traffic (`generating` flag serializes runs) |
| `GET /api/v1/posts` (alias `/api/posts`) | JSON list of posts (used by tooling) |
| `GET /api/v1/gdlr/media` | GoodLayers media list JSON |
| `POST /admin/api/gdlr/ajax` | GoodLayers page-builder ajax endpoint |
| `POST /intake/{form,comment,review}` | public intake → `submissions`/`comments`, rate-limited by `intake_rate`, CORS = `settings.site_url` + its `edit.` sibling. `/comment` writes the `comments` table directly (status decided by Discussion settings), 303-redirects to `_page` + `#comment-submitted\|held\|closed\|error`; auto-approved comments regenerate the post file in the background |
| `POST /form/{slug}`, `/comment`, `/review` | public aliases of intake |
| `GET /{*public}` (`mod.rs::public_page`) | wildcard public site: `cache/<slug>/index.html` hit → serve; miss → normalize `es/`/`{tour_base}/` prefixes → `render_slug` → atomic write-back (publish only); `/{slug}.md` + `Accept: text/markdown` serve/generate `cache/<slug>/index.md`; extension-carrying misses redirect to `asset_origin`; extension-less misses are junk-prefix-filtered + `miss_rate`-limited (60/min/IP) |

Everything else → 404.

## Post status / terms contract

- `posts.status` ∈ `publish|future|draft|pending|trash` — `future` is a scheduled publish (`published_at` in the future); the session middleware flips due rows to `publish` and rewrites their files.
- Taxonomies: `category`, `post_tag`, plus tour taxonomies (`duree`,
  `region`, `style`, `profil` / ES variants `duracion`, `region-es`,
  `estilo`, `perfil`) and any imported extras (`Activites`, `Pays`). Term
  screens work for any slug present in `terms`/`post_terms` — no whitelist.
- `terms` is the registry used by term screens; `post_terms` holds per-post
  assignments. `save_term` keeps `post_terms` names/slugs in sync on rename.
