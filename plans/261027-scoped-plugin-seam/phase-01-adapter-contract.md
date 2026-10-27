---
phase: 1
title: "Adapter contract + kind registry + settings namespaces"
status: pending
priority: P1
effort: "2d"
dependencies: ["audit phases 1-7"]
---

# Phase 1: `PluginAdapter` trait, kind registry, settings namespaces

## Goal

One contract for all plugins. `PluginAdapter` + `Backend` enum + `KindDef`
replace `EditorPlugin`-only thinking; a `kind_registry` module owns the
`posts.kind → (plugin, KindDef)` map so core never names a plugin kind.
`settings` keys gain a `plugin:` namespace so plugin options can't collide
with core keys. No behavior change yet — this phase is types + registry
plumbing only; tourmaster is migrated in phase 02.

## Findings covered

- **T1** — `EditorPlugin` (`src/admin/plugins/mod.rs:18-31`) handles
  metabox/load/save but has no screens, import hooks, render, or backend
  label. Extend into `PluginAdapter`.
- **T2** — `PLUGINS` static + `find(kind)` (`src/admin/plugins/mod.rs:33-37`)
  is the registration point; it becomes the adapter registry and gains a
  `kind → adapter` index.
- **T3** — `KINDS`/`KindSpec` (`src/admin/posts.rs:7-47`) hardcodes
  `post`/`page`/`tour` admin sections; `spec_for_kind`'s
  `.unwrap_or(&KINDS[0])` (`src/admin/posts.rs:39-41`) silently
  post-ifies unknown kinds (red-team R1). Kind lookup moves to the
  registry; unknown kinds become errors, not posts.
- **T4** — `settings` table is flat key/value (`src/schema.rs`; option
  import at `src/import.rs:128-156` writes raw keys). Plugin options
  (`tourmaster_general`, `gdlr-core-*` at `src/admin/gdlr_pb.rs:322-326`)
  need a namespace so the write whitelist has a mechanical boundary.

## Key insights

- `EditorPlugin` already encodes the four hooks a kind editor needs;
  `KindDef { kind, section, noun, labels, head, url_base_key?, editor: Option<EditorHooks> }`
  preserves them verbatim — migration is a rename, not a rewrite.
- `db::posts` calls `plugins::find` at exactly two sites
  (`src/db/posts.rs:165` load, `src/db/posts.rs:1106` save). Swapping the
  lookup implementation to registry-indexed keeps both call sites stable.
- `import.rs` `kind_of` (`src/import.rs:294-302`) is a closed match; the
  registry makes it open-ended without changing the surrounding loop.
- Settings namespace: today `put_setting(conn, "gdlr-core-skins", v)`
  writes bare keys (`src/admin/gdlr_pb.rs:325`). Introduce
  `setting_key(slug, key) -> "plugin:{slug}:{key}"` helper NOW so phase 07
  can mechanically whitelist `plugin:goodlayers:*` without touching core
  keys. Migration of existing keys deferred to phase 05 (reads accept
  both during transition).
- `tax_menu` (`src/admin/tax.rs:472-477`) hardcodes "everything else →
  tours". Registry gains `taxonomies() -> &[&str]` on `KindDef` so tax
  screens route by owner kind, not a catch-all.

## Files to modify

- `src/admin/plugins/mod.rs` — define `PluginAdapter` trait, `Backend`
  enum, `KindDef`, `ScreenDef`, `ImportHooks`, `EditorHooks` (existing
  load/metabox/assets/save fns), `Registry` struct with
  `by_kind: HashMap<&'static str, &'static PluginAdapter>` +
  `by_slug`; `ADAPTERS` static; `adapter_for_kind`, `adapter_for_slug`,
  `kind_def`, `spec_for_kind`/`spec_for_section` replacements.
  Keep `EditorPlugin`/`find` as thin shims delegating to the registry
  until phase 02 deletes them.
- `src/admin/posts.rs` — `KINDS` → registry-driven `spec_for_kind` /
  `spec_for_section`; `spec_*` return `Option`; every caller updated
  (sites: `src/admin/posts.rs:666,719,814,974-981,1243`;
  `src/admin/revisions.rs:35,46,96`); unknown kind → 404 list page.
  Core `post`/`page` KindSpecs live in registry as built-ins.
- `src/admin/tax.rs` — `tax_menu` consults `kind_def(tax.owner)` instead
  of the catch-all; tour taxonomies list moves into tourmaster `KindDef`
  in phase 02, so phase 01 keeps the literal default but reads it from
  the registry.
- `src/db/misc.rs` or new `src/db/settings.rs` — `setting_key(slug,key)`,
  `namespaced_get`/`namespaced_put` helpers.
- `src/lib.rs` / `src/db/mod.rs` — module decls.

## Implementation steps

1. Write `PluginAdapter`, `Backend`, `KindDef`, `ScreenDef`,
   `ImportHooks`, `EditorHooks` in `src/admin/plugins/mod.rs`; move
   `EditorPlugin` fields into `EditorHooks` unchanged.
2. Build `Registry` over a `ADAPTERS: &[&'static dyn PluginAdapter]` —
   keep it `&'static`/`phf`-style statics (no `Box`/startup mutation)
   so registration stays compile-time, matching `PLUGINS` precedent.
   `lazy`/`OnceLock` index maps are fine.
3. Register core `post`/`page` as built-in `KindDef`s inside the
   registry (no adapter) — `kind_def("post")` must work with zero
   plugins so core sites need no registration.
4. Change `spec_for_kind`/`spec_for_section` to `Option`; update all
   listed callers; add a test asserting `spec_for_kind("bogus").is_none()`
   and edit page 404s for unregistered kind (red-team R1).
5. Add `setting_key`/`namespaced_*` helpers; convert `gdlr_pb`'s
   `get_setting`/`put_setting` (`src/admin/gdlr_pb.rs:322-326`) and
   `tourmaster_general` read (`src/render.rs:1244-1249`) to read
   `plugin:<slug>:` first, fall back to legacy bare key.
6. `tax_menu`: look up `adapter.taxonomies()`; keep literal fallback
   for `category`/`post_tag` → posts.

## Verification

```bash
cargo check && cargo test
cargo test kind_registry
grep -rn 'unwrap_or(&KINDS' src/   # expect: gone
grep -rn 'plugins::find' src/      # expect: only inside registry impl
```

Manual: `/admin/posts`, `/admin/pages`, `/admin/tours` lists render;
`/admin/edit/<tour id>` metabox unchanged; `/admin/new/bogus` → 404.

## Todos

- [ ] `PluginAdapter`/`Backend`/`KindDef`/`ScreenDef`/`ImportHooks` defined
- [ ] Registry indexed by kind + slug; core kinds built-in
- [ ] `spec_for_kind`/`spec_for_section` → `Option`, callers updated
- [ ] `setting_key` namespace helpers; gdlr + tourmaster_general dual-read
- [ ] `tax_menu` registry-driven
- [ ] Tests: unknown kind 404, dual-read settings, registry lookup

## Risks

- `Option`-returning `spec_for_*` touches ~10 call sites — mechanical,
  compile-checked.
- Settings dual-read adds one query miss per legacy key; bounded by
  small option count; dropped in phase 05 after key migration.
- Registry is process-global static — plugin set can't vary per site at
  runtime yet. Accepted: per-site opt-in is a `Backend::PhpBridge`
  concern (phase 06), not a registry concern; document the invariant.
