#!/bin/bash
set -e

# Configuration
VPS_HOST="192.227.193.148"
VPS_USER="root"
SSH_KEY="/Users/shoup/.ssh/jeebs_vps"
APP_DIR="/root/JeebsAI"

echo "🚀 Direct Rsync Deployment to $VPS_HOST:$APP_DIR"
echo "⚠️  Safety Check: This will overwrite code on the VPS."

read -p "Are you sure? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Deployment cancelled."
    exit 1
fi

# Build local to ensure it compiles (optional, but good practice)
 echo "🔨 Building locally..."
 cargo build --release

echo "📦 Syncing files..."
rsync -avz --exclude 'target' --exclude '.git' --exclude '.env' --exclude 'jeebs.db*' --exclude '*.log' \
    -e "ssh -i $SSH_KEY" \
    ./ "$VPS_USER@$VPS_HOST:$APP_DIR/"

echo "🔄 Remote Build & Restart..."
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_HOST" << EOF
    set -e
    cd "$APP_DIR"
    echo "🔨 Building release on VPS..."
    cargo build --release
    echo "🔄 Restarting service..."
    systemctl restart jeebs
    systemctl status jeebs --no-pager | grep "Active:"
EOF

echo "✅ Done!"