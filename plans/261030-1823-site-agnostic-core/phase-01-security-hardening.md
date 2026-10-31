---
title: "Phase 1: Security Hardening"
status: todo
priority: P0
---

# Phase 1: Security Hardening

## Overview

Close the two MEDIUM findings and three LOW findings from the security
audit. All are surgical fixes with clear reachability proofs already
established.

## File Ownership

- `src/admin/bridge/` (mod.rs dispatch, php_cgi.rs sandbox grants, mu-plugin/bridge.php)
- `src/admin/menus.rs`, `src/render/menu_url.rs`, `src/render/site_data.rs` (menu_li_mode only)
- `src/admin/admin.rs` (intake_post body cap), `src/admin/auth.rs` (2FA token)
- `src/admin/serve.rs` (wp_asset headers — coordinate: phase 2 also owns serve.rs; do headers fix here, leave structure for phase 2)

## Requirements

- [ ] M1: `bridge_dispatch` (`src/admin/mod.rs:1114-1154`) strips `Cookie`, `Authorization`, `X-Forwarded-*`, `X-Real-IP` before `fwd.push`; forwards a synthetic `X-Rpress-User`/`X-Rpress-Role` identity header instead. CSRF form fields are never forwarded (POST body filtering — drop `csrf`/`_csrf` keys when rebuilding the proxied form body; if body is proxied raw, document and gate).
- [ ] M1b: review the `mu-plugins` write grant in `php_cgi.rs:189,283-287` — a compromised child can plant persistent PHP. Tighten to read-only if the bridge flow doesn't require writes; if writes are needed (WP mu-plugin drop-in generation), move generation to host-side pre-spawn so the child gets a read-only bind.
- [ ] M2: `menu_li_mode` in `render/site_data.rs:673-708` applies `esc_attr()` to `url`, `tattr`, `classes`. `resolve_menu_url` in `menu_url.rs:76-89` gets a scheme allowlist (`https?`, `/`, `#`, `mailto:`) — non-matching custom URLs render as `#`. Decide `title` deliberately: if image-menu markup is needed (see comment at site_data.rs), sanitize to an allowlist (`<img>` with safe attrs only) rather than raw.
- [ ] M2b: `menus_add`/`menus_save` (`admin/menus.rs:837-846`) validate scheme server-side at save time (reject `javascript:`/`data:`/`vbscript:`), so DB never stores a hostile URL even via direct import — add the same allowlist check to the import path for `menu_items.url` if trivially reachable.
- [ ] L1: intake body cap — replace the `Content-Length` header check (`admin/admin.rs:1252`) with `RequestBodyLimitLayer` (or `DefaultBodyLimit::max(32*1024)`) scoped to intake routes; verify chunked-encoded POST >32KB now 413s.
- [ ] L2: `wp_asset` responses (`serve.rs:650-732`) gain `X-Content-Type-Options: nosniff`; for `.svg` specifically add `Content-Disposition: attachment` OR `Content-Security-Policy: script-src 'none'` — pick the one that doesn't break vendored SVG usage (fonts/flags are `<img>`-loaded, so attachment is safe).
- [ ] L3 (optional, cheap): move 2FA pending token from `?p=` query to a `HttpOnly` cookie or POST-only transport (`auth.rs:181,256-345`).

## Implementation Steps

1. `bridge_dispatch`: build `fwd` from an allowlist (`Host`→bridge target, `Content-Type`, `Content-Length`, `Accept`, `User-Agent`) instead of copying all headers. Verify no PHP screen relies on inbound cookies (mu-plugin auto-login uses `X-Rpress-Bridge` bearer, not the session cookie).
2. Sandbox grants: read `php_cgi.rs` seatbelt profile + bwrap spec; make `mu-plugins` read-only unless bridge writes there at runtime — check `mu-plugin/bridge.php` and spawn sequence for who creates the drop-in.
3. `menu_url.rs`: add `fn safe_menu_url(url:&str)->String` with scheme allowlist; call it in `resolve_menu_url` custom branch.
4. `site_data.rs`: `esc_attr` on `url`/`tattr`/`classes` interpolations; img-allowlist sanitizer for `title` if markup required.
5. `menus.rs`: same allowlist at `menus_add`/`menus_save` — invalid scheme → flash error, no write.
6. Intake: `DefaultBodyLimit::max(32_768)` on the intake POST routes (check where routes register in `mod.rs serve()`); remove the Content-Length pre-check (keep as early reject too — both is fine).
7. `wp_asset`: add `nosniff` header to all responses; `attachment` for svg.
8. Tests: add cases to `tests/hardening.rs` — forged `javascript:` menu URL renders `#`; chunked intake >32KB → 413; bridge fwd header list lacks `cookie`/`authorization` (unit-test the filter fn).

## Success Criteria

- Throwaway repro: save a custom menu item with `javascript:alert(1)` → rejected at save; a DB-seeded one renders `href="#"`.
- `curl -X POST --data-binary @big --header "Transfer-Encoding: chunked" /form/contact` (>32KB) → 413.
- Inspect proxied request at PHP side (or unit test): no `Cookie` header reaches `php -S`.
- `cargo test` green; `visual_smoke.sh` passes (menu output unchanged for legit URLs).

## Risk Assessment

- M1 strip may break a bridge feature relying on an inbound header — mitigation: allowlist includes everything `http.rs` actually uses; grep `bridge.php` for `$_SERVER['HTTP_*']` reads and keep only what it needs.
- SVG-as-attachment: verify no vendored SVG is inlined via `use`/`object` in admin chrome — grep templates for `.svg` references.
- Menu `title` sanitize could strip legit markup (the image-menu case) — mitigation: allowlist `<img src|alt|class|width|height>` only, test with the reference site's image menu items.
