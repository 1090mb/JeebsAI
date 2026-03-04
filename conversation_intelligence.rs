/// Conversation Intelligence: Semantic Intent Detection
///
/// Provides context-aware intent detection that goes beyond simple keyword matching.
/// Understands sentence structure, conversation history, and semantic markers to
/// accurately identify what the user is trying to accomplish.

use crate::conversation_context::ConversationMessage;

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticIntent {
    pub primary_intent: String,
    pub confidence: f32,
    pub reasoning: String,
    pub context_used: bool,
}

/// Detect intent using sentence structure analysis
pub fn sentence_structure_intent(message: &str) -> Option<(String, f32)> {
    let lower = message.to_lowercase();
    let lower = lower.trim();

    // Imperative sentences (commands, requests for action)
    if lower.starts_with("help ")
        || lower.starts_with("show ")
        || lower.starts_with("explain ")
        || lower.starts_with("tell ")
        || lower.starts_with("describe ")
        || lower.starts_with("imagine ")
    {
        return Some(("instruct".to_string(), 0.85));
    }

    // Conditional/hypothetical sentences
    if lower.starts_with("if ") || lower.starts_with("what if ") || lower.starts_with("assume ") {
        return Some(("explore".to_string(), 0.80));
    }

    // Story/narrative structure (information sharing, not questions)
    if lower.starts_with("so ") || lower.starts_with("basically ") || lower.starts_with("essentially ") {
        return Some(("explain".to_string(), 0.70));
    }

    None
}

/// Detect intent from semantic markers and phrases
pub fn semantic_markers_intent(message: &str) -> Option<(String, f32)> {
    let lower = message.to_lowercase();

    // "Tell me more" / "Elaborate" patterns - these are elaboration requests
    if lower.contains("tell me more")
        || lower.contains("elaborate")
        || lower.contains("go deeper")
        || lower.contains("dive deeper")
        || lower.contains("more details")
    {
        return Some(("explain".to_string(), 0.88));
    }

    // Comparison patterns
    if lower.contains("how does") && lower.contains("compare")
        || lower.contains("compare")
        || lower.contains("difference between")
        || lower.contains("similar")
    {
        return Some(("compare".to_string(), 0.85));
    }

    // Example patterns
    if lower.contains("example of") || lower.contains("for instance") || lower.contains("like what") {
        return Some(("example".to_string(), 0.85));
    }

    // Reasoning patterns
    if lower.contains("why") && !lower.starts_with("why ") {
        // "Why is..." at the start is handled elsewhere, this is embedded why
        return Some(("reasoning".to_string(), 0.82));
    }

    // Practical/how-to patterns
    if lower.contains("how to") || lower.contains("how do i") || lower.contains("how can i") {
        return Some(("instruct".to_string(), 0.85));
    }

    None
}

/// Detect intent from conversation history (3-turn lookback)
pub fn conversation_history_intent(
    current_message: &str,
    history: &[ConversationMessage],
) -> Option<(String, f32)> {
    if history.is_empty() {
        return None;
    }

    let last_messages: Vec<_> = history.iter().rev().take(3).collect();

    // If previous response was an explanation, "more?" or "another?" is asking for elaboration
    if current_message.len() < 30 {
        let lower = current_message.to_lowercase();
        if (lower.contains("more") || lower.contains("another") || lower.contains("continue"))
            && last_messages.len() > 1
            && last_messages[0].role == "jeebs"
        {
            return Some(("explore".to_string(), 0.80));
        }

        // Short follow-ups to explanations
        if (lower.contains("why") || lower.contains("how"))
            && last_messages.len() > 1
            && last_messages[0].role == "jeebs"
        {
            return Some(("reasoning".to_string(), 0.78));
        }
    }

    // If user asked a question, a statement might be providing context for a follow-up question
    if history.len() >= 2 {
        if let Some(prev_user_msg) = history.iter().rev().find(|m| m.role == "user") {
            let prev_lower = prev_user_msg.content.to_lowercase();
            if prev_lower.contains("?") && !current_message.contains("?") {
                // User previously asked a question, now making a statement - likely providing context
                return Some(("clarify".to_string(), 0.70));
            }
        }
    }

    None
}

/// Detect if message contains embedded questions (e.g., "Hi, how do I fix X?")
pub fn extract_embedded_questions(message: &str) -> Vec<String> {
    let sentences: Vec<&str> = message.split(|c| c == '.' || c == '!' || c == '?').collect();

    sentences
        .into_iter()
        .filter(|s| s.contains('?'))
        .map(|s| s.trim().to_string())
        .collect()
}

/// Comprehensive semantic intent detection
pub fn detect_semantic_intent(
    current_message: &str,
    history: Option<&[ConversationMessage]>,
) -> SemanticIntent {
    // Step 1: Try sentence structure analysis first (most reliable when it matches)
    if let Some((intent, conf)) = sentence_structure_intent(current_message) {
        return SemanticIntent {
            primary_intent: intent,
            confidence: conf,
            reasoning: "Detected via sentence structure".to_string(),
            context_used: false,
        };
    }

    // Step 2: Try semantic markers (very specific patterns)
    if let Some((intent, conf)) = semantic_markers_intent(current_message) {
        return SemanticIntent {
            primary_intent: intent,
            confidence: conf,
            reasoning: "Detected via semantic markers".to_string(),
            context_used: false,
        };
    }

    // Step 3: Use conversation history if available
    if let Some(hist) = history {
        if let Some((intent, conf)) = conversation_history_intent(current_message, hist) {
            return SemanticIntent {
                primary_intent: intent,
                confidence: conf,
                reasoning: "Detected via conversation context".to_string(),
                context_used: true,
            };
        }
    }

    // Fallback: Return generic intent (this goes to caller for basic keyword matching as last resort)
    SemanticIntent {
        primary_intent: "ask".to_string(),
        confidence: 0.0, // Signal to use basic keyword matching
        reasoning: "No semantic markers detected".to_string(),
        context_used: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_structure_intent() {
        assert_eq!(
            sentence_structure_intent("Help me understand recursion"),
            Some(("instruct".to_string(), 0.85))
        );
        assert_eq!(
            sentence_structure_intent("Imagine a world where..."),
            Some(("explore".to_string(), 0.80))
        );
    }

    #[test]
    fn test_semantic_markers() {
        let (intent, conf) = semantic_markers_intent("Tell me more about that").unwrap();
        assert_eq!(intent, "explain");
        assert!(conf > 0.8);

        let (intent, _) = semantic_markers_intent("How does this compare to X?").unwrap();
        assert_eq!(intent, "compare");
    }

    #[test]
    fn test_embedded_questions() {
        let questions = extract_embedded_questions("Hi there! How do I fix this issue?");
        assert!(questions.iter().any(|q| q.contains("How do I")));
    }
}
