#!/bin/bash
set -e

# Ensure we are running from the project root
cd "$(dirname "$0")/.."

# Configuration
VPS_HOST="192.227.193.148"
VPS_USER="root"
SSH_KEY="/Users/shoup/.ssh/jeebs_vps"
APP_DIR="/root/JeebsAI"
DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/1476367489490358272/k5Kn7xztOWFsXYOSnmwuHCiF3CQ1WXMxvYvzt4KSf_t0zdx36mVNbj7II9y-vwA6oEOd"
GITHUB_TOKEN="${GITHUB_TOKEN:-}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Failure handling
function notify_failure {
    local exit_code=$?
    local line_num=$BASH_LINENO
    echo -e "${RED}❌ Deployment Failed! (Exit code: $exit_code on line $line_num)${NC}"
    if [ -n "$DISCORD_WEBHOOK_URL" ]; then
        echo -e "${RED}🔔 Sending failure notification...${NC}"
        curl -s -H "Content-Type: application/json" \
             -d "{\"content\": \"❌ **JeebsAI Deployment FAILED!**\n\n- Target: $VPS_HOST\n- Exit Code: $exit_code at line $line_num\n- Time: $(date)\"}" \
             "$DISCORD_WEBHOOK_URL" > /dev/null
    fi
}
trap notify_failure ERR

echo -e "${BLUE}🚀 JeebsAI - Push and Deploy${NC}"
echo "Target: $VPS_USER@$VPS_HOST:$APP_DIR"

# Check for GITHUB_TOKEN
if [ -z "$GITHUB_TOKEN" ]; then
    echo -e "${YELLOW}⚠️  No GITHUB_TOKEN environment variable found.${NC}"
    echo -e "${YELLOW}   If the repository is private, deployment may fail unless you have SSH agent forwarding setup.${NC}"
    echo -e "${YELLOW}   To fix, run: export GITHUB_TOKEN='your_token' before running this script.${NC}"
fi

# Pre-deployment notification
if [ -n "$DISCORD_WEBHOOK_URL" ]; then
    echo -e "${BLUE}🔔 Sending start notification...${NC}"
    curl -s -H "Content-Type: application/json" \
         -d "{\"content\": \"⏳ **JeebsAI Deployment Started**\n\n🚀 Target: $VPS_HOST\n🕒 Time: $(date)\"}" \
         "$DISCORD_WEBHOOK_URL" > /dev/null
fi

# Fix macOS metadata corruption in .git folder (common on external drives)
echo -e "${BLUE}🧹 Cleaning up git metadata...${NC}"
find .git -name "._*" -delete 2>/dev/null || true

# 1. Git Push
echo -e "${BLUE}📦 Pushing to GitHub...${NC}"
git add .
git commit -m "Update: Deployment $(date +'%Y-%m-%d %H:%M')" || echo "Nothing to commit"
git push origin main

# 2. Remote Deploy
echo -e "${BLUE}📡 Connecting to VPS...${NC}"
ssh -A -o ConnectTimeout=30 -i "$SSH_KEY" "$VPS_USER@$VPS_HOST" << EOF
    set -e
    echo "📂 Navigating to $APP_DIR..."
    
    # Ensure directory exists
    mkdir -p "$APP_DIR"
    cd "$APP_DIR"
    
    # Define URLs (using local GITHUB_TOKEN if available)
    # Note: We escape variables we want evaluated on the remote, but leave GITHUB_TOKEN to expand locally
    HTTPS_URL="https://github.com/Deployed-Labs/JeebsAI.git"
    SSH_URL="git@github.com:Deployed-Labs/JeebsAI.git"
    
    if [ -n "$GITHUB_TOKEN" ]; then
        HTTPS_URL="https://oauth2:$GITHUB_TOKEN@github.com/Deployed-Labs/JeebsAI.git"
    fi

    # Helper to try cloning with fallback
    try_clone() {
        echo "⬇️  Cloning repository..."
        # 1. Try HTTPS (with token if provided)
        if git clone "\$HTTPS_URL" .; then return 0; fi
        
        # 2. Try SSH (uses agent forwarding if available)
        echo "⚠️  HTTPS clone failed. Trying SSH..."
        if git clone "\$SSH_URL" .; then return 0; fi
        
        return 1
    }

    # Check if it's a git repo, if not clone it (or init and pull if empty)
    if [ ! -d ".git" ]; then
        if ! try_clone; then
            echo "❌ Clone failed. Please check permissions or GITHUB_TOKEN."
            exit 1
        fi
    fi

    echo "⬇️  Updating deployment script..."
    # Clean up potential corruption on VPS side
    find .git -name "._*" -delete 2>/dev/null || true

    # Try to fetch. If it fails (auth error or corruption), try to recover.
    if ! git fetch origin; then
        echo "⚠️  Git fetch failed. Attempting to fix remote URL..."
        
        # Update remote URL to the one with token (or standard HTTPS)
        git remote set-url origin "\$HTTPS_URL"
        
        if ! git fetch origin; then
            echo "⚠️  Fetch failed again. Re-cloning repository..."
            cd ..
            rm -rf "$APP_DIR"
            mkdir -p "$APP_DIR"
            cd "$APP_DIR"
            try_clone
        fi
    else
        git reset --hard origin/main
    fi
    
    echo "🚀 Running full VPS deployment..."
    export REPO_URL="\$HTTPS_URL"
    chmod +x deploy_to_vps.sh
    ./deploy_to_vps.sh
EOF

echo -e "${GREEN}🎉 Deployment Complete!${NC}"

# Post-deployment notification
if [ -n "$DISCORD_WEBHOOK_URL" ]; then
    echo -e "${BLUE}🔔 Sending Discord notification...${NC}"
    curl -s -H "Content-Type: application/json" \
         -d "{\"content\": \"🚀 **JeebsAI Deployment Complete!**\n\n✅ Target: $VPS_HOST\n🕒 Time: $(date)\"}" \
         "$DISCORD_WEBHOOK_URL" > /dev/null
fi