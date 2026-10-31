---
title: "Phase 2: Serve-Path Performance"
status: todo
priority: P1
---

# Phase 2: Serve-Path Performance

## Overview

Remove all blocking I/O from the async executor on the public serve path,
release the global sessions mutex around SQLite work, and fix static-asset
serving semantics. Targets HIGH findings P1/P2 plus related MEDIUMs.

## File Ownership

- `src/admin/serve.rs` (primary)
- `src/admin/mod.rs` (sessions plumbing only — `get_sess`, `sessions` map)
- `src/admin/admin.rs` (only if intake limit moved to router in phase 1 needed adjustment — otherwise untouched)
- `src/db/mod.rs` (conn open; read-conn cache if pursued)

## Requirements

- [ ] P1: `public_root` (`serve.rs:48-58`) wraps `db::conn()` + `render_slug_by_id_public()` in `tokio::task::spawn_blocking`, mirroring `public_page` (serve.rs:289-296). Cold `/` must not block a tokio worker.
- [ ] P2: `get_sess` (`mod.rs:490-546`) narrows the `app.sessions.lock()` critical section to map ops only: lookup token in map; on miss, drop lock → `session_get` DB call → re-lock → insert. `session_set_csrf` and `session_touch` (DB writes) run outside the lock. Preserve semantics: concurrent rehydrate of same token must not double-insert (check-then-act under second lock).
- [ ] M-miss: move the whole miss-resolution block (`db::conn`, `lang::config`, `md_for_slug` — which opens a *second* conn at serve.rs:306 — `translation_canonical`) into the same `spawn_blocking` as the render (serve.rs:165-260). Share one `Connection` per miss where lifetimes allow (pass `&Connection` into `md_for_slug`-equivalent or refactor to take conn).
- [ ] M-sessions: periodic sweep of `app.sessions` + `sessions` table on the existing minute tick (same cadence owner as `publish_due`, serve.rs:542-564): evict expired in-memory entries; `DELETE FROM sessions WHERE expires_at < now` bounded.
- [ ] M-assets: `wp_asset`/`uploads`/`front_css` reuse `file_response`'s ETag (serve.rs:715-794); add `Cache-Control: max-age=86400` for fingerprinted/immutable vendored assets (`?ver=` or content-hashed names), `no-cache` for mutable ones. Match the security headers added in phase 1.
- [ ] M-range: replace `fs::read` whole-file in `safe_read_under` consumers for **uploads/wp_asset** with streamed responses supporting `Range` (`tower_http::services::fs::ServeFile` or `tokio::fs::File` + `ReaderStream` + `Content-Range`). Keep `fs::read` for cache-hit HTML (small, hot). Preserve `symlink_metadata` + canonical-prefix confinement — the streamer must resolve through the same `safe_read_under` gate.
- [ ] L-writecache: fold `write_cache`'s `create_dir_all`+`write`+`rename` into the same `spawn_blocking` closure as the render (serve.rs:384-400).
- [ ] L-singleflight (optional here, may defer): `DashMap<slug, Shared<JoinHandle>>` dedup for concurrent same-slug misses (serve.rs:289-330). Acceptable to document as deferred if it complicates the response type.
- [ ] L-conn (optional): per-`SiteCtx` read-connection cache or a small `r2d2` pool if the 6-PRAGMA open (~100-300µs) shows up in the smoke profile; otherwise document decision not to pool (rusqlite `Connection: !Sync` — needs parking_lot Mutex or pool).

## Implementation Steps

1. Refactor `public_root` — extract shared `render_miss(sc, slug-ish)` helper used by both `public_root` and `public_page` so the spawn_blocking shape is one code path.
2. `get_sess` restructure — write the new flow explicitly: `lock().get(token)` → if Some valid → touch-outside-lock → return; if None → `session_get` (no lock) → `lock().entry(token).or_insert(...)` → handle race where another task already inserted (use existing entry).
3. Minute tick: locate the existing tick/publish_due call site; add `sweep_sessions(&app)` there + SQL `DELETE FROM sessions WHERE expires_at < ?` (new `sitefx`/`schema` sql file per layering rule).
4. Asset responses: extend `file_response` usage or add `asset_response` helper producing ETag + Cache-Control + nosniff (+ phase-1 svg rule).
5. Range: evaluate `tower_http::services::fs` first (least code); verify it composes with the canonical-prefix check (pre-resolve path, then ServeFile on the resolved file — never pass user path to ServeFile directly).
6. Verify: `wrk`/`ab` or a bash loop hammering `/`, a missing slug, and a big upload before/after; record p50.

## Success Criteria

- Cold `/` and cold `/<slug>` render inside `spawn_blocking` — provable by a tokio-console or a test asserting handler doesn't block (or simply code review + `cargo test`).
- Concurrent admin requests no longer serialize on session DB I/O: two parallel `get_sess` calls where one session needs rehydrate don't block the other (unit test with a slow/faulty second conn is hard — at minimum lock-scope review + test that rehydrate-once-per-token holds).
- `curl -H "Range: bytes=0-1023" /uploads/big.mp4` → `206` + `Content-Range`.
- Second `GET` of a vendored asset with `If-None-Match` → `304`.
- `visual_smoke.sh` + `cargo test` green.

## Risk Assessment

- Lock-scope refactor of `get_sess` is the security-sensitive change: a botched check-then-act can double-mint CSRF or resurrect an expired session. Mitigation: keep the existing expiry checks inside the second lock; add test for expired-token path.
- ServeFile/ServeDir bypasses our charset/mime gates if misapplied — rule: resolve + gate with `safe_read_under` FIRST, stream the resolved canonical path only.
- Range support on uploads could leak metadata via `If-Range`/`multipart` — stick to single-range support (tower_http does this correctly already).
