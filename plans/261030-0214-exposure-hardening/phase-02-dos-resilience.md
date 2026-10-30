---
title: "Phase 2: DoS resilience"
status: todo
---

# Phase 2: DoS resilience

## Overview

Hai amplifier availability (severity low, nhưng dễ bị nhắm bởi botnet):
(1) rate-map deny-new-on-full khóa cả internet lẫn kẻ tấn công,
(2) `fs::read`/`stat` blocking trên tokio worker cho mọi public request.

## Requirements

- [ ] `intake_rate`/`miss_rate`: evict key cũ nhất khi đầy (bounded-LRU) —
      benign cho victim; attacker vẫn bị giới hạn per-key.
- [ ] **`login_hits` khác biệt (audit fix):** evict-stalest trần cho login
      cho phép **self-eviction** — >10k IP spray reset chính counter của
      attacker → 20/min thành uncapped. Bắt buộc một trong hai:
      (i) key theo /24 (IPv4) + /64 (IPv6) — spray cùng subnet đếm chung;
      hoặc (ii) global budget: tổng pushes/min mọi key ≤ ~200 (spray tự
      throttle). Chọn (i)+(ii) nếu rẻ; minimum (i).
- [ ] `safe_read_under` async: `spawn_blocking` bọn toàn bộ metadata+read
      trên public path (public_page candidates, uploads, wp_asset, front_css,
      not_found). Sync stat trước để quyết offload không cần thiết — offload
      luôn trên nhánh file; giữ bản sync cho call site đã trong
      spawn_blocking (bridge proxy).
- [ ] Hành vi deny tổng thể không đổi: 20/min login, miss-render, intake.

## Implementation Steps

1. `src/intake.rs` `Rate::allow` (~15-38): đầy → remove key có last-Instant
   cũ nhất rồi insert. Giữ `RATE_MAX_KEYS`.
2. `src/admin/auth.rs` `login_hits` (~118-138): chuyển key IP → subnet
   (`/24` v4, `/64` v6 — helper `login_key(ip)`); kèm evict-stalest khi đầy.
   Response text không đổi ("Too many attempts").
3. `src/admin/mod.rs`: `async fn read_out_async`/`safe_read_under_async`
   (spawn_blocking quanh nguyên hàm); public handlers đổi sang `.await`.
4. Benchmark: `ab -n 500 -c 20 /` song song `ab` file upload lớn — p99 không
   tệ hơn trước.

## Todo

- [ ] Rate evict-stalest cho intake/miss maps
- [ ] login_hits subnet-keyed + evict (hoặc global budget)
- [ ] Async file reads trên public path
- [ ] Re-bench `ab` mixed load; e2e `check` pass

## Success Criteria

- Spray >10k unique IPs (cùng subnet khác nhau): IP thật vẫn login được
  **và** attacker vẫn chịu 20/min theo /24 — không reset.
- 20 song-song GET file lớn → request `/` nhỏ vẫn trong p99 cũ.

## Risk Assessment

- Subnet keying: botnet /64 IPv6 khác prefix vẫn né được phần nào — kèm
  global budget nếu muốn cứng; ghi trong code.
- spawn_blocking mỗi file: pool 512 + TCP backlog bound — chấp nhận.
