---
title: elementor-save-persistence
date: 2026-10-31
---

# elementor-save-persistence

Implemented plan 261031-1922-elementor-save-persistence (4 phases).

- Pull matching moved from SQL LIKE to literal `instr(col,?1)=1`
  (postmeta + options). `WriteSpec::validate` now forbids `%`, allows
  `_`; call sites stripped their trailing `%` (gdlr-core, traveltour).
  Semantic note: literal prefix is slightly wider than `X-%` (a
  hypothetical `gdlr-coreish` key now matches) — bounded, prefix-anchored.
- New `PullTable::PostFields`: wp_posts.post_content → posts.body_html,
  post_status → posts.status, resolved via posts.wp_id. body_json is
  rebuilt ONLY when it's the generic single text-box import wrapper or
  empty (m04 rules); GDLR shapes untouched. Elementor WRITE_SPECS:
  `_elementor` PostMeta + PostFields — saves now persist into rpress.db.
  `key_map`/`key_prefix` unused → validate enforces `""`/None.
- seeded_post truthful: php_cgi/playground only set it when
  project_post returned true; reseed_post returns Ok(false) when
  pull::locate finds no wp.db (flag not cleared → can't claim).
- Escaped doubled-mount collapse fixed: `\\/wp-` → `\/wp-`; extracted
  collapse_doubled_mount() for testability; followers gained `}` `,`.
  body_json read errors now abort pull (no silent wrapper rebuild).

Verified: cargo test --lib 76/76 green incl. 5 new pull/collapse tests.
Live smoke on blueskies: elementor editor loaded post 3795 (2.6MB),
pull copied 15-18 `_elementor*` rows into post_meta, body_html present;
bridge dir deleted + respawn → editor reloads saved layout; post 27
reseed switch works; /wp-content/rpress-seed/post.json 404; zero
doubled mounts (plain+escaped) in proxied bodies.

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
