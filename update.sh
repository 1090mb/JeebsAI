#!/usr/bin/env bash
#
# SIMPLE UPDATE SCRIPT - Run this on your VPS after pushing changes
# Just run: ./update.sh (or bash update.sh)
#
set -e

echo "🚀 Updating JeebsAI..."
echo ""

# Stash any local changes automatically
echo "📦 Stashing local changes..."
git stash save "Auto-stash $(date +%Y%m%d_%H%M%S)" 2>/dev/null || echo "Nothing to stash"

# Pull latest code
echo "📥 Pulling from GitHub..."
git pull origin main

# Restart service automatically
echo "🔄 Restarting Jeebs..."
if systemctl is-active --quiet jeebs 2>/dev/null; then
    sudo systemctl restart jeebs
    sleep 2
    if systemctl is-active --quiet jeebs; then
        echo "✅ Jeebs service restarted!"
    else
        echo "❌ Service failed. Check: journalctl -u jeebs -n 50"
        exit 1
    fi
elif systemctl is-active --quiet jeebs-docker 2>/dev/null; then
    docker-compose restart
    echo "✅ Docker containers restarted!"
else
    echo "⚠️  No service detected. Restart manually."
fi

echo ""
echo "════════════════════════════════════════════════════════"
echo "✅ UPDATE COMPLETE!"
echo "════════════════════════════════════════════════════════"
echo ""
echo "Changes are now live on your VPS!"
echo ""
