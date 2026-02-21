#!/usr/bin/env bash
#
# Quick Fix for VPS Pull Error - Run this on your VPS
#
set -e

echo "🔧 Fixing pull error and updating JeebsAI..."
echo ""

# Stash local changes (including Cargo.lock)
echo "📦 Stashing local changes..."
git stash save "Auto-stash before pull $(date +%Y%m%d_%H%M%S)"
echo "✅ Local changes stashed"
echo ""

# Pull latest changes
echo "📥 Pulling latest code from GitHub..."
git pull origin main
echo "✅ Pull complete"
echo ""

# Restart Jeebs service
echo "🔄 Restarting Jeebs service..."
if systemctl is-active --quiet jeebs 2>/dev/null; then
    sudo systemctl restart jeebs
    sleep 2
    if systemctl is-active --quiet jeebs; then
        echo "✅ Jeebs service restarted successfully!"
    else
        echo "❌ Jeebs service failed to start. Check logs with: journalctl -u jeebs -n 50"
    fi
elif systemctl is-active --quiet jeebs-docker 2>/dev/null; then
    docker-compose down
    docker-compose up -d --build
    echo "✅ Docker containers restarted!"
else
    echo "⚠️  No service detected. Restart manually if needed."
fi

echo ""
echo "════════════════════════════════════════════════════════"
echo "✅ DEPLOYMENT COMPLETE!"
echo "════════════════════════════════════════════════════════"
echo ""
echo "🎓 Access Topic Learning at:"
echo "   http://your-vps-ip/webui/admin_dashboard.html"
echo ""
echo "Look for the orange 'Topic Learning' section!"
echo ""
