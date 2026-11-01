---
title: Bridge seed byte-exactness + GDLR media library
date: 2026-10-31
summary: Fixed double-wrap/cap/media failures in the php_cgi bridge; seeded attachment library; verified edit→pull round-trip byte-exact.
---

# Bridge seed byte-exactness + GDLR media library

## What happened
Working plan `elementor-faithful-seed` on the php_cgi bridge hit five compounding defects:

1. `rpress-install.php` never defined `RPRESS_BRIDGE_PLUGINS` (it lives only in wp-config.php), so tourmaster's activation hook never ran → CPT edit screens 403'd (administrator lacked edit_tours caps).
2. sqlite-db-integration's `update_option('active_plugins', …)` returned false (its `INSERT` fallback violates the option PK on an existing row), so `active_plugins` stayed `a:0:{}` — plugins never marked active despite the option write.
3. `register_activation_hook(__FILE__)` inside a symlinked plugin dir registers under the resolved realpath, so `activate_tourmaster/tourmaster.php` never matched; needed a second `do_action` under `activate_<realpath>`.
4. `update_post_meta($pid, $k, wp_slash($v))` still double-wrapped already-serialized GDLR blobs (`is_serialized` rejects `s:N` prefixes / unbalanced braces) — observed `s:79252:"s:79241:"a:19:…"` compounding on every respawn+pull. Bypassed meta machinery entirely: `$wpdb` insert/update byte-exact.
5. WP 6.8 defaulted to the block editor: post.php mounted an empty React shell, hid `#wpbody`, left metaboxes unmounted → blank page. Forced `use_block_editor_for_post{,_type}` = false and stripped `is-fullscreen-mode`/`block-editor-page` + unhid `#wpbody-content` on `admin_footer`.

For the GDLR media modal, `wp.db` had zero attachment rows so `query-attachments` returned empty even though the modal opened. `project_post` now emits `attachments` (media table → WP post objects, guid = relative uploads path) + `attachment_meta` (`_wp_attached_file`, `_wp_attachment_metadata` PHP-serialized via `crate::php_serialize::serialize`, alt). Both PHP seeders upsert them keyed on `wp_posts.ID`. `media-form` nonce minted for `upload-attachment`/`media-create-image-subsizes`; CPT caps granted via `user_has_cap` filter (tour/room/car/coupon/service + `manage_*`) instead of relying on fragile activation hooks.

## Verified
- aucoeur tour edit: 26 GDLR items rendered, `gdlr-core-page-builder` meta = 78,626 bytes raw `a:19:` in wp.db (identical to rpress.db).
- Round-trip: injected marker into a text-box `data-value` → `#publish` → wp.db bytes → pull → rpress.db (78,650) → reverted → byte-exact 78,626.
- Media modal opens; Media Library lists 80 real attachments with thumbnails; select→insert fills the element field.
- blueskies Elementor: navigator = 82 elements = seeded tree; iframe DOM = 82 `.elementor-element` nodes; `_elementor_data` = 20,162B valid JSON, byte-identical rpress.db↔wp.db. `_elementor_page_assets` quadruple-wraps (`s:"s:"s:"s:"…`) in rpress.db unwrapped back to raw `a:2:`.
- `cargo test --lib`: 76/76 pass.
- Code-reviewer: PASS on all five criteria; one real edge fixed — `bridge_pull__select_post_fields` now excludes `attachment`/`revision`/`auto-draft` (a locally-uploaded media row with `wp_id=NULL` could collide with a low real `wp_id` and overwrite content on pull).

## Decision
Meta seeding writes `$wpdb` directly in both `rpress-install.php` and `mu-plugin/bridge.php` (WP_INSTALLING vs admin_init lifecycle — duplication is genuine, not drift). `update_post_meta`'s unslash/serialize pipeline is unsuitable for byte-exact blobs.

## Next steps
- Cosmetic: the Elementor preview iframe's spinner overlay persists (parent ready-event race) — content renders fine underneath; revisit only if it blocks UX.
- `bridge_pull__select_post_fields` post_type filter could also gain `nav_menu_item`/`wpcf7_contact_form` exclusions if other kinds appear in wp.db.

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
