---
phase: 1
title: "Mu-plugin wp_slash fix + unit verification"
status: pending
priority: P1
effort: "1h"
dependencies: []
---

# Phase 1: Mu-plugin wp_slash fix + unit verification

## Goal

`update_post_meta` in the seed path receives `wp_slash`'d values so
`wp_unslash` inside WP stores the byte-exact raw value.

## Facts verified (do NOT re-derive)

- `src/admin/bridge/mu-plugin/bridge.php:269-280` — the seed loop:
  `foreach ($j['meta'] as $mk => $mv) { update_post_meta($pid, $mk, $mv); }`
- `update_post_meta`/`add_post_meta` unslash every value before the
  INSERT — the WP-meta API contract is "pass raw, get raw" ONLY for
  values without backslashes; JSON/serialized strings must be pre-slashed.
- `wp.db` proof: revisions 3797/3798 hold the corrupted 26,330-byte
  `_elementor_data` (unescaped inner quotes, `u00e9` without backslash).
- GDLR meta (`gdlr-core-page-builder`) is PHP-serialized with NO `\`
  chars — survives unslash today and stays correct under wp_slash
  (slashes added then removed → identity).
- `wp_insert_post` post_content path is unaffected (content had no `\`
  sequences; also `sanitize_post_field` unslashes — but post_content in
  WP IS the slashed-unslashed round trip by design; leave it).
- Also check: `reseed_post` (bridge/mod.rs) rewrites the SAME post.json
  via `project_post` — the mu-plugin seed loop is the single insertion
  point for meta; no second path needs the fix.
- `maybe_serialize`: `update_post_meta` serializes arrays; `$mv` is a
  string from json_decode($j['meta']) — a PHP string, not array → stored
  verbatim after unslash. Correct.

## Files to Modify

- `src/admin/bridge/mu-plugin/bridge.php` — seed loop:
  `update_post_meta($pid, $mk, wp_slash($mv));` with a comment naming the
  unslash contract (one line changed + comment).
- OPTIONAL hardening (same file): after seeding, immediately
  `get_post_meta` + `json_decode` `_elementor_data`; if it fails, log to
  `error_log` so a future silent corruption is loud in the bridge log.

## Tasks & Steps

1. Apply the one-line wp_slash change + comment.
2. Add the optional `json_decode` sanity log (cheap, catches regressions).
3. `cargo build` (mu-plugin is a file asset; confirm it's embedded via
   `include_str!`/`include_bytes!` or copied at spawn — check how
   bridge.php reaches the docroot; adjust nothing else).

## Verification

- `cargo test --lib` green (no new unit coverage needed — mu-plugin is
  PHP; verification is the live check in phase 2).
- Manual: respawn bridge → `sqlite3 wp.db "SELECT meta_value FROM
  wp_postmeta WHERE meta_key='_elementor_data'"` parses as JSON.
