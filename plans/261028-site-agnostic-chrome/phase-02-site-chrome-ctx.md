---
phase: 2
title: "Site-scoped chrome ctx: literals → settings/ctx vars"
status: done
priority: P1
effort: "1.5d"
dependencies: [1]
---

# Phase 2: site-scoped chrome context

## Goal

Every literal in `chrome.rs` and every aucoeur string in the shared
base templates is either (a) a `settings` value, (b) a widget/seed
value, or (c) a template `{{ var }}` fed by `chrome_vars`/`bare_ctx`/
`site_data`. After this phase a site with no aucoeur data renders zero
aucoeur strings.

## Findings covered

- `src/render/chrome.rs` — SUFFIX body class (`:213`, `:222`), nav_map
  literal (`:257`, fixed in phase 01), `is_contact` rappel-gratuit pair
  (`:337`), og:image fallback `6444` + aucoeur logo path/alt (`:361-372`),
  author `hieutuyen` (`:396`), `slider-1.jpg` banner (`:486`),
  `traveltour-page-title-wrap` (`:491`), footer sidebar `footer_{1..4}-
  {lang}` loop (`:527-531`).
- `templates/base_fr.html`/`base_es.html` — topbar links, social hrefs,
  flag `<img src>` hardcoded paths, logo `<img>`, footer column bodies,
  copyright text, sticky slider markup, chaty config JSON, JSON-LD org
  block.
- `src/intake.rs` FORM_SLUGS — 6 hardcoded aucoeur CF7 slugs (move to a
  setting so per-site forms resolve).
- `src/generate.rs` `bare_ctx` (`:594-651`) — missing all of the above
  for post-less renders (404, archives).

## Key insights

- **Three literal classes, three mechanisms** — decide per literal:
  | Literal | Mechanism |
  |---|---|
  | site_name/logo/copyright/social/flag paths | `settings` keys (logo → `plugin:theme:logo` media path, or `site_logo` media id — pick media id, consistent with `featured_id` handling) |
  | footer column bodies | widgets — already seeded via theme seeds (`footer_{1..4}-{lang}`); render reads them, no new mechanism |
  | body SUFFIX / page-title-wrap / og fallback | theme-adapter config keys (traveltour-specific), phase 03 fully moves them — here they become `site.theme.*` ctx vars the adapter seeds |
  | chaty config / JSON-LD org block | `plugin:theme:*` JSON settings; template `{% if site.chaty %}` guarded — absent → nothing rendered |
  | `is_contact` slug | `settings.contact_page_slug` per lang, or compute: post whose id == `theme_locations`-style `contact_page` setting — simplest: `plugin:theme:contact_slug` JSON map lang→slug |
  | FORM_SLUGS | `settings.form_slugs` JSON array, seeded by import from captured CF7 forms (form posts already imported — scan `post_type=wpcf7_contact_form`/`flamingo`/`page` with form markup at import and persist slugs) |
  | author `hieutuyen` | posts.author/display_name from the post's `post_author` — the value should come from `users`/postmeta, not a constant |
  | `slider-1.jpg` banner + og fallback 6444 | `settings` keys `banner_media`/`default_og_media`; empty → omit block |
- **One ctx shape**: `chrome_vars` and `bare_ctx` both build `site`
  = `{name, url, langs, flags, menu_main, menu_footer, footer_widgets,
  social, logo, chaty, jsonld, theme{body_suffix,page_title_wrap,
  contact_slugs,banner,default_og}}` — plus `base` (base-template name
  for the `{% extends base %}` conversion landing in phase 03; inject
  it here so phase 03 is pure template edits). `site_data` grows
  these; templates reference `site.*` only — base_fr and base_es
  converge toward identical structure (full merge happens in phase 03
  when theme dirs land).
- **Settings seeding on import**: importer already writes `blogname`,
  `stylesheet`, `custom_css`, `languages`; extend the same allowlist
  region (`import.rs:162-166`) for: `blogdescription`, logo (WP
  `theme_mods.custom_logo` → attachment id), social links (theme mods /
  widget payloads — if aucoeur's social links came from widget data the
  seed is already present; verify during implementation which source
  carried them, prefer widget), `nav_menu_locations` (phase 01).
- **404/archives**: `bare_ctx` gets the full `site` map — same helper,
  no forked ctx shape.

## Files to modify

- `src/render/chrome.rs` — replace every literal above with ctx reads;
  `body_class` takes the suffix from `site.theme.body_suffix`
  (signature change — update callers `render.rs`, `render_post.rs`,
  `render_archive.rs`, `generate.rs`).
- `src/render/site_data.rs` — `site_data` emits the extended `site`
  map (media path lookups for logo/banner via `media_detail`).
- `src/intake.rs` — FORM_SLUGS → `settings.form_slugs` lookup w/
  empty-default.
- `src/generate.rs` — `bare_ctx` injects `site`; `term_out_paths`
  es/fr heuristics (`:333-352`) → `taxonomy_map` (already the seam-plan
  mechanism); 404 body class literal (`:612`) → `site.theme.body_suffix`.
- `src/render_archive.rs` — wp_term_id map (`:111-200`) + FR tax_labels
  → `taxonomy_map` + per-site `terms.lang` (archive markup stays; data
  source swaps). DESC_PAGES alias → `taxonomy_map` alias entries.
- `src/import.rs` — options allowlist + post-import seed pass writing
  `plugin:theme:*` and `form_slugs` settings (idempotent, `INSERT OR
  IGNORE`-style via `setting_set` only when absent).
- `src/db/settings.rs` (or wherever `setting_get` lives — follow
  `db/menus.rs` pattern) — typed getters for the new keys.
- `templates/base_fr.html`, `templates/base_es.html` — literals →
  `{{ site.* }}`/`{% if %}` guards; footer columns iterate
  `site.footer_widgets`; JSON-LD emitted from `site.jsonld` var.

## Implementation steps

1. Inventory pass: grep every `aucoeur|hieutuyen|slider-1|rappel` in
   `src/` + `templates/` (excluding `reference/`), produce the literal
   table, assign mechanism per the table above.
2. Settings keys + getters + import seed pass.
3. `site_data` extension; `chrome_vars`/`bare_ctx` emit `site` + `base`.

4. `body_class` signature change + all callers.
5. Template rewrite: base_fr first, diff aucoeur output; port the same
   diff to base_es (fixing its copied-French desktop menu as a bonus —
   dynamic nav already covers it via phase 01 macro).
6. FORM_SLUGS → setting; verify form intake on aucoeur still matches
   (its 6 slugs must be seeded).
7. `cargo check && cargo test`; grep sweep → 0 literals outside
   adapter/seed dirs.

## Verification

```bash
cargo check && cargo test
grep -rn 'aucoeur\|hieutuyen\|rappel-gratuit\|slider-1' src templates \
  | grep -v 'reference/' | grep -v 'plugins/theme_traveltour'   # expect 0
sqlite3 site/imagedmc/work/rpress.db \
  "select key from settings where key like 'plugin:theme%' or key in
   ('form_slugs','blogdescription')"          # seeded for real sites
# render imagetravel page; grep generated HTML for 'aucoeur' → 0
```

## Todos
- [ ] Literal inventory table produced
- [ ] Settings keys + import seed pass
- [ ] `site` map in `site_data`; `chrome_vars` + `bare_ctx` emit it
- [ ] `body_class`/`is_contact`/og/banner/author literals ctx-driven
- [ ] base_fr/base_es literal-free; aucoeur diff clean
- [ ] FORM_SLUGS → `form_slugs` setting
- [ ] render_archive term map + labels data-driven

## Risks

- `body_class` signature change touches 4+ callers — compile-checked;
  risk is a missed *dynamic* caller (none found; all static).
- Social links source unknown until implementation — if they live in
  widget JSON, phase 1 seeds already carry them and no new setting is
  needed; decide on first site inspected.
- Aucoeur visual diff may shift whitespace where guarded blocks emit —
  regression gate compares structure/text, not bytes, for this phase.
