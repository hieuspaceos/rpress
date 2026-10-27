---
title: "Core extras — FTS5 search, TOC, share/reading-time, redirects UI"
description: "Four native core additions confirmed by user: SQLite FTS5 search replacing external pagefind, native TOC + share buttons + reading time in post chrome, and a redirects manager admin screen on the existing redirects table."
status: pending
priority: P2
effort: "2-3d"
tags: [search, fts5, toc, redirects, seo, core]
created: 2026-10-27
---

# Core extras

## Premise (user-confirmed 2026-10-27)

Four universal features belong in core, not plugins:

## 1. FTS5 search (native, replaces pagefind dep)

- Today: `pagefind` external binary optional at generate; `/recherche`
  degrades to a notice if missing.
- Target: SQLite FTS5 (rusqlite `bundled` already compiles it — verify
  the feature is enabled in our build flags).
- Index: posts(title, body text, excerpt) + terms; per-lang tables or a
  lang column. Rebuild on generate + incremental update on post save
  (hook into the phase-5 regen funnel — same invalidation path).
- Route: `/search` (and per-lang `/es/buscar` mapping via lang config —
  do NOT hardcode `/recherche` again) renders results server-side;
  still static-friendly (query param, no state).

## 2. TOC (native)

- Render-time: extract h2/h3 from body_html → nested list, inject into
  post chrome (template include, theme-toggleable).
- Anchor ids: ensure headings have stable ids at render (slugify) —
  needed for TOC links AND deep links.

## 3. Share buttons + reading time

- Share: static anchor set (facebook, x, whatsapp, telegram, copy-link)
  emitted in post chrome — no third-party JS.
- Reading time: `word_count / 200` at render → `X phút đọc` (localized
  via lang config).

## 4. Redirects manager UI

- `redirects` table + `redirects.map` emit already exist — only missing
  the admin screen.
- New `admin/redirects.rs`: list (pager), add old_slug→new_url (301/302),
  edit, delete, optional CSV import. Post-save → regenerate
  `redirects.map` via the regen path.

## Non-goals

- Analytics, A/B testing, realtime anything.
- Newsletter stays in `261027-mail-providers` (subscribe→mail).

## Success criteria

- [ ] `/search?q=` returns FTS5 results with zero external deps.
- [ ] TOC renders on posts with ≥2 headings; deep links work.
- [ ] Share + reading time on post chrome, toggleable.
- [ ] Redirects admin CRUD updates redirects.map.
- [ ] `cargo check` + `cargo test` clean.

## To be phased by ak:plan

Outcome contract — needs scout (FTS5 feature flag, search route home,
template chrome injection, regen funnel hook points) → phases → red-team.
Suggested phase split: (1) FTS5 search, (2) TOC+share+reading-time chrome,
(3) redirects UI.
