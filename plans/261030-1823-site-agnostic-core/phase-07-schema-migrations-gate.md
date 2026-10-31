---
title: "Phase 7: Schema Migrations + Site-Agnostic Gate"
status: done
priority: P1
---

# Phase 7: Schema Migrations + Site-Agnostic Gate

## Overview

Two infrastructure pieces: (a) a versioned `schema_migrations` runner
replacing the additive-only MIGRATIONS triples, and (b) an enforcement gate
in `check-layers.sh` so "no site/domain literals in src/" stops being
aspirational.

**Land the migration runner early**: phase 4's `post_translations` drop
should ride this runner. Implement runner first (small), then phase 4, then
the gate last (it validates the final tree).

## File Ownership

- `src/schema.rs`, `src/db/sql/schema__*.sql`
- `scripts/check-layers.sh`
- `docs/architecture.md`, `docs/code-standards.md` (doc the gate + migration contract)

## Requirements — migrations runner

- [ ] `schema_migrations(id INTEGER PRIMARY KEY, name TEXT UNIQUE, applied_at TEXT)` table created at boot before MIGRATIONS run.
- [ ] `MIGRATIONS` becomes ordered `&[(name, &str /*SQL or fn-id*/)]`; each runs once inside its own transaction, records `applied_at`, skips if applied. Existing `(table,column,DDL)` triples are ported to named entries (they're already idempotent — the migration row just records them).
- [ ] Non-additive changes (rename/drop/backfill) expressible as migration entries — a `fn(&Connection)` arm in the runner enum, not just SQL strings.
- [ ] First consumer: `drop_post_translations` migration from phase 4 (assert `count(pairs) == count(migrated groups)` pre-drop; panic loudly, never drop silently).
- [ ] Boot-time: runner runs before `plugins::migrate_all`; adapter migrations remain separate (they own their own tables) — document the boundary.
- [ ] Old DBs: `PRAGMA table_info` probes already applied → runner marks those entries applied without re-running (backfill `schema_migrations` by re-probing idempotence, or simply record applied since the triples were no-ops anyway).

## Requirements — agnostic gate

- [ ] `check-layers.sh` gains a literal gate: `grep -rn -E "traveltour|tevily|aucoeur|duree|duracion|region-es|estilo|perfil|blueskies|imagetravel" src/` — matches allowed only inside `src/plugins/theme_traveltour/`, `src/seed/`(if it exists), and lines marked with a `# gate-allow: <reason>` comment (grep `-v` on the marker).
- [ ] Language literals: `grep -rn -E '\b"fr"|"es"|"de"|"/es/"|"/fr/"' src/admin/ src/render*.rs` — allowed only in `lang.rs` internals + gate-allow-marked lines (lang config data lives in settings/DB, not code). After phase 4 this should be near-empty.
- [ ] Gate exits non-zero on violation with the offending file:line list — same style as the existing SQL-layering check output.
- [ ] Docs: `code-standards.md` rule 9 rewritten from "No site/domain literals" (unenforced) to reference the gate + how to mark a legit exception; `architecture.md` updated for the migration runner + theme contract (phase 5 owns theme doc edits — coordinate to avoid double-editing the same sections; this phase owns the migrations paragraph + layering-rule paragraph).

## Implementation Steps

1. `schema_migrations` table + runner in `schema.rs`; port existing MIGRATIONS triples to named entries; verify fresh DB + existing reference DBs all reach the same final schema (`PRAGMA schema_version`-level equality via dump compare).
2. Phase 4 consumes `drop_post_translations` as `Migration::Fn`.
3. Gate: write the grep checks in `check-layers.sh`; run; expect failures only in files phases 4-5 didn't finish — fix or mark, never blanket-allow.
4. Docs pass: architecture.md layering/migration/language sections; code-standards.md rule 9.

## Success Criteria

- Fresh migrate → `SELECT name FROM schema_migrations` lists every migration once; re-boot is a no-op.
- Old-format DB (pre-runner): opens, schema converges, migrations recorded.
- `./scripts/check-layers.sh` fails when a `traveltour` literal is planted in `src/render.rs` and passes after removal (prove the gate with a throwaway test).
- `cargo test` green; `visual_smoke.sh` green.

## Risk Assessment

- Runner must not re-apply an already-applied destructive migration — the `name UNIQUE` + tx-per-migration makes replay impossible by construction.
- Gate false-positives (e.g. a post slug or comment text mentioning "traveltour" inside a string literal that's data, not code): the `gate-allow:` marker is the escape valve; keep the pattern list small and curated.
- `dedupe_post_meta`-style existing one-offs become migrations — port them as `Migration::Fn` with the same body, name them clearly.
