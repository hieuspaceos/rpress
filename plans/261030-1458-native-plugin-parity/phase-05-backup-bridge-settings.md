# Phase 05 — Backup/export + bridge plugin settings surface

## Overview

- **Priority:** P2 — replaces Updraft (present in the dump's history)
  and gives bridged plugins a visible config home.
- **Deliverable:** `rpress site backup` + `/admin/tools/backup` download;
  bridge status/config block on bridged plugins' rows.

## Requirements

### Backup

1. CLI `rpress site backup <site>`:
   - output `work/<site>/backups/<yyyymmdd-hhmmss>.tar.gz`;
   - contents: `rpress.db` (via `sqlite3 .backup` or a `VACUUM INTO`
     to get a consistent snapshot — never raw-copy a live DB) +
     `www/` (uploads + vendored assets) + `sites/<site>.env`;
   - keep last N archives (`backup_keep` setting, default 5);
   - prints size + path.
2. Admin `/admin/tools/backup`:
   - GET = page with "Backup now" button + archive list (name, size,
     date) + download link per archive;
   - POST = runs the same backup routine synchronously (sites are
     ≤500MB DB by the check limit — synchronous is fine; timeout 120s);
   - menu entry under a new "Tools" dock item or Settings —
     decide during impl; default: Settings → "Backup".
3. Restore path is *documented* (`rpress site init --backup
   <archive>` already exists) — no restore UI this phase.

### Bridge plugin settings surface

1. The plugins index (phase 1) renders, for each `Backend::PhpBridge`
   adapter, a `Bridge` panel row:
   - bridge state — workdir path, PHP-CGI socket/binary the bridge
     uses, last pull time — read from the same sources
     `rpress bridge <site> inspect` uses;
   - `Open` → first registered `ScreenDef` path;
   - `Settings` → the adapter's own `settings` screen **if it declares
     one** — bridged adapters implement `screens()` the same way and
     may point it at their bridge settings page; where the adapter
     has no native settings handler, the row shows the bridge screen
     link only (honest — no fake native form over a bridged store).
2. `rpress bridge` config itself (PHP path, workdir, upstream URL) —
   expose the resolved values read-only on `/admin/tools/bridge` or
   fold into each bridged plugin's panel; don't invent a write form
   for things env-driven (`sites/<site>.env`).

## Success criteria

- `rpress site backup aucoeur-test` produces a tar.gz containing a
  `rpress.db` that opens (`sqlite3 :memory: … .restore` smoke) and a
  `www/` tree.
- `/admin/tools/backup` lists the archive, downloads it (Content-
  Disposition), "Backup now" adds a fresh row.
- Bridged adapters' rows on `/admin/plugins` show the Bridge panel
  with real values from the running `site_bridge` state.
- Rotation: 6th backup deletes the oldest.

## Risk

- `VACUUM INTO` doubles disk transiently — the check limit is 500MB,
  so worst case ~1GB scratch; acceptable, note it in the admin copy.
- Download path traversal: serve only `work/<site>/backups/*.tar.gz`
  basenames — reuse `safe_read`/`asset_ext_ok` pattern.
