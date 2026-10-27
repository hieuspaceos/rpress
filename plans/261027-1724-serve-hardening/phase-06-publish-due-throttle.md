---
title: "Phase 6: publish_due throttle"
status: todo
---

# Phase 6: publish_due throttle

## Overview

`get_sess` gọi `publish_due(sc)` trên **mọi** authed request
(`mod.rs:845`) — mỗi admin request phát một write-check transaction để
xem có post nào đến hạn schedule không. Nhiều editor + admin traffic cao
→ transaction quay liên tục chỉ để trả "không có gì".
- `publish_due` là WP `wp_schedule_publish` equivalent — semantic cần giữ:
  post scheduled đến hạn phải tự publish kể cả khi không ai đụng admin.
- Shape thật (đọc code 2026-10-28): `mod.rs:915 fn publish_due(sc)` gọi
  `crud::publish_due(&sc.db) -> Vec<(id, slug, lang, kind)>` — một
  transaction SELECT+UPDATE (blocking sync, `db/posts.rs:1344`); nếu có
  flip, gọi `admin::regen_posts` — regen đã chạy background sẵn.
- Hai cách:
  a) **Throttle in-memory**: `AtomicU64` last-check per site — vẫn để
     blocking call trong request path khi hết hạn.
  b) **Background loop**: đưa vào reaper loop 60s có sẵn trong `serve()`
     (đang sweep bridge TTL) — ra khỏi request path hoàn toàn.
- Chọn **(b)**: blocking DB call không còn trên async worker; tần suất
  bounded 60s. Lợi ích phụ lớn: hiện tại scheduled post **chỉ flip khi
  có admin request** — site không ai login thì post không bao giờ
  publish. Sang loop: publish đều đặn không phụ thuộc ai đăng nhập.
- Chọn **(b)**: bỏ khỏi request path sạch sẽ, tần suất rõ ràng (60s),
  code tồn tại sẵn (cùng loop với bridge reap). Giữ một lần check ở
  login/boot nếu muốn freshness ngay sau mở phiên — optional, không cần.

## Related Code Files

- Sửa: `src/admin/mod.rs` — `get_sess` (bỏ `publish_due` call), reaper
  loop trong `serve` (thêm `publish_due` mỗi site)
- `src/db/posts.rs:1344` — `crud::publish_due` (đọc, không sửa)

## Implementation Steps
1. `mod.rs:915 publish_due(sc)` là sync wrapper: `crud::publish_due`
   (transaction) + `admin::regen_posts` khi có flip. Trong reaper task
   (async), bọc **toàn bộ wrapper** vào `spawn_blocking` — transaction
   blocking không được nằm trên worker dù regen đã async.
2. Bỏ call trong `get_sess` (`mod.rs:845`) + comment đã chuyển sang loop.
3. Trong reaper task (`serve`, sleep 60s): thêm per-site
   `spawn_blocking(move || publish_due(&sc))`. Loop đã iterate
   `sites.values()` — per-site đúng.
4. Schedule granularity: post đến hạn publish trong ≤60s, **kể cả khi
   không có request nào** — tốt hơn hiện tại (chỉ flip khi có admin
   traffic). Document trong comment.
5. Per-site isolation: một site Err không làm chết sweep — giữ pattern
   `let Ok(..) = .. else return` mà wrapper hiện dùng.
6. Edge: multi-site — loop đã iterate `sites.values()`; `publish_due`
   per-site đúng.

## Todo

- [ ] `publish_due` ra khỏi `get_sess`
- [ ] Chạy trong reaper loop mỗi 60s, mọi site
- [ ] `spawn_blocking` nếu publish_due blocking
- [ ] Test: schedule post quá khứ → sau ≤1 sweep nó publish (nếu có test
      harness cho publish_due sẵn thì mở rộng; không thì smoke thủ công)
- [ ] `cargo test` pass

## Success Criteria

- Admin request không còn phát write-check transaction (đếm bằng log
  hoặc trace đơn giản).
- Post scheduled vẫn tự publish trong ≤ ~60s sau hạn, kể cả khi không
  có request nào.

## Risk Assessment

- Mất "freshness" per-request: post đến hạn có thể trễ tới 60s — chấp
  nhận và document (WP cron cũng không chính xác hơn; nó chỉ check khi
  có traffic — của ta còn đều hơn).
- Reaper loop fail một site không được chết cả loop: bọc per-site trong
  error handling (hiện bridge reap đã isolate per map — giữ pattern).

## Security Considerations

- Không đổi auth/authz: chỉ dời khi nào write-check chạy.
- `publish_due` chạy ngoài request → không còn phụ thuộc ai đăng nhập;
  giảm luôn một write-lock contention trên DB khi nhiều editor.
