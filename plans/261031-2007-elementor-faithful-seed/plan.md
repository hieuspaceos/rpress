---
title: "elementor-faithful-seed"
description: "Fix mu-plugin seed meta insertion eating backslashes (wp_unslash) so Elementor pages open with their real multi-section layout instead of a collapsed text-editor stub; plus GDLR media modal (wp.media) not opening inside the bridge."
status: done
priority: P1
effort: "0.5d"
tags: [bridge, elementor, seed, audit-fix]
created: 2026-10-31
---

# elementor-faithful-seed

## Overview

Observed on blueskies after the persistence fix (plan 261031-1922): opening
post 3795 in the Elementor editor shows **1 container + 1 text-editor
widget**, not the real 91-element layout (16 sections / 25 columns /
50 widgets).

Second defect (user-reported, aucoeur GDLR builder): clicking "add image"
inside a text/other element does not open the WP media frame. The GDLR
image uploader calls `wp.media({...})` (html-option.js:544+) — the bridge
must load `wp_enqueue_media()` + `wp-includes/js/media-*` and serve
`admin-ajax` attachment queries (`query-attachments` action + nonce)
for the frame to populate. Likely missing enqueue, missing nonce for
the attachments query, or a rewrite/asset 404 — root-cause in phase 5.

Root cause verified end-to-end:

- `dump.sql` holds the full `_elementor_data` (29,979 bytes of escaped JSON).
- `project_post` copies it verbatim into `post.json` seed (26,752 raw JSON) ✓.
- mu-plugin `bridge.php` seeds it via `update_post_meta($pid, '_elementor_data', $mv)`.
- `update_post_meta` → `wp_unslash` strips backslashes: `\"` → `"`,
  `\\u00e9` → `u00e9`, `\\'` → `'`. Stored `_elementor_data` becomes
  **invalid JSON** (verified in `wp.db` revisions 3797/3798 — unescaped
  inner quotes at `class="font-claude-…"`).
- Elementor `json_decode` fails → treats post as non-Elementor → rebuilds
  default layout (1 container + text-editor around post_content) → first
  save writes the 10,049-byte stub → our pull faithfully persisted the
  stub. The pull layer is NOT at fault.

Fix: pass `wp_slash($mv)` to `update_post_meta` so unslash returns the
raw string. Uniform across all seed meta — safe for plain values
(nothing to strip) and PHP-serialized values (gdlr: slashes around
quotes restored to originals; verified gdlr seeds contain no `\`/`\\u`
sequences to corrupt).

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | Seed meta written byte-exact (wp_slash) — Elementor data parses | P1 |
| 2 | Editor shows the REAL multi-section tree in browser | P1 |
| 3 | Edit→save→respawn→edit persists across bridge lifecycle | P1 |
| 4 | Aucoeur GDLR builder unaffected (regression check) | P2 |
| 5 | GDLR media modal opens + attach selects work | P2 |
## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [Mu-plugin wp_slash fix + unit verification](./phase-01-slash-fix.md) | Done (superseded: $wpdb-direct byte-exact seeding in installer + mu-plugin; `_elementor_data` 20,162B valid JSON, byte-identical rpress.db↔wp.db) |
| 2 | [Browser: faithful Elementor tree](./phase-02-browser-tree.md) | Done (navigator = 82 elements = seeded tree; iframe DOM = 82 `.elementor-element` nodes; cosmetic: iframe spinner overlay persists — parent ready-event race, non-blocking) |
| 3 | [Browser: edit/save/respawn round-trip](./phase-03-roundtrip.md) | Done (marker edit→wp.db→pull→byte-exact revert verified) |
| 4 | [Aucoeur GDLR builder check](./phase-04-gdlr-check.md) | Done (26 items render, meta byte-exact 78,626 `a:19:`) |
| 5 | [GDLR media modal fix + browser check](./phase-05-media-modal.md) | Done (9,619 attachments seeded; library browses + inserts) |

## Success Criteria

- [ ] `wp.db` `_elementor_data` for post 3795 is valid JSON with
      91 elements (16 section/25 column/50 widget).
- [ ] Elementor editor Structure panel shows multi-section tree, not
      1 container.
- [ ] Edit a widget → Publish → bridge respawn → reopened editor still
      shows the edit inside the real tree.
- [ ] `cargo test --lib` stays green; GDLR editor on aucoeur opens with
      real sections (no regression from uniform wp_slash).
- [ ] Clicking "add image" in a GDLR element opens the WP media frame
      and a selected attachment URL lands in the element's field.

<!-- slug: elementor-faithful-seed -->
