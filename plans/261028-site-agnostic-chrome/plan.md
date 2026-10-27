---
title: "Site-agnostic chrome — dynamic nav, site-scoped base ctx, per-theme templates"
description: "Every rendered page wears aucoeurvietnam.com chrome because base_fr/base_es hardcode its nav/logo/footer and chrome.rs embeds its literals. Make chrome data-driven: header nav from menus/menu_items via theme_locations, site literals from settings/widgets, base templates resolved per settings.stylesheet through theme adapters, and traveltour-style-custom CSS carried per-site. Harvest path and public-path no-adapter invariant stay untouched."
status: pending
priority: P1
effort: "8-9d"
branch: main
tags: [chrome, templates, menus, multi-site, theme-adapter]
created: 2026-10-28
---

# Site-agnostic chrome

## Premise (confirmed by scout, 2026-10-28)

Four sites are served; three are visually broken because the shared
render path embeds aucoeur chrome:

- `templates/base_fr.html` / `templates/base_es.html` — hardcoded
  aucoeur topbar, social links, flag <img> paths, logo
  (`2024/04/logo-agence-de-voyage-au-vietnam.jpg`), ~150 menu <li>,
  footer columns, copyright, chaty config, JSON-LD org block. base_es's
  DESKTOP menu block is verbatim French copied from base_fr.
- `src/render/chrome.rs` — traveltour body-class SUFFIX (`:213`,`:222`),
  hardcoded taxonomy list in `nav_map` (`:257`), `is_contact`
  rappel-gratuit literal (`:337`), og:image fallback post 6444 + aucoeur
  logo (`:361-372`), author `hieutuyen` (`:396`), `slider-1.jpg` banner
  (`:486`), `footer_{1..4}-{lang}` sidebar loop (`:527-531`).
- `templates/front-css/traveltour-style-custom.css` (+ `.scoped.css`) —
  absolute `https://aucoeurvietnam.com` asset URLs; blueskies 404s on
  traveltour CSS because it runs `tevily_child`.
- `src/render_archive.rs:111-200` — frozen aucoeur wp_term_id taxonomy
  map + FR tax_labels; `src/intake.rs` FORM_SLUGS; `src/generate.rs`
  es/fr path heuristics.

Data needed to fix this is already in every site's DB: `menus` +
`menu_items` (imported at `src/import.rs:591-654`, incl. parent_id,
object_id, item_type, raw custom url), `theme_locations` table +
`settings.theme_locations_available` machinery
(`src/schema.rs:108-111`, `src/db/menus.rs:366-400`, admin UI
`src/admin/menus.rs:443-475`), `settings.stylesheet`/`site_name`/
`custom_css`, `terms`/`post_terms`, and `site.menus` already built by
`site_data` (`src/render/site_data.rs:318-363`) but only fed to element
templates — never to the base ctx.

## Goals

| # | Goal |
|---|------|
| G1 | Header nav renders from `menus`/`menu_items` per site+lang; aucoeur renders an equivalent nav from its own imported rows (regression gate) |
| G2 | Base ctx carries `site` (name, menus resolved, flags, social, logo, footer widgets) via `chrome_vars`/`bare_ctx`/`site_data`; no aucoeur literal survives in shared templates or `chrome.rs` |
| G3 | Base template resolution is per-`settings.stylesheet`: theme adapters own `base_*.html` + seed data; `theme_traveltour` seeds only when stylesheet claims traveltour |
| G4 | `traveltour-style-custom.css` is per-site data carried from wp-content/uploads at import, not a repo file with absolute aucoeur URLs |
| G5 | Harvest path (frozen origin HTML) bypasses templates entirely — unchanged; public path stays `fs::read`-only, no adapter/DB work |

## Phases

| # | Phase | Depends on | Effort | Status |
|---|-------|-----------|--------|--------|
| 01 | [Dynamic header nav from menus/menu_items + theme_locations](./phase-01-dynamic-nav.md) | — | 1.5d | Done |
| 02 | [Site-scoped chrome ctx: literals → settings/ctx vars](./phase-02-site-chrome-ctx.md) | 01 | 1.5d | Done |
| 03 | [Theme separation](./phase-03-theme-separation.md) | 02 | 2d | Done |
| 04 | [Per-site theme CSS: traveltour-style-custom from site uploads](./phase-04-site-css.md) | 03 | 0.5d | Done |
| 05 | [Visual smoke harness + aucoeur regression gate](./phase-05-visual-smoke.md) | 01–04, 06 | 1d | Pending |
| 06 | [Tevily base templates authored from harvest/origin markup](./phase-06-tevily-bases.md) | 03 | 1.5d | Pending |

Total ≈ 8 dev-days. Phases 01–02 are sequential (nav lands in ctx
first, then the ctx grows). 04 is small but must wait for 03's adapter
seam. 06 authors the real tevily bases (split out of 03 after red-team:
harvest-markup porting is its own effort, not a skeleton). 05 is the
gate over everything.

## Dependency graph

- **01 → 02**: nav rendering needs `site` in base ctx; 02 generalizes
  the mechanism 01 introduces (menu resolution inside `site_data`).
- **02 → 03**: theme adapters take over the now-parameterized
  templates; separating them before parameterizing would freeze aucoeur
  literals into three template copies.
- **03 → 06**: the tevily adapter + loader seam land in 03; 06 fills
  `templates/theme/tevily/` with real markup ported from blueskies
  harvest/origin captures. `templates/theme/base-minimal.html` is only
  deleted after 06's bases exist.
- **03 → 04**: per-site CSS storage location is decided by the adapter
  contract (theme adapter declares its css assets).
- **05 after all**: it asserts G1–G5 end-to-end.
- Public serve path untouched by every phase — `fs::read` only
  (`src/admin/mod.rs` `read_out`); each phase's verification re-asserts.

## Key decisions

- **Menu location resolution (red-team amended)**: `menus.location` is
  imported as the nav menu term *slug* (`src/import.rs:606` binds slug
  → location column), NOT the WP `nav_menu_locations` map, and WP's map
  is **per-lang** — aucoeur's `main_menu` binds menu 156 (FR) and 579
  (ES) inside one `theme_mods` option, so `theme_locations`' current
  `location→menu_id` shape (`schema.rs:108-111`) cannot hold it. Plan:
  (a) add a `lang` column to `theme_locations` (PK becomes
  `(location, lang)`; update `save_locations`/`list_locations` at
  `db/menus.rs:377-400`, `menus__location_set`/`menus__locations_list`
  SQL, and the admin Manage-Locations signatures at
  `admin/menus.rs:443-475,921-925` — admin assigns per declared
  location; lang filter comes from the menu's own `menus.lang`);
  (b) `theme_mods_*` options are keyed by the **parent** theme slug —
  `theme_mods_traveltour` for stylesheet `traveltour-child` — so import
  probes `settings.template` first, then `settings.stylesheet`;
  imagetravel's dump has ZERO theme_mods rows, so the fallback chain is
  load-bearing, not decorative, and phase 01 includes a no-theme_mods
  verification step; (c) skip `menu_id=0` unassigned entries (aucoeur's
  `right_menu;i:0`); (d) location slugs are theme-owned —
  `main_menu`/`mobile_menu`/`right_menu` for traveltour, NOT
  `main`/`primary`: the render pick chain reads the theme adapter's
  `preferred_locations` (tevily's read from the blueskies dump at
  implementation), then same-`menus.lang` row, then first menu.
- **URL resolution for menu items** (new `src/render/menu_url.rs`):
  resolved URLs stay **`{site_url}`-absolute** — `apply_navx`
  (`render.rs:251-265`) builds absolute hrefs to match, so
  site-relative URLs silently kill every `current-*` class and fail
  the aucoeur regression. `item_type=post_type` → `object_id` →
  `posts` row (join on `posts.wp_id` — wp id, not local id) →
  rel_path rules (kind bases + lang prefix); `taxonomy` → term path
  via `taxonomy_map` (join `terms.wp_term_id`); `custom` → raw url
  with origin strip — **only** the site's own origin + declared alias
  hosts (aucoeur base_fr references `test.aucoeurvietnam.com`); foreign
  hosts (facebook/youtube/…) pass through unchanged. Generalize
  `strip_origin_urls` (`src/admin/plugins/goodlayers/boot.rs:332`)
  into `src/url_util.rs`. `lang-switcher` → flag markup via `flags`.
- **Dead/orphaned menu items**: WP hides a menu item whose object is
  gone. `menu_url` returning None → drop the item AND its subtree —
  never emit `href="#"` (the aucoeur parity diff counts `<li>`). Same
  for orphan `parent_id`: `build_menu_tree`
  (`site_data.rs:351-355`) currently treats a missing parent as root —
  change to drop-subtree.
- **Template mechanism (red-team amended)**: two hard fixes, not one.
  (a) `minijinja::path_loader` takes ONE dir (`render.rs:868-869`) —
  phase 03 installs a custom `set_loader` closure probing the theme
  dir first, then shared `templates/`; `template_name`'s existence
  checks probe the theme dir first too (~6 env/template_name
  callsites: `render.rs:509,868-869`, `render_post.rs`, `generate.rs`,
  `render_archive.rs`, `lang.rs:97-104` — enumerate at implementation).
  (b) Page templates literally `{% extends "base_fr.html" if lang !=
  'es' else "base_es.html" %}` in ~10 files (404/archive/archive_post/
  page/post/reviews/search/single_post/single_tour/tour + base users) —
  dir ordering cannot fix the NAME; every extends converts to
  `{% extends base %}` with `base` injected by the render ctx (value
  = theme-dir-relative `base_{lang}.html` resolved by template_name).
  No new template language, no runtime theme switching.
- **Deferred findings** (out of scope, with reason):
  - `templates/reference/*` — frozen WP captures for diffing; expected
    to contain aucoeur literals (verify-only fixtures).
  - `templates/elements/site_reviews_static.html` — frozen review
    snapshot; real reviews render via `site_reviews()`; static file is a
    fallback only — phase 03 MAY relocate it to the traveltour adapter
    but content stays frozen.
  - `src/schema.rs` fresh-site seeds (aucoeur form pages) — fresh
    `site_init` is a dev convenience; fixing it is cosmetic vs the four
    real sites; noted for a follow-up.
  - `src/lang.rs` `t()`/`month_names` gaps (no vi translation table) —
    a translation-data task, not chrome; blueskies VI chrome still
    renders with EN strings until a VI table lands.

## Risks

- **Aucoeur regression**: its imported `menu_items` must reproduce the
  ~150-item nav WP rendered. Mitigation: phase 05 pixel/structure
  diff on aucoeur home + tour + archive before anything merges; menu
  ordering is `menu_order` + parent tree, already faithful in import.
- **`object_id` mismatch**: `nav_map` doc notes menu_items.object_id
  does not line up with `terms.id` — object_ids are WP ids; resolution
  must join via `posts.wp_id`/`terms.wp_term_id`, not local ids.
  Verification step in phase 01.
- **Menu-less sites**: a site whose import captured no nav_menu term
  falls back to empty nav + theme default links (WP behavior) — never
  crash, never other-site nav.
- **theme_mods absent / per-lang schema change**: some dumps lack
  `theme_mods_*` (imagetravel: zero rows) — import treats as optional,
  fallback chain covers it, but the `(location, lang)` PK change on
  `theme_locations` touches schema + admin save path; existing DBs need
  `ALTER TABLE` handling (add column, backfill lang from menu).
- **Extends-name change**: `{% extends base %}` fails hard when `base`
  is missing from ctx — every render path (incl. `bare_ctx`, widgets
  `template_from_str` fragments) must inject it; compile can't catch a
  template-level miss, the phase-05 smoke does.
- **Alias hosts for strip_origin**: maintained as a small
  `plugin:theme:url_aliases` setting (aucoeur seeds
  `test.aucoeurvietnam.com`); unknown-alias URLs stay absolute — a
  wrong-strip risk is worse than a foreign-host leak.
- **CSS scoping**: `.scoped.css` variants exist for the Live Mode canvas
  (`goodlayers/boot.rs:1534-1538`); per-site CSS needs a scoped twin —
  regenerate by running the same scoper at import, don't ship a second
  manual file.

## Validation sweep (2026-10-28)

- Every `file:line` citation above re-verified against working tree:
  `menus.location` bound to slug `import.rs:606`; `theme_locations` DDL
  `schema.rs:108-111`; `available_locations` `db/menus.rs:370`;
  `chrome_vars` `chrome.rs:301`; `nav_map` literal list `chrome.rs:257`;
  `bare_ctx` `generate.rs:596`; `apply_navx` `render.rs:243`;
  `strip_origin_urls` `goodlayers/boot.rs:332`; `template_name`
  `lang.rs:97`; stylesheet read in `bridge/playground.rs:137-166`;
  `theme_mods` NOT in import allowlist `import.rs:162-166` (gap →
  phase 01 imports it).
- `templates/theme/base-minimal.html` exists and is orphaned (zero
  references — verified). Phase 06 authors real tevily bases FIRST;
  only then is base-minimal deleted.
- Sequencing vs `261027-scoped-plugin-seam`: that plan already built the
  `Data`-backend `theme_traveltour` adapter + `taxonomy_map` option —
  this plan extends it (seed gating, template dir), does not rebuild.
- Red-team amendments applied 2026-10-28: per-lang `theme_locations`
  (`lang` column), `{% extends base %}` ctx fix (10 templates),
  single-dir `path_loader` → `set_loader` closure, absolute-URL
  invariant for apply_navx, drop-not-`#` for dead items, orphan-parent
  drop, theme-owned location slugs, parent-theme `theme_mods` keying,
  tevily-base authoring split into phase 06.

## Success criteria

- [ ] aucoeur home/inner nav identical item-for-item to current output
      (diff on menu <li> sequences, current-* classes preserved)
- [ ] imagetravel header shows ITS Main menu items (VI + EN), its own
      logo/site_name, no aucoeur links anywhere in generated HTML
- [ ] blueskies renders styled under tevily_child: base template +
      tevily css resolve, zero traveltour 404s
- [ ] dmc/imagedmc inner pages use site's own chrome; harvest pages byte
      identical to today
- [ ] `grep -rn aucoeurvietnam templates/ src/ | grep -v reference` → 0
      outside explicitly-scoped files
- [ ] `grep -rn 'traveltour\|rappel-gratuit\|hieutuyen' src/ | grep -v
      plugins/theme_traveltour` → 0
- [ ] `cargo check && cargo test` clean; public path still `fs::read`

<!-- slug: 261028-site-agnostic-chrome -->
