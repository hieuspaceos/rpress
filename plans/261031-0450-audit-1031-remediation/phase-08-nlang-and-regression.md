---
phase: 8
title: "N-language + regression gates"
status: completed
priority: P2
effort: "3h"
dependencies: [6]
---

# Phase 8: N-language + regression gates

## Goal

Remove the two-language assumptions the architecture doc claims are
gone, and land permanent tests for the classes of bug this audit
found so they can't regress quietly.

## Files to Create / Modify

- Modify: `src/plugins/mod.rs:665-692` — `theme_cat_lang` hardcodes
  `es_roots`/`es_roots_lang`/`ends_with("-es")` → generalize to
  per-lang root sets (audit re-check: `tax_lang`/`tax_pairs` in
  `ThemeDef` are already generic `&'static [(&str,&str)]` lists —
  only the taxonomy-map readers and the `es_roots` settings blob are
  2-lang. Smaller blast radius than first estimated).
- Modify: `src/plugins/mod.rs:705-716` — `localized_taxonomy` already
  iterates `tax_pairs`; extend so twin resolution consults `tax_lang`
  for **any** registered secondary lang.
- Modify: `src/plugins/tourmaster/widgets.rs:160-164` — depends on
  phase 6's `lang::url_prefix` change; verify the 3-lang case.
- Modify: `docs/architecture.md` — update the "Secondary-language
  taxonomies use suffixed slugs" paragraph once N-lang resolution
  lands.
- Modify: `tests/audit_remediation.rs` + `tests/tour_base.rs` —
  add 3-lang fixtures (e.g. `fr,es,de`).

## Requirements

- [x] A 3-language site (`settings.languages = "fr,es,de"` or
      equivalent) resolves category-archive lang correctly for **all**
      secondary langs — no silent fallback to primary on the 3rd tree.
- [x] `theme_cat_lang` returns the matching lang for any
      `-{lang}`-suffixed slug registered in the theme's taxonomy map.
- [x] TOML theme spec: `tax_pairs`/`tax_lang` are already generic
      lists — extend the *settings blob* (`es_roots` → `lang_roots`)
      instead; spec schema only needs a change if the blob shape does.
- [x] New regression tests cover: session-sweep type check,
      prefixed-dump table coverage, `month` guard, tour-option object
      guard, `search_months` rollover sanity (regression coverage for
      a withdrawn finding — keep the test, drop the fix), spec metabox
      `__present`, `safe_back` backslash, `clone_post` tour columns.

## Risk Assessment

The taxonomy-map schema (`es_roots`) is imported WP data —
`localized_taxonomy` resolves through a per-site settings blob.
Generalizing to N langs means **migrating the settings blob shape**:
existing 2-lang sites must keep working (read `es_roots` as a
special case of `{lang}_roots`, or add a migration that rewrites
the key). Pick the additive option: keep `es_roots` readable, add a
generic `lang_roots` map that wins when present.

## Implementation Steps

1. Extend `ThemeDef`/taxonomy-map readers to accept
   `lang_roots: Map<lang, roots>`; keep `es_roots` as fallback.
2. `theme_cat_lang`: iterate registered secondary langs, match
   `-{lang}` suffix → return that lang.
3. `localized_taxonomy`: same generalization.
4. 3-lang test fixture + regression tests.
5. Doc update.

## Verification

- `cargo test` green incl. a 3-lang archive resolution test.
- Existing aucoeur fixtures (fr/es) unchanged — no regression on
  the 2-lang path.
- `docs/architecture.md` language paragraph matches implementation.
