---
title: "Phase 6 — Regression gates + doc resync"
phase: 6
status: pending
priority: P1
effort: 2h
files:
  - tests/site_env.rs
  - tests/hardening.rs
  - tests/exposure.rs
  - scripts/check-layers.sh
  - docs/architecture.md
  - CMS-OPERATIONS.md
  - README.md
---

# Phase 6 — Regression gates + doc resync

Land the permanent gates so these bug classes can't silently return.

## Tasks

### 6.1 Test coverage matrix — consolidate per-phase tests

Every fix phase added tests; this phase makes sure the suite
coheres and adds the missing e2e assertions:

| Finding | Gate | Where |
|---|---|---|
| SITE_URL persist | save→load round-trip; domain==origin bail | tests/site_env.rs |
| .md under limiter | 70× `.md` misses → 429 | tests/exposure.rs |
| publish_due timer | seeded future post flips w/o admin request | tests/hardening.rs |
| ez-toc XSS | `&lt;script&gt;` heading → escaped TOC | unit test render_post |
| bridge kill slug | `--slug ../..` → non-zero, no fs change | tests/hardening.rs |
| ?tax= XSS | raw quote never reaches href | tests/hardening.rs (curl) |
| deploy www exclude | Dest::Local push → no bridge/, no .secret-* | tests/site_env.rs |
| install_php escape | crafted theme → balanced literal | unit test php_cgi |
| pull wp_id miss | row skipped, no wrong-post meta | unit test pull.rs |
| pull multi-value | OR REPLACE last-wins | unit test pull.rs |
| backup excludes | tar listing lacks bridge/ | tests/hardening.rs |
| import --force | lived-in db refuses clobber | tests/import_expect.rs |
| open-kind render | nav_menu_item slug → 404 | tests/exposure.rs |
| plugin min_role | editor save-option 200 / settings 403 | tests/hardening.rs |
| front-page .md | `/{front}.md` reaches `/.md`? decide + test | tests/exposure.rs |
| css_scope | non-ASCII survives; @import per decision | unit test css_scope |

### 6.2 `scripts/check-layers.sh` — committed invariant checker

```sh
# fails if: raw SQL outside src/db (except schema.rs/migrations),
# crate::admin::plugins referenced outside src/plugins adapters,
# serve fns referenced but defined in admin/mod.rs.
```

Wire into `visual_smoke.sh` or document as the audit gate — it runs
in <1s, no excuse to skip.

### 6.3 Doc resync — `docs/architecture.md`, `README.md`,
`CMS-OPERATIONS.md`

- architecture.md: layer table = real post-refactor graph; remove the
  "no domain literals" overclaims that audit disproved or restate them
  accurately; fix `/api/v1/posts` description (returns `{"count":N}`).
- README: sandbox claim gains "macOS Seatbelt; other hosts
  unconfined — see phase-02 decision" honesty; backup section notes
  live-token content.
- CMS-OPERATIONS.md: add the `--force` requirement for re-import on
  lived-in DBs.

### 6.4 Full-site end-to-end proof

```sh
sh scripts/e2e-site.sh sites/aucoeur-test.env all
sh scripts/e2e-site.sh sites/blueskies.env all   # second theme family
sh scripts/visual_smoke.sh                        # all sites
cargo test                                        # full suite
```

Plus a live `rpress site serve` + browser pass on: posts list with a
`?tax=` filter, tour edit (editor role), a scheduled post flipping,
`GET /<slug>.md` burst → 429.

## Acceptance

- All gates above pass in one clean run.
- `git status` review: no leftover scaffolding; phase reports note
  any deliberate deferrals (with reason) in plan.md status table.
