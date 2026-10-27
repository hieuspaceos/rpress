---
title: "Phase 1: Start"
status: todo
---

# Phase 1: Start — audit evidence pack

## Overview

Tất cả phát hiện đã có evidence trong audit session 2026-10-27 —
không cần scout thêm. Phase này chỉ làm 1 việc: chụp lại baseline
để so sánh sau fix.

## Requirements

- [ ] Ghi lại hành vi hiện tại của 4 lỗ hổng (repro)

## Repro baseline (chạy trên site test có sẵn)

```bash
cargo build --release
./target/release/rpress serve --site default --db rpress.db \
    --templates templates --out cache --www www --bind 127.0.0.1:8901
```

1. **`.md` bypass**: `for i in $(seq 1 70); do curl -so /dev/null -w '%{http_code}\n' http://127.0.0.1:8901/khong-ton-tai-$i.md; done`
   → kỳ vọng hiện tại: toàn 404, **không** 429 nào (rate-limit không bắn).
2. **Argon2 block**: `ab -n 50 -c 10` POST /admin/login với cred sai →
   quan sát GET / (cached) latency tăng cùng lúc.
3. **Vec serve**: đặt file 500 MB vào `www/wp-content/uploads/`, curl nó,
   theo dõi RSS process.
4. **Stampede**: xóa 1 file cache của slug hot, `ab -n 50 -c 50` vào slug đó
   → đếm số render (log hoặc mtime phân biệt được).

## Todo

- [ ] Chạy 4 repro, dán output vào `reports/audit-baseline.md`
- [ ] Nếu repro nào không tái hiện được → note lại, điều chỉnh phase tương ứng

## Success Criteria

`reports/audit-baseline.md` tồn tại với output thật của từng repro;
không có mục "chưa kiểm được" nào bị bỏ trống lý do.
