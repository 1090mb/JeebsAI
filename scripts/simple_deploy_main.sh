#!/usr/bin/env bash
set -euo pipefail

# Simple VPS deploy script: pull main, build, install and restart service.
# Usage: sudo ./scripts/simple_deploy_main.sh [deploy_dir] [service_name]
# Defaults: deploy_dir=/root/JeebsAI service_name=jeebs

DEPLOY_DIR=${1:-/root/JeebsAI}
SERVICE_NAME=${2:-jeebs}
GITHUB_REPO=${3:-Deployed-Labs/JeebsAI}

echo "Deploying main -> $DEPLOY_DIR (service: $SERVICE_NAME)"

mkdir -p "$(dirname "$DEPLOY_DIR")"
if [ ! -d "$DEPLOY_DIR" ]; then
  echo "Cloning repo to $DEPLOY_DIR"
  git clone --depth 1 "https://github.com/$GITHUB_REPO.git" "$DEPLOY_DIR"
fi

cd "$DEPLOY_DIR"

if [ ! -d .git ]; then
  echo "Error: not a git repo: $DEPLOY_DIR" >&2
  exit 1
fi

# Ensure clean authoritative state of main
git fetch origin --prune
git checkout main
git reset --hard origin/main

# Ensure env for sessions
if [ ! -f /etc/jeebs.env ]; then
  echo "Creating /etc/jeebs.env"
  SESSION_KEY_B64=$(head -c 24 /dev/urandom | base64 | tr -d '\n')
  printf "SESSION_KEY_B64=%s\n" "$SESSION_KEY_B64" > /etc/jeebs.env
  chmod 600 /etc/jeebs.env || true
fi

# Build (requires Rust toolchain)
if command -v cargo >/dev/null 2>&1 && [ -f Cargo.toml ]; then
  echo "Building release"
  cargo build --release
fi

# Ensure binary location
BINARY_SRC=""
if [ -f target/release/jeebs ]; then
  BINARY_SRC=target/release/jeebs
elif [ -f jeebs ]; then
  BINARY_SRC=jeebs
else
  echo "Warning: release binary not found; continuing" >&2
fi

BINARY_DST="$DEPLOY_DIR/jeebs"
if [ -n "$BINARY_SRC" ]; then
  install -m 0755 "$BINARY_SRC" "$BINARY_DST"
fi

# Install minimal systemd unit if missing or update ExecStart
SERVICE_PATH="/etc/systemd/system/${SERVICE_NAME}.service"
if [ -f deploy/jeebs.service ]; then
  echo "Installing provided unit file"
  sed "s|ExecStart=.*|ExecStart=$BINARY_DST|" deploy/jeebs.service > "$SERVICE_PATH" || cp deploy/jeebs.service "$SERVICE_PATH"
else
  cat > "$SERVICE_PATH" <<EOF
[Unit]
Description=JeebsAI service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$DEPLOY_DIR
EnvironmentFile=/etc/jeebs.env
ExecStart=$BINARY_DST
Restart=always
RestartSec=5
TimeoutStopSec=20

[Install]
WantedBy=multi-user.target
EOF
fi

chmod 644 "$SERVICE_PATH" || true
systemctl daemon-reload || true
systemctl enable "$SERVICE_NAME" || true
systemctl restart "$SERVICE_NAME"

echo "Service status:"
systemctl status "$SERVICE_NAME" --no-pager -l || true

echo "Deployed. VERSION:"
[ -f VERSION ] && cat VERSION || echo "(no VERSION file)"
