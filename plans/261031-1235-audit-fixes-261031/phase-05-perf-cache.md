---
title: "Phase 5: Perf & cache"
status: todo
priority: P2
---

# Phase 5: Perf & cache

## Overview

Audit §3 H1–H4: render-path memory copies, global cache bust, per-render conn
in admin chrome, and no single-flight on render-miss. (H5 import RAM cap is
deferred — documented limitation, not a fix candidate in this pass. H6 SQL
layering moves to phase 7.)

## Requirements

- [x] H1 `site_data_cached` (`render/site_data.rs:23-36`): cache `Arc<JVal>` —
  `RwLock<HashMap<(String,String), Arc<JVal>>>`, clone the `Arc` not the value;
  `render_node` menu-tree deep clone (`render/nodes.rs:594-599`) — take `&` or
  share via `Arc` if signature allows; else leave with comment.
- [x] H2 `site_data_bust()` global flush (`generate.rs:469`, `site_data.rs:14`):
  bust by site key — call sites have `sc`/`site_url`; change signature to
  `site_data_bust(site_url: &str)` retaining lang entries for that site only.
  Cache-key collision `(site_url, lang)` across same-url sites: acceptable —
  document; adding site-identity to key = scope creep unless a real two-site-
  same-url deploy exists (it doesn't — serve keys by host/path).
- [x] H3 `kind_has_posts` (`admin/chrome.rs:184-198`): hoist — open one
  `Connection` per chrome render, pass to kinds loop (check fn signature;
  likely takes `&Path` per call — change to `&Connection`).
- [x] H4 single-flight render-miss (`serve.rs:202-215`): dedup concurrent
  misses per path — `DashMap<String, Shared<JoinHandle/OnceCell>>` or simpler:
  `Mutex<HashSet<path>>` + wait-poll. Prefer boring: `DashMap<PathBuf, Arc<tokio::sync::OnceCell>>` or reuse an existing crate dep already in Cargo.toml — check `dashmap`/`parking_lot` presence (parking_lot confirmed used in site_data.rs).

## Implementation Steps

1. `site_data.rs`: change map value to `Arc<JVal>`; update `site_data_cached` return + `site_data_bust(site_url)`; fix all callers (`generate.rs:469`, `render.rs`, `site_serve.rs` — grep `site_data_bust\|site_data_cached`).
2. `admin/chrome.rs`: refactor `kind_has_posts(&conn, kind)`; open conn once at caller.
3. `serve.rs`: single-flight around the miss-render block (202-215); on completion insert into cache path, second waiter then hits cache read — simplest correct shape: leader renders+writes cache, followers fall through to cache read after leader finishes.
4. Check Cargo.toml deps before adding dashmap.

## Todo

- [x] `Arc<JVal>` site_data cache
- [x] Per-site bust
- [x] `kind_has_posts` conn hoist
- [x] Single-flight miss render
- [x] `cargo test` green; cache-coherence tests still pass

## Success Criteria

- `cargo test` green (exposure/cache-coherence tests cover the bust paths).
- Concurrent `curl` ×20 on one uncached path → 1 render (verify via log/atomic counter).

## Risk

- `site_data_bust` signature change ripples to many callers — mechanical but wide.
- Single-flight must not leak entries on panic — use scope guard or remove entry in Drop/finally.
