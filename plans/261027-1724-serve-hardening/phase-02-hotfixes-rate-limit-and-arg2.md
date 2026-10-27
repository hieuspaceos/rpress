---
title: "Phase 2: Hotfixes — .md rate-limit + argon2 spawn_blocking"
status: todo
---

# Phase 2: Hotfixes — .md rate-limit + argon2 spawn_blocking

## Overview

Hai fix nhỏ, độc lập, rủi ro thấp — gộp một phase vì mỗi cái <30 dòng.
Ưu tiên P0: đây là 2 lỗ hổng bị exploit trực tiếp từ internet.

## Key Insights

- **Bug 1**: `public_page` (`src/admin/mod.rs:487`) xử lý `md_path.is_some()`
  (lookup DB + render markdown + `write_cache`) *trước* block rate-limit
  ở dòng ~519. `.md` trên slug không tồn tại vẫn chạy `md_for_slug` → conn mở,
  `post_markdown_id` query — không giới hạn.
- **Bug 2**: `login` (`src/admin/auth.rs`) gọi `crate_auth::verify_password`
  (argon2, ~50–100ms CPU blocking) trực tiếp trong async handler — cả đường
- **Bug 1**: `public_page` (`src/admin/mod.rs:487`) xử lý `md_path.is_some()`
  *trước* block rate-limit ở dòng ~519. Đường `.md` gồm `md_for_slug`
  (conn-open + `post_markdown_id` lookup — rẻ) và khi hit: render markdown
  + `write_cache` — cả hai đều không qua limiter. Đây là lỗ hổng nhỏ hơn
  "render không giới hạn" nhưng vẫn là bypass có write-back đắt.
- **Bug 2**: `login` (`src/admin/auth.rs`) gọi `crate_auth::verify_password`
  (argon2, ~50–100ms CPU blocking) trực tiếp trong async handler — cả đường
  "unknown email" lẫn đường verify thật. Đây là **hai callsite duy nhất**
  trong codebase (grep xác nhận 2026-10-28). Render path đã có precedent
  đúng: `spawn_blocking` ở `mod.rs:553` (comment H14).
- Sửa: `src/admin/auth.rs` — `login` handler
- Test: `tests/` (thêm vào file test hiện có phù hợp, ví dụ `hardening.rs`)

## Implementation Steps

### 2a. .md rate-limit

1. Di chuyển block `miss_rate` check (hiện ở ~dòng 519, ngay trước
   `lang::split_path`) lên **trước** nhánh `if md_path.is_some()`
### 2a. .md rate-limit — dời nhánh, không dời limiter

1. **Không** dời block `miss_rate` lên trước: làm vậy đặt limiter trước
   extension-check + `redirect_prefix`/asset-redirect gate → mỗi
   asset-miss 307 hợp lệ tốn 1 lượt 60/phút → site nhiều asset tự
   throttle. Thay vào đó: **dời toàn bộ nhánh `if md_path.is_some()`
   (dòng 487–503) xuống sau block rate-limit** (ngay trước
   `lang::split_path`). Limiter giữ nguyên vị trí.
2. Thứ tự cuối: cache-hit loop → charset/junk/pagination (đã trước) →
   extension asset gate → rate-limit → `.md` branch → normalization →
   render slug. `.md` canonical-rel gate (M22) nằm trong nhánh — đi
   theo nó xuống dưới limiter, không đổi nội dung.
3. Hệ quả mong muốn: `.md` miss giờ tốn lượt; `.md` hit cache
   (`{clean}/index.md` candidate trong loop dòng 443) vẫn thoát sớm —
   không qua limiter, đúng semantics cache-hit.
4. Edge giữ nguyên: `.md` canonical 301 vẫn hoạt động — nó chỉ chạy sau
   khi request đã qua limiter (một 301 được phát vẫn tính 1 lượt — fair).
### 2b. argon2 off worker

1. Trong `login`, bọc cả hai callsite `crate_auth::verify_password`
   (dummy hash nhánh unknown-email ở ~dòng 136 + hash thật ở ~dòng 141)
   trong `tokio::task::spawn_blocking`.
2. Giữ semantic: vẫn verify 1 lần mỗi request, không cache kết quả.
3. `spawn_blocking` cần owned data: clone `password` (đã là `String`) và
   `hash`/`dummy_hash` (`&'static str` — copy) vào closure; lấy bool ra
   bằng `.await.unwrap_or(false)` — join-fail phải fail closed (từ chối
   login), không panic-thành-pass.
4. Grep đã xác nhận chỉ có 2 callsite trong `login` — không có caller
   async nào khác của `verify_password` (app-passwords dùng HMAC, không
   argon2).

## Todo

- [ ] Nhánh `.md` nằm sau block `miss_rate` trong `public_page`
- [ ] `.md` request 429 sau ngưỡng; `.md` hit cache vẫn serve bình thường
- [ ] `verify_password` không còn callsite inline nào trong async fn
- [ ] Test: flood `.md` → có 429 (mở rộng `tests/hardening.rs`)
- [ ] `cargo test` pass

## Success Criteria

- Repro #1 từ phase-01 giờ trả 429 từ request thứ 61 trở đi.
- Repro #2: login flood song song với GET / → p95 trang cached không
  đổi đáng kể so với baseline không-flood.

## Risk Assessment

- Dời rate-limit lên trước canonical-rel gate của `.md`: một IP bị
  throttle vẫn nhận 301 đúng (gate nằm sau rate-limit — ok, 301 không
  phải tài nguyên đắt). Không đảo thứ tự gate-canonical.
- `spawn_blocking` cho login: login là rare path — thêm ~µs dispatch,
  không đáng kể.

## Security Considerations

- Fix 1 đóng đường render-free khỏi rate-limit — mục tiêu của nó.
- Fix 2 không làm giảm cost argon2 cho attacker (vẫn verify đầy đủ) —
  chỉ bảo vệ worker pool, giữ nguyên timing-defense của A14.
