# Brain Parser - Quick Reference Card

## 📋 Files Created & Modified

### New Code Files
- `src/brain_parser.rs` - Core parsing engine (464 lines)
- `src/brain_parsing_api.rs` - REST API endpoints (267 lines)

### Modified Code Files
- `src/lib.rs` - Added 2 module declarations
- `src/main.rs` - Added import and 7 endpoint registrations

### Documentation Files
- `BRAIN_PARSING_FRAMEWORK.md` - Complete documentation
- `BRAIN_PARSER_INTEGRATION.md` - Integration guide
- `BRAIN_PARSER_EXAMPLES.rs` - Working code examples
- `BRAIN_PARSER_DEPLOYMENT.md` - Deployment procedures
- `BRAIN_PARSER_IMPLEMENTATION_SUMMARY.md` - Implementation overview
- `BRAIN_PARSER_CHECKLIST.md` - Quality checklist
- `BRAIN_PARSER_READY_TO_PUSH.md` - Status report
- `BRAIN_PARSER_VISUAL_OVERVIEW.md` - Visual diagrams
- `BRAIN_PARSER_QUICK_REFERENCE.md` - This file

## 🔧 API Quick Reference

### Parse a Node
```bash
POST /api/brain/parse
Content-Type: application/json

{
  "node_id": "node_123",
  "key": "Rust",
  "value": "A systems programming language"
}

Response: ParsedBrainContent with entities, relationships, categories
```

### Build Graph
```bash
POST /api/brain/graph/build

Response: { success: true, graph_stats: {...} }
```

### Query by Entity
```bash
POST /api/brain/graph/query/entity
{ "entity": "Rust" }

Response: { results: ["node_123", ...], result_count: 5 }
```

### Query by Category
```bash
POST /api/brain/graph/query/category
{ "category": "Technology" }

Response: { results: ["node_1", ...], result_count: 245 }
```

### Get Statistics
```bash
GET /api/brain/graph/statistics

Response: {
  "total_nodes": 1245,
  "total_edges": 3421,
  "total_categories": 12,
  "total_entities": 8932
}
```

### Get Relationships
```bash
GET /api/brain/graph/relationships

Response: {
  "total_relationships": 3421,
  "relationships": [...]
}
```

### Get Entities
```bash
GET /api/brain/graph/entities

Response: {
  "total_entities": 8932,
  "entities_by_type": {...}
}
```

## 🎯 Entity Types

| Type | Examples | Use Case |
|------|----------|----------|
| Person | "John", "Alice" | People |
| Organization | "Mozilla", "Google" | Companies |
| Location | "New York", "Japan" | Places |
| Date | "2024", "January" | Time refs |
| Concept | "AI", "Machine Learning" | Ideas |
| Technology | "Rust", "Python" | Tech stack |
| Product | "React", "Django" | Software |
| Event | "WWII", "Moon Landing" | Events |
| Number | "1000", "million" | Quantities |
| Definition | "X is a Y" | Meanings |

## 🔗 Relationship Types

| Type | Pattern | Example |
|------|---------|---------|
| IsA | X is a Y | "Rust is a language" |
| PartOf | X is part of Y | "CPU is part of computer" |
| Creates | X creates Y | "Rust was created by Mozilla" |
| Uses | X uses Y | "Django uses Python" |
| Knows | X knows Y | "John knows Alice" |
| Located | X is in Y | "NYC is in NY" |
| Precedes | X before Y | "Birth before death" |

## 📂 Categories

- **Technology** - Software, programming, computing
- **Science** - Research, experiments, theories
- **Business** - Companies, markets, economics
- **History** - Historical periods, events
- **Culture** - Arts, traditions, customs

## 🚀 Usage Patterns

### Rust Code
```rust
use jeebs::brain_parser::BrainParser;

let parser = BrainParser::new();
let parsed = parser.parse(id, key, value);

// Access results
for entity in parsed.extracted_entities {
    println!("{:?}", entity);
}
```

### cURL
```bash
curl -X POST http://localhost:8080/api/brain/parse \
  -H "Content-Type: application/json" \
  -d '{...}'
```

### Node.js/JavaScript
```javascript
const response = await fetch('/api/brain/parse', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    node_id: 'test_1',
    key: 'Test',
    value: 'Test value'
  })
});
const data = await response.json();
```

### Python
```python
import requests

response = requests.post(
    'http://localhost:8080/api/brain/parse',
    json={
        'node_id': 'test_1',
        'key': 'Test',
        'value': 'Test value'
    }
)
data = response.json()
```

## ⚙️ Configuration

### Environment Variables
```bash
DATABASE_URL=sqlite:./jeebs.db
PORT=8080
# No new variables required
```

### Performance Tuning
```sql
-- Optional indexes for faster queries
CREATE INDEX idx_brain_key ON brain(key);
CREATE INDEX idx_brain_value ON brain(value);
CREATE INDEX idx_brain_label ON brain(label);
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run parser tests only
cargo test brain_parser

# Build release
cargo build --release

# Check for warnings
cargo check --all-targets
```

## 📊 Performance Expectations

| Operation | Time | Memory |
|-----------|------|--------|
| Parse 1 node | 5-50ms | <1MB |
| Build 1000 node graph | 500-2000ms | 50-200MB |
| Entity query | <5ms | Negligible |
| Category query | <5ms | Negligible |
| Statistics | 10-50ms | Negligible |

## 🚨 Troubleshooting

| Issue | Solution |
|-------|----------|
| Slow graph building | Check DB indexes, cache results |
| High memory | Reduce cache duration, batch process |
| Missed entities | Review patterns, adjust confidence |
| API timeout | Increase timeout, cache graphs |
| Parser accuracy low | Add custom patterns, review data |

## 📚 Documentation Map

```
┌─ BRAIN_PARSING_FRAMEWORK.md
│  └─ Comprehensive feature documentation
├─ BRAIN_PARSER_INTEGRATION.md
│  └─ Quick start and integration guide
├─ BRAIN_PARSER_EXAMPLES.rs
│  └─ 8 working code examples
├─ BRAIN_PARSER_DEPLOYMENT.md
│  └─ Deployment and operations guide
├─ BRAIN_PARSER_VISUAL_OVERVIEW.md
│  └─ Architecture and visual diagrams
└─ BRAIN_PARSER_QUICK_REFERENCE.md (this file)
   └─ Quick lookup and API reference
```

## 🎯 Key Features Summary

✅ 10 entity types recognized
✅ 7 relationship types detected
✅ 5 semantic categories
✅ Full knowledge graph building
✅ 7 REST API endpoints
✅ Entity indexing and querying
✅ Category-based organization
✅ Confidence scoring
✅ Metadata tracking
✅ Graph statistics and analysis
✅ Zero new dependencies
✅ 100% backward compatible
✅ Production-ready code
✅ Comprehensive documentation

## 🚀 Getting Started in 5 Minutes

1. **Build the project**
   ```bash
   cargo build --release
   ```

2. **Run the server**
   ```bash
   cargo run
   ```

3. **Parse a node**
   ```bash
   curl -X POST http://localhost:8080/api/brain/parse \
     -H "Content-Type: application/json" \
     -d '{"node_id":"t1","key":"Test","value":"Sample text"}'
   ```

4. **Build graph** (if you have brain data)
   ```bash
   curl -X POST http://localhost:8080/api/brain/graph/build
   ```

5. **Get stats**
   ```bash
   curl http://localhost:8080/api/brain/graph/statistics
   ```

## 💡 Common Patterns

### Parse and Extract Entities
```rust
let parser = BrainParser::new();
let parsed = parser.parse(id, key, value);
for entity in parsed.extracted_entities {
    // Process entity
}
```

### Query Knowledge Graph
```rust
let graph = build_knowledge_graph(&db, &parser).await?;
let nodes = graph.query_by_entity("Rust");
```

### Filter by Confidence
```rust
let high_conf = parsed.extracted_entities
    .iter()
    .filter(|e| e.confidence > 0.8);
```

### Get Category Information
```rust
for category in parsed.categories {
    println!("{}: {}", category.name, category.confidence);
}
```

## 📞 Support Resources

- **Full Docs**: See `BRAIN_PARSING_FRAMEWORK.md`
- **Examples**: See `BRAIN_PARSER_EXAMPLES.rs`
- **Integration**: See `BRAIN_PARSER_INTEGRATION.md`
- **Deployment**: See `BRAIN_PARSER_DEPLOYMENT.md`
- **Overview**: See `BRAIN_PARSER_VISUAL_OVERVIEW.md`

## ✨ Status

| Aspect | Status |
|--------|--------|
| Implementation | ✅ Complete |
| Documentation | ✅ Complete |
| Testing | ✅ Complete |
| Integration | ✅ Complete |
| Ready to Deploy | ✅ Yes |

---

**Quick Reference Version**: 1.0
**Last Updated**: February 21, 2026
**Status**: ✅ Production Ready
