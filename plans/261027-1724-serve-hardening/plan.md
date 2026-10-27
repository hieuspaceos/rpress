---
title: "serve-hardening"
description: "Đóng 4 lỗ hổng scale/security phát hiện trong audit 2026-10-27 + 2 hardening item đã biết"
status: pending
priority: P1
effort: "1-2 ngày"
tags: [security, scale, audit-remediation]
created: 2026-10-27
---

# serve-hardening

## Overview

Audit serve path (2026-10-27) phát hiện 4 lỗ hổng trên `rpress serve` khi
chạy trên VPS nhỏ (1 vCPU / 2 GB) dưới traffic thật, cộng 2 item hardening
đã biết trong code comments. Mọi fix đều local — không đụng kiến trúc
static-first.

Phát hiện từ audit:

| # | Lỗ hổng | Vị trí | Hậu quả |
|---|---------|--------|---------|
| 1 | `.md` on-demand render chạy trước `miss_rate` gate | `src/admin/mod.rs:487` | Bot bypass rate-limit, đốt CPU render + rác hóa cache |
| 2 | `argon2 verify` chạy inline trên async worker | `src/admin/auth.rs` login | Credential spray pin toàn runtime vào argon2 (~100ms/request) |
| 3 | File serve đọc hết vào `Vec<u8>` | `safe_read_under`, `uploads` | Media lớn (video) → vài request song song = OOM trên 2 GB |
| 4 | `publish_due` chạy mọi authed request | `src/admin/mod.rs:845` | Write-check transaction quay liên tục, thêm lock contention |
| 5 | Không single-flight render — N miss cùng slug render N lần | `public_page` | Stampede sau cache-bust; race window đã ghi nhận trong comment |
| 6 | Không global cap cho PHP bridge processes | `src/admin/bridge/` | 5+ editor mở 5 plugin khác nhau → OOM (mỗi bridge ~200-300 MB) |

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | Rate-limit che mọi on-demand render path (.md + html) | P0 |
| 2 | Argon2 không block tokio worker | P0 |
| 3 | File serve streaming — không đọc hết file vào RAM | P1 |
| 4 | Mỗi slug chỉ render 1 lần kể cả khi N request cùng miss | P1 |
| 5 | Tổng số PHP bridge process bị cap theo RAM budget | P1 |
| 6 | publish_due throttle — không write-check mỗi request | P2 |

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | [Start](./phase-01-start.md) | Pending |
| 2 | [Hotfixes: .md rate-limit + argon2 spawn_blocking](./phase-02-hotfixes-rate-limit-and-arg2.md) | Pending |
| 3 | [Single-flight miss render](./phase-03-single-flight-render.md) | Pending |
| 4 | [Global PHP bridge cap](./phase-04-bridge-global-cap.md) | Pending |
| 5 | [Streaming file serve](./phase-05-streaming-file-serve.md) | Pending |
| 6 | [publish_due throttle](./phase-06-publish-due-throttle.md) | Pending |

## Success Criteria

- [ ] `curl /<random>.md` spam bị 429 sau 60 req/phút/IP
- [ ] Login flood không làm public page latency tăng đáng kể
- [ ] Serve file 500 MB không alloc 500 MB (peak RSS tăng < 50 MB/file đang serve)
- [ ] 50 concurrent miss cùng 1 slug → đúng 1 render chạy
- [ ] Spawn bridge thứ N+1 khi đã đủ cap → evict bridge idle lâu nhất hoặc 503 có lý do
- [ ] `cargo test` + `cargo clippy` sạch; `./scripts/e2e-site.sh` smoke pass

<!-- slug: serve-hardening -->
