#!/usr/bin/env bash
set -e

echo "👥 Pushing User Chat System with Admin Commands..."
echo ""

git add src/user_chat.rs
git add src/lib.rs
git add src/main.rs
git add src/cortex.rs
git add USER_CHAT_SYSTEM_COMPLETE.txt

git commit -m "Add User Chat System - PGP Registration & Admin Commands

FEATURE: Normal users can chat with Jeebs via PGP authentication

USER CHAT SYSTEM:

✅ User Registration
   • Via PGP sign-on method
   • Username (3-32 chars)
   • PGP public key
   • Self-registration

✅ User Authentication
   • PGP cryptographic verification
   • Session-based authentication
   • JWT tokens for API access
   • No passwords needed

✅ User Chat
   • POST /api/chat endpoint
   • Username tracked
   • Secure and authenticated
   • Unregistered users blocked

✅ Admin Privileges
   • Only root admin (1090mb) has privileges
   • No other accounts are admins
   • Admin commands hidden from regular users
   • Privilege escalation prevented

ADMIN COMMANDS (root admin only):

• admin help - Show admin commands
• admin users - List registered users
• admin stats - System statistics
• admin logs [N] - Show log entries
• admin database - Database statistics
• admin training now - Start training cycle
• admin internet on/off - Toggle internet
• admin training on/off - Toggle training
• admin reset [username] - Reset user data
• admin ban/unban [user] - Ban/unban user
• admin broadcast [msg] - Message all users

NON-ADMIN USERS:

• Regular chat works normally
• Admin commands return permission denied
• No privilege escalation possible
• Clear security boundaries

NEW MODULE: src/user_chat.rs

Functions:
  • user_chat() - Main chat endpoint
  • chat_status() - Check auth status
  • is_user_authenticated() - Auth check
  • is_root_admin() - Admin check
  • get_username() - Get username

ENDPOINTS:

POST /api/chat
  • Requires authentication
  • Returns response + username + admin flag

GET /api/chat/status
  • Shows auth status
  • Shows username and admin flag

SECURITY:

✅ Authentication required
✅ PGP-based sign-on
✅ No unregistered access
✅ Admin isolation
✅ Full logging
✅ IP tracking

READY FOR PRODUCTION!" || echo "Already committed"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ USER CHAT SYSTEM PUSHED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Deploy on VPS:"
echo ""
echo "  bash update.sh"
echo ""
echo "Features:"
echo "  • Normal users can chat (PGP registration)"
echo "  • Unregistered users blocked"
echo "  • Admin commands for root admin"
echo "  • No privilege escalation"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "👥 User chat system ready!"
echo ""
