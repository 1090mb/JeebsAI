pub mod admin;
pub mod auth;
pub mod chat;
pub mod cli;
pub mod content_extractor;
pub mod cortex;
pub mod data_synthesis;
pub mod evolution;
pub mod knowledge_retrieval;
pub mod language_learning;
pub mod logging;
pub mod plugins;
pub mod proposals;
pub mod question_learning;
pub mod security;
pub mod server;
pub mod state;
pub mod toggle_manager;
pub mod updater;
pub mod user_chat;
pub mod utils;

// Re-export AppState for convenience
pub use crate::state::AppState;
