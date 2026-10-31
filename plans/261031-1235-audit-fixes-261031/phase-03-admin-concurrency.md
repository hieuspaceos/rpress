---
title: "Phase 3: Admin concurrency — spawn_blocking"
status: todo
priority: P1
---

# Phase 3: Admin concurrency — spawn_blocking

## Overview

Audit §2.1 (C1): the public path offloads SQLite work correctly
(`serve.rs:43-47, 256-262`) but `/admin/*` does not — rusqlite calls and argon2
run inline on the async runtime. One slow query stalls every concurrent
request including public renders.

## Confirmed gaps

- `admin/posts.rs:580-591` `trash_post` — inline `crud::set_post_status`
  (bulk_posts at 613-638 does `spawn_blocking` for the same operation).
- `admin/posts.rs:48-56` `section`, `1585-1603` `save_post` — inline.
- `admin/comments.rs`, `users.rs`, `menus.rs`, `tax.rs`, `widgets.rs`,
  `settings.rs` — **zero** `spawn_blocking` (~4.7k LOC CRUD).
- `admin/auth.rs:139-158` argon2 (~50-100ms CPU) inline; `dummy_hash()` doubles it.
- `admin/serve.rs:163` `db::conn` + `lang::config` inline in `public_page` before miss-path.
- `admin/mod.rs:384-395` minute tick: `serve::publish_due`, `sweep_sessions`,
  `bridge::reap` (kill/wait) on the async task.
- `SiteCtx::settings()` (`mod.rs:97-116`) opens a conn sync — called from
  `site_url`/`asset_origin`/`allowed_origins` inside async paths.

## Requirements

- [x] Every `db::`/`crud::`/`udb::` call in `src/admin/` runs inside
  `tokio::task::spawn_blocking` (or a small `block(|| …)` helper wrapping it —
  check if `serve.rs` already has such a helper to reuse; do not invent a
  second convention)
- [x] argon2 verify + `dummy_hash` wrapped (CPU-bound, same rule)
- [x] Minute tick: wrap each unit of blocking work, or spawn a blocking task
  per tick — keep ordering publish_due → sweep → reap
- [x] `SiteCtx::settings()` — either call sites move to spawn_blocking, or
  settings get cached in `App`/`SiteCtx` behind a lock (assess which is
  boring-er; a once-per-request cached settings read is fine)
- [x] Extend `scripts/check-layers.sh` with a grep rule flagging
  `crud::`/`udb::`/`db::` outside `spawn_blocking` in `src/admin/*.rs`
  (with documented allowlist mechanism consistent with existing gate-allows)

## Implementation Steps

1. Inventory: `grep -rn "crud::\|udb::\|db::conn\|udb::" src/admin/*.rs` minus existing `spawn_blocking` — produce the full call list first.
2. Check how public path wraps (read `serve.rs:43-47`); extract/reuse helper if one exists (`block`/`db_blocking`); otherwise define `pub(crate) async fn block<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T { spawn_blocking(f).await.expect("blocking task panicked") }` in `admin/mod.rs` — panic propagation is correct (a panicked DB closure = bug, 500 is right).
3. Convert file-by-file; keep closures `move`ing owned `PathBuf`/`String` in and `Result` out — do not change handler signatures.
4. `serve.rs:163` inline conn in `public_page` → move into the existing miss-path spawn or wrap separately (whichever touches less code).
5. Minute tick in `mod.rs:384-395` → wrap whole body in one `spawn_blocking` (sequential semantics preserved).
6. Add check-layers rule; run it — must be clean before merge.

## Related files

- Modify: `src/admin/{posts,comments,users,menus,tax,widgets,settings,auth,serve,mod}.rs`, `scripts/check-layers.sh`
- Reference: `src/admin/serve.rs` public-path offload pattern

## Todo

- [x] Call inventory produced
- [x] Helper decided/created
- [x] All admin files converted
- [x] auth.rs argon2 wrapped
- [x] Minute tick wrapped
- [x] `SiteCtx::settings()` resolved
- [x] check-layers rule added and passes

## Success Criteria

- `grep -rn "crud::\|udb::" src/admin/*.rs | grep -v spawn_blocking` returns only allowlisted sites.
- Under `cargo test` + a manual `while sleep` loop hitting admin during a bulk op, public page latency stays flat (qualitative check acceptable).
- `cargo test` green.

## Risk

- 4.7k LOC touch — high churn, low logic change. `Response` types are `Send`;
  closures must not capture `&Connection` across the boundary (rusqlite
  `Connection` is not `Sync` — open inside the closure, which is already the
  pattern since handlers take `db: &Path`).
- `SiteCtx` may be `!Send`/`!Sync` — check before wrapping its users.
