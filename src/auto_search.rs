use chrono::Local;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoSearchResults {
    pub query: String,
    pub found_results: usize,
    pub learned_facts: usize,
    pub sources_used: Vec<String>,
    pub confidence_improved: bool,
}

/// Main orchestrator for automatic web learning
pub async fn auto_search_for_query(
    db: &SqlitePool,
    client: &Client,
    query: &str,
    _context: &str,
    confidence_threshold: f64,
) -> Result<AutoSearchResults, String> {
    let mut all_results = Vec::new();
    let mut sources_used = Vec::new();
    let original_query = query.to_string();

    // Extract keywords for smarter searching
    let search_query = extract_keywords(query);

    // Try Wikipedia first
    if let Ok(wiki_results) = fetch_wikipedia_results(client, &search_query, 3).await {
        all_results.extend(wiki_results);
        sources_used.push("Wikipedia".to_string());
    }

    // Fall back to Google search
    if all_results.is_empty() {
        if let Ok(google_results) = fetch_google_results(client, &search_query, 3).await {
            all_results.extend(google_results);
            sources_used.push("Google".to_string());
        }
    }

    // Fall back to DuckDuckGo
    if all_results.is_empty() {
        if let Ok(ddg_results) = fetch_duckduckgo_results(client, &search_query, 3).await {
            all_results.extend(ddg_results);
            sources_used.push("DuckDuckGo".to_string());
        }
    }

    // Score and filter results
    let filtered_results: Vec<SearchResult> = all_results
        .iter()
        .filter(|r| {
            let quality = score_knowledge_quality(&r.snippet);
            quality > confidence_threshold
        })
        .cloned()
        .collect();

    // Extract and store knowledge
    let learned_count = extract_and_store_knowledge(db, &filtered_results, &original_query)
        .await
        .unwrap_or(0);

    Ok(AutoSearchResults {
        query: original_query,
        found_results: filtered_results.len(),
        learned_facts: learned_count,
        sources_used,
        confidence_improved: learned_count > 0,
    })
}

/// Fetch results from Wikipedia
async fn fetch_wikipedia_results(
    client: &Client,
    query: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>, String> {
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&srlimit={}",
        urlencoding::encode(query),
        max_results
    );

    let response = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err("Wikipedia search failed".to_string());
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    if let Some(search) = json
        .get("query")
        .and_then(|q| q.get("search"))
        .and_then(|s| s.as_array())
    {
        for item in search.iter().take(max_results) {
            if let (Some(title), Some(snippet)) =
                (item.get("title").and_then(|t| t.as_str()), item.get("snippet").and_then(|s| s.as_str()))
            {
                results.push(SearchResult {
                    title: title.to_string(),
                    url: format!(
                        "https://en.wikipedia.org/wiki/{}",
                        title.replace(" ", "_")
                    ),
                    snippet: snippet.to_string(),
                    source: "Wikipedia".to_string(),
                });
            }
        }
    }

    Ok(results)
}

/// Fetch results from Google Search (basic scraping fallback)
async fn fetch_google_results(
    client: &Client,
    query: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>, String> {
    let url = format!(
        "https://www.google.com/search?q={}",
        urlencoding::encode(query)
    );

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err("Google search failed".to_string());
    }

    let html = response.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&html);

    let results_selector = Selector::parse("div.g").map_err(|_| "Failed to parse selector")?;
    let title_selector = Selector::parse("h3").map_err(|_| "Failed to parse title selector")?;
    let snippet_selector = Selector::parse("span.st").map_err(|_| "Failed to parse snippet selector")?;
    let link_selector = Selector::parse("a").map_err(|_| "Failed to parse link selector")?;

    let mut results = Vec::new();
    for element in document.select(&results_selector).take(max_results) {
        let title = element
            .select(&title_selector)
            .next()
            .and_then(|el| el.text().next())
            .unwrap_or("No title")
            .to_string();

        let snippet = element
            .select(&snippet_selector)
            .next()
            .and_then(|el| el.text().next())
            .unwrap_or("No snippet")
            .to_string();

        let url = element
            .select(&link_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .unwrap_or("")
            .to_string();

        if !title.is_empty() && !snippet.is_empty() {
            results.push(SearchResult {
                title,
                url,
                snippet,
                source: "Google".to_string(),
            });
        }
    }

    Ok(results)
}

/// Fetch results from DuckDuckGo - new function
pub async fn fetch_duckduckgo_results(
    client: &Client,
    query: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>, String> {
    let url = format!(
        "https://html.duckduckgo.com/?q={}&t=h_",
        urlencoding::encode(query)
    );

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 KHTML, like Gecko) Chrome/91.0",
        )
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err("DuckDuckGo search failed".to_string());
    }

    let html = response.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&html);

    let results_selector =
        Selector::parse("div.results div.result").map_err(|_| "Failed to parse selector")?;
    let title_selector =
        Selector::parse("a.result__url").map_err(|_| "Failed to parse title selector")?;
    let snippet_selector = Selector::parse("a.result__snippet")
        .map_err(|_| "Failed to parse snippet selector")?;

    let mut results = Vec::new();
    for element in document.select(&results_selector).take(max_results) {
        let title = element
            .select(&title_selector)
            .next()
            .and_then(|el| el.text().next())
            .unwrap_or("No title")
            .to_string();

        let snippet = element
            .select(&snippet_selector)
            .next()
            .and_then(|el| el.text().next())
            .unwrap_or("No snippet")
            .to_string();

        let url = element
            .select(&Selector::parse("a.result__url").unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .unwrap_or("")
            .to_string();

        if !title.is_empty() && !snippet.is_empty() && !url.is_empty() {
            results.push(SearchResult {
                title,
                url,
                snippet,
                source: "DuckDuckGo".to_string(),
            });
        }
    }

    Ok(results)
}

/// Score knowledge quality (0.0-1.0)
pub fn score_knowledge_quality(content: &str) -> f64 {
    let mut score: f64 = 0.5;

    // Check minimum length
    if content.len() < 50 {
        return 0.0;
    }

    // Check for low-quality indicators
    let low_quality_phrases = [
        "i don't know",
        "404",
        "error",
        "not found",
        "page not found",
        "no results",
        "search failed",
        "unknown",
    ];

    for phrase in &low_quality_phrases {
        if content.to_lowercase().contains(phrase) {
            return 0.0;
        }
    }

    // Increase score for content length
    if content.len() > 100 {
        score += 0.2;
    }
    if content.len() > 200 {
        score += 0.2;
    }

    // Check for stop-word heavy content
    let stop_words = [
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "by",
    ];
    let words: Vec<&str> = content.split_whitespace().collect();
    let stop_count = words
        .iter()
        .filter(|w| stop_words.contains(&w.to_lowercase().as_str()))
        .count();

    if words.len() > 0 {
        let stop_ratio = stop_count as f64 / words.len() as f64;
        if stop_ratio > 0.5 {
            score = (score * 0.5).min(0.3);
        }
    }

    // Check for meaningful content indicators
    if content.contains("http") || content.contains("www") {
        score = (score + 0.15).min(1.0);
    }

    // Boost for technical or specific content
    let technical_indicators = ["algorithm", "data", "system", "method", "process"];
    if technical_indicators
        .iter()
        .any(|t| content.to_lowercase().contains(t))
    {
        score = (score + 0.15).min(1.0);
    }

    score.max(0.0).min(1.0)
}

/// Extract and store knowledge from search results
pub async fn extract_and_store_knowledge(
    db: &SqlitePool,
    results: &[SearchResult],
    original_query: &str,
) -> Result<usize, String> {
    let mut stored_count = 0;

    for result in results {
        let quality_score = score_knowledge_quality(&result.snippet);
        if quality_score < 0.3 {
            continue; // Skip low-quality results
        }

        let node_id = format!(
            "auto:{}",
            blake3::hash(
                format!("{}:{}", result.url, Local::now().to_rfc3339()).as_bytes()
            ).to_hex()
        );

        // Create brain node
        let payload = json!({
            "type": "auto_learned",
            "source": result.source,
            "url": result.url,
            "title": result.title,
            "snippet": result.snippet,
            "original_query": original_query,
            "quality_score": quality_score,
            "learned_at": Local::now().to_rfc3339(),
        });

        // Store in brain_nodes
        if let Err(e) = sqlx::query(
            "INSERT OR REPLACE INTO brain_nodes (id, label, summary, data, created_at)
             VALUES (?, ?, ?, ?, ?)"
        )
            .bind(&node_id)
            .bind(&result.title)
            .bind(&result.snippet)
            .bind(serde_json::to_vec(&payload).unwrap_or_default())
            .bind(Local::now().to_rfc3339())
            .execute(db)
            .await
        {
            eprintln!("Failed to store brain node: {}", e);
            continue;
        }

        // Store knowledge triple for retrieval
        let _ = sqlx::query(
            "INSERT OR REPLACE INTO knowledge_triples (subject, predicate, object, confidence, created_at)
             VALUES (?, ?, ?, ?, ?)"
        )
            .bind(original_query)
            .bind("learned_from")
            .bind(&result.title)
            .bind(quality_score)
            .bind(Local::now().to_rfc3339())
            .execute(db)
            .await;

        stored_count += 1;
    }

    Ok(stored_count)
}

/// Extract keywords from query for better searching
fn extract_keywords(query: &str) -> String {
    let stop_words = [
        "what", "is", "how", "why", "when", "where", "who", "which", "should", "can", "do", "does",
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "by", "with",
    ];

    query
        .split_whitespace()
        .filter(|word| !stop_words.contains(&word.to_lowercase().as_str()))
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}
