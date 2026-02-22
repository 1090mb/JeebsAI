// Example: Brain Parser Usage Demonstrations
// This file shows various ways to use the brain parsing framework

use jeebs::brain_parser::{
    BrainParser, build_knowledge_graph, ParsedBrainContent,
    Entity, EntityType, Relationship, RelationType
};
use sqlx::SqlitePool;

/// Example 1: Basic Entity Extraction
pub fn example_basic_entity_extraction() {
    let parser = BrainParser::new();

    let parsed = parser.parse(
        "node_001".to_string(),
        "Rust Language".to_string(),
        "Rust is a systems programming language created by Mozilla in 2010. \
         It focuses on safety, speed, and concurrency. Used for building \
         reliable and efficient software.".to_string(),
    );

    println!("=== Example 1: Basic Entity Extraction ===");
    println!("Node ID: {}", parsed.node_id);
    println!("Original Key: {}", parsed.original_key);
    println!("\nExtracted Entities:");
    for entity in &parsed.extracted_entities {
        println!("  - Value: {}", entity.value);
        println!("    Type: {:?}", entity.entity_type);
        println!("    Confidence: {}", entity.confidence);
        println!("    Context: {}", entity.context.as_ref().unwrap_or(&"None".to_string()));
    }

    println!("\nIdentified Topics: {:?}", parsed.topics);
    println!("\nInferred Categories:");
    for category in &parsed.categories {
        println!("  - {}: {}", category.name, category.confidence);
        if !category.subcategories.is_empty() {
            println!("    Subcategories: {:?}", category.subcategories);
        }
    }

    println!("\nMetadata:");
    println!("  Word Count: {}", parsed.metadata.word_count);
    println!("  Sentence Count: {}", parsed.metadata.sentence_count);
    println!("  Overall Confidence: {}", parsed.metadata.confidence_overall);
    println!("  Language: {}", parsed.metadata.language);
}

/// Example 2: Relationship Detection
pub fn example_relationship_detection() {
    let parser = BrainParser::new();

    let parsed = parser.parse(
        "node_002".to_string(),
        "Python Programming".to_string(),
        "Python is a high-level programming language. It is used in data science \
         and machine learning. Django is a web framework built with Python.".to_string(),
    );

    println!("\n=== Example 2: Relationship Detection ===");
    println!("Detected Relationships:");
    for (i, rel) in parsed.relationships.iter().enumerate() {
        println!("\nRelationship {}:", i + 1);
        println!("  Subject: {}", rel.subject);
        println!("  Predicate: {}", rel.predicate);
        println!("  Object: {}", rel.object);
        println!("  Type: {:?}", rel.relationship_type);
        println!("  Confidence: {}", rel.confidence);
    }
}

/// Example 3: Category-Based Organization
pub fn example_category_organization() {
    let parser = BrainParser::new();

    // Technology content
    let tech_parsed = parser.parse(
        "node_003".to_string(),
        "AI and Neural Networks".to_string(),
        "Neural networks are computational models inspired by biological neural networks. \
         They are fundamental to deep learning and artificial intelligence research.".to_string(),
    );

    // Business content
    let biz_parsed = parser.parse(
        "node_004".to_string(),
        "Market Analysis".to_string(),
        "The tech market is growing rapidly. Companies are investing heavily in AI startups. \
         Revenue from enterprise software continues to increase.".to_string(),
    );

    println!("\n=== Example 3: Category-Based Organization ===");

    println!("\nTechnology Content:");
    println!("Key: {}", tech_parsed.original_key);
    println!("Categories: {}",
        tech_parsed.categories
            .iter()
            .map(|c| format!("{} ({})", c.name, c.confidence))
            .collect::<Vec<_>>()
            .join(", ")
    );

    println!("\nBusiness Content:");
    println!("Key: {}", biz_parsed.original_key);
    println!("Categories: {}",
        biz_parsed.categories
            .iter()
            .map(|c| format!("{} ({})", c.name, c.confidence))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

/// Example 4: Building and Querying Knowledge Graph
pub async fn example_knowledge_graph(db: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let parser = BrainParser::new();

    // Build complete graph from database
    let graph = build_knowledge_graph(db, &parser).await?;

    println!("\n=== Example 4: Knowledge Graph ===");
    println!("Graph Statistics:");
    println!("  Total Nodes: {}", graph.nodes.len());
    println!("  Total Edges: {}", graph.edges.len());
    println!("  Total Categories: {}", graph.categories.len());
    println!("  Total Entities: {}", graph.entity_index.len());

    // Query by entity
    let rust_nodes = graph.query_by_entity("Rust");
    println!("\nNodes containing 'Rust': {}", rust_nodes.len());

    // Query by category
    let tech_nodes = graph.query_by_category("Technology");
    println!("Nodes in Technology category: {}", tech_nodes.len());

    // Get categories
    println!("\nAll Categories:");
    for category in graph.categories.keys() {
        let count = graph.categories[category].len();
        println!("  - {}: {} nodes", category, count);
    }

    // Print some top entities
    println!("\nTop Entities (first 10):");
    let mut entity_list: Vec<_> = graph.entity_index.iter().collect();
    entity_list.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    for (entity, nodes) in entity_list.iter().take(10) {
        println!("  - {}: {} occurrences", entity, nodes.len());
    }

    Ok(())
}

/// Example 5: Semantic Similarity Analysis
pub fn example_semantic_analysis() {
    let parser = BrainParser::new();

    // Parse related content
    let node1 = parser.parse(
        "node_005".to_string(),
        "Data Science".to_string(),
        "Data science combines statistics, programming, and domain knowledge to \
         extract insights from data.".to_string(),
    );

    let node2 = parser.parse(
        "node_006".to_string(),
        "Machine Learning".to_string(),
        "Machine learning is a subset of artificial intelligence focused on algorithms \
         that learn from data.".to_string(),
    );

    println!("\n=== Example 5: Semantic Similarity ===");

    // Compare entities
    let entities1: Vec<_> = node1.extracted_entities.iter()
        .map(|e| &e.value).collect();
    let entities2: Vec<_> = node2.extracted_entities.iter()
        .map(|e| &e.value).collect();

    println!("Node 1 entities: {:?}", entities1);
    println!("Node 2 entities: {:?}", entities2);

    // Find common entities
    let common: Vec<_> = entities1.iter()
        .filter(|e| entities2.contains(e))
        .collect();
    println!("Common entities: {:?}", common);

    // Compare categories
    let cats1: Vec<_> = node1.categories.iter().map(|c| &c.name).collect();
    let cats2: Vec<_> = node2.categories.iter().map(|c| &c.name).collect();
    println!("\nNode 1 categories: {:?}", cats1);
    println!("Node 2 categories: {:?}", cats2);
}

/// Example 6: Confidence-Based Filtering
pub fn example_confidence_filtering() {
    let parser = BrainParser::new();

    let parsed = parser.parse(
        "node_007".to_string(),
        "Complex Topic".to_string(),
        "Natural language processing combines linguistics and computer science. \
         Named entity recognition identifies entities in text. \
         Sentiment analysis determines emotional tone.".to_string(),
    );

    println!("\n=== Example 6: Confidence-Based Filtering ===");

    // High confidence entities
    let high_confidence: Vec<_> = parsed.extracted_entities
        .iter()
        .filter(|e| e.confidence > 0.8)
        .collect();

    println!("High Confidence Entities (>0.8):");
    for entity in high_confidence {
        println!("  - {} ({})", entity.value, entity.confidence);
    }

    // High confidence categories
    let high_conf_cats: Vec<_> = parsed.categories
        .iter()
        .filter(|c| c.confidence > 0.7)
        .collect();

    println!("\nHigh Confidence Categories (>0.7):");
    for cat in high_conf_cats {
        println!("  - {} ({})", cat.name, cat.confidence);
    }
}

/// Example 7: Batch Processing Multiple Nodes
pub fn example_batch_processing() {
    let parser = BrainParser::new();

    let nodes = vec![
        ("node_008", "Python", "High-level programming language"),
        ("node_009", "Java", "Object-oriented programming language"),
        ("node_010", "Go", "Compiled language for systems programming"),
    ];

    println!("\n=== Example 7: Batch Processing ===");

    let mut all_entities = std::collections::HashMap::new();

    for (id, key, value) in nodes {
        let parsed = parser.parse(
            id.to_string(),
            key.to_string(),
            value.to_string(),
        );

        for entity in parsed.extracted_entities {
            *all_entities.entry(entity.value).or_insert(0) += 1;
        }
    }

    println!("Entity Frequency Across Batch:");
    for (entity, count) in all_entities {
        println!("  - {}: {}", entity, count);
    }
}

/// Example 8: Custom Entity Type Usage
pub fn example_custom_entities() {
    let parser = BrainParser::new();

    let parsed = parser.parse(
        "node_011".to_string(),
        "Technology Stack".to_string(),
        "Using Rust and Python with React frontend. The system is built on Kubernetes \
         and deployed with Docker.".to_string(),
    );

    println!("\n=== Example 8: Technology Entity Detection ===");

    let tech_entities: Vec<_> = parsed.extracted_entities
        .iter()
        .filter(|e| e.entity_type == EntityType::Technology)
        .collect();

    if !tech_entities.is_empty() {
        println!("Detected Technologies:");
        for tech in tech_entities {
            println!("  - {}", tech.value);
        }
    } else {
        println!("No technology entities detected");
    }
}

// Main function to run examples
pub fn run_all_examples() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Brain Parser Framework - Usage Examples                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    example_basic_entity_extraction();
    example_relationship_detection();
    example_category_organization();
    example_semantic_analysis();
    example_confidence_filtering();
    example_batch_processing();
    example_custom_entities();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              Examples Complete!                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_examples_run() {
        // This test ensures all example functions compile and run
        example_basic_entity_extraction();
        example_relationship_detection();
        example_category_organization();
        example_semantic_analysis();
        example_confidence_filtering();
        example_batch_processing();
        example_custom_entities();
    }
}
