---
title: "Audit remediation: 7 phases, all findings fixed"
date: 2026-10-27
summary: "Full-repo audit remediation completed — 2 critical, 15 high, 22 medium findings fixed across 7 phases; 66 tests green; 3-round review on phase 6 caught dedupe keeper + remember-me regressions"
---

# Audit remediation: 7 phases, all findings fixed

## What happened
Executed plans/261027-0310-audit-remediation end-to-end: 7 phases covering public content exposure (draft leak via render_slug_by_id), shell/path injection (sh -c sinks, MYSQL_PWD), import/dump correctness (.sql.gz, sqldump lexer, import_expect parity), tour_base threading, cache/regen consistency (invalidate_dependents funnel), admin/DB integrity (delete cascades, csrf persistence+constant-time, php_serialize hardening, post_meta dedupe), and perf/i18n/structural (site_data cache, spawn_blocking, N-lang flags, render.rs split 3743→1168+5 modules).

## Key decisions
- post_meta dedupe keeps the covering-index-first row (MIN(meta_value)), NOT MIN(rowid) — verified on real dbs that LIMIT 1 was serving the index-first row; wrong keeper would have silently swapped ~65% of dup values.
- site_data cache landed only after measurement showed 54.6% of render time; nav_map (1.8µs) deliberately uncached — data-gated, not speculative.
- php_serialize MAX_DEPTH=256, not the suggested 512 — 512 frames measured to overflow the 2MiB test stack.
- Unique post_meta(post_id,meta_key) accepted as contract break: WP multi-value metas collapse to MIN(value); loss is counted+reported via rep.dropped_meta, documented in CMS-OPERATIONS.md.
- render.rs split done last as pure moves, re-exports preserve crate::render::* surface.

## Verification
66 tests green (lib 26 + 7 integration suites incl. new php_serialize/schema/cattest), cargo check 0 warnings, smoke: 7/7 routes on blueskies, release binary 12.4MB (-32.4%).

## Next steps
Native additions planned (not in this commit's code): mail subsystem (providers+newsletter+anti-spam), engagement widgets (popup/FAB/chat), core extras (FTS5 search, TOC, redirects UI), scoped plugin seam (PluginAdapter + PhpBridge for gdlr).

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
