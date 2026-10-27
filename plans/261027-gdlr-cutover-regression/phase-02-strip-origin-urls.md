---
phase: 2
title: "Strip captured origin URLs from gdlr defs JSON"
status: done
priority: P2
effort: "1h"
---

# Phase 2: Strip origin URLs from defs JSON

## Problem

`templates/admin/wp/gdlr/elements.json` (136 hits) and `wrappers.json` (3)
were dumped from a live bridge and embed `http://127.0.0.1:64123/wp-content/…`
absolute URLs. `boot.rs:1263-1264` emits
`{GDLR_URL}/{thumb}` for non-`/` values → `/admin/wp/gdlr/http://127.0.0.1:64123/…`
(broken image). Any 'image'/'thumbnail' value passed through to preview
markup leaks the dead host.

## Change

Two layers, both cheap:

1. **Normalize the JSON files** (source of truth): strip `http://127.0.0.1:64123`
   → path-only (`/wp-content/plugins/goodlayers-core/…` becomes
   `/plugins/goodlayers-core/…`? — NO: verify what shape `defs()` consumers
   expect). Check `boot.rs` thumb construction + `page_builder_var`/`icons`
   consumers, then rewrite values so emitted URLs resolve under
   `/admin/wp/gdlr/` or are absolute-site paths, whichever matches the old
   Rust-port output (`git show 413906b^:src/admin/gdlr_pb.rs` thumbnails).

2. **Defense in depth in `defs()`** (`boot.rs`): after `serde_json::from_str`,
   walk the defs and rewrite any string starting with `http://`/`https://`
   that contains `/wp-content/` → keep only the `/wp-content/…` tail mapped
   onto `GDLR_URL` prefix where the old port did so. Small recursive helper,
   ~20 lines, comments noting why (JSON is a dump artifact, not hand-authored).

## Verify

```bash
grep -rc 'http://' templates/admin/wp/gdlr/*.json   # expect 0
cargo test --lib
# live: open element picker on aucoeur-test edit screen; thumbnails <img>
# resolve 200 (check via browser network or curl the emitted src)
```

## Risks

- The `127.0.0.1:64123` port is the *old* playground instance — these are
  dead regardless of correctness; normalization is the only right answer.
- If a thumbnail legitimately points at an external CDN URL (grep first —
  likely none), the rewrite must only strip hosts matching `/wp-content/`.
