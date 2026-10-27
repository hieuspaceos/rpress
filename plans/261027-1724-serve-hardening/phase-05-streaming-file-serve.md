---
title: "Phase 5: Streaming file serve"
status: todo
---

# Phase 5: Streaming file serve

## Overview

Mọi file serve (`safe_read_under` → `fs::read` → `Vec<u8>`) đọc hết vào
RAM. Một video 500 MB trong uploads = 500 MB alloc mỗi request — vài
request đồng thời OOM con 2 GB. Chuyển sang streaming body.

## Key Insights

- `safe_read_under` trả `Option<Vec<u8>>` — dùng ở nhiều nơi: `read_out`
  (cache html — file NHỎ, vài chục KB), `uploads` (media — file LỚN),
  `wp_asset`, `front_css`, `not_found` (404.html).
- Không cần stream hết mọi path: HTML cache files < ~200 KB, fs::read
  rẻ. Đích thật là **uploads + bất cứ file > ngưỡng**.
- Hai lựa chọn:
  a) Stream toàn bộ: `tokio::fs::File` + `Body::from_stream` /
     `StreamReader` — phức tạp hơn vì `safe_read_under` phải chuyển sang
     async (canonicalize + symlink check vẫn sync được, chỉ read là async).
  b) **Ngưỡng**: < 8 MB → Vec như cũ; ≥ 8 MB → stream. Đơn giản, giữ
     signature sync cho phần lớn callsites. Uploads (file to) gần như
     luôn đi nhánh stream.
- Chọn (b): bề mặt thay đổi nhỏ nhất; thêm `safe_open_under` trả
  `std::fs::File` cùng invariant bảo mật (canonicalize, reject symlink,
  trong root) — symlink check phải trên `symlink_metadata` của path mở,
  giống `safe_read_under` hiện tại.
- Cargo.toml: **`tokio-util` chưa có trong deps** (grep xác nhận
  2026-10-28) — cần thêm `tokio-util = { version = "0.7",
  features = ["io"] }` cho `ReaderStream`. `axum::Body::from_stream`
  nhận TryStream — `ReaderStream<tokio::fs::File>` khớp trực tiếp.
- Range requests: video seekable là nice-to-have; **không** làm ở phase
  này (scope creep) — note lại.

## Related Code Files

- Sửa: `src/admin/mod.rs` — `safe_read_under` (giữ), thêm
  `safe_open_under`, `uploads` handler, có thể `read_out` cho file lớn
- Cargo.toml: cần `tokio` fs + `tokio-util` (io ReaderStream) — check
  dependency hiện có; `tokio-util` có thể chưa có → thêm feature `io`.

## Implementation Steps

1. `safe_open_under(root, rel) -> Option<(std::fs::File, Metadata)>`:
   cùng gate với `safe_read_under` (reject `..`, symlink, ngoài root).
   Trả File + metadata để caller đọc len.
2. `uploads`: metadata.len() ≥ 8 MB → `tokio::fs::File::from_std` +
   `ReaderStream` → `Body::from_stream`; giữ mime/attachment headers,
   thêm `Content-Length`. < 8 MB → path Vec cũ (đổi `fs::read` →
   `File::read` cho đồng nhất, hoặc giữ safe_read_under).
3. `read_out`/`public_page`: cache html nhỏ — giữ Vec; **nhưng** file
   `wp-content/*` từ `www` có thể lớn (plugin asset zip?) → áp ngưỡng
   tương tự ở `read_out` nếu đơn giản, hoặc chỉ uploads trong phase này.
   Quyết: chỉ uploads + một helper chung; các caller khác migrate nếu
   rẻ.
4. `wp_asset`/`front_css`/`not_found`: file nhỏ — giữ Vec.
5. Test: đặt file ~20 MB vào uploads root của test site; request nó;
   assert 200 + Content-Length + body khớp; RSS process không nhảy
   ~20 MB (đo bằng /proc hoặc bỏ qua, assert functional trước).

## Todo

- [ ] `safe_open_under` cùng security contract
- [ ] `uploads` stream ≥ 8 MB với Content-Length
- [ ] `tokio-util` dependency (feature `io`) nếu chưa có
- [ ] Test serve file lớn + regression mime/attachment
- [ ] `cargo test` pass

## Success Criteria

- Repro #3 phase-01: serve file 500 MB → request hoàn tất, RSS tăng
  < 50 MB (đo được trên macOS qua `ps` trong lúc curl).
- Không regression: mime types, nosniff, attachment disposition, 404
  cho path ngoài root/symlink.

## Risk Assessment

- `from_std` cần file trên thread runtime — mở sync (`safe_open_under`
  dùng std::fs) rồi convert; blocking open ~µs, chấp nhận (hoặc
  spawn_blocking nếu muốn sạch — ghi chú trong code).
- Stream không có built-in timeout: client rớt kết nối → axum drop body,
  reader dừng — ok.
- Symlink check trước open có TOCTOU nhỏ (attacker swap file sau check):
  thuyết phục — `www/` chỉ ghi bởi admin/deploy, không phải request path;
  giữ comment ghi nhận.

## Security Considerations

- Giữ nguyên toàn bộ gate của `safe_read_under` trong `safe_open_under`:
  `..`, leading `/`, backslash, symlink, canonicalize-in-root.
- Content-Disposition attachment cho extension nguy hiểm vẫn áp trên
  stream path — test regression bắt buộc.
