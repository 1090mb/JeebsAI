#!/usr/bin/env bash
set -euo pipefail

# Push current branch (or specified branch) into `main` on origin safely.
# After merge this script will build a release artifact and create a GitHub Release
# using the `gh` CLI. Usage: ./scripts/push_to_main.sh [branch] [remote]

BRANCH=${1:-$(git rev-parse --abbrev-ref HEAD)}
ORIGIN=${2:-origin}

echo "Preparing to push branch '$BRANCH' into $ORIGIN/main"

if [ "$BRANCH" = "main" ]; then
  echo "On 'main' already: fetching and pushing local main to $ORIGIN/main"
  git fetch "$ORIGIN" main
  git pull --ff-only "$ORIGIN" main
  git push "$ORIGIN" main
  echo "Pushed main to $ORIGIN/main"
  exit 0
fi

# Ensure working tree clean
if [ -n "$(git status --porcelain)" ]; then
  echo "Working tree is dirty. Commit or stash changes first." >&2
  git status --porcelain
  exit 2
fi

orig_branch=$(git rev-parse --abbrev-ref HEAD)

# Push branch to origin first (so origin has branch to merge)
echo "Pushing branch '$BRANCH' to $ORIGIN"
git push "$ORIGIN" "$BRANCH"

# Switch to main and update
git fetch "$ORIGIN" main
git checkout main
git pull --ff-only "$ORIGIN" main || true

# Try to merge the branch into main. If conflicts occur, abort and restore.
if git merge --no-ff --no-edit "$ORIGIN/$BRANCH"; then
  echo "Merge succeeded. Pushing main to $ORIGIN"
  git push "$ORIGIN" main

  # After pushing main, create a release using gh CLI.
  # Determine tag from VERSION file if present, else use short commit+timestamp.
  if [ -f VERSION ]; then
    TAG=$(cat VERSION | tr -d " \n\r")
  else
    TAG="$(git rev-parse --short HEAD)-$(date +%s)"
  fi

  # Ensure we have a clean build artifact
  if [ -f Cargo.toml ]; then
    echo "Building release artifact (cargo build --release)"
    cargo build --release
    ARTIFACT=target/release/jeebs
    if [ -f "$ARTIFACT" ]; then
      ARCHIVE="jeebs-${TAG}.tar.gz"
      tar -czf "$ARCHIVE" -C "$(dirname "$ARTIFACT")" "$(basename "$ARTIFACT")" VERSION || true
      echo "Created artifact $ARCHIVE"
    else
      echo "Warning: build artifact not found at $ARTIFACT" >&2
      ARCHIVE=""
    fi
  else
    echo "No Cargo.toml found; creating source archive"
    ARCHIVE="jeebs-src-${TAG}.tar.gz"
    git archive --format=tar.gz -o "$ARCHIVE" HEAD
  fi

  if command -v gh >/dev/null 2>&1; then
    echo "Creating GitHub release $TAG"
    # If release tag already exists, create a unique tag suffix
    if gh release view "$TAG" >/dev/null 2>&1; then
      TAG="${TAG}-$(date +%s)"
      echo "Tag existed; using new tag $TAG"
    fi
    if [ -n "${ARCHIVE}" ] && [ -f "${ARCHIVE}" ]; then
      gh release create "$TAG" "$ARCHIVE" -t "$TAG" -n "Release $TAG"
    else
      gh release create "$TAG" -t "$TAG" -n "Release $TAG"
    fi
    echo "Release created: $TAG"
  else
    echo "gh CLI not installed or not authenticated; skipping GitHub Release creation." >&2
  fi

  echo "Merge and push complete. Returning to '$orig_branch'"
  git checkout "$orig_branch"
  exit 0
else
  echo "Merge failed due to conflicts; aborting merge and restoring branch." >&2
  git merge --abort || true
  git checkout "$orig_branch"
  exit 3
fi
