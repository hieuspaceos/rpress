#!/usr/bin/env bash
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

[ "$fail" = 0 ] && echo "layers OK"
exit "$fail"
