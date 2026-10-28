---
phase: 3
title: "Migrate babe adapter to spec (prove parity by deletion)"
status: pending
priority: P2
effort: "0.5-1d"
dependencies: [1, 2]
---

# Phase 3: Migrate babe adapter to spec

## Goal

Delete the handwritten babe Rust stack; `plugins/specs/babe.toml` +
harvest produce byte-equivalent dmc/imagedmc output. This is the parity
proof for the whole plan: the new mechanism replaces the old cost.

## Changes

1. Write `plugins/specs/babe.toml`:

```toml
slug = "babe"
claim_types = ["to_book"]
option_prefixes = ["babe_"]

[[kinds]]
kind = "to_book"
section = "bookings"
noun = "booking"
url_base = "booking"
taxonomies = ["ba_location", "ba_type", "categories"]
harvest = true
```

2. Delete:
   - `src/admin/plugins/babe.rs` + `src/admin/plugins/babe/render.rs`
   - `templates/single_book.html`
   - `templates/elements/babe_enquiry.html`
   - `&babe::ADAPTER` line + `mod babe;` in `src/admin/plugins/mod.rs`
3. Keep (shared, not babe-specific):
   - `theme_tevily` adapter — declares `ThemeDef`, not stack logic.
   - Enquiry form caveat (verified): harvested page keeps the origin
     CF7 `<form action>` with `harvest_page` rewriting origin→site_url —
     so it posts to `{site_url}/booking/.../#wpcf7-f7-ol`, a URL rpress
     doesn't serve → form looks alive but submits to 404. Decide in
     implementation: (a) harvest-layer rewrite mapping `wpcf7` actions
     to `/form/enquiry` + seeded `forms` row, or (b) leave dead (form
     degrades like origin JS-disabled). Choose (a) — a visibly broken
     form is worse than a mapped one.
4. Admin bookings list/edit screens come from `KindDef` — spec produces
   identical `/admin/section/bookings` behavior.

## Divergences to resolve before deleting

| Current Rust behavior | Spec equivalent | OK to drop? |
|---|---|---|
| `render.rs` hero/chips/steps/share/enquiry markup | origin-harvested HTML | Yes — harvest is *more* accurate |
| `single_book.html` template | same | Yes |
| `render.rs` `canonical`, `hreflang`, `translation_urls` | origin `<link rel=canonical>` + hreflang in harvested head | Verify — origin already emits these (WP); harvest keeps them |
| `import_hooks` claim `to_book` | spec `claim_types` | Equal |
| `option_prefixes = ["babe_"]` | spec | Equal |
| `title`/`duration` chips, itinerary accordion | origin markup | Yes |
| `site_data_feeds` / menu taxonomy chips (in render.rs) | baked into origin HTML | Yes — origin already renders menus |

If harvested HTML diverges visually from what `render.rs` produced (e.g.
missing rewrite, origin markers stripped), fix the **harvest rewrite
layer** (`harvest_page`), not reintroduce the Rust scaffold.

## Rollback

Spec + deleted files are pure git — revert restores the Rust adapter.
`harvest_posts` rows left by import are inert if spec/adapter gone
(generate only fetches listed ids).

## Verification

- `cargo check` clean after deletion; `cargo test` all pass (babe
  adapter had no dedicated tests; spec tests from phase 01 cover claim).
- Regenerate imagedmc + dmc; `git diff` on `out/` (or file-hash
  comparison of `booking/**`) — only expected diffs: harvest markup vs
  scaffold markup (better, not broken). Routes count identical.
- `scripts/e2e-site.sh sites/dmc.env serve --smoke`: `/booking/*` 200,
  contains origin markers (`tevily`/`gsc-booking`/`elementor`).
- aucoeur + blueskies: `git status` on out/ clean vs baseline — Rust
  adapters untouched.