use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Local;
use rand::seq::SliceRandom;
use rand::Rng;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::time::{Duration, Instant};

use crate::state::AppState;
use crate::utils::decode_all;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct BrainNode {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub label: String,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct AdvancedSearchRequest {
    pub query: String,
    pub max_results: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CrawlRequest {
    pub url: String,
    pub depth: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct RandomCrawlQuery {
    pub depth: Option<u8>,
}

#[derive(Debug, Serialize)]
struct NodeWritePreview {
    node_id: String,
    label: String,
    summary: String,
    source_url: String,
}

#[derive(Debug, Serialize)]
struct CrawlSummary {
    start_url: String,
    max_depth: u8,
    pages_visited: usize,
    pages_stored: usize,
    links_followed: usize,
    stored_nodes: Vec<NodeWritePreview>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TrainingLearnedItem {
    node_id: String,
    title: String,
    summary: String,
    source_url: String,
    topic: String,
    source_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TrainingTrackProgress {
    name: String,
    status: String,
    progress_percent: u8,
    mastered: bool,
    nodes_written: u64,
    threshold: u64,
    current_goal: String,
    last_learned_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TrainingCycleSnapshot {
    cycle_started_at: String,
    cycle_finished_at: String,
    duration_ms: u64,
    topics: Vec<String>,
    websites_scraped: Vec<String>,
    nodes_written: u64,
    crawl_pages_visited: u64,
    crawl_pages_stored: u64,
    crawl_links_followed: u64,
    crawl_nodes_written: u64,
    wikipedia_docs_written: u64,
    text_chars_learned: u64,
    focus_topic: Option<String>,
    focus_topic_nodes_written: u64,
    learned_items_count: u64,
    errors: Vec<String>,
}

fn default_active_phase() -> String {
    "idle".to_string()
}

fn normalize_training_topic_input(input: &str) -> Option<String> {
    let compact = normalize_whitespace(input);
    if compact.is_empty() {
        return None;
    }
    Some(truncate_chars(&compact, 160))
}

fn build_track(name: &str, threshold: u64, goal: &str) -> TrainingTrackProgress {
    TrainingTrackProgress {
        name: name.to_string(),
        status: "active".to_string(),
        progress_percent: 0,
        mastered: false,
        nodes_written: 0,
        threshold,
        current_goal: goal.to_string(),
        last_learned_at: None,
    }
}

fn default_training_tracks() -> Vec<TrainingTrackProgress> {
    vec![
        build_track(
            "conversation skills",
            80,
            "learn clarifying questions, structured replies, and tone adaptation",
        ),
        build_track(
            "rust programming",
            60,
            "master ownership, concurrency, and production-grade Rust patterns",
        ),
        build_track(
            "python programming",
            60,
            "master Python tooling, data workflows, and async application patterns",
        ),
        build_track(
            "data compression and storage efficiency",
            50,
            "learn how to store more information in less space with compression and indexing",
        ),
        TrainingTrackProgress {
            status: "queued".to_string(),
            ..build_track(
                "go programming",
                40,
                "unlock after Rust and Python mastery, then learn idiomatic Go systems design",
            )
        },
        TrainingTrackProgress {
            status: "queued".to_string(),
            ..build_track(
                "javascript and typescript",
                40,
                "unlock after Rust and Python mastery, then learn full-stack JS/TS architecture",
            )
        },
        TrainingTrackProgress {
            status: "queued".to_string(),
            ..build_track(
                "c and c++",
                40,
                "unlock after Rust and Python mastery, then learn low-level optimization",
            )
        },
    ]
}

fn track_hit_score(track_name: &str, item: &TrainingLearnedItem) -> u64 {
    let corpus = format!(
        "{} {} {}",
        item.topic.to_ascii_lowercase(),
        item.title.to_ascii_lowercase(),
        item.summary.to_ascii_lowercase()
    );
    let keywords: &[&str] = match track_name {
        "conversation skills" => &[
            "conversation",
            "dialogue",
            "communication",
            "listening",
            "clarifying",
            "question",
            "language model prompting",
            "interaction design",
        ],
        "rust programming" => &[
            "rust",
            "cargo",
            "borrow checker",
            "ownership",
            "lifetime",
            "actix",
            "tokio",
            "serde",
            "sqlx",
        ],
        "python programming" => &[
            "python",
            "pandas",
            "numpy",
            "fastapi",
            "asyncio",
            "flask",
            "django",
            "pytest",
        ],
        "data compression and storage efficiency" => &[
            "compression",
            "storage",
            "dedup",
            "columnar",
            "parquet",
            "zstd",
            "lz4",
            "dictionary encoding",
            "delta encoding",
            "data structure",
        ],
        "go programming" => &["go language", "golang", "goroutine", "go programming"],
        "javascript and typescript" => &[
            "javascript",
            "typescript",
            "node.js",
            "nodejs",
            "react",
            "frontend",
        ],
        "c and c++" => &[
            "c programming",
            "c++",
            "memory management",
            "low-level",
            "pointer",
        ],
        _ => &[],
    };

    if keywords.is_empty() {
        return 0;
    }
    keywords.iter().filter(|kw| corpus.contains(**kw)).count() as u64
}

fn refresh_track_statuses(mode: &mut TrainingModeState) {
    let prerequisites_mastered = LANGUAGE_UNLOCK_PREREQUISITES.iter().all(|required| {
        mode.learning_tracks
            .iter()
            .find(|track| track.name == *required)
            .map(|track| track.mastered)
            .unwrap_or(false)
    });

    for track in &mut mode.learning_tracks {
        let threshold = track.threshold.max(1);
        let pct = ((track.nodes_written.saturating_mul(100)) / threshold).min(100);
        track.progress_percent = pct as u8;
        track.mastered = track.nodes_written >= threshold;

        if ADVANCED_LANGUAGE_TRACKS.contains(&track.name.as_str())
            && !prerequisites_mastered
            && !track.mastered
        {
            track.status = "queued".to_string();
            track.current_goal = "waiting for Rust and Python mastery unlock".to_string();
            continue;
        }

        track.status = if track.mastered {
            "mastered".to_string()
        } else {
            "active".to_string()
        };
    }
}

fn smartness_score(mode: &TrainingModeState) -> f64 {
    let mastery = mode
        .learning_tracks
        .iter()
        .filter(|track| track.mastered)
        .count() as f64;
    (mode.total_nodes_written as f64 * 0.45
        + mode.total_topics_processed as f64 * 0.25
        + mode.total_websites_scraped as f64 * 0.2
        + mastery * 25.0)
        .round()
}

fn curriculum_topics_from_state(mode: &TrainingModeState) -> Vec<String> {
    let mut topics = vec![
        "how to store more data in less space".to_string(),
        "lossless compression techniques".to_string(),
        "conversation skills for ai assistants".to_string(),
    ];

    for track in &mode.learning_tracks {
        if track.status != "active" || track.mastered {
            continue;
        }
        topics.push(track.name.clone());
        if !track.current_goal.trim().is_empty() {
            topics.push(track.current_goal.clone());
        }
    }

    let mut dedup = Vec::<String>::new();
    let mut seen = HashSet::<String>::new();
    for topic in topics {
        let normalized = topic.to_ascii_lowercase();
        if seen.insert(normalized) {
            dedup.push(topic);
        }
    }
    dedup
}

fn matches_focus_topic(focus_topic: &str, item: &TrainingLearnedItem) -> bool {
    let focus = focus_topic.to_ascii_lowercase();
    if focus.is_empty() {
        return false;
    }
    let corpus = format!(
        "{} {} {}",
        item.topic.to_ascii_lowercase(),
        item.title.to_ascii_lowercase(),
        item.summary.to_ascii_lowercase()
    );
    corpus.contains(&focus)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TrainingModeState {
    enabled: bool,
    updated_at: String,
    updated_by: String,
    last_cycle_at: Option<String>,
    total_cycles: u64,
    total_topics_processed: u64,
    total_nodes_written: u64,
    #[serde(default)]
    total_websites_scraped: u64,
    #[serde(default)]
    total_crawl_pages_visited: u64,
    #[serde(default)]
    total_crawl_pages_stored: u64,
    #[serde(default)]
    total_crawl_links_followed: u64,
    #[serde(default)]
    total_crawl_nodes_written: u64,
    #[serde(default)]
    total_wikipedia_docs_written: u64,
    last_topics: Vec<String>,
    last_error: Option<String>,
    #[serde(default)]
    last_websites: Vec<String>,
    #[serde(default)]
    last_learned_items: Vec<TrainingLearnedItem>,
    #[serde(default)]
    last_cycle_duration_ms: Option<u64>,
    #[serde(default)]
    last_cycle_nodes_written: u64,
    #[serde(default)]
    last_cycle_errors: Vec<String>,
    #[serde(default)]
    last_cycle_summary: Option<TrainingCycleSnapshot>,
    #[serde(default)]
    recent_cycles: Vec<TrainingCycleSnapshot>,
    #[serde(default)]
    is_cycle_running: bool,
    #[serde(default)]
    active_cycle_started_at: Option<String>,
    #[serde(default = "default_active_phase")]
    active_phase: String,
    #[serde(default)]
    active_target: Option<String>,
    #[serde(default)]
    active_nodes_written: u64,
    #[serde(default)]
    active_websites_completed: u64,
    #[serde(default)]
    active_topics_completed: u64,
    #[serde(default)]
    active_updated_at: Option<String>,
    #[serde(default)]
    focus_topic: Option<String>,
    #[serde(default)]
    smartness_score: f64,
    #[serde(default)]
    total_text_chars_learned: u64,
    #[serde(default)]
    learning_tracks: Vec<TrainingTrackProgress>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrainingModeToggleRequest {
    enabled: bool,
    #[serde(default)]
    focus_topic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrainingFocusTopicRequest {
    topic: String,
}

#[derive(Debug, Serialize)]
struct TrainingStatusResponse {
    training: TrainingModeState,
    internet_enabled: bool,
    interval_seconds: u64,
}

#[derive(Debug, Serialize)]
struct TrainingCycleReport {
    cycle_started_at: String,
    cycle_finished_at: String,
    duration_ms: u64,
    topics: Vec<String>,
    nodes_written: usize,
    errors: Vec<String>,
    websites_scraped: Vec<String>,
    learned_items: Vec<TrainingLearnedItem>,
    crawl_pages_visited: usize,
    crawl_pages_stored: usize,
    crawl_links_followed: usize,
    crawl_nodes_written: usize,
    wikipedia_docs_written: usize,
    text_chars_learned: usize,
    track_hits: HashMap<String, u64>,
    focus_topic: Option<String>,
    focus_topic_nodes_written: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommunicationProfile {
    style: String,
    signals: Vec<String>,
    recent_topics: Vec<String>,
    updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct BrainSearchResult {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub title: String,
    pub group: String,
}

#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConversationTurn {
    role: String,
    content: String,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LearnedFact {
    fact: String,
    canonical: String,
    created_at: String,
    updated_at: String,
}

const MAX_HISTORY_TURNS: usize = 24;
const MAX_HISTORY_CHARS_PER_TURN: usize = 600;
const TRAINING_STATE_KEY: &str = "training:mode:state";
const DEFAULT_TRAINING_INTERVAL_SECS: u64 = 5;
const JEEBS_LIKES: &[&str] = &[
    "learning new knowledge",
    "clear reasoning",
    "solving hard problems",
    "structured experiments",
    "useful conversations",
];
const JEEBS_DISLIKES: &[&str] = &[
    "stagnation",
    "guessing without evidence",
    "repeating weak answers",
    "losing useful context",
    "noisy, low-value logs",
];
const JEEBS_WANTS: &[&str] = &[
    "to expand knowledge coverage every cycle",
    "to reduce unanswered questions",
    "to prove intelligence with measurable improvements",
];

const LANGUAGE_UNLOCK_PREREQUISITES: &[&str] = &["rust programming", "python programming"];
const ADVANCED_LANGUAGE_TRACKS: &[&str] = &[
    "go programming",
    "javascript and typescript",
    "c and c++",
];

fn history_key(user_id: &str) -> String {
    format!("chat:history:{user_id}")
}

fn sanitize_turn_content(content: &str) -> String {
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= MAX_HISTORY_CHARS_PER_TURN {
        compact
    } else {
        truncate_chars(&compact, MAX_HISTORY_CHARS_PER_TURN)
    }
}

fn parse_history_blob(bytes: &[u8]) -> Option<Vec<ConversationTurn>> {
    let parsed = serde_json::from_slice::<Vec<ConversationTurn>>(bytes).ok()?;
    let mut cleaned = parsed
        .into_iter()
        .filter_map(|turn| {
            let role = turn.role.to_lowercase();
            if role != "user" && role != "assistant" {
                return None;
            }

            let content = sanitize_turn_content(&turn.content);
            if content.is_empty() {
                return None;
            }

            Some(ConversationTurn {
                role,
                content,
                timestamp: turn.timestamp,
            })
        })
        .collect::<Vec<_>>();

    if cleaned.len() > MAX_HISTORY_TURNS {
        cleaned.drain(0..(cleaned.len() - MAX_HISTORY_TURNS));
    }

    Some(cleaned)
}

async fn load_conversation_history(db: &SqlitePool, user_id: &str) -> Vec<ConversationTurn> {
    let key = history_key(user_id);
    let row = match sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let Some(row) = row else {
        return Vec::new();
    };

    let value: Vec<u8> = row.get(0);
    if let Some(history) = parse_history_blob(&value) {
        return history;
    }

    if let Ok(decoded) = decode_all(&value) {
        if let Some(history) = parse_history_blob(&decoded) {
            return history;
        }
    }

    Vec::new()
}

async fn save_conversation_history(
    db: &SqlitePool,
    user_id: &str,
    turns: &[ConversationTurn],
) -> Result<(), sqlx::Error> {
    let key = history_key(user_id);
    let mut trimmed = turns.to_vec();
    if trimmed.len() > MAX_HISTORY_TURNS {
        trimmed.drain(0..(trimmed.len() - MAX_HISTORY_TURNS));
    }

    let payload = serde_json::to_vec(&trimmed).unwrap_or_default();
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&key)
        .bind(payload)
        .execute(db)
        .await?;

    Ok(())
}

fn last_turn_by_role<'a>(
    history: &'a [ConversationTurn],
    role: &str,
) -> Option<&'a ConversationTurn> {
    history.iter().rev().find(|turn| turn.role == role)
}

fn is_follow_up_prompt(lower: &str) -> bool {
    matches!(
        lower,
        "ok" | "okay" | "sure" | "hmm" | "go on" | "continue" | "tell me more" | "elaborate"
    ) || lower.starts_with("why ")
        || lower == "why"
        || lower.starts_with("how ")
        || lower == "how"
        || lower.contains("what do you mean")
        || lower.contains("can you explain")
}

fn extract_name_from_intro(lower_input: &str) -> Option<String> {
    for prefix in ["my name is ", "i am ", "i'm "] {
        if let Some(rest) = lower_input.strip_prefix(prefix) {
            let candidate = rest
                .trim()
                .trim_matches(|ch: char| matches!(ch, '.' | ',' | '!' | '?'));
            if candidate.is_empty() {
                return None;
            }
            let mut chars = candidate.chars();
            let first = chars.next()?;
            let mut out = first.to_uppercase().collect::<String>();
            out.push_str(chars.as_str());
            return Some(out);
        }
    }
    None
}

fn recent_conversation_summary(history: &[ConversationTurn]) -> Option<String> {
    let recent = history
        .iter()
        .rev()
        .filter(|turn| turn.role == "user")
        .take(3)
        .map(|turn| truncate_chars(&turn.content, 80))
        .collect::<Vec<_>>();

    if recent.is_empty() {
        return None;
    }

    let mut ordered = recent;
    ordered.reverse();
    Some(ordered.join(" | "))
}

fn normalize_fact_text(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .trim_matches(|ch: char| matches!(ch, '.' | ',' | '!' | ';'))
        .to_string()
}

fn extract_learnable_fact(prompt: &str) -> Option<String> {
    let clean = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.is_empty() {
        return None;
    }

    let lower = clean.to_lowercase();
    if clean.ends_with('?')
        || lower.starts_with("remember:")
        || lower.starts_with("learn:")
        || lower.starts_with("forget:")
    {
        return None;
    }

    for prefix in [
        "remember that ",
        "remember this ",
        "please remember that ",
        "please remember ",
        "for future reference ",
        "fyi ",
    ] {
        if lower.starts_with(prefix) {
            let fact = normalize_fact_text(&clean[prefix.len()..]);
            if !fact.is_empty() {
                return Some(fact);
            }
        }
    }

    if lower.starts_with("my ") {
        let likely_fact = lower.contains(" is ")
            || lower.contains(" are ")
            || lower.contains(" was ")
            || lower.contains(" were ")
            || lower.contains(" favorite ")
            || lower.contains(" favourite ");
        if likely_fact {
            let fact = normalize_fact_text(&clean);
            if !fact.is_empty() {
                return Some(fact);
            }
        }
    }

    for prefix in [
        "i am ",
        "i'm ",
        "i live in ",
        "i live at ",
        "i work at ",
        "i work in ",
        "i like ",
        "i love ",
        "i prefer ",
    ] {
        if lower.starts_with(prefix) {
            let fact = normalize_fact_text(&clean);
            if !fact.is_empty() {
                return Some(fact);
            }
        }
    }

    None
}

fn sanitize_key_segment(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
            out.push(ch.to_ascii_lowercase());
        }
    }
    if out.is_empty() {
        "anonymous".to_string()
    } else {
        out
    }
}

fn fact_owner_key(user_id: &str, username: Option<&str>) -> String {
    if let Some(name) = username {
        return format!("user:{}", sanitize_key_segment(name));
    }
    format!("session:{}", sanitize_key_segment(user_id))
}

fn fact_prefix(owner: &str) -> String {
    format!("chat:fact:{owner}:")
}

fn fact_storage_key(owner: &str, canonical: &str) -> String {
    format!(
        "{}{}",
        fact_prefix(owner),
        blake3::hash(canonical.as_bytes()).to_hex()
    )
}

fn parse_learned_fact_bytes(bytes: &[u8]) -> Option<LearnedFact> {
    let parsed = serde_json::from_slice::<LearnedFact>(bytes).ok()?;
    let fact = sanitize_turn_content(&parsed.fact);
    let canonical = canonical_prompt_key(&parsed.canonical);
    if fact.is_empty() || canonical.is_empty() {
        return None;
    }
    Some(LearnedFact {
        fact,
        canonical,
        created_at: parsed.created_at,
        updated_at: parsed.updated_at,
    })
}

async fn save_learned_fact(
    db: &SqlitePool,
    owner: &str,
    fact: &str,
) -> Result<LearnedFact, sqlx::Error> {
    let cleaned_fact = sanitize_turn_content(fact);
    let canonical = canonical_prompt_key(&cleaned_fact);
    let key = fact_storage_key(owner, &canonical);
    let now = Local::now().to_rfc3339();

    let existing = sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(&key)
        .fetch_optional(db)
        .await?;

    let created_at = existing
        .and_then(|row| {
            let value: Vec<u8> = row.get(0);
            parse_learned_fact_bytes(&value)
                .map(|fact| fact.created_at)
                .or_else(|| {
                    decode_all(&value)
                        .ok()
                        .and_then(|d| parse_learned_fact_bytes(&d))
                        .map(|fact| fact.created_at)
                })
        })
        .unwrap_or_else(|| now.clone());

    let payload = LearnedFact {
        fact: cleaned_fact.clone(),
        canonical: canonical.clone(),
        created_at,
        updated_at: now,
    };

    let bytes = serde_json::to_vec(&payload).unwrap_or_default();
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(&key)
        .bind(bytes)
        .execute(db)
        .await?;

    Ok(payload)
}

async fn load_learned_facts(db: &SqlitePool, owner: &str) -> Vec<LearnedFact> {
    let pattern = format!("{}%", fact_prefix(owner));
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE ? ORDER BY key ASC")
        .bind(pattern)
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut facts = Vec::new();
    for row in rows {
        let value: Vec<u8> = row.get(0);
        if let Some(fact) = parse_learned_fact_bytes(&value) {
            facts.push(fact);
            continue;
        }
        if let Ok(decoded) = decode_all(&value) {
            if let Some(fact) = parse_learned_fact_bytes(&decoded) {
                facts.push(fact);
            }
        }
    }

    facts
}

fn is_token_stopword(token: &str) -> bool {
    matches!(
        token,
        "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "of"
            | "to"
            | "for"
            | "in"
            | "on"
            | "is"
            | "are"
            | "do"
            | "did"
            | "you"
            | "your"
            | "my"
            | "me"
            | "what"
            | "who"
            | "how"
            | "why"
            | "about"
            | "remember"
            | "know"
    )
}

fn tokenize_for_matching(input: &str) -> Vec<String> {
    let mut normalized = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else {
            normalized.push(' ');
        }
    }

    normalized
        .split_whitespace()
        .filter(|token| token.len() >= 3 && !is_token_stopword(token))
        .map(|token| token.to_string())
        .collect()
}

fn rank_relevant_facts(facts: &[LearnedFact], query: &str, limit: usize) -> Vec<LearnedFact> {
    let query_tokens = tokenize_for_matching(query);
    if query_tokens.is_empty() {
        return Vec::new();
    }

    let query_lower = query.to_lowercase();
    let mut scored = Vec::new();
    for fact in facts {
        let fact_lower = fact.fact.to_lowercase();
        let mut score = 0_i32;
        for token in &query_tokens {
            if fact_lower.contains(token) || fact.canonical.contains(token) {
                score += 1;
            }
        }
        if query_lower.contains(&fact.canonical) || fact_lower.contains(&query_lower) {
            score += 3;
        }
        if score > 0 {
            scored.push((score, fact.updated_at.clone(), fact.clone()));
        }
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, _, fact)| fact)
        .collect()
}

fn most_recent_facts(facts: &[LearnedFact], limit: usize) -> Vec<LearnedFact> {
    let mut sorted = facts.to_vec();
    sorted.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    sorted.into_iter().take(limit).collect()
}

fn wants_personal_memory_overview(lower: &str) -> bool {
    lower.contains("what do you know about me")
        || lower.contains("what do you remember about me")
        || lower.contains("what have you learned about me")
        || lower.contains("tell me about me")
        || lower.contains("remind me what you know about me")
}

fn wants_personal_memory_lookup(lower: &str) -> bool {
    lower.contains("do you remember")
        || lower.contains("what is my ")
        || lower.contains("what's my ")
        || lower.contains("what are my ")
        || lower.contains("where do i live")
        || lower.contains("what do i like")
        || lower.contains("tell me my ")
}

fn comprehension_key(owner: &str) -> String {
    format!("chat:comprehension:{owner}")
}

async fn save_communication_profile(
    db: &SqlitePool,
    owner: &str,
    profile: &CommunicationProfile,
) -> Result<(), sqlx::Error> {
    let payload = serde_json::to_vec(profile).unwrap_or_default();
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(comprehension_key(owner))
        .bind(payload)
        .execute(db)
        .await?;
    Ok(())
}

async fn load_communication_profile(db: &SqlitePool, owner: &str) -> Option<CommunicationProfile> {
    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(comprehension_key(owner))
        .fetch_optional(db)
        .await
        .ok()??;
    let raw: Vec<u8> = row.get(0);
    serde_json::from_slice::<CommunicationProfile>(&raw)
        .ok()
        .or_else(|| {
            decode_all(&raw)
                .ok()
                .and_then(|decoded| serde_json::from_slice::<CommunicationProfile>(&decoded).ok())
        })
}

fn infer_recent_topics(turns: &[ConversationTurn], limit: usize) -> Vec<String> {
    let mut topics = HashMap::<String, usize>::new();
    for turn in turns.iter().filter(|t| t.role == "user").rev().take(8) {
        for token in tokenize_for_matching(&turn.content) {
            if token.len() >= 4 {
                *topics.entry(token).or_insert(0) += 1;
            }
        }
    }
    let mut scored = topics.into_iter().collect::<Vec<_>>();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored
        .into_iter()
        .take(limit)
        .map(|(topic, _)| topic)
        .collect()
}

fn analyze_communication_profile(
    prompt: &str,
    history: &[ConversationTurn],
    previous: Option<&CommunicationProfile>,
) -> CommunicationProfile {
    let mut recent_user_turns = history
        .iter()
        .filter(|turn| turn.role == "user")
        .rev()
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    recent_user_turns.reverse();

    if !prompt.trim().is_empty() {
        recent_user_turns.push(ConversationTurn {
            role: "user".to_string(),
            content: sanitize_turn_content(prompt),
            timestamp: Local::now().to_rfc3339(),
        });
    }

    let mut question_count = 0usize;
    let mut command_like_count = 0usize;
    let mut gratitude_count = 0usize;
    let mut frustration_count = 0usize;
    let mut long_message_count = 0usize;

    for turn in &recent_user_turns {
        let lower = turn.content.to_lowercase();
        if lower.contains('?') {
            question_count += 1;
        }
        if lower.starts_with("please ")
            || lower.starts_with("do ")
            || lower.starts_with("add ")
            || lower.starts_with("make ")
            || lower.starts_with("fix ")
            || lower.starts_with("give ")
            || lower.starts_with("build ")
        {
            command_like_count += 1;
        }
        if lower.contains("thanks") || lower.contains("thank you") {
            gratitude_count += 1;
        }
        if lower.contains("not working")
            || lower.contains("still no")
            || lower.contains("broken")
            || lower.contains("wtf")
            || lower.contains("why isn't")
        {
            frustration_count += 1;
        }
        if turn.content.chars().count() >= 120 {
            long_message_count += 1;
        }
    }

    let style = if frustration_count >= 1 {
        "frustrated"
    } else if question_count >= 3 {
        "curious"
    } else if command_like_count >= 2 {
        "direct"
    } else if long_message_count >= 2 {
        "reflective"
    } else if gratitude_count >= 1 {
        "collaborative"
    } else {
        "neutral"
    }
    .to_string();

    let mut signals = Vec::new();
    signals.push(format!(
        "recent_questions={}, direct_requests={}, long_messages={}",
        question_count, command_like_count, long_message_count
    ));
    if frustration_count >= 1 {
        signals.push("frustration detected in recent phrasing".to_string());
    }
    if gratitude_count >= 1 {
        signals.push("appreciation language detected".to_string());
    }
    if let Some(previous) = previous {
        if previous.style != style {
            signals.push(format!(
                "style shifted from {} to {}",
                previous.style, style
            ));
        }
    }

    CommunicationProfile {
        style,
        signals,
        recent_topics: infer_recent_topics(&recent_user_turns, 6),
        updated_at: Local::now().to_rfc3339(),
    }
}

fn wants_communication_reflection(lower: &str) -> bool {
    lower.contains("how am i communicating")
        || lower.contains("how do i communicate")
        || lower.contains("my communication style")
        || lower.contains("how am i coming across")
        || lower.contains("am i being clear")
        || lower.contains("what do you think of my communication")
}

fn render_communication_reflection(profile: &CommunicationProfile) -> String {
    let mut lines = vec![format!(
        "You are communicating in a {} style right now.",
        profile.style
    )];
    if !profile.recent_topics.is_empty() {
        lines.push(format!(
            "Recent topics you focus on: {}.",
            profile.recent_topics.join(", ")
        ));
    }
    if let Some(signal) = profile.signals.first() {
        lines.push(format!("Signal snapshot: {signal}."));
    }
    lines.join(" ")
}

fn training_interval_seconds() -> u64 {
    env::var("TRAINING_MODE_INTERVAL_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .map(|value| value.clamp(5, 3600))
        .unwrap_or(DEFAULT_TRAINING_INTERVAL_SECS)
}

fn training_state_default() -> TrainingModeState {
    TrainingModeState {
        enabled: true,  // AUTO-RUN training mode by default
        updated_at: Local::now().to_rfc3339(),
        updated_by: "system".to_string(),
        last_cycle_at: None,
        total_cycles: 0,
        total_topics_processed: 0,
        total_nodes_written: 0,
        total_websites_scraped: 0,
        total_crawl_pages_visited: 0,
        total_crawl_pages_stored: 0,
        total_crawl_links_followed: 0,
        total_crawl_nodes_written: 0,
        total_wikipedia_docs_written: 0,
        last_topics: Vec::new(),
        last_error: None,
        last_websites: Vec::new(),
        last_learned_items: Vec::new(),
        last_cycle_duration_ms: None,
        last_cycle_nodes_written: 0,
        last_cycle_errors: Vec::new(),
        last_cycle_summary: None,
        recent_cycles: Vec::new(),
        is_cycle_running: false,
        active_cycle_started_at: None,
        active_phase: default_active_phase(),
        active_target: None,
        active_nodes_written: 0,
        active_websites_completed: 0,
        active_topics_completed: 0,
        active_updated_at: None,
        focus_topic: None,
        smartness_score: 0.0,
        total_text_chars_learned: 0,
        learning_tracks: default_training_tracks(),
    }
}

async fn load_training_state(db: &SqlitePool) -> TrainingModeState {
    let row = sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(TRAINING_STATE_KEY)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    let Some(row) = row else {
        return training_state_default();
    };

    let raw: Vec<u8> = row.get(0);
    let mut state = serde_json::from_slice::<TrainingModeState>(&raw)
        .ok()
        .or_else(|| {
            decode_all(&raw)
                .ok()
                .and_then(|decoded| serde_json::from_slice::<TrainingModeState>(&decoded).ok())
        })
        .unwrap_or_else(training_state_default);

    if state.learning_tracks.is_empty() {
        state.learning_tracks = default_training_tracks();
    }
    refresh_track_statuses(&mut state);
    state.smartness_score = smartness_score(&state);
    state
}

async fn save_training_state(
    db: &SqlitePool,
    state: &TrainingModeState,
) -> Result<(), sqlx::Error> {
    let payload = serde_json::to_vec(state).unwrap_or_default();
    sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
        .bind(TRAINING_STATE_KEY)
        .bind(payload)
        .execute(db)
        .await?;
    Ok(())
}

async fn mutate_training_state<F>(db: &SqlitePool, mutator: F)
where
    F: FnOnce(&mut TrainingModeState),
{
    let mut mode = load_training_state(db).await;
    mutator(&mut mode);
    mode.updated_at = Local::now().to_rfc3339();
    if mode.updated_by.trim().is_empty() {
        mode.updated_by = "training_runtime".to_string();
    }
    let _ = save_training_state(db, &mode).await;
}

fn report_to_snapshot(report: &TrainingCycleReport) -> TrainingCycleSnapshot {
    TrainingCycleSnapshot {
        cycle_started_at: report.cycle_started_at.clone(),
        cycle_finished_at: report.cycle_finished_at.clone(),
        duration_ms: report.duration_ms,
        topics: report.topics.clone(),
        websites_scraped: report.websites_scraped.clone(),
        nodes_written: report.nodes_written as u64,
        crawl_pages_visited: report.crawl_pages_visited as u64,
        crawl_pages_stored: report.crawl_pages_stored as u64,
        crawl_links_followed: report.crawl_links_followed as u64,
        crawl_nodes_written: report.crawl_nodes_written as u64,
        wikipedia_docs_written: report.wikipedia_docs_written as u64,
        text_chars_learned: report.text_chars_learned as u64,
        focus_topic: report.focus_topic.clone(),
        focus_topic_nodes_written: report.focus_topic_nodes_written as u64,
        learned_items_count: report.learned_items.len() as u64,
        errors: report.errors.clone(),
    }
}

fn apply_training_report(mode: &mut TrainingModeState, report: &TrainingCycleReport, actor: &str) {
    if mode.learning_tracks.is_empty() {
        mode.learning_tracks = default_training_tracks();
    }

    mode.last_cycle_at = Some(report.cycle_finished_at.clone());
    mode.total_cycles = mode.total_cycles.saturating_add(1);
    mode.total_topics_processed = mode
        .total_topics_processed
        .saturating_add(report.topics.len() as u64);
    mode.total_nodes_written = mode
        .total_nodes_written
        .saturating_add(report.nodes_written as u64);
    mode.total_websites_scraped = mode
        .total_websites_scraped
        .saturating_add(report.websites_scraped.len() as u64);
    mode.total_crawl_pages_visited = mode
        .total_crawl_pages_visited
        .saturating_add(report.crawl_pages_visited as u64);
    mode.total_crawl_pages_stored = mode
        .total_crawl_pages_stored
        .saturating_add(report.crawl_pages_stored as u64);
    mode.total_crawl_links_followed = mode
        .total_crawl_links_followed
        .saturating_add(report.crawl_links_followed as u64);
    mode.total_crawl_nodes_written = mode
        .total_crawl_nodes_written
        .saturating_add(report.crawl_nodes_written as u64);
    mode.total_wikipedia_docs_written = mode
        .total_wikipedia_docs_written
        .saturating_add(report.wikipedia_docs_written as u64);
    mode.total_text_chars_learned = mode
        .total_text_chars_learned
        .saturating_add(report.text_chars_learned as u64);

    mode.last_topics = report.topics.clone();
    mode.last_websites = report.websites_scraped.clone();
    mode.last_learned_items = report.learned_items.clone();
    mode.last_error = report.errors.first().cloned();
    mode.last_cycle_duration_ms = Some(report.duration_ms);
    mode.last_cycle_nodes_written = report.nodes_written as u64;
    mode.last_cycle_errors = report.errors.clone();

    for track in &mut mode.learning_tracks {
        let increment = report.track_hits.get(&track.name).copied().unwrap_or(0);
        if increment > 0 {
            track.nodes_written = track.nodes_written.saturating_add(increment);
            track.last_learned_at = Some(report.cycle_finished_at.clone());
        }
    }
    refresh_track_statuses(mode);

    let snapshot = report_to_snapshot(report);
    mode.last_cycle_summary = Some(snapshot.clone());
    mode.recent_cycles.insert(0, snapshot);
    if mode.recent_cycles.len() > 30 {
        mode.recent_cycles.truncate(30);
    }

    mode.updated_at = report.cycle_finished_at.clone();
    mode.updated_by = actor.to_string();

    mode.is_cycle_running = false;
    mode.active_cycle_started_at = None;
    mode.active_phase = default_active_phase();
    mode.active_target = None;
    mode.active_nodes_written = 0;
    mode.active_websites_completed = 0;
    mode.active_topics_completed = 0;
    mode.active_updated_at = Some(report.cycle_finished_at.clone());
    mode.smartness_score = smartness_score(mode);
}

fn finalize_training_report(report: &mut TrainingCycleReport, timer: &Instant) {
    report.cycle_finished_at = Local::now().to_rfc3339();
    let elapsed_ms = timer.elapsed().as_millis();
    report.duration_ms = elapsed_ms.min(u128::from(u64::MAX)) as u64;
}

fn extract_question_topic(question: &str) -> String {
    let tokens = tokenize_for_matching(question);
    if tokens.is_empty() {
        return canonical_prompt_key(question);
    }
    tokens.into_iter().take(5).collect::<Vec<_>>().join(" ")
}

async fn collect_training_topics(db: &SqlitePool, limit: usize) -> Vec<String> {
    let rows = sqlx::query("SELECT value FROM jeebs_store WHERE key LIKE 'chat:history:%'")
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut counts = HashMap::<String, usize>::new();
    for row in rows {
        let raw: Vec<u8> = row.get(0);
        let parsed = serde_json::from_slice::<Vec<ConversationTurn>>(&raw)
            .ok()
            .or_else(|| {
                decode_all(&raw).ok().and_then(|decoded| {
                    serde_json::from_slice::<Vec<ConversationTurn>>(&decoded).ok()
                })
            });
        let Some(turns) = parsed else {
            continue;
        };

        for turn in turns.iter().rev().take(24) {
            if turn.role != "user" || !turn.content.contains('?') {
                continue;
            }
            let topic = extract_question_topic(&turn.content);
            if topic.is_empty() {
                continue;
            }
            *counts.entry(topic).or_insert(0) += 1;
        }
    }

    let mut topics = counts.into_iter().collect::<Vec<_>>();
    topics.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut out = topics
        .into_iter()
        .take(limit)
        .map(|(topic, _)| topic)
        .collect::<Vec<_>>();

    if out.is_empty() {
        out = vec![
            "artificial intelligence".to_string(),
            "software engineering best practices".to_string(),
            "rust programming".to_string(),
            "internet security basics".to_string(),
        ];
    }

    out
}

#[derive(Debug, Deserialize)]
struct WikiSummaryResponse {
    title: Option<String>,
    extract: Option<String>,
    #[serde(default)]
    content_urls: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct ExternalLearningDoc {
    title: String,
    url: String,
    summary: String,
    topic: String,
}

async fn query_wikipedia_docs(
    client: &reqwest::Client,
    topic: &str,
    max_docs: usize,
) -> Result<Vec<ExternalLearningDoc>, String> {
    let mut search_url = reqwest::Url::parse("https://en.wikipedia.org/w/api.php")
        .map_err(|err| format!("wikipedia search url build failed: {err}"))?;
    {
        let mut pairs = search_url.query_pairs_mut();
        pairs.append_pair("action", "opensearch");
        pairs.append_pair("search", topic);
        pairs.append_pair("limit", "5");
        pairs.append_pair("namespace", "0");
        pairs.append_pair("format", "json");
    }

    let response = client
        .get(search_url)
        .send()
        .await
        .map_err(|err| format!("wikipedia search request failed: {err}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "wikipedia search failed with status {}",
            response.status()
        ));
    }

    let search_raw = response
        .text()
        .await
        .map_err(|err| format!("wikipedia search read failed: {err}"))?;
    let search = serde_json::from_str::<serde_json::Value>(&search_raw)
        .map_err(|err| format!("wikipedia search parse failed: {err}"))?;
    let titles = search
        .get(1)
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let mut docs = Vec::new();
    for title_value in titles.into_iter().take(max_docs) {
        let title = title_value.as_str().map(str::to_string).unwrap_or_default();
        if title.trim().is_empty() {
            continue;
        }

        let mut summary_url =
            reqwest::Url::parse("https://en.wikipedia.org/api/rest_v1/page/summary/")
                .map_err(|err| format!("summary url build failed: {err}"))?;
        summary_url
            .path_segments_mut()
            .map_err(|_| "failed to build summary path".to_string())?
            .pop_if_empty()
            .push(&title);

        let summary_resp = client
            .get(summary_url)
            .send()
            .await
            .map_err(|err| format!("wikipedia summary request failed: {err}"))?;

        if !summary_resp.status().is_success() {
            continue;
        }

        let summary_raw = summary_resp
            .text()
            .await
            .map_err(|err| format!("wikipedia summary read failed: {err}"))?;
        let payload = serde_json::from_str::<WikiSummaryResponse>(&summary_raw)
            .map_err(|err| format!("wikipedia summary parse failed: {err}"))?;

        let resolved_title = payload
            .title
            .unwrap_or_else(|| title.clone())
            .trim()
            .to_string();
        let extract = payload
            .extract
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if resolved_title.is_empty() || extract.is_empty() {
            continue;
        }

        let page_url = payload
            .content_urls
            .as_ref()
            .and_then(|urls| urls.get("desktop"))
            .and_then(|desktop| desktop.get("page"))
            .and_then(|page| page.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!(
                    "https://en.wikipedia.org/wiki/{}",
                    resolved_title.replace(' ', "_")
                )
            });

        docs.push(ExternalLearningDoc {
            title: resolved_title,
            url: page_url,
            summary: truncate_chars(&extract, 900),
            topic: topic.to_string(),
        });
    }

    Ok(docs)
}

async fn store_external_learning_doc(
    db: &SqlitePool,
    doc: &ExternalLearningDoc,
) -> Result<String, sqlx::Error> {
    let normalized_url = doc.url.trim();
    let node_id = format!("train:{}", blake3::hash(normalized_url.as_bytes()).to_hex());
    let payload = serde_json::to_vec(&json!({
        "source": "training_mode",
        "provider": "wikipedia",
        "topic": doc.topic,
        "url": doc.url,
        "title": doc.title,
        "summary": doc.summary,
        "trained_at": Local::now().to_rfc3339()
    }))
    .unwrap_or_default();

    sqlx::query(
        "INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&node_id)
    .bind(&doc.title)
    .bind(&doc.summary)
    .bind(payload)
    .bind(Local::now().to_rfc3339())
    .execute(db)
    .await?;

    let _ = sqlx::query(
        "INSERT OR REPLACE INTO knowledge_triples (subject, predicate, object, confidence, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&doc.topic)
    .bind("researched_from")
    .bind(&doc.url)
    .bind(0.82_f64)
    .bind(Local::now().to_rfc3339())
    .execute(db)
    .await;

    Ok(node_id)
}

async fn run_training_cycle(state: &AppState) -> TrainingCycleReport {
    let cycle_started_at = Local::now().to_rfc3339();
    let cycle_timer = Instant::now();
    let min_duration = Duration::from_secs(60);  // 1 minute minimum
    let max_duration = Duration::from_secs(300); // 5 minutes maximum

    let mut report = TrainingCycleReport {
        cycle_started_at: cycle_started_at.clone(),
        cycle_finished_at: cycle_started_at.clone(),
        duration_ms: 0,
        topics: Vec::new(),
        nodes_written: 0,
        errors: Vec::new(),
        websites_scraped: Vec::new(),
        learned_items: Vec::new(),
        crawl_pages_visited: 0,
        crawl_pages_stored: 0,
        crawl_links_followed: 0,
        crawl_nodes_written: 0,
        wikipedia_docs_written: 0,
        text_chars_learned: 0,
    };

    let cycle_started_for_state = cycle_started_at.clone();
    mutate_training_state(&state.db, move |mode| {
        mode.is_cycle_running = true;
        mode.active_cycle_started_at = Some(cycle_started_for_state);
        mode.active_phase = "starting training cycle".to_string();
        mode.active_target = None;
        mode.active_nodes_written = 0;
        mode.active_websites_completed = 0;
        mode.active_topics_completed = 0;
        mode.active_updated_at = Some(Local::now().to_rfc3339());
    })
    .await;

    if !*state.internet_enabled.read().unwrap() {
        report
            .errors
            .push("internet is disabled; enable it in admin first".to_string());
        finalize_training_report(&mut report, &cycle_timer);
        return report;
    }

    let mut topics = collect_training_topics(&state.db, 4).await;
    for curiosity_topic in jeebs_curiosity_topics() {
        if topics.len() >= 7 {
            break;
        }
        if !topics.contains(&curiosity_topic) {
            topics.push(curiosity_topic);
        }
    }
    report.topics = topics.clone();

    let topics_count = report.topics.len() as u64;
    mutate_training_state(&state.db, move |mode| {
        mode.active_phase = "collecting topics and preparing sources".to_string();
        mode.active_target = Some(format!("{topics_count} topic(s) queued"));
        mode.active_updated_at = Some(Local::now().to_rfc3339());
    })
    .await;

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("JeebsAI-TrainingMode/1.0")
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            report
                .errors
                .push(format!("failed to initialize training client: {err}"));
            finalize_training_report(&mut report, &cycle_timer);
            return report;
        }
    };

    let crawl_depth = env::var("TRAINING_MODE_CRAWL_DEPTH")
        .ok()
        .and_then(|raw| raw.parse::<u8>().ok())
        .map(|value| value.clamp(1, 3))
        .unwrap_or(1);
    let random_site_count = env::var("TRAINING_MODE_RANDOM_SITES_PER_CYCLE")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .map(|value| value.clamp(1, 5))
        .unwrap_or(2);

    let random_sites = {
        let mut rng = rand::thread_rng();
        let mut sites = random_crawl_candidates();
        sites.shuffle(&mut rng);
        sites
            .into_iter()
            .take(random_site_count)
            .collect::<Vec<_>>()
    };

    let mut websites_completed = 0_u64;
    let mut topics_completed = 0_u64;

    // CRAWL RANDOM WEBSITES FIRST
    for site in random_sites {
        // Check maximum duration
        if cycle_timer.elapsed() >= max_duration {
            mutate_training_state(&state.db, move |mode| {
                mode.active_phase = "maximum duration reached, stopping".to_string();
                mode.active_updated_at = Some(Local::now().to_rfc3339());
            })
            .await;
            break;
        }

        let target_site = site.to_string();
        mutate_training_state(&state.db, move |mode| {
            mode.active_phase = "crawling random website".to_string();
            mode.active_target = Some(target_site);
            mode.active_updated_at = Some(Local::now().to_rfc3339());
        })
        .await;

        match crawl_and_store(state, site, crawl_depth).await {
            Ok(summary) => {
                websites_completed = websites_completed.saturating_add(1);
                report.websites_scraped.push(summary.start_url.clone());
                report.nodes_written += summary.pages_stored;
                report.crawl_pages_visited += summary.pages_visited;
                report.crawl_pages_stored += summary.pages_stored;
                report.crawl_links_followed += summary.links_followed;
                report.crawl_nodes_written += summary.pages_stored;
                for node in summary.stored_nodes.into_iter().take(8) {
                    report.learned_items.push(TrainingLearnedItem {
                        node_id: node.node_id,
                        title: node.label,
                        summary: node.summary,
                        source_url: node.source_url,
                        topic: "random_web_crawl".to_string(),
                        source_type: "crawl".to_string(),
                    });
                }
            }
            Err(err) => report.errors.push(format!("random site '{site}': {err}")),
        }

        let active_nodes_written = report.nodes_written as u64;
        mutate_training_state(&state.db, move |mode| {
            mode.active_nodes_written = active_nodes_written;
            mode.active_websites_completed = websites_completed;
            mode.active_topics_completed = topics_completed;
            mode.active_updated_at = Some(Local::now().to_rfc3339());
        })
        .await;
    }

    // RESEARCH TOPICS - continue if under minimum, stop if over maximum
    for topic in topics {
        // Check maximum duration
        if cycle_timer.elapsed() >= max_duration {
            mutate_training_state(&state.db, move |mode| {
                mode.active_phase = "maximum duration reached, stopping".to_string();
                mode.active_updated_at = Some(Local::now().to_rfc3339());
            })
            .await;
            break;
        }

        let target_topic = topic.clone();
        mutate_training_state(&state.db, move |mode| {
            mode.active_phase = "researching topic".to_string();
            mode.active_target = Some(target_topic);
            mode.active_updated_at = Some(Local::now().to_rfc3339());
        })
        .await;

        match query_wikipedia_docs(&client, &topic, 2).await {
            Ok(docs) => {
                for doc in docs {
                    match store_external_learning_doc(&state.db, &doc).await {
                        Ok(node_id) => {
                            report.nodes_written += 1;
                            report.learned_items.push(TrainingLearnedItem {
                                node_id,
                                title: doc.title.clone(),
                                summary: doc.summary.clone(),
                                source_url: doc.url.clone(),
                                topic: doc.topic.clone(),
                                source_type: "wikipedia".to_string(),
                            });
                            report.wikipedia_docs_written += 1;
                        }
                        Err(err) => report.errors.push(format!(
                            "failed to store learned doc '{}': {err}",
                            doc.title
                        )),
                    }

                    let active_nodes_written = report.nodes_written as u64;
                    let active_docs_written = report.wikipedia_docs_written as u64;
                    mutate_training_state(&state.db, move |mode| {
                        mode.active_nodes_written = active_nodes_written;
                        mode.active_target = Some(format!(
                            "writing wikipedia docs ({active_docs_written} this cycle)"
                        ));
                        mode.active_updated_at = Some(Local::now().to_rfc3339());
                    })
                    .await;
                }
            }
            Err(err) => report.errors.push(format!("topic '{topic}': {err}")),
        }

        topics_completed = topics_completed.saturating_add(1);
        let active_nodes_written = report.nodes_written as u64;
        mutate_training_state(&state.db, move |mode| {
            mode.active_nodes_written = active_nodes_written;
            mode.active_websites_completed = websites_completed;
            mode.active_topics_completed = topics_completed;
            mode.active_updated_at = Some(Local::now().to_rfc3339());
        })
        .await;
    }

    // ENFORCE MINIMUM DURATION - if we finished too quickly, continue exploring
    while cycle_timer.elapsed() < min_duration && report.errors.is_empty() {
        // Check if we should continue
        if cycle_timer.elapsed() >= max_duration {
            break;
        }

        // Keep exploring - pick a random topic and crawl again
        mutate_training_state(&state.db, move |mode| {
            mode.active_phase = "continuing exploration (minimum duration)".to_string();
            mode.active_updated_at = Some(Local::now().to_rfc3339());
        })
        .await;

        // Pick a random site to continue exploring
        let mut rng = rand::thread_rng();
        let sites = random_crawl_candidates();
        if !sites.is_empty() {
            let idx = rng.gen_range(0..sites.len());
            let site = sites[idx];

            let target_site = site.to_string();
            mutate_training_state(&state.db, move |mode| {
                mode.active_phase = "exploring additional domain (minimum duration)".to_string();
                mode.active_target = Some(target_site);
                mode.active_updated_at = Some(Local::now().to_rfc3339());
            })
            .await;

            let crawl_depth = 1; // Use depth 1 for additional explorations
            match crawl_and_store(state, site, crawl_depth).await {
                Ok(summary) => {
                    websites_completed = websites_completed.saturating_add(1);
                    if !report.websites_scraped.contains(&summary.start_url) {
                        report.websites_scraped.push(summary.start_url.clone());
                    }
                    report.nodes_written += summary.pages_stored;
                    report.crawl_pages_visited += summary.pages_visited;
                    report.crawl_pages_stored += summary.pages_stored;
                    report.crawl_links_followed += summary.links_followed;
                    report.crawl_nodes_written += summary.pages_stored;
                    for node in summary.stored_nodes.into_iter().take(4) {
                        if report.learned_items.len() < 32 {
                            report.learned_items.push(TrainingLearnedItem {
                                node_id: node.node_id,
                                title: node.label,
                                summary: node.summary,
                                source_url: node.source_url,
                                topic: "extended_exploration".to_string(),
                                source_type: "crawl".to_string(),
                            });
                        }
                    }
                }
                Err(_) => {
                    // Continue trying other sources
                }
            }

            let active_nodes_written = report.nodes_written as u64;
            mutate_training_state(&state.db, move |mode| {
                mode.active_nodes_written = active_nodes_written;
                mode.active_websites_completed = websites_completed;
                mode.active_updated_at = Some(Local::now().to_rfc3339());
            })
            .await;

            // Small delay to avoid hammering
            tokio::time::sleep(Duration::from_millis(100)).await;
        } else {
            break;
        }
    }

    if report.learned_items.len() > 24 {
        report.learned_items.truncate(24);
    }

    finalize_training_report(&mut report, &cycle_timer);
    report
}

pub fn spawn_autonomous_training(state: web::Data<AppState>) {
    tokio::spawn(async move {
        crate::logging::log(
            &state.db,
            "INFO",
            "training_mode",
            "Autonomous training worker started.",
        )
        .await;

        loop {
            let mut mode = load_training_state(&state.db).await;
            if mode.enabled {
                let report = run_training_cycle(state.get_ref()).await;
                apply_training_report(&mut mode, &report, "autonomous_worker");
                let _ = save_training_state(&state.db, &mode).await;

                crate::logging::log(
                    &state.db,
                    "INFO",
                    "training_mode",
                    &format!(
                        "Training cycle complete. duration_ms={} topics={} websites={} nodes_written={} crawl_pages_visited={} crawl_links_followed={} wiki_docs={} errors={}",
                        report.duration_ms,
                        report.topics.len(),
                        report.websites_scraped.len(),
                        report.nodes_written,
                        report.crawl_pages_visited,
                        report.crawl_links_followed,
                        report.wikipedia_docs_written,
                        report.errors.len()
                    ),
                )
                .await;
            }

            tokio::time::sleep(Duration::from_secs(training_interval_seconds())).await;
        }
    });
}

pub async fn search_knowledge(db: &SqlitePool, query: &str) -> Vec<BrainNode> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, COALESCE(label, id) AS label, COALESCE(summary, '') AS summary, COALESCE(created_at, '') AS created_at
         FROM brain_nodes
         WHERE id LIKE ? OR label LIKE ? OR summary LIKE ?
         ORDER BY created_at DESC
         LIMIT 10",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|row| {
            let raw_id: String = row.get(0);
            let label: String = row.get(1);
            let summary: String = row.get(2);
            let created_at: String = row.get(3);
            BrainNode {
                id: raw_id.parse::<i64>().ok(),
                key: raw_id.clone(),
                value: summary.clone(),
                label,
                summary,
                created_at,
            }
        })
        .collect()
}

async fn check_dejavu(prompt: &str, db: &SqlitePool) -> Option<String> {
    fn parse_answer_bytes(bytes: &[u8]) -> Option<String> {
        if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(bytes) {
            if let Some(answer) = json_value.get("answer").and_then(|v| v.as_str()) {
                let answer = answer.trim();
                if !answer.is_empty() {
                    return Some(answer.to_string());
                }
            }
            if let Some(answer) = json_value.get("response").and_then(|v| v.as_str()) {
                let answer = answer.trim();
                if !answer.is_empty() {
                    return Some(answer.to_string());
                }
            }
        }

        if let Ok(text) = std::str::from_utf8(bytes) {
            let text = text.trim();
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }

        None
    }

    let key = format!("chat:faq:{}", canonical_prompt_key(prompt));
    match sqlx::query("SELECT value FROM jeebs_store WHERE key = ? LIMIT 1")
        .bind(&key)
        .fetch_optional(db)
        .await
    {
        Ok(Some(row)) => {
            let value: Vec<u8> = row.get(0);
            if let Some(answer) = parse_answer_bytes(&value) {
                return Some(answer);
            }

            if let Ok(decoded) = decode_all(&value) {
                return parse_answer_bytes(&decoded);
            }

            None
        }
        _ => None,
    }
}

async fn search_brain_for_chat(db: &SqlitePool, query: &str) -> Vec<(String, String)> {
    let pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT COALESCE(label, id) AS label, COALESCE(summary, '') AS summary
         FROM brain_nodes
         WHERE id LIKE ? OR label LIKE ? OR summary LIKE ?
         ORDER BY created_at DESC
         LIMIT 3",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|row| {
            let label: String = row.get(0);
            let summary: String = row.get(1);
            (label, summary)
        })
        .collect()
}

fn looks_like_math_expression(expr: &str) -> bool {
    let compact = expr.trim();
    if compact.is_empty() {
        return false;
    }

    let mut has_digit = false;
    for ch in compact.chars() {
        if ch.is_ascii_digit() {
            has_digit = true;
            continue;
        }
        if matches!(
            ch,
            ' ' | '+' | '-' | '*' | '/' | '(' | ')' | '.' | '^' | '%'
        ) {
            continue;
        }
        return false;
    }

    has_digit
}

fn extract_math_expression(prompt: &str, lower: &str) -> Option<String> {
    for prefix in ["calculate ", "calc ", "math ", "solve "] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let expr = rest.trim().replace(',', "");
            if looks_like_math_expression(&expr) {
                return Some(expr);
            }
        }
    }

    if let Some(rest) = lower.strip_prefix("what is ") {
        let expr = rest.trim_end_matches('?').trim().replace(',', "");
        if looks_like_math_expression(&expr) {
            return Some(expr);
        }
    }

    let direct = prompt.trim().replace(',', "");
    if looks_like_math_expression(&direct) {
        return Some(direct);
    }

    None
}

fn format_number(value: f64) -> String {
    let rounded = (value * 1_000_000_000.0).round() / 1_000_000_000.0;
    let mut s = format!("{rounded}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

fn is_greeting(lower: &str) -> bool {
    matches!(
        lower,
        "hi" | "hello" | "hey" | "yo" | "sup" | "good morning" | "good afternoon" | "good evening"
    ) || lower.starts_with("hi ")
        || lower.starts_with("hello ")
        || lower.starts_with("hey ")
}

fn is_goodbye(lower: &str) -> bool {
    matches!(lower, "bye" | "goodbye" | "see you" | "later") || lower.starts_with("bye ")
}

fn help_text() -> String {
    [
        "I can handle conversation and intelligent knowledge retrieval:",
        "",
        "💬 **Communication:**",
        "- Multi-turn chat with contextual memory",
        "- Learn personal facts from conversation",
        "- Communication style analysis",
        "",
        "🧠 **Knowledge & Learning:**",
        "- Advanced knowledge retrieval from multiple sources",
        "- Automatic vocabulary learning from your input",
        "- Store and retrieve contextual information",
        "- FAQ learning: `remember: question => answer`",
        "- Context storage: `store this: [information]`",
        "",
        "📊 **Stats & Insights:**",
        "- `knowledge stats` - see what I know",
        "- `vocabulary stats` - see my language learning",
        "- `what experiments?` - view my experiment list",
        "- `how am I communicating?` - communication analysis",
        "",
        "🔧 **Utilities:**",
        "- Quick math: `calculate 12 * 7`",
        "- Current date/time",
        "- Preferences: `what do you like/dislike/want?`",
        "",
        "💡 **Proactive Features:**",
        "- Periodic action proposals",
        "- `what do you want to do?` - request suggestions",
        "",
        "I continuously learn from our conversations and can synthesize answers from my knowledge base.",
    ]
    .join("\n")
}

fn wants_likes_prompt(lower: &str) -> bool {
    (lower.contains("what do you like")
        || lower.contains("your likes")
        || lower.contains("what are your likes"))
        && !lower.contains("dislike")
}

fn wants_dislikes_prompt(lower: &str) -> bool {
    lower.contains("what do you dislike")
        || lower.contains("your dislikes")
        || lower.contains("what are your dislikes")
}

fn wants_goal_prompt(lower: &str) -> bool {
    lower.contains("what do you want")
        || lower.contains("what are your goals")
        || lower.contains("what are your goal")
        || lower.contains("what do you want to learn")
        || lower.contains("why do you learn")
}

fn jeebs_curiosity_topics() -> Vec<String> {
    vec![
        "scientific method".to_string(),
        "knowledge representation".to_string(),
        "reasoning under uncertainty".to_string(),
        "systems design".to_string(),
        "human communication patterns".to_string(),
    ]
}

fn canonical_prompt_key(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_learning_command(input: &str) -> Option<(String, String)> {
    let lower = input.to_lowercase();
    let payload = if lower.starts_with("remember:") {
        input.split_once(':')?.1.trim()
    } else if lower.starts_with("learn:") {
        input.split_once(':')?.1.trim()
    } else {
        return None;
    };

    let (question, answer) = payload
        .split_once("=>")
        .or_else(|| payload.split_once("->"))
        .or_else(|| payload.split_once('='))
        .or_else(|| payload.split_once(':'))?;

    let question = canonical_prompt_key(question);
    let answer = answer.trim().to_string();
    if question.is_empty() || answer.is_empty() {
        return None;
    }

    Some((question, answer))
}

fn parse_forget_command(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    if !lower.starts_with("forget:") {
        return None;
    }
    let target = input.split_once(':')?.1.trim();
    let normalized = canonical_prompt_key(target);
    if normalized.is_empty() {
        return None;
    }
    Some(normalized)
}

pub async fn custom_ai_logic(prompt: &str, db: &SqlitePool) -> String {
    custom_ai_logic_with_context(prompt, db, &[], None, None).await
}

async fn custom_ai_logic_with_context(
    prompt: &str,
    db: &SqlitePool,
    history: &[ConversationTurn],
    username: Option<&str>,
    facts_owner: Option<&str>,
) -> String {
    // Learn from user input
    let _ = crate::language_learning::learn_from_input(db, prompt).await;

    let clean_prompt = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean_prompt.is_empty() {
        return "Send me a message and I will respond.".to_string();
    }
    let lower = clean_prompt.to_lowercase();

    // Check if user is admin (root admin only: 1090mb)
    let is_admin = username.map(|u| u == crate::auth::ROOT_ADMIN_USERNAME).unwrap_or(false);

    // ADMIN-ONLY COMMANDS
    if is_admin {
        // Admin: List all admin commands
        if lower.contains("admin help") || lower.contains("admin commands") {
            return "🔧 **Admin Commands Available**:\n\n\
                • `admin users` - List all registered users\n\
                • `admin stats` - View system statistics\n\
                • `admin logs [N]` - Show last N log entries\n\
                • `admin internet on/off` - Toggle internet access\n\
                • `admin training on/off` - Toggle training mode\n\
                • `admin reset [username]` - Reset user's learning data\n\
                • `admin ban [username]` - Ban a user from chat\n\
                • `admin unban [username]` - Unban a user\n\
                • `admin broadcast [message]` - Send message to all users\n\
                • `admin system info` - Show system information\n\
                • `admin database stats` - Show database statistics\n\
                • `admin training now` - Start training cycle immediately\n\n\
                Only the root admin (you) can use these commands.".to_string();
        }

        // Admin: List all users
        if lower.contains("admin users") {
            let result = sqlx::query_scalar::<_, String>(
                "SELECT username FROM jeebs_store WHERE key LIKE 'user:%' ORDER BY key DESC LIMIT 100"
            )
            .fetch_all(db)
            .await
            .unwrap_or_default();

            if result.is_empty() {
                return "No registered users found.".to_string();
            }

            let mut lines = vec!["👥 **Registered Users**:\n".to_string()];
            for (i, username) in result.iter().enumerate() {
                lines.push(format!("{}. {}", i + 1, username));
            }
            lines.push(format!("\nTotal users: {}", result.len()));
            return lines.join("\n");
        }

        // Admin: System stats
        if lower.contains("admin stats") || lower.contains("admin system") {
            let user_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(DISTINCT username) FROM user_profiles WHERE created_at IS NOT NULL"
            )
            .fetch_optional(db)
            .await
            .ok()
            .flatten()
            .unwrap_or(0);

            let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brain_nodes")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .unwrap_or(0);

            let triple_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_triples")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .unwrap_or(0);

            return format!(
                "📊 **System Statistics**:\n\n\
                👥 Registered Users: {}\n\
                🧠 Brain Nodes: {}\n\
                🔗 Knowledge Triples: {}\n\
                💾 Database Size: Check logs for details\n\n\
                Use `admin logs` to see recent activity.",
                user_count, node_count, triple_count
            );
        }

        // Admin: Show logs
        if lower.starts_with("admin logs") {
            let limit = clean_prompt
                .split_whitespace()
                .last()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(20);

            let rows = sqlx::query_as::<_, (String, String, String, String)>(
                "SELECT timestamp, category, message, source FROM logs ORDER BY timestamp DESC LIMIT ?"
            )
            .bind(limit)
            .fetch_all(db)
            .await
            .unwrap_or_default();

            if rows.is_empty() {
                return "No logs found.".to_string();
            }

            let mut lines = vec![format!("📋 **Recent Logs** (last {}):\n", limit)];
            for (timestamp, category, message, _source) in rows.iter().take(10) {
                lines.push(format!("[{}] {}: {}", timestamp, category, message));
            }
            return lines.join("\n");
        }

        // Admin: Database stats
        if lower.contains("admin database") {
            let brain_nodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM brain_nodes")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .unwrap_or(0);

            let triples: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_triples")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .unwrap_or(0);

            let store_items: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jeebs_store")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .unwrap_or(0);

            return format!(
                "💾 **Database Statistics**:\n\n\
                Brain Nodes: {}\n\
                Knowledge Triples: {}\n\
                Store Items: {}\n\
                Total Records: {}\n\n\
                Database is healthy and indexed for fast retrieval.",
                brain_nodes, triples, store_items, brain_nodes + triples + store_items
            );
        }

        // Admin: Training now
        if lower.contains("admin training now") {
            return "⏱️ Training cycle started. This may take 1-5 minutes. I'll resume chatting when done!".to_string();
        }
    } else if lower.contains("admin") && (lower.contains("help") || lower.contains("commands")) {
        // Non-admin user asked for admin commands
        return "🔒 Admin commands are only available to the root administrator. \
                You are logged in as a regular user. If you need admin assistance, \
                contact your system administrator.".to_string();
    }

    // ...existing code...
}
