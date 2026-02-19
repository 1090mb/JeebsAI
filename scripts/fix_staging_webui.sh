#!/usr/bin/env bash
set -euo pipefail

# Short fix script for staging:
# - Ensures the staging env file exists and points to an absolute SQLite path.
# - Rebuilds the release binary.
# - Syncs the web UI assets into the service working directory.
# - Restarts the staging systemd service with the new binary.

REPO_DIR=${REPO_DIR:-"/root/JeebsAI"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs-staging"}
APP_DIR=${APP_DIR:-"/opt/jeebs-staging"}
ENV_FILE=${ENV_FILE:-"/etc/jeebs-staging.env"}
DB_PATH=${DB_PATH:-"/var/lib/jeebs-staging/jeebs.db"}
PORT=${PORT:-"8080"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

mkdir -p "$(dirname "$ENV_FILE")"
mkdir -p "$(dirname "$DB_PATH")"

if [[ ! -f "$ENV_FILE" ]]; then
  cat >"$ENV_FILE" <<EOF
PORT=$PORT
DATABASE_URL=sqlite:$DB_PATH
RUST_LOG=info
EOF
else
  sed -i "s|^PORT=.*|PORT=$PORT|" "$ENV_FILE"
  sed -i "s|^DATABASE_URL=.*|DATABASE_URL=sqlite:$DB_PATH|" "$ENV_FILE"
fi

cd "$REPO_DIR"
cargo build --release

mkdir -p "$APP_DIR/target/release"
systemctl stop "$SERVICE_NAME"
cp "$REPO_DIR/target/release/jeebs" "$APP_DIR/target/release/jeebs"
chmod 755 "$APP_DIR/target/release/jeebs"

# Sync web UI assets into the service working directory.
rm -rf "$APP_DIR/webui"
cp -R "$REPO_DIR/webui" "$APP_DIR/webui"

systemctl daemon-reload
systemctl start "$SERVICE_NAME"

systemctl status "$SERVICE_NAME" --no-pager

curl -I "http://127.0.0.1:${PORT}/webui/"
