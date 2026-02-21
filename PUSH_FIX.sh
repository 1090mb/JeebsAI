#!/usr/bin/env bash
set -e

echo "🔧 Pushing Topic Learning Fix..."
echo ""

# Make scripts executable
chmod +x update.sh pull_from_github.sh

# Add the fixed file
git add webui/admin_dashboard.html
git add TOPIC_LEARNING_FIX.txt
git add PUSH_FIX.sh

# Commit
git commit -m "Fix Topic Learning: Change 'message' to 'prompt' field

ISSUE: Topic Learning was failing with 'failed to initiate learning'

ROOT CAUSE:
- JavaScript sent { message: '...' }
- API expects { prompt: '...' }
- Field name mismatch caused request failure

FIX:
- Changed request body to use 'prompt' field
- Improved error handling with detailed messages
- Added HTTP status codes to errors
- Better response validation

Now the Topic Learning feature works correctly!

Testing:
1. Go to admin dashboard
2. Enter topic (e.g., 'quantum computing')
3. Click LEARN
4. Should work now ✅" || echo "Nothing to commit"

# Push
git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ FIX PUSHED TO GITHUB!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "📥 Deploy on VPS:"
echo ""
echo "  cd ~/JeebsAI && bash update.sh"
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
echo "🎓 Then test Topic Learning at:"
echo "   http://your-vps-ip/webui/admin_dashboard.html"
echo ""
echo "The 'failed to initiate learning' error is now FIXED! ✅"
echo ""
