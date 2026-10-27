---
title: "Phase 4: Global PHP bridge process cap"
status: todo
---

# Phase 4: Global PHP bridge process cap

## Overview

`Bridges` map hiện chỉ giới hạn bằng TTL idle (10 phút) — không có cap
trên tổng process. Mỗi bridge = npx node + WP-WASM ~200–300 MB → trên
VPS 2 GB, ~5 spawn đồng thời là OOM. Cần cap tổng + evict under pressure.

## Key Insights

- `reap` (`bridge/mod.rs:196`) đã sweep mỗi 60s theo TTL — extend nó
  thành "reap theo TTL **hoặc** khi tổng > cap".
- Cap phải tính **toàn cục**, không per-site: `Bridges` là per-SiteCtx.
  **`bridge_for` hiện chỉ nhận một `&Bridges`** — không thấy các site
  khác → signature phải đổi để nhận global registry (ví dụ tham chiếu
  `state.sites` map hoặc một `GlobalBridges` mới gom mọi site map).
  Nhiều site × nhiều plugin mới là kịch bản OOM thật.
- Spawn path: `bridge_for` → `spawn`. Ba điểm enforce:
  a) **Reserve-slot**: inc counter *trước* khi spawn (giải phóng nếu
     spawn Err). Nếu không reserve, hai request cùng thấy count < cap
     sẽ double-spawn qua cap.
  b) **Trước spawn khi đầy**: count ≥ cap → evict bridge `last_used`
     xa nhất toàn cục (`try_lock` trên entry — locked = đang phục vụ,
     skip); nếu không evict được gì → Err "bridge capacity".
  c) Reaper định kỳ: ngoài TTL, đếm live entries thật trong mọi map
     (không tin counter) và evict-oldest cho tới cap — reconcile drift.
- `RESPAWN_COOLDOWN` 60s hiện per-key — evict-under-pressure không được
  tính là "boot failure": spawn mới sau evict khác key vẫn phải được.

## Architecture

- `BRIDGE_MAX` const (default ~4) + env `RPRESS_BRIDGE_MAX` override —
  theo convention env hiện có (`RPRESS_TRUSTED_PROXY`, `RPRESS_ASSET_ORIGINS`).
- Global registry: `bridge_for` cần thấy mọi site → truyền global view
  (sites map hoặc registry riêng) vào signature; per-site `Bridges`
  giữ nguyên là storage. Counter `Arc<AtomicUsize>` trên `App` chỉ là
  fast-path check — nguồn đúng là số entry live; reaper reconcile.
- Eviction pick: quét mọi site map, chọn `last_used` xa nhất —
  cần read `BridgeState.last_used` dưới `try_lock` (locked = in-flight,
  không evict), không blocking.
- Spawn blocked → degrade screen: `bridge_for` Err đã có path hiển thị
  lỗi ở caller (README mu-plugin comment "renders the degrade screen") —
  tái dùng, không thêm UI mới.

- Sửa: `src/admin/bridge/mod.rs` — `bridge_for`, `reap`, thêm
  `evict_oldest`, counter hooks
- Sửa: `src/admin/mod.rs` — `App` (counter field), reaper loop truyền
  sites + cap
- Đọc lại `site_serve.rs`/`site_bridge.rs` — multi-site boot path cũng
  đi qua `serve()` chung nên counter một nơi là đủ; verify.

## Implementation Steps

1. Thêm `BRIDGE_MAX` + env override; lưu vào `App` hoặc const-eval tại
   dùng.
2. `App.bridge_count: Arc<AtomicUsize>` — **reserve trước spawn**
   (fetch_add khi quyết định spawn; spawn Err → fetch_sub). dec trong
   `kill`/reap/evict. Child chết tự nhiên: dec khi entry bị
   drop/respawn.
3. `bridge_for` (signature mở rộng nhận global view): reserve → nếu
   count > cap → `evict_oldest` toàn cục (`try_lock`); evict không được
   → release slot + Err "bridge capacity" → caller degrade screen.
4. `reap`: sau TTL sweep, đếm live entries thật trong mọi map;
   evict-oldest tới cap; reconcile `bridge_count` = số đếm được.
5. `goodlayers` always-on: evict nó cũng được (spawn lại khi cần) —
   không ngoại lệ; ngoại lệ của nó là opt-in, không phải pinning.

## Todo

- [ ] `RPRESS_BRIDGE_MAX` + default
- [ ] Global counter inc/dec đúng mọi path
- [ ] `evict_oldest` cross-site
- [ ] Spawn-blocked error path → degrade screen
- [ ] Test: mock spawn (trait `PhpRuntime` đã tách) → spawn đến cap,
      spawn tiếp → oldest bị kill; mọi active → lỗi
- [ ] `cargo test` pass

## Success Criteria

- `RPRESS_BRIDGE_MAX=2`: spawn slug A, B → spawn C kills A (nếu A idle
  nhất); spawn D khi cả B,C active <TTL → lỗi rõ, không OOM.
- Counter không rò: sau vài chu kỳ spawn/kill, `bridge_count` khớp số
  entry thật trong các map.

## Risk Assessment

- Evict bridge editor đang dùng (last_used vừa cập nhật nhưng request
  trong-flight): `bridge_for` giữ `BridgeHandle` lock suốt request —
  evict phải skip entry đang locked. Thử `try_lock`; fail → skip.
- Counter drift nếu child chết mà không ai gọi kill: reconcile trong
  reaper — đếm lại entry thật mỗi sweep thay vì tin counter mù.

## Security Considerations

- Cap = resource bound — chính nó là security fix (OOM protection).
- Eviction không cho attacker ép spawn storm: cooldown per-key giữ
  nguyên; spawn-blocked path không spam npx.
