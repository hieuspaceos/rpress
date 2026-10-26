#!/usr/bin/env bash
# Compare cms render output to captured reference HTML.
# Normalizes whitespace; reports class-set overlap (byte-identical is phase-3).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CMS="${ROOT}/target/release/cms"
DB="${ROOT}/cms.db"
TPL="${ROOT}/templates"
REF="${ROOT}/templates/reference"
OUT="${TMPDIR:-/tmp}/cms-diff"
mkdir -p "$OUT"

norm() { python3 -c "
import sys,re
t=open(sys.argv[1],encoding='utf-8',errors='replace').read()
t=re.sub(r'\s+',' ',t)
print(t)
" "$1"; }

pairs=(
  "homepage|home.html"
  "rappel-gratuit|page-form.html"
  "du-nord-au-sud-vietnam-12-jours|tour.html"
  "cai-be-dans-le-delta-du-mekong|post.html"
  "avis-des-clients|reviews.html"
)

echo "== render vs reference (class overlap) =="
for pair in "${pairs[@]}"; do
  slug="${pair%%|*}"
  file="${pair##*|}"
  "$CMS" render slug "$slug" --db "$DB" --templates "$TPL" --out "$OUT/$file" >/dev/null
  python3 - "$REF/$file" "$OUT/$file" "$slug" <<'PY'
import sys,re
ref,ren,slug=sys.argv[1],sys.argv[2],sys.argv[3]
def classes(p):
    t=open(p,encoding='utf-8',errors='replace').read()
    return set(re.findall(r'gdlr-core-[a-z0-9-]+', t))
a,b=classes(ref),classes(ren)
ov=len(a&b); print(f"{slug:40} ref={len(a):3} rend={len(b):3} overlap={ov:3} ({100*ov/max(len(a),1):.0f}%)")
PY
done
