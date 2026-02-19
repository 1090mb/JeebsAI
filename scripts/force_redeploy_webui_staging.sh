#!/usr/bin/env bash
set -euo pipefail

# Force redeploy staging with the /webui static route.
# - Verifies /webui route exists in src/main.rs.
# - Rebuilds the release binary.
# - Replaces the staging binary and syncs webui assets.
# - Restarts the service and probes /webui/.

REPO_DIR=${REPO_DIR:-"/root/JeebsAI"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs-staging"}
APP_DIR=${APP_DIR:-"/opt/jeebs-staging"}
PORT=${PORT:-"8080"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

cd "$REPO_DIR"

grep -n 'Files::new("/webui"' src/main.rs >/dev/null || {
  echo "ERROR: /webui static route missing in src/main.rs" >&2
  exit 1
}

cargo build --release

mkdir -p "$APP_DIR/target/release"
systemctl stop "$SERVICE_NAME"
cp "$REPO_DIR/target/release/jeebs" "$APP_DIR/target/release/jeebs"
chmod 755 "$APP_DIR/target/release/jeebs"

rm -rf "$APP_DIR/webui"
cp -R "$REPO_DIR/webui" "$APP_DIR/webui"

systemctl daemon-reload
systemctl start "$SERVICE_NAME"

systemctl status "$SERVICE_NAME" --no-pager
curl -I "http://127.0.0.1:${PORT}/webui/"
