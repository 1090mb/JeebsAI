// Brain module for knowledge graph and training

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use actix_web::{post, web, Responder, HttpResponse};
use reqwest;
use scraper::{Html, Selector};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::io::{Read, Write};

use crate::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BrainNode {
    pub id: String, // blake3 hash of canonical name or URL
    pub label: String, // concept name or page title
    pub summary: String, // summary or main content
    pub sources: Vec<String>, // URLs or admin notes
    pub edges: HashSet<String>, // related node ids
    pub last_trained: String, // date string
}

pub fn brain_node_key(id: &str) -> String {
    format!("brain:node:{}", id)
}

pub fn store_brain_node(db: &sled::Db, node: &BrainNode) {
    let key = brain_node_key(&node.id);
    let val = serde_json::to_vec(node).unwrap();
    let compressed = encode_all(&val[..], 1).unwrap();
    db.insert(key, compressed).unwrap();
    db.flush().unwrap();
}

pub fn get_brain_node(db: &sled::Db, id: &str) -> Option<BrainNode> {
    let key = brain_node_key(id);
    db.get(key).ok().flatten().and_then(|v| {
        decode_all(v.as_ref()).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok())
    })
}

pub fn add_brain_edge(db: &sled::Db, from: &str, to: &str) {
    if let Some(mut node) = get_brain_node(db, from) {
        node.edges.insert(to.to_string());
        store_brain_node(db, &node);
    }
}

#[derive(Deserialize)]
pub struct TrainRequest {
    pub url: String,
}

#[post("/api/admin/train")]
pub async fn admin_train(
    data: web::Data<AppState>,
    req: web::Json<TrainRequest>,
) -> impl Responder {
    // ...existing code for training logic...
    HttpResponse::Ok().json(serde_json::json!({"ok": true}))
}

// Compression helpers
fn encode_all(input: &[u8], level: u32) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(input)?;
    encoder.finish()
}

fn decode_all(input: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(input);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}
