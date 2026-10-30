---
title: "Phase 6: Tail perf, lifecycle prune, spawn_blocking"
status: todo
---

# Phase 6: Tail perf, lifecycle prune, spawn_blocking

## Overview

Batch of smaller wins and data-lifecycle hygiene — all low-risk, any subset
shippable.

## Requirements / Steps

- [ ] **spawn_blocking admin GETs**: wrap DB+render bodies of `dashboard`,
      `list_admin` (posts.rs:20-67), `edit_post` in `tokio::task::spawn_blocking`
      (pattern at admin.rs:1104) — 20-40ms blocking pages stall the event
      loop for concurrent public traffic (tail latency at 7200 req/s).
- [ ] **Static serving**: canonicalize `www`/`out` roots once into SiteCtx
      (currently `canonicalize` + `symlink_metadata` per request);
      stream files via `tower_http::services::fs::ServeDir`/`tokio::fs`
      instead of `fs::read` into `Vec<u8>` (multi-MB images buffer in RAM);
      add ETag/Last-Modified conditional GET → 304 for repeat hits.
- [ ] **wp_asset/htmx vendored reads**: cache file bytes (`OnceCell`/LRU)
      or send `immutable` + ETag (admin.rs:4-15, mod.rs:1573-1640).
- [ ] **Revisions cap**: keep latest N (50) per post — prune oldest in
      `purge_duties`; `body_json` snapshots are the biggest DB growth
      vector (hundreds of KB per save on builder pages).
- [ ] **Sessions GC**: `DELETE FROM sessions WHERE expires_at <
      unixepoch()` in `purge_duties`.
- [ ] **Trash/spam submissions purge**: submissions/comments trash rows get
      a scheduled prune (currently unbounded).
- [ ] **Sessions map**: `Arc<Sess>` instead of per-request clone; minor.

## Todo

- [ ] admin GETs off the reactor
- [ ] ServeDir/conditional GET for cache-hit path
- [ ] vendored asset caching
- [ ] revisions/sessions/submissions prune in purge_duties
- [ ] rebuild; re-benchmark `/`, `/admin`, save burst; e2e `check`

## Success Criteria

- p99 public latency under mixed admin+public load improves.
- RSS stable across repeated admin sessions (no file-byte churn).
- DB growth rate for 100 saves < pre-change.

## Risk Assessment

- ServeDir path-security: keep existing `safe_read_under` symlink checks or
  replicate equivalent; do not regress on traversal.
