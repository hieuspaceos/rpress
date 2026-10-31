---
phase: 2
title: "`ignore_types` spec field + storage-CPT decluttering"
status: pending
priority: P2
effort: "1.5h"
dependencies: []
---

# Phase 2: `ignore_types` spec field + storage-CPT decluttering

## Goal

Plugin-internal storage CPTs (dashboards, popup themes, form
submissions, video blocks) stop appearing as `UNCLAIMED` suggestions —
they're types import deliberately skips, not spec work to do.

## Findings driving this phase

Every site lists noise suggestions:
`itsec-dash-card`/`itsec-dashboard` (iThemes dashboard cards),
`popup`/`popup_theme`/`aoc_popup` (Popup Maker internals),
`pp_video_block` (Presto Player block records),
`flamingo_contact`/`flamingo_inbound` (Flamingo mail archive —
arguably wanted data, but definitely not a "generic page"),
`mc4wp-form`, `wpcode`, `language_switcher`, `polylang_mo`,
`rank_math_schema`, `rm_content_editor`. Suggesting
`plugins/specs/X.toml: claim_types += [...]` for these is wrong —
import must *skip* them, not claim them.

## Files to Create / Modify

- Modify: `src/plugins/spec.rs` — accept `ignore_types = [...]` in
  spec TOMLs (sibling of `claim_types`); expose via `ImportHooks` or a
  parallel `ignore_types` accessor.
- Modify: `src/plugins/mod.rs` — registry aggregation:
  `ignored_type(name)` checking all adapters (same probe style as
  `claim_type`).
- Modify: `src/detect.rs` — `claimed()`/`run()` third bucket:
  `{t} ({n}) → ignored by spec (storage CPT — skipped at import)`.
  Excluded from the unclaimed suggestion list.
- Modify: whichever adapter spec owns each noise type — add
  `ignore_types`:
  - a shared `plugins/specs/wp-internals.toml` is cleaner than one per
    plugin for cross-cutting storage (itsec_*, wpcode, polylang_mo,
    language_switcher…) — decide during implementation; adapter-owned
    ones (flamingo → flamingo spec if added later) stay local.

## Requirements

- [ ] `ignore_types` parses from TOML and resolves at runtime
- [ ] dmc/imagedmc/imagetravel suggestions drop itsec_*, popup*,
  pp_video_block, flamingo_*, mc4wp-form, wpcode, polylang_mo,
  language_switcher, rank_math_schema, rm_content_editor
- [ ] A type in BOTH `claim_types` and `ignore_types` — claim wins
  (document choice; panic on ambiguity is acceptable, same style as
  adapter collisions elsewhere)

## Verification

- `cargo test` green incl. a spec-parse test for `ignore_types`
- detect re-run: suggestion list on dmc shrinks to real work only
