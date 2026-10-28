---
title: "Declarative plugin adapters — spec + harvest-first rendering"
description: "Stop growing core Rust per WP stack. New plugin stacks on-board as a small TOML spec (claim types, kinds, option prefixes) and harvested origin HTML; Rust adapters remain only for plugins needing admin/save semantics. Runs on top of 261027-scoped-plugin-seam."
status: completed
priority: P1
effort: "2-4d"
branch: main
tags: [architecture, plugins, adapters, harvest, dx]
created: 2026-10-28
---

# Declarative plugin adapters — spec + harvest-first

## Premise (problem observed 2026-10-28)

The `261027-scoped-plugin-seam` architecture is correct — core never names
plugins, all dispatch flows through `PluginAdapter` — but on-boarding a new
WP stack still costs handwritten Rust + templates per stack:

- `babe.rs` (~80 LOC) + `babe/render.rs` (~220 LOC) + `single_book.html` +
  `babe_enquiry.html` for the tevily/BABE stack — and it was still wrong
  (route 200, layout missing) until the markup was rebuilt by hand.
- The same tevily+BABE stack serves `dmc`, `imagedmc`, `imagetravel` —
  one adapter, three sites; but a *fourth* stack starts the cost over.

Every stack repeats the same checklist: CPT claim → KindDef → taxonomy →
single render → option prefix. Only the *shape* changes, not the logic.

## Insight

The codebase already owns every piece needed to make on-boarding data-driven:

| Existing piece | Location | New role |
|---|---|---|
| `PluginAdapter` trait + `ADAPTERS` registry | `src/admin/plugins/mod.rs:202,379` | Registry gains generic `SpecAdapter`s built from TOML files |
| `harvest_posts` table + `legacy_origin` fetch | `src/import.rs:490-558`, `src/generate.rs:61-116` | Default render path for unrendered plugin kinds — pixel-perfect by construction |
| `claim_type` dispatch | `src/import.rs:379-399` | Spec claims CPTs without a `claim()` fn |
| `KindDef` (kind/section/noun/url_base) | `src/admin/plugins/mod.rs` | Spec carries identical fields |

## Design

```
plugins/
  specs/
    babe.toml          # slug, kinds, claim_types, option_prefixes, harvest = true
    wp-travel-engine.toml  (future stack — spec only, zero Rust)
```

1. **Spec format** (`plugins/specs/*.toml`): `slug`, per-kind
   `{kind, section, noun, url_base, taxonomies}`, `option_prefixes`,
   `claim_types`, `harvest`. Loaded once at `LazyLock` time next to
   `ADAPTERS`; parse failure = loud startup error, never silent skip.
2. **`SpecAdapter`** (`src/admin/plugins/spec.rs`): a concrete
   `PluginAdapter` impl wrapping one parsed spec — `kinds()` returns
   `KindDef`s built from spec; `import_hooks()` claims `claim_types`;
   `render()` returns `Ok(None)`. Registry merges `ADAPTERS ∪ specs`
   under the same duplicate-kind/section asserts.
3. **Harvest at import** (phase 02): import fetches origin-rendered HTML
   once into `harvested_pages(post_id, html, fetched_at)` for spec kinds
   with `harvest = true`; generate emits stored rows verbatim. Origin
   only needs to be live during `import` — port-once semantics,
   dead-origin regenerates keep working.
4. **Native render stays the escape hatch**: `render()` returning
   `Some(html)` wins over harvest (kind spec `harvest = false`); Rust
   adapters keep priority — specs only fill gaps. tourmaster/submissions
   stay Rust (admin save semantics, not renderable markup).

## Why not X

- *Keep writing Rust per stack*: proven today to ship broken-then-fixed
  scaffolds; cost scales with stacks.
- *Meta-spec driven generic renderer* (template + field map): real option
  when admin-editing must reflect on the page or a stack has no
  harvestable origin even at import — deferred; harvest-at-import covers
  the port-once mirror case for free.
- *PHP bridge for everything*: opt-in infra still planned
  (`261027-scoped-plugin-seam` phases 06-07) for genuinely dynamic output;
  overkill for static mirrors.

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | New plugin stack on-boards as a TOML spec, zero Rust | P1 |
| 2 | Unrendered spec kinds produce correct pages from DB-stored harvest (origin needed only at import) | P1 |
| 3 | Existing adapters untouched in behavior; spec/Rust coexist | P2 |

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [Declarative adapter spec](./phase-01-declarative-adapter-spec.md) | Done |
| 2 | [Harvest-at-import for unrendered kinds](./phase-02-harvest-fallback-for-unrendered-kinds.md) | Done |
| 3 | [Migrate babe adapter to spec](./phase-03-migrate-babe-adapter-to-spec.md) | Done |

## Dependencies

- Phase 02 depends on 01 (spec drives `harvest` flag).
- Phase 03 depends on 01+02 (proves parity by deleting Rust).
- Requires `261027-scoped-plugin-seam` phases 01-03 (adapter contract,
  claim dispatch, kind registry) — landed.

## Success Criteria

- [x] `plugins/specs/babe.toml` alone produces identical dmc/imagedmc
      output — better than the scaffold: booking pages are origin-
      harvested HTML (harvest replaces render.rs markup by construction).
- [x] Deleted `src/admin/plugins/babe*` + `templates/single_book.html`
      + `templates/elements/babe_enquiry.html`; `cargo check` clean,
      imagedmc+dmc regenerate (233/393 pages, /booking/* 200).
- [x] Third stack needs only a new `.toml` — documented in README
      ("Declarative plugin adapters"). Note: `claim_at` slot table caps
      spec count at 8 (loud panic to extend).
- [x] `cargo check` + `cargo test` clean; aucoeur check passes (no spec
      kinds, no harvested_pages, elementor path unchanged).

<!-- slug: declarative-plugin-adapters -->