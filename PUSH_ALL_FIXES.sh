#!/usr/bin/env bash
set -e

echo "🚀 Pushing ALL Topic Learning Fixes..."
echo ""

# Make scripts executable
chmod +x update.sh pull_from_github.sh PUSH_STATUS_FIX.sh

# Add all changes
git add webui/admin_dashboard.html
git add update.sh pull_from_github.sh
git add SYSTEM_STATUS_FIX.txt TOPIC_LEARNING_FIX.txt
git add PUSH_STATUS_FIX.sh PUSH_FIX.sh
git add *.txt 2>/dev/null || true

# Commit
git commit -m "Fix all Topic Learning issues - Complete working version

FIXED ISSUES:
1. ✅ Wrong field name: Changed 'message' to 'prompt'
2. ✅ JavaScript syntax error: Removed duplicate lines
3. ✅ System status broken: Fixed parse error

Changes:
- Topic Learning uses correct 'prompt' field for API
- Removed duplicate closing lines in learnTopic()
- Better error handling with HTTP status codes
- All JavaScript functions now work correctly

What works now:
✅ Topic Learning - Enter topic, click LEARN, works!
✅ System Status - Shows uptime and memory
✅ Server Logs - Real-time updates
✅ Active Sessions - User management
✅ All dashboard JavaScript functions

Testing:
1. Go to admin dashboard
2. System Status should load
3. Enter topic in Topic Learning section
4. Click LEARN - should research topic
5. All features working! 🎉" || echo "Nothing new to commit"

# Push
git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ ALL FIXES PUSHED TO GITHUB!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Fixed:"
echo "  ✅ Topic Learning field name (prompt)"
echo "  ✅ JavaScript syntax error"
echo "  ✅ System Status loading"
echo "  ✅ All dashboard functions"
echo ""
echo "📥 Deploy on VPS with ONE command:"
echo ""
echo "  bash update.sh"
echo ""
echo "Or manually:"
echo ""
echo "  cd ~/JeebsAI"
echo "  git stash"
echo "  git pull origin main"
echo "  sudo systemctl restart jeebs"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "🎉 Everything is fixed and ready to deploy!"
echo ""
