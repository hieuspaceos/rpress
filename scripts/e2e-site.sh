#!/bin/sh
# Site bring-up for rpress. Generic — one env file per site in sites/.
#
#   scripts/e2e-site.sh <site.env> <step>
#
# Steps: pull | import | configure | generate | check | parity | serve | all
#
# `all` = pull + import + configure + generate + check.
# State lives in work/<SITE>/ (db, cache, www, dump, credentials).
set -eu
cd "$(dirname "$0")/.."
ENV_FILE=${1:?usage: e2e-site.sh <site.env> <step>}
ACTION=${2:-all}
# shellcheck disable=SC1090
. "./$ENV_FILE"

: "${SITE:?env missing SITE}"
ROOT=$(pwd)
WORK=$ROOT/work/$SITE
BIN=$ROOT/target/release/rpress
VPS=${VPS_SSH:-$ROOT/../vps-ssh/vps.sh}
DB=$WORK/rpress.db
CACHE=$WORK/cache
WWW=$WORK/www
DUMP=$WORK/dump.sql
TEMPLATES=${TEMPLATES:-$ROOT/templates}
BIND=${BIND:-127.0.0.1:8400}
HOST=${HOST:-*}
NAME=${NAME:-$SITE}
TOUR_BASE=${TOUR_BASE:-voyage}
LOCAL_URL=${LOCAL_URL:-http://localhost:${BIND##*:}}

export PATH="$HOME/.cargo/bin:$PATH"
need() { command -v "$1" >/dev/null || { echo "missing $1" >&2; exit 1; }; }

pull() {
  need rsync
  # pull copies FROM a remote/local clone. Per-site env must opt in with
  # CLONE_SOURCE_OK=yes so this never runs against a production docroot
  # by accident.
  if [ "${CLONE_SOURCE_OK:-no}" != yes ]; then
    echo "refusing: set CLONE_SOURCE_OK=yes in $ENV_FILE (pull reads a CLONE, never prod)" >&2
    exit 1
  fi
  mkdir -p "$WORK"
  if [ -n "${DUMP_LOCAL:-}" ]; then
    cp "$DUMP_LOCAL" "$DUMP"
  elif [ -n "${DUMP_REMOTE:-}" ]; then
    [ -n "${VPS_HOST:?env missing VPS_HOST}" ]
    "$VPS" list >/dev/null
    mkdir -p "$WORK/dump-src"
    "$ROOT/../vps-ssh/.askpass/rsync-pull.sh" "$VPS_HOST" "$DUMP_REMOTE" "$WORK/dump-src/"
    mv "$WORK/dump-src/$(basename "$DUMP_REMOTE")" "$DUMP"; rmdir "$WORK/dump-src"
  else
    echo "env needs DUMP_REMOTE or DUMP_LOCAL" >&2; exit 1
  fi
  test -s "$DUMP"
  head -c 300 "$DUMP" | grep -q -e TABLE -e INSERT -e dump \
    || { echo "dump does not look like SQL" >&2; exit 1; }

  if [ -n "${WP_CONTENT_LOCAL:-}" ]; then
    mkdir -p "$WWW/wp-content"
    for part in uploads themes plugins; do
      [ -d "$WP_CONTENT_LOCAL/$part" ] || continue
      rsync -a "$WP_CONTENT_LOCAL/$part/" "$WWW/wp-content/$part/"
    done
    for part in js css fonts images; do
      [ -d "$WP_CONTENT_LOCAL/../wp-includes/$part" ] || continue
      rsync -a "$WP_CONTENT_LOCAL/../wp-includes/$part/" "$WWW/wp-includes/$part/"
    done
  elif [ -n "${WP_ROOT:-}" ]; then
    for part in uploads themes plugins; do
      "$ROOT/../vps-ssh/.askpass/rsync-pull.sh" "$VPS_HOST" \
        "$WP_ROOT/wp-content/$part/" "$WWW/wp-content/$part/"
    done
    # WP core static assets (jquery, block-editor css, fonts) — referenced
    # by theme markup; serving them from www avoids a remote 307 per page.
    for part in js css fonts images; do
      "$ROOT/../vps-ssh/.askpass/rsync-pull.sh" "$VPS_HOST" \
        "$WP_ROOT/wp-includes/$part/" "$WWW/wp-includes/$part/"
    done
  fi
  # Repo-owned front assets — always present locally, never 307.
  for f in front.css nav.js; do
    [ -f "$ROOT/www/$f" ] && cp "$ROOT/www/$f" "$WWW/$f"
  done
  echo "pull ok: $(du -h "$DUMP" | awk '{print $1}') dump"
}

import() {
  test -x "$BIN" || cargo build --release
  test -s "$DUMP"
  rm -f "$DB" "$DB-shm" "$DB-wal"
  rm -rf "$CACHE"
  "$BIN" import parse --dump "$DUMP" --db "$DB" --report "$WORK/import-report.txt"
  "$BIN" import verify --db "$DB"
}

configure() {
  : "${LOCAL_URL:?}"
  # Origin = whatever siteurl the dump carried. It is kept twice:
  #   asset_origin  → 307 fallback for missing on-disk assets
  #   legacy_origin → render-time rewrite of imported content links
  # Plain-text URL columns the renderer never touches get renamed in place
  # (menus, redirect targets) — the same end state cms.db carries.
  sqlite3 "$DB" "
    INSERT OR REPLACE INTO settings (key, value) VALUES
      ('asset_origin', COALESCE(
        NULLIF((SELECT value FROM settings WHERE key='asset_origin'), ''),
        (SELECT value FROM settings WHERE key='site_url'))),
      ('legacy_origin', COALESCE(
        NULLIF((SELECT value FROM settings WHERE key='legacy_origin'), ''),
        NULLIF((SELECT value FROM settings WHERE key='asset_origin'), ''),
        (SELECT value FROM settings WHERE key='site_url'))),
      ('site_url', '$LOCAL_URL');
    UPDATE menu_items SET url = REPLACE(url, (SELECT value FROM settings WHERE key='legacy_origin'), '$LOCAL_URL')
      WHERE url LIKE '%' || (SELECT value FROM settings WHERE key='legacy_origin') || '%';
    UPDATE redirects SET new_url = REPLACE(new_url, (SELECT value FROM settings WHERE key='legacy_origin'), '$LOCAL_URL')
      WHERE new_url LIKE '%' || (SELECT value FROM settings WHERE key='legacy_origin') || '%';"
  # First admin — migrate parity. Idempotent: only when users table is empty.
  n=$(sqlite3 "$DB" "SELECT count(*) FROM users;")
  if [ "$n" = 0 ]; then
    pass=$(LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 20)
    "$BIN" user add "admin@${SITE}.local" "$pass" --role admin --db "$DB" >/dev/null
    printf 'email=admin@%s.local\npassword=%s\n' "$SITE" "$pass" >"$WORK/credentials.txt"
    chmod 600 "$WORK/credentials.txt"
    echo "admin credentials -> work/$SITE/credentials.txt"
  fi
  sqlite3 "$DB" "SELECT key, substr(value,1,60) FROM settings WHERE key IN ('site_url','asset_origin','site_name','comment_moderation');"
}

generate() {
  "$BIN" generate --db "$DB" --templates "$TEMPLATES" --out "$CACHE" --site-url "$LOCAL_URL"
  [ -f "$CACHE/llms.txt" ] && cp "$CACHE/llms.txt" "$WWW/llms.txt"
}

check() {
  fail=0
  # Sample rendered paths straight from the DB — no per-site hardcoding.
  # Path rule mirrors generate::rel_path: primary lang has no prefix; every
  # other lang writes under /<lang>/; tours add /voyage/.
  front=$(sqlite3 "$DB" "SELECT CAST(value AS INTEGER) FROM settings WHERE key='page_on_front';")
  primary=$(sqlite3 "$DB" "SELECT COALESCE((SELECT value FROM settings WHERE key='primary_lang'),'fr');")
  langs=$(sqlite3 "$DB" "SELECT COALESCE((SELECT value FROM settings WHERE key='languages'), '$primary');")
  while IFS='|' read -r id slug lang kind; do
    rel="$slug"
    [ "$kind" = tour ] && rel="voyage/$slug"
    [ "$lang" != "$primary" ] && rel="$lang/$rel"
    [ "$id" = "$front" ] && rel=""
    f="$CACHE/${rel:+$rel/}index.html"
    if [ -s "$f" ]; then echo "ok   /$rel"; else echo "MISS /$rel"; fail=1; fi
  done <<EOF
$(sqlite3 "$DB" "
    SELECT id, slug, lang, kind FROM posts
    WHERE status='publish' AND kind IN ('page','post','tour')
    ORDER BY id LIMIT 6;")
EOF
  [ "$fail" = 1 ] && return 1
  for u in index.html sitemap.xml robots.txt; do
    [ -s "$CACHE/$u" ] || echo "WARN cache/$u missing"
  done
  OLDIFS=$IFS; IFS=','
  for lang in $langs; do
    [ "$lang" = "$primary" ] && continue
    [ -s "$CACHE/$lang/index.html" ] || echo "WARN cache/$lang/index.html missing"
  done
  IFS=$OLDIFS
  [ -s "$WWW/llms.txt" ] || echo "WARN www/llms.txt missing"
  # Internal links must point at LOCAL_URL after configure, not the origin.
  origin=$(sqlite3 "$DB" "SELECT value FROM settings WHERE key='asset_origin';" 2>/dev/null || true)
  if [ -n "$origin" ] && [ "$origin" != "$LOCAL_URL" ]; then
    leak=$(grep -rl "$origin/" "$CACHE" --include='*.html' 2>/dev/null | head -3 | wc -l | tr -d ' ')
    [ "$leak" -gt 0 ] && echo "WARN $leak+ pages still link $origin (content-level refs)"
  fi
  return 0
}

parity() {
  # Structural diff vs the origin WP site — generic quality gate.
  # Compares gdlr-/tourmaster- class sets on sampled paths (origin vs cache).
  : "${SITE_URL:?env missing SITE_URL (origin for parity)}"
  sqlite3 "$DB" "
    SELECT p.slug, p.lang, p.kind, p.id ==
      CAST(COALESCE((SELECT value FROM settings WHERE key='page_on_front'),'0') AS INTEGER)
    FROM posts p
    WHERE p.status='publish' AND p.kind IN ('page','post','tour')
    ORDER BY p.id LIMIT 8;" > "$WORK/.parity-sample"
  primary=$(sqlite3 "$DB" "SELECT COALESCE((SELECT value FROM settings WHERE key='primary_lang'),'fr');")
  python3 - "$SITE_URL" "$CACHE" "$WORK/.parity-sample" "$primary" <<'PY'
import re, sys, urllib.request, pathlib
origin, cache, primary = sys.argv[1].rstrip('/'), pathlib.Path(sys.argv[2]), sys.argv[4]
paths = ['/']
for line in pathlib.Path(sys.argv[3]).read_text().splitlines():
    slug, lang, kind, is_front = (line.split('|') + [''] * 4)[:4]
    if is_front == '1': continue  # front page is '/' already
    rel = f"{kind == 'tour' and 'voyage/' or ''}{slug}"
    if lang != primary: rel = f"{lang}/{rel}"
    paths.append('/' + rel + '/')
def sig(t):
    return set(re.findall(r'(?:gdlr-core|tourmaster|traveltour)[a-z0-9-]*', t))
for p in paths:
    f = cache / p.strip('/') / 'index.html'
    if not f.exists(): f = cache / 'index.html' if p == '/' else f
    if not f.exists(): print(f"{p:44} no local file"); continue
    try:
        req = urllib.request.Request(origin + p, headers={'User-Agent': 'Mozilla/5.0'})
        live = urllib.request.urlopen(req, timeout=20).read().decode('utf-8', 'replace')
    except Exception as e:
        print(f"{p:44} origin fetch failed: {e}"); continue
    a, b = sig(live), sig(f.read_text(errors='replace'))
    mark = 'OK' if len(a & b) / max(len(a), 1) > 0.8 else 'DIFF'
    print(f"{mark} {p:40} origin={len(a):3} local={len(b):3} overlap={len(a&b):3}")
PY
}

serve() {
  echo "admin http://${BIND}/admin"
  exec "$BIN" serve --bind "$BIND" --db "$DB" --templates "$TEMPLATES" \
    --out "$CACHE" --www "$WWW" --host "$HOST" --site "$SITE" \
    --name "$NAME" --tour-base "$TOUR_BASE"
}

case $ACTION in
  pull) pull ;; import) import ;; configure) configure ;;
  generate) generate ;; check) check ;; parity) parity ;; serve) serve ;;
  all) pull; import; configure; generate; check ;;
  *) echo "usage: $0 <env> pull|import|configure|generate|check|parity|serve|all" >&2; exit 2 ;;
esac
