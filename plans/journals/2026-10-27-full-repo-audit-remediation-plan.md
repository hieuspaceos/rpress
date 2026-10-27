---
title: Full-repo audit + remediation plan
date: 2026-10-27
summary: 4-agent audit of ~36k LOC found 2 critical/15 high/22 medium; remediation plan red-teamed and validated into 7 phases
---

# Full-repo audit + remediation plan

## What happened

Audited rpress with 4 parallel reviewers (security, admin+db, render, ops/import).
Key findings: drafts publicly served via render__select_by_id (not just
select_id_lang — red team caught the bigger hole), shell injection sinks in
site_init/site_deploy/site_build check(), import_expect undercounts vs insert
filters, hardcoded 'voyage' ~25 sites incl. format strings + widgets.rs,
cache invalidation misses dependents + .md mirror + alias variants,
deploy leaks LOCAL_URL, user/term delete cascades incomplete, php_serialize
OOM, MAX(id)+1 races, blocking DB on tokio workers.

Plan `plans/261027-0310-audit-remediation/` — 7 phases, P1-P3, dependencies
declared. Red team (4 lenses) produced 30 findings, 27 accepted; validation
interview resolved env-clone/bin_mdcheck/page-N/intake decisions.

## Decision

- .sql.gz decompression = flate2 magic-bytes in import reader (single contract).
- invalidate_dependents wired inside regen_posts funnel (covers all callers).
- Deploy rewrite on staged copy only, keyed on {local_url, prev domain, localhost:*}.
- Generation-counter site cache rejected; lazy media_map + env cache, benchmark decides.
- render.rs split deferred to plan's last commit.

## Next steps

/ak:cook plans/261027-0310-audit-remediation/plan.md

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
