#!/usr/bin/env bash
#
# Quick Push - Push changes to GitHub
#
set -e

echo "🚀 Pushing JeebsAI to GitHub"
echo "==========================================="

# Add all changes
echo "📦 Staging changes..."
git add .

# Commit
echo "💾 Committing..."
git commit -m "Update JeebsAI: Latest changes"

# Push to GitHub
echo "⬆️  Pushing to GitHub..."
git push origin main

echo ""
echo "✅ Successfully pushed to GitHub!"
echo ""
echo "GitHub Actions will handle deployment."
