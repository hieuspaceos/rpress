---
phase: 4
title: "Bridge constants fallback from dump.sql (stylesheet / active_plugins)"
status: done
priority: P2
effort: "2h"
dependencies: [1]
---

# Phase 4: Bridge constants fallback from `dump.sql`

## Problem (code-reviewer finding, medium)

`playground.rs` reads `stylesheet` and `active_plugins` from `settings` —
keys only exist when the site was imported **after** `import.rs`'s
allowlist grew them. blueskies/dmc/imagedmc DBs (imported pre-cutover) lack
both → bridge boots `twentytwentyfive`, no theme filters
(`gdlr_core_skin_option_slug`), no element-provider activation (tour/room
elements would 500 on sites that have tourmaster mounted).

Their `dump.sql` files still exist (`work/<site>/dump.sql`) and contain the
rows — `grep "'stylesheet'" dump.sql` → `'stylesheet','tevily_child'`.

## Change

In `playground.rs` `spawn()`, after the `settings` reads come up empty,
fall back to parsing `<site>/../dump.sql` — the import *source* — for the
two option rows. Reuse the same SQL literal parsing the importer uses
(check `src/import.rs` — the option row regex is already there; extract a
small helper `option_from_dump(dump: &Path, key: &str) -> Option<String>`
in `import.rs` or `db/misc.rs`, unit-test it on a fixture line).

Fallback applies to both keys; `settings` row still wins when present
(import-time edits/migrations stay authoritative).

Also write the resolved values back into `settings` (`INSERT OR REPLACE`)
so the fallback runs once per site, not per spawn — the DB stays the
single read path afterward.

## Edge cases

- `dump.sql` missing/huge → read is a grep-style line scan, not full parse;
  bail silently to current behavior (theme-less bridge still works for
  plain PB editing — just no skin slug filter).
- SQL-escaped quotes (`\'`/`\\"`) inside the serialized value — reuse the
  importer's unescape, don't re-implement.
- `active_plugins` unserialize failure → skip `RPRESS_BRIDGE_PLUGINS`
  (current behavior), still seed `stylesheet`.

## Verify

```bash
cargo test --lib   # new helper unit tests
# live: blueskies spawn → mu-plugins theme = tevily_child
sqlite3 work/blueskies/rpress.db \
  "SELECT value FROM settings WHERE key='stylesheet'"   # after first spawn
pgrep -fl wp-playground-cli | grep -o 'RPRESS_BRIDGE_THEME [^ ]*'
```

## Risks

- None structural: read-only on dump, idempotent seed write.
- If a site's dump.sql was deleted post-import, behavior = today's (silent
  skip) — documented in the helper comment.
