#!/usr/bin/env bash
set -euo pipefail

# VPS deploy script: pull a branch, build (if present), and restart systemd service.
# Usage on VPS: sudo ./scripts/vps_deploy.sh [branch] [deploy_dir]

# Can accept: branch or tag. If TAG_RELEASE=true we'll treat first arg as a release tag.
ARG=${1:-main}
DEPLOY_DIR=${2:-/root/JeebsAI}
SERVICE_NAME=${3:-jeebs}
GITHUB_REPO=${4:-Deployed-Labs/JeebsAI}
USE_RELEASE=${USE_RELEASE:-1}

# If ARG looks like a tag (starts with v or contains '-'), prefer release download when available
BRANCH="$ARG"

echo "Deploying branch '$BRANCH' into $DEPLOY_DIR and restarting service '$SERVICE_NAME'"

if [ ! -d "$DEPLOY_DIR" ]; then
  echo "Error: deploy dir '$DEPLOY_DIR' does not exist." >&2
  exit 2
fi

cd "$DEPLOY_DIR"

# Ensure repository is present
if [ ! -d .git ]; then
  echo "No git repository found in $DEPLOY_DIR" >&2
  exit 3
fi

git fetch --all --prune

# Checkout the branch (create local tracking branch if needed)
if git rev-parse --verify --quiet "$BRANCH" >/dev/null; then
  git checkout "$BRANCH"
else
  git checkout -B "$BRANCH" "origin/$BRANCH"
fi

# Reset to remote

# If using release assets, try downloading the release matching BRANCH (tag)
if [ "$USE_RELEASE" = "1" ]; then
  echo "Attempting to download release for tag: $BRANCH"
  if command -v gh >/dev/null 2>&1; then
    set +e
    gh release download "$BRANCH" --repo "$GITHUB_REPO" -p "jeebs-*.tar.gz" -D . 2>/dev/null
    rc=$?
    set -e
    if [ $rc -eq 0 ]; then
      ARCHIVE=$(ls jeebs-*.tar.gz 2>/dev/null | head -n1 || true)
      if [ -n "$ARCHIVE" ]; then
        echo "Found release archive: $ARCHIVE"
        tar -xzf "$ARCHIVE" -C .
        if [ -f jeebs ]; then
          mv jeebs target/release/jeebs || true
        fi
      fi
    else
      echo "No release asset found via gh for $BRANCH, falling back to git pull"
    fi
  else
    echo "gh CLI not available on VPS; falling back to git pull"
  fi
fi

# If we don't have a release artifact, update from git
if [ ! -f target/release/jeebs ]; then
  echo "No release binary present; updating git branch $BRANCH"
  git fetch --all --prune
  if git rev-parse --verify --quiet "$BRANCH" >/dev/null; then
    git checkout "$BRANCH"
  else
    git checkout -B "$BRANCH" "origin/$BRANCH"
  fi
  git reset --hard "origin/$BRANCH"
  if [ -f Cargo.toml ]; then
    echo "Building release (cargo build --release)"
    cargo build --release
  fi
fi

echo "Reloading systemd and restarting service: $SERVICE_NAME"
systemctl daemon-reload || true
systemctl restart "$SERVICE_NAME"

sleep 1
systemctl status "$SERVICE_NAME" --no-pager -l || true

if [ -f VERSION ]; then
  echo "Current VERSION file content:"
  cat VERSION || true
fi

echo "Deploy finished"
