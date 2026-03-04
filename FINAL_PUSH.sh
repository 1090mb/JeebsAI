#!/usr/bin/env bash
set -e

echo "🚀 Pushing ALL update scripts and Topic Learning feature..."

# Make executable
chmod +x update.sh pull_from_github.sh vps_fix_pull.sh push_update_scripts.sh PUSH_NOW.sh

# Add everything
git add -A

# Commit
git commit -m "Add complete update workflow with easy scripts

✨ Topic Learning Feature:
- Textbox in admin dashboard for entering topics
- LEARN button with Enter key support
- Real-time status updates

🚀 Easy Update Scripts:
- update.sh: One-command VPS update (auto-stash, pull, restart)
- pull_from_github.sh: Interactive update with prompts
- vps_fix_pull.sh: Handles Cargo.lock conflicts
- PUSH_NOW.sh: Simple push from local machine

📚 Complete Documentation:
- UPDATE_WORKFLOW.txt: Full workflow guide
- YES_UPDATE_SCRIPTS_READY.txt: Quick reference
- VPS_FIX_NOW.txt: Fix for merge conflicts

Now updating is super simple:
  LOCAL: bash PUSH_NOW.sh
  VPS:   bash update.sh" || echo "Already committed"

# Push
git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ EVERYTHING PUSHED TO GITHUB!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "📥 ON YOUR VPS, RUN THIS ONCE:"
echo ""
echo "cd ~/JeebsAI && git stash && git pull origin main && chmod +x update.sh && sudo systemctl restart jeebs"
echo ""
echo "THEN FOR ALL FUTURE UPDATES, JUST:"
echo ""
echo "  bash update.sh"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "🎓 Topic Learning will be live at:"
echo "   http://your-vps-ip/admin_dashboard.html"
echo ""
echo "════════════════════════════════════════════════════════════"
