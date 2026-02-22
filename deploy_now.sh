#!/usr/bin/env bash
#
# ONE-LINE DEPLOYMENT
# Run this command: bash deploy_now.sh
#

set -e

echo "🚀 Deploying Topic Learning Feature..."

# Make sure we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Not in JeebsAI directory"
    exit 1
fi

# Add ALL changes (including deletions and new files)
git add .

# Commit if there are changes
if ! git diff --cached --quiet 2>/dev/null; then
    git commit -m "Update JeebsAI: Merge and commit all changes"
fi

# Push to GitHub
git push origin main

echo ""
echo "✅ PUSHED TO GITHUB!"
echo ""
echo "🚀 Deployment triggered via GitHub Actions."
