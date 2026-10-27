---
phase: 3
title: "Tour extraction: render/generate/lang/widgets/import decoupling"
status: pending
priority: P1
effort: "3d"
dependencies: [2]
---

# Phase 3: tour extraction — the biggest move

## Goal

Core no longer contains the string `"tour"` outside the registry lookup.
`render_tour.rs`, `tours`-table JOINs in `render.rs` site_data, tour
widgets (`widgets__recent_tours`, `tour_category` widget type,
`sidebar-tour` seeds), tour URL rules in `lang.rs`/`generate.rs`, and the
`import.rs` `kind_of`/`tourmaster_general` literals all become
adapter-driven. `tour_base` stops being a core `SiteEnv` field and
becomes the adapter's `url_base` config. Public HTML output is
byte-identical.

## Findings covered

- **T9** — `render.rs` hardcodes tour in `KINDS`-style lists
  (`src/render.rs:89`), dispatch arm `"tour" => render_tour` (`:522-546`),
  display-slug/canonical special cases (`:260-277`, `:822-834`,
  `:1009-1010` JOIN tours, `:1113-1115` tour JSON, `:1357` site_data),
  `tourmaster_general` read (`:1244-1249`), `post_markdown` tour branch.
- **T10** — `generate.rs` tour branches: raw-redirect display slugs
  (`:73-93`), `rel_path_with_cfg` `kind=="tour"` prefix (`:248-256`),
  `post_out_rel` (`:264-284`), taxonomy archive comment/loop (`:172-194`).
- **T11** — `lang::page_path`/`page_url` `kind=="tour"` → `tour_base`
  segment (`src/lang.rs:80-109`) — the URL layout rule itself is
  plugin-owned.
- **T12** — `widgets.rs`: `tour_category` widget type
  (`src/widgets.rs:133`), recent-tours card renderer + SQL
  (`src/widgets.rs:974-1010`, `widgets__recent_tours.sql`), sidebar-tour
  seeds (`src/widgets.rs:281-283,323-327`, `src/seed/sidebar-tour-*.json`),
  `tourmaster-tour-option` unserialize at `:990`.
- **T13** — `import.rs`: `"tour" => Some("tour")` (`:298`),
  `tourmaster_general` option whitelist (`:137`); tours rows are
  populated post-import by `promote_tours`, not in the posts loop — the
  `import_hooks` seam is therefore `kind_of` + option-prefix claims, not
  a per-row insert callback (simpler than roadmap sketch).
- **T14** — `SiteEnv.tour_base` (`src/site.rs:45,98,124`),
  `SiteCtx.tour_base` (`src/admin/mod.rs:43`), CLI `--tour-base`
  (`src/site_init.rs:30,69`, `src/site_deploy.rs:434-447`), and ~25
  threaded params (`render.rs`, `render_tour.rs`, `generate.rs`,
  `sitefx.rs`, `site_build.rs:514,546-547`, `site_deploy.rs:246`,
  `site_serve.rs`) — source-of-truth swap, not de-threading (R2).

## Key insights

- **URL layout = adapter data, not core code.** `KindDef.url_base_key:
  Option<&str>` (phase 01) resolves once per call-site *entry point*:
  `generate::run`, `render::render_slug`, `emit_site_files`,
  `admin::serve`. Each already receives a `tour_base: &str`; phase 03
  changes only *who computes the string* —
  `resolve_url_base(env_or_settings) -> &str` at the top of each entry
  point — and renames the param to `kind_bases: &KindBases` (small map
  `{kind → url_prefix}`). Functions below keep one string because only
  the *resolved* kind's base is ever needed downstream (R2). The
  `lang::page_path` signature changes from `tour_base: &str` to
  `kind_base: Option<&str>` — same threading depth.
- **render_tour.rs becomes `plugins/tourmaster/render.rs`** behind
  `adapter.render(conn, post, ctx) -> Option<Html>`; the `"tour" =>`
  dispatch arm in `render.rs:522` becomes `adapter_for_kind(kind).render(...)`.
  `display_slug`, `translation_urls`, `tour__*.sql` queries move with it.
- **site_data tours JOIN** (`src/render.rs:1000-1010`) is a template
  data feed — generalize to `adapter.site_data_feeds(conn, lang) ->
  Vec<(key, Value)>`; tour adapter emits `"tours"` key; core merges.
  Same for `site-review` testimonials feed (`src/render.rs:1131-1136`) —
  extract behind the same hook now (cheap, proves generality).
- **Widgets**: `tour_category` + `recent_tours` renderers move behind
  `adapter.widget_types() -> &[WidgetDef]`; seed JSON files move to
  `plugins/tourmaster/seed/`; `sidebar-tour` sidebar registration
  (`src/widgets.rs:80`) becomes adapter-declared `sidebars()`.
- **Import**: `kind_of` consults `adapter.import_hooks().wp_post_types`
  first (`"tour"→"tour"`, `"site-review"→"site-review"`), falls back to
  core `"post"|"page"`. Option whitelist gains
  `hook.option_prefixes` (`"tourmaster_"`) instead of the literal
  (`src/import.rs:137`). Post-import, `hook.after_import(conn)` runs
  `promote_tours` — same work, plugin-owned (R4).
- **`SiteEnv.tour_base` migration**: `TOUR_BASE=` stays the persisted
  key (back-compat) but becomes `plugin:tourmaster:url_base` in
  `SiteEnv`'s semantic meaning; `SiteEnv` keeps the field for one
  release, marked `#[deprecated]`-style comment, sourced into
  `KindBases` at entry points. CLI `--tour-base` flag stays, documented
  as "tour plugin URL base" (phase 05 covers rename path).
- **`tax archive path rules`** in `generate.rs:172-194` +
  `render_archive.rs` — tour taxonomies get `rel` prefix rules from
  adapter `taxonomy_bases()` map (`tour_category→"tour-category"` etc. —
  read the literal map in `render_archive.rs` at implementation; it is
  currently inline in archive path fns).

## Files to modify

- `src/admin/plugins/tourmaster/` — new `render.rs` (moved
  `render_tour.rs`), `widgets.rs` (recent_tours + tour_category),
  `seed/` (sidebar-tour-*.json), `import.rs` (wp type map + option
  prefixes + after_import).
- `src/render_tour.rs` — deleted (content moved).
- `src/render.rs` — remove tour literals; dispatch via adapter
  `render()` + `site_data_feeds`; `site-review` feed extracted to its
  own hook.
- `src/generate.rs`, `src/sitefx.rs` — `tour_base: &str` →
  `&KindBases`; rel-path fns consult bases.
- `src/lang.rs` — `page_path`/`page_url` take `kind_base: Option<&str>`.
- `src/widgets.rs` — registry-driven widget types/sidebars/seeds;
  remove tour literals.
- `src/import.rs` — `kind_of` via registry; option whitelist via
  `option_prefixes`.
- `src/site.rs`, `src/site_init.rs`, `src/site_serve.rs`,
  `src/site_build.rs`, `src/site_deploy.rs`, `src/admin/mod.rs` —
  `tour_base` sourcing → `KindBases::resolve`.
- `src/db/posts.rs`, `src/admin/posts.rs` — `plugins::find` →
  `adapter_for_kind`; `save_tour_option` deleted (moved in phase 02).
- `src/main.rs` — CLI flag doc/through-plumbing.

## Implementation steps

1. Add `KindBases` (`HashMap<&'static str, String>`) +
   `resolve(env, settings, registry)`; thread through entry points
   listed in T14 — pure refactor, no behavior change; `cargo test`.
2. `import.rs`: registry-driven `kind_of` + `option_prefixes` +
   `after_import`; move `promote_tours` call into hook.
3. Move `render_tour.rs` → `plugins/tourmaster/render.rs`; swap
   `render.rs:522` arm for adapter dispatch; move `tour__*.sql`
   ownership (files stay in `src/db/sql/` — document location
   convention: plugin SQL files prefixed `plugin_<slug>__*`; rename
   `tour__*` → `plugin_tourmaster__*` mechanically).
4. `site_data` feeds: implement `adapter.site_data_feeds`; move tours
   JOIN + testimonials SELECT behind it.
5. Widgets: `WidgetDef`/`sidebars()`/`seeds()` on adapter; move
   renderers + seeds + `tour_category` type.
6. `lang.rs` signature swap; `generate.rs`/`sitefx.rs` `KindBases`.
7. `SiteEnv.tour_base` → deprecated passthrough; `SiteCtx.tour_base`
   → `kind_bases`; update `admin/mod.rs` ~10 `sc.tour_base` reads
   (`:310,455,495-501,523,544-549`).
8. Final grep: `"tour"|tour_base|TOUR_BASE|tourmaster` in `src/` must
   hit only `plugins/tourmaster/**`, registry internals, and the
   deprecated SiteEnv passthrough.

## Verification

```bash
cargo check && cargo test
cargo test generate render
grep -rn 'tour' src --include='*.rs' -l | grep -v plugins/tourmaster   # only passthrough sites
# Byte-parity: rebuild a known site before/after, diff -r out/
```

Manual: full `site build` on traveltour fixture → `diff -r` old vs new
`out/` tree must be empty; tour page render, tour archive, tour sidebar
widgets, recent-tours cards, sitemap, redirects.map, llms.txt all
present and identical; `/admin/tours` CRUD roundtrip.

## Todos

- [ ] `KindBases` + resolve + threaded (step 1 compiles green alone)
- [ ] import_hooks dispatch + after_import(promote_tours)
- [ ] render_tour move + adapter render dispatch + sql renames
- [ ] site_data_feeds (tours + site-review)
- [ ] widget types/sidebars/seeds via adapter
- [ ] lang.rs/generate.rs KindBases swap
- [ ] SiteEnv/SiteCtx passthrough deprecation
- [ ] Literal sweep green; byte-parity diff on fixture site

## Risks

- **Byte-parity is the gate.** Any HTML diff = bug; the phase is not
  done until `diff -r` is empty on the fixture site.
- `sql!()` rename (`tour__*` → `plugin_tourmaster__*`) touches every
  `sql!` reference — mechanical, compiler catches misses; keep file
  list small by renaming files + macro args together.
- `KindBases` map lookup is per-call — negligible vs render cost.
- `tour` kind in existing DBs has no registered adapter if tourmaster
  feature is off → rows exist but no renderer; serve still works
  (static files) — document: unregistered kinds are invisible in admin
  lists and skipped in generate (matches "off switch" comment at
  `src/admin/plugins/mod.rs:3-4`).
