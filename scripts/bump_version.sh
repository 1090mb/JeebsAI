#!/usr/bin/env bash
set -euo pipefail
# Simple semantic patch: increments patch version (vMAJOR.MINOR.PATCH)
# Updates VERSION file, commits, tags, and optionally creates a GitHub release.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [ ! -f VERSION ]; then
  echo "v0.0.0" > VERSION
fi

cur=$(cat VERSION | tr -d '\n' | tr -d '\r')
if [[ "$cur" =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  major=${BASH_REMATCH[1]}
  minor=${BASH_REMATCH[2]}
  patch=${BASH_REMATCH[3]}
  patch=$((patch+1))
  new_version="v${major}.${minor}.${patch}"
else
  echo "WARNING: VERSION content not recognized: $cur" >&2
  new_version="v0.0.1"
fi

echo "Bumping VERSION: $cur -> $new_version"
echo "$new_version" > VERSION

git add VERSION
git commit -m "chore(release): bump version to $new_version [ci skip]" || echo "No commit (no changes)"
git tag -a "$new_version" -m "Release $new_version" || echo "Tag exists or failed"

# Push commit and tag
git push origin HEAD || echo "Failed to push branch (check permissions)"
git push origin "$new_version" || echo "Failed to push tag (check permissions)"

# Create GitHub release if gh is available
if command -v gh >/dev/null 2>&1; then
  gh release create "$new_version" --title "Release $new_version" --notes "Release $new_version" || true
fi

echo "Done. New version: $new_version"
