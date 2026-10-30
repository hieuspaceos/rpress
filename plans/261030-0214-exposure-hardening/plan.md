---
title: "exposure-hardening (audit 2026-10-30)"
description: "Vá 2 lỗ hổng unauthenticated info-disclosure + hardening DoS/CRLF + native TOTP 2FA trong rpress core. Không đụng plugin/theme — chuyện của site gốc."
status: pending
effort: 1.5d
tags: [security, hardening]
created: 2026-10-30
---

# exposure-hardening

## Overview

Security-reviewer audit (agent NecessaryParrotfish, 2026-10-30) đọc toàn bộ
live surface của `rpress serve`. Kết luận: **không có** unauthenticated
RCE / SQLi / path traversal (3-lớp defense đều đứng, SQL 100% parameterized,
CSRF + session coverage đầy đủ). Nhưng tìm thấy 5 điểm còn hở — plan này vá
tất cả, chỉ trong `rpress/src`, không chạm vendored plugin/theme files.

Residual risk hiện tại (ước tính, internet-facing, 1 năm):
takeover unauth <2% | lộ source plugin ~30-50% | DoS có chủ đích ~40-60% nếu
bị nhắm | brute-force admin <1% (password >12 ký tự).

**Audit note (2026-10-30, code-reviewer):** plan đã qua 1 vòng adversarial
review. Blocking issues đã vá vào phase text: ASSET_EXT thiếu eot/otf/cur
(phá icon fonts), evict-stalest gây self-eviction trên login_hits (phải
subnet-key), `%2e%2e%2f` traversal vào bridge docroot, và App-Password
bypass TOTP (phải revoke rows + chặn add khi enabled).

## Goals

| # | Goal | Priority |
|---|------|----------|
| 1 | Đóng 2 unauthenticated disclosure endpoints (gdlr asset no-filter, front-css uploads read) | P1 |
| 2 | DoS resilience: rate-map LRU thay deny-new; fs I/O off event loop | P2 |
| 3 | Bridge request-line sanitize (defense-in-depth) | P3 |
| 4 | Native TOTP 2FA cho admin login — chống password-lộ, giảm <1% → ~0.1% | P1 |
## Phases

| # | Phase | Status |
| 1 | [Phase 1: Unauthenticated asset disclosure](./phase-01-asset-disclosure.md) | Pending |
| 2 | [Phase 2: DoS resilience](./phase-02-dos-resilience.md) | Pending |
| 3 | [Phase 3: Bridge path sanitization](./phase-03-bridge-path-sanitization.md) | Pending |
| 4 | [Phase 4: Admin 2FA (native TOTP)](./phase-04-admin-2fa-totp.md) | Pending |

## Non-goals

- Plugin/theme vulns (Tour Master SQLi, v.v.) — vá trên WP origin, không phải rpress.
- Vendored file content review (credentials trong plugin bundles) — review riêng khi vendoring.

## Success Criteria

- [ ] `GET /admin/wp/gdlr/<file>` với ext ngoài allowlist → 404 (unauth).
- [ ] `GET /admin/front-css/<non-css>` → 404 hoặc từ chối; có `nosniff`.
- [ ] Rate-map đầy: IP mới không bị khóa; evict key cũ nhất.
- [ ] Public file serve path không block reactor (tokio::fs hoặc spawn_blocking cho file lớn).
- [ ] `rest` chứa `\r\n`/control bytes → rejected trước khi format request line.
- [ ] User bật `totp_enabled` → login yêu cầu mã TOTP (QR setup bằng `qrcode` crate, SVG inline) hoặc backup code one-time; sai → reject + rate-limit; user không bật → flow cũ.

<!-- slug: exposure-hardening -->