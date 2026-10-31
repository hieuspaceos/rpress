---
title: "Phase 6: Plugin Spec Extension — Metaboxes, Screens, Render Hooks"
status: done
priority: P1
---

# Phase 6: Plugin Spec Extension

## Overview

TOML spec currently covers ~30% of the `PluginAdapter` contract (audit E3):
no `editor` hooks, no `ScreenDef` screens, no `render()`/`submenu()`/`seeds()`.
Goal: a "dumb CPT with metaboxes + a settings screen" onboards without Rust.
Genuinely dynamic behavior still goes through the Rust adapter escape hatch.

## File Ownership

- `src/plugins/spec.rs` — spec parsing + generated `PluginAdapter` impl
- `plugins/specs/*.toml` — schema extension + babe.toml as proving case
- `build.rs` — spec embedding (same mechanism)
- `src/admin/posts.rs` — only if the generated editor needs a metabox render hook into the shared edit screen (kind `editor: None` today — check how Rust adapters' `editor` hook is invoked; extend spec to produce a declarative metabox descriptor consumed by the same hook)
- `src/plugins/mod.rs` — `KindDef`/`ScreenDef` fields if the generated adapter needs them (minimal touch)
- `docs/` — spec schema reference

## Design

Extend the TOML schema (spec.rs:29-60) with:

```toml
slug = "events"
claim_types = ["tribe_events"]

[[kinds]]
kind = "tribe_events"
section = "events"
noun = "Event"
url_base = "event"
taxonomies = ["tribe_events_cat"]

# NEW: declarative metabox fields → generated editor + save
[[kinds.metaboxes]]
id = "event-details"
title = "Event Details"

[[kinds.metaboxes.fields]]
meta_key = "_EventStartDate"
label = "Start"
type = "datetime"          # text|textarea|datetime|number|checkbox|select|media
# options = [...]          # for select
# media_kind = "image"     # for media

# NEW: declarative settings screen → generated ScreenDef
[[screens]]
path = "settings"
title = "Events Settings"

[[screens.fields]]
option_key = "events_per_page"
label = "Per page"
type = "number"
```

Generated adapter behavior:
- `kinds()` → as today, plus `editor: Some(generated_metabox_editor)` when
  `metaboxes` present — render inputs from descriptors; save = `post_meta`
  upserts (same CSRF/auth envelope as Rust editors; reuse the existing
  save-path helpers — check how tourmaster's editor persists meta).
- `screens()` → `ScreenDef` entries; handler renders a settings form from
  `screens.fields` descriptors and saves `settings`/`option` rows under the
  adapter's `plugin:<slug>:` namespace.
- `render()`/`submenu()`/`seeds()` stay Rust-only (documented) — declarative
  render is the theme/harvest lane's job.
- Collision/typo rules unchanged: `deny_unknown_fields`, panic on bad spec,
  Rust adapter wins on duplicate claims.

## Requirements

- [ ] Spec schema: `metaboxes` + `screens` sections parsed, `deny_unknown_fields`, fail-loud.
- [ ] Generated metabox editor: renders inside the shared post edit screen for the claimed kind; saves meta on post save; supports field types listed; escapes everything (`esc`/`esc_attr`); CSRF covered by the enclosing form.
- [ ] Generated settings screen: `/admin/plugin-<slug>/<path>` route via existing `screen_for` dispatch; form + save through `plugin:<slug>:` namespaced options; admin-only.
- [ ] `babe.toml` (or a new `events` example) proves the path end-to-end: metabox appears on its CPT edit screen, value round-trips; settings screen saves + reloads.
- [ ] Raise or remove the 8-spec cap (spec.rs:232-259 `claim_at` slot table): move to a `Vec`/`HashMap` built at `REGISTRY` init — no fixed slot table.
- [ ] Docs: `docs/` gains a spec schema reference (all keys, field types, escape-hatch rules); README "Declarative plugin adapters" section updated.
- [ ] Per-site plugin selection stays out of scope (mod.rs invariant) — document it.

## Implementation Steps

1. Read how Rust adapters' `editor` hook is wired into `admin/posts.rs` edit screen + save (KindDef.editor ~mod.rs:42). Design `MetaboxField`/`ScreenField` descriptor structs matching that hook's signature.
2. Extend spec parser (serde structs) + codegen: produce a `SpecAdapter` holding `&'static` descriptors; `kinds()` returns KindDefs with `editor`/`screens` filled.
3. Implement `spec_editor_html(ctx, fields)` + `spec_screen_html/save` in spec.rs — reuse `html_admin`/`esc`/`check_csrf` envelopes.
4. Replace `claim_at` fixed slots with registry Vec.
5. Write the proving spec; smoke it on a seeded DB.
6. Test: spec with metabox → edit screen contains field → POST saves meta → reload shows value; settings screen saves namespaced option; unknown field in TOML → panic with slug+key.

## Success Criteria

- A new CPT with metaboxes + settings screen onboards via one TOML file — verified by the proving spec.
- `cargo test` green; `check-layers.sh` passes.
- Spec docs list exactly what's TOML-able vs Rust-only.

## Risk Assessment

- The shared post save path must dispatch generated-editor saves — check how `save_post` routes to `KindDef.editor` today; the generated save must reuse the same transaction/envelope (bulk edit, quick edit must not clobber metabox meta — verify quick_edit touches only its own fields, it does per audit).
- Field types are a finite enum — resist adding `repeater`/`wysiwyg` now (YAGNI); Rust hatch covers them.
- Spec count growth: embedding is compile-time (build.rs) — acceptable; runtime-loaded specs are explicitly out of scope (would break the no-FS-dependency invariant).
