#!/bin/bash
#
# Configures macOS to stop creating ._ metadata files on external/network drives
# and cleans up existing ones in the current directory.
#

echo "🔧 Configuring macOS to stop creating ._ files on external drives..."

# Prevent ._ files on Network volumes
defaults write com.apple.desktopservices DSDontWriteNetworkStores -bool true

# Prevent ._ files on USB volumes
defaults write com.apple.desktopservices DSDontWriteUSBStores -bool true

echo "✅ Configuration updated. Restarting Finder..."
killall Finder

echo "🧹 Cleaning up existing ._ files in the current directory..."
find . -name "._*" -print -delete

echo "🎉 Done! macOS should now be cleaner with external drives."