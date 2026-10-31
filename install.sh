#!/usr/bin/env bash
# rpress installer — one command on a fresh Linux VPS:
#   curl -fsSL https://raw.githubusercontent.com/hieuspaceos/rpress/main/install.sh | bash
# Or clone the repo and run ./install.sh from the repo root.
#
# What it does, in order (every step is skippable/interactive):
#   1. Detect OS/arch; install ONLY what's missing (curl/rsync; php-cli +
#      php-sqlite3 + bubblewrap if the site will use the PHP bridge).
#   2. Put `rpress` in /usr/local/bin — prefer a published Linux binary from
#      GitHub Releases; fall back to building from source (Rust installed via
#      rustup when absent).
#   3. `rpress site init <slug>` — import the WP dump / clone via ssh.
#   4. Write a systemd unit running `rpress site serve <slug>` and start it.
#   5. Print the nginx proxy block + admin URL. nginx itself is optional:
#      the site binds 127.0.0.1 by default, so nothing is exposed until a
#      reverse proxy (nginx, caddy, Cloudflare Tunnel…) points at it.
#
# Env overrides (non-interactive use):
#   RPRESS_SITE=slug          skip the slug prompt
#   RPRESS_NO_SERVICE=1       skip the systemd unit (bare install)
#   RPRESS_RELEASE_REPO=x/y  GitHub repo to fetch the binary from
set -euo pipefail

RPRESS_RELEASE_REPO="${RPRESS_RELEASE_REPO:-hieuspaceos/rpress}"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)  TARGET="x86_64-unknown-linux-gnu" ;;
  aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
  *) echo "unsupported arch: $ARCH (need x86_64 or aarch64 Linux)"; exit 1 ;;
esac
[ "$(uname -s)" = "Linux" ] || { echo "this installer targets Linux; on macOS just \`cargo build --release\`"; exit 1; }

say()  { printf '\033[1m== %s\033[0m\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }
as_root() { if [ "$(id -u)" = 0 ]; then "$@"; else sudo "$@"; fi }

# --- 1. prerequisites -------------------------------------------------------
say "checking prerequisites"
need_pkgs=()
have curl || need_pkgs+=(curl)
have rsync || need_pkgs+=(rsync)
if [ ${#need_pkgs[@]} -gt 0 ]; then
  if have apt-get; then as_root apt-get update -qq && as_root apt-get install -y "${need_pkgs[@]}"
  elif have dnf; then as_root dnf install -y "${need_pkgs[@]}"
  else echo "please install: ${need_pkgs[*]}"; exit 1; fi
fi

# --- 2. the binary ----------------------------------------------------------
BIN=""
if have rpress; then
  BIN="$(command -v rpress)"; say "rpress already installed: $BIN"
else
  # Prefer the published release asset.
  REL="https://github.com/${RPRESS_RELEASE_REPO}/releases/latest/download/rpress-${TARGET}.tar.gz"
  say "trying release binary: $REL"
  if tmp="$(mktemp -d)" && curl -fsSL "$REL" -o "$tmp/rpress.tar.gz" 2>/dev/null; then
    tar -xzf "$tmp/rpress.tar.gz" -C "$tmp"
    as_root install -m 755 "$tmp/rpress" /usr/local/bin/rpress
    BIN=/usr/local/bin/rpress
  else
    say "no release binary — building from source (needs Rust + repo clone)"
    if ! have cargo; then
      curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs | sh -s -- -y --minimal-profile
      # shellcheck disable=SC1091
      . "$HOME/.cargo/env"
    fi
    # Locate the repo: either we're inside it, or clone.
    if [ -f "$(dirname "$0")/../Cargo.toml" ] && grep -q 'name = "rpress"' "$(dirname "$0")/../Cargo.toml" 2>/dev/null; then
      SRC="$(cd "$(dirname "$0")/.." && pwd)"
    else
      SRC="${RPRESS_SRC:-$HOME/rpress-src}"
      [ -d "$SRC/.git" ] || git clone "${RPRESS_GIT_REPO:-https://github.com/${RPRESS_RELEASE_REPO}}" "$SRC"
      SRC="$SRC/rpress"
    fi
    cargo build --release --manifest-path "$SRC/Cargo.toml"
    as_root install -m 755 "$SRC/target/release/rpress" /usr/local/bin/rpress
    BIN=/usr/local/bin/rpress
  fi
fi
"$BIN" --version >/dev/null && say "rpress installed: $BIN"

# --- 3. the site ------------------------------------------------------------
SITE="${RPRESS_SITE:-}"
if [ -z "$SITE" ]; then
  if [ -t 0 ]; then read -rp "site slug [mysite]: " SITE; SITE="${SITE:-mysite}"
  else SITE="mysite"; fi
fi
ROOT="$(pwd)"
mkdir -p "$ROOT/sites" "$ROOT/work"

if [ ! -f "$ROOT/sites/$SITE.env" ]; then
  say "running \`rpress site init $SITE\` — follow the prompts (dump path or ssh clone)"
  (cd "$ROOT" && "$BIN" site init "$SITE")
else
  say "sites/$SITE.env already exists — skipping init"
fi

# --- 4. systemd service ------------------------------------------------------
if [ "${RPRESS_NO_SERVICE:-0}" = "1" ] || ! have systemctl; then
  say "skipping systemd unit (RPRESS_NO_SERVICE=$RPRESS_NO_SERVICE or no systemd)"
  echo "run manually: cd $ROOT && $BIN site serve $SITE"
  exit 0
fi

UNIT="/etc/systemd/system/rpress-${SITE}.service"
say "writing systemd unit $UNIT"
# Loopback-only egress confines the PHP bridge (it can't be netns-isolated
# without root; rpress itself needs no outbound traffic). Only safe when
# the site binds loopback — a public BIND would also lose INBOUND
# connections, so emit the deny lines only for loopback binds.
BIND_ENV=$(grep -E '^BIND=' "$ROOT/sites/$SITE.env" 2>/dev/null | cut -d= -f2- || true)
EGRESS=""
case "${BIND_ENV:-127.0.0.1}" in
  127.0.0.1*|localhost*|*\[::1\]*)
    EGRESS="IPAddressAllow=localhost
IPAddressDeny=any" ;;
  *) say "non-loopback BIND='$BIND_ENV' — skipping IPAddressAllow/Deny hardening" ;;
esac
as_root tee "$UNIT" >/dev/null <<EOF
[Unit]
Description=rpress ${SITE} (WordPress-replacement static site)
After=network.target

[Service]
Type=simple
WorkingDirectory=${ROOT}
ExecStart=${BIN} site serve ${SITE}
Restart=always
RestartSec=3
# Hardening (the PHP bridge also runs inside this unit — keep it tight):
ProtectSystem=strict
ReadWritePaths=${ROOT}/work
ProtectHome=yes
PrivateTmp=yes
NoNewPrivileges=yes
RestrictSUIDSGID=yes
LockPersonality=yes
${EGRESS}

[Install]
WantedBy=multi-user.target
EOF
as_root systemctl daemon-reload
as_root systemctl enable --now "rpress-${SITE}.service"

# --- 5. done -----------------------------------------------------------------
BIND=$(grep -E '^BIND=' "$ROOT/sites/$SITE.env" | cut -d= -f2- || echo 127.0.0.1:8410)
say "done — rpress serving $SITE on $BIND"
cat <<EOF

Next (pick one):

  • nginx reverse proxy:
      server {
        listen 80;
        server_name your-domain.com;
        location / { proxy_pass http://${BIND};
                     proxy_set_header Host \$host; }
      }

  • admin UI:    http://${BIND}/admin
  • logs:        journalctl -u rpress-${SITE} -f
  • backup:      ${BIN} site backup ${SITE}
EOF
