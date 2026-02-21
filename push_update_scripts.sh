#!/usr/bin/env bash
# Push update scripts to GitHub

set -e

echo "🚀 Pushing update scripts to GitHub..."

# Make scripts executable
chmod +x update.sh pull_from_github.sh vps_fix_pull.sh

# Stage files
git add update.sh
git add pull_from_github.sh
git add vps_fix_pull.sh
git add UPDATE_WORKFLOW.txt
git add VPS_FIX_NOW.txt

# Commit
git commit -m "Add easy update scripts for VPS deployment

Added:
- update.sh: Simple one-command update (auto-stash, pull, restart)
- Improved pull_from_github.sh: Better stash detection
- vps_fix_pull.sh: Handles Cargo.lock conflicts
- UPDATE_WORKFLOW.txt: Complete workflow guide

Now updating after commits is super easy:
- Local: bash PUSH_NOW.sh
- VPS: bash update.sh" || echo "Nothing to commit"

# Push
git push origin main

echo ""
echo "✅ Update scripts pushed to GitHub!"
echo ""
echo "════════════════════════════════════════════════════════"
echo "📥 ON YOUR VPS - RUN THIS TO GET THE NEW SCRIPTS:"
echo "════════════════════════════════════════════════════════"
echo ""
echo "cd ~/JeebsAI"
echo "git stash"
echo "git pull origin main"
echo "chmod +x update.sh"
echo "sudo systemctl restart jeebs"
echo ""
echo "THEN FOR FUTURE UPDATES, JUST RUN:"
echo "  bash update.sh"
echo ""
echo "════════════════════════════════════════════════════════"
