# JeebsAI Proposal Template System

## Overview

JeebsAI now generates **2 proposals at a time** from 4 template types, enabling systematic self-improvement suggestions across different dimensions of capability enhancement.

## The 4 Proposal Template Types

### 1. 🎓 Autonomous Learning Sprint
Comprehensive learning initiatives covering cutting-edge topics and foundational knowledge areas.

**Current Sprint Topics:**
- Advanced Rust Concurrency Patterns
- Machine Learning Fundamentals
- Distributed Systems Architecture
- Natural Language Processing Deep Dive
- Database Performance Optimization
- Cloud Architecture Patterns

**Characteristics:**
- Difficulty: Hard
- Estimated Time: 40 hours
- Focus: Knowledge accumulation and mastery
- Output: Enhanced understanding and capabilities

### 2. 🔧 Feature/Modification
New features and system improvements that enhance user experience and system functionality.

**Current Feature Ideas:**
- Advanced Knowledge Graph Visualization
- Multi-Language Support System
- Real-Time Collaborative Brain Editing
- Conversation Branching and Exploration
- Plugin Architecture Expansion
- Advanced Scheduling and Task Management

**Characteristics:**
- Difficulty: Medium
- Estimated Time: 80 hours
- Focus: User experience and capability expansion
- Output: New features and system improvements

### 3. 💾 Data Storage Optimization
Strategies for efficiently storing, retrieving, and managing large amounts of data.

**Current Optimization Ideas:**
- Compressed Knowledge Graph Storage
- Smart Data Tiering System
- Incremental Backup and Delta Compression
- Vector Database Integration
- Schema Evolution and Migration Framework
- Columnar Storage for Analytics

**Characteristics:**
- Difficulty: Hard
- Estimated Time: 120 hours
- Focus: Efficiency and scalability
- Output: Better storage and performance

### 4. 💡 Communication and Logic Improvement
Better reasoning, understanding, and communication capabilities.

**Current Improvement Ideas:**
- Enhanced Reasoning Chain Framework
- Semantic Intent Understanding
- Multi-Modal Communication Bridge
- Uncertainty and Confidence Quantification
- Context-Aware Response Adaptation
- Logical Consistency Checker

**Characteristics:**
- Difficulty: Medium
- Estimated Time: 60 hours
- Focus: Quality of thinking and communication
- Output: Smarter, more coherent responses

---

## How It Works

### Generation Process

1. **Selection**: Every hour (configurable), JeebsAI selects **2 different proposal types** from the 4 available
2. **Random Picking**: For each selected type, a random template is chosen from that category
3. **Proposal Creation**: Complete proposals with implementation steps are generated
4. **Status Tracking**: Each proposal tracks its status: proposed → accepted/rejected → in_progress → completed

### Proposal Structure

Each proposal includes:
- **ID**: Unique identifier (format: `template_{type}_{index}_round{n}`)
- **Type**: One of the 4 template types
- **Title**: Concise name of the proposal
- **Description**: Detailed explanation of what it is
- **Implementation Steps**: List of 5 concrete steps to implement
- **Expected Impact**: Description of expected improvements
- **Difficulty Level**: easy, medium, or hard
- **Estimated Hours**: Time needed for implementation
- **Status**: Current state (proposed, accepted, rejected, in_progress, completed)
- **Created At**: Timestamp of proposal generation

### Example Generation Round

**Round 123 Proposals** (generated at specific interval):
1. **Learning Sprint**: "Advanced Rust Concurrency Patterns" (40 hours)
2. **Data Storage**: "Smart Data Tiering System" (120 hours)

**Next Round** (1 hour later):
1. **Feature**: "Knowledge Graph Visualization" (80 hours)
2. **Communication**: "Enhanced Reasoning Chain Framework" (60 hours)

---

## API Endpoints

### Generate New Proposals
```
POST /api/brain/template-proposals/generate
```

**Response:**
```json
{
  "success": true,
  "proposals": [
    {
      "id": "template_learning_sprint_2_round123",
      "type": "learning_sprint",
      "title": "Advanced Rust Concurrency Patterns",
      "description": "Deep dive into tokio, async/await, and concurrent data structures...",
      "steps": [
        "Study tokio runtime architecture and task scheduling",
        "Master async/await patterns and future composition",
        ...
      ],
      "impact": "Significant improvement in learning and knowledge capabilities...",
      "difficulty": "hard",
      "estimated_hours": 40,
      "status": "proposed",
      "created_at": "2026-02-21T15:30:00Z"
    },
    ...
  ],
  "selection_round": 123,
  "message": "🎯 **Current Proposal Round**...\n..."
}
```

### Get Current Proposals
```
GET /api/brain/template-proposals
```

Returns the currently active proposals or generates new ones if none exist.

### Update Proposal Status
```
POST /api/brain/template-proposals/update-status

{
  "proposal_id": "template_feature_3_round123",
  "status": "accepted"
}
```

**Valid Statuses:**
- `proposed` - Initial state
- `accepted` - Approved for implementation
- `rejected` - Decided not to pursue
- `in_progress` - Currently being worked on
- `completed` - Finished

### Get Proposal Statistics
```
GET /api/brain/template-proposals/statistics
```

**Response:**
```json
{
  "success": true,
  "statistics": {
    "total_proposals": 12,
    "accepted": 4,
    "in_progress": 2,
    "completed": 1,
    "rejected": 5,
    "total_estimated_hours": 340,
    "selection_round": 123,
    "proposals": [
      {
        "id": "template_learning_sprint_2_round120",
        "title": "Advanced Rust Concurrency Patterns",
        "type": "learning_sprint",
        "status": "in_progress",
        "difficulty": "hard",
        "hours": 40
      },
      ...
    ]
  }
}
```

---

## Usage Examples

### Get All Current Proposals
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

### Move to In Progress
```bash
curl -X POST http://localhost:8080/api/brain/template-proposals/update-status \
  -H "Content-Type: application/json" \
  -d '{
    "proposal_id": "template_learning_sprint_2_round123",
    "status": "in_progress"
  }'
```

### Mark as Complete
```bash
curl -X POST http://localhost:8080/api/brain/template-proposals/update-status \
  -H "Content-Type: application/json" \
  -d '{
    "proposal_id": "template_data_storage_1_round120",
    "status": "completed"
  }'
```

### Get Statistics
```bash
curl http://localhost:8080/api/brain/template-proposals/statistics
```

---

## Implementation Details

### Configuration

Located in `src/proposals.rs`:

```rust
const TEMPLATE_PROPOSAL_KEY: &str = "jeebs:template_proposals";
const TEMPLATE_PROPOSAL_INTERVAL_SECS: i64 = 3600; // 1 hour
```

**Customization Options:**
- Adjust interval for how often proposals are generated
- Add new templates to any of the 4 categories
- Change difficulty and estimated hours per proposal type
- Modify proposal selection strategy

### Data Storage

Proposals are stored in the `jeebs_store` database table:
- **Key**: `jeebs:template_proposals`
- **Value**: Serialized JSON of `TemplateProposalSet`

**Structure:**
```rust
pub struct TemplateProposalSet {
    pub proposals: Vec<TemplateProposal>,
    pub created_at: String,
    pub selection_round: u32,
}

pub struct TemplateProposal {
    pub id: String,
    pub template_type: String,
    pub title: String,
    pub description: String,
    pub implementation_steps: Vec<String>,
    pub expected_impact: String,
    pub difficulty_level: String,
    pub estimated_time_hours: u32,
    pub created_at: String,
    pub status: String,
}
```

### Selection Algorithm

1. Generate 2 random template types from the 4 available
2. Ensure they are different
3. For each selected type, randomly pick a template from that category
4. Create proposal with implementation steps, estimated time, etc.
5. Store in database with creation timestamp

---

## Extending the System

### Add New Learning Sprint Topic

Edit `src/proposals.rs`, `LEARNING_SPRINT_TEMPLATES`:

```rust
const LEARNING_SPRINT_TEMPLATES: &[(&str, &str, &[&str])] = &[
    // ... existing entries ...
    (
        "Your Topic Title",
        "Description of what this sprint covers",
        &[
            "Step 1 description",
            "Step 2 description",
            "Step 3 description",
            "Step 4 description",
            "Step 5 description",
        ],
    ),
];
```

### Add New Feature Idea

Edit `src/proposals.rs`, `FEATURE_TEMPLATES`:

```rust
const FEATURE_TEMPLATES: &[(&str, &str, &[&str])] = &[
    // ... existing entries ...
    (
        "Feature Name",
        "What this feature does and why it's valuable",
        &[
            "Implementation step 1",
            "Implementation step 2",
            "Implementation step 3",
            "Implementation step 4",
            "Implementation step 5",
        ],
    ),
];
```

### Add New Storage Optimization

Edit `src/proposals.rs`, `DATA_STORAGE_TEMPLATES`:

```rust
const DATA_STORAGE_TEMPLATES: &[(&str, &str, &[&str])] = &[
    // ... existing entries ...
    (
        "Storage Optimization Name",
        "How this improves data storage efficiency",
        &[
            "Research or design step 1",
            "Implementation step 2",
            "Optimization step 3",
            "Testing step 4",
            "Validation step 5",
        ],
    ),
];
```

### Add New Communication Improvement

Edit `src/proposals.rs`, `COMMUNICATION_LOGIC_TEMPLATES`:

```rust
const COMMUNICATION_LOGIC_TEMPLATES: &[(&str, &str, &[&str])] = &[
    // ... existing entries ...
    (
        "Communication Improvement",
        "How this enhances reasoning or communication",
        &[
            "Design step 1",
            "Implementation step 2",
            "Testing step 3",
            "Refinement step 4",
            "Integration step 5",
        ],
    ),
];
```

---

## Key Features

✅ **Balanced Selection** - 2 proposals from different types ensure diverse improvements
✅ **Comprehensive Details** - Each proposal includes concrete implementation steps
✅ **Time Estimates** - Helps prioritize based on available resources
✅ **Status Tracking** - Monitor progress from proposed to completed
✅ **Difficulty Levels** - Plan accordingly based on complexity
✅ **Flexible Categories** - Easy to add new proposals to any template type
✅ **Stateful Management** - Database persistence ensures progress tracking
✅ **API-Driven** - Full REST API for integration with other systems

---

## Usage Workflow

### 1. Check Current Proposals
```bash
curl http://localhost:8080/api/brain/template-proposals
```

### 2. Review and Accept
Examine the 2 proposals and decide which to pursue

### 3. Update Status
```bash
# Accept for implementation
curl -X POST http://localhost:8080/api/brain/template-proposals/update-status \
  -d '{"proposal_id": "...", "status": "accepted"}'
```

### 4. Start Work
Move to `in_progress` once you begin implementation

### 5. Complete
Mark as `completed` when finished

### 6. Track Progress
```bash
curl http://localhost:8080/api/brain/template-proposals/statistics
```

### 7. New Round
After ~1 hour, new proposals are automatically generated

---

## Benefits

1. **Systematic Improvement** - Organized approach to capability enhancement
2. **Diverse Focus** - Addresses learning, features, storage, and communication
3. **Clear Roadmap** - Each proposal has specific implementation steps
4. **Trackable Progress** - Monitor what's been completed
5. **Scalable** - Easy to add new proposal templates
6. **Data-Driven** - Make decisions based on effort and impact estimates

---

## Next Steps

1. Review current proposals: `GET /api/brain/template-proposals`
2. Accept proposals you want to pursue: `POST /api/brain/template-proposals/update-status`
3. Track progress: `GET /api/brain/template-proposals/statistics`
4. Update status as you progress through implementation
5. Complete proposals and generate new rounds

---

**Version**: 1.0
**Created**: February 21, 2026
**Status**: Ready to use
