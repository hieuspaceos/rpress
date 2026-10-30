---
title: "Phase 5: Per-site cached settings + publish_due throttle"
status: todo
---

# Phase 5: Per-site cached settings + publish_due throttle

## Overview

`setting_value()` opens a brand-new SQLite connection **per call**
(`db/misc.rs:44`). `site_url`/`asset_origin`/`lang::config`/
`theme_template_dir` re-query + `read_dir` per render — counted ≥6 conn
opens per `/admin` page and 3-5 redundant reads per miss-render. Separately,
`publish_due` runs a full `status='future'` scan inside `get_sess` on
**every authed request**.

## Requirements

- [ ] Settings reads hit memory, not a fresh conn.
- [ ] `LangConfig` + `stylesheet → theme_template_dir` resolved once per
      site and busted on the same funnel that busts render caches.
- [ ] `publish_due` runs at most once per ~30-60s per site.

## Implementation Steps

1. `SiteCtx` gains `settings: RwLock<HashMap<String,String>>` (or
   `OnceCell<Arc<…>>` + generation counter) loaded lazily from
   `SELECT key,value FROM settings`; `setting_value_conn` variants read it.
   Bust points: every `setting_put`, `bust_cache`, settings-save handler —
   grep all writers and funnel through `sc.settings_bust()`.
2. `lang::config(&conn)` → `sc.lang_config()` cached; `theme_template_dir`
   → cache `stylesheet→dir` resolution (read_dir once).
3. `publish_due` gate: `Mutex<Instant>` per SiteCtx; skip if ran <30s ago
   (semantics unchanged — 'future' flips at minute granularity anyway).
4. `site_data` category N+1 (`render/site_data.rs:88`) → one grouped
   `IN`/`JOIN` query; `resolve_menu_url` per-item settings → hoist;
   `front_id` per-lang recompute → `HashSet` once per render.

## Todo

- [ ] settings map + bust hooks wired
- [ ] LangConfig + theme_dir cached
- [ ] publish_due throttled
- [ ] site_data/menu_url/front_id N+1 fixes
- [ ] rebuild; measure `/admin` p50 and cold miss-render before/after

## Success Criteria

- `/admin` dashboard p50 drops materially from ~390ms.
- Settings save → immediate effect on next render (no stale cache).

## Risk Assessment

- Stale settings: mitigate by funneling ALL writes through the bust hook —
  checklist every `setting_put`/`INSERT INTO settings` site in code review.
