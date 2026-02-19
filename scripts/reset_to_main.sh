#!/usr/bin/env bash
set -euo pipefail

# Reset your repo to the main branch on origin.
# - Fetches latest from GitHub.
# - Checks out and resets to origin/main.
# - Cleans up any local changes.
# - Verifies you're on main.

REPO_DIR=${REPO_DIR:-"/root/JeebsAI"}

cd "$REPO_DIR"

echo "Fetching from origin..."
git fetch origin

echo "Checking out main branch..."
git checkout main

echo "Resetting to origin/main..."
git reset --hard origin/main

echo "Cleaning up local files..."
git clean -fd

echo ""
echo "Current branch:"
git branch

echo ""
echo "Latest commits:"
git log --oneline -n 5

echo ""
echo "✓ You are now on the main branch."
