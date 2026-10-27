---
phase: 2
title: "Shell & path injection"
status: done
priority: P1
effort: "5h"
dependencies: []
---

# Phase 2: Shell & path injection

## Goal

Every path/config value that reaches a shell command — local or over ssh —
is either argv-passed or routed through a shared `shell_quote()`; site
slugs are validated; DB passwords stop appearing in remote `ps`.

## Findings covered

- **H1** — local `sh -c` zcat/grep/gunzip with interpolated dump paths
  (`site_init.rs:419-421, 443, 489-491`)
- **H2** — remote shell injection via wp-config values + wp_root
  (`site_init.rs:191-203, 212-216, 276-288`)
- **M1** — `mkdir -p '{dir}'` from `--dest` (`site_deploy.rs:178-184,187-205`)
- **M5** — site slug path traversal + `SITE=` filename trust (`site.rs:16-19`,
  `site_build.rs:72`, `site_init.rs:33`)
- **M6** — `MYSQL_PWD` inline in ssh cmd; wp_creds regex misses `"` defines
  and can partially match trailing expressions
- **NEW (red-team)** — `site build` check phase: `sh -c` grep interpolates
  `settings.asset_origin` (imported, attacker-influenced) unescaped
  (`site_build.rs:582-590`); rsync `{target}:{remote_path}` remote-path
  expansion is env-controlled (low risk, note only)
- **M2** — moved to plan-level open questions (needs user answer)

## Key insights

- `shell_quote()` exists at `src/site_build.rs:419` but is **private** —
  move it to `src/site.rs` (already imported by all three files). Do NOT
  copy it a second time or create a `use crate::site_build` back-import.
- Prefer `Command::new(prog).arg(path)`; for the zcat|head pipeline spawn
  `zcat` directly and read bounded stdout (`.take(n)` + kill child).
- wp-config-derived strings are attacker-influenced data: quote every
  interpolation AND validate parsed creds against `^[A-Za-z0-9_.$-]+$`
  so `DB_USER` = `root';id;#` fails fast instead of injecting.
- `wp_creds` regex extension: accept `'…'` or `"…"` but anchor the whole
  value (`'([^']*)'\s*\)`) and bail on trailing expressions
  (`'a'.'b'` must error, not silently produce `a`).
- Legit `sh -c` uses stay: `command -v {const_name}` (site_build.rs:428,
  site_deploy.rs:242), stream_pull pipeline (site_build.rs:398-399).
  Done-criterion = "zero `sh -c` with interpolated non-constant strings".
- `.sql.gz` decompression contract belongs to phase 3 (flate2 magic-bytes
  in the import reader). Pull keeps copying the file untouched — do not
  add a pull-time gunzip here.

- `sites/*.env` — add clone-provenance comments (Validation: all three
  are confirmed clones/test; keep CLONE_SOURCE_OK, document like
  aucoeur-test.env)

- `src/site_init.rs` — replace 3 `sh -c` sinks (419-421, 443, 489-491)
  with direct `Command`; shell_quote all remote interpolations (wp_root,
  db_host, db_user, db_name, prefix, paths); creds charset validation +
  anchored regex; MYSQL_PWD via stdin or `--defaults-extra-file`
- `src/site.rs` — receive `shell_quote` (moved from site_build); site slug
  `^[a-z0-9][a-z0-9-]*$` on init AND `SiteEnv::load` (SITE= vs filename
  stem mismatch → hard error)
- `src/site_build.rs` — check-phase sink 582-590: drop `sh -c`, grep via
  argv `Command` + Rust-side match on file contents
- `src/site_deploy.rs` — shell_quote remote dir in ssh_mkdir/vps_mkdir;
  audit `join_remote` callers
- `sites/*.env` — deferred: annotate clone provenance AFTER user answers
  the plan-level open question (which sites are real clones)

## Implementation steps

1. Move `shell_quote` to `src/site.rs` as `pub(crate)`; update site_build's
   existing callers.
2. Extract `run_quiet(cmd, args)` + `gz_head(path)` helpers in site_init;
   replace the 3 `sh -c` sinks; kill child after bounded read.
3. Wrap every `{value}` interpolated into remote command strings with
   `shell_quote()`: wp_root, db_host, db_user, db_name, prefix, dump paths,
   dest dirs.
4. `wp_creds`: anchored whole-value regex for `'`/`"`; reject constants/
   concatenations with clear error; post-parse charset validation.
5. MYSQL_PWD: ssh stdin (`read -rs P; MYSQL_PWD="$P" mysqldump …`) or a
   remote temp defaults-file removed after; prefer defaults-extra-file.
6. Site slug validation at arg ingestion + `SiteEnv::load` cross-check.
7. `ssh_mkdir`/`vps_mkdir` quote dir; grep whole repo for remaining
   `format!` inside `.args(["-c"` and fix every sink (incl. site_build
   check phase at 582-590).

## Verification

- `cargo test` — shell_quote round-trip on quote-containing strings; site
  slug cases; wp-config parse cases (single/double quotes, trailing expr).
- `cargo check`; enumerate sinks: `grep -rn 'args(\["-c"' src/` → only
  const/legit uses remain.
- Manual: `rpress site init --source local` on a dir containing
  `x'; touch /tmp/pwned; '.sql.gz` → no `/tmp/pwned`, clean error.

## Todo
- [x] Move shell_quote to site.rs; replace 3 local sh -c sinks
- [x] Quote every remote interpolation + creds charset validation
- [x] Anchored wp_creds regex; MYSQL_PWD off the command line
- [x] Site slug validation at init + load cross-check
- [x] Deploy mkdir quoting + join_remote audit
- [x] check-phase asset_origin sink (site_build.rs:582-590)
- [x] sites/*.env clone-provenance comments

## Risks

- `read -s`/defaults-extra-file availability on remote mysqldump —
  Debian/Ubuntu support defaults-extra-file; fall back to stdin.
- Removing `sh -c` for zcat|head changes SIGPIPE semantics — read child
  stdout bounded and kill.
