---
phase: 7
title: "gdlr_pb → PhpBridge cutover"
status: pending
priority: P1
effort: "3d"
dependencies: [6]
---

# Phase 7: GoodLayers via PhpBridge — delete the 2412-line port

## Goal

The vendored `page-builder.js`/`html-option.js` keep working unchanged,
but their ajax now reaches the real GoodLayers plugin running on the
bridge WP. `src/admin/gdlr_pb.rs` (2412 lines) is deleted; `html_decode`
survives as a standalone helper for public render. The Rust port was
already confirmed replaceable by the user — editors actively use the
builder, so the bridge must be *feature-parity before deletion*, not
after.

## Findings covered

- **T27** — Boot surface: `gdlr_pb::boot` emits the builder DOM +
  `page_builder_var` (`src/admin/gdlr_pb.rs:1794-1874`), `asset_tags`
  (`:1932-1985`) vendored asset `<script>`/`<link>` tags, editor
  integration at `src/admin/posts.rs:994-1077` (`body_is_gdlr` gate,
  gdlr metabox div, `gdlr-json` hidden field at `:1255`).
- **T28** — Ajax surface: `POST /admin/api/gdlr/ajax`
  (`src/admin/mod.rs:193` → `gdlr_pb::ajax` `:2100-2110` →
  `ajax_dispatch` `:2113-2299`) with 13 WP actions:
  `gdlr_core_get_pb_options`, `gdlr_core_get_skin_options`,
  `gdlr_core_update_skin`, `gdlr_core_get_pb_element_preview`,
  `gdlr_core_get_pb_wrapper_preview`, `gdlr_core_save_page_builder_data`,
  `gdlr_core_pb_update_option`, `gdlr_core_{get,save,remove}_pb_{custom,sync}_template`,
  `gdlr_get_gallery_options`. Also `GET /api/v1/gdlr/media`
  (`src/admin/mod.rs:192`, `src/admin/gdlr.rs:4`) for the upload picker.
- **T29** — Settings keys the port owns: `gdlr-core-skins`,
  `gdlr-core-skins-value`, `gdlr-core-pb-template-page`,
  `gdlr-core-pb-templates`, `gdlr-core-pb-custom-template`,
  `gdlr-core-pb-sync-template` (`src/admin/gdlr_pb.rs:2167-2293`) —
  WriteSpec whitelist = `post_meta` `gdlr-core-page-builder`/`gdlr-core-%`
  + `settings` `plugin:goodlayers:%`.
- **T30** — `html_decode` (`src/admin/gdlr_pb.rs:1308-1335`) used by
  public render via `html_decode_pub` (`:1303`, called
  `src/render.rs:2019`) — the ONLY piece that must survive; extract to
  `src/html_entities.rs` or `src/render` helper.

## Key insights

- **The bridge replays admin-ajax verbatim.** Vendored JS posts
  `action=...&security=...&value[...]` form bodies — the same wire
  format `parse_form_nested` (`src/admin/gdlr_pb.rs:1989`) decodes
  today. The GoodLayers adapter's `/admin/plugins/goodlayers/ajax`
  screen just proxies the raw body to bridge WP `admin-ajax.php` —
  no parsing needed on the rpress side at all. That is the core
  simplification: delete `ajax_dispatch` entirely.
- **Boot = thin shim page.** The edit screen still needs the
  `gdlr-core-page-builder` DOM skeleton + `page_builder_var` +
  `html_option_val` localization. Two options: (a) keep ~200 lines of
  boot/asset emission in the adapter (the JS-facing contract — DOM ids
  and JS vars — is what the port reimplements; the *heavy* parts being
  deleted are the option-element renderers and ajax handlers); (b)
  proxy the boot HTML from bridge WP. Decision: **(a)** — the boot
  skeleton is ~150 lines of static templates + the two JSON vars;
  ajaxurl inside them points at `/admin/plugins/goodlayers/ajax`
  (rewrite of `src/admin/gdlr_pb.rs:1801,1975,2332`). Everything else
  dies.
- **Save path unchanged publicly**: JS save action
  `gdlr_core_save_page_builder_data` lands on bridge WP, which writes
  `gdlr-core-page-builder` meta in wp.db; the pull copies it to
  rpress.db `post_meta`. `save_body_json` (`src/db/...`, called at
  `src/admin/gdlr_pb.rs:2228`) must remain reachable — the pull's
  WriteSpec includes `posts.body_json` for that one action ONLY:
  add `{posts.body_json, post_id scope}` to goodlayers' WriteSpec —
  justified: it is exactly what the Rust port wrote, same authority.
  Alternatively the adapter exposes `save_body_json` as a declared
  capability rather than a generic table write — prefer this: capability
  list `["save_body_json", "post_meta:gdlr-core-%",
  "settings:plugin:goodlayers:%"]` — mechanical and tighter (R3).
- **Nonce** (R6): `page_builder_var.security` value changes from
  rpress CSRF to a bridge-issued nonce; the proxy passes the bridge
  token; rpress auth check happens at the outer `/admin/plugins/`
  handler before proxying (`need_auth` precedent at
  `gdlr_pb.rs:2101-2107` stays, now in core dispatch).
- **`fallback_def`** (`src/admin/gdlr_pb.rs:2125-2140`, theme-defined
  elements like `tour`) — the real GoodLayers+theme on bridge WP owns
  this natively; deletion removes the workaround.
- Deletion scope: `src/admin/gdlr_pb.rs` and `src/admin/gdlr.rs`
  (media list → bridge WP media, or keep tiny rpress media JSON —
  check which picker the JS actually calls; `gdlr.rs:4` is only 20
  lines, keep if trivial).

## Files to modify

- `src/admin/plugins/goodlayers/` (new) — `mod.rs`
  (`Backend::PhpBridge`, `admin_screens`: `ajax` proxy +
  `boot(post_id)`; `WriteSpec` capabilities), `boot.rs` (~150 lines:
  DOM skeleton + `page_builder_var` + `html_option_var` + `asset_tags`
  — moved and slimmed from gdlr_pb), vendored JS/CSS stay in
  `www/wp-content/plugins/gdlr/**`.
- `src/admin/gdlr_pb.rs` — DELETED. `src/admin/gdlr.rs` — deleted or
  20-line keep (decide on JS call inspection).
- `src/admin/posts.rs` — `gdlr` gate (`:994-1077`) now calls
  `plugins::goodlayers::boot(...)` via adapter `editor_boot` hook;
  `gdlr-json` hidden field + save flow unchanged.
- `src/admin/mod.rs` — route `/admin/api/gdlr/ajax` → 307 to
  `/admin/plugins/goodlayers/ajax` (or repoint the JS var; do both —
  shim + new ajaxurl).
- `src/render.rs:2019` — `html_decode` import from new location.
- `src/html_decode.rs` (new, ~30 lines) or move into `php_serialize`-adjacent util.
- `src/db/settings.rs` — goodlayers legacy keys → `plugin:goodlayers:` list.

## Implementation steps

1. Extract `html_decode` → new util; update `render.rs:2019` + delete
   `html_decode_pub`. (Can land standalone first.)
2. Create `goodlayers` adapter: boot.rs moved+slimmed; ajax screen =
   `ProxyTo` (phase 06 machinery); `WriteSpec` capabilities list.
3. `ajaxurl`/`ajax_url` rewrites in boot vars → new endpoint; 307 shim
   on old route; check `page_builder_var`/`gdlr_core_pbf`/`html_option_var`
   emission sites (`:1801,:1975,:2332`).
4. Edit screen: `body_is_gdlr` branch calls adapter boot; confirm the
   `gdlr-json`→`save_body_json` flow still lands via bridge save action.
5. WriteSpec wire-up: body_json capability + meta/settings prefixes;
   verify pull copies post-save (edit→save→reopen roundtrip).
6. Feature-parity checklist on real site (below) — ONLY THEN delete
   `gdlr_pb.rs`.
7. Delete `ajax_dispatch`, option renderers, `parse_form_nested` (unless
   reused — grep before deleting; it is `pub(crate)`).

## Verification

```bash
cargo check && cargo test
grep -rn 'gdlr_pb' src/   # expect: gone after step 6
wc -l src/admin/plugins/goodlayers/boot.rs   # ~150 lines vs old 2412
```

Feature-parity checklist (real site, real editor session):
- [ ] Page Builder metabox renders on a gdlr page/post edit screen
- [ ] Element add/edit lightbox opens with correct option fields
- [ ] Element preview matches public output
- [ ] Save → reopen: layout persisted (`posts.body_json` updated)
- [ ] Skins: list, apply, save
- [ ] Templates: save custom, insert, remove; sync-template mark
- [ ] Gallery options lightbox; media picker works
- [ ] Tour elements (theme-defined) still editable — now via real theme
- [ ] Public render unchanged (byte-parity on regenerate)

## Todos

- [ ] `html_decode` extracted (independent micro-PR)
- [ ] goodlayers adapter + slim boot + ajax proxy
- [ ] ajaxurl rewrites + 307 shim
- [ ] Edit-screen integration via adapter hook
- [ ] WriteSpec capabilities enforced in pull
- [ ] Parity checklist passed
- [ ] `gdlr_pb.rs` (+`gdlr.rs` if unused) deleted; `parse_form_nested`
      grep-checked and deleted if orphaned

## Risks

- **Parity before deletion** is the hard gate — checklist must be
  signed off on a real editor session; `gdlr_pb.rs` stays in tree
  (compile-gated or plainly present) until it passes.
- Bridge save→pull latency: editor hits Save then reloads fast — pull
  runs synchronously in the same request (phase 06), so the data is in
  rpress.db before the response returns; no window.
- Vendored JS may pin other absolute URLs — full grep of
  `www/wp-content/plugins/gdlr/**/*.js` for `/admin/api/` and
  `admin-ajax` during step 3.
- `body_is_gdlr` (`src/db/misc.rs:868`) stays core — it detects body
  *format*, not plugin ownership; document why it's exempt.
