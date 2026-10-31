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

[ "$fail" = 0 ] && echo "layers OK"
exit "$fail"
