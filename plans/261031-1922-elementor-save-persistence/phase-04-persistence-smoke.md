---
phase: 4
title: "End-to-end persistence smoke"
status: done
priority: P2
effort: "1h"
dependencies: [1, 2, 3]
---

# Phase 4: End-to-end persistence smoke

## Goal

Prove the whole loop on blueskies: Elementor edit → pull-back →
respawn → edit still there; post-switch reseed still correct.

## Steps

1. `cargo build`, serve blueskies with `PLUGINS_PHP=elementor`.
2. Login, open `/admin/plugins/elementor/wp-admin/post.php?post=3795&action=elementor`.
3. Make a trivial edit (or POST a save via admin-ajax from the editor)
   → confirm `post_meta._elementor_data` row lands in `rpress.db`.
4. Kill the bridge (`sqlite3`-free: delete `www/bridge/elementor` or
   wait TTL / send reap) → reopen the editor → saved layout visible.
5. Switch to `post=27` → editor loads that post (reseed path).
6. Check public URL `/wp-content/rpress-seed/post.json` → 404.
7. Grep proxied body for `admin/plugins/elementor/admin/plugins/` → none.

## Verification

- All seven steps observed, not assumed.
- `cargo test --lib` green at end.
