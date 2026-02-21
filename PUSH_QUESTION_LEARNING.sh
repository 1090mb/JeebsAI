#!/usr/bin/env bash
set -e

echo "🧠 Pushing Question Learning System..."
echo ""

git add src/question_learning.rs
git add src/cortex.rs
git add src/lib.rs
git add Cargo.toml
git add QUESTION_LEARNING_SYSTEM.txt

git commit -m "Add Question Learning System - Ask questions, learn answers, remember forever

FEATURE: Jeebs can now ask questions to Google and learn answers

HOW IT WORKS:

1. User asks question in chat
2. Jeebs checks if answer already in memory
3. If not found, searches Google for answer
4. Extracts answer from search results
5. Stores Q&A pair in knowledge base
6. Returns answer to user
7. Remembers for next time!

USAGE EXAMPLES:

Ask questions:
  • 'ask: what is photosynthesis?'
  • 'ask about: how does DNA work?'
  • 'what is machine learning?'

View learned:
  • 'what questions have you asked?'
  • 'recent questions'
  • 'show me what you've learned'

Statistics:
  • 'question statistics'
  • 'how many questions have you learned?'
  • 'q&a stats'

FEATURES:

✅ Web search integration
   • Uses Google search results
   • Extracts featured snippets
   • Falls back to descriptions

✅ Automatic storage
   • Stores Q&A in brain nodes
   • Also stores as knowledge triples
   • Fast lookup in jeebs_store

✅ Memory retrieval
   • Checks memory before searching
   • Instant answers on repeat questions
   • No duplicate web requests

✅ Organization
   • Categorize Q&As
   • Track confidence levels
   • Store source URLs

✅ Statistics
   • Track Q&A count
   • Break down by category
   • Measure learning growth

NEW MODULE: src/question_learning.rs

Structures:
  • LearnedQA - Q&A pair struct

Functions:
  • store_qa_pair() - Save Q&A
  • find_answer_in_memory() - Check memory
  • ask_web_question() - Search web
  • ask_jeebs_question() - Full Q&A flow
  • get_recent_questions() - View history
  • get_qa_statistics() - Get stats
  • get_learned_qa_by_category() - Category search

CHAT INTEGRATION:

Added to cortex.rs:
  • Question asking commands
  • Memory retrieval commands
  • Statistics viewing commands
  • Integrated with knowledge system

DEPENDENCIES:

Added:
  • urlencoding = '2.1' for URL encoding

DATABASE:

No schema changes needed:
  • Uses existing brain_nodes
  • Uses existing knowledge_triples
  • Uses existing jeebs_store

BENEFITS:

✅ Build personal knowledge base
✅ Never forget information again
✅ Fast answer retrieval
✅ Track learning history
✅ Organize knowledge by topic
✅ Exponential knowledge growth

Ready for production!" || echo "Already committed"

git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ QUESTION LEARNING SYSTEM PUSHED!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Deploy on VPS:"
echo ""
echo "  bash update.sh"
echo ""
echo "Then try in chat:"
echo ""
echo "  • 'ask: what is photosynthesis?'"
echo "  • 'what have you learned?'"
echo "  • 'question statistics'"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "🧠 Jeebs can now ask questions and learn answers!"
echo ""
