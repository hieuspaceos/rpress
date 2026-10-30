---
title: "Native plugin parity + per-plugin settings"
description: "Close the remaining native gaps (Chaty widget #2 + popup, metaslider shortcode, CF7 extensions) and give every plugin — native or bridged — a WP-style settings screen under /admin/plugins."
status: pending
priority: P1
effort: 18h
branch: main
tags: [feature, admin, plugins]
blockedBy: []
blocks: []
created: 2026-10-30
---

# Native plugin parity + per-plugin settings

## Overview

Goal: every plugin a migrated site needs runs **natively in rpress**, and
each one — native or still on the PHP bridge — owns a settings screen
under `/admin/plugins/<slug>/settings`, the same mental model WP gives.

The only intentional PHP-bridge remnant after this plan is the **GoodLayers
page-builder editor** (explicit scope boundary: page-builder stays
bridged). Everything else — Chaty, wpDiscuz, CF7 and its extensions,
metaslider, site-reviews, head-footer-code — becomes native, with a
visible settings surface.

## Current state (evidence)

- `chaty_js`/`chaty_css` (`src/render/site_data.rs:1111`) emits **only
  widget 0** (`chaty_total_settings`-indexed channel scan stops at
  suffixless `cht_*` keys). `cht_active_1=1` + `cht_cta_1="Habla con
  nosotros"` exists in the dump — an ES widget that never renders.
  Origin HTML carries `popup_*` keys the emitter ignores.
- `[metaslider id="6426"]` appears in 4 posts; `expand_shortcodes`
  (`src/render/shortcode.rs`) passes it through verbatim → dead text on
  the generated page. `ml-slider` posts are already imported (kind
  `ml-slider` + `ml-slide`, 29 slides).
- wpDiscuz front-end is native (render + `/intake/comment`) but none of
  its `wpdiscuz_*` options are imported — no settings surface exists.
- No backup/export feature exists at all (`site_init` only *reads*
  backups on import).
- CF7 `country-phone-field` renders as a plain text input; `simple-
  recaptcha` does nothing on intake.
- `/admin/plugins/<slug>` already dispatches `ScreenDef`s per adapter —
  the settings screens plug into that contract, no new router needed.
- The adapter registry (`src/admin/plugins/mod.rs`) already has
  `Backend::PhpBridge` and per-adapter `screens()` — bridged plugins can
  declare settings screens the same way, or link out to their existing
  bridged screens.

## Design

### `/admin/plugins` — plugins index (new)

WP-parity Plugins list. Each row: name, description line, **Backend
badge** (`Native` / `Bridge`), and links:

- native → `Settings` → `/admin/plugins/<slug>/settings`
- bridge → `Open` → its existing `/admin/plugins/<slug>/<path>` screen;
  plus `Bridge status` row (workdir, last pull, PHP binary) read from the
  same state `rpress bridge` inspects.

Registered adapters supply `PluginDescriptor { name, blurb }`; core owns
the table. Adapters not yet registered (metaslider, wpdiscuz options,
chaty) get registered as `screens`-only adapters — `kinds()` empty,
`backend()` `Rust` (or `Data` where they only read settings).

### Settings screens contract

A settings screen is a `ScreenDef` at path `settings`, GET+POST:

- Form fields map **1:1 onto the option keys the WP plugin used**
  (`cht_*`, `chaty_*`, `wpdiscuz_*`, `ml-slider_*`, `auhfc_*`). No schema
  migration — import already writes them to `settings`, runtime readers
  keep working, rollback is free.
- Save = `INSERT OR REPLACE INTO settings` inside one transaction +
  `regen` of affected public pages (channel/options changes are
  site-wide → full `generate`, reusing `admin::regen_*` plumbing).
- CSRF via existing `check_csrf`; auth via `need_auth` in `handle`.
- Complex values (serialized channel lists, traffic rules) get
  structured editors that round-trip through `php_serialize::{serialize,
  unserialize}` — never a raw blob textarea for things users edit daily;
  a raw "Advanced (serialized)" textarea is acceptable for cold keys.

### Native feature work folded into the same adapters

- **Chaty widget #2 + popup keys** — `chaty_js` iterates
  `0..chaty_total_settings`-derived indices (`""` and `"_1"` suffixes),
  reads `cht_active{_i}` gate, emits every active widget into
  `chaty_widgets[]`; popup keys join `settings` of their widget.
- **Metaslider shortcode** — `[metaslider id]` expands to a static
  slider (the `ml-slider` post's ordered `ml-slide` children →
  `<div class="metaslider">` + `<img>` list + the plugin's own vendored
  JS if the dump ships it; else a tiny inline fade). No bridge.
- **CF7 extensions** — `[country-phone]` field in form markup renders a
  `<select>` of country codes + phone input; `simple-recaptcha` becomes
  a honeypot hidden field that `/intake/form` silently drops (bot
  posts the field → 200 no-op).
- **Backup** — `rpress site backup <site>` CLI produces
  `work/<site>/backups/<ts>.tar.gz` (`rpress.db` + `www/`); admin
  `/admin/plugins/core/backup` (or `/admin/tools/backup` — decided in
  phase) streams the archive for download. No WP equivalent needed —
  this replaces Updraft.
- **head-footer-code** — `auhfc_settings_*` keys are already imported;
  render injects them at `wp_head`/`wp_footer` positions (verify they
  already land — gap check in phase 3).

## Phases

| Phase | Name | Status |
|-------|------|--------|
| 1 | [Plugins index + settings framework](./phase-01-plugins-index-settings-framework.md) | Pending |
| 2 | [Chaty full emit + settings screen](./phase-02-chaty.md) | Pending |
| 3 | [Frontend gaps: metaslider, CF7 ext, head-footer](./phase-03-frontend-gaps.md) | Pending |
| 4 | [wpDiscuz + remaining native settings screens](./phase-04-wpdiscuz-settings.md) | Pending |
| 5 | [Backup/export + bridge plugin settings surface](./phase-05-backup-bridge-settings.md) | Pending |

## Non-goals

- GoodLayers **page-builder editor** stays on the PHP bridge — porting
  it is a separate plan.
- Rewriting plugin option schemas to namespaced `plugin:<slug>:*` —
  `legacy_option_keys` mechanism stays as-is; settings screens write the
  historical keys.
- wpDiscuz internal tables (`wpdiscuz_form*`, votes) — imported only
  where the front-end needs them; the settings screen covers
  `wpdiscuz_options`-style keys actually present in the dump.

## Verification

Every phase ends with the e2e gates that already exist:

```sh
sh scripts/e2e-site.sh sites/aucoeur-test.env import generate check serve
```

plus screen/browser spot checks for new admin pages, and a byte-level
diff of a rendered page vs origin for each emitted feature (parity =
the `sig()` overlap > 0.8 the script already uses).
