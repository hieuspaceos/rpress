---
phase: 3
title: "Browser: edit/save/respawn round-trip"
status: done
priority: P1
effort: "1h"
dependencies: [2]
---

# Phase 3: Browser — edit/save/respawn round-trip

## Goal

A real widget edit inside the faithful tree persists through save →
pull → bridge respawn → reopened editor — the full loop this plan set
out to prove.

## Steps

1. In the browser (phase 2 session): pick one heading widget
   (`"Points Forts"`), edit its text via the panel or
   `$e.run("document/elements/settings")`, then Publish/Update.
2. Wait for save ack; check `wp.db` post_content + `_elementor_data`
   contain the marker; check `rpress.db` after pull has the same
   (`post_meta._elementor_data` valid, `body_html` updated).
3. Kill the bridge: `rm -rf www/bridge/elementor` (or delete pid +
   dir). Wait for respawn on next request.
4. Reopen the same editor URL → Structure still multi-section AND the
   edited text is present in the real tree (not a rebuilt stub).
5. Revert the test edit, save again, verify clean state in both DBs.
6. Check `body_json` in rpress.db: if it was the generic text-box
   wrapper, it must now be regenerated around the new `post_content`;
   no `_elementor_data` JSON may ever land in `body_json`.

## Verification

- Marker visible in reopened editor after respawn.
- `_elementor_data` in rpress.db stays json_valid with ~91 elements
  (plus/minus the edit), never the 2-node stub.
- `rpress.db.body_html` == wp.db post_content; `status` mirrors.
- No doubled `/admin/plugins/elementor/admin/plugins/` in proxied
  bodies (plain or `\/`-escaped).
