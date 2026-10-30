# Phase 02 — Chaty full emit + settings screen

## Overview

- **Priority:** P1 — user-visible floating button is the headline gap.
- **Deliverable:** `chaty` adapter with widget-N emit (fr + es), popup
  keys, and `/admin/plugins/chaty/settings`.

## Requirements

### Emit (render side)

1. `chaty_js` emits the **full per-widget settings map**, mirroring the
   origin's ~35-key shape (verified keys: `cta_type, cta_body, cta_head,
   cta_head_bg_color, cta_head_text_color, show_close_button, position,
   custom_position, bottom_spacing, side_spacing, icon_view,
   default_state, cta_text, cta_text_color, cta_bg_color, show_cta,
   is_pending_mesg_enabled, pending_mesg_count{,_color,_bgcolor},
   widget_icon{,_url}, font_family, widget_size, custom_widget_size,
   is_google_analytics_enabled, close_text, widget_color,
   widget_icon_color, widget_rgb_color, has_custom_css, custom_css,
   widget_token, widget_index, attention_effect`). Source each from the
   `cht_*{sfx}`/`chaty_*{sfx}` setting of the same name with the
   origin-observed defaults — not a hand-picked subset.
2. Widget indices: `""` and `"_1"` suffixes, gated by
   `cht_active{sfx}="1"`. The origin emits only widget 0 on the fr
   page; `_1` is configured but gated — carry **both** widgets' data in
   `chaty_widgets[]` and emit `chaty_custom_traffic_rules{sfx}` into the
   widget so the vendored `script.js` applies the same gating WP does.
   Do NOT pre-filter per lang in the HTML.
3. Channels: widget 0 uses `cht_social_{Name}` (suffixless); widget 1
   uses `cht_social_1_{Name}` — generalize the existing pattern loop to
   `cht_social{_sfx}_{Name}` and emit the **whole unserialized record**
   per channel (`value`, `hover_text`, `svg_icon`, `chatway_position`,
   nested `wp_popup_*`/CTA fields) instead of only `{id, value}` —
   that record is what produces `popup_headline`/`popup_nickname`/
   `popup_profile`/`popup_head_bg_color` in the origin blob. Skip
   catalog keys whose value unserializes to empty (`a:0:{}` or `N;`).
4. Regen: settings changes → `regen_site`.

### Settings screen (`src/admin/plugins/chaty.rs`)

`ScreenDef { path: "settings", title: "Chaty Settings" }`:

- **General** (per widget, tabs `Widget FR / Widget ES`): position,
  icon view, CTA text/bg/text color, bottom/side spacing, default
  state, active toggle → writes `cht_*{sfx}`, `chaty_*{sfx}` keys.
- **Channels** (per widget): table editor — rows of
  `cht_social{_sfx}_{Name}`; each row = channel dropdown (the 28
  `cht_social_*` names in the dump) + value field; serialized round-trip
  via `php_serialize`.
- **Popup/CTA**: the `wp_popup_*`-style fields live **inside each
  channel's serialized blob** — surface headline/nickname/profile/bg
  as named inputs on the channel editor rows, not a separate section.
- **Triggers**: `chaty_trigger_on_*`, `chaty_trigger_time`, scroll/exit
  checkboxes.
- Registers into `ADAPTERS` as a screens-only adapter (`kinds()` =
  `&[]`, `backend()` = `Backend::Rust`). `legacy_option_keys` only
  takes exact keys — for `cht_*`/`chaty_*` breadth, either enumerate
  the concrete keys observed in the dump or leave legacy keys alone
  (the settings screen writes the historical keys anyway).

## Success criteria

- Generated `/` HTML `chaty_widgets[0].settings` carries the full key
  set (≥30 keys) and `channels[0]` carries `svg_icon` +
  `hover_text`/`popup` fields — diff each key against the origin blob.
- `parity` step against `https://test.aucoeurvietnam.com/` sig-overlap
  ≥ 0.8 still holds.
- `/admin/plugins/chaty/settings` GET renders; POST changes
  `cht_cta_1` → regenerate → the new CTA appears in the emitted
  `chaty_widgets` blob (page gating stays client-side).
- Inactive widget (`cht_active_1=0` → toggle off) drops from emitted
  array after regen.

## Risk

- Chaty JS reads `chaty_widgets` by index — identifier order must be
  stable; iterate suffix order `["","_1","_2",…]` deterministically.
- The widget may pick per-lang visibility client-side via traffic
  rules (`chaty_custom_traffic_rules`); emit them verbatim — do not
  pre-filter per lang in HTML or the es widget hides on fr pages.
