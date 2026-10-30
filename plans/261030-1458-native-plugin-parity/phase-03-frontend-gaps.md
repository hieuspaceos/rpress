# Phase 03 — Frontend gaps: metaslider, CF7 extensions, head-footer

## Overview

- **Priority:** P1 — dead shortcodes and missing form widgets on the
  public site.
- **Deliverable:** `[metaslider]` expands to real slides;
  `[country-phone]` gets a country select; CF7 honeypot; verify
  `auhfc_*` injection points.

**Audit additions (found during cook):**
- `forms_markup::KNOWN` lacks `phonetext`/`countrytext`/
  `cf7sr-turnstile`/`hidden`/`dynamictext` — `parse()` errors →
  `submit_form` returns `bad markup` for EVERY form carrying them
  (submissions table: 0 rows). Parser must accept these types first.
- `[contact-form-7]` is hardcoded to one captured template
  (`cf7_78de6b9.html`) — 7 different forms all render Rappel Gratuit.
  Replace with a dynamic `forms.markup` → inputs renderer keyed by
  the shortcode `id` → `forms.slug`/`wp_id`.

## Requirements
### Metaslider

1. Import check: `ml-slider`/`ml-slide` kinds are already in `posts`
   (29 slides). Slide→slider linkage is the **`ml-slider` taxonomy**:
   `post_terms` rows map each `ml-slide` to a term whose `slug` is the
   `ml-slider` post's `wp_id` (verified: `post_terms.post_id=5914 →
   taxonomy=ml-slider, slug='5912'`). Order = `posts.menu_order`.
   Image = `posts.featured_media_id` (from `_thumbnail_id` meta).
   `post_parent` is empty — do NOT join on it.
2. `expand_shortcodes` arm for `"metaslider"`: resolve `id` →
   `posts.wp_id` of the `ml-slider` post → its `ml-slider`-taxonomy
   term members, ordered by `menu_order` → emit:

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

1. `submit_form` (`src/intake.rs:107`) already silently drops
   submissions posting a non-empty `_gotcha` — the intake side is done.
   The gap is the **markup side**: form render must append
   `<input name="_gotcha" style="display:none">` (the field name is
   fixed by the existing intake check — do NOT invent `_wpcf7_*`).
2. Settings key `cf7_honeypot_enabled` (new, default on) — surfaced in
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
- POST `/intake/form` with `_gotcha=bot` → 200, no `submissions`
  row. Clean POST → row lands.
- `parity` still ≥ 0.8.
- No change on pages without these shortcodes (byte-identical output
  on 3 sampled slugs).

## Risk

- `ml-slider` taxonomy term `slug` stores the slider's `wp_id` (not
  `posts.id` — they coincide on this site but JOIN on `wp_id` to
  be safe).
- `[country-phone]` tag syntax in `forms.markup` may carry per-form
  country lists — parse attrs before falling back to the default list.
