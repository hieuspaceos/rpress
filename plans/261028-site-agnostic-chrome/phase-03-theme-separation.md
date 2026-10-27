---
phase: 3
title: "Theme separation: per-stylesheet base resolution + adapter gating"
status: pending
priority: P1
effort: "2d"
dependencies: [2]
---

# Phase 3: theme separation

## Goal

Base templates resolve per `settings.stylesheet` through theme
adapters. Two mechanics are required (red-team correction):
(a) a custom `set_loader` closure probing `templates/theme/<slug>/`
then shared `templates/` — `minijinja::path_loader` takes ONE dir
(`render.rs:868-869`); (b) converting the ~10 page templates' literal
`{% extends "base_fr.html" if lang != 'es' else "base_es.html" %}`
into `{% extends base %}` with `base` injected from ctx — dir ordering
cannot fix a hardcoded NAME. `theme_traveltour` owns the traveltour
bases + seed gating; `theme_tevily` registers with `ThemeDef` and an
EMPTY template dir — real tevily markup is authored in phase 06.
theme_traveltour's unconditional seeding (taxonomy_map + 28 seed JSONs
with logo refs/'Nos destinations'/imagetraveldmc.com links) is gated on
the site's stylesheet actually being traveltour.

## Findings covered

- `src/lang.rs:97` `template_name` — picks `base_{lang}` → primary →
  `base_fr` → `base_es` by filesystem existence; no theme concept.
- `src/render/chrome.rs:213,222` body SUFFIX, `:491` page-title-wrap,
  `:361-372` og fallback — traveltour values in core (phase 02 made
  them `site.theme.*` ctx vars; this phase moves the *seed source*).
- `templates/base_fr.html`, `base_es.html` — traveltour markup; must
  move under the theme dir or stay as traveltour's base set.
- `templates/theme/base-minimal.html` — orphaned generic base (zero
  references, verified). Deletion is deferred to phase 06: tevily
  bases are authored first (from harvest markup), then this file is
  removed — authoring against a blank dir is safer than deleting the
  only non-traveltour base early.
- `src/admin/plugins/theme_traveltour/` — `mod.rs`, `taxonomy_map.json`,
  `seed/`, `widgets_theme.rs`; `migrate()` runs unconditionally at
  `src/import.rs:746-750` — seeds aucoeur data into every site.
- `src/intake.rs` FORM_SLUGS, `src/render_archive.rs` term maps —
  already data-driven after phase 02; their *seed* lives with the
  adapter that declares the taxonomies.
- `src/render_post.rs:377-391` — head-css splice anchors on
  `id='traveltour-style-core-css'`/`id='tourmaster-custom-style-css'`
  link ids — theme-coupled injection points; anchor ids must come from
  the adapter (or tolerate absence — splice only when the anchor
  exists, which is already the code's behavior via `if let Some(i)`;
  keep tolerant behavior, document it).
- `templates/elements/site_reviews_static.html` — frozen aucoeur
  reviews; move into `theme_traveltour` dir (it is traveltour-theme
  furniture), content unchanged. Deferred per master plan but the
  relocate is one `git mv`.

## Key insights

- **Loader**: `minijinja::path_loader(dir)` takes ONE dir
  (`render.rs:868-869`) — replace with a custom `set_loader` closure
  `probe [templates/theme/<slug>/, templates/]` returning the first
  hit. Same closure shape for every `Environment` build site
  (~6 callsites incl. `template_name`'s existence checks at
  `lang.rs:97-104`: probe theme dir first, then shared).
- **Extends fix**: ~10 page templates hardcode
  `{% extends "base_fr.html" if lang != 'es' else "base_es.html" %}`
  (404/archive/archive_post/page/post/reviews/search/single_post/
  single_tour/tour). All convert to `{% extends base %}`; every render
  ctx (chrome_vars paths AND `bare_ctx` at `generate.rs:596`) injects
  `base` = theme-dir-relative name resolved by `template_name`. A
  missing `base` fails loudly at render — phase-05 smoke is the catch.
- **Stylesheet → theme slug**: `stylesheet` may be a child
  (`traveltour-child`, `tevily_child`). Adapter declares
  `matches_stylesheet(&str) -> bool` — explicit, no normalization
  guessing. `ThemeDef` also carries `preferred_locations: &[&str]`
  (traveltour: `main_menu`, `mobile_menu`, `right_menu`; tevily: read
  from blueskies dump at implementation) for the phase-01 pick chain.
- **Adapter API addition** (`PluginAdapter` trait,
  `src/admin/plugins/mod.rs:18-31`): add
  `theme(&self) -> Option<ThemeDef>` where
  `ThemeDef { matches_stylesheet: fn(&str)->bool, template_dir: &str,
  body_suffix, page_title_wrap, contact_slugs, preferred_locations }`.
  Only theme adapters return `Some`. Public path never calls adapters —
  generate/serve read the resolved `site.theme.*` values baked at
  import/seed time (invariant preserved: the *values* travel via
  settings, the adapter only *writes* them at migrate()).
- **Gating seeding**: `theme_traveltour::migrate()` currently seeds at
  import for every DB. Change: seed only when
  `settings.stylesheet` matches the adapter; else skip (log). For
  already-onboarded sites the bad rows are in place — migration step:
  on migrate, if stylesheet ≠ traveltour AND `plugin:theme:*` rows are
  the aucoeur defaults, delete them (safe: values are frozen seeds,
  re-derivable).
- **theme_tevily adapter**: new `Data` backend adapter, `slug=theme-
  tevily` — registers `matches_stylesheet("tevily*")`,
  `preferred_locations` + minimal `taxonomy_map` seed +
  `theme_locations_available` + `body_suffix` tevily classes. Its
  `template_dir` is `templates/theme/tevily/` which starts EMPTY in
  this phase — loader falls back to shared `templates/` (traveltour
  bases) until phase 06 lands real tevily markup; blueskies is broken
  today anyway, a fallback render is not a regression.
- **`template_name` change**: signature gains the theme template dir —
  callers: `render.rs:509`, `generate.rs`, `render_archive.rs`,
  `render_post.rs` (enumerate at implementation). Lang fallback order
  stays (`base_{lang}` → primary → other bases) but probes theme dir
  first.

## Files to modify

- `src/admin/plugins/mod.rs` — `ThemeDef` + `theme()` on the adapter
  trait (default `None`).
- `src/lang.rs` — `template_name` gains theme dir prefix.
- `src/render.rs`, `src/generate.rs`, `src/render_archive.rs`,
  `src/render_post.rs` — callsites pass the theme dir / stylesheet.
- `src/admin/plugins/theme_traveltour/mod.rs` — implement `theme()`;
  gate `migrate()`/`after_import` on stylesheet match; add
  unmatched-default-cleanup step; absorb `base_fr.html`/`base_es.html`
  into `templates/theme/traveltour/`; relocate
  `site_reviews_static.html`; seed aucoeur-specific settings
  (chaty, jsonld, contact_slugs, banner, logo defaults) via
  `plugin:theme:*`.
- `src/admin/plugins/theme_tevily/` (new) — adapter module only:
  `ThemeDef` (`tevily*` match, `preferred_locations` from blueskies
  dump), `taxonomy_map` + `theme_locations_available` seeds,
  `body_suffix`. `templates/theme/tevily/` created EMPTY — phase 06
  authors the real bases.
- `src/import.rs` — migrate() callsite unchanged location, now
  conditional inside adapter.
- `templates/base_fr.html`, `templates/base_es.html` — `git mv` to
  `templates/theme/traveltour/`. `base-minimal.html` stays until
  phase 06.
- `templates/*.html` (~10 files: 404, archive, archive_post, page,
  post, reviews, search, single_post, single_tour, tour) — literal
  extends → `{% extends base %}`; moved traveltour copies get the
  same change.
- `src/render.rs`, `src/generate.rs`, `src/render_archive.rs`,
  `src/render_post.rs` — every ctx injects `base` (value from
  `template_name`); `bare_ctx` (`generate.rs:596`) included.
- `src/render_post.rs:377-391` — no code change needed (tolerant
  splice); add comment noting anchors are theme-supplied.

## Implementation steps

1. `ThemeDef` + `theme()` trait method; registry helper
   `theme_for_stylesheet(&str) -> Option<&dyn PluginAdapter>`.
2. Custom `set_loader` closure (theme dir → shared) at every env build
   site; `template_name` probes theme dir first; compile-fix all
   callsites (~6).
3. Convert ~10 templates' extends to `{% extends base %}`; inject
   `base` in every ctx incl. `bare_ctx`.
4. `git mv` traveltour bases into `templates/theme/traveltour/` (with
   the extends change already inside); render aucoeur — diff must be
   clean (same templates, new location + name via ctx).
5. theme_traveltour migrate() gating + unmatched-seed cleanup; verify
   dmc DB drops aucoeur `plugin:theme:*` rows.
6. theme_tevily adapter module (no templates yet): matches `tevily*`,
   seeds minimal map/locations/body_suffix.
7. blueskies generate: resolves tevily adapter; loader falls back to
   shared traveltour bases (acceptable interim — phase 06 replaces);
   seeds are tevily's.
8. `cargo check && cargo test`.

## Verification

```bash
cargo check && cargo test
ls templates/theme/                       # traveltour/ tevily/(empty)
grep -rn 'extends "base_' templates src   # expect 0 — all {% extends base %}
sqlite3 site/blueskies/work/rpress.db \
  "select key from settings where key like 'plugin:theme%'"  # tevily seeds
# generate blueskies page → renders (fallback base), no crash
# aucoeur regenerated home diff vs phase-02 output → clean
```

## Todos
- [ ] `ThemeDef` + `theme()` on PluginAdapter (incl. preferred_locations)
- [ ] Custom set_loader (theme→shared) at all env sites; template_name probes theme dir
- [ ] ~10 templates → `{% extends base %}`; `base` in every ctx
- [ ] Traveltour bases moved under `templates/theme/traveltour/`
- [ ] theme_traveltour migrate() stylesheet-gated + stale-seed cleanup
- [ ] theme_tevily adapter module (empty template dir)
- [ ] site_reviews_static relocated
- [ ] Aucoeur diff clean; blueskies renders via adapter+fallback

## Risks

- **File ownership**: phases 03 and 04 both touch `templates/front-css`
  conventions — 03 declares adapter css assets; 04 fills storage.
  Sequential per dependency table; no parallel edit.
- Stylesheet normalization edge (`traveltour-child` vs `traveltour` vs
  future names) — the `matches_stylesheet` hook keeps it explicit;
  default no-match → error logged + shared `templates/` fallback, never
  another site's chrome.
- Blueskies still renders traveltour bases until phase 06 — an
  explicit interim state (no worse than today's aucoeur chrome);
  phase-05 smoke asserts the adapter+seeds resolved even when markup
  falls back.
- Rollback: theme dirs are additive + `git mv` — `git revert` restores;
  gated migrate() is idempotent, rerun re-seeds.
