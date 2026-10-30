---
title: "Phase 2 — Bridge + deploy trust-boundary fixes"
phase: 2
status: pending
priority: P0
effort: 3h
files:
  - src/admin/bridge/php_cgi.rs
  - src/admin/bridge/pull.rs
  - src/admin/bridge/mod.rs
  - src/admin/mod.rs
  - src/backup.rs
  - src/admin/backup.rs
  - src/site_deploy.rs
---

# Phase 2 — Bridge + deploy trust boundary

The bridge is the biggest trust surface; several of its guards are
weaker than the docs claim. Fix or make failures explicit.

## Tasks

### 2.1 `install_php` escaping — `php_cgi.rs:484-540`

- Route every interpolated value through `php_str()` (the fn
  `wp_config()` already uses): `{fqdb}`, `{docroot}`, `{token}`,
  `{theme}`, `{plugin}`, `{port}`(numeric — validate digits instead).
- `theme`/`plugin` are dump/option-derived — the threat is a crafted
  `'` in the value → code exec in the install step.
- Regression: unit-test `install_php` string with `theme =
  "x');phpinfo();//"` → assert emitted PHP contains the escaped
  literal, no unbalanced quote outside strings.

### 2.2 Linux no-sandbox — make it explicit — `php_cgi.rs:236-267`

- Where `sandbox-exec` is absent: eprintln a loud warning per spawn
  ("bridge PHP is UNSANDBOXED on this host — filesystem and network
  are unrestricted"), and gate unconfined spawn behind env
  `RPRESS_BRIDGE_NO_SANDBOX=1` **unless** an explicit operator flag
  exists. Decide: default = refuse to spawn unconfined on Linux
  (breaks bridge there, loudly) vs warn-and-continue (status quo,
  honest). Pick warn-and-continue with the warning — matching
  `RuntimeKind::Playground` fallback still exists; and record the
  decision + reason in the phase report.
- Fix the misleading comment at :234.

### 2.3 Pull-back hardening — `pull.rs:130-192`, `admin/mod.rs:1734`

- `mod.rs:1734`: `let _ = pull(...)` → log failures
  (`eprintln!("bridge pull failed: {e:#}")`) — edits must never be
  silently lost.
- id resolution: `COALESCE((SELECT id WHERE wp_id=?1), ?1)` → try
  `wp_id` lookup; on miss **skip the row and count skipped** (a
  WP-created post has no rpress row — stamping its meta onto rowid N
  is data corruption). Log skipped count in the pull summary.
- post_meta write: `DELETE` + `INSERT` → `INSERT OR REPLACE` (kills
  the multi-value crash and the delete-then-fail window).
- Regression: fixture wp.db with (a) wp_id not in rpress posts → row
  skipped, no meta on unrelated post; (b) two rows same
  (post_id,meta_key) → pull succeeds, last wins.

### 2.4 Stale-pidfile reaping — `bridge/mod.rs:133-148`

- `ps -o comm=` match → `ps -o args=` and require the child's actual
  argv signature (`-S 127.0.0.1:<port>` or the docroot path) before
  kill. A PID recycled into an unrelated php/node process must not be
  killed.

### 2.5 Backup: exclude bridge, warn on tokens — `src/backup.rs`,
`src/admin/backup.rs`

- `tar -chzf` stays `-h` for `www/wp-content` (real files wanted)
  but stage excludes `www/bridge/**` entirely — the bridge docroot is
  rebuildable state (spawn regenerates it), never backup material.
- Admin backup page gains a visible warning: "archive contains live
  session tokens, app passwords, and pending TOTP secrets — treat as
  a credential file" (the info is true whether or not we exclude
  bridge).
- `backup.rs:16` `work.join("rpress.db")` → derive from the site's
  configured `--db` path (thread `sc.db` through instead of assuming
  the filename).
- Regression: backup archive listing contains no `bridge/` entries;
  `--db custom.db` site backs up `custom.db` content.

### 2.6 Deploy `-wal` consistency — `src/site_deploy.rs:239-246,391-403`

- `std::fs::copy(rpress.db, stagedb)` → `VACUUM INTO stagedb` from a
  read connection (consistent snapshot, folds WAL in).
- `deploy_vps` ships `target/release/rpress` — add an OS/arch sanity:
  if `cfg!(target_os)` of the built binary doesn't match a plausible
  remote (check `file` output or read ELF/Mach-O magic), warn loudly +
  require `--force` to proceed.
- Regression: stage a db with dirty WAL (write, no checkpoint) →
  staged db contains the rows.

## Acceptance

- `cargo test` green incl. new pull + env tests.
- Manual: `rpress bridge` kill/spawn cycle still works; a crafted
  theme string can't break `install_php` (test proves).
- Backup tarball inspected: no `bridge/`, warning visible on admin
  page.
