#!/usr/bin/env bash
set -e

echo "💾 Pushing Toggle State Persistence..."
echo ""

git add src/toggle_manager.rs
git add src/lib.rs
git add src/main.rs
git add src/admin/internet.rs
git add src/cortex.rs
git add TOGGLE_STATE_PERSISTENCE.txt

git commit -m "Add Toggle State Persistence - Remember user preferences

FEATURE: Remember Internet and Training Mode toggle positions

HOW IT WORKS:

1. On startup:
   • Load saved toggle positions from database
   • Use user's last settings
   • Internet OFF/ON (as you left it)
   • Training OFF/ON (as you left it)

2. When you change toggle:
   • Save new position to database
   • Immediately applied
   • Persisted forever (until changed)

3. On restart:
   • Loads your saved preferences
   • Uses your settings
   • Never auto-changes toggles

NEW MODULE: src/toggle_manager.rs

Functions:
  • load_toggle_states() - Load both toggles on startup
  • save_internet_toggle_state() - Save internet position
  • save_training_toggle_state() - Save training position
  • get_toggle_states() - Get current positions

Storage:
  • toggle:internet_enabled (jeebs_store)
  • toggle:training_enabled (jeebs_store)

MODIFIED:

src/main.rs:
  • Load toggles on startup
  • Use saved preferences
  • Only spawn training if enabled

src/admin/internet.rs:
  • Save internet toggle when changed
  • Persist to database

src/cortex.rs:
  • Save training toggle when changed
  • Persist to database

BENEFITS:

✅ No surprises after restart
✅ User preferences respected
✅ Permanent memory
✅ Safe defaults
✅ Convenient

EXAMPLES:

Production:
  Set: Internet OFF, Training OFF
  → Stays OFF after every restart

Development:
  Set: Internet ON, Training ON
  → Stays ON after every restart

Debug:
  Temporarily toggle OFF for debug
  → Stays OFF until you toggle back ON

Ready for production!" || echo "Already committed"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ TOGGLE STATE PERSISTENCE PUSHED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Deploy on VPS:"
echo ""
echo "  bash update.sh"
echo ""
echo "Now toggle positions are SAVED and REMEMBERED!"
echo ""
echo "Toggles will stay in the position YOU set them,
even after server restarts, until YOU change them again."
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
