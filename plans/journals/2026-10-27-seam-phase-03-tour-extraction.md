---
title: "Seam phase 03: tour extraction - byte-parity refactor, zero HTML diff"
date: 2026-10-27
summary: "All tour ownership moved behind PluginAdapter: render_tour.rs relocated, KindBases replaces the threaded tour_base string, widgets/seeds/import hooks adapter-driven; byte-parity verified — diff -r over 1239 generated pages is empty"
---

# Seam phase 03: tour extraction

## What happened

Biggest phase of the seam plan (commit after `204f041`). Three parallel
slices on disjoint file sets, all merged green:

- **Render path**: `render_tour.rs` → `plugins/tourmaster/render.rs`
  verbatim; `render_slug`'s `"tour" =>` arm dispatches
  `adapter_for_kind(kind).render(conn, templates, &RenderCtx)`.
  `RenderCtx` (was `PostCtx`) lives in `plugins/mod.rs` —
  `tour_base` field renamed `kind_base`. `display_slug` became a trait
  method (slug canonicalization is plugin-owned).
- **URL threading**: `KindBases = HashMap<kind, url-base>` resolved once
  per entry point from `KindDef.url_base_key`; `lang::page_path`/
  `page_url` take `kind_base: Option<&str>`; `SiteCtx.tour_base` →
  `kind_bases` + compat helper `admin::tour_base(sc)`. `SiteEnv.tour_base`
  + `--tour-base` stay verbatim (deprecated-by-comment; phase 05 owns
  the migration path).
- **site_data feeds**: the tours JOIN + `tourmaster_general`-derived
  keys moved behind `adapter.site_data_feeds(conn, lang, site_url,
  kind_base)`; site-review testimonials extracted to a core fn (no
  adapter registered for that kind).
- **Widgets**: `sidebar-tour` region, `tour_list`/`tour_category` types +
  field specs, the recent-tours renderer, and 3 seed JSONs moved to
  `tourmaster/widgets.rs` + `tourmaster/seed/`; core merges adapter
  types/sidebars/seeds.
- **Import**: `kind_of` consults `claim_type` per adapter before core's
  post|page match; option whitelist iterates `option_prefixes`
  (`"tourmaster_"` literal gone); `after_import` runs the promote after
  tx commit — a behavior note: promote now also runs on `site build`
  imports, not only at admin boot.
- **Leftovers swept**: admin menus metaboxes registry-driven,
  posts.rs list labels via `kind_def`, chrome.rs `page_url` via
  `kind_url_base`.

## Byte-parity gate

Baseline binary built from pre-phase-03 HEAD in a worktree; both
binaries ran `site build aucoeur-test --only generate` against the same
DB with `site_url` pinned. `diff -r` over 1239 generated pages:
**0 differences** — the gate the phase file demands.

Caveat found mid-verify: `site build` rewrites `settings.site_url` via
the `configure` SQL (LOCAL_URL), so a deploy-written DB value made the
first parity run meaningless; pinning `site_url` before both runs is
required procedure.

## Verified

- `cargo check --all-targets` 0 warnings; 68/68 tests.
- Live: tour single page (FR+ES), `/region/<term>` archive,
  `sidebar-tour` widgets + tourmaster classes in output, `/admin/tours`
  200, dashboard `44 Tours`, `tourmaster: 68 rows promoted` at boot.

## Remaining `"tour"` literals (sanctioned)

Element-type names in `nodes.rs` (`"tour"`, `"tour_search"` are page-
builder data vocabulary), WP-verbatim CSS/rest strings in chrome.rs
(body_class, `wp/v2/tour`), boundary params keeping `tour_base` names
(pub API compat), `SiteEnv`/CLI pass-through, db tests, fixture data.
`tour__*.sql` files kept — renaming to `plugin_<slug>__*` convention
deferred (pure mechanical churn, zero behavior).

## Next

Phase 04 — theme-as-data: CAT_TREE / etiquette / ES_ROOTS / seeds move
from code to per-site data.
