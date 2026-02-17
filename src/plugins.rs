use std::pin::Pin;
use std::future::Future;
use sqlx::{SqlitePool, Row};
use crate::utils::{encode_all, decode_all};
use chrono::Local;
use reqwest::Client;
use meval;
use sysinfo::System;
use base64::Engine;
use serde_json::json;
use base64 as b64;
use blake3;
use sha2::{Sha256, Digest};
use md5;
use hex;
use crate::security::generate_password;

// Plugin trait used across the app (object-safe)
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn handle(&self, prompt: String, db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>>;
}

// Time plugin — returns current time
pub struct TimePlugin;
impl Plugin for TimePlugin {
    fn name(&self) -> &str { "Time" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.to_lowercase();
            if p.contains("time") || p.contains("what time") {
                Some(Local::now().to_rfc3339())
            } else { None }
        })
    }
}

// Calc plugin — evaluates simple math expressions using `meval`.
pub struct CalcPlugin;
impl Plugin for CalcPlugin {
    fn name(&self) -> &str { "Calc" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.trim();
            let lower = p.to_lowercase();
            let expr = if lower.starts_with("calc ") { Some(p[5..].trim()) }
                else if lower.starts_with("calculate ") { Some(p[10..].trim()) }
                else if p.chars().all(|c| c.is_ascii() && (c.is_whitespace() || "0123456789.+-*/()%".contains(c))) { Some(p) }
                else { None };

            if let Some(e) = expr {
                match meval::eval_str(e) {
                    Ok(v) => Some(format!("{}", v)),
                    Err(err) => Some(format!("Calculation error: {}", err)),
                }
            } else { None }
        })
    }
}

// Weather plugin — uses wttr.in for a lightweight no-key weather lookup.
pub struct WeatherPlugin;
impl Plugin for WeatherPlugin {
    fn name(&self) -> &str { "Weather" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("weather") {
                let parts: Vec<_> = prompt.splitn(2, ' ').collect();
                let location = parts.get(1).map(|s| s.trim()).filter(|s| !s.is_empty()).unwrap_or("_");
                let client = Client::new();
                let url = format!("https://wttr.in/{}?format=3", urlencoding::encode(location));
                if let Ok(resp) = client.get(&url).send().await {
                    if let Ok(text) = resp.text().await { return Some(text); }
                }
                return Some("Failed to fetch weather.".to_string());
            }
            None
        })
    }
}

// NewsPlugin — lightweight top-stories fetch (keeps interface used by main)
pub struct NewsPlugin;
impl NewsPlugin { pub fn new() -> Self { Self } }
impl Plugin for NewsPlugin {
    fn name(&self) -> &str { "News" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.to_lowercase();
            if p.contains("news") || p.contains("headlines") {
                // Use Hacker News public API (no key)
                let client = Client::new();
                if let Ok(resp) = client.get("https://hacker-news.firebaseio.com/v0/topstories.json").send().await {
                    if let Ok(ids) = resp.json::<Vec<u64>>().await {
                        let mut headlines = Vec::new();
                        for id in ids.into_iter().take(5) {
                            let item_url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
                            if let Ok(item_resp) = client.get(&item_url).send().await {
                                if let Ok(item_json) = item_resp.json::<serde_json::Value>().await {
                                    if let Some(title) = item_json["title"].as_str() {
                                        let url = item_json["url"].as_str().unwrap_or("");
                                        headlines.push(format!("- {} ({})", title, url));
                                    }
                                }
                            }
                        }
                        if !headlines.is_empty() { return Some(format!("Latest headlines:\n{}", headlines.join("\n"))); }
                    }
                }
                return Some("Failed to fetch news.".to_string());
            }
            None
        })
    }
}

// Memory plugin — simple key/value memory stored in `jeebs_store`.
pub struct MemoryPlugin;
impl Plugin for MemoryPlugin {
    fn name(&self) -> &str { "Memory" }
    fn handle(&self, prompt: String, db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("remember ") {
                let rest = prompt[8..].trim();
                if let Some((k, v)) = rest.split_once('=') {
                    let key = format!("mem:{}", k.trim());
                    let payload = json!({ "value": v.trim(), "created_at": Local::now().to_rfc3339() });
                    if let Ok(bytes) = serde_json::to_vec(&payload) {
                        if let Ok(enc) = encode_all(&bytes, 1) {
                            let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(enc).execute(&db).await;
                            return Some(format!("Remembered {}={}", k.trim(), v.trim()));
                        }
                    }
                    return Some("Failed to store memory.".to_string());
                }
                return Some("Usage: remember KEY=VALUE".to_string());
            } else if lower.starts_with("recall ") {
                let key = prompt[7..].trim();
                let full = format!("mem:{}", key);
                if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&full).fetch_optional(&db).await {
                    let val: Vec<u8> = row.get(0);
                    if let Ok(bytes) = decode_all(&val) {
                        if let Ok(vj) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                            return Some(vj["value"].as_str().unwrap_or("").to_string());
                        }
                    }
                }
                return Some(format!("No memory found for '{}'.", key));
            } else if lower.starts_with("memories") || lower.starts_with("list memories") {
                if let Ok(rows) = sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE 'mem:%'").fetch_all(&db).await {
                    let items: Vec<String> = rows.into_iter().filter_map(|r| {
                        let k: String = r.get(0);
                        let val: Vec<u8> = r.get(1);
                        decode_all(&val).ok().and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok()).and_then(|vj| vj["value"].as_str().map(|s| format!("{} -> {}", k.strip_prefix("mem:").unwrap_or(&k), s.to_string())))
                    }).collect();
                    return Some(format!("Memories:\n{}", items.join("\n")));
                }
                return Some("No memories found.".to_string());
            }
            None
        })
    }
}

// System plugin — returns basic system stats
pub struct SystemPlugin;
impl Plugin for SystemPlugin {
    fn name(&self) -> &str { "System" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.to_lowercase();
            if p.contains("system") || p.contains("cpu") || p.contains("ram") || p.contains("memory") {
                let mut sys = System::new_all();
                sys.refresh_all();
                let total = sys.total_memory();
                let free = sys.available_memory();
                let used = total.saturating_sub(free);
                return Some(format!("CPU cores: {} | Memory used: {} / {} KB", sys.cpus().len(), used, total));
            }
            None
        })
    }
}

// Summary plugin — trivial summarizer (first 2 sentences / truncated)
pub struct SummaryPlugin;
impl Plugin for SummaryPlugin {
    fn name(&self) -> &str { "Summary" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("summarize ") {
                let text = prompt[10..].trim();
                let sentences: Vec<_> = text.split('.').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                if sentences.is_empty() { return Some("Nothing to summarize.".to_string()); }
                let summary = sentences.into_iter().take(2).collect::<Vec<_>>().join(". ");
                return Some(summary.chars().take(400).collect::<String>());
            }
            None
        })
    }
}

// Translate plugin — supports simple 'uppercase'/'lowercase' and pig-latin demo
fn pig_latin_word(w: &str) -> String {
    let w = w.trim();
    if w.is_empty() { return "".to_string(); }
    let first = w.chars().next().unwrap();
    if "aeiouAEIOU".contains(first) { format!("{}-ay", w) } else { format!("{}-{}ay", &w[1..], first) }
}
pub struct TranslatePlugin;
impl Plugin for TranslatePlugin {
    fn name(&self) -> &str { "Translate" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("translate to ") {
                if let Some((lang, rest)) = prompt[13..].split_once(':') {
                    let lang = lang.trim().to_lowercase();
                    let text = rest.trim();
                    match lang.as_str() {
                        "uppercase" => return Some(text.to_uppercase()),
                        "lowercase" => return Some(text.to_lowercase()),
                        "pig" => return Some(text.split_whitespace().map(pig_latin_word).collect::<Vec<_>>().join(" ")),
                        _ => return Some("Translation for that language is not supported in this build.".to_string()),
                    }
                }
                return Some("Usage: translate to <lang>: <text> (supported: uppercase, lowercase, pig)".to_string());
            }
            None
        })
    }
}

// Password generator plugin
pub struct PasswordPlugin;
impl Plugin for PasswordPlugin {
    fn name(&self) -> &str { "Password" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("password") || lower.starts_with("gen password") || lower.starts_with("generate password") {
                let parts: Vec<_> = prompt.split_whitespace().collect();
                let len = parts.iter().find_map(|p| p.parse::<usize>().ok()).unwrap_or(16);
                return Some(generate_password(len));
            }
            None
        })
    }
}

// Hash plugin — supports blake3 (default), sha256 and md5
pub struct HashPlugin;
impl Plugin for HashPlugin {
    fn name(&self) -> &str { "Hash" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("hash ") {
                let rest = prompt[5..].trim();
                if rest.starts_with("sha256 ") {
                    let txt = &rest[7..];
                    let mut hasher = Sha256::new();
                    hasher.update(txt.as_bytes());
                    return Some(hex::encode(hasher.finalize()));
                } else if rest.starts_with("md5 ") {
                    let txt = &rest[4..];
                    // compute MD5 via md5::Md5
                    let mut hasher = md5::Md5::new();
                    hasher.update(txt.as_bytes());
                    let digest = hasher.finalize();
                    return Some(hex::encode(digest));
                } else {
                    return Some(blake3::hash(rest.as_bytes()).to_hex().to_string());
                }
            }
            None
        })
    }
}

// Base64 plugin
pub struct Base64Plugin;
impl Plugin for Base64Plugin {
    fn name(&self) -> &str { "Base64" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("b64 encode ") {
                let txt = prompt[11..].trim();
                return Some(b64::engine::general_purpose::STANDARD.encode(txt.as_bytes()));
            } else if lower.starts_with("b64 decode ") {
                let txt = prompt[11..].trim();
                if let Ok(bytes) = b64::engine::general_purpose::STANDARD.decode(txt) { return Some(String::from_utf8_lossy(&bytes).to_string()); }
                return Some("Invalid base64 input".to_string());
            }
            None
        })
    }
}

// Simple Logic plugin — evaluates trivial "true and false" style prompts
pub struct LogicPlugin;
impl Plugin for LogicPlugin {
    fn name(&self) -> &str { "Logic" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let p = prompt.to_lowercase();
            if p.contains(" and ") || p.contains(" or ") || p.contains("not ") {
                let tokens: Vec<_> = p.split_whitespace().collect();
                // very naive eval: treat "true"/"false" and boolean ops left-to-right
                let mut acc: Option<bool> = None;
                let mut op: Option<&str> = None;
                for t in tokens {
                    match t {
                        "true" => { if let Some(a) = acc { acc = Some(match op { Some("and") => a && true, Some("or") => a || true, _ => true }) } else { acc = Some(true); } }
                        "false" => { if let Some(a) = acc { acc = Some(match op { Some("and") => a && false, Some("or") => a || false, _ => false }) } else { acc = Some(false); } }
                        "and" | "or" => op = Some(t),
                        "not" => { op = Some("not"); if let Some(a) = acc { acc = Some(!a); } }
                        _ => {}
                    }
                }
                if let Some(v) = acc { return Some(format!("{}", v)); }
            }
            None
        })
    }
}

// Contact plugin — extracts simple email/phone patterns
pub struct ContactPlugin;
impl Plugin for ContactPlugin {
    fn name(&self) -> &str { "Contact" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let re = regex::Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap();
            if let Some(m) = re.find(&prompt) { return Some(format!("Found email: {}", m.as_str())); }
            None
        })
    }
}

// Website status plugin — quick HTTP check
pub struct WebsiteStatusPlugin;
impl Plugin for WebsiteStatusPlugin {
    fn name(&self) -> &str { "WebsiteStatus" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            if lower.starts_with("status ") || lower.starts_with("check ") {
                let parts: Vec<_> = prompt.split_whitespace().collect();
                if let Some(url) = parts.get(1) {
                    let client = Client::new();
                    if let Ok(r) = client.get(*url).send().await {
                        return Some(format!("{} -> {}", url, r.status()));
                    }
                    return Some(format!("Failed to reach {}", url));
                }
            }
            None
        })
    }
}

// Todo plugin — simple list stored in jeebs_store under key `todo:list`
pub struct TodoPlugin;
impl Plugin for TodoPlugin {
    fn name(&self) -> &str { "Todo" }
    fn handle(&self, prompt: String, db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            let lower = prompt.to_lowercase();
            let key = "todo:list".to_string();
            if lower.starts_with("todo add ") {
                let item = prompt[9..].trim();
                let mut items: Vec<String> = Vec::new();
                if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&db).await {
                    let val: Vec<u8> = row.get(0);
                    if let Ok(bytes) = decode_all(&val) {
                        if let Ok(existing) = serde_json::from_slice::<Vec<String>>(&bytes) { items = existing; }
                    }
                }
                items.push(item.to_string());
                if let Ok(bytes) = serde_json::to_vec(&items) {
                    if let Ok(enc) = encode_all(&bytes, 1) {
                        let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(enc).execute(&db).await;
                        return Some("Todo added.".to_string());
                    }
                }
                return Some("Failed to add todo.".to_string());
            } else if lower.starts_with("todo list") {
                if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&db).await {
                    let val: Vec<u8> = row.get(0);
                    if let Ok(bytes) = decode_all(&val) {
                        if let Ok(items) = serde_json::from_slice::<Vec<String>>(&bytes) {
                            let out = items.into_iter().enumerate().map(|(i, s)| format!("{}: {}", i + 1, s)).collect::<Vec<_>>().join("\n");
                            return Some(out);
                        }
                    }
                }
                return Some("No todos.".to_string());
            } else if lower.starts_with("todo remove ") {
                if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?").bind(&key).fetch_optional(&db).await {
                    let val: Vec<u8> = row.get(0);
                    if let Ok(bytes) = decode_all(&val) {
                        if let Ok(mut items) = serde_json::from_slice::<Vec<String>>(&bytes) {
                            if let Ok(idx) = prompt[12..].trim().parse::<usize>() {
                                if idx > 0 && idx <= items.len() {
                                    items.remove(idx-1);
                                    if let Ok(bytes2) = serde_json::to_vec(&items) {
                                        if let Ok(enc) = encode_all(&bytes2, 1) {
                                            let _ = sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)").bind(&key).bind(enc).execute(&db).await;
                                            return Some("Removed.".to_string());
                                        }
                                    }
                                }
                            }
                            return Some("Invalid index".to_string());
                        }
                    }
                }
                return Some("No todos to remove.".to_string());
            }
            None
        })
    }
}

// Error plugin (small test helper)
pub struct ErrorPlugin;
impl Plugin for ErrorPlugin {
    fn name(&self) -> &str { "ErrorTest" }
    fn handle(&self, prompt: String, _db: SqlitePool) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
        Box::pin(async move {
            if prompt.to_lowercase().contains("trigger error") { Some("Error: simulated failure".to_string()) } else { None }
        })
    }
}

// Dynamic plugin loader (no-op; runtime plugins may be added to `/plugins` directory)
pub fn load_dynamic_plugins(_dir: &str) -> Vec<Box<dyn Plugin>> { Vec::new() }