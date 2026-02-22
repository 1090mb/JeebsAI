# Proposal Template System - Quick Reference

## What Is It?

JeebsAI automatically generates **2 proposals at a time** from 4 template categories:
1. 🎓 **Learning Sprint** - Autonomous learning initiatives
2. 🔧 **Feature/Modification** - New features and improvements
3. 💾 **Data Storage** - Optimization strategies
4. 💡 **Communication/Logic** - Better reasoning and communication

## Quick Start

### View Current Proposals
```bash
curl http://localhost:8080/api/brain/template-proposals
```

### Accept a Proposal
```bash
curl -X POST http://localhost:8080/api/brain/template-proposals/update-status \
  -H "Content-Type: application/json" \
  -d '{
    "proposal_id": "template_feature_3_round123",
    "status": "accepted"
  }'
```

### Track Progress
```bash
curl http://localhost:8080/api/brain/template-proposals/statistics
```

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/brain/template-proposals` | GET | Get current proposals |
| `/api/brain/template-proposals/generate` | POST | Generate new proposals |
| `/api/brain/template-proposals/update-status` | POST | Update proposal status |
| `/api/brain/template-proposals/statistics` | GET | Get progress statistics |

## Proposal Status Flow

```
proposed → accepted → in_progress → completed
            ↓
          rejected
```

## The 4 Template Types

### 1. Learning Sprint (40 hours each)
- Advanced Rust Concurrency
- Machine Learning Fundamentals
- Distributed Systems
- Natural Language Processing
- Database Optimization
- Cloud Architecture

### 2. Features (80 hours each)
- Knowledge Graph Visualization
- Multi-Language Support
- Collaborative Brain Editing
- Conversation Branching
- Plugin Architecture
- Task Management

### 3. Data Storage (120 hours each)
- Compressed Knowledge Graphs
- Smart Data Tiering
- Incremental Backup
- Vector Databases
- Schema Evolution
- Columnar Storage

### 4. Communication (60 hours each)
- Reasoning Chain Framework
- Intent Understanding
- Multi-Modal Communication
- Confidence Quantification
- Response Adaptation
- Consistency Checker

## Proposal Structure

Each proposal includes:
- **Title**: Name of proposal
- **Description**: What it does
- **Implementation Steps**: 5 concrete steps
- **Expected Impact**: Benefits
- **Difficulty**: easy/medium/hard
- **Hours**: Estimated time (40-120)
- **Status**: proposed/accepted/rejected/in_progress/completed

## How Generation Works

**Every hour (3600 seconds):**
1. Randomly pick 2 different template types
2. Randomly pick a template from each type
3. Create complete proposals with steps
4. Store with timestamps and status

**Result:** 2 diverse proposals covering different improvement areas

## Status Transitions

```json
// Propose a feature
{
  "proposal_id": "template_feature_3_round123",
  "status": "proposed"
}

// Accept for implementation
{
  "proposal_id": "template_feature_3_round123",
  "status": "accepted"
}

// Start working on it
{
  "proposal_id": "template_feature_3_round123",
  "status": "in_progress"
}

// Finish
{
  "proposal_id": "template_feature_3_round123",
  "status": "completed"
}

// Or reject
{
  "proposal_id": "template_learning_sprint_2_round123",
  "status": "rejected"
}
```

## Statistics

```bash
curl http://localhost:8080/api/brain/template-proposals/statistics
```

Returns:
- Total proposals generated
- How many accepted/rejected/in_progress/completed
- Total estimated hours remaining
- Current selection round
- Details of each proposal

## Common Tasks

### View All Current Proposals
```bash
curl http://localhost:8080/api/brain/template-proposals | jq .proposals
```

### Accept a Proposal
```bash
curl -X POST http://localhost:8080/api/brain/template-proposals/update-status \
  -d '{"proposal_id":"template_feature_1_round123","status":"accepted"}'
```

### Mark as In Progress
```bash
curl -X POST http://localhost:8080/api/brain/template-proposals/update-status \
  -d '{"proposal_id":"template_learning_sprint_2_round123","status":"in_progress"}'
```

### Mark as Completed
```bash
curl -X POST http://localhost:8080/api/brain/template-proposals/update-status \
  -d '{"proposal_id":"template_feature_3_round123","status":"completed"}'
```

### Reject a Proposal
```bash
curl -X POST http://localhost:8080/api/brain/template-proposals/update-status \
  -d '{"proposal_id":"template_data_storage_1_round123","status":"rejected"}'
```

### Get Stats
```bash
curl http://localhost:8080/api/brain/template-proposals/statistics | jq .statistics
```

### Generate New Proposals Now
```bash
curl -X POST http://localhost:8080/api/brain/template-proposals/generate
```

## Key Features

✅ Balanced distribution of proposal types
✅ Concrete implementation steps for each proposal
✅ Time estimates for planning
✅ Progress tracking with status transitions
✅ Difficulty levels (easy/medium/hard)
✅ Database persistence
✅ REST API for integration
✅ Extensible template system

## Benefits

1. **Systematic Improvement** - Organized self-improvement
2. **Diverse Focus** - Covers learning, features, storage, communication
3. **Clear Roadmap** - Each proposal has 5 implementation steps
4. **Progress Tracking** - See what's been completed
5. **Flexible** - Easy to add new templates
6. **Effort Planning** - Know how many hours each takes

## Example Workflow

```
Day 1:
  - Check proposals: GET /api/brain/template-proposals
  - See: Learning Sprint + Data Storage proposals
  - Accept both: POST with status="accepted"

Day 2-5:
  - Start work: POST with status="in_progress"

Day 20+:
  - Finish: POST with status="completed"
  - Check stats: GET /api/brain/template-proposals/statistics

Day 21:
  - New proposals generated automatically
  - Repeat process
```

## Full Documentation

See **PROPOSAL_TEMPLATE_SYSTEM.md** for complete details including:
- How to add new proposal templates
- Detailed configuration options
- Database schema information
- Extension examples

---

**Quick Start**: `curl http://localhost:8080/api/brain/template-proposals`
