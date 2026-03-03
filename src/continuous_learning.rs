/// Continuous Learning Engine
///
/// Processes interactions and automatically learns new facts, relationships,
/// and concepts from successful conversations.
///
/// Components:
/// - Interaction processing: Extract learnings from chat exchanges
/// - Knowledge synthesis: Create new facts from inferred knowledge
/// - Relationship building: Create connections between concepts
/// - Confidence updating: Adjust confidence scores based on feedback
/// - Memory consolidation: Integrate new learning into brain

use serde_json::{json, Value};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct InteractionLearning {
    pub interaction_id: String,
    pub user_query: String,
    pub jeebs_response: String,
    pub concepts_mentioned: Vec<String>,
    pub inferred_relationships: Vec<(String, String, String)>,
    pub confidence: f32,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct LearnedFact {
    pub fact: String,
    pub source: String,
    pub confidence: f32,
    pub related_concepts: Vec<String>,
}

/// Process an interaction to extract new learning
pub async fn process_interaction_for_learning(
    pool: &SqlitePool,
    data: &InteractionLearning,
) -> Result<Vec<LearnedFact>, String> {
    println!(
        "[Learning] Processing interaction {} with {} concepts",
        data.interaction_id,
        data.concepts_mentioned.len()
    );

    let mut learned_facts = Vec::new();

    // Extract facts from response
    let facts = extract_facts_from_response(&data.jeebs_response, &data.concepts_mentioned);

    for fact in facts {
        // Store in brain_nodes_v2
        let fact_id = format!("interaction:{}", uuid::Uuid::new_v4());

        sqlx::query(
            "INSERT INTO brain_nodes_v2 (id, fact, category)
             VALUES (?, ?, 'interaction_derived')",
        )
        .bind(&fact_id)
        .bind(&fact)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to store fact: {}", e))?;

        learned_facts.push(LearnedFact {
            fact: fact.clone(),
            source: "interaction_derived".to_string(),
            confidence: data.confidence,
            related_concepts: data.concepts_mentioned.clone(),
        });

        println!("[Learning] Stored fact: {} (confidence: {:.0}%)", fact, data.confidence * 100.0);
    }

    // Create relationships from inferred connections
    for (subject, predicate, object) in &data.inferred_relationships {
        // Store as knowledge triple
        sqlx::query(
            "INSERT INTO knowledge_triples (subject, predicate, object, confidence)
             VALUES (?, ?, ?, ?)
             ON CONFLICT DO NOTHING",
        )
        .bind(subject)
        .bind(predicate)
        .bind(object)
        .bind(data.confidence as f64)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to store relationship: {}", e))?;

        // Create or update connection graph
        let _ = sqlx::query(
            "INSERT INTO connections (id, from_node, to_node, strength)
             VALUES (?, ?, ?, ?)
             ON CONFLICT DO NOTHING",
        )
        .bind(format!("conn:{}_{}", subject, object))
        .bind(subject)
        .bind(object)
        .bind(data.confidence as f64)
        .execute(pool)
        .await;

        println!(
            "[Learning] Created relationship: {} {} {}",
            subject, predicate, object
        );
    }

    Ok(learned_facts)
}

/// Extract facts from a response using pattern matching and NLP
fn extract_facts_from_response(response: &str, concepts: &[String]) -> Vec<String> {
    let mut facts = Vec::new();

    // Split response into sentences
    let sentences: Vec<&str> = response
        .split(&['.', '!', '?'][..])
        .filter(|s| !s.trim().is_empty())
        .collect();

    for sentence in sentences {
        let trimmed = sentence.trim();

        // Check if sentence mentions any key concepts
        for concept in concepts {
            if trimmed.to_lowercase().contains(&concept.to_lowercase()) {
                // This is a potential fact
                let clean_fact = trimmed
                    .replace("Based on my knowledge regarding", "")
                    .replace("•", "")
                    .trim()
                    .to_string();

                if clean_fact.len() > 10 && clean_fact.len() < 500 {
                    facts.push(clean_fact);
                    break;
                }
            }
        }
    }

    facts.dedup();
    println!("[Learning] Extracted {} facts from response", facts.len());
    facts
}

/// Update confidence scores based on user feedback
pub async fn update_confidence_from_feedback(
    pool: &SqlitePool,
    interaction_id: &str,
    feedback_score: f32,
) -> Result<(), String> {
    // Retrieve the interaction learning record
    let row = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT value FROM jeebs_store WHERE key = ?",
    )
    .bind(format!("interaction_learning:{}", interaction_id))
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    if let Some((value,)) = row {
        if let Ok(mut record) = serde_json::from_slice::<Value>(&value) {
            // Update confidence with feedback
            if let Some(current_conf) = record.get("confidence").and_then(|v| v.as_f64()) {
                let updated_conf = (current_conf * 0.7 + feedback_score as f64 * 0.3)
                    .min(0.99)
                    .max(0.1);

                record["confidence"] = json!(updated_conf);
                record["feedback_score"] = json!(feedback_score);

                sqlx::query(
                    "UPDATE jeebs_store SET value = ? WHERE key = ?",
                )
                .bind(
                    serde_json::to_vec(&record)
                        .map_err(|e| format!("Serialization error: {}", e))?,
                )
                .bind(format!("interaction_learning:{}", interaction_id))
                .execute(pool)
                .await
                .map_err(|e| format!("Update failed: {}", e))?;

                println!(
                    "[Learning] Updated confidence for interaction {}: {:.2} → {:.2}",
                    interaction_id, current_conf, updated_conf
                );
            }
        }
    }

    Ok(())
}

/// Consolidate learnings periodically (batch process interactions)
pub async fn consolidate_learnings(
    pool: &SqlitePool,
) -> Result<ConsolidationReport, String> {
    println!("[Learning] Starting learning consolidation");

    let mut report = ConsolidationReport {
        interactions_processed: 0,
        facts_learned: 0,
        relationships_created: 0,
        total_confidence: 0.0,
    };

    // Get unprocessed interaction learnings
    let rows = sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT key, value FROM jeebs_store WHERE key LIKE 'interaction_learning:%'
         LIMIT 100",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    for (_key, value) in rows {
        if let Ok(record) = serde_json::from_slice::<Value>(&value) {
            if let (Some(topic), Some(concepts), Some(confidence)) = (
                record.get("topic").and_then(|v| v.as_str()),
                record.get("concepts_learned").and_then(|v| v.as_array()),
                record.get("confidence").and_then(|v| v.as_f64()),
            ) {
                let concept_strings: Vec<String> = concepts
                    .iter()
                    .filter_map(|c| c.as_str().map(|s| s.to_string()))
                    .collect();

                report.interactions_processed += 1;
                report.total_confidence += confidence;

                // Find or create learning session for topic
                let session_key = format!("learnsession:{}", topic.replace(" ", "_"));

                // Store consolidated learning
                let consolidation = json!({
                    "timestamp": chrono::Local::now().to_rfc3339(),
                    "topic": topic,
                    "concepts_learned": concept_strings,
                    "confidence": confidence,
                    "consolidated": true
                });

                sqlx::query(
                    "INSERT INTO jeebs_store (key, value) VALUES (?, ?)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                )
                .bind(&session_key)
                .bind(
                    serde_json::to_vec(&consolidation)
                        .map_err(|e| format!("Serialization error: {}", e))?,
                )
                .execute(pool)
                .await
                .map_err(|e| format!("Storage failed: {}", e))?;

                report.facts_learned += 1;
            }
        }
    }

    // Log consolidation
    sqlx::query(
        "INSERT INTO action_logs (timestamp, action, details, severity)
         VALUES (?, 'consolidate_learnings', ?, 'info')",
    )
    .bind(chrono::Local::now().to_rfc3339())
    .bind(format!(
        "Consolidated {} interactions, {} facts learned, avg confidence {:.2}",
        report.interactions_processed,
        report.facts_learned,
        if report.interactions_processed > 0 {
            report.total_confidence / report.interactions_processed as f64
        } else {
            0.0
        }
    ))
    .execute(pool)
    .await
    .ok();

    println!("[Learning] Consolidation complete: {} interactions, {} facts learned",
        report.interactions_processed,
        report.facts_learned
    );

    Ok(report)
}

#[derive(Debug)]
pub struct ConsolidationReport {
    pub interactions_processed: i32,
    pub facts_learned: i32,
    pub relationships_created: i32,
    pub total_confidence: f64,
}

/// Get learning statistics
pub async fn get_learning_statistics(pool: &SqlitePool) -> Result<Value, String> {
    // Count facts by source
    let sources: Vec<(String, i64)> = sqlx::query_as(
        "SELECT category, COUNT(*) as count FROM brain_nodes_v2
         GROUP BY category ORDER BY count DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    // Get average confidence
    let avg_conf: Option<(f64,)> = sqlx::query_as(
        "SELECT AVG(confidence) FROM knowledge_triples",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    // Get total interactions processed
    let interaction_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM jeebs_store WHERE key LIKE 'interaction_learning:%'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(json!({
        "total_interactions_learned": interaction_count.0,
        "facts_by_source": sources.into_iter()
            .map(|(source, count)| json!({ "source": source, "count": count }))
            .collect::<Vec<_>>(),
        "average_confidence": avg_conf.map(|(c,)| c).unwrap_or(0.5),
        "learning_active": true,
        "timestamp": chrono::Local::now().to_rfc3339()
    }))
}
