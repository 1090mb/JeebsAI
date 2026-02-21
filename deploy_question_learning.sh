#!/usr/bin/env bash
set -e

chmod +x PUSH_QUESTION_LEARNING.sh

git add -A

git commit -m "COMPLETE: Question Learning System

✅ IMPLEMENTED:

Ask questions to Google:
  • Searches Google for answers
  • Extracts featured snippets
  • Stores question and answer

Store Q&A pairs:
  • Saves in brain nodes
  • Also stores as triples
  • Fast lookup in jeebs_store

Remember and recall:
  • Checks memory on repeat questions
  • Returns instant answers
  • Never re-searches web

User commands:
  • 'ask: what is photosynthesis?'
  • 'what questions have you asked?'
  • 'question statistics'

Features:

✅ Web integration
✅ Automatic storage
✅ Memory recall
✅ Question history
✅ Learning statistics
✅ Category organization

Ready for production!" || echo "Already staged"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ QUESTION LEARNING SYSTEM DEPLOYED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Deploy: bash update.sh on VPS"
echo ""
echo "Try in chat:"
echo "  • 'ask: what is photosynthesis?'"
echo "  • 'what have you learned?'"
echo "  • 'question statistics'"
echo ""
echo "🧠 Jeebs can now ask and learn!"
echo ""
