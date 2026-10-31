---
title: "Elementor on existing PhpBridge"
description: "Run real Elementor (this tree: 4.3.x, no Pro) inside the existing PhpCgi sidecar. GDLR stays the reference adapter. v1 = blueskies only."
status: pending
priority: P1
effort: 4d
branch: main
tags: [feature, plugins, php-bridge, elementor]
blockedBy: []
blocks: []
created: 2026-10-31
---

# Elementor on PhpBridge

Source of truth for this plan: Elementor PHP in
`work/blueskies/www/wp-content/plugins/elementor/` (4.3.x, **no** `elementor-pro`)
+ rpress `src/admin/bridge/` + GoodLayers adapter.

## Outcome

On blueskies, a page that already has `_elementor_data` opens **Edit with Elementor**
in rpress admin. The canvas is the real plugin, same PhpCgi sidecar GDLR uses.
Save updates rpress.db + that slug’s public HTML **without** fetching vinhhyblueskies.com.

WPBakery/Divi are **not** v1. The transport fixes (query, JSON rewrite, workers)
are shared so the next builder is an adapter file.

## What Elementor actually does (from this plugin tree)

```mermaid
sequenceDiagram
  participant Browser
  participant Rpress as rpress /admin/plugins/elementor
  participant PHP as php -S WP
  Browser->>Rpress: GET post.php?post=ID&action=elementor
  Rpress->>PHP: + X-Rpress-Bridge
  Note over PHP: editor.php init() via admin_action_elementor<br/>print_root_template(); die
  PHP-->>Browser: editor shell HTML
  Browser->>Rpress: POST admin-ajax.php action=elementor_ajax
  Note over PHP: nonce elementor_ajax<br/>actions JSON: get_document_config, save_builder, …
  Browser->>Rpress: GET /?p=ID&elementor-preview=ID
  Note over PHP: frontend iframe, not wp-admin
```

| Surface | WP entry | Elementor code | Notes |
|---|---|---|---|
| Editor shell | `wp-admin/post.php?post={id}&action=elementor` | `core/editor/editor.php` `admin_action_elementor` → `init()` | Needs **query string**. `http.rs` currently **rejects `?`**. |
| Ajax bus | `POST wp-admin/admin-ajax.php` `action=elementor_ajax` | `core/common/modules/ajax/module.php` nonce **`elementor_ajax`** | Same file GDLR already proxies as `…/ajax`. Actions in JSON `actions`. |
| Save | ajax action `save_builder` | `documents-manager.php` `ajax_save` → `Document::save` | Writes `_elementor_data` JSON (`wp_slash(wp_json_encode)`), then `save_plain_text`. |
| Load config | ajax `get_document_config` | same manager | Needs the post in wp.db (seed). |
| Preview iframe | **frontend** `get_permalink()` + `?elementor-preview={id}&ver=` | `document.php:1000` forces **empty permalink_structure** so URL is `/?p=ID&elementor-preview=ID` | Not wp-admin. php -S must serve `index.php?p=`. |
| Heartbeat | `admin-ajax.php?action=heartbeat` every 15s | `editor.php` heartbeat_settings | Queues on single-worker php -S. |
| REST `/wp-json/elementor/v1/*` | `index.php` rewrite | atomic widgets, global classes, AI, cache | **php -S has no WP rewrite router** (`php_cgi.rs` `-S -t docroot` only). GDLR never needed this. Core 4.3 editor save path is **ajax**, not REST. Treat REST as phase-1.5 if canvas JS 404s `/wp-json`. |

### What save writes (do not guess `body_html`)

From `document.php` + `includes/db.php`:

| Destination | Key / path | Content |
|---|---|---|
| post meta | `_elementor_data` | JSON array of elements |
| post meta | `_elementor_edit_mode` | `builder` |
| post meta | `_elementor_page_settings` | page settings |
| post meta | `_elementor_css` | CSS meta |
| post meta | `_elementor_version`, `_elementor_template_type`, `_elementor_controls_usage`, `_elementor_page_assets` | small |
| `wp_posts.post_content` | `save_plain_text()` | **widget plain text**, not rendered HTML |
| filesystem | `uploads/elementor/css/post-{id}.css` | regenerated after save (`Post_CSS::delete` then frontend rebuild) |

**Therefore `PullTable::BodyJson` / copying `post_content` → public HTML is wrong.**
Public rpress pages today are **harvest of PHP-rendered frontend**. After a local save,
re-render by GETting the bridge frontend `/?p={wp_id}` (no `elementor-preview`) and
storing that HTML (`harvested_pages` or `body_html`), then skip origin harvest.

Blueskies DB: **56** posts with `_elementor_data` (pages like `contact-2`, `terms-conditions`).
Blog `nui-chua-douc-langur` is **not** Elementor (classic + tevily meta) — screenshot #3
was missing **theme CSS**, not missing the builder.

No `elementor-pro` in `www/wp-content/plugins/`. Tevily widgets come from `tevily-themer`
(already activated via `RPRESS_BRIDGE_PLUGINS` if listed in origin `active_plugins`).

## What the PhpBridge core already does (reuse, don’t fork)

| Piece | Location | GDLR use | Elementor need |
|---|---|---|---|
| Sidecar `php -S` + sandbox | `bridge/php_cgi.rs` | yes | same process |
| Bearer `X-Rpress-Bridge` | `mu-plugin/bridge.php` | every request | browser never sends it → **all** hops via `bridge_dispatch` |
| Mount `/admin/plugins/{slug}/{*rest}` | `mod.rs:356` | `ajax` | `wp-admin/post.php`, `wp-admin/admin-ajax.php`, `/?p=` |
| `bridge_path` | adapter | `ajax` → `admin-ajax.php` | map those three |
| Query string | `http.rs:30` **forbid `?`** | unused (POST body) | **must forward** `post`, `action`, `p`, `elementor-preview` |
| HTML Location/body rewrite | `plugin_dispatch.rs:250-286` | WP_HOME port → mount | also **JSON** (`save_builder` return `urls`) |
| Seed one post + all meta | `project_post` | gdlr meta | `_elementor_data` is a **JSON string** — seed as string, `update_post_meta` OK |
| `WriteSpec` PostMeta LIKE | `pull.rs` | `gdlr-core-%` | `_elementor%` |
| Nonce mint | mu-plugin **gdlr-only** | `gdlr_core_*` | mint `_nonce` / `_ajax_nonce` = `wp_create_nonce('elementor_ajax')` when `action=elementor_ajax` |
| uid 1 | mu-plugin | yes | required (`is_editable_by_current_user`) |
| Single worker | `php_cgi.rs:14` | sequential ajax | preview iframe + heartbeat **parallel** → `PHP_CLI_SERVER_WORKERS≥4` or queue |
| Assets `/wp-content/plugins/elementor/**` | `serve.rs` www | n/a | already 200, do not proxy |

`opt_in`: Elementor env-gated (`PLUGINS_PHP=elementor`). GDLR stays always-on.

## Transport contract (shared; do this before the adapter)

1. **Query:** split path vs query; path keep today’s charset reject; query allow `A-Za-z0-9_=&%-`.
2. **WP_HOME / SITEURL / PLUGIN_URL** = `http://127.0.0.1:{rpress-bind}/admin/plugins/elementor`
   so `admin_url('post.php')` and `admin_url('admin-ajax.php')` and `/?p=` already sit on the mount.
3. Rewrite **html + json + javascript** `WP_HOME` → mount (today html-only).
4. **php router:** `php -S … router.php` that `require index.php` for non-files
   (`/wp-json/…`, `/?p=`). Without it REST 404s; `/?p=` may still hit `index.php` on php -S.
5. `PHP_CLI_SERVER_WORKERS=4`.
6. GDLR regression: `POST /admin/plugins/goodlayers/ajax` still 200.

## Adapter (`src/plugins/elementor/`, follow `goodlayers/mod.rs`)

```
slug            = "elementor"
backend        = PhpBridge
wp_plugin_file  = "elementor/elementor.php"
write_specs     = PostMeta "_elementor%"
                  Settings "elementor_%" → plugin:elementor:
bridge_path     = ajax            → /wp-admin/admin-ajax.php
                  wp-admin/post.php → /wp-admin/post.php  (query preserved)
                  "" or index     → /index.php            (preview /?p=)
```

Post-edit button (not GDLR metabox): if `opt_in` && row in `post_meta` `_elementor_data`
→ link `/admin/plugins/elementor/wp-admin/post.php?post={wp_id}&action=elementor`.
Leave `posts.rs` `body_is_gdlr` branch **untouched**.

## After save (render authority)

1. Existing pull copies `_elementor%` meta (wp_id → rpress id).
2. New: GET bridge `/?p={wp_id}` (frontend, uid 1) → write `harvested_pages` / `body_html`.
3. `generate.rs`: if `_elementor_data` && stored HTML nonempty → **do not**
   `harvest::page(&legacy_origin, …)` (that clobber is why local edits would vanish).
4. Regen `cache/{slug}/index.html` from that HTML.
5. CSS: `uploads/elementor/css/post-{id}.css` is under `www` (symlink); public serve already reads it.

Do **not** treat `post_content` as the public page (`save_plain_text` is plaintext).

## Non-goals

- Elementor Pro, template kit cloud, AI, connect.elementor.com (sandbox denies outbound anyway)
- Reimplementing widgets in Rust
- WPBakery / Divi / Bricks in this plan
- Changing GDLR boot / tinymce
- Making `nui-chua-douc-langur` an Elementor page (it isn’t)

## Risks (code-backed)

- Preview iframe + shell both occupy php workers; without `PHP_CLI_SERVER_WORKERS` the iframe waits on the shell forever if the shell request hasn’t finished — shell `die`s after HTML, so order is OK if the iframe starts after load. Heartbeat during iframe is the real overlap.
- Theme-builder chrome (header/footer `elementor_library`) is **other posts**. Seed is one post. Canvas of a page may miss header until those ids are projected. Defer unless spike shows empty chrome.
- `update_post_meta` on seed: `_elementor_data` must remain a JSON **string**. Don’t `json_decode` then pass an array (double-serialize).
- `unfiltered_html` for uid 1: `wp_install` admin has it; keep it or `kses_post_deep` strips widgets.

## Acceptance

- [ ] `http.rs` accepts `post.php?post=20&action=elementor`; still rejects `..` and CRLF
- [ ] Blueskies `contact-2` (has `_elementor_data`): canvas HTML 200, no 403, ajax `get_document_config` 200
- [ ] Save changes `_elementor_data` length/content in `rpress.db`
- [ ] Public cache for that slug changes; `generate` does not HTTP origin for it
- [ ] Aucoeur GDLR metabox + ajax save unchanged
- [ ] No `PLUGINS_PHP=elementor` → no button
- [ ] `cargo test` + `check-layers.sh`

## Phases

| # | Phase | Status |
|---|-------|--------|
| 0 | Spike: query-forward + canvas 200 + ajax 200 on blueskies | Pending |
| 1 | Transport: query, JSON rewrite, WP_HOME=mount, router.php, workers | Pending |
| 2 | Elementor adapter + button + nonce mint | Pending |
| 3 | Frontend GET after save → stored HTML; generate harvest-skip | Pending |
| 4 | `sites/blueskies.env PLUGINS_PHP=elementor` + smoke | Pending |

**Go/no-go after phase 0.** If php -S deadlocks on preview iframe, stop and switch runtime (php-fpm) before writing the adapter.
