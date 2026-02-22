# Deep Learning System - Quick Reference

## What It Does

Makes JeebsAI:
1. **Learn Deeper** - Spend 40+ hours on topics
2. **Remember Better** - Store detailed facts with sources
3. **Use Knowledge** - Automatically include learned facts in chat responses
4. **Track Progress** - See expertise levels grow from novice to expert

---

## Quick Start

### 1. Start Learning
```bash
curl -X POST http://localhost:8080/api/learning/start-deep-learning \
  -H "Content-Type: application/json" \
  -d '{"topic": "Rust Programming"}'

# Returns session_id
```

### 2. Add Facts While Learning
```bash
curl -X POST http://localhost:8080/api/learning/add-fact \
  -d '{
    "session_id": "your-session-id",
    "fact": "Rust prevents null pointer exceptions at compile time",
    "source": "The Rust Book",
    "importance": 0.95
  }'
```

### 3. Add Practice Problems
```bash
curl -X POST http://localhost:8080/api/learning/add-practice-problem \
  -d '{
    "session_id": "your-session-id",
    "problem": "Implement a linked list in Rust",
    "solution": "struct Node { value: i32, next: Option<Box<Node>> }",
    "explanation": "This shows Rust ownership...",
    "difficulty": "hard"
  }'
```

### 4. Check Progress
```bash
curl http://localhost:8080/api/learning/statistics
```

---

## 6 API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/learning/start-deep-learning` | POST | Begin learning a topic |
| `/api/learning/add-fact` | POST | Add a fact to learning session |
| `/api/learning/add-practice-problem` | POST | Add practice problem for deeper learning |
| `/api/learning/sessions` | GET | View all learning sessions |
| `/api/learning/statistics` | GET | Get detailed learning stats |
| `/api/learning/summary` | GET | Get human-readable summary |

---

## Learning Progression

```
Novice        Learning      Intermediate    Advanced       Expert
 0-5h         5-15h         15-30h          30-50h         50h+
 Level 1      Level 2       Level 3         Level 4        Level 5
```

---

## Subtopics Generated Automatically

### Rust Topic
- Ownership and borrowing
- Lifetimes
- Traits and generics
- Memory safety
- Concurrency and async

### Machine Learning Topic
- Neural networks
- Supervised learning
- Unsupervised learning
- Deep learning
- Reinforcement learning

### Database Topic
- Relational models
- Indexing strategies
- Query optimization
- Transactions
- Replication

---

## Example: Learning Rust in Depth

### Hour 1: Start and Add Initial Facts
```bash
# Start session
curl -X POST http://localhost:8080/api/learning/start-deep-learning \
  -d '{"topic": "Rust Programming"}'

# Add facts about ownership
curl -X POST http://localhost:8080/api/learning/add-fact \
  -d '{
    "session_id": "sess-123",
    "fact": "Every value has a single owner in Rust",
    "source": "The Rust Book Chapter 4",
    "importance": 1.0
  }'

# Status: novice (1 hour)
```

### Hours 2-10: Add More Facts
```bash
# Add facts about different aspects
curl -X POST http://localhost:8080/api/learning/add-fact \
  -d '{
    "session_id": "sess-123",
    "fact": "Borrowing allows temporary access without taking ownership",
    "source": "The Rust Book Chapter 4",
    "importance": 0.95
  }'

# Status: learning (8 hours)
```

### Hours 11-25: Add Practice Problems
```bash
# Practice with code
curl -X POST http://localhost:8080/api/learning/add-practice-problem \
  -d '{
    "session_id": "sess-123",
    "problem": "Write a function that borrows a string",
    "solution": "fn print_string(s: &String) { println!(\"{}\", s); }",
    "explanation": "The & creates a borrow, not transferring ownership",
    "difficulty": "easy"
  }'

# Status: intermediate (20 hours)
```

### Hours 26-40: Deep Dive
```bash
# Add advanced problems
curl -X POST http://localhost:8080/api/learning/add-practice-problem \
  -d '{
    "session_id": "sess-123",
    "problem": "Implement a generic binary search tree in Rust",
    "solution": "[complex implementation]",
    "explanation": "This combines ownership, borrowing, and generics...",
    "difficulty": "hard"
  }'

# Status: advanced (35 hours)
```

### After 40+ Hours: Expert Status
```bash
# Check final status
curl http://localhost:8080/api/learning/statistics

{
  "topic": "Rust Programming",
  "status": "expert",
  "depth_level": 5,
  "study_hours": 42.5,
  "facts_learned": 28,
  "confidence": 0.89
}
```

### Then Chat About Rust
```
User: "How do I handle ownership in Rust?"

System detects: "Rust" topic
Retrieves facts: 
  - Every value has a single owner
  - Borrowing allows temporary access
  - References can be mutable or immutable
  
Enhanced Response:
"[Original response]

**From my knowledge:**
1. Every value has a single owner in Rust
2. Borrowing allows temporary access without taking ownership
3. References can be either mutable (&mut T) or immutable (&T)

**My expertise in related areas:**
1. Rust Programming (Level 5/5)"
```

---

## Key Metrics

### Per Topic
- **Expertise Level**: 1-10 (how good at this topic)
- **Study Hours**: Total time spent learning
- **Facts Learned**: Number of facts stored
- **Confidence**: Measured mastery (0.0-1.0)
- **Status**: novice/learning/intermediate/advanced/expert
- **Applications**: How many times used in chat

### Overall
- **Total Sessions**: Number of topics being learned
- **Total Study Hours**: Cumulative time invested
- **Total Facts**: Overall knowledge base
- **Average Confidence**: Overall expertise level

---

## Facts Include

- **The fact** itself
- **Source** (where learned from)
- **Importance** (0.0-1.0 scale)
- **Usage count** (how often used in responses)
- **Related concepts** (linked knowledge)

---

## Practice Problems Have

- **Problem statement**
- **Solution code/answer**
- **Explanation** (why this solution)
- **Difficulty**: easy (1h) / medium (2h) / hard (4h)
- **Solved flag** (for tracking)
- **Attempt count** (how many tries)

---

## Smart Features

✅ **Automatic Subtopic Generation** - System breaks down topics
✅ **Knowledge Gap Identification** - Knows what's not learned
✅ **Concept Linking** - Connects related facts
✅ **Usage Tracking** - Records when facts are used
✅ **Confidence Measurement** - Tracks mastery level
✅ **Progressive Difficulty** - Easy → Medium → Hard problems
✅ **Progress Visualization** - See growth from novice to expert

---

## Fact Usage Example

When user asks about learned topic:

```
User: "Tell me about distributed systems"

System:
1. Detects "distributed systems" topic
2. Retrieves 5 most relevant facts
3. Checks expertise level (4/10)
4. Adds facts to response
5. Records usage stats

Response includes:
- Original answer
- 3-5 learned facts
- Expertise level indicator
```

---

## Study Time Estimates

- **Easy Problem**: 1 hour added
- **Medium Problem**: 2 hours added
- **Hard Problem**: 4 hours added
- **Each Fact**: 0.5 hours added

---

## Commands Cheat Sheet

```bash
# Start learning
curl -X POST http://localhost:8080/api/learning/start-deep-learning \
  -d '{"topic":"Topic Name"}'

# Add fact
curl -X POST http://localhost:8080/api/learning/add-fact \
  -d '{"session_id":"id","fact":"text","source":"cite","importance":0.8}'

# Add problem
curl -X POST http://localhost:8080/api/learning/add-practice-problem \
  -d '{"session_id":"id","problem":"q","solution":"a","explanation":"why","difficulty":"easy"}'

# View sessions
curl http://localhost:8080/api/learning/sessions

# Get stats
curl http://localhost:8080/api/learning/statistics

# Get summary
curl http://localhost:8080/api/learning/summary
```

---

**Status**: ✅ Ready to Use
**Documentation**: DEEP_LEARNING_SYSTEM.md (full guide)
