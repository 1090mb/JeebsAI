#!/usr/bin/env bash
# Push the improved pull script to GitHub

set -e

echo "🔧 Pushing improved pull script to GitHub..."

# Make sure script is executable
chmod +x pull_from_github.sh
chmod +x vps_fix_pull.sh

# Stage files
git add pull_from_github.sh vps_fix_pull.sh VPS_FIX_NOW.txt

# Commit
git commit -m "Improve pull script to handle Cargo.lock conflicts automatically" || echo "Nothing to commit"

# Push
git push origin main

echo ""
echo "✅ Updated pull script pushed to GitHub!"
echo ""
echo "════════════════════════════════════════════════════════"
echo "📥 FOR YOUR VPS - RUN THIS NOW:"
echo "════════════════════════════════════════════════════════"
echo ""
echo "cd ~/JeebsAI && git stash && git pull origin main && sudo systemctl restart jeebs"
echo ""
echo "════════════════════════════════════════════════════════"
