---
phase: 5
title: "Visual smoke harness + aucoeur regression gate"
status: done
priority: P1
effort: "1d"
dependencies: [1, 2, 3, 4, 6]
---

# Phase 5: visual smoke harness

## Goal

A repeatable check that any site renders with its own chrome. Per site:
generate a fixed page set, assert structural properties (nav items,
site_name, no foreign-domain strings), and capture screenshots for
visual review. Aucoeur gets a strict regression gate — its rendered
output must match today's.

## Findings covered

- Verified visual state (scout): aucoeur home+inner OK today; dmc/
  imagedmc home OK via harvest but ~76% inner pages on aucoeur chrome;
  blueskies fully unstyled (traveltour css 404 under tevily_child);
  imagetravel styled but aucoeur nav/footer on every page.
- Harvest path bypasses templates — its pages must stay byte-identical.

## Test matrix

| Site | Pages | Assert |
|---|---|---|
| aucoeur-test | home FR, one tour, one post, one archive, one ES page | nav <li> sequence identical to pre-change baseline; logo/footer present; `current-*` classes on matching items |
| imagedmc/dmc | home (harvested — byte compare), 2-3 inner templated pages, one archive | inner pages: site menus, dmc footer widgets, zero `aucoeur` strings; harvest page byte-identical |
| blueskies | home EN, one VI page, one inner page | tevily base classes present, `traveltour-body` absent, menu = its DB menus, no css 404s in generated page's link set; a missing `base` ctx var (extends failure) surfaces as a render error here |
| imagetravel | home VI, EN page, inner page | nav = its Main menu labels (VI), footer = its widgets, own logo |

## Mechanism

- `scripts/visual_smoke.sh` (new, small): for each site dir —
  `rpress generate`, then a per-site assert block using grep/xmllint
  on generated HTML. Structural asserts are greps on deterministic
  content (menu label sequence, site_name in <title>/footer, absence
  patterns, `{% extends base %}` resolved — i.e. no raw template-error
  pages). Exit non-zero on any miss.
- Aucoeur baseline: snapshot current generated nav block (home <ul>
  sequence + footer text) into `tests/fixtures/aucoeur-nav-baseline.txt`
  before phases land; smoke diffs it. Footer/sidebar text compare is
  exact; nav compares item sequence + hrefs (tolerate whitespace).
- Screenshots (optional step, needs Playwright or `shot-scraper`):
  serve each site dir, capture 4 pages, save to
  `plans/261028-site-agnostic-chrome/reports/` for eyeball review —
  screenshots are a review aid, not the gate (the gate is structural).
- Harvest invariance: `git`/`sha256` compare on harvested html files
  before/after — must be byte-identical.

## Files to modify

- `scripts/visual_smoke.sh` (new).
- `tests/fixtures/aucoeur-nav-baseline.txt` (new) — captured pre-merge.
- No src/ changes; this phase only adds the harness + docs entry
  (AGENTS.md or docs/ — match repo convention for checklists).

## Implementation steps

1. Capture aucoeur baseline (nav sequence + footer) from current main
   BEFORE phases 01–04 merge.
2. Write the per-site assert blocks.
3. Run full matrix; fix any phase gap it exposes (feeds back to the
   owning phase).
4. Add the checklist to repo docs (where release checks live).

## Verification

```bash
./scripts/visual_smoke.sh            # all-sites structural gate
sha256sum www/imagedmc/**/*.html | diff - baseline   # harvest identical
cargo check && cargo test
```

## Todos
- [ ] aucoeur baseline fixture captured
- [ ] smoke script green on all 4 sites
- [ ] harvest byte-parity confirmed
- [ ] checklist added to repo docs

## Risks

- Baseline capture must happen pre-merge — sequencing note in master
  plan; if phases land first, regenerate baseline from `git stash` of
  the last pre-change commit.
- Screenshot step needs a local serve + capture tool; degrade to
  structural-only if unavailable — asserts carry the gate.
