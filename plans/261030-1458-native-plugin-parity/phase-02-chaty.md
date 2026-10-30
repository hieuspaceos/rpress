# Phase 02 — Chaty full emit + settings screen

## Overview

- **Priority:** P1 — user-visible floating button is the headline gap.
- **Deliverable:** `chaty` adapter with widget-N emit (fr + es), popup
  keys, and `/admin/plugins/chaty/settings`.

## Requirements

### Emit (render side)

1. `chaty_js` iterates widget indices discovered from
   `chaty_total_settings` **and** the `_1`, `_2`… suffixed keys present
   in `settings` (the dump carries `cht_*_1` — the scan must key off
   presence, not just the counter):

   ```rust
   // suffixes: ["", "_1", …] — for each i in 0..max
   let active = get(&format!("cht_active{sfx}")) == "1";
   ```

   Every active widget pushes into `chaty_widgets[]` with its own
   `cht_*{sfx}`/`chaty_*{sfx}` options; identifier = index.

2. Popup keys — scan settings keys like `popup_%`, `cht_popup_%`,
   `chaty_popup_%`; fold the suffix-matched ones into each widget's
   `settings` object. Verify against the origin's `popup_headline`/
   `popup_nickname`/`popup_profile`/`popup_head_bg_color` emitted blob.

3. Channels per widget: suffixless keys are `cht_social_*` (widget 0);
   widget 1 uses `cht_social_1_*` — generalize the existing
   `cht_social_{n}_{Name}` loop to
   `cht_social{_sfx}_{Name}`.

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
- **Popup**: `popup_*` text/color fields.
- **Triggers**: `chaty_trigger_on_*`, `chaty_trigger_time`, scroll/exit
  checkboxes.
- Registers into `ADAPTERS` as a screens-only adapter (`kinds()` =
  `&[]`, `backend()` = `Backend::Rust`, `legacy_option_keys` =
  `["cht_*","chaty_*"]` prefix claim — note: `legacy_option_keys` takes
  exact keys, so list the concrete ones or teach the seam prefixes —
  prefer prefix-claim, whichever `migrate_all` supports today).

## Success criteria

- Generated `/` HTML contains **2** entries in `chaty_widgets` (fr +
  es) vs 1 before; `popup_*` keys appear when populated.
- `parity` step against `https://test.aucoeurvietnam.com/` sig-overlap
  ≥ 0.8 still holds.
- `/admin/plugins/chaty/settings` GET renders; POST changes
  `cht_cta_1` → regenerate → `/es/` output contains the new CTA.
- Inactive widget (`cht_active_1=0` → toggle off) drops from emitted
  array after regen.

## Risk

- Chaty JS reads `chaty_widgets` by index — identifier order must be
  stable; iterate suffix order `["","_1","_2",…]` deterministically.
- The widget may pick per-lang visibility client-side via traffic
  rules (`chaty_custom_traffic_rules`); emit them verbatim — do not
  pre-filter per lang in HTML or the es widget hides on fr pages.
