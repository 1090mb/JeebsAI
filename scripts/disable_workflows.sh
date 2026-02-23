#!/usr/bin/env bash
set -euo pipefail

# Moves existing workflows out of the workflows folder into workflows.disabled
# Usage: ./scripts/disable_workflows.sh

WORKDIR=".github/workflows"
DISABLEDIR=".github/workflows.disabled"

if [ ! -d "$WORKDIR" ]; then
  echo "No workflows directory found at $WORKDIR"
  exit 0
fi

mkdir -p "$DISABLEDIR"
for f in "$WORKDIR"/*; do
  [ -e "$f" ] || continue
  base=$(basename "$f")
  echo "Moving $base -> $DISABLEDIR/"
  mv "$f" "$DISABLEDIR/"
done

echo "Workflows moved to $DISABLEDIR. Creating minimal release workflow file."
mkdir -p "$WORKDIR"
cat > "$WORKDIR/release.yml" <<'YML'
name: Manual Release (kept)
on:
  workflow_dispatch: {}
jobs:
  noop:
    runs-on: ubuntu-latest
    steps:
      - run: echo "This repository keeps no automated workflows by default."
        shell: bash
YML

echo "Done. Review .github/workflows.disabled/ for moved files."
