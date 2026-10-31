---
title: "Phase 4: N-Language Admin — Retire post_translations"
status: todo
priority: P0
---

# Phase 4: N-Language Admin — Retire `post_translations`

## Overview

The biggest blocker. Public render is already N-language via
`post_translation_members`; admin is hardwired fr/es and writes a second
model (`post_translations` with fixed `fr_post_id`/`es_post_id` columns).
User decision: **retire `post_translations` entirely** — one N-lang model,
admin driven by `lang::config`.

## File Ownership

- `src/admin/posts.rs` (~12 literal sites: :76 `/es` prefix, :102-103, :186-190, :371 quick-edit select, :764 lang whitelist, :845 `new_post` default "fr", :1269-1270 flags, :1339-1344 radio pair, :1474)
- `src/db/posts.rs` (:262-263 pair queries, :518-526, :1206 literal `'fr'` member write, pair-table CRUD)
- `src/plugins/mod.rs` (`HEAD_POST`/`HEAD_PAGE` ~L103 literal lang heads)
- `src/plugins/spec.rs` (:63 `HEAD_SPEC`)
- `src/render.rs`, `src/plugins/tourmaster/render.rs`, `src/import.rs` — all `post_translations` readers/writers
- `src/db/sql/` — delete pair-table sql files; new member-table queries
- `src/schema.rs` + `src/db/sql/schema__ddl.sql` — drop table via migration (phase 7 provides the runner; this phase may ship the migration content but should land after 7, or carry an ad-hoc drop — decide ordering: run this phase AFTER phase 7's `schema_migrations` table exists, OR ship the drop as a MIGRATIONS-style imperative with a TODO removed by 7. **Chosen: phase 7 moves before this one in implementation order — see plan notes.**)
- `src/lang.rs` (admin-side helpers if needed: `languages()`, `lang_label(code)`)
- `templates/` admin lang widgets if any contain lang literals

## Data Migration (do first inside this phase)

1. Backfill `post_translation_members` from `post_translations`: for each
   `(fr,es)` pair → `group_id = 'mig-' || fr_post_id || '-' || es_post_id`,
   members `(fr_post_id,'fr'),(es_post_id,'es')`. Skip posts already in a
   group (don't double-group). Must compose with groups imported from
   Polylang — check whether pair rows duplicate member rows before grouping.
2. Rewrite every `post_translations` read path onto members:
   - `posts__select_es_post_id_from` → group-member lookup by post_id+lang.
   - `render__select_fr_post_id_es_post_id` → members join.
   - `render__select_front_pair` → front group member query (exists: `render__select_front_group_member`).
   - `posts__insert_into_post_translations`/`posts__delete_from_post_translations` → member insert/delete within group (create group if absent).
3. Drop `post_translations` + indexes `idx_trans_pair`, `idx_trans_es` (schema.rs:57-62) via the migrations runner (phase 7) or interim imperative.

## Requirements

- [ ] All lang UI in admin/posts.rs driven by `lang::config(conn).languages` (list of langs + primary): quick-edit `<select>` options, radio pairing, flags (lang.rs already has `flag_file`), `/es` URL prefix → `prefix_for(cfg,lang)`, `new_post` default → `cfg.primary`.
- [ ] `quick_edit`/`save_post`/`bulk_apply` lang whitelist: `cfg.languages.contains(lang)` instead of `matches!("fr"|"es")` (posts.rs:764).
- [ ] Pairing UI generalizes: "linked translation" picker lists all languages lacking a member in this post's group (N-lang pairing), not a single fr↔es radio. Keep UI simple: dropdown of lang + post picker per lang.
- [ ] `db/posts.rs:1206` — member mirror writes the post's actual lang, not literal `'fr'`.
- [ ] `HEAD_POST`/`HEAD_PAGE`/`HEAD_SPEC` lang head literals → generated from `cfg.languages` (plugins/mod.rs ~L103, spec.rs:63).
- [ ] `import.rs` writes members only (keep writing `post_translations`? No — import targets the new model directly; the `import__insert_or_ignore_11` pair insert deleted or repurposed to member insert).
- [ ] `render.rs` + `tourmaster/render.rs` pair readers → members.
- [ ] Edit screen "language" section shows N languages; "translations" metabox shows group members with per-lang add/edit links.
- [ ] Update `docs/architecture.md:135-138` language-model section + README admin caveat line.

## Implementation Steps

1. Inventory every `post_translations`/`fr_post_id`/`es_post_id`/`'fr'`/`'es'`/`"/es"`/`"fr"`/`"es"` literal in src/ (grep list — start from audit's ~12 sites + sql files above). Produce the literal ledger; every entry either deleted or driven by cfg.
2. Migration SQL + runner hook; verify on a copy of a real site DB: group coverage = 100% of former pairs; no orphaned members.
3. `LangConfig`-driven admin: add `fn langs(c:&Connection)->Vec<(code,label,flag)>` helper if needed; thread `cfg` into the post edit/list render fns (they already have conn access — build cfg once per screen).
4. Pairing writes: `link_translation(conn, post_id, lang, other_post_id)` = ensure group exists for post_id (create `group_id = 'g-'||post_id` if none) → insert member `(group, lang, other)`. `unlink` removes member; group with 1 member may persist (harmless) or auto-delete — pick auto-delete for cleanliness.
5. Update tests: `tests/cache_coherence.rs`, `render__test_*` sql fixtures use pairs — reseed via members (fixture files already show member-seed style: `render__test_insert_group.sql`).
6. New test: seed a 3-lang site (fr/es/de) → admin list renders 3 lang filters; quick-edit a post to `de` persists; pair fr↔de creates group; hreflang on public render includes all 3.

## Success Criteria

- `grep -rn "post_translations\|fr_post_id\|es_post_id" src/ tests/` → only the migration code itself.
- `grep -rn '"fr"\|"es"\|"/es"\|matches!(.*fr.*es' src/admin/` → empty (cfg-driven).
- A seeded de-site: create post lang=de, pair with fr sibling, public render emits hreflang fr+es+de alternates — verified by test or throwaway script.
- Existing fr/es sites: after migration, admin pairing UI identical for 2-lang users; visual_smoke passes.
- `cargo test` + `check-layers.sh` green.

## Risk Assessment

- **Data loss**: dropping `post_translations` loses pairs not yet in members — mitigation: migration asserts `count(pairs) == count(migrated groups)` before DROP; keep the table (deprecated, renamed `_post_translations_legacy`) for one release? User chose retire — implement assert + drop, no legacy keep.
- Admin UI for N>2 pairing is a new interaction — keep it a simple per-lang select, don't build a matrix.
- `import.rs` dump may carry Polylang pair data — check the dump parser path writes members directly (audit says it already does for members; remove pair insert only after confirming no consumer).
- Ordering: this phase touches schema drop — sequence AFTER phase 7's runner exists, or implement drop imperatively and let 7 absorb it into migrations history. Plan order lists 7 last; **cook should run the migration-runner part of phase 7 first** (it's small) — noted in phase-07 as "land runner early".
