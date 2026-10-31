---
phase: 2
title: "Elementor write_specs + PostFields pull"
status: done
priority: P1
effort: "4h"
dependencies: [1]
---

# Phase 2: Elementor write_specs + PostFields pull

## Goal

Elementor edits persist into `rpress.db`: `_elementor%` post_meta rows
land verbatim, and the compiled `post_content` lands on
`posts.body_html` (+ `body_json` refresh for imported text-box bodies).

## Key facts verified against codebase (do NOT re-derive)

- `render.rs:482` parses `posts.body_json` as a **GDLR/rpress node
  tree** — `_elementor_data` (Elementor's own JSON) must NEVER go
  there; renderer cannot consume it.
- `posts.body_html` IS `post_content` (`schema.rs:292` backfill,
  `render__select_body_html.sql`). Elementor's save path writes its
  compiled markup to `wp_posts.post_content` — pull that.
- Elementor posts imported as generic text-box body_json
  (`[{"template":"element","type":"text-box","value":{"content":…}}]`
  confirmed on blueskies post 3795): `body_html` alone is not enough —
  when `body_json` is exactly this shape it must be regenerated or the
  render keeps serving the stale tree. GDLR bodies (different shape)
  must NOT be touched.
- Pull today only handles `wp_postmeta`/`wp_options`; a new
  `PullTable::PostFields` is required for `post_content`/`post_status`.

## Files to Modify

- `src/admin/bridge/pull.rs`:
  - Add `PullTable::PostFields` with fixed semantics: spec carries the
    wp_posts column set to copy (`post_content` → `posts.body_html`,
    `post_status` → `posts.status`) for resolved wp_id→rpress posts
    whose `body_json` is the generic text-box wrapper (or empty).
    `key_prefix` is unused for this table — validate forbids setting it
    (pass `""`).
  - SQL additions in `src/db/sql/`: select `post_content`/`post_status`
    from `wp_posts`; update `posts.body_html`, `posts.status`, and
    rebuild `body_json` `[{"template":"element","type":"text-box",
    "value":{"content":<post_content>}}]` only when the current
    body_json parses to exactly that shape. Reuse `m04_backfill`
    decision rules (schema.rs:292-326) for the shape check.
- `src/plugins/elementor.rs`:
  ```rust
  static WRITE_SPECS: &[WriteSpec] = &[
      WriteSpec { table: PullTable::PostMeta, key_column: "meta_key",
                  key_prefix: "_elementor", key_map: None },
      WriteSpec { table: PullTable::PostFields, key_column: "meta_key",
                  key_prefix: "", key_map: None },
  ];
  ```
  (`BodyJson` spec NOT used — wrong destination.)
  Remove the spike comment.
- Check overlaps: `_elementor` literal prefix also matches
  `_elementor_css`, `_elementor_edit_mode`, `_elementor_template_type`,
  `_elementor_page_assets` — intended, they're all plugin-owned.
  Verified: no other adapter's literal prefix is a `_elementor`-prefix.

## Tasks & Steps

1. Implement `PullTable::PostFields` + SQL + validate rules.
2. Add specs to `elementor.rs`.
3. Extend `posts__update` coverage tests: after pull,
   `posts.body_html` == wp_posts.post_content for the seeded post;
   generic text-box body_json regenerated; GDLR-shaped body_json
   untouched; `status` mirrors `post_status`.
4. Multilingual note for later (out of scope here): seed is one flat
   post — document in plan that translation-linked saves need a future
   design, no code now.

## Verification

- `cargo test --lib bridge::pull` green incl. PostFields cases.
- Live check on blueskies after an Elementor save:
  `SELECT length(body_html) FROM posts WHERE id=3795` grows/changes and
  `post_meta.meta_key='_elementor_data'` exists.
