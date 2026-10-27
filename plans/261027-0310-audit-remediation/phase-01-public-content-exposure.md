---
phase: 1
title: "Public content exposure"
status: done
priority: P1
effort: "6h"
dependencies: []
---

# Phase 1: Public content exposure

## Goal

No public request path can serve non-publish content or non-canonical URLs
with 200; preview stays admin-only; the asset-origin leak is closed.

## Findings covered

- **C1** — drafts publicly served (the real hole is bigger than the audit
  wrote: `render__select_by_id` also leaks drafts — see Key insights)
- **H3** — primary-lang prefix `/fr/<slug>` aliases + shadowed slugs
- **M8** — missing translation serves primary-lang post with 200
- **M9** — asset-miss 307 leaks `asset_origin` + open-redirect primitive
  (moved here from phase 2 — it is a serve-boundary fix)
- **L** — `front_id_for_lang` non-deterministic `__trashed` pick; `clean`
  charset allowlist; unknown-Host fallback on multi-site
- **NEW (red-team)** — old-slug redirect + `public_root` serve drafts and
  *cache* them; `.md`/alias files survive unpublish (partially phase 5)

## Key insights

- **Corrected premise** (red-team finding): `render__select_id_lang` has
  exactly ONE caller — `render_slug` (render.rs:137) — and all its callers
  are publish-only paths. Preview uses a *different* query:
  `preview_post` → `render_preview` → `render_slug_by_id` →
  `render__select_by_id` (`status!='trash'` — leaks drafts, pending, AND
  future posts).
- The real exposure surface is therefore `render_slug_by_id`, which is
  PUBLIC via: `public_root` (admin/mod.rs:268), the old-slug redirect
  fallback (render.rs:222-230), and front-translation fallbacks
  (render.rs:161,229). It also fabricates `status="publish"` so the
  write-back gate can't see real status.
- Fix shape: `select_id_lang` → `status='publish'` (one line, zero caller
  migration); `render_slug_by_id` splits into a **public** variant
  (`status='publish'`, returns real status) and keeps `select_by_id` for
  `render_preview` only; `public_root`/old-slug/front-fallback callers
  switch to the public variant.
- `front_id_for_lang`/`page_on_front` reads also lack status predicates —
  a draft `page_on_front` would render at `/`; add `status='publish'` to
  the front-post lookup.
- Asset-miss 307 ordering bug: extension-redirect runs BEFORE
  `is_junk_prefix` (mod.rs:350-356 vs 359-364); `asset_origin` is
  settings-table data (attacker-influenced) with only
  `origin.starts_with("http")` — open redirect primitive.
- Pre-existing alias cache files (`{out}/{primary}/<slug>/`) keep serving
  after the 301 lands — the cache-hit path reads files BEFORE split_path
  (mod.rs:332-337). Needs a purge, not just a gate.
- `site_for` returns `&SiteCtx` (mod.rs:503-512) — can't express "no site"
  without touching every handler; use a router-level `host_known` pre-check
  instead.
- Admin writes translation links to legacy `post_translations` pair table
  while members lookups read `post_translation_members` (import-only
  writes). Front/translation fixes must reconcile or they degrade on
  admin-maintained sites.

## Files to modify

- `src/db/sql/render__select_id_lang.sql` — `status='publish'`
- `src/db/sql/render__select_by_id.sql` — stays for preview; NEW
  `render__select_by_id_public.sql` (`status='publish'`)
- `src/render.rs` — `render_slug` + `render_slug_by_id`: public callers use
  the public variant; return REAL status (no fabricated "publish"); old-slug
  fallback + front-translation fallback guarded; `front_id_for_lang`
  deterministic + publish-only; front-post lookup gets status predicate
- `src/lang.rs` — primary-lang prefix → canonicalize signal
- `src/admin/mod.rs` — `public_page`: `status != publish → 404` defense;
  `public_root`: public-variant + status guard; `clean` charset allowlist;
  `host_known` pre-check for multi-site (421/404 unknown host); **asset 307
  block**: prefix allowlist + `is_junk_prefix` checked BEFORE extension
  redirect + `asset_origin` host validated (config allowlist or same-site
  only) + uploads-miss 307 gated same way
- `src/db/posts.rs` — admin translation-link writes upsert
  `post_translation_members` alongside `post_translations`

## Implementation steps

1. `render__select_id_lang.sql`: `IN ('publish','draft')` → `='publish'`.
2. New `render__select_by_id_public.sql` (`status='publish'`); split
   `render_slug_by_id` so public callers (`public_root`, old-slug fallback,
   front fallbacks) return `(html, real_status)` and require publish;
   `render_preview` keeps `select_by_id`.
3. `public_page` + `public_root`: `status != "publish" → 404` regardless
   (belt-and-suspenders).
4. `lang::split_path`: first segment == `cfg.primary` → canonicalize →
   `301 /{rest}`. Check this BEFORE rendering; note cache-hit path runs
   earlier — see step 9's purge.
5. Non-empty resolved lang → require `lang=?2` exactly (drop the
   `'' OR` fallback); missing → 301 to translation-group canonical or 404.
6. `front_id_for_lang`: try `post_translation_members` mapping to primary
   `page_on_front` first; `__trashed` fallback gets `ORDER BY id LIMIT 1`
   + publish predicate; `page_on_front` read gets status predicate.
   Also upsert members on admin translation-link writes.
7. `clean` allowlist `[a-z0-9/_%.~-]`; multi-site `host_known` router
   pre-check → 421 for unknown Host.
8. Asset-origin: reorder `is_junk_prefix` before extension-redirect;
   allowlist redirect prefixes (`wp-content/`, `wp-includes/`, `uploads/`);
   validate `asset_origin` (allowlisted hosts or same-site); gate the
   uploads-handler 307 identically.
9. Upgrade sweep: delete `{out}/{primary_lang}/` dirs and any
   `{clean}.html` alias files on first run after deploy (or document as a
   manual `rm` step in the phase verification).
10. Reconciliation (shared with phase 5): every cache candidate suffix
    public_page can write — `index.html`, `index.md`, `{clean}.html`,
    per-lang alias dirs — must be enumerated for invalidation/removal.

## Verification

- `cargo test`: draft slug → 404; draft reachable via old-slug → 404 (and
  no cache write); draft `page_on_front` → `/` 404/redirect; `/fr/x` → 301;
  `/es/<fr-only-slug>` → 301/404 not 200-FR; `/wp-admin/x.php` → 404 not
  307; `/.env` → 404 not 307.
- Sandbox curls against work/blueskies or similar.

## Todo

- [x] select_id_lang publish-only; render_slug_by_id public/preview split
      returning real status
- [x] public_page/public_root publish guard
- [x] Primary-lang prefix → 301 (before render; + alias-file purge)
- [x] Non-primary lang exact match + group-canonical redirect
- [x] front_id_for_lang deterministic + publish predicates; admin upserts
      members
- [x] clean charset; host_known multi-site check
- [x] asset-origin: allowlist + ordering + origin validation + uploads gate
- [x] Enumerate all cacheable candidate suffixes for invalidation contract

## Risks

- Preview must keep working — `render_preview` is the only caller allowed
  `select_by_id`; verify preview link in admin posts list still 200s a draft.
- `asset_origin` allowlist must be configured per-site (env); a missing
  allowlist + non-empty asset_origin = default deny (404 on miss), which
  is safe but may regress sites relying on origin fallback — flag in the
  phase verification.
- Purge of `{out}/{primary}/` assumes no legitimate content renders under
  the primary prefix (true by definition after step 4) — do it once, loudly.
