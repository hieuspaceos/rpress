---
phase: 6
title: "Tourmaster, widgets & spec metabox"
status: completed
priority: P1
effort: "3h"
dependencies: [1]
---

# Phase 6: Tourmaster, widgets & spec metabox

## Goal

Eliminate the public-path panics, the stored-XSS gaps, the silent
data loss on clone/save, and the compounding year bug in the tour
plugin + the two spec-metabox footguns.

## Files to Create / Modify

- Modify: `src/plugins/tourmaster/save.rs:71-74` — `weekday_of` OOB.
- Modify: `src/plugins/tourmaster/save.rs:120-140` — filter `month`
  to `1..=12` at parse.
- Modify: `src/plugins/tourmaster/save.rs:227-235` — bail when
  `tourmaster-tour-option` parses to non-object JSON.
- Modify: `src/plugins/tourmaster/render.rs:612-623` — `hex_rgba`
  non-ASCII panic.
- Modify: `src/plugins/tourmaster/widgets.rs:160-164, 223, 234` —
  hardcoded `/es` prefix + `&`-unescaped title.
- Modify: `src/plugins/tourmaster/elements.rs:465-470` — combobox
  label `esc()`.
- Modify: `src/plugins/tourmaster/mod.rs:105-112` +
  `src/db/sql/posts__insert_into_tours.sql` — clone missing columns.
- Modify: `src/plugins/goodlayers/boot.rs:770-776, 1114, 1181` —
  `rnd_id()` single draw.
- Modify: `src/plugins/spec.rs:606-622` — metabox `box_present`
  marker.
- Modify: `src/plugins/spec.rs:149-186` — metabox key collision
  validation at SPECS init.

## Requirements

- [x] `month` values filtered to `1..=12` at parse → `weekday_of`
      can't index OOB. `month=13`/`0` → skipped, not panicked.
- [x] `tourmaster-tour-option` bails (4xx/error) when the JSON body
      parses but isn't an object — no `a:0:{}` overwrite + meta wipe.
- [x] **Withdrawn (false positive)**: the agent's `search_months`
      year-ratcheting claim at `render.rs:593-605` was re-traced —
      `y += (mi-1)/12` is at most +1/iteration (i<12), the rollover
      math is correct. No change; keep a small test anyway since the
      function has no coverage.
- [x] `hex_rgba` gates `h.is_ascii()` (or byte-chunks) — a
      multi-byte stored color 500s nothing.
- [x] Tour URL prefix + `Desde`/`De` label use
      `crate::lang::url_prefix`/config — not `lang == "es"` literals.
- [x] Tour card title uses the shared `esc()` (escapes `&` first) —
      `&lt;img onerror&gt;` stays text.
- [x] `<option>` labels in combobox/multi_combobox `esc()`ed.
- [x] `clone_post` copies `rating`, `gallery_json`, `tour_time` —
      the `_24` variant already copied `rating`; bring the live
      statement to parity.
- [x] `rnd_id()` drawn once per element id, reused for the jQuery
      selector — preview init scripts actually run.
- [x] Spec metabox emits a hidden `__present` marker input so a
      checkbox-only box can be unchecked (unchecked → all keys off).
- [x] SPECS init panics/warns on metabox keys colliding with
      `PostForm` field names (`title`, `status`, `slug`, `kind`) or
      reserved prefixes (`trans_*`, `rank_math`).

## Implementation Steps

1. The two panics + the wipe guard first (all in save.rs/render.rs).
2. `rnd_id` single-draw at all 3 sites.
3. clone_post columns + combobox labels + title escaping.
4. spec.rs: `__present` hidden input + init-time key validation.
5. `lang::` literals in widgets.rs → config lookup.

## Verification

- New tests in `tests/audit_remediation.rs`:
  `weekday_of`-guarded month filter; `search_months` rollover sanity;
  spec metabox `__present` uncheck; `hex_rgba` non-ASCII.
- Existing tour_base.rs stays green.
- Manual: builder preview init script id matches its selector.
