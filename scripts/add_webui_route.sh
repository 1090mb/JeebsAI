#!/usr/bin/env bash
set -euo pipefail

# Adds the /webui static route to src/main.rs (idempotent).
# Usage:
#   ./scripts/add_webui_route.sh /root/JeebsAI/src/main.rs

FILE=${1:-"/root/JeebsAI/src/main.rs"}

if [[ ! -f "$FILE" ]]; then
  echo "ERROR: file not found: $FILE" >&2
  exit 1
fi

# Add import if missing.
if ! grep -q 'use actix_files::Files;' "$FILE"; then
  sed -i '/use actix_web::cookie::Key;/a use actix_files::Files;' "$FILE"
fi

# Add static route if missing.
if ! grep -q 'Files::new("/webui"' "$FILE"; then
  sed -i '/\.service(auth::login_pgp)/a \            .service(Files::new("/webui", "./webui").index_file("index.html"))' "$FILE"
fi

echo "Updated: $FILE"
