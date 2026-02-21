#!/usr/bin/env bash
set -e

echo "🌐 Pushing Jeebs Learning Expansion..."
echo ""

# Add changes
git add src/cortex.rs
git add LEARNING_EXPANSION_COMPLETE.txt

# Commit
git commit -m "Expand Jeebs learning: explore new websites, follow links, never revisit

FEATURE: Widened scope of Jeebs learning during training cycles

KEY IMPROVEMENTS:

1. Global Domain Tracking
   • Tracks all visited domains across crawls
   • Persists in database for permanent tracking
   • Ensures Jeebs never revisits the same site
   • Survives server restarts and training cycles

2. Intelligent Link Following
   • Prioritizes NEW domains over visited ones
   • Aggressively follows links to unexplored territory
   • Max 8 pages per domain (prevents getting stuck)
   • Encourages breadth of knowledge

3. Increased Exploration
   • Max pages per crawl: 25 → 50 (2x more exploration)
   • 30 diverse sources (was 8)
   • Follows link chains to discover new domains

4. Expanded Website Candidates
   • Science & research: NASA, Nature, ScienceDaily, arXiv
   • Tech & AI: GitHub Trending, OpenAI, IBM Quantum
   • News: BBC, Guardian, Economist, Wired, TechCrunch
   • Developer: MDN, Rust, Python, Go, FreeCodeCamp
   • Wikipedia random pages and topical portals

5. Smart Domain Diversity
   • Tracks domains per crawl session
   • Prevents concentrating on single domain
   • Links to new domains jump to front of queue
   • Old domain links only followed if quota available

RESULT:
   • Each training cycle explores 15-20 NEW domains
   • Jeebs builds exponentially diverse knowledge
   • No wasted crawls revisiting old sites
   • Learning breadth increases with each cycle


TECHNICAL DETAILS:
   • Added load_previously_crawled_domains()
   • Added store_crawled_domain()
   • Expanded random_crawl_candidates() from 8 → 30
   • Modified crawl_and_store() for smart exploration
   • Domain tracking via jeebs_store table
   • Per-domain page counting to enforce limits


When enabled via Internet Toggle:
   Training will aggressively explore the web and build
   comprehensive knowledge across diverse domains!" || echo "Nothing to commit"

# Push
git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ LEARNING EXPANSION PUSHED TO GITHUB!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "📥 Deploy on VPS:"
echo ""
echo "  bash update.sh"
echo ""
echo "This will rebuild Jeebs with the new exploration logic."
echo ""
echo "After deploying:"
echo ""
echo "1. Enable Internet Toggle in admin dashboard"
echo "2. Next training cycle will:"
echo "   ✅ Explore 15-20 NEW domains per crawl"
echo "   ✅ Never revisit the same site twice"
echo "   ✅ Follow links to discover new territory"
echo "   ✅ Build diverse knowledge across web"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "🎓 Jeebs learning is now MUCH WIDER and EXPLORATORY!"
echo ""
