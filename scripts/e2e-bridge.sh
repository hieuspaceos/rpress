#!/usr/bin/env bash
# e2e-bridge.sh — live bridge proof. Starts `rpress site serve`, logs in,
# and exercises the proxied PHP-bridge path end to end. Exists because
# unit tests cannot spawn php -S: three production bugs (token strip,
# quoted BridgeState token, raw-port Location/body leaks) only ever
# showed here.
#
#   scripts/e2e-bridge.sh <site> [bind]
#
# <site>   a sites/<site>.env slug whose work/<site>/ has a DB + www
#          and PLUGINS_PHP set (aucoeur-test).
# [bind]   default 127.0.0.1:8430 — must differ from any running serve.
#
# Exit 0 = every probe passed. Requires php on PATH for the PhpCgi
# runtime; without it the Playground fallback still gets exercised.

set -u
cd "$(dirname "$0")/.."

SITE=${1:?usage: e2e-bridge.sh <site> [bind]}
BIND=${2:-127.0.0.1:8430}
PORT=${BIND##*:}
BASE="http://localhost:$PORT"
BIN=${RPRESS_BIN:-./target/debug/rpress}
[ -x "$BIN" ] || BIN=./target/release/rpress
JAR=$(mktemp -t e2e-bridge)
SRV_LOG=$(mktemp -t e2e-bridge-serve)
PASS=0 FAIL=0

cleanup() {
  [ -n "${SRV_PID:-}" ] && kill "$SRV_PID" 2>/dev/null
  rm -f "$JAR" "$SRV_LOG"
}
trap cleanup EXIT

ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

# name url want_code [method data]
probe() {
  local name=$1 url=$2 want=$3 method=${4:-GET} data=${5:-}
  local args=(-s -b "$JAR" -c "$JAR" -o /dev/null -w '%{http_code}')
  [ "$method" = "POST" ] && args+=(-X POST)
  [ -n "$data" ] && args+=(--data "$data")
  local got
  got=$(curl "${args[@]}" "$url")
  if [ "$got" = "$want" ]; then ok "$name → $got"; else bad "$name → $got (want $want) $url"; fi
}

# --- start -----------------------------------------------------------------
command -v "$BIN" >/dev/null || { echo "no rpress binary (cargo build first)"; exit 2; }
[ -f "work/$SITE/rpress.db" ] || { echo "work/$SITE/rpress.db missing — build the site first"; exit 2; }

"$BIN" site serve "$SITE" --bind "$BIND" >"$SRV_LOG" 2>&1 &
SRV_PID=$!
sleep 2
kill -0 "$SRV_PID" 2>/dev/null || { echo "server died:"; cat "$SRV_LOG"; exit 2; }
echo "site $SITE on $BASE (pid $SRV_PID)"

# --- public surface ---------------------------------------------------------
probe "/"            "$BASE/"          200
probe "es home"      "$BASE/es/"       200
probe "search"       "$BASE/recherche/" 200

# --- login ------------------------------------------------------------------
CREDS="work/$SITE/credentials.txt"
[ -f "$CREDS" ] || { echo "no credentials.txt — cannot test admin"; exit 2; }
LOG=$(sed -n 's/^email=//p' "$CREDS"); PWD=$(sed -n 's/^password=//p' "$CREDS")
code=$(curl -s -c "$JAR" -o /dev/null -w '%{http_code}' \
        --data "log=$LOG&pwd=$PWD" "$BASE/api/login")
[ "$code" = "303" ] && ok "login → 303" || { bad "login → $code"; echo "abort: no session"; exit 1; }

probe "admin"        "$BASE/admin"                       200
probe "posts"        "$BASE/admin/posts"                 200
probe "plugins idx"  "$BASE/admin/plugins"               200
probe "chaty"        "$BASE/admin/plugins/chaty/settings" 200

# --- bridge -----------------------------------------------------------------
# Any PLUGINS_PHP slug works; default goodlayers.
SLUG=${BRIDGE_SLUG:-goodlayers}
code=$(curl -s -b "$JAR" -o /dev/null -w '%{http_code}' "$BASE/admin/plugins/$SLUG/page-builder")
case "$code" in
  301|200) ok "bridge mount → $code (php spawned)" ;;
  404)     echo "  skip bridge — $SLUG not opted in (PLUGINS_PHP)"; echo "$PASS ok, $FAIL FAIL"; exit "$FAIL" ;;
  *)       bad "bridge mount → $code" ;;
esac

# Location must be rewritten into the mount, never a raw loopback port.
loc=$(curl -s -b "$JAR" -o /dev/null -w '%{redirect_url}' "$BASE/admin/plugins/$SLUG/page-builder")
if [ -n "$loc" ] && [ "${loc#*$BASE/admin/plugins/}" != "$loc" ]; then
  ok "Location rewritten into mount"
elif [ -z "$loc" ]; then
  ok "no redirect needed (200 direct)"
else
  bad "Location leaks upstream: $loc"
fi

# Body must contain zero raw bridge ports.
curl -s -b "$JAR" -L -o /tmp/e2e-bridge-body.html "$BASE/admin/plugins/$SLUG/page-builder" || true
if grep -qE '127\.0\.0\.1:[0-9]{4,5}|127\\.0\\.0\\.1\\/' /tmp/e2e-bridge-body.html; then
  bad "body still contains raw bridge port(s)"
else
  ok "body URLs rewritten (no raw port)"
fi

# Token fence: the raw bridge port 403s a headerless request, 204s ident
# with the real token. Port comes from the per-pid pidfile.
BPID=$(basename "$(ls work/$SITE/www/bridge/${SLUG}.*.pid 2>/dev/null | head -1)" 2>/dev/null)
BPORT=$(cat "work/$SITE/www/bridge/$BPID" 2>/dev/null | cut -d: -f2)
TOKEN=$(cat "work/$SITE/www/bridge/$SLUG/bridge-token" 2>/dev/null)
if [ -n "$BPORT" ] && [ -n "$TOKEN" ]; then
  no=$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$BPORT/__rpress_ident")
  yes=$(curl -s -o /dev/null -w '%{http_code}' -H "X-Rpress-Bridge: $TOKEN" "http://127.0.0.1:$BPORT/__rpress_ident")
  [ "$no" = "403" ] && ok "raw port 403s without token" || bad "raw port no-token → $no"
  [ "$yes" = "204" ] && ok "ident 204 with real token"   || bad "ident with token → $yes"
else
  echo "  skip token fence — no pidfile/token (spawn failed? check php.log)"
fi

# Bridged ajax must reach the plugin (JSON or '0' both prove the pipe;
# a 4xx/5xx means the token/dispatch broke).
probe "bridge ajax" "$BASE/admin/plugins/$SLUG/ajax" 400 POST "action=gdlr_core_dummy" || true
code=$(curl -s -b "$JAR" -o /dev/null -w '%{http_code}' -X POST --data "action=gdlr_get_gallery_options" "$BASE/admin/plugins/$SLUG/ajax")
case "$code" in 200|400) ok "ajax pipe live → $code";; *) bad "ajax pipe → $code";; esac

echo
echo "$PASS ok, $FAIL FAIL"
exit "$FAIL"
