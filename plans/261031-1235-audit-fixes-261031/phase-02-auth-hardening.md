---
title: "Phase 2: Auth hardening"
status: todo
priority: P1
---

# Phase 2: Auth hardening

## Overview

Audit §1.3–1.5: backup codes carry only 32 bits of entropy and are stored as
bare SHA-256 (offline-bruteforceable from a leaked `rpress.db`); the systemd
unit runs everything as root; `pwd.trim()` at login mismatches verbatim
hashing → permanent lockout for whitespace passwords.

## Requirements

- [x] `backup_codes_new` (`src/auth.rs:156-169`): ≥128-bit entropy per code —
  e.g. 2×u64 or 16 random bytes formatted `xxxxx-xxxxx-xxxxx` (keep
  human-typeable format)
- [x] Storage: keep SHA-256 only if entropy ≥128-bit (offline brute infeasible);
  otherwise switch to argon2 with per-code salt. Prefer the simple fix:
  128-bit + SHA-256 is sufficient.
- [x] Existing stored 32-bit codes: invalidate on next TOTP verify or on
  `totp_regen`; do NOT silently keep weak codes. Migration = regenerate.
- [x] `install.sh` unit: add `User=rpress` system user (`useradd -r -d ${ROOT}/work -s /usr/sbin/nologin rpress`, `chown -R rpress:rpress ${ROOT}/work`) or `DynamicUser=yes` + `StateDirectory=rpress-${SITE}` — pick `User=` since `ReadWritePaths=${ROOT}/work` already exists
- [x] `admin/auth.rs:106`: drop `.trim()` on `form.pwd` (keep `log` trim+lowercase)
- [x] Update `admin/auth.rs` TOTP pending-token flow only if code format check hardcodes `xxxx-xxxx` length — grep `\d{4}-\d{4}` / len checks

## Implementation Steps

1. `src/auth.rs`: rewrite `backup_codes_new` to draw 16 bytes/code; format
   `{:04x}` chunks to stay typeable (e.g. 5 groups of 4 hex = 80 bits is NOT
   enough — need ≥8 hex groups for 128 bits, or base32 26 chars). Decide
   format: `xxxxx-xxxxx-xxxxx-xxxxx-xxxxx` base32 (128-bit, Crockford) matches
   existing `base32_encode` helper at `auth.rs:150-153`.
2. TOTP disable/verify path (`udb::totp_*`, `admin/auth.rs`): on successful
   2FA enable or backup-code use, if stored hashes correspond to old-format
   codes (code length check before hashing — old = 9 chars `xxxx-xxxx`),
   accept but flag for regen in UI, or simply regenerate-on-enable.
   Simplest: bump a `totp` meta version; old-format codes → force regen banner.
3. `install.sh`: add `useradd`/`chown` block before `tee "$UNIT"`; add `User=rpress` in `[Service]`; keep `NoNewPrivileges`, `ProtectSystem=strict`, `ReadWritePaths`.
4. `admin/auth.rs:106`: `let password = form.pwd.to_string();` — verify no other path trims pwd before verify.
5. `README.md`/docs: note that reinstalls must re-run `install.sh` or add `User=` manually.

## Related files

- Modify: `src/auth.rs`, `src/admin/auth.rs`, `install.sh`
- Check: `src/admin/users.rs` (TOTP regen UI), `src/db/` `totp_*` helpers

## Todo

- [x] 128-bit backup codes + format decision implemented
- [x] Old-format code invalidation path
- [x] `User=rpress` + chown in install.sh
- [x] `pwd.trim()` removed
- [x] Manual test: enable 2FA, consume a backup code, restart, verify login

## Success Criteria

- Generated codes ≥128-bit; stored as SHA-256 of the code.
- Whitespace-trailing password account can log in (was: permanent lockout).
- `systemd-analyze verify` / shellcheck pass on install.sh; unit shows `User=rpress`.
- `cargo test` green.

## Risk

- Format change breaks `totp` UI copy that shows `xxxx-xxxx` — grep admin templates for the pattern and update hints.
- Root→user switch needs `chown` of existing `work/` + `rpress.db` on upgrade path; document manual step for existing installs.
