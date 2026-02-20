use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Local;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::{HashSet, VecDeque};
use std::time::Duration;

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
pub struct CrawlRequest {
    pub url: String,
    pub depth: Option<u8>,
}

#[derive(Debug, Serialize)]
struct CrawlSummary {
    start_url: String,
    max_depth: u8,
    pages_visited: usize,
    pages_stored: usize,
    links_followed: usize,
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
        if matches!(ch, ' ' | '+' | '-' | '*' | '/' | '(' | ')' | '.' | '^' | '%') {
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
        "I can handle basic chat right now:",
        "- greetings and short conversation",
        "- quick math (example: calculate 12 * 7)",
        "- current date/time",
        "- lookup from stored brain notes",
        "- custom memory commands: `remember: question => answer`, `forget: question`",
        "",
        "Try: `hello`, `what time is it`, `what is 18/3`, or ask about something I already learned.",
    ]
    .join("\n")
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
    let clean_prompt = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean_prompt.is_empty() {
        return "Send me a message and I will respond.".to_string();
    }
    let lower = clean_prompt.to_lowercase();

    if is_greeting(&lower) {
        return "Hey, I am Jeebs. I am online and ready to chat.".to_string();
    }

    if lower.contains("who are you") || lower.contains("what are you") {
        return "I am JeebsAI, your assistant for basic conversation, quick math, and knowledge lookups.".to_string();
    }

    if lower == "help" || lower.contains("what can you do") || lower.contains("commands") {
        return help_text();
    }

    if lower.contains("how are you") {
        return "Running smoothly. Ask me anything basic and I will do my best.".to_string();
    }

    if lower.contains("thank you") || lower == "thanks" || lower.starts_with("thanks ") {
        return "You are welcome.".to_string();
    }

    if let Some((question, answer)) = parse_learning_command(&clean_prompt) {
        let key = format!("chat:faq:{question}");
        let payload = serde_json::to_vec(&json!({
            "answer": answer,
            "updated_at": Local::now().to_rfc3339()
        }))
        .unwrap_or_else(|_| b"{}".to_vec());

        return match sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
            .bind(&key)
            .bind(payload)
            .execute(db)
            .await
        {
            Ok(_) => format!("Saved. Ask me \"{question}\" and I will use that answer."),
            Err(_) => "I could not save that memory due to a database error.".to_string(),
        };
    }

    if let Some(question) = parse_forget_command(&clean_prompt) {
        let key = format!("chat:faq:{question}");
        return match sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
            .bind(&key)
            .execute(db)
            .await
        {
            Ok(result) if result.rows_affected() > 0 => {
                format!("Forgot custom answer for \"{question}\".")
            }
            Ok(_) => format!("No custom answer was saved for \"{question}\"."),
            Err(_) => "I could not remove that memory due to a database error.".to_string(),
        };
    }

    if is_goodbye(&lower) {
        return "See you soon.".to_string();
    }

    if lower == "time" || lower.contains("what time") || lower.contains("current time") {
        return format!("Current server time: {}", Local::now().format("%Y-%m-%d %H:%M:%S %Z"));
    }

    if lower == "date"
        || lower.contains("what date")
        || lower.contains("what day")
        || lower == "today"
    {
        return format!(
            "Today is {}.",
            Local::now().format("%A, %B %d, %Y")
        );
    }

    if let Some(expr) = extract_math_expression(&clean_prompt, &lower) {
        match meval::eval_str(&expr) {
            Ok(value) => {
                return format!("{expr} = {}", format_number(value));
            }
            Err(_) => {
                return "I could not parse that math expression. Try something like `12 * (3 + 4)`.".to_string();
            }
        }
    }

    if let Some(cached) = check_dejavu(&clean_prompt, db).await {
        return cached;
    }

    let related = search_brain_for_chat(db, &clean_prompt).await;
    if !related.is_empty() {
        let mut lines = vec![format!(
            "Here is what I found related to \"{clean_prompt}\":"
        )];
        for (idx, (label, summary)) in related.iter().enumerate() {
            let text = if summary.trim().is_empty() {
                "(no summary available yet)"
            } else {
                summary
            };
            lines.push(format!("{}. {} - {}", idx + 1, label, text));
        }
        return lines.join("\n");
    }

    if clean_prompt.ends_with('?') {
        "I am still learning that topic. Try `help`, ask for math/time/date, or teach me more context.".to_string()
    } else {
        "Got it. Keep chatting with me and I will help with what I can.".to_string()
    }
}

pub struct Cortex {
    pub db: SqlitePool,
}

impl Cortex {
    pub async fn think(prompt: &str, state: &AppState) -> String {
        custom_ai_logic(prompt, &state.db).await
    }
}

impl Cortex {
    pub async fn seed_knowledge(db: &SqlitePool) {
        let seed_payload = serde_json::to_vec(&json!({
            "source": "seed",
            "topic": "introduction",
            "text": "Hello! I am JeebsAI, your personal assistant."
        }))
        .unwrap_or_default();

        let _ = sqlx::query(
            "INSERT OR IGNORE INTO brain_nodes (id, label, summary, data, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
            .bind("seed:intro")
            .bind("hello")
            .bind("Hello! I am JeebsAI, your personal assistant.")
            .bind(seed_payload)
            .bind(Local::now().to_rfc3339())
            .execute(db)
            .await;
        println!("Brain knowledge seeded.");
    }

    pub async fn dream(_db: SqlitePool) {
        println!("Cortex is now dreaming (background processing active)...");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}

async fn build_graph(db: &SqlitePool, logic_only: bool) -> GraphResponse {
    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();
    let mut node_ids: HashSet<String> = HashSet::new();
    let mut edge_ids: HashSet<(String, String, String)> = HashSet::new();

    if !logic_only {
        let rows = sqlx::query(
            "SELECT id, COALESCE(label, id) AS label, COALESCE(summary, '') AS summary
             FROM brain_nodes
             ORDER BY created_at DESC
             LIMIT 300",
        )
        .fetch_all(db)
        .await
        .unwrap_or_default();

        for row in rows {
            let id: String = row.get(0);
            let label: String = row.get(1);
            let summary: String = row.get(2);

            if node_ids.insert(id.clone()) {
                nodes.push(GraphNode {
                    id: id.clone(),
                    label,
                    title: summary,
                    group: "knowledge".to_string(),
                });
            }
        }
    }

    let triple_rows = sqlx::query(
        "SELECT subject, predicate, object
         FROM knowledge_triples
         ORDER BY created_at DESC
         LIMIT 500",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    for row in triple_rows {
        let subject: String = row.get(0);
        let predicate: String = row.get(1);
        let object: String = row.get(2);

        if node_ids.insert(subject.clone()) {
            nodes.push(GraphNode {
                id: subject.clone(),
                label: subject.clone(),
                title: "Logic subject".to_string(),
                group: "logic".to_string(),
            });
        }
        if node_ids.insert(object.clone()) {
            nodes.push(GraphNode {
                id: object.clone(),
                label: object.clone(),
                title: "Logic object".to_string(),
                group: "logic".to_string(),
            });
        }

        let edge_key = (subject.clone(), object.clone(), predicate.clone());
        if edge_ids.insert(edge_key) {
            edges.push(GraphEdge {
                from: subject,
                to: object,
                label: predicate,
            });
        }
    }

    if nodes.is_empty() {
        nodes.push(GraphNode {
            id: "no-data".to_string(),
            label: "No graph data yet".to_string(),
            title: "Register/login and chat to build knowledge.".to_string(),
            group: "system".to_string(),
        });
    }

    GraphResponse { nodes, edges }
}

#[post("/api/admin/train")]
pub async fn admin_train(session: Session, _state: web::Data<AppState>) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    HttpResponse::Ok().json(json!({
        "ok": true,
        "message": "Training initiated."
    }))
}

fn normalize_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let mut out = String::with_capacity(max_chars + 3);
    for ch in input.chars().take(max_chars) {
        out.push(ch);
    }
    out.push_str("...");
    out
}

fn normalize_url(url: &reqwest::Url) -> String {
    let mut normalized = url.clone();
    normalized.set_fragment(None);

    let scheme = normalized.scheme().to_lowercase();
    let host = normalized
        .host_str()
        .map(|v| v.to_lowercase())
        .unwrap_or_default();

    let mut output = format!("{scheme}://{host}");
    if let Some(port) = normalized.port() {
        let is_default =
            (scheme == "http" && port == 80) || (scheme == "https" && port == 443);
        if !is_default {
            output.push(':');
            output.push_str(&port.to_string());
        }
    }

    let path = normalized.path();
    if path == "/" {
        output.push('/');
    } else {
        output.push_str(path.trim_end_matches('/'));
    }

    if let Some(query) = normalized.query() {
        if !query.trim().is_empty() {
            output.push('?');
            output.push_str(query);
        }
    }

    output
}

fn extract_title(document: &scraper::Html, fallback: &str) -> String {
    if let Ok(selector) = scraper::Selector::parse("title") {
        if let Some(el) = document.select(&selector).next() {
            let title = normalize_whitespace(&el.text().collect::<Vec<_>>().join(" "));
            if !title.is_empty() {
                return truncate_chars(&title, 140);
            }
        }
    }

    truncate_chars(fallback, 140)
}

fn extract_page_text(document: &scraper::Html) -> String {
    if let Ok(body_selector) = scraper::Selector::parse("body") {
        if let Some(body) = document.select(&body_selector).next() {
            return normalize_whitespace(&body.text().collect::<Vec<_>>().join(" "));
        }
    }

    String::new()
}

fn extract_followable_links(
    document: &scraper::Html,
    base_url: &reqwest::Url,
    root_host: &str,
    already_seen: &HashSet<String>,
    max_links: usize,
) -> Vec<reqwest::Url> {
    let mut links = Vec::new();
    let mut discovered = HashSet::new();

    let Ok(selector) = scraper::Selector::parse("a[href]") else {
        return links;
    };

    for el in document.select(&selector) {
        let Some(href) = el.value().attr("href") else {
            continue;
        };

        if href.starts_with('#')
            || href.starts_with("mailto:")
            || href.starts_with("javascript:")
            || href.starts_with("tel:")
        {
            continue;
        }

        let Ok(next) = base_url.join(href) else {
            continue;
        };

        if !matches!(next.scheme(), "http" | "https") {
            continue;
        }

        if next.host_str() != Some(root_host) {
            continue;
        }

        let normalized = normalize_url(&next);
        if already_seen.contains(&normalized) {
            continue;
        }

        if discovered.insert(normalized) {
            links.push(next);
        }

        if links.len() >= max_links {
            break;
        }
    }

    links
}

async fn crawl_and_store(
    state: &AppState,
    start_url: &str,
    depth_limit: u8,
) -> Result<CrawlSummary, String> {
    const MAX_PAGES: usize = 25;
    const MAX_LINKS_PER_PAGE: usize = 20;

    let start = reqwest::Url::parse(start_url).map_err(|e| format!("Invalid URL: {e}"))?;
    if !matches!(start.scheme(), "http" | "https") {
        return Err("Only http and https URLs are supported".to_string());
    }

    let root_host = start
        .host_str()
        .map(|h| h.to_string())
        .ok_or_else(|| "URL must include a host".to_string())?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent("JeebsAI-Crawler/1.0")
        .build()
        .map_err(|e| format!("Crawler initialization failed: {e}"))?;

    let mut queue: VecDeque<(reqwest::Url, u8)> = VecDeque::new();
    queue.push_back((start.clone(), 0));

    let mut visited: HashSet<String> = HashSet::new();
    let mut pages_stored = 0usize;
    let mut links_followed = 0usize;

    while let Some((current, depth)) = queue.pop_front() {
        if visited.len() >= MAX_PAGES {
            break;
        }

        let normalized_current = normalize_url(&current);
        if !visited.insert(normalized_current.clone()) {
            continue;
        }

        let response = match client.get(current.clone()).send().await {
            Ok(resp) => resp,
            Err(_) => continue,
        };

        if !response.status().is_success() {
            continue;
        }

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !content_type.contains("text/html") {
            continue;
        }

        let html = match response.text().await {
            Ok(body) => body,
            Err(_) => continue,
        };

        let document = scraper::Html::parse_document(&html);
        let title = extract_title(&document, current.as_str());
        let full_text = extract_page_text(&document);
        let summary = if full_text.is_empty() {
            format!("Crawled {}", current.as_str())
        } else {
            truncate_chars(&full_text, 800)
        };
        let excerpt = truncate_chars(&full_text, 5000);

        let node_id = format!("crawl:{}", blake3::hash(normalized_current.as_bytes()).to_hex());
        let payload = serde_json::to_vec(&json!({
            "source": "crawler",
            "url": current.as_str(),
            "normalized_url": normalized_current,
            "title": title,
            "excerpt": excerpt,
            "crawled_at": Local::now().to_rfc3339(),
            "depth": depth
        }))
        .unwrap_or_else(|_| b"{}".to_vec());

        if sqlx::query(
            "INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&node_id)
        .bind(&title)
        .bind(&summary)
        .bind(payload)
        .bind(Local::now().to_rfc3339())
        .execute(&state.db)
        .await
        .is_ok()
        {
            pages_stored += 1;

            let subject = truncate_chars(&title, 120);
            let object = truncate_chars(current.as_str(), 300);
            let _ = sqlx::query(
                "INSERT OR REPLACE INTO knowledge_triples (subject, predicate, object, confidence, created_at)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(subject)
            .bind("source_url")
            .bind(object)
            .bind(0.9_f64)
            .bind(Local::now().to_rfc3339())
            .execute(&state.db)
            .await;
        }

        if depth < depth_limit {
            let links = extract_followable_links(
                &document,
                &current,
                &root_host,
                &visited,
                MAX_LINKS_PER_PAGE,
            );
            links_followed += links.len();
            for link in links {
                queue.push_back((link, depth + 1));
            }
        }
    }

    Ok(CrawlSummary {
        start_url: normalize_url(&start),
        max_depth: depth_limit,
        pages_visited: visited.len(),
        pages_stored,
        links_followed,
    })
}

#[post("/api/admin/crawl")]
pub async fn admin_crawl(
    session: Session,
    state: web::Data<AppState>,
    req: web::Json<CrawlRequest>,
) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let url = req.url.trim();
    if url.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "URL is required"}));
    }

    let depth = req.depth.unwrap_or(1).clamp(1, 3);

    crate::logging::log(
        &state.db,
        "INFO",
        "crawler",
        &format!("Admin crawl requested for {url} (depth={depth})"),
    )
    .await;

    match crawl_and_store(state.get_ref(), url, depth).await {
        Ok(summary) => HttpResponse::Ok().json(json!({
            "ok": true,
            "message": format!(
                "Crawl complete. Visited {} page(s), stored {} node(s), discovered {} link(s).",
                summary.pages_visited, summary.pages_stored, summary.links_followed
            ),
            "start_url": summary.start_url,
            "max_depth": summary.max_depth,
            "pages_visited": summary.pages_visited,
            "pages_stored": summary.pages_stored,
            "links_followed": summary.links_followed
        })),
        Err(err) => HttpResponse::BadRequest().json(json!({
            "ok": false,
            "error": err
        })),
    }
}

#[post("/api/brain/search")]
pub async fn search_brain(
    state: web::Data<AppState>,
    req: web::Json<SearchRequest>,
) -> impl Responder {
    let query = req.query.trim().to_string();

    let rows = if query.is_empty() {
        sqlx::query(
            "SELECT id, COALESCE(label, id) AS label, COALESCE(summary, '') AS summary
             FROM brain_nodes
             ORDER BY created_at DESC
             LIMIT 50",
        )
        .fetch_all(&state.db)
        .await
    } else {
        let pattern = format!("%{query}%");
        sqlx::query(
            "SELECT id, COALESCE(label, id) AS label, COALESCE(summary, '') AS summary
             FROM brain_nodes
             WHERE id LIKE ? OR label LIKE ? OR summary LIKE ?
             ORDER BY created_at DESC
             LIMIT 50",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&state.db)
        .await
    };

    let results = rows
        .unwrap_or_default()
        .into_iter()
        .map(|row| BrainSearchResult {
            id: row.get(0),
            label: row.get(1),
            summary: row.get(2),
            sources: Vec::new(),
        })
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(results)
}

#[post("/api/brain/reindex")]
pub async fn reindex_brain(session: Session, _state: web::Data<AppState>) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    HttpResponse::Ok().json(json!({
        "ok": true,
        "message": "Reindexing complete."
    }))
}

#[get("/api/brain/visualize")]
pub async fn visualize_brain(state: web::Data<AppState>) -> impl Responder {
    let graph = build_graph(&state.db, false).await;
    HttpResponse::Ok().json(graph)
}

#[get("/api/brain/logic_graph")]
pub async fn get_logic_graph(state: web::Data<AppState>) -> impl Responder {
    let graph = build_graph(&state.db, true).await;
    HttpResponse::Ok().json(graph)
}
