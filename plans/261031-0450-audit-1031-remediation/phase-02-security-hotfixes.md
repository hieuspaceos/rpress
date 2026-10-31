---
phase: 2
title: "Security hotfixes"
status: completed
priority: P1
effort: "2.5h"
dependencies: [1]
---

# Phase 2: Security hotfixes

## Goal

Close the fail-open auth boundary, the unsandboxed-PHP escape, the
open redirect, and the remaining defense-in-depth gaps — the findings
an external attacker or compromised vendored plugin can reach.

## Files to Create / Modify

- Modify: `src/admin/bridge/mu-plugin/bridge.php:28-34` — token fence.
- Modify: `src/admin/bridge/playground.rs:82-83` — stop writing
  `rpress-bridge.php` into the persistent site tree
  `www/wp-content/mu-plugins/`; it belongs only inside the bridge
  docroot. Also delete the stray file from any live `www/` it already
  landed in (migration note for ops, not code).
- Modify: `src/admin/bridge/php_cgi.rs:237-250, 537-548` — sandbox
  selection + spawn fallback.
- Modify: `src/admin/bridge/php_cgi.rs:278-300` — bwrap profile.
- Modify: `src/admin/admin.rs:1121-1125` — `safe_back` guard.
- Modify: `src/admin/mod.rs:1064-1102` — `plugin_screen` POST path.
- Modify: `src/admin/serve.rs:686-692` — `request_host` IPv6.
- Modify: `src/admin/mod.rs:438-455` — `client_ip` proxy warning.

## Requirements

- [x] `bridge.php` fence is `if ($tok === '' || !hash_equals($tok, $got))`
      — a PHP host without the constant gets 403, never admin identity.
- [x] `rpress-bridge.php` is no longer written under
      `www/wp-content/mu-plugins/` (site tree stays clean; docroot-only).
- [x] When neither `sandbox-exec` nor `bwrap` exists, bridge spawn
      **fails closed** with a clear error unless env
      `RPRESS_BRIDGE_UNSAFE=1` is set explicitly — no silent
      full-privilege `php -S`.
- [x] bwrap egress: **true loopback-only isolation is unreachable
      without root** — `--unshare-net` kills host→child reachability
      (rpress proxies to the child's loopback port from the host
      netns), and `slirp4netns`/`pasta` ADD outbound NAT — the
      opposite of what's wanted. Deployable fix: keep `--share-net`,
      correct the stale code comment claiming loopback-only, and make
      `install.sh` add `IPAddressAllow=localhost IPAddressDeny=any`
      to the generated systemd unit so the whole rpress service is
      loopback-egress-only (verify first that rpress needs no other
      outbound — `pull.rs` is inbound). Document residual risk in
      `docs/architecture.md`.
- [x] `safe_back` rejects `\`:
      `p.starts_with('/') && !p.starts_with("//") && !p.contains('\\')`.
      Do NOT use a strict charset allowlist — legitimate `_page` values
      carry `?query` and `#frag`, which `^/[A-Za-z0-9/_.~%-]+$` breaks.
- [x] `POST /admin/plugins/{slug}/*` enforces `check_csrf` for
      adapter `ScreenDef` posts; the bridged wp-admin path documents the
      SameSite=Lax reliance in a comment + `docs/architecture.md`.
- [x] `request_host` handles `[v6]:port` (take up to `]` when host
      starts with `[`; else split on last `:` only if suffix parses u16).
- [x] Startup warns once when bind is loopback, `RPRESS_TRUSTED_PROXY`
      is unset, and `settings.site_url` is a non-localhost production URL.

## Implementation Steps

1. bridge.php: flip the fence; grep `www/` for stray deployed copies
   and note removal in the commit message.
2. php_cgi: replace the `None` arm `eprintln!`-and-spawn with an error;
   gate the unsafe path behind `RPRESS_BRIDGE_UNSAFE`.
3. bwrap: keep `--share-net`, fix the misleading comment, patch
   `install.sh` systemd unit with `IPAddressAllow`/`Deny`; verify a
   bridged request still works; document the residual (user-space
   rpress can't self-isolate netns).
4. `safe_back`: reject `\` (keep `?`/`#` allowed); regression test
   posting `_page=/\evil.com` expects redirect to `/`.
5. `plugin_screen`: add `check_csrf(&s, …)` on the non-bridge POST arm.
6. `request_host`, `client_ip` warning.

## Security Considerations

The bridge mu-plugin currently grants any request `wp_set_current_user(1)`
when loaded outside the token context — treat this as a
credential-equivalent leak: rotate nothing, but verify no deployed
`www/` still carries the file.

## Verification

- `cargo test` green + new tests: redirect rejects `/\evil.com`;
  `weekday_of`/hex guards (phase 6 uses the same fixture file).
- Manual: `RPRESS_BRIDGE_UNSAFE` unset + no sandbox tool → spawn error,
  not a bare `php -S`.
- `php -l` on edited bridge.php; fence test via a PHP CLI harness
  (`php -r` including the file with token undefined → 403 path).
