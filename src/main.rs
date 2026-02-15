mod admin;
mod brain;
mod auth;

use admin::user::*;
use brain::*;
use auth::*;
		let hash = Argon2::default().hash_password(req.new_password.as_bytes(), &PasswordSalt::generate()).unwrap().to_string();
		user_json["password"] = serde_json::Value::String(hash);
		db.insert(user_key, serde_json::to_vec(&user_json).unwrap()).unwrap();
		return actix_web::HttpResponse::Ok().json(serde_json::json!({"ok": true}));
	}
	actix_web::HttpResponse::BadRequest().json(serde_json::json!({"error": "User not found"}))
}
use std::process::Command;
use tokio::time::{sleep, Duration};
use serde_json::Value;

// --- GitHub Auto-Update Watcher ---
async fn github_update_watcher() {
	let repo = "1090mb/JeebsAI";
	let mut last_tag = get_current_version();
	loop {
		match get_latest_github_release(repo).await {
			Ok((tag, asset_url)) if tag != last_tag => {
				if let Err(e) = download_and_replace(&asset_url).await {
					eprintln!("[update] Download failed: {}", e);
				} else {
					println!("[update] Updated to {}. Restarting...", tag);
					restart_jeebs();
					break;
				}
			}
			_ => {}
		}
		sleep(Duration::from_secs(600)).await;
	}
}

fn get_current_version() -> String {
	env!("CARGO_PKG_VERSION").to_string()
}

async fn get_latest_github_release(repo: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
	let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
	let client = reqwest::Client::new();
	let resp = client.get(&url)
		.header("User-Agent", "jeebs-updater")
		.send().await?;
	let json: Value = resp.json().await?;
	let tag = json["tag_name"].as_str().unwrap_or("").to_string();
	let branch = json["target_commitish"].as_str().unwrap_or("");
	if branch != "main" {
		return Err("Not a main branch release".into());
	}
	let assets = json["assets"].as_array().unwrap_or(&vec![]);
	let asset_url = assets.iter()
		.find(|a| a["name"].as_str().unwrap_or("").ends_with(".tar.gz"))
		.and_then(|a| a["browser_download_url"].as_str())
		.unwrap_or("").to_string();
	Ok((tag, asset_url))
}

async fn download_and_replace(asset_url: &str) -> Result<(), Box<dyn std::error::Error>> {
	let resp = reqwest::get(asset_url).await?;
	let bytes = resp.bytes().await?;
	std::fs::write("update.tar.gz", &bytes)?;
	// Extract and replace binary (assumes tarball contains the binary)
	Command::new("tar").args(["-xzf", "update.tar.gz", "-C", "."]).status()?;
	Ok(())
}

fn restart_jeebs() {
	// This will work if Jeebs is run under a supervisor (systemd, etc.)
	// For direct exec, you can use std::process::exit(0) and let a wrapper script restart it
	std::process::exit(0);
}
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use reqwest;
use scraper::{Html, Selector};

// --- Jeebs Brain: Knowledge Graph Node ---
#[derive(Serialize, Deserialize, Clone, Debug)]
struct BrainNode {
	id: String, // blake3 hash of canonical name or URL
	label: String, // concept name or page title
	summary: String, // summary or main content
	sources: Vec<String>, // URLs or admin notes
	edges: HashSet<String>, // related node ids
	last_trained: String, // date string
}

// --- Jeebs Brain: Storage/Access ---
fn brain_node_key(id: &str) -> String {
	format!("brain:node:{}", id)
}

fn store_brain_node(db: &sled::Db, node: &BrainNode) {
	let key = brain_node_key(&node.id);
	let val = serde_json::to_vec(node).unwrap();
	let compressed = encode_all(&val[..], 1).unwrap();
	db.insert(key, compressed).unwrap();
	db.flush().unwrap();
}

fn get_brain_node(db: &sled::Db, id: &str) -> Option<BrainNode> {
	let key = brain_node_key(id);
	db.get(key).ok().flatten().and_then(|v| {
		decode_all(v.as_ref()).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok())
	})
}

fn add_brain_edge(db: &sled::Db, from: &str, to: &str) {
	if let Some(mut node) = get_brain_node(db, from) {
		node.edges.insert(to.to_string());
		store_brain_node(db, &node);
	}
}

// --- Admin Training Mode: Web Scraping & Learning ---
// ...existing code...
	// Parse HTML and extract title and main text
	let doc = Html::parse_document(&body);
	let title = doc.select(&Selector::parse("title").unwrap()).next().map(|e| e.text().collect::<String>()).unwrap_or_else(|| url.to_string());
	let mut text = String::new();
	for sel in &["article", "main", "body"] {
		if let Ok(selector) = Selector::parse(sel) {
			for el in doc.select(&selector) {
				text.push_str(&el.text().collect::<Vec<_>>().join(" "));
			}
		}
	}
	if text.is_empty() { text = body.chars().take(1000).collect(); }
	// Summarize (simple: first 400 chars)
	let summary = text.chars().take(400).collect::<String>();
	// Node id: blake3 hash of url
	let id = blake3::hash(url.as_bytes()).to_hex().to_string();
	let now = chrono::Local::now().to_rfc3339();
	let node = BrainNode {
		id: id.clone(),
		label: title,
		summary,
		sources: vec![url.to_string()],
		edges: HashSet::new(),
		last_trained: now,
	};
	store_brain_node(db, &node);
	HttpResponse::Ok().json(json!({"ok": true, "id": id, "label": node.label }))
}
use rand::{distributions::Alphanumeric, Rng};
#[derive(Deserialize)]
struct RegisterRequest {
	username: String,
	password: String,
	email: String,
}

#[derive(Deserialize)]
struct ResetRequest {
	username: String,
	email: String,
}

#[derive(Deserialize)]
struct NewPasswordRequest {
	username: String,
	token: String,
	new_password: String,
}

#[post("/api/request_reset")]
async fn request_reset(
	data: web::Data<AppState>,
	req: web::Json<ResetRequest>,
) -> impl Responder {
	let db = &data.db;
	let username = req.username.trim().to_lowercase();
	let user_key = format!("user:{}", username);
	if let Some(stored) = db.get(&user_key).unwrap() {
		let user_json: serde_json::Value = serde_json::from_slice(&stored).unwrap_or_default();
		if user_json["email"] == req.email {
			let token: String = rand::thread_rng().sample_iter(&Alphanumeric).take(32).map(char::from).collect();
			let token_key = format!("reset:{}", username);
			db.insert(token_key, token.as_bytes()).unwrap();
			db.flush().unwrap();
			// Simulate sending email
			println!("[EMAIL] To: {} | Reset token: {}", req.email, token);
			return HttpResponse::Ok().json(json!({"ok": true, "message": "Reset token sent (see server log)"}));
		}
	}
	HttpResponse::BadRequest().json(json!({"error": "User/email not found"}))
}

#[post("/api/reset_password")]
async fn reset_password(
	data: web::Data<AppState>,
	req: web::Json<NewPasswordRequest>,
) -> impl Responder {
	let db = &data.db;
	let username = req.username.trim().to_lowercase();
	let token_key = format!("reset:{}", username);
	if let Some(stored_token) = db.get(&token_key).unwrap() {
		let stored_token = String::from_utf8(stored_token.to_vec()).unwrap();
		if stored_token == req.token {
			let user_key = format!("user:{}", username);
			if let Some(user_val) = db.get(&user_key).unwrap() {
				let mut user_json: serde_json::Value = serde_json::from_slice(&user_val).unwrap_or_default();
				let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
				let hash = Argon2::default().hash_password(req.new_password.as_bytes(), &salt).unwrap().to_string();
				user_json["password"] = serde_json::Value::String(hash);
				db.insert(user_key, serde_json::to_vec(&user_json).unwrap()).unwrap();
				db.remove(token_key).unwrap();
				db.flush().unwrap();
				return HttpResponse::Ok().json(json!({"ok": true, "message": "Password reset successful"}));
			}
		}
	}
	HttpResponse::BadRequest().json(json!({"error": "Invalid token or user"}))
}
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
#[derive(Deserialize)]
struct RegisterRequest {
	username: String,
	password: String,
}

#[post("/api/register")]
async fn register(
	data: web::Data<AppState>,
	req: web::Json<RegisterRequest>,
) -> impl Responder {
	let db = &data.db;
	let username = req.username.trim().to_lowercase();
	let user_key = format!("user:{}", username);
	if db.get(&user_key).unwrap().is_some() {
		return HttpResponse::BadRequest().json(json!({"error": "User already exists"}));
	}
	let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
	let hash = Argon2::default().hash_password(req.password.as_bytes(), &salt).unwrap().to_string();
	let user_json = serde_json::json!({
		"username": username,
		"password": hash,
		"email": req.email,
		"verified": false
	});
	db.insert(user_key, serde_json::to_vec(&user_json).unwrap()).unwrap();
	db.flush().unwrap();
	// Simulate sending verification email
	let verify_token: String = rand::thread_rng().sample_iter(&Alphanumeric).take(32).map(char::from).collect();
	let verify_key = format!("verify:{}", username);
	db.insert(verify_key, verify_token.as_bytes()).unwrap();
	db.flush().unwrap();
	println!("[EMAIL] To: {} | Verification token: {}", req.email, verify_token);
	HttpResponse::Ok().json(json!({"ok": true, "message": "Verification email sent (see server log)"}))
}

#[post("/api/verify_email")]
async fn verify_email(
	data: web::Data<AppState>,
	req: web::Json<NewPasswordRequest>, // reuse struct: username, token
) -> impl Responder {
	let db = &data.db;
	let username = req.username.trim().to_lowercase();
	let verify_key = format!("verify:{}", username);
	if let Some(stored_token) = db.get(&verify_key).unwrap() {
		let stored_token = String::from_utf8(stored_token.to_vec()).unwrap();
		if stored_token == req.token {
			let user_key = format!("user:{}", username);
			if let Some(user_val) = db.get(&user_key).unwrap() {
				let mut user_json: serde_json::Value = serde_json::from_slice(&user_val).unwrap_or_default();
				user_json["verified"] = serde_json::Value::Bool(true);
				db.insert(user_key, serde_json::to_vec(&user_json).unwrap()).unwrap();
				db.remove(verify_key).unwrap();
				db.flush().unwrap();
				return HttpResponse::Ok().json(json!({"ok": true, "message": "Email verified"}));
			}
		}
	}
	HttpResponse::BadRequest().json(json!({"error": "Invalid token or user"}))
}


use sled::Db;
use std::io::{self, Write};
use zstd::stream::{encode_all, decode_all};
use actix_web::{web, App, HttpServer, Responder, HttpResponse, post, get, HttpRequest, Error};
use serde_json::json;
#[derive(Deserialize)]
struct LoginRequest {
	username: String,
	password: String,
}

#[post("/api/login")]
async fn login(
	data: web::Data<AppState>,
	session: Session,
	req: web::Json<LoginRequest>,
) -> impl Responder {
	let db = &data.db;
	let username = req.username.trim().to_lowercase();
	let user_key = format!("user:{}", username);
	if let Some(stored) = db.get(&user_key).unwrap() {
		let stored_hash = String::from_utf8(stored.to_vec()).unwrap();
		let parsed_hash = PasswordHash::new(&stored_hash).unwrap();
		if Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_ok() {
			let is_admin = username == "admin";
			session.insert("logged_in", true).unwrap();
			session.insert("username", &username).unwrap();
			session.insert("is_admin", is_admin).unwrap();
			return HttpResponse::Ok().json(json!({"ok": true, "is_admin": is_admin, "username": username }));
		}
	}
	HttpResponse::Unauthorized().json(json!({"error": "Invalid username or password"}))
}

#[post("/api/logout")]
async fn logout(session: Session) -> impl Responder {
	session.purge();
	HttpResponse::Ok().json(json!({"ok": true}))
}
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::middleware::Logger;
use actix_files::Files;
use serde::{Deserialize, Serialize};

fn main() {
	// Open sled database for unique storage
	let db: Db = sled::open("jeebs_db").expect("open sled db");
	println!("Welcome to Jeebs AI CLI!");

	loop {
		print!("\nEnter a prompt (or 'exit'): ");
		io::stdout().flush().unwrap();
		let mut input = String::new();
		io::stdin().read_line(&mut input).unwrap();
		let prompt = input.trim();
		if prompt == "exit" { break; }

		// Try to recall previous response
		let memory = retrieve_response(&db, prompt);
		let response = if let Some(mem) = memory {
			format!("[Recall] {}", mem)
		} else {
			custom_ai_logic(prompt, &db)
		};

		// Store prompt and response in a unique, compressed way: key = hash(prompt), value = compressed response
			jeebs_think(prompt, &db)
		let compressed = encode_all(response.as_bytes(), 1).expect("compress");
		db.insert(key, compressed).unwrap();
		db.flush().unwrap();

		println!("AI: {}", response);
	}
	println!("Goodbye!");
}

// Custom AI logic: keyword-based, memory, and pattern matching
fn custom_ai_logic(prompt: &str, db: &sled::Db) -> String {
		let prompt_lower = prompt.to_lowercase();
		if prompt_lower.contains("what are you thinking") {
// Retrieve a stored response for a prompt (if any), decompressing on demand
fn jeebs_think(prompt: &str, db: &sled::Db) -> String {
	use chrono::Local;
	let prompt_lower = prompt.to_lowercase();
	// VPS-friendly: always flush output, avoid color codes, keep responses concise
	// Contextual memory: recall last user input (if stored)
	if prompt_lower == "what did i just say" {
		if let Ok(Some(val)) = db.get(b"last_prompt") {
			if let Ok(decompressed) = decode_all(val.as_ref()) {
				if let Ok(text) = String::from_utf8(decompressed) {
					return format!("You just said: '{}'.", text);
				}
			}
		}
		return "I don't have any previous input from you yet.".to_string();
	}
	// Reasoning: simple logic puzzles
	if prompt_lower.contains("if a > b and b > c") {
		return "Then a > c by transitivity.".to_string();
	}
	if prompt_lower.contains("if x is even") && prompt_lower.contains("x + 1") {
		return "If x is even, x + 1 is odd.".to_string();
	}
	// Pattern: count words
	if prompt_lower.starts_with("count words in ") {
		let text = prompt[15..].trim();
		let count = text.split_whitespace().count();
		return format!("There are {} words.", count);
	}
	// Pattern: reverse text
	if prompt_lower.starts_with("reverse ") {
		let text = prompt[8..].trim();
		return text.chars().rev().collect::<String>();
	}
	// Pattern: repeat after me
	if prompt_lower.starts_with("repeat after me: ") {
		let text = prompt[17..].trim();
		return format!("{}", text);
	}
	// Pattern: summarize
	if prompt_lower.starts_with("summarize ") {
		let text = prompt[10..].trim();
		let words: Vec<&str> = text.split_whitespace().collect();
		if words.len() > 8 {
			return format!("Summary: {} ... {}", words[..4].join(" "), words[words.len()-4..].join(" "));
		} else {
			return format!("Summary: {}", text);
		}
	}
	// Greetings
	if prompt_lower.contains("hello") || prompt_lower.contains("hi") {
		return "Hello! I'm Jeebs. How can I help you today?".to_string();
	}
	if prompt_lower.contains("how are you") {
		return "I'm Jeebs, just code, but I'm running smoothly!".to_string();
	}
	if prompt_lower.contains("what are you thinking") {
		return "I'm thinking about how to help you next!".to_string();
	}

	#[derive(Deserialize)]
	struct JeebsRequest {
		prompt: String,
	}

	#[derive(Serialize)]
	struct JeebsResponse {
		response: String,
	}

	#[post("/api/jeebs")]
	async fn jeebs_api(
		data: web::Data<AppState>,
		req: web::Json<JeebsRequest>,
		session: Session,
		http_req: HttpRequest,
	) -> Result<HttpResponse, Error> {
		// Require login
		let logged_in = session.get::<bool>("logged_in").unwrap_or(Some(false)).unwrap_or(false);
		let username = session.get::<String>("username").unwrap_or(None);
		if !logged_in {
			return Ok(HttpResponse::Unauthorized().json(json!({"error": "Not logged in"})));
		}
		let db = &data.db;
		let prompt = req.prompt.trim();
		let user_id = if let Some(uid) = session.get::<String>("user_id").unwrap_or(None) {
			uid
		} else {
			let new_id = uuid::Uuid::new_v4().to_string();
			session.insert("user_id", &new_id).unwrap();
			new_id
		};
		println!("[API] user_id={} username={:?} ip={:?} prompt=\"{}\"", user_id, username, http_req.peer_addr(), prompt);
		let memory = retrieve_response(db, prompt);
		let response = if let Some(mem) = memory {
			format!("[Recall] {}", mem)
		} else {
			jeebs_think(prompt, db)
		};
		// Store prompt/response
		let key = blake3::hash(prompt.as_bytes()).as_bytes().to_vec();
		let compressed = encode_all(response.as_bytes(), 1).expect("compress");
		db.insert(key, compressed).unwrap();
		db.flush().unwrap();
		Ok(HttpResponse::Ok().json(JeebsResponse { response }))
	}

	struct AppState {
		db: Db,
	}

	#[actix_web::main]
	async fn main() -> std::io::Result<()> {
		// Open sled database for unique storage
		let db: Db = sled::open("jeebs_db").expect("open sled db");

		// Start GitHub update watcher in background
		tokio::spawn(github_update_watcher());

		// Start web server in background
		let db_web = db.clone();
		let secret_key = Key::generate();
		let web_server = HttpServer::new(move || {
			App::new()
				.wrap(Logger::default())
				.wrap(SessionMiddleware::new(
					CookieSessionStore::default(),
					secret_key.clone(),
				))
				.app_data(web::Data::new(AppState { db: db_web.clone() }))
				.service(auth::register)
				.service(auth::reset)
				.service(login)
				.service(logout)
				.service(request_reset)
				.service(reset_password)
				.service(verify_email)
				.service(jeebs_api)
				.service(brain::admin_train)
				.service(admin::admin_list_users)
				.service(admin::admin_delete_user)
				.service(admin::admin_reset_user_password)
				.service(Files::new("/", "webui").index_file("index.html"))
		})
		.bind(("0.0.0.0", 8080))?
		.run();

		// CLI loop (optional, can be removed if only web is needed)
		println!("Jeebs is running! Web API at http://localhost:8080/api/jeebs");
		tokio::spawn(async move {
			let stdin = std::io::stdin();
			let mut input = String::new();
			loop {
				print!("\nEnter a prompt (or 'exit'): ");
				std::io::stdout().flush().unwrap();
				input.clear();
				stdin.read_line(&mut input).unwrap();
				let prompt = input.trim();
				if prompt == "exit" { break; }
				let memory = retrieve_response(&db, prompt);
				let response = if let Some(mem) = memory {
					format!("[Recall] {}", mem)
				} else {
					jeebs_think(prompt, &db)
				};
				let key = blake3::hash(prompt.as_bytes()).as_bytes().to_vec();
				let compressed = encode_all(response.as_bytes(), 1).expect("compress");
				db.insert(key, compressed).unwrap();
				db.flush().unwrap();
				println!("Jeebs: {}", response);
			}
			println!("Goodbye from Jeebs!");
		});

		web_server.await
	}
				"Bananas are berries, but strawberries aren't.",
				"Octopuses have three hearts.",
				"A group of flamingos is called a 'flamboyance'.",
				"The Eiffel Tower can be 15 cm taller during hot days."
			];
			let idx = Local::now().timestamp() as usize % facts.len();
			return facts[idx].to_string();
		}
		_ => {}
	}
	// Basic conversation memory: store and recall name
	if prompt_lower.starts_with("my name is ") {
		let name = prompt.trim_start_matches(|c: char| !c.is_alphabetic()).trim_start_matches("my name is ").trim();
		let _ = db.insert(b"user_name", encode_all(name.as_bytes(), 1).unwrap());
		let _ = db.flush();
		return format!("Nice to meet you, {}! I'm Jeebs.", name);
	}
	if prompt_lower.contains("what's my name") || prompt_lower.contains("what is my name") {
		if let Some(val) = db.get(b"user_name").ok().flatten() {
			if let Ok(decompressed) = decode_all(val.as_ref()) {
				if let Ok(name) = String::from_utf8(decompressed) {
					return format!("Your name is {}!", name);
				}
			}
		}
		return "I don't know your name yet. Tell me with 'my name is ...'".to_string();
	}
	// VPS: store last prompt for context
	let _ = db.insert(b"last_prompt", encode_all(prompt.as_bytes(), 1).unwrap());
	let _ = db.flush();
	// Fallback: context-aware
	if let Ok(Some(val)) = db.get(b"last_prompt") {
		if let Ok(decompressed) = decode_all(val.as_ref()) {
			if let Ok(last_prompt) = String::from_utf8(decompressed) {
				return format!("I'm not sure how to respond to '{}'. Last thing you said was '{}'. Try: hello, how are you, date, calc 2+2, who are you, what is rust, tell me a joke, random fact, my name is ...", prompt, last_prompt);
			}
		}
	}
	format!("I'm not sure how to respond to: '{}'. Try: hello, how are you, date, calc 2+2, who are you, what is rust, tell me a joke, random fact, my name is ...", prompt)
fn retrieve_response(db: &sled::Db, prompt: &str) -> Option<String> {
	let key = blake3::hash(prompt.as_bytes()).as_bytes().to_vec();
	if let Some(val) = db.get(key).ok().flatten() {
		if let Ok(decompressed) = decode_all(val.as_ref()) {
			if let Ok(text) = String::from_utf8(decompressed) {
				return Some(text);
			}
		}
	}
	None
}
