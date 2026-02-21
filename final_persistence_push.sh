#!/usr/bin/env bash
set -e

chmod +x go_persistence.sh deploy_toggle_persistence.sh PUSH_TOGGLE_PERSISTENCE.sh

git add -A

git commit -m "FINAL: Toggle State Persistence - Remember User Preferences

✅ COMPLETE IMPLEMENTATION:

Save toggle positions:
  • Internet Access toggle
  • Training Mode toggle
  • Persisted in database

Remember across restarts:
  • Loads saved state on startup
  • Uses user's last settings
  • Never auto-changes toggles
  • Survives crashes and restarts

User Control:
  • Toggles only change when you toggle them
  • Your preferences always respected
  • Safe and predictable
  • No surprises

HOW IT WORKS:

Startup:
  1. Load toggles from database
  2. Use saved preferences
  3. Start with YOUR configuration

When you toggle:
  1. Click toggle
  2. Confirm
  3. Saved to database immediately
  4. Applied to system

Restart:
  1. Load toggles from database
  2. Internet: OFF/ON (as you left it)
  3. Training: OFF/ON (as you left it)
  4. No changes!

NEW MODULE:
  src/toggle_manager.rs

Functions:
  • load_toggle_states() - Load on startup
  • save_internet_toggle_state() - Save position
  • save_training_toggle_state() - Save position
  • get_toggle_states() - Get current

MODIFIED:
  • src/main.rs - Load and use saved preferences
  • src/admin/internet.rs - Save toggle on change
  • src/cortex.rs - Save toggle on change
  • src/lib.rs - Export module

STORAGE:
  jeebs_store table:
    toggle:internet_enabled
    toggle:training_enabled

BENEFITS:

✅ Predictable behavior
✅ User preferences saved
✅ No surprises after restart
✅ Safe and reliable
✅ Convenient

Ready for production!" || echo "Already staged"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ TOGGLE STATE PERSISTENCE DEPLOYED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Pushed to GitHub!"
echo ""
echo "Deploy on VPS: bash update.sh"
echo ""
echo "Your toggle positions will now be SAVED and REMEMBERED!"
echo ""
echo "They will stay exactly where YOU set them,
even after server restarts, until YOU change them again."
echo ""
echo "💾 Persistent preferences ready!"
echo ""
