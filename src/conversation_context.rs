/// Conversation Context Manager
///
/// Understands multi-turn conversations by:
/// - Tracking conversation history and context
/// - Identifying topic continuity and shifts
/// - Extracting user intent from context
/// - Maintaining focus on current topic
/// - Preventing context loss across turns
/// - Tracking question-answer pairs for better dialogue

use serde_json::json;
use sqlx::SqlitePool;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConversationPair {
    pub question: String,
    pub answer: String,
    pub question_intent: String,
    pub related_topics: Vec<String>,
}

/// Represents different ways a topic entered the conversation
#[derive(Debug, Clone, PartialEq)]
pub enum TopicEntryType {
    UserExplicit,      // User said "Let's talk about X"
    UserImplicit,      // Inferred from question about X
    SubtopicExploration,  // Related to current topic
    Tangent,            // Off-topic but related
    Correction,         // User corrected/clarified
}

/// A topic node in the conversation hierarchy
#[derive(Debug, Clone)]
pub struct TopicNode {
    pub name: String,
    pub depth: u32,  // How specific (0=broad, 3=very specific)
    pub relevance: f32,  // 0.0-1.0, decays over time
    pub parent_topic: Option<String>,  // Hierarchy
    pub entry_type: TopicEntryType,  // How this topic was introduced
}

#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub session_id: String,
    pub user_id: Option<String>,
    pub messages: Vec<ConversationMessage>,
    // NEW: Topic hierarchy instead of flat topics
    pub topic_stack: Vec<TopicNode>,  // Stack with depth/relevance
    pub topic_relationships: HashMap<String, f32>,  // topic → parent weight
    // KEPT FOR BACKWARD COMPATIBILITY:
    pub current_topic: String,
    pub previous_topics: Vec<String>,
    pub user_intent: String,
    pub focus_level: f32,
    pub conversation_pairs: Vec<ConversationPair>, // Track Q&A pairs for context
    pub last_user_question: Option<String>,         // Remember the last question
}

#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub role: String, // "user" or "jeebs"
    pub content: String,
    pub timestamp: String,
    pub topics_mentioned: Vec<String>,
    pub intent: String,
}

#[derive(Debug, Clone)]
pub struct UserIntent {
    pub primary: String, // "ask", "clarify", "explore", "learn"
    pub sentiment: f32,  // -1.0 to 1.0
    pub confidence: f32, // 0.0 to 1.0
    pub requires_explanation: bool,
}

/// Load or create conversation context
pub async fn load_conversation_context(
    pool: &SqlitePool,
    session_id: &str,
    user_id: Option<&str>,
) -> Result<ConversationContext, String> {
    // Try to fetch existing conversation from chat_history
    // Increased from 20 to 100 messages for better context retention
    let history = sqlx::query_as::<_, (String, String)>(
        "SELECT role, message FROM chat_history
         WHERE session_id = ?
         ORDER BY created_at DESC
         LIMIT 100",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut messages = Vec::new();
    let mut topics = Vec::new();

    for (role, content) in history.iter().rev() {
        let msg_topics = extract_topics(&content);
        topics.extend(msg_topics.clone());

        messages.push(ConversationMessage {
            role: role.clone(),
            content: content.clone(),
            timestamp: chrono::Local::now().to_rfc3339(),
            topics_mentioned: msg_topics,
            intent: String::new(),
        });
    }

    topics.sort();
    topics.dedup_by(|a, b| a.to_lowercase().eq(&b.to_lowercase()));

    let current_topic = messages
        .first()
        .and_then(|m| m.topics_mentioned.first())
        .cloned()
        .unwrap_or_else(|| "general".to_string());

    println!(
        "[Context] Loaded {} messages, {} topics, focus: {}",
        messages.len(),
        topics.len(),
        current_topic
    );

    // Extract conversation pairs (Q&A pairs) from recent messages for better dialogue understanding
    let conversation_pairs = extract_conversation_pairs(&messages);

    // Remember the last user question if one exists
    let last_user_question = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone());

    // Build topic hierarchy: current topic with depth 1, previous topics with decay
    let mut topic_stack = Vec::new();
    if !current_topic.is_empty() && current_topic != "general" {
        topic_stack.push(TopicNode {
            name: current_topic.clone(),
            depth: 1,
            relevance: 1.0,
            parent_topic: None,
            entry_type: TopicEntryType::UserImplicit,
        });
    }

    // Add previous topics as related topics with decaying relevance
    for (idx, topic) in topics.iter().take(3).enumerate() {
        if topic != &current_topic && !topic.is_empty() {
            let relevance = 1.0 - (idx as f32 * 0.25);  // Decay relevance
            topic_stack.push(TopicNode {
                name: topic.clone(),
                depth: 0,
                relevance: relevance.max(0.3),
                parent_topic: if !current_topic.is_empty() && current_topic != "general" {
                    Some(current_topic.clone())
                } else {
                    None
                },
                entry_type: TopicEntryType::UserImplicit,
            });
        }
    }

    let topic_relationships = HashMap::new();  // Will be populated as conversation evolves

    Ok(ConversationContext {
        session_id: session_id.to_string(),
        user_id: user_id.map(|s| s.to_string()),
        messages,
        topic_stack,
        topic_relationships,
        current_topic,
        previous_topics: topics,
        user_intent: "ask".to_string(),
        focus_level: 0.8,
        conversation_pairs,
        last_user_question,
    })
}

/// Analyze user message to extract intent and topics
pub fn analyze_user_message(message: &str) -> UserIntent {
    let lower = message.to_lowercase();

    // Detect intent with higher precision
    let primary = if lower.contains("why") {
        "reasoning"
    } else if lower.contains("explain") || lower.contains("elaborate") {
        "explain"
    } else if lower.contains("example") || lower.contains("show me") {
        "example"
    } else if lower.contains("how") || lower.contains("can you") || lower.contains("could you") {
        "instruct"
    } else if lower.contains("compare") || lower.contains("similar") || lower.contains("difference") {
        "compare"
    } else if lower.contains("what") || lower.contains("tell me") || lower.contains("define") {
        "ask"
    } else if lower.contains("more") || lower.contains("another") || lower.contains("continue") {
        "explore"
    } else if (lower.contains("?") && message.len() < 20) || lower.contains("right?") || lower.contains("correct?") {
        "clarify"
    } else {
        "ask"
    };

    // Detect sentiment with more nuance
    let sentiment = if lower.contains("awesome")
        || lower.contains("excellent")
        || lower.contains("perfect")
        || lower.contains("great")
        || lower.contains("love")
        || lower.contains("thanks") {
        0.9
    } else if lower.contains("good") || lower.contains("nice") {
        0.6
    } else if lower.contains("confusing") || lower.contains("unclear") {
        -0.5
    } else if lower.contains("wrong")
        || lower.contains("bad")
        || lower.contains("hate")
        || lower.contains("terrible")
        || lower.contains("disagree") {
        -0.8
    } else {
        0.2
    };

    // Detect if needs explanation
    let requires_explanation = message.len() > 20
        && (lower.contains("why")
            || lower.contains("explain")
            || lower.contains("understand")
            || lower.contains("how does")
            || lower.contains("what makes"));

    UserIntent {
        primary: primary.to_string(),
        sentiment,
        confidence: if lower.contains("?") { 0.85 } else { 0.70 },
        requires_explanation,
    }
}

/// Analyze user message with semantic intelligence and conversation history
pub fn analyze_user_message_semantic(message: &str, history: &[ConversationMessage]) -> UserIntent {
    use crate::conversation_intelligence;

    // Try semantic intent detection first (uses conversation context)
    let semantic_intent = conversation_intelligence::detect_semantic_intent(
        message,
        if history.is_empty() { None } else { Some(history) },
    );

    // Use semantic intent if it has good confidence, otherwise use fallback
    let primary = if semantic_intent.confidence > 0.65 {
        semantic_intent.primary_intent
    } else {
        // Fall back to basic keyword matching
        analyze_user_message(message).primary
    };

    // Still use the existing logic for sentiment detection
    let lower = message.to_lowercase();
    let sentiment = if lower.contains("awesome")
        || lower.contains("excellent")
        || lower.contains("perfect")
        || lower.contains("great")
        || lower.contains("love")
        || lower.contains("thanks") {
        0.9
    } else if lower.contains("good") || lower.contains("nice") {
        0.6
    } else if lower.contains("confusing") || lower.contains("unclear") {
        -0.5
    } else if lower.contains("wrong")
        || lower.contains("bad")
        || lower.contains("hate")
        || lower.contains("terrible")
        || lower.contains("disagree") {
        -0.8
    } else {
        0.2
    };

    let requires_explanation = message.len() > 20
        && (lower.contains("why")
            || lower.contains("explain")
            || lower.contains("understand")
            || lower.contains("how does")
            || lower.contains("what makes"));

    UserIntent {
        primary,
        sentiment,
        confidence: if semantic_intent.confidence > 0.65 {
            semantic_intent.confidence
        } else if lower.contains("?") {
            0.85
        } else {
            0.70
        },
        requires_explanation,
    }
}

/// Extract topics from message
fn extract_topics(message: &str) -> Vec<String> {
    let words: Vec<&str> = message
        .split_whitespace()
        .filter(|w| w.len() > 4 && !is_common_word(w))
        .collect();

    words
        .iter()
        .take(5)
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
}

/// Extract question-answer pairs from conversation messages
fn extract_conversation_pairs(messages: &[ConversationMessage]) -> Vec<ConversationPair> {
    let mut pairs = Vec::new();

    for i in 0..messages.len() {
        // Find user messages (questions) and their following AI responses (answers)
        if messages[i].role == "user" && i + 1 < messages.len() && messages[i + 1].role == "jeebs" {
            let question = messages[i].content.clone();
            let answer = messages[i + 1].content.clone();
            let question_intent = analyze_user_message(&question).primary;
            let related_topics = messages[i].topics_mentioned.clone();

            pairs.push(ConversationPair {
                question,
                answer,
                question_intent,
                related_topics,
            });
        }
    }

    // Keep only the most recent 5 pairs for context efficiency
    pairs.iter().rev().take(5).cloned().collect::<Vec<_>>().into_iter().rev().collect()
}

/// Check if word is common filler
fn is_common_word(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "the" | "and" | "that" | "this" | "what" | "with" | "from" | "about" | "which"
            | "their" | "would" | "there" | "these" | "could" | "should" | "think"
            | "like" | "know" | "make" | "just" | "very" | "more" | "also" | "even"
            | "only" | "some" | "such" | "when" | "where" | "come" | "over" | "have"
            | "been" | "does" | "most" | "many" | "actually" | "really" | "still"
    )
}

/// Build concise context summary for response generation
pub fn build_context_summary(context: &ConversationContext) -> String {
    let topic_str = context
        .previous_topics
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    let recent_turns = context.messages.len().min(3);

    format!(
        "Topic: {} | Context: {} recent messages | Topics: {}",
        context.current_topic, recent_turns, topic_str
    )
}

/// Get most recent user question
pub fn get_last_user_question(context: &ConversationContext) -> Option<String> {
    context
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
}

/// Detect if user is continuing existing topic or shifting
pub fn detect_topic_shift(context: &ConversationContext, new_message: &str) -> bool {
    let new_topics = extract_topics(new_message);

    // If no overlap with previous topics, it's a shift
    let overlap = new_topics
        .iter()
        .filter(|t| context.previous_topics.iter().any(|pt| pt.eq_ignore_ascii_case(t)))
        .count();

    overlap == 0 && !new_topics.is_empty()
}

/// Store conversation state for persistence
pub async fn save_conversation_state(
    pool: &SqlitePool,
    context: &ConversationContext,
) -> Result<(), String> {
    let state = json!({
        "session_id": context.session_id,
        "current_topic": context.current_topic,
        "previous_topics": context.previous_topics,
        "message_count": context.messages.len(),
        "timestamp": chrono::Local::now().to_rfc3339()
    });

    let key = format!("conversation_state:{}", context.session_id);

    sqlx::query(
        "INSERT INTO jeebs_store (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&key)
    .bind(
        serde_json::to_vec(&state)
            .map_err(|e| format!("Serialization error: {}", e))?,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(())
}

/// Summarize conversation for context efficiency
pub fn summarize_conversation(context: &ConversationContext) -> String {
    let mut summary = String::new();

    if context.messages.len() > 5 {
        let recent = &context.messages[0..3];
        summary.push_str("Recent: ");
        for msg in recent {
            let preview = if msg.content.len() > 50 {
                format!("{}...", &msg.content[..50])
            } else {
                msg.content.clone()
            };
            summary.push_str(&format!("[{}] ", preview));
        }
    }

    summary
}
