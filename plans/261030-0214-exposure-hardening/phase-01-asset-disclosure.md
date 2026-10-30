---
title: "Phase 1: Unauthenticated asset disclosure"
status: todo
---

# Phase 1: Unauthenticated asset disclosure

## Overview

2 route public đọc file không giới hạn — severity medium + low, fix rẻ nhất
và giảm risk lớn nhất (~30-50% → ~0 cho scenario lộ source/config).

## Requirements

- [ ] `/admin/wp/{*path}` nhánh `gdlr/` áp cùng `ASSET_EXT` allowlist như nhánh `<slug>/`.
- [ ] `/admin/front-css/{*path}` chỉ serve `.css` + `X-Content-Type-Options: nosniff`, và (tuỳ chọn) giới lookups vào twin `*.scoped.css` + templates/front-css, không toàn uploads.
- [ ] `/admin/wp/{*path}` nhánh `gdlr/` áp allowlist **mở rộng** — audit
      chứng minh ASSET_EXT cũ thiếu `eot/otf/cur/map` (fontawesome + ilightbox
      trong goodlayers-core thật sự dùng → sẽ phá Live Mode). List đề xuất:
      `css,js,map,woff2,woff,ttf,otf,eot,svg,png,jpg,jpeg,gif,ico,webp,cur`.
      `.json` **không** đưa vào allowlist (composer.json lộ fingerprint) —
      nếu Live Mode cần `elements.json`/`icons.json`, serve qua nhánh
      `templates/admin/wp` root (xem bước 2), không qua allowlist.
- [ ] Root read `templates/admin/wp` (nhánh đầu wp_asset) cũng unfiltered —
      `SHA256SUMS`/`elements.json`/`icons.json` đang lộ public. Quyết định:
      áp cùng allowlist cho root read này (extensionless `SHA256SUMS` sẽ
      404 — verify không gì legit dùng nó runtime) hoặc ghi rõ chấp nhận.
- [ ] `/admin/front-css/{*path}` chỉ serve `.css` + `X-Content-Type-Options:
      nosniff`; uploads root giữ `.css` bất kỳ (không bắt buộc `.scoped.css`
      — twins non-scoped tồn tại thật trong templates/front-css).
- [ ] `no-store`/`nosniff` đều giữ nguyên trên các nhánh sẵn có.

## Implementation Steps

1. `src/admin/mod.rs` `wp_asset` (~1696-1760): trích `ASSET_EXT` thành
   module-scope `ALLOWED_ASSET_EXT` (list mở rộng trên); helper
   `asset_ext_ok(path)`; áp cho cả nhánh `gdlr/` (goodlayers-core, `<rest>`,
   `<stem>`, `templates/wp-plugins/<stem>`, bundled fallback) **và** root
   read `templates/admin/wp` nếu quyết định gate. Fail → 404.
2. `front_css` (~1806): `ensure ext == "css"` trước `safe_read_under`
   uploads + `nosniff` trong response.
3. `serve_uploads` giữ nguyên; verify `.svg/.html` uploads không rò qua
   `front_css` sau gate, và `served_upload_mime` vẫn force-download.

## Todo

- [ ] `ALLOWED_ASSET_EXT` module const + `asset_ext_ok` + wire cả 2 nhánh
- [ ] `front_css` ext gate + nosniff
- [ ] Manual verify curl:
  - `curl /admin/wp/gdlr/goodlayers-core.php` → 404
  - `curl /admin/wp/gdlr/tourmaster/readme.txt` → 404
  - `curl /admin/wp/gdlr/style.css` → 200
  - `curl '/admin/wp/gdlr/plugins/fontawesome/fonts/fontawesome-webfont.eot'` → 200
  - `curl '/admin/wp/gdlr/plugins/skin-ilightbox/closedhand.cur'` → 200
  - `curl /admin/front-css/x.svg` → 404; `curl /admin/front-css/style.css` → 200 + nosniff
  - Mở `/admin/plugins/goodlayers/page-builder/edit/<id>` — icons Live Mode không vỡ

## Success Criteria

- File không-asset không đọc được unauthenticated; icon fonts/cursors trong
  builder vẫn 200; admin chrome không hỏng.

## Risk Assessment

- Allowlist thiếu ext thật → 404 asset → vỡ UI. Mitigate: list đã đối
  chứng với `find templates/wp-plugins work/*/www/wp-content/plugins -type f`;
  verify curl font/cursor cụ thể trong Todo.
