---
title: "Phase 4: Admin 2FA (native TOTP)"
status: todo
---

# Phase 4: Admin 2FA (native TOTP)

## Overview

Thêm TOTP (RFC 6238) native vào `rpress` — không phải WP plugin (plugin PHP
không có nơi chạy trên static site). Login trở thành 2 bước: password
đúng → mã 6 số từ authenticator app → session mới được mint. Giảm risk
khi password bị lộ (reuse/breach) — attack vector duy nhất còn lại sau
khi phần exposure vá xong.

## Requirements

- [ ] Schema: `users.totp_secret TEXT` (20 bytes, base32, 160-bit entropy),
      `users.totp_enabled INTEGER`, `users.totp_last_step INTEGER` (chống
      replay trong cùng window), `users.backup_codes TEXT` (JSON array của
      `sha256_hex(code)` — **không lưu plaintext**: đọc DB = lộ hết codes).
- [ ] **BLOCKING (audit): Application Passwords bypass TOTP.** `need_auth`
      chấp nhận `Authorization: Basic` → `app_pass_sess` → Sess đầy quyền.
      Bắt buộc: (i) chặn `users_app_pass_add` khi `totp_enabled=1`, **và**
      `DELETE FROM app_passwords WHERE user_id=?` khi enable 2FA (else
      credential đã mint sẵn vẫn bypass); (ii) `session_delete_user` khi
      enable để revoke sessions cũ đã mint.
- [ ] Verify step giữa `verify_password` và session mint; fail TOTP đếm
      `login_hits`; response không phân biệt "user có/không 2FA" (sai
      password hay sai TOTP đều redirect `/admin/login` generic).
- [ ] Pending-token (32B random, TTL 5 phút) map trong `App`: **cap 1k
      entries + evict-oldest + sweep TTL on insert** (spray valid-password
      không được phình map — attacker biết password nhưng không biết TOTP
      là đúng threat model).
- [ ] Profile `/admin/profile`: QR `otpauth://totp/rpress:{email}?secret=
      {b32}&issuer={site}&algorithm=SHA256&digits=6&period=30` (crate
      `qrcode`, SVG inline) + secret text + confirm-first-code → enable;
      disable + regenerate backup codes đều yêu cầu **mã TOTP hiện tại**
      + csrf; 10 backup codes one-time.
- [ ] Verify + mint **atomic**: backup-code consume trong cùng write tx
      với session insert (chống 2 request song song dùng 1 code 2 lần);
      `totp_last_step` update cùng tx; re-read user row tại verify (role
      có thể đổi giữa flow — không snapshot từ pending token).
- [ ] Off-by-default: `totp_enabled=0` → flow cũ y nguyên.
- [ ] Rescue path documented: `sqlite3 rpress.db "UPDATE users SET
      totp_enabled=0 WHERE id=?"` — ghi vào `CMS-OPERATIONS.md`.
- [ ] `eprintln!` log khi admin disable 2FA của user khác (convention có
      sẵn trong auth.rs — không có audit-log infra).

## Implementation Steps

1. `Cargo.toml`: `qrcode` (SVG render) + `sha2` (đã có transitively? verify —
   nếu chưa thì add; TOTP dùng **HMAC-SHA-256**: ~30 dòng, authenticator
   apps mainstream (GA/Aegis/1Password) nhận `algorithm=SHA256`; ghi caveat
   app cũ chỉ hỗ SHA-1 → nếu cần SHA-1 thì thêm `sha1`+`hmac` micro-deps,
   quyết định lúc implement bằng cách test thật với 1 app).
2. `schema.rs` MIGRATIONS: 4 cột users mới (ALTER TABLE pattern có sẵn).
3. `src/auth.rs`: `totp_at(secret_b32, step)` HMAC-SHA-256 (reuse
   `hmac_sha256` nếu có, else ~20 dòng), `totp_verify(secret, code,
   last_step)` với drift ±1 step, reject `step <= last_step`; `base32_decode`;
   `gen_backup_codes()` → 10×`xxxx-xxxx`.
4. `src/db/users.rs`: `totp_get/set/clear/last_step_update`,
   `backup_codes_{get,consume}` (consume = remove hash khỏi JSON array
   trong write tx), `app_pass_revoke_all(user_id)`, `session_delete_user`.
5. `src/admin/auth.rs` login: sau `verify_password` OK + `totp_enabled`:
   pending-token → `HashMap<tok,(user_id,Instant,usize fails)>` trong `App`;
   redirect `/admin/2fa?p={tok}`; GET render form code (invalid/expired tok
   → redirect `/admin/login`, không render form — tránh enumerate);
   POST verify: TOTP OK (step > last → update last_step) hoặc backup-code
   consume → mint session + `session_delete_user` cũ + xoá pending;
   `fails >= 5` → xoá pending + đếm `login_hits`.
6. `src/admin/users.rs` `user_edit_html` profile branch: section 2FA —
   off → Enable → setup page (QR svg + secret + input confirm); on →
   status + Regen codes + Disable (cả 2 POST kèm csrf + current code);
   enable handler: `app_pass_revoke_all` + `session_delete_user`.
7. `mod.rs` routes: `/admin/2fa` GET+POST (không need_auth — đang giữa
   login; chỉ pending-token gate).

## Todo

- [ ] schema cols + db helpers (totp/backup/app_pass revoke/session delete)
- [ ] totp HMAC-SHA-256 + base32 + backup codes (hashed)
- [ ] pending-token map (cap+TTL+sweep) + `/admin/2fa` routes
- [ ] profile setup UI (QR svg + confirm + disable + regen)
- [ ] Verify: off-2FA login unchanged; on-2FA sai mã → reject + rate-limit;
      backup code one-time; concurrent POST không double-mint; app_pass
      bị revoke khi enable; sessions cũ bị revoke
## Success Criteria

- User bật 2FA → login yêu cầu mã; mã sai → fail như sai password +
  rate-limit; backup code dùng được 1 lần rồi chết; Basic-auth app
  password không bypass được (rows đã xoá khi enable, add bị chặn).
- User không bật → không đổi gì.
- `cargo check`, e2e `check` pass; `/admin` flow 200 như cũ.

## Risk Assessment

- Lockout self-service: mất phone + backup codes → rescue chỉ còn
  `sqlite3 users SET totp_enabled=0` — document trong CMS-OPERATIONS.md.
- Pending-token trong RAM: restart giữa login → user login lại — by design.
- HMAC-SHA-256: app authenticator cũ không hỗ → fallback thêm `sha1`+`hmac`
  micro-deps nếu test app thật fail.
- Session revoke on enable: user bị kick mọi device khi bật 2FA — intended
  (sessions mint pre-2FA không được tồn tại).
