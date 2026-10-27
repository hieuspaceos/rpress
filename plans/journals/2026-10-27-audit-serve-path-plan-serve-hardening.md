---
title: Audit serve path → plan serve-hardening
date: 2026-10-27
summary: Audit rpress serve/admin surface; phát hiện 4 lỗ hổng mới + 2 hardening item → plan 6 phase tại plans/261027-1724-serve-hardening
---

# Audit serve path → plan serve-hardening

## What happened

Audit serve path của rpress trên `src/admin/mod.rs`, `src/admin/auth.rs`,
`src/intake.rs`, `src/admin/bridge/` sau khi so sánh với twelve-dollar-server-challenge
(VPS $12, 1 vCPU / 2 GB, hàng chục nghìn concurrent users).

Phòng thủ đã kiểm chứng có sẵn: safe_read_under chống traversal+symlink,
charset allowlist public path, XFF chỉ tin trusted proxy, login rate-limit
+ constant-shape timing argon2, CSRF per-session ct_eq, write_cache atomic,
asset_redirect_target chống open-redirect, bridge spawn không qua shell,
pull.rs whitelist một chiều.

4 lỗ hổng mới:
1. `.md` on-demand render chạy TRƯỚC miss_rate gate (mod.rs:487 vs 519) —
   bot bypass rate-limit hoàn toàn.
2. argon2 verify inline trong async login handler — credential spray pin
   toàn tokio worker (~100ms CPU/request).
3. Mọi file serve đọc hết Vec<u8> — uploads 500 MB = OOM vài request.
4. publish_due chạy mọi authed request — write-check quay liên tục.

Cộng 2 item code đã tự ghi: không single-flight render (stampede sau
bust_cache) và không global cap cho PHP bridge processes (OOM khi nhiều
editor mở nhiều plugin).

## Decision

Plan 6 phase tại plans/261027-1724-serve-hardening:
P1 baseline repro → P2 hotfixes (.md rate-limit + argon2 spawn_blocking)
→ P3 single-flight render → P4 bridge global cap (counter + evict-oldest,
RPRESS_BRIDGE_MAX) → P5 streaming serve (safe_open_under, ngưỡng 8 MB)
→ P6 publish_due sang reaper loop 60s.

Chọn đơn giản có chủ đích: KHÔNG làm dependency-aware invalidation,
KHÔNG range requests, KHÔNG timeout cho render — ghi trong phase files.

## Next steps

`/ak:cook plans/261027-1724-serve-hardening/plan.md` — bắt đầu phase 1
repro baseline.

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
