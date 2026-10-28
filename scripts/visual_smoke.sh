#!/usr/bin/env bash
# visual_smoke.sh — per-site structural render gate (plan 261028 phase 05).
#
# For each work/<site> dir: `rpress render slug` a small page set and
# grep the emitted HTML for site-correct chrome: own menus, own
# site_name, no foreign-domain chrome, no unresolved jinja. Aucoeur
# additionally diffs its nav + footer against the captured baseline
# (tests/fixtures/aucoeur-*-baseline.txt) — byte-parity gate on the
# only site whose pre-change output is trusted.
#
# Usage: ./scripts/visual_smoke.sh [site ...]   (default: all four)
# Exit:  non-zero on first batch of misses (each failure is printed).

set -u
cd "$(dirname "$0")/.."
BIN=./target/debug/rpress
[ -x "$BIN" ] || BIN=rpress
TEMPLATES=templates
FAIL=0
chk() { # name actual expected
  if [ "$2" = "$3" ]; then echo "  ok  $1"; else echo "  FAIL $1 (got [$2], want [$3])"; FAIL=1; fi
}

render() { # db slug url outfile
  "$BIN" render slug --db "$1" --templates "$TEMPLATES" --out "$4" --site-url "$3" "$2" >/dev/null 2>&1
}

nav_labels() { # extract <a> labels inside main menu ul (theme-agnostic)
  python3 - "$1" <<'PY'
import re,sys
s=open(sys.argv[1],encoding='utf-8',errors='replace').read()
m=re.search(r'gva-main-menu|sf-menu',s)
if not m: sys.exit()
seg=s[m.start():m.start()+30000]
items=re.findall(r'>([^<]{2,60})</a>',seg)
seen=[]
for t in items:
    t=t.strip()
    if t and t not in seen: seen.append(t)
print('\n'.join(seen[:15]))
PY
}

run_site() {
  local site="$1" url="$2"; shift 2
  local db="work/$site/rpress.db"
  [ -f "$db" ] || { echo "skip $site (no db)"; return; }
  echo "== $site"
  for spec in "$@"; do
    slug="${spec%%:*}"; label="${spec#*:}"
    out="/tmp/vsmoke-$site-${slug//\//_}.html"
    render "$db" "$slug" "$url" "$out"
    [ -s "$out" ] || { echo "  FAIL render $slug"; FAIL=1; continue; }
    chk "$label: no-jinja" "$(grep -c '{{' "$out")" 0
    chk "$label: no-aucoeur" "$(grep -c 'aucoeurvietnam\|Au Coeur\|wp-theme-traveltour' "$out")" 0
  done
}

# ---- aucoeur-test: strict baseline ----
if [ $# -eq 0 ] || [[ " $* " == *" aucoeur-test "* ]]; then
  site=aucoeur-test; db=work/$site/rpress.db; url=http://localhost:8400
  echo "== $site (baseline gate)"
  out=/tmp/vsmoke-auc-home.html
  render "$db" homepage "$url" "$out"
  if [ -s "$out" ]; then
    nav_labels "$out" > /tmp/vsmoke-auc-nav.txt
    python3 - <<'PY' > /tmp/vsmoke-auc-navlabels.txt
import re
s=open('/tmp/vsmoke-auc-home.html',encoding='utf-8',errors='replace').read()
m=re.search(r'<ul id="menu-main-menu-1" class="sf-menu">(.*?)</ul>\s*</div>',s,re.S)
for u,t in re.findall(r'<a href="([^"]+)"[^>]*>([^<]+)</a>',m.group(1)):
    print(f"{t.strip()} -> {u}")
PY
    if diff -q /tmp/vsmoke-auc-navlabels.txt tests/fixtures/aucoeur-nav-baseline.txt >/dev/null; then
      echo "  ok  nav baseline identical"
    else
      echo "  FAIL nav baseline drift:"; diff tests/fixtures/aucoeur-nav-baseline.txt /tmp/vsmoke-auc-navlabels.txt | head -10; FAIL=1
    fi
    ft=$(python3 - <<'PY'
import re
s=open('/tmp/vsmoke-auc-home.html',encoding='utf-8',errors='replace').read()
m=re.search(r'<footer.*?</footer>',s,re.S)
print(' '.join(re.sub(r'<[^>]+>',' ',m.group(0)).split()) if m else 'NOFOOTER')
PY
)
    base=$(cat tests/fixtures/aucoeur-footer-baseline.txt)
    chk "footer baseline" "$ft" "$base"
  else
    echo "  FAIL render homepage"; FAIL=1
  fi
fi

# ---- blueskies (tevily) ----
if [ $# -eq 0 ] || [[ " $* " == *" blueskies "* ]]; then
  site=blueskies; db=work/$site/rpress.db; url=http://localhost:8411
  echo "== $site"
  out=/tmp/vsmoke-bsk-shop.html
  render "$db" shop "$url" "$out"
  chk "shop renders" "$([ -s "$out" ] && echo 1)" 1
  n=$(grep -o 'tevily' "$out" | wc -l | tr -d ' '); [ "$n" -gt 20 ] && echo "  ok  tevily markup ($n refs)" || { echo "  FAIL tevily markup ($n)"; FAIL=1; }
  n=$(grep -o 'traveltour' "$out" | wc -l | tr -d ' '); chk "no traveltour" "$n" 0
  n=$(grep -o 'menu-title' "$out" | wc -l | tr -d ' '); [ "$n" -ge 10 ] && echo "  ok  menu items ($n)" || { echo "  FAIL menu items ($n)"; FAIL=1; }
  # trustpilot widget href is intentional external content, not chrome
  n=$(grep -o 'href="[^"]*vinhhyblueskies' "$out" | grep -vc 'trustpilot'); chk "no origin links" "$n" 0
fi

# ---- imagetravel (traveltour-child, vi) ----
if [ $# -eq 0 ] || [[ " $* " == *" imagetravel "* ]]; then
  site=imagetravel; db=work/$site/rpress.db; url=http://localhost:8414
  echo "== $site"
  slug=$(sqlite3 "$db" "SELECT slug FROM posts WHERE status='publish' AND kind='page' LIMIT 1")
  out=/tmp/vsmoke-imt.html
  render "$db" "$slug" "$url" "$out"
  chk "page renders" "$([ -s "$out" ] && echo 1)" 1
  # mailto: footer text is origin-copied content — chrome check looks at
  # <a href= links + classes only.
  n=$(grep -o 'href="[^"]*aucoeur' "$out" | grep -vc 'mailto:'); chk "no aucoeur chrome" "$n" 0
  n=$(grep -o 'menu-item' "$out" | wc -l | tr -d ' '); [ "$n" -gt 5 ] && echo "  ok  nav items ($n)" || { echo "  FAIL nav items ($n)"; FAIL=1; }
fi

echo
[ "$FAIL" = 0 ] && echo "visual_smoke: PASS" || { echo "visual_smoke: FAIL"; exit 1; }
