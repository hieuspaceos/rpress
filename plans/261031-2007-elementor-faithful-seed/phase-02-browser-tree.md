---
phase: 2
title: "Browser: faithful Elementor tree"
status: pending
priority: P1
effort: "1h"
dependencies: [1]
---

# Phase 2: Browser — faithful Elementor tree

## Goal

In a real browser, the Elementor editor for post 3795 renders the full
multi-section widget tree — proof the seeded `_elementor_data` parses.

## Steps (all observable, none assumed)

1. `cargo build`; serve blueskies with `PLUGINS_PHP=elementor`.
2. Delete `www/bridge/elementor` (force fresh seed with fixed mu-plugin).
3. Log in (`work/blueskies/credentials.txt`), open
   `/admin/plugins/elementor/wp-admin/post.php?post=3795&action=elementor`.
4. Wait for `#elementor-editor-wrapper`; open the Structure panel.
5. Assert via JS: `elementor.getContainer('document').children.length > 2`
   (real tree ≈ 16 top-level sections, not 1 container).
6. DB check: `sqlite3 wp.db "SELECT json_valid(meta_value) FROM
   wp_postmeta WHERE post_id=3795 AND meta_key='_elementor_data'"` → 1;
   count `elType` occurrences → 91.
7. Screenshot the editor — visual proof of sections/columns/widgets.

## Verification

- Structure panel shows >1 top-level container.
- `json_valid(_elementor_data)=1` in wp.db AND rpress.db after pull.
- No "The content area was not found"/fallback editor in the canvas.
