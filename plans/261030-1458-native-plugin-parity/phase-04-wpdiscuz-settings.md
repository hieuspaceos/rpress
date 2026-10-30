# Phase 04 — wpDiscuz + remaining native settings screens

## Overview

- **Priority:** P2 — front-end is already native; this gives each
  native plugin its WP-style settings home.
- **Deliverable:** `wpdiscuz`, `cf7`, `metaslider`, `site-reviews`,
  `head-footer-code` adapters with `/admin/plugins/<slug>/settings`.

## Requirements

### wpDiscuz

1. Extend the options whitelist in `import.rs` with `wpdiscuz_` prefix
   (the dump carries `wpdiscuz_form*`, `wpdiscuz_*` keys — currently
   dropped).
2. `wpdiscuz` adapter (screens-only): settings form over the flat keys
   users actually touch — `wpdiscuz_form_general_options` (structure of
   form fields is a serialized blob; first pass = SerializedTextarea
   with round-trip), plus top-level toggles surfaced from the blob
   (comment ordering, default form) as first-class fields where the
   structure is stable.
3. Comment settings that already live in core (`comment_moderation`,
   `thread_comments`, `require_name_email`…) stay on
   `/admin/settings/discussion` — the wpdiscuz screen *links* there,
   doesn't duplicate.

### CF7

`/admin/plugins/cf7/settings`: `cf7_honeypot_enabled`, default
country-phone country list, and a submissions-retention field
(`cf7_keep_days`, 0=forever) if the inbox already prunes — check
`contact.rs` for an existing knob first; don't add a setting that has
no consumer.

### Metaslider

`/admin/plugins/metaslider/settings`: `ml-slider_settings` defaults +
per-slider edit link into the `ml-slider` post editor (they're posts —
settings link out to `/admin/edit/<id>` rather than re-implementing).

### Site Reviews

`/admin/plugins/site-reviews/settings`: form config keys present in the
dump (`site_reviews_settings`-style keys — verify names from
`settings`/`active_plugins` first; if none exist the screen shows
review-moderation links only and stays honest about it).

### head-footer-code

`/admin/plugins/head-footer-code/settings`: three textareas
(`auhfc_settings_sitewide` subkeys head/footer, homepage, article) —
serialized round-trip; fields for the sitewide bucket only on first
pass.

## Success criteria

- Each screen GETs 200 with fields populated from existing settings;
  POST persists and survives a second GET.
- `import verify` still OK after whitelist widening; a re-import lands
  `wpdiscuz_*` rows in `settings`.
- The plugins index shows all five with `Native` badge + working
  `Settings` link.
- No consumer-less settings: every form field maps to a key the
  runtime reads (assert by grepping the key name in `src/`).

## Risk

- Serialized blob editors corrupting structure — always round-trip
  through `php_serialize::{unserialize,serialize}`; reject a save whose
  re-serialization fails rather than storing garbage.
- Option-prefix claims: `wpdiscuz_` prefix in `import.rs` is the only
  import change; everything else reads what's already in `settings`.
