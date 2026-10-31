---
phase: 3
title: "seeded_post truthfulness + escaped-collapse fix"
status: done
priority: P1
effort: "1.5h"
dependencies: []
---

# Phase 3: `seeded_post` truthfulness + escaped-collapse fix

## Goal

`seeded_post` only ever claims a seed that actually landed; escaped
doubled-mount URLs collapse correctly.

## Files to Modify

- `src/admin/bridge/php_cgi.rs:754` — `seeded_post: post_id` becomes
  `seeded_post: if project_post returned true { post_id } else { None }`.
  Currently `let _ = project_post(...)` swallows the bool at :509.
  Capture it: `let seeded = if let Some(id)=post_id { project_post(...)?.then_some(id) } else { None };`
  …but `?` propagates errors — seed absent (`Ok(false)`) is NOT an
  error: `let seeded = match post_id { Some(id) => if project_post(rpress_db,&content,id)? { Some(id) } else { None }, None => None };`
- `src/admin/plugin_dispatch.rs:363` — `after.starts_with("\\\\/wp-")`
  expects literal `\\/`; the escaped mount emits `\/` → change to
  `after.starts_with("\\/wp-")` (one backslash in the string literal:
  matches `\/wp-`).

## Tasks & Steps

1. php_cgi spawn: capture `project_post` bool → `seeded_post`.
2. playground spawn: same (bool already exists at :144).
3. `reseed_post`: `let Some(db_file) = pull::locate(wp_db) else { return Ok(false) };`
   then conn+delete with `?` (existing `Context` usage stays).
4. Fix the `\\\\/wp-` literal; unit-test the collapse with a body
   containing `\/admin\/plugins\/elementor\/admin\/plugins\/elementor\/wp-admin\/x`
   → expect single mount.
5. Collapse-follower edge: the escaped branch whitelist omits `}`, `,`,
   `\\/` — a doubled `{mount_esc}{mount_esc}` directly before a JSON
   close/comma or another escaped slash is also an artifact. Add those
   followers if observed in proxied bodies.

## Verification

- `cargo test --lib` — new tests for both.
- Logically trace: absent post → `seeded_post=None` → next request
  triggers reseed path again (not skipped).
