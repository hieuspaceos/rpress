---
phase: 5
title: "GDLR media modal fix + browser check"
status: pending
priority: P2
effort: "2h"
dependencies: [4]
---

# Phase 5: GDLR media modal fix + browser check

## Goal

Clicking "add image" inside a GDLR element lightbox (or any
`gdlr-core-upload-image-button`) opens the WP media modal, lists
attachments, and a selection fills the element's image field.

## Facts verified (do NOT re-derive)

- The click path: `html-option.js:544` → `.gdlr-core-upload-image-button`
  → `wp.media({title, button})` → `frame.open()`. Requires `wp.media`
  global = `wp-includes/js/media-models.js` + `media-views.js` +
  `wp.mediaelement` + `wp-backbone` + `wp-util` (all enqueued by
  `wp_enqueue_media()`).
- `wp_enqueue_media()` is called by the plugin only inside
  `gdlr_core_html_option()` (html-option.php:22) when rendering the
  option HTML — the element lightbox reuses that markup but may be
  rendered before/without the enqueue on the bridge page.
- Media frame fetches attachments via `admin-ajax.php`
  `action=query-attachments` — verified NO nonce required in WP core
  (ajax-actions.php:3032, capability check only; bridge uid 1 =
  administrator → `upload_files` held). The two media actions that DO
  check `check_ajax_referer('media-form')` are `upload-attachment` and
  `media-create-image-subsizes` (:2519,:2580) — uploads, not browsing.
- mu-plugin mints nonces only for `gdlr_core_*` and `elementor_ajax`;
  `upload-attachment` would die('-1') if uploads are attempted.
- Possible primary cause: `wp.media` undefined — `wp_enqueue_media()`
  runs only inside `gdlr_core_html_option()` (html-option.php:22);
  the element lightbox may render without it on the bridge screen.
  Secondary: `wp-includes/js/media-*.js` 404 through the proxy, or
  attachments query returns empty (wp.db has no media library rows —
  only the seeded post).

## Design decision required (plan-time finding)

`query-attachments` on the bridge returns nothing meaningful: wp.db
contains ONLY the seeded post + revisions — zero `attachment` posts,
no `wp_posts` media rows, no uploads dir in wp.db. Even if `wp.media`
loads and the modal opens, the library is EMPTY and uploads need
`upload-attachment` nonce + a writable uploads path. Decide ONE:
  a) Seed the site's attachment posts into wp.db at spawn (extend
     project_post to emit `attachments.json`; mu-plugin inserts them
     as post_type=attachment with guid/URL pointing at the public
     `/wp-content/uploads/` path) — read-only browse is enough for
     "add image" UX.
  b) Seed on-demand per element (lazy attachment fetch via a new
     ajax pass-through) — more code, same result.
  c) Accept empty library + enable uploads only — user uploads via
     bridge (nonce for `upload-attachment` must be minted).
Recommend (a): cheap (post_meta rows already in rpress.db — pull the
attachment posts + `_wp_attachment_metadata`/`_wp_attached_file` into
the same seed projection), matches the "wp.db disposable, rpress.db
truth" model.

## Steps

1. Reproduce: open tour 6416 edit → open a text/image element → click
   "Add Image" → capture console errors + the failing network request
   (HAR/`tab.requests`) before touching code.
2. Fix per evidence, most likely:
   - `wp.media` undefined → enqueue `wp_enqueue_media()` on bridge
     admin pages via `admin_enqueue_scripts` in the mu-plugin.
   - `media-form` nonce for `upload-attachment` /
     `media-create-image-subsizes` → extend the mu-plugin's admin_init
     nonce minting (same pattern as `gdlr_core_*`).
   - `wp-includes/js/media-*.js` 404 → check `bridge_path` rewrites.
   - THEN implement decision (a): seed attachment posts into wp.db so
     the library lists images (extend `project_post`/seed JSON to
     include attachments + their meta).
3. Re-test in browser: modal opens, attachments list, pick an image →
   field filled; save element.

## Verification

- Modal opens with populated library (not empty/blank frame).
- Selected attachment URL lands in the element field; save persists
  (post_meta pulled to rpress.db).
- Console clean of media/attach ajax errors.
