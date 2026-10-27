---
title: "gdlr cutover regression fixes — bundle plugin, URL strip, probe/mutex, settings backfill"
description: "Post-audit remediation for 413906b: PB ajax dead on blueskies/dmc/imagedmc (no goodlayers-core plugin), stale 127.0.0.1 URLs in defs JSON, blocking probe under global bridges mutex, missing stylesheet/active_plugins on pre-cutover DBs"
status: done
priority: P1
effort: "1-2d"
branch: main
tags: [bugfix, bridge, regression, audit]
created: 2026-10-27
---

# gdlr cutover regression fixes

## Premise

Phase-07 deleted `gdlr_pb.rs` (2410-line Rust port) after the real-site
parity checklist passed on **aucoeur-test only**. Two-persona audit
(code-reviewer + security-reviewer) confirmed the cutover regresses every
site that lacks a mounted `goodlayers-core` plugin dir — the PB editor is
**format-universal** (`body_is_gdlr` matches `element`/`text-box`/… nodes),
not plugin-specific, so blueskies/dmc/imagedmc all use it while carrying no
goodlayers plugin. `/admin/api/gdlr/ajax` now 307s unconditionally into the
bridge, which errors/not-enabled → editor dead.

## Audit findings → phases

| # | Finding | Severity | Phase |
|---|---------|----------|-------|
| C1 | PB ajax dead on sites without mounted goodlayers-core (3/4 sites) | critical | 01 |
| M1 | `elements.json`/`wrappers.json` carry 139 absolute `http://127.0.0.1:64123/…` URLs → broken thumbs | medium | 02 |
| M2 | `bridge_for` blocking 2s `healthy()` probe + ≤30s respawn under global `bridges` Mutex → serialized stall / respawn storm | medium | 03 |
| M3 | Pre-cutover DBs lack `stylesheet`/`active_plugins` settings → bridge boots theme-less (twentytwentyfive), element providers missing | medium | 04 |
| L* | Stale comments/doc drift (`gdlr` slug, old fallback doc, dead `page-builder-var.json`) | low | 01 |

Out of scope (accepted lows, documented in audit): `free_port` TOCTOU +
squatter-token (localhost-only), request-line CRLF via `rest` (admin-authed),
`--define` fail-open (verified working), `rpress_seed_done` ordering
(self-heals on next wp.db lifetime).

## Goal

Every site with gdlr-format `body_json` gets a working PB editor through the
bridge — identical to aucoeur-test — without per-site plugin files.

## Phases

| # | Phase | Depends on | Effort | Status |
|---|-------|-----------|--------|--------|
| 01 | [Bundle goodlayers-core; always-on PB bridge](./phase-01-bundle-plugin.md) | — | 4h | Done |
| 02 | [Strip origin URLs in defs JSON](./phase-02-strip-origin-urls.md) | — | 1h | Done |
| 03 | [Probe outside mutex + respawn rate-limit](./phase-03-probe-mutex.md) | 01 (same file) | 3h | Done |
| 04 | [Bridge constants fallback from dump.sql](./phase-04-dump-backfill.md) | 01 (same file) | 2h | Done |

## Success criteria

- `blueskies` + `dmc` + `imagedmc`: PB ajax POST returns real plugin JSON
  (`{"status":"success",…}`) — verified live via serve+login+POST.
- No `127.0.0.1`/`localhost` host remains reachable through emitted HTML.
- `cargo check` clean; `cargo test` all pass; concurrent PB requests on a
  busy bridge do not trigger respawn.
- Doc/comment drift fixed (`goodlayers` slug, no stale fallback wording).

<!-- slug: gdlr-cutover-regression -->
