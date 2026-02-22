# Brain Parsing Framework

## Overview

The Brain Parsing Framework provides a comprehensive system for JeebsAI to parse, analyze, and organize information stored in its brain. It enables automatic extraction of structured knowledge from unstructured brain node data, building a complete knowledge graph that can be queried and analyzed.

## Features

### 1. **Entity Extraction**
Automatically identifies and extracts various types of entities from brain content:
- **Person**: Named individuals
- **Organization**: Companies, groups, institutions
- **Location**: Geographic locations and places
- **Date**: Temporal references and time periods
- **Concept**: Abstract ideas and concepts
- **Technology**: Programming languages, frameworks, tools
- **Product**: Software products and services
- **Event**: Historical and current events
- **Number**: Quantities and numerical values
- **Definition**: Formal definitions and explanations

Each extracted entity includes:
- Entity type classification
- Confidence score
- Contextual information

### 2. **Relationship Detection**
Identifies and maps relationships between entities:
- **IsA**: "X is a Y" (classification)
- **PartOf**: "X is part of Y" (composition)
- **Creates**: "X creates Y" (authorship/creation)
- **Uses**: "X uses Y" (usage/dependency)
- **Knows**: "X knows Y" (social connections)
- **Located**: "X is located in Y" (spatial relationships)
- **Precedes**: "X happens before Y" (temporal relationships)

### 3. **Category Inference**
Automatically categorizes content into semantic categories:
- **Technology**: Software, programming, computing
- **Science**: Research, experiments, theories
- **Business**: Companies, markets, economics
- **History**: Historical periods, events, civilizations
- **Culture**: Arts, traditions, customs

### 4. **Knowledge Graph Construction**
Builds a queryable graph of all parsed information with:
- Node indexing by entity and category
- Relationship mapping
- Similarity scoring
- Graph statistics and analysis

## API Endpoints

### Parse Individual Node
**POST** `/api/brain/parse`

Parse a single brain node and extract structured information.

```json
{
  "node_id": "node_123",
  "key": "Rust Language",
  "value": "A systems programming language created by Mozilla focused on safety and performance"
}
```

Response:
```json
{
  "success": true,
  "parsed_content": {
    "node_id": "node_123",
    "original_key": "Rust Language",
    "extracted_entities": [
      {
        "entity_type": "Technology",
        "value": "Rust",
        "confidence": 0.95,
        "context": "A systems programming language..."
      }
    ],
    "relationships": [...],
    "topics": ["Programming", "Systems"],
    "categories": [
      {
        "name": "Technology",
        "confidence": 0.85,
        "subcategories": ["Programming", "Languages"]
      }
    ],
    "metadata": {
      "source": "brain_parser",
      "confidence_overall": 0.80,
      "processing_timestamp": "2026-02-21T...",
      "word_count": 15,
      "sentence_count": 1,
      "language": "English"
    }
  }
}
```

### Build Complete Knowledge Graph
**POST** `/api/brain/graph/build`

Analyze all brain nodes and build a complete knowledge graph.

Response:
```json
{
  "success": true,
  "graph_stats": {
    "node_count": 1245,
    "edge_count": 3421,
    "categories": ["Technology", "Science", "Business"],
    "entity_count": 8932
  }
}
```

### Query by Entity
**POST** `/api/brain/graph/query/entity`

Find all brain nodes containing a specific entity.

```json
{
  "entity": "Rust"
}
```

Response:
```json
{
  "success": true,
  "query_type": "entity",
  "results": ["node_123", "node_456", "node_789"],
  "result_count": 3
}
```

### Query by Category
**POST** `/api/brain/graph/query/category`

Find all brain nodes in a specific category.

```json
{
  "category": "Technology"
}
```

Response:
```json
{
  "success": true,
  "query_type": "category",
  "results": ["node_1", "node_2", "node_3", ...],
  "result_count": 245
}
```

### Get Graph Statistics
**GET** `/api/brain/graph/statistics`

Get detailed statistics about the knowledge graph.

Response:
```json
{
  "total_nodes": 1245,
  "total_edges": 3421,
  "total_categories": 12,
  "total_entities": 8932,
  "nodes": [
    {
      "id": "node_123",
      "key": "Rust Language",
      "entities_count": 5,
      "relationships_count": 3,
      "categories": ["Technology"]
    },
    ...
  ]
}
```

### Analyze Relationships
**GET** `/api/brain/graph/relationships`

Get all relationships detected across the knowledge graph.

Response:
```json
{
  "success": true,
  "total_relationships": 3421,
  "relationships": [
    {
      "subject": "Rust",
      "predicate": "is a",
      "object": "Programming Language",
      "confidence": 0.92,
      "type": "IsA"
    },
    ...
  ]
}
```

### Get Entities Report
**GET** `/api/brain/graph/entities`

Get a report of all extracted entities grouped by type.

Response:
```json
{
  "success": true,
  "total_entities": 8932,
  "entities_by_type": {
    "Technology": [
      {
        "value": "Rust",
        "confidence": 0.95
      },
      ...
    ],
    "Person": [
      {
        "value": "Graydon Hoare",
        "confidence": 0.88
      },
      ...
    ]
  }
}
```

## Usage Examples

### Example 1: Parse a Brain Node
```rust
use jeebs::brain_parser::BrainParser;

let parser = BrainParser::new();
let parsed = parser.parse(
    "node_1".to_string(),
    "Rust".to_string(),
    "A systems programming language focused on safety and performance".to_string(),
);

println!("Entities: {:?}", parsed.extracted_entities);
println!("Categories: {:?}", parsed.categories);
```

### Example 2: Build Knowledge Graph
```rust
use jeebs::brain_parser::{BrainParser, build_knowledge_graph};

let parser = BrainParser::new();
let graph = build_knowledge_graph(&db, &parser).await?;

// Query the graph
let rust_nodes = graph.query_by_entity("Rust");
let tech_nodes = graph.query_by_category("Technology");
```

### Example 3: API Call with curl
```bash
# Parse a single node
curl -X POST http://localhost:8080/api/brain/parse \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "node_123",
    "key": "Python",
    "value": "A high-level, interpreted programming language"
  }'

# Get graph statistics
curl http://localhost:8080/api/brain/graph/statistics

# Query by entity
curl -X POST http://localhost:8080/api/brain/graph/query/entity \
  -H "Content-Type: application/json" \
  -d '{"entity": "Python"}'
```

## Architecture

### Parser Components

1. **Entity Patterns**: Regex-based patterns for detecting different entity types
2. **Relationship Patterns**: Pattern matching for relationship predicates
3. **Category Keywords**: Semantic keyword mappings for category inference
4. **Confidence Scoring**: Weighted scoring based on match quality and source

### Knowledge Graph Structure

```
KnowledgeGraph
├── nodes: HashMap<NodeId, GraphNode>
├── edges: Vec<GraphEdge>
├── categories: HashMap<Category, Vec<NodeIds>>
└── entity_index: HashMap<Entity, Vec<NodeIds>>

GraphNode
├── id: String
├── content: ParsedBrainContent
├── related_nodes: Vec<NodeIds>
└── similarity_score: f64

ParsedBrainContent
├── extracted_entities: Vec<Entity>
├── relationships: Vec<Relationship>
├── topics: Vec<String>
├── categories: Vec<Category>
└── metadata: BrainMetadata
```

## Extending the Framework

### Add Custom Entity Types
```rust
pub enum EntityType {
    // ... existing types ...
    Custom(String),
}
```

### Add Custom Relationship Types
```rust
pub enum RelationType {
    // ... existing types ...
    Custom(String),
}
```

### Add Custom Category Keywords
Modify the `build_category_keywords()` method in `BrainParser`:

```rust
keywords.insert(
    "Custom Category".to_string(),
    vec!["keyword1", "keyword2", "keyword3"]
        .iter()
        .map(|s| s.to_string())
        .collect(),
);
```

## Performance Considerations

- **Caching**: Graph building can be cached and incrementally updated
- **Lazy Evaluation**: Query results are computed on-demand
- **Batch Processing**: Process multiple nodes concurrently for better performance
- **Database Indexes**: Ensure proper indexing on brain table for fast queries

## Testing

The framework includes unit tests:

```bash
cargo test brain_parser
```

Test examples:
- Entity extraction accuracy
- Relationship detection
- Category inference
- Graph query functionality

## Future Enhancements

1. **ML-Based Classification**: Use machine learning for improved entity and relationship detection
2. **Semantic Similarity**: Compute semantic similarity between nodes using embeddings
3. **Graph Visualization**: Generate interactive visualizations of the knowledge graph
4. **Query Optimization**: Advanced graph query DSL for complex searches
5. **Temporal Analysis**: Track how knowledge evolves over time
6. **Multi-Language Support**: Language detection and processing for non-English content
7. **Custom Extractors**: Pluggable entity and relationship extractors

## Contributing

When extending the framework:
1. Add tests for new patterns and classifiers
2. Update documentation with new entity/relationship types
3. Ensure backward compatibility with existing parsed content
4. Profile performance on large graphs

## License

This framework is part of JeebsAI and follows the same MIT license.
