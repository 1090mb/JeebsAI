# Brain Parser Integration Guide

## Quick Start

The Brain Parsing Framework has been integrated into JeebsAI. Here's how to use it:

## Installation

The framework is already integrated into the codebase:

1. **Module Registration** (`src/lib.rs`): ✅ Complete
2. **API Endpoints** (`src/main.rs`): ✅ Registered
3. **Core Parser** (`src/brain_parser.rs`): ✅ Implemented
4. **API Handlers** (`src/brain_parsing_api.rs`): ✅ Implemented

## Building

```bash
# Build the project with brain parser support
cargo build --release

# Run tests
cargo test brain_parser
```

## Accessing Brain Parsing Features

### Via Web API

#### 1. Parse a Single Node
```bash
curl -X POST http://localhost:8080/api/brain/parse \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "brain_001",
    "key": "Machine Learning",
    "value": "ML is a subset of artificial intelligence that enables systems to learn from data"
  }'
```

#### 2. Build Knowledge Graph
```bash
curl -X POST http://localhost:8080/api/brain/graph/build
```

#### 3. Query Knowledge Graph
```bash
# By entity
curl -X POST http://localhost:8080/api/brain/graph/query/entity \
  -H "Content-Type: application/json" \
  -d '{"entity": "Machine Learning"}'

# By category
curl -X POST http://localhost:8080/api/brain/graph/query/category \
  -H "Content-Type: application/json" \
  -d '{"category": "Technology"}'
```

#### 4. Get Statistics
```bash
curl http://localhost:8080/api/brain/graph/statistics
```

#### 5. Get Relationships
```bash
curl http://localhost:8080/api/brain/graph/relationships
```

#### 6. Get Entities Report
```bash
curl http://localhost:8080/api/brain/graph/entities
```

### Via Rust Code

```rust
use jeebs::brain_parser::{BrainParser, build_knowledge_graph};

// Create parser
let parser = BrainParser::new();

// Parse a node
let parsed = parser.parse(
    "node_1".to_string(),
    "Rust".to_string(),
    "Systems programming language".to_string(),
);

// Access parsed data
for entity in &parsed.extracted_entities {
    println!("Entity: {} (Type: {:?})", entity.value, entity.entity_type);
}

// Build full knowledge graph
let graph = build_knowledge_graph(&db, &parser).await?;

// Query the graph
let results = graph.query_by_entity("Rust");
println!("Found {} nodes containing 'Rust'", results.len());
```

## Integration Points

### 1. With Existing Knowledge Retrieval
The parser complements the existing `knowledge_retrieval.rs` by providing:
- Structured entity extraction
- Relationship mapping
- Category organization
- Enhanced ranking via semantic understanding

### 2. With Language Learning
The parser can feed extracted entities and patterns to:
- Vocabulary learning (entity values)
- Pattern recognition (relationship types)
- Topic modeling (category inference)

### 3. With Cortex Training
Use parser results for:
- Focused learning on specific entities
- Relationship-based knowledge organization
- Category-targeted training cycles

## Example: Enhanced Knowledge Retrieval

```rust
use jeebs::brain_parser::BrainParser;
use jeebs::knowledge_retrieval::retrieve_knowledge;

pub async fn enhanced_knowledge_search(
    db: &SqlitePool,
    query: &str,
) -> Result<EnhancedResults, String> {
    // Get traditional search results
    let retrieval = retrieve_knowledge(db, query, 10).await?;
    
    // Build parser for semantic analysis
    let parser = BrainParser::new();
    let graph = build_knowledge_graph(db, &parser).await?;
    
    // Enhance results with semantic information
    let mut enhanced = retrieval;
    
    // Parse the query itself to understand intent
    let query_parsed = parser.parse(
        "query".to_string(),
        "search_query".to_string(),
        query.to_string(),
    );
    
    // Use extracted query entities to find related nodes
    for entity in &query_parsed.extracted_entities {
        let related = graph.query_by_entity(&entity.value);
        enhanced.related_node_ids = related;
    }
    
    Ok(enhanced)
}
```

## Configuration

The parser uses sensible defaults but can be customized:

### Custom Entity Patterns
Modify `BrainParser::build_entity_patterns()` to add new patterns:

```rust
// Add custom pattern for URLs
patterns.insert(
    EntityType::Custom("URL".to_string()),
    vec![
        Regex::new(r"https?://[^\s]+").unwrap(),
    ],
);
```

### Custom Categories
Modify `BrainParser::build_category_keywords()`:

```rust
keywords.insert(
    "Medical".to_string(),
    vec!["disease", "treatment", "diagnosis", "medicine"]
        .iter()
        .map(|s| s.to_string())
        .collect(),
);
```

## Performance Tips

1. **Cache Graph Building**: Build graph once, reuse for multiple queries
   ```rust
   let cached_graph = Arc::new(build_knowledge_graph(db, &parser).await?);
   ```

2. **Batch Processing**: Process multiple nodes concurrently
   ```rust
   let futures = nodes.iter().map(|node| {
       async { parser.parse(node.id, node.key, node.value) }
   });
   ```

3. **Lazy Evaluation**: Query only needed subsets of the graph

4. **Index Optimization**: Ensure database has indexes on frequently queried columns
   ```sql
   CREATE INDEX idx_brain_key ON brain(key);
   CREATE INDEX idx_brain_value ON brain(value);
   ```

## Testing the Integration

```bash
# Run all tests
cargo test

# Run parser-specific tests
cargo test brain_parser

# Run integration tests (if applicable)
cargo test --test integration_tests
```

## Troubleshooting

### Parser Not Finding Entities
- Check regex patterns are correct
- Verify input text quality
- Increase confidence threshold if too strict

### Knowledge Graph Build Slow
- Check database query performance
- Consider caching results
- Profile with larger datasets

### Entity Classification Incorrect
- Review and adjust confidence scores
- Add more specific patterns
- Consider implementing ML-based classification

## Next Steps

1. **Monitor Performance**: Track query times and graph size
2. **Gather Metrics**: Collect entity and relationship statistics
3. **Iterate Patterns**: Refine patterns based on real data
4. **Add ML**: Implement machine learning for improved classification
5. **Visualize**: Create web UI for graph visualization

## Documentation

- **Full Framework Docs**: See `BRAIN_PARSING_FRAMEWORK.md`
- **API Reference**: See inline code comments in `src/brain_parser.rs` and `src/brain_parsing_api.rs`
- **Examples**: Check this guide and test cases

## Support

For issues or questions:
1. Check the main `BRAIN_PARSING_FRAMEWORK.md` documentation
2. Review inline code comments
3. Check test cases for usage examples
4. Review git history for changes

---

**Ready to use!** The brain parsing framework is fully integrated and ready for use. Start with the API endpoints or use it directly in Rust code.
