---
title: "audit-1031 remediation — 8 phases, 6 parallel subagents, all findings closed"
date: 2026-10-31
summary: "Post-audit remediation of plans/reports/audit-2026-10-31.md: 16 findings closed — bridge fail-closed, atomic import, unixepoch sessions, save_term txn, UTC published_at, magic-byte gate, spec __present, taxonomy roots, pull lifecycle. 133/133 tests, commit eac0d94."
---

# audit-1031 remediation — 8 phases, 6 parallel subagents, all findings closed

# audit-1031 remediation

## What happened

A 4-agent audit was distilled into `plans/reports/audit-2026-10-31.md`,
then executed as `plans/261031-0450-audit-1031-remediation/` — an
8-phase plan run via 6 parallel subagents plus inline work. Committed
as eac0d94 (54 files, +2416/-368).

Key fixes landed:

- `bridge.php` token fence flipped to **fail-open→fail-closed** on
  missing `RPRESS_BRIDGE_TOKEN`; unsandboxed php spawn is now
  fail-closed behind `RPRESS_BRIDGE_UNSAFE`.
- Atomic import: stage to temp dir + rename; `pull()`/pidfile lifecycle
  cleaned up.
- Session sweep migrated to `unixepoch()`; `save_term` wrapped in a
  transaction; `published_at` normalized to UTC.
- Sibling cache invalidation; `flatten()` sweep; magic-byte gate on
  uploads; tourmaster panic guards; spec `__present` markers; a single
  `esc()` inside `html_decode`; N-language taxonomy roots.

## The brutal truth

The audit "fixed" `search_months` and marked the finding withdrawn —
then the regression test caught it red-handed: cumulative year drift
was still real. We reinstated the finding and rewrote the math to an
absolute month index. Embarrassing that a finding got withdrawn on a
plausibility argument; the test was the only honest arbiter.

macOS `/bin/sh` is POSIX-mode bash: `BASH_VERSION` is set but process
substitution is disabled. Our first `check-layers.sh` guard trusted
`BASH_VERSION` and silently ran the bash-only path under sh. Now it
feature-probes instead of sniffing variables. If you test "is this
bash," test the *capability*.

## Notable events

1. **Reinstated finding**: `search_months` drift — rewritten to absolute
   month index after the regression test disproved the withdrawal.
2. **sh≠bash**: `check-layers.sh` uses a feature probe for process
   substitution; verified under both `sh` and `bash`.
3. **Reviewer nits fixed inline**: 502 body was leaking the error chain
   to clients; `pull()` had a `locate`-None path that could misbehave.

## Verified

- `cargo test` 133/133.
- `check-layers.sh` passes under both `/bin/sh` and `bash`.
- Code-reviewer pass approved.

## Watchpoints

- `RPRESS_BRIDGE_UNSAFE` is an escape hatch, not a mode — deployments
  that set it re-open the unsandboxed spawn the audit closed.
- `__present` in the spec is load-bearing now; future field additions
  must populate it or the spec gate regresses.

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
