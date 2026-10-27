---
title: "Bridge bring-up: Playground WP answers real gdlr ajax through rpress proxy"
date: 2026-10-27
summary: "Phase-07 milestone — the PHP bridge is alive end-to-end. Five runtime
facts discovered the hard way; each now encoded in playground.rs/mu-plugin.
`gdlr_core_get_skin_options` returns success JSON through
/admin/plugins/goodlayers/ajax; token fence 403s unauthenticated hits."
---

# Bridge bring-up (seam phase-07)

## What works now (live-verified on aucoeur-test)

`POST /admin/plugins/goodlayers/ajax` with `action=gdlr_core_get_skin_options`
returns `{"status":"success","option_content":"…Skin Options…"}` — the real
GoodLayers plugin answering inside Playground WP. Direct hits without the
`X-Rpress-Bridge` token get 403 from the mu-plugin fence. Plugin activates +
post seed lands on the first admin request after install.

## Runtime facts that took hours to find

1. **`--mount-before-install` is required for `wp-content/database`** —
   post-install mounts shadow the tables WP just created → infinite
   install loop (`is_blog_installed` false forever).
2. **wp-content must mount per-subdir, never wholesale** — mounting the
   whole dir (or `plugins`/`themes` wholesale) shadows the sqlite `db.php`
   drop-in and `sqlite-database-integration`; wpdb falls back to mysqli,
   `siteurl` reads empty → install loop again.
3. **Process env does not propagate into PHP-WASM** — `getenv` returns
   false. `--define NAME value` lands as PHP constants; the mu-plugin
   reads `defined('RPRESS_BRIDGE_TOKEN'/'RPRESS_BRIDGE_PLUGIN')`.
4. **Real wp.db is `wp-content/database/.ht.<hash>/.ht.sqlite`** — one
   level down under a random hash dir; resolved lazily in `pull::locate()`
   since the file only exists after WP's first write.
5. **admin-ajax `wp_ajax_*` only fires for logged-in users** — the
   mu-plugin `wp_set_current_user(1)` on `admin_init` and mints a real
   `gdlr_core_page_builder` nonce into `$_POST['security']` for
   `gdlr_core_*` actions (our admin page holds no WP nonce).

## Bug fixed in our proxy (http.rs)

Forwarded `content-length` was skipped as hop-by-hop but also suppressed
our own re-emit → the bridge server read a bodyless request and answered
`400 Connection: close` with an empty body. Now always `Content-Length:
{body.len()}`; `expect` also stripped.

## Architecture change in this session

`spawn` no longer SQL-projects into wp.db (tables didn't exist yet —
WP creates them during install). Instead `project_post` writes
`wp-content/rpress-seed/post.json` and the mu-plugin `wp_insert_post`s it
once on the first admin request (guarded by `rpress_seed_done` option).
Pull translates WP `post_id` → rpress `posts.id` via `wp_id` fallback.

## Known gaps

- First proxied request while WP is still installing gets the
  `install.php` 302 (install takes ~15-25s cold). Acceptable: spawn is
  admin-triggered; retry lands.
- A request arriving before seed's `after_setup_theme`-time activation
  can see `die('0')` once — second request is clean. Worth a spawn-time
  "warm" request to `/wp-admin/` after probe, not yet implemented.
- `post_id` is derived from the URL; if the *first* bridged request is an
  `ajax` call with no post id in path, post.json is never seeded. Phase-07
  checklist should hit an edit screen first.
- `bridges` is an in-memory map keyed `site:slug` — if the child dies out
  from under rpress (external kill), the cached port is stale and every
  proxied request degrades to the "Bridge unavailable" page until rpress
  restarts. Needs either a spawn-on-connect-failure retry or a TTL
  re-probe before reuse.

## Editor bring-up (same session, later)

The admin edit screen now runs the plugin's own JS end-to-end:

- **Asset fallback** — `/admin/wp/gdlr/<rest>` resolves
  `plugins/goodlayers-core/<rest>`, then `plugins/<rest>`, then
  `plugins/<stem>/<rest>` (covers `gdlr/tourmaster.css` →
  `plugins/tourmaster/tourmaster.css`). All framework/include/plugin
  assets + nav images serve 200.
- **Defs dump** — mu-plugin action `rpress_dump_defs` reflects
  `gdlr_core_page_builder_element::pb_elements` /
  `_wrapper::pb_elements` per-method (`get_settings`, `get_options`,
  `get_template` — elements have no `get_template`, it lives in
  `page_builder_var.template.element` shared HTML). Output:
  `templates/admin/wp/gdlr/{elements,wrappers,page-builder-var}.json`
  (73 elements, 3 wrapper types, column map 10..60).
- **icons.json** — 9 icon fonts / 6034 icons, parsed out of
  `plugins/icon-font-controls.php` (`gdlr_core_get_icon_font_list` cases).
- **JS shims** (`templates/admin/wp/gdlr/`):
  - `wp-media-shim.js` — `wp.media()` opens a picker backed by
    `/api/v1/gdlr/media`, fires `select` with a `.toJSON()`-shaped
    selection so `attachment.id`/`attachment.url` work unmodified.
  - `jquery-ready-shim.js` — `send_to_editor` (caret insert into
    `wpActiveEditor` textarea) + `QTags` stub.
  - `adapter.js` — canvas serializer (`pb_update_tempalte_val` port;
    root selector is `#gdlr-core-page-builder-container >
    [data-template]` — walking `#gdlr-core-page-builder-body` children
    gave `[{}]` because body's only child is the container div) into
    hidden `#gdlr-json` on form submit, plus an **init watchdog**:
    page-builder.js registers `new gdlrCorePageBuilder()` via jQuery
    document-ready but with our inline script placement the ready
    promise resolves without running it (handler stays registered in
    `jQuery._data(document,'events').ready`, no pageerror, silent dead
    UI). On `window.load` we run registered ready handlers if the
    Save-PB button still has no click binding.
- **Live-verified in the headless browser** (`/admin/edit/7380`):
  metabox renders (71 element tiles, wrapper/column/flexbox nav),
  live-mode canvas shows the real post preview; block-mode toggle
  works; drag Accordion/TextBox head item onto a column container
  adds the item (200ms `dragging_threshold` + `mousemove` must go to
  `document.body` — dispatching on `document` silently no-ops);
  item edit opens the options lightbox via
  `POST /admin/plugins/goodlayers/ajax` (307 → bridge) with a working
  TinyMCE; Save PB POSTs `gdlr_core_save_page_builder_data` through
  the bridge → `{"status":"success","head":"Successfully Save"}`;
  form-submit serialize produces the exact 14086-byte `body_json`.

## Bridge-scoped vs universal (scope note)

Everything under `templates/admin/wp/gdlr/` and the walker/
`get_template` DOM contract is **GoodLayers-only**. Reusable across
future builders: the `wp.media`/`send_to_editor`/`QTags` shims, the
defs-dump reflection pattern, the init watchdog mechanism, and the
`plugins/<stem>/<rest>` asset fallback pattern. A second builder is
an adapter file + a dump action + a small walker — not a rewrite.

## Files touched

- `src/admin/bridge/mu-plugin/bridge.php` — rewritten: token fence via
  constants, uid-1 identity, gdlr nonce mint, self-seed.
- `src/admin/bridge/playground.rs` — `--define` constants, per-subdir
  mounts, `--mount-before-install` db dir, `project_post` → post.json,
  test updated.
- `src/admin/bridge/pull.rs` — `locate()` for the `.ht.<hash>` path;
  post_id→rpress id translation through `posts.wp_id`.
- `src/admin/bridge/http.rs` — Content-Length fix; `expect` stripped.
- `src/admin/mod.rs` — pull site now resolves via `pull::locate`.
- `src/admin/plugins/{mod,goodlayers/mod}.rs` — `wp_plugin_file` trait
  hook + declaration (earlier in session).
- `plans/261027-scoped-plugin-seam/phase-07-gdlr-bridge.md` — status
  in-progress + findings block.
- `src/admin/mod.rs` — `wp_asset` sibling-plugin stem fallback.
- `src/admin/bridge/mu-plugin/bridge.php` — `rpress_dump_defs` action.
- `templates/admin/wp/gdlr/` — new: `elements.json`, `wrappers.json`,
  `page-builder-var.json`, `icons.json`, `jquery-ready-shim.js`,
  `wp-media-shim.js`, `adapter.js`.

## Verify

```bash
cargo check --all-targets && cargo test --lib   # clean, 36/36
rpress site serve aucoeur-test
curl -b ck -X POST localhost:8410/admin/plugins/goodlayers/ajax \
  -d 'action=gdlr_core_get_skin_options'        # {"status":"success",…}
```

## Completion (later same session) — parity gate passed, port deleted

Real-session parity checklist: 9/9 pass (metabox, element lightbox via
real plugin, preview≈public, save→reopen body_json, skins list/apply/
save, templates CRUD + sync mark, gallery lightbox + media picker,
tour element editable via real theme, public render byte-parity across
regenerations).

Then executed the deletion:

- `src/admin/gdlr_pb.rs` (2410 lines) — DELETED. Boot emitters moved to
  `src/admin/plugins/goodlayers/boot.rs` (1624 lines: DOM skeleton,
  `page_builder_var`/`html_option_val`/`gdlr_core_tmce`, canvas item
  templates — the JS contract only; no admin-ajax logic). Dead with the
  port: `ajax`/`ajax_dispatch`, `parse_form_nested`, `process_post_data`,
  all option renderers (`option_element`, `options_lightbox`,
  `combobox_options`, `radioimage_opts`, `skin_list`, `thumbnail_sizes`,
  `opt_val`, `fallback_def`, `gallery_options_lightbox`, `url_decode`,
  `insert_nested`, `collapse_numeric`, `fmt_pixel`, `icon_font_type`,
  `ok_resp`/`fail`), `FONT_LIST`.
- `src/admin/gdlr.rs` — kept (17 lines): `/api/v1/gdlr/media` feeds the
  wp-media shim's picker list; vendored JS calls it directly.
- `src/admin/mod.rs` — `gdlr_ajax` reduced to a bare 307 →
  `/admin/plugins/goodlayers/ajax`; `opt_in` fallback arm removed
  (there is no fallback left: the port is gone).
- `src/import.rs` — `active_plugins` kept on import (bridge parity
  needs the origin list).

Bridge fixes found during parity verification:

- `bridge_for` now drops + respawns entries whose `rt.healthy()` fails
  (dead-child probe) — previously a killed child meant "Bridge
  unavailable" until rpress restarted.
- `RPRESS_BRIDGE_PLUGINS` — mu-plugin re-activates extra plugins, but
  ONLY element providers: playground.rs scans mounted plugin dirs for
  `gdlr_core_page_builder_element::add_element` and passes the
  intersection with `active_plugins`. Activating all 21 origin plugins
  brought ml-slider's install wizard, which 302-hijacked every
  admin-ajax request. Final set on aucoeur-test: `tourmaster` only —
  the tour element options lightbox (114 KB of real option HTML)
  renders through the bridge.
- mu-plugin mints `gdlr_core_html_option` nonces for `gdlr_get_*`
  actions (html-option.js posts `html_option_val.nonce`, not
  `page_builder_var.pb_nonce`) — was the gallery-options 500.

Verification after deletion: `cargo check` clean, `cargo test` 36/36,
`wc -l boot.rs` = 1624 (vs 2410), live edit screen → metabox renders,
element lightbox opens via bridge, `/admin/api/gdlr/ajax` 307s to the
proxied screen.
