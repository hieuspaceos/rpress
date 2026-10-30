---
title: "Phase 3: Bridge path sanitization"
status: todo
---

# Phase 3: Bridge path sanitization

## Overview

Defense-in-depth (severity low, admin-only): axum `Path` percent-decodes
`{*rest}`; `bridge_dispatch` nối thẳng vào request line HTTP của php -S.
CRLF trong `rest` → inject header/dòng mới vào request nội bộ.

## Requirements

- [ ] Reject `rest` chứa byte control (\r, \n, \0, space) **và** segment `..`
      (audit: `%2e%2e%2f` decode thành `../` → traversal thoát php -S docroot —
      second-order vuln thật, không chỉ defense-in-depth).
- [ ] Reject leading `//` và `:` trong path (absolute-URI/host confusion
      trong request line). `?`/`#` trong decoded rest → reject luôn (rest là
      path-only; nhúng query giả không có use case hợp lệ).
- [ ] Không đổi hành vi với path hợp lệ (`/wp-admin/...`, ajax mapping).

## Implementation Steps

1. `src/admin/bridge/http.rs` `request()` (~15): guard sớm —
   ```rust
   anyhow::ensure!(!path.bytes().any(|b| b < 0x20 || b == b' ' || b == 0x7f));
   anyhow::ensure!(!path.split('/').any(|s| s == ".."));
   anyhow::ensure!(!path.contains('?') && !path.contains('#') && !path.contains(':') && !path.starts_with("//"));
   ```
   Tầng transport vì mọi caller đều qua đây.
2. Test tay:
   - `curl '/admin/plugins/goodlayers/x%0D%0AX-Evil:%201'` → fail, không forward
   - `curl '/admin/plugins/goodlayers/%2e%2e/%2e%2e/etc/passwd'` → fail
   - Editor flow legit (`page-builder/edit/<id>`) vẫn 200

## Todo

- [ ] Guard control-bytes + `..` + `?`/`#`/`:` + `//` trong http::request
- [ ] Verify curl injection + traversal fail; editor flow 200

## Success Criteria

- CRLF inject và `..` traversal đều không tới php -S; proxy hợp lệ không đổi.

## Risk Assessment

- Zero-risk gần như: guard chỉ reject input đã sai protocol. `%2f` decode
  thành `/` vẫn cho qua (legit — adapter paths dùng dấu /).
