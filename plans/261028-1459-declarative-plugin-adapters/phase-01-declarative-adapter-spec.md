---
phase: 1
title: "Declarative adapter spec + SpecAdapter"
status: pending
priority: P1
effort: "1d"
dependencies: []
---

# Phase 1: Declarative adapter spec + SpecAdapter

## Goal

A plugin stack registers into the seam via a TOML spec file instead of a
hand-written `PluginAdapter` impl — same registry, same asserts, zero Rust.

## Spec format

`plugins/specs/<slug>.toml`, embedded at compile time — `build.rs`
scans the dir and generates a `(&'static str slug, &'static str
toml_src)` table (avoids runtime path resolution: the binary runs from
`work/<site>/`, never the repo root). Parsed at `REGISTRY` LazyLock init:

```toml
slug = "babe"
claim_types = ["to_book"]          # wp post_type → kind
option_prefixes = ["babe_"]        # settings keys the stack owns
legacy_option_keys = []            # pre-seam settings keys to namespace

[[kinds]]
kind = "to_book"
section = "bookings"
noun = "booking"
url_base = "booking"
taxonomies = ["ba_location", "ba_type", "categories"]
harvest = true                     # default render = origin HTML
```

Rules:
- Every field required except `option_prefixes`/`legacy_option_keys`/
  `taxonomies`/`harvest` (defaults: empty, `false`).
- Unknown keys → error at load (typo protection); missing file for a
  referenced slug is impossible (all `*.toml` in the dir are loaded).
- Spec kinds/sub-sections pass through the identical duplicate-kind /
  duplicate-section asserts as `ADAPTERS` — a spec colliding with a Rust
  adapter fails startup loudly.

## Files

- Create: `plugins/specs/` dir, `src/admin/plugins/spec.rs`, `build.rs`
  (spec embedding — repo has none today; verify it doesn't collide with
  an existing `build.rs` — `ls build.rs` confirmed absent 2026-10-28)
- Modify: `src/admin/plugins/mod.rs` (registry + `adapters()` merge),
  `Cargo.toml` (`toml` crate — user approved the dep)
- `src/import.rs` — **no change needed**: spec claims ride
  `adapters()` (see step 3 below)

## Implementation

1. `SpecAdapter` holds `Spec` parsed once (`LazyLock`); implements
   `PluginAdapter` by delegating `slug()`, `kinds()` (pre-built
   `Vec<KindDef>` leaked to `&'static [KindDef]` via `Box::leak` —
   `kinds()` signature is `&'static`), `import_hooks()` (claim via
   `claim_types` table lookup + `option_prefixes` returned verbatim;
   `post_imported`/`after_import` = no-op), `legacy_option_keys()`.
   `render()` = `Ok(None)` → caller falls to harvest/bare-body.
2. `mod.rs`: `adapters()` (`src/admin/plugins/mod.rs:443`, currently
   returns `ADAPTERS` verbatim) returns `ADAPTERS ∪ spec-adapters` —
   then `kind_of()` in `src/import.rs:382-386`, admin dispatch, and
   every `adapters()` consumer cover specs with zero call-site changes.
   Registry asserts unchanged; spec↔Rust collisions panic at startup.
   Constraint: `adapters()` returns `&'static` — spec adapters must be
   `Box::leak`ed `SpecAdapter` values in a `LazyLock<Vec>` alongside
   `REGISTRY` build.
3. `admin.rs` submenu/screens: `SpecAdapter.screens()` returns `&[]` —
   list screen for spec kinds comes free via `KindDef.section` (same as
   today's `babe`, which has no screens).

## Edge cases

- `harvest = true` kinds need `options["siteurl"]` present at **import**
  (phase 02 fetches then); absent → bare-body render, never blank.
- `url_base` empty = kind renders at root (`rel_path_with_cfg` already
  handles `None`/`""`).
- Spec parse errors: `REGISTRY` init panics with spec name + toml error —
  specs are repo-authored, fail-loud is correct.

## Verification

- `cargo check` clean.
- New unit test in `spec.rs`: parse a fixture spec → `KindDef` fields
  match; `claim_type` claims `to_book` → `to_book`.
- Boot `rpress serve` on imagedmc: no behavior change yet (specs dir
  empty or fixture only).