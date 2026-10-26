---
title: "Phase 1: Security Hotfixes"
status: todo
priority: P1
---

# Phase 1: Security Hotfixes

## Overview

Close the two direct security findings first: scriptable legacy uploads served inline from `/wp-content/uploads/*`, and CSRF-free logout. This phase is intentionally small and front-loaded because it reduces production risk before broader refactors.

## Requirements

- [ ] Local upload serving must not inline SVG, XML, HTML, JS, CSS, or other scriptable text from `www/wp-content/uploads`.
- [ ] Disallowed/scriptable upload extensions must return a safe response: either `403/404`, or `application/octet-stream` plus `Content-Disposition: attachment`.
- [ ] Existing safe media types continue to work for images, PDFs, common video/audio, and fonts.
- [ ] `POST /api/logout` must validate CSRF before removing the session and clearing the cookie.
- [ ] Admin chrome logout form must include the current session CSRF token.

## Related Code Files

| File | Action | Notes |
|---|---|---|
| `src/admin/mod.rs` | Modify | Upload serving MIME policy and possibly helper extraction. Evidence: upload MIME map around current local upload handler. |
| `src/admin/auth.rs` | Modify | Logout handler currently deletes session without CSRF. |
| `src/admin/mod.rs` | Modify | Admin chrome/logout form currently lacks hidden CSRF. |
| `tests/` | Modify/create | Add behavior tests if route helpers can be exercised without full server. |

## Implementation Steps

1. Locate the local upload serving branch in `src/admin/mod.rs` and extract a small MIME decision helper if none exists.
2. Define one explicit allowlist for inline uploads. Treat `svg`, `xml`, `html`, `htm`, `js`, `mjs`, `css`, and unknown text-like files as non-inline.
3. For non-inline scriptable files, choose the boring safe behavior: return `application/octet-stream` with `Content-Disposition: attachment`; if the current route style makes that awkward, return `403` for these extensions.
4. Update comments so they match implementation. Do not leave the current false “svg → octet-stream” comment unless the code actually does it.
5. Update the admin logout form to post `csrf`.
6. Update `auth::logout` to parse form data and require `check_csrf` before deleting the session. Preserve cookie clearing after a valid logout.
7. Add tests for the MIME decision helper and logout CSRF behavior where feasible. If full route testing is too heavy, test extracted pure helpers and document manual smoke coverage.

## Todo

- [ ] Add/adjust upload MIME policy helper.
- [ ] Block or force-download scriptable legacy uploads.
- [ ] Add CSRF field to logout form.
- [ ] Enforce CSRF in logout handler.
- [ ] Add focused tests or helper-level tests.

## Success Criteria

- `GET /wp-content/uploads/x.svg` cannot execute inline from local uploads.
- `GET /wp-content/uploads/x.js` and `.css` cannot execute inline from local uploads.
- Safe image/media upload paths still resolve with expected content type.
- `POST /api/logout` without CSRF returns forbidden and leaves the session intact.
- `POST /api/logout` with valid CSRF logs out successfully.
- `cargo test` and `cargo check` pass.

## Risk Assessment

- Blocking inline SVG may affect sites that intentionally embedded SVG uploads. Security wins; provide force-download instead of inline rendering.
- Logout form route signature changes can break callers. Search all `/api/logout` forms and update them in the same phase.

## Security Considerations

This phase directly reduces stored-XSS and CSRF/session disruption risk. Do not broaden upload allowlists without a test proving the content type is non-scriptable in browsers.
