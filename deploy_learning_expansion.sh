#!/usr/bin/env bash
set -e

echo "🌐 Pushing Jeebs Learning Expansion to GitHub..."
echo ""

# Make scripts executable
chmod +x PUSH_LEARNING_EXPANSION.sh

# Add all changes
git add src/cortex.rs
git add LEARNING_EXPANSION_COMPLETE.txt
git add LEARNING_EXPANSION_READY.txt
git add PUSH_LEARNING_EXPANSION.sh

# Commit
git commit -m "Expand Jeebs learning: explore new websites, follow links, never revisit

FEATURE: Widened scope of Jeebs learning during training cycles

REQUEST FULFILLED:
  ✅ Follow links to explore new websites
  ✅ Go to web pages Jeebs has never seen
  ✅ Never go to the same site twice
  ✅ Widen the scope of learning

IMPLEMENTATION:

1. Global Domain Tracking
   • Persists in database
   • Reads on startup
   • Prevents ALL revisits
   • Survives restarts

2. Intelligent Link Following
   • Prioritizes NEW domains
   • Explores 15-20 domains per crawl
   • Max 8 pages per domain
   • Follows chains to discover new territory

3. Expanded Exploration
   • Pages per crawl: 25 → 50 (2x)
   • Sources: 8 → 30+ diverse websites
   • Categories: Science, Tech, News, Dev, Wikipedia

4. Smart Queue Management
   • New domain links jump to priority
   • Old domain links deprioritized
   • Breadth-first exploration
   • Prevents getting stuck

RESULT:
   Each training cycle explores completely NEW domains
   Never wastes time revisiting old sites
   Exponential knowledge growth
   Comprehensive coverage across web

When enabled:
   ✅ Training explores 15-20 new domains
   ✅ 50 pages per cycle (was 25)
   ✅ Database prevents ANY revisit
   ✅ Link following discovers new areas
   ✅ Knowledge grows exponentially" || echo "Already committed"

# Push
git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ LEARNING EXPANSION PUSHED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Deploy on VPS:"
echo ""
echo "  bash update.sh"
echo ""
echo "Then enable internet in admin dashboard if not already."
echo ""
echo "Jeebs will now:"
echo "  ✅ Explore new websites"
echo "  ✅ Follow links to discover new territory"
echo "  ✅ NEVER revisit the same site"
echo "  ✅ Widen knowledge across diverse domains"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
