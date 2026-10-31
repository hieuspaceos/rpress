#!/usr/bin/env bash
# Requires bash process substitution: under `sh` (POSIX-mode bash on
# macOS sets BASH_VERSION but disables `< <(…)`) probe for the feature
# and re-exec real bash so `sh scripts/check-layers.sh` works instead of
# dying mid-file with a syntax error.
if ! ( eval 'true < <(true)' ) 2>/dev/null; then exec bash "$0" "$@"; fi
# Layering gate — run in CI or pre-commit. Fails when new code violates the
# architecture contract in docs/architecture.md:
#   1. No SQL literal outside src/db/ (exceptions: src/plugins/, import.rs,
#      sqldump.rs, schema.rs probes, tests/).
#   2. No `crate::admin::plugins` / `admin::plugins::` paths — the registry
#      kernel lives at src/plugins/ since phase-05.
#   3. `sql!("name")` names must resolve to src/db/sql/<name>.sql.
# Exit 0 = clean; prints each violation then exits 1.
set -u
cd "$(dirname "$0")/.."
fail=0

# --- 1. inline SQL literals outside db/ -------------------------------------
while IFS= read -r hit; do
  file=${hit%%:*}
  case "$file" in
    # site_deploy: table AND column are format vars — no static base.
    src/db/*|src/plugins/*|src/import.rs|src/sqldump.rs|src/schema.rs|src/site_deploy.rs|tests/*) continue ;;
  esac
  # at statement start inside a string.
  if echo "$hit" | grep -qE '"(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|REPLACE INTO|PRAGMA) '; then
    echo "INLINE-SQL $hit"
    fail=1
  fi
done < <(grep -rn --include='*.rs' -E '"(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|REPLACE INTO|PRAGMA) ' src/ tests/ 2>/dev/null | grep -v '\.contains("')

# --- 2. stale admin::plugins paths ------------------------------------------
while IFS= read -r hit; do
  echo "STALE-PATH $hit"
  fail=1
done < <(grep -rn --include='*.rs' -E 'crate::admin::plugins|admin::plugins::' src/ tests/ 2>/dev/null)

# --- 3. sql!("name") resolves -----------------------------------------------
missing=$(grep -rhoE 'sql!\("[a-z0-9_]+"\)' src/ tests/ 2>/dev/null \
  | sed -E 's/sql!\("([a-z0-9_]+)"\)/\1/' | sort -u \
  | while read -r n; do [ -f "src/db/sql/$n.sql" ] || echo "$n"; done)
if [ -n "$missing" ]; then
  echo "$missing" | while read -r n; do echo "MISSING-SQL-FILE $n"; done
  fail=1
fi

# --- 4. site/domain literals (the "site-agnostic binary" gate) --------------
# Curated site names + site-specific taxonomy slugs must not appear in src/
# outside exempted paths (verbatim-WP-port modules, the importer's seed
# data), test code (everything after a file's first `#[cfg(test)]`), comment-
# only lines, or lines marked `// gate-allow: <reason>`.
# Add a site name to SITE_WORDS when a new reference site lands.
SITE_WORDS='traveltour|tevily|aucoeur|duree|duracion|region-es|estilo|perfil|blueskies|imagetravel'
while IFS= read -r hit; do
  echo "SITE-LITERAL $hit"
  fail=1
done < <(grep -rn --include='*.rs' -E "$SITE_WORDS" src/ 2>/dev/null \
  | grep -vE '^src/(import\.rs|plugins/(theme_traveltour|goodlayers)/)' \
  | grep -v 'gate-allow' \
  | grep -vE ':[0-9]+:[[:space:]]*(//|\*|/\*)' \
  | awk -F: '
      # pass 1 (real files): lowest #[cfg(test)]/mod tests line per file —
      # hits at/after it are test code and exempt.
      FILENAME != "-" {
        if ($0 ~ /^[[:space:]]*#\[cfg\(test\)\]|^[[:space:]]*mod[[:space:]]+tests/ && (!(FILENAME in t) || FNR < t[FILENAME])) t[FILENAME] = FNR;
        next
      }
      { file=$1; line=$2; if (!(file in t) || line+0 < t[file]+0) print }' \
  $(grep -rlE --include='*.rs' '#\[cfg\(test\)\]|mod tests' src/ 2>/dev/null) -)

# --- 5. language literals on the live path ----------------------------------
# fr/es/de (and /xx/ prefixes) belong to lang config data, not code: only
# lang.rs internals, test code, comments, and gate-allow lines may carry them.
while IFS= read -r hit; do
  echo "LANG-LITERAL $hit"
  fail=1
done < <(grep -rn --include='*.rs' -E '"fr"|"es"|"de"|"/es/"|"/fr/"|"/de/"' \
  src/admin/ src/render.rs src/render_post.rs src/render_archive.rs src/render/ \
  src/site_build.rs src/site_serve.rs src/site_deploy.rs src/site_init.rs 2>/dev/null \
  | grep -v 'gate-allow' \
  | grep -vE ':[0-9]+:[[:space:]]*(//|\*|/\*)' \
  | awk -F: '
      FILENAME != "-" {
        if ($0 ~ /^[[:space:]]*#\[cfg\(test\)\]|^[[:space:]]*mod[[:space:]]+tests/ && (!(FILENAME in t) || FNR < t[FILENAME])) t[FILENAME] = FNR;
        next
      }
      { file=$1; line=$2; if (!(file in t) || line+0 < t[file]+0) print }' \
  $(grep -rlE --include='*.rs' '#\[cfg\(test\)\]|mod tests' src/ 2>/dev/null) -)

# --- 6. blocking DB/argon2 calls on tokio threads (src/admin, src/plugins) ---
# rusqlite + argon2 are synchronous — run inline on an async worker they stall
# the whole runtime (audit-261031 §2.1 C1). Every DB-open / db-helper call /
# argon2 call in src/admin and src/plugins must sit inside a
# `tokio::task::spawn_blocking` closure (plugin adapters run on the same
# tokio workers — tourmaster save-option was the live hole this covers).
# Exempt: lines marked `// gate-allow: <reason>`, a whole fn whose
# signature line carries gate-allow (sync helpers whose callers all wrap),
# test code after the file's first `#[cfg(test)]`/`mod tests`, and the
# `spawn_blocking` call line itself. The awk below tracks spawn ranges by
# counting braces on a string/comment-stripped copy of each line.
while IFS= read -r hit; do
  echo "BLOCKING-ON-ASYNC $hit"
  fail=1
done < <(awk '
  BEGIN { SQ = sprintf("%c", 39) }
  FNR == 1 { depth = 0; pending = 0; testcode = 0; inraw = 0; rawn = 0; allowfn = 0 }
  {
    line = $0
    # strings/comments stripped copy — braces inside literals must not
    # skew the spawn_blocking depth tracking.
    s = line; out = ""; n = length(s); i = 1
    while (i <= n) {
      c = substr(s, i, 1)
      if (inraw) {
        if (c == "\"" && substr(s, i + 1, rawn) == hashes) { inraw = 0; i += rawn + 1 } else { i++ }
        continue
      }
      if (c == "/" && substr(s, i + 1, 1) == "/") break
      if (c == "\"") {
        i++
        while (i <= n) {
          d = substr(s, i, 1)
          if (d == "\\") { i += 2; continue }
          if (d == "\"") { i++; break }
          i++
        }
        continue
      }
      if (c == "r" && match(substr(s, i + 1), /^#+"/)) {
        rawn = RLENGTH - 1; hashes = substr(s, i + 1, rawn); inraw = 1; i += RLENGTH + 1; continue
      }
      if (c == SQ) {
        # char literal (SQ x SQ or SQ \e SQ) → skip; bare lifetime keeps SQ
        if (substr(s, i + 2, 1) == SQ) { i += 3; continue }
        if (substr(s, i + 2, 1) == "\\" && substr(s, i + 4, 1) == SQ) { i += 5; continue }
      }
      out = out c; i++
    }
    if (line ~ /^[ \t]*(pub(\(crate\))? )?(async )?fn /) allowfn = (line ~ /gate-allow/)
    if (!testcode && !allowfn && depth == 0 && line !~ /spawn_blocking/ && line !~ /gate-allow/ &&
        line ~ /(crate::db|crate::auth|crud|udb|subdb|fdb|sdb|crate_auth)::(hash_password|verify_password)[ \t]*\(|(crate::db|crud|udb|subdb|fdb|sdb)(::[A-Za-z_][A-Za-z0-9_]*)+[ \t]*\(|rusqlite::Connection::open[ \t]*\(|dummy_hash[ \t]*\(|server_key[ \t]*\(|crate::lang::config[ \t]*\(/) {
      print FILENAME ":" FNR ":" line
    }
    if (line ~ /^[ \t]*#\[cfg\(test\)\]|^[ \t]*mod[ \t]+tests/) testcode = 1
    armed = (depth == 0 && out ~ /spawn_blocking/)
    opens = gsub(/[{]/, "", out); closes = gsub(/[}]/, "", out)
    if (depth > 0) {
      depth += opens - closes
      if (depth < 0) depth = 0
    } else if (pending || armed) {
      if (opens > 0) { depth = opens - closes; if (depth < 0) depth = 0; pending = 0 }
    }
  }' src/admin/*.rs src/plugins/*.rs src/plugins/*/*.rs)

[ "$fail" = 0 ] && echo "layers OK"
exit "$fail"
