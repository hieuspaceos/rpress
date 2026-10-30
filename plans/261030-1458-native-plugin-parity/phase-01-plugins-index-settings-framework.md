# Phase 01 — Plugins index + settings framework

## Overview

- **Priority:** P1 — the `/admin/plugins` list is the front door every
  later phase hangs a screen off.
- **Deliverable:** `/admin/plugins` index page + `PluginDescriptor` on
  `PluginAdapter` + settings screen plumbing (route, save helper, regen
  helper) + first screen stub.

## Requirements

1. `PluginAdapter` gains:

   ```rust
   /// Display name + one-line blurb for the plugins index.
   fn descriptor(&self) -> PluginDescriptor { ... }
   /// Option key prefixes this adapter's settings screen edits;
   /// used by the index to warn on unclaimed keys.
   fn settings_prefixes(&self) -> &'static [&'static str] { &[] }
   ```

   `PluginDescriptor { name: &'static str, blurb: &'static str }`.

2. New route `GET /admin/plugins` (exact path, registered **before**
   `/admin/plugins/{slug}/{*rest}` so `plugins` isn't swallowed as a
   slug — verify dispatch order) renders a WP-parity table:

   | Plugin | Backend | Actions |
   |---|---|---|
   | name + blurb | Native / Bridge badge | Settings · Open (bridge) |

   - rows for every registered adapter (even `kinds()`-empty ones);
   - **also** rows for the 4 bridging candidates so the user sees the
     whole plugin surface: entries come from `active_plugins` option
     parsed via `php_serialize`, unioned with registered adapters —
     unadopted active plugins render with backend `—` and no actions
     (visible, honest, no fake controls).
   - nav: add "Plugins" to the dock menu (`src/admin/mod.rs` menu list,
     `dashicons-admin-plugins`, tint `--violet`).

3. `register_settings_screen(slug)` helper — the boilerplate every
   per-plugin settings screen shares:

   ```rust
   /// GET renders a form for `fields` (name/label/input-type/current
   /// value from settings); POST writes every posted field via
   /// `INSERT OR REPLACE INTO settings` in one tx, then calls
   /// `regen_site` unless the screen opts out.
   ```

   FieldSpec: `key`, `label`, `kind` (Text/Textarea/Select/Checkbox/
   SerializedTextarea for PHP-serialized values that round-trip
   through `php_serialize`), `choices` for selects. This is *not* a
   schema — one screen can bypass the helper for bespoke editors.

4. `regen_site(sc)` factored out of the existing `admin::regen_*`
   plumbing so a settings save invalidates the generated cache the same
   way a post save does.

5. Index page marks `Backend::PhpBridge` rows with a badge + links to
   the adapter's existing first `ScreenDef` (tourmaster screens,
   goodlayers boot) rather than 404.

## Success criteria

- `GET /admin/plugins` 200s, lists: post/page core omitted, tourmaster,
  goodlayers, theme_traveltour, theme_tevily + any active-plugin rows
  without adapters (chaty etc. until later phases).
- Backend badges render `Native` for Rust adapters, `Bridge` for
  `Backend::PhpBridge`.
- `/admin/plugins/<slug>/settings` dispatched through `screen_for` for
  the first adapter that registers one in phase 2.
- No behavior change on existing routes (dispatch order regression
  test: `/admin/plugins/tourmaster/*` still resolves).

## Risk

- `plugins` colliding with the `{slug}` param — keep the static route
  *first* in the axum chain; axum prefers static segments anyway, but
  the test proves it.
- `active_plugins` parsing: it's `a:N:{i:0;s:N:"dir/file.php";…}` —
  `php_serialize::unserialize` already exists; reuse it, tolerate
  missing key (fresh DB → registered adapters only).
