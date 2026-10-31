---
title: "Phase 1: Stored XSS fixes"
status: todo
priority: P1
---

# Phase 1: Stored XSS fixes

## Overview

Close the two confirmed stored-XSS chains (audit §1.1, §1.2) and sweep for
same-class sites. Both give editor/attacker → admin CSRF-token theft because
admin pages embed `s.csrf` in the DOM (`admin/posts.rs:1435`).

## Root causes

- `esc()` (`src/html_decode.rs:43-47`) escapes `& < > "` only — **not `'`**.
- `comment_html()` (`src/render_post.rs:643-647`) puts `esc(&c.author)` into
  single-quoted `alt='…'` — `'` in a comment name breaks the attribute. The
  payload persists in the rendered page **and the write-back cache**.
- `seo_inner()` (`src/admin/rankmath.rs:47-55`) interpolates `a.keyword` raw
  into HTML; `posts.rs:560-564` assigns the fetch response to `td.innerHTML`.
  `rank_math_focus_keyword` is editor-controllable (`db/posts.rs:903`).

## Requirements

- [x] `comment_html` attribute context uses `esc_attr` (or double-quoted attr + `esc`)
- [x] `seo_inner` escapes `a.keyword` with `esc()`
- [x] Global sweep: grep `='{` and `alt='` / `title='` / `value='` patterns in `src/`; every single-quoted attribute interpolation uses `esc_attr`
- [x] Audit every `innerHTML =` assignment in `src/admin/` against server-escaped content; `posts.rs:1206` and `rankmath.rs:542` already replace `<` client-side — verify, don't duplicate escape
- [x] Intake: reject `"` `'` in `wc_name` at `intake.rs` submit_comment (defense in depth, alongside output escaping)

## Implementation Steps

1. `src/render_post.rs`: switch `alt='{author}'` to `alt="{esc}"` **and** `src='…{grav}…'` audit — gravatar hash is hex-only so safe; apply `esc_attr` where single quotes remain.
2. `src/admin/rankmath.rs`: `esc(&a.keyword)` (import existing shared `esc`).
3. `grep -rn "='{" src/ | grep -v esc_attr` — fix every hit.
4. `grep -rn "innerHTML" src/admin/` — for each, confirm the fragment is server-escaped or the JS escapes; document verdict per site.
5. `src/intake.rs` `submit_comment`: reject names containing `'` or `"` (or strip → `esc` handles rest anyway; rejecting is simpler signal).

## Related files

- Modify: `src/render_post.rs`, `src/admin/rankmath.rs`, `src/intake.rs`, any `='{` sweep hits
- Reference: `src/html_decode.rs` (`esc`, `esc_attr`), `docs/audit-261031.md` §1

## Todo

- [x] Fix comment `alt` attribute
- [x] Fix `seo_inner` keyword escaping
- [x] `='{` sweep complete, all sites fixed or verified
- [x] `innerHTML` audit documented (list each site + verdict)
- [x] Intake name-quote rejection

## Success Criteria

- Comment named `x' onerror='alert(1)` renders inert in generated HTML and in cached output.
- Focus keyword `<img src=x onerror=alert(1)>` shows as literal text in /admin/posts SEO cell.
- `cargo test` green; add a unit test asserting `comment_html` output escapes `'`.

## Risk

- `alt` quote change alters golden-render fixtures — regenerate/update fixtures if tests diff HTML.
