# Phase 3 — Custom icons, Ferris mascot, motion

## Overview

- **Priority:** P1
- **Status:** Complete
- Three additive pieces: a hand-drawn icon set replacing Dashicons, the
  Ferris crab mascot (`atrium.js`), and the global motion pass. Each is
  independently revertible.

## Key Insights

- Dashicons render as `::before` font glyphs on `.dashicons-*`; swapping is
  `content:none` + `background`/`mask` data-URI override — markup untouched.
  Unmapped classes keep the original glyph (dashicons.woff stays loaded as
  fallback — never an empty box).
- `atrium.js` is the mock's engine; it **injects `#ferris` itself** (no
  markup dependency), is loaded once via `chrome_end` (`src/admin/mod.rs:1360`),
  and must be singleton-guarded because htmx fragment swaps re-execute page
  scripts (`window.__ferrisInit` guard).
- There is no `<script>` insertion point in an "admin footer" — `html_admin`
  ends with `{body}{end}`; `chrome_end` is the injection site.
- `atrium.js` runs only on logged-in screens — not on `html_login`
  (login gets a static Ferris mark in CSS, no engine).
- Motion budget: ≤160ms transforms/opacity only; `prefers-reduced-motion`
  kills all of it including Ferris; content never depends on animation.
- Security: `atrium.js` is same-origin full-privilege admin JS — it must
  read no secrets (CSRF token at mod.rs:1347, ajaxurl :962 are DOM-visible
  anyway but untouched), make no network calls, and be first-party code we
  write (no vendored minified blob). Icons are build-time data-URIs —
  never runtime-interpolated strings into `url()`/`content:`.

## Requirements

### Icons (`atrium.css` icon section)

1. Hand-drawn set (~30 glyphs per Phase-1 `reports/icons.md`), rounded
   ~1.6px stroke on 20px viewBox; each nav section tinted via `--icon-tint`.
2. Map every inventoried class: menu icons + in-screen `dashicons-*`
   (arrows, close, plus, minus, visibility). Unmapped = Dashicon fallback.
3. `.dashicons-*::before { content:none }` only for mapped classes; inline
   SVG data-URIs as `mask`/`background` — no per-icon request.
4. Icons must not resize their 20px containers.

### Ferris (`atrium.js` + CSS)

5. Engine: entrance scuttle → edge patrol with facing flips → leg/step idle
   → eye blink → cursor hunt (parked ≥0.9s within 500px → rev 480ms →
   charge with streaks + puffs) → dodge (<80px → bolt) → speech lines on
   click (≥10 press-floor lines) → `.tag` on hover.
6. Singleton guard `window.__ferrisInit`; clamps to viewport; crab's
   `pointer-events` confined to its 64px box; particles `pointer-events:none`.
7. Hidden under `prefers-reduced-motion` (static or absent); node injected
   by the script so `noscript` renders nothing.

### Motion pass

8. Menu tile hover lift, running-dot, badge pop; notice drop-in 160ms;
   checkbox rotate-on-check 100ms; button press-travel; tooltip rise;
   filter pill slide. Nothing else — no reveal-on-load, no loops besides
   Ferris idle.

## Implementation Steps

1. Draw glyphs in `assets/icons.svg`, export per-icon data-URIs into the
   `atrium.css` icon section keyed by class.
2. `templates/admin/wp/atrium.js` — self-invoking engine + chatter +
   particles; injects `#ferris`.
3. `src/admin/mod.rs` `chrome_end` (~:1360) —
   `<script src="/admin/wp/atrium.js" defer></script>`.
4. Screenshot icon regions; click-through hunt/dodge at two viewport sizes;
   test folded/responsive nav + htmx fragment swaps (post save, list
   refresh) for Ferris duplication.

## Todo List

- [x] Icon set drawn and mapped for every inventoried class
- [x] `atrium.js`: enter → patrol → hunt → charge → dodge → chatter
- [x] Singleton guard vs htmx fragment re-execution
- [x] Speech lines + hover tag
- [x] Motion pass applied, all ≤160ms
- [x] Reduced-motion and no-JS verified; no JS on login page

## Success Criteria

- Zero empty icon boxes; menu icons are the custom set.
- Ferris alive and interactive; exactly one instance after htmx swaps;
  screen fully usable with JS off.

## Risk Assessment

- An injected node over content could steal clicks → confined pointer-events,
  edge-biased positions, small footprint.
- htmx swaps re-running init → `__ferrisInit` guard, verified by post-save
  and list-refresh actions.
- SVG data-URI size: ~30 small glyphs, target <8KB total.
