# Phase 03 — Frontend gaps: metaslider, CF7 extensions, head-footer

## Overview

- **Priority:** P1 — dead shortcodes and missing form widgets on the
  public site.
- **Deliverable:** `[metaslider]` expands to real slides;
  `[country-phone]` gets a country select; CF7 honeypot; verify
  `auhfc_*` injection points.

## Requirements

### Metaslider

1. Import check: `ml-slider`/`ml-slide` kinds are already in `posts`
   (29 slides). Slides keep their `post_meta` (`ml-slider_settings`,
   ordering via menu order/attachment).
2. `expand_shortcodes` arm for `"metaslider"`: resolve `id` → the
   `ml-slider` post → ordered `ml-slide` children (kind or
   `post_parent` link — inspect actual schema; `ml-slide` rows carry
   `post_parent=ml-slider id`) → emit:

   ```html
   <div class="metaslider metaslider-6426 ml-slider">
     <ul class="slides"><li><img src="…" alt="…"/></li>…</ul>
   </div>
   ```

   Fade via the site's existing slider CSS if present in `www/`; else
   `display:block` first-slide + a 20-line inline fade JS — no external
   dependency.
3. Post meta `ml-slider_settings` (serialized: width/height/effect)
   drives wrapper style where cheap (`max-width`), else ignored.

### CF7 country-phone

1. Form markup render: `[country-phone* …]` in `forms.markup` parses
   to `<select>` of ~10 common country codes + `<input type="tel">` —
   mirror the plugin's data attrs (`data-country`, `data-preferred`).
   Default list comes from the form tag attrs, fallback `+84,+33,+34,+1`.
2. On submit, `/intake/form` concatenates code+number into the field's
   stored name — matches what CF7 saves (same field name the markup
   declares).

### CF7 simple-recaptcha → honeypot

1. Markup render appends `<input name="_wpcf7_honey" style="display:none">`.
2. `/intake/form`: if `_wpcf7_honey` non-empty → return 200 (silent
   drop, bots get no signal).
3. Settings key `cf7_honeypot_enabled` (new, default on) — surfaced in
   the phase-4 CF7 settings screen.

### head-footer-code audit

1. `auhfc_settings_sitewide` is already imported. Check whether render
   injects it — grep `auhfc` in `src/render*`; if absent, add the
   `head`/`footer`/`article` buckets into `site_data` `header`/`footer`
   injection points.

## Success criteria

- The 4 `[metaslider]` posts render actual images (spot-check slug
  containing `metaslider id="6426"`).
- A CF7 form page containing `[country-phone]` renders a `<select>`
  with the same name the plugin uses.
- POST `/intake/form` with `_wpcf7_honey=bot` → 200, no `submissions`
  row. Clean POST → row lands.
- `parity` still ≥ 0.8.
- No change on pages without these shortcodes (byte-identical output
  on 3 sampled slugs).

## Risk

- `ml-slide` linkage may live in `post_meta` (`_thumbnail_id`, ordered
  via `menu_order`) — inspect the actual rows before writing the
  resolver; do NOT guess the join.
- Honeypot field name must not collide with real CF7 field names —
  prefix `_wpcf7_` reserved.
