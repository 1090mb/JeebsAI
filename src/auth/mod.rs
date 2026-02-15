// Auth module for registration and password reset

use actix_web::{post, web, Responder, HttpResponse};
use serde::Deserialize;
use rand::{distributions::Alphanumeric, Rng};

use crate::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetRequest {
    pub username: String,
    pub email: String,
}

// ...existing code for registration and reset handlers...
// Placeholder for actual logic

#[post("/api/register")]
pub async fn register(
    data: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    // Registration logic here
    HttpResponse::Ok().json(serde_json::json!({"ok": true}))
}

#[post("/api/reset")]
pub async fn reset(
    data: web::Data<AppState>,
    req: web::Json<ResetRequest>,
) -> impl Responder {
    // Reset logic here
    HttpResponse::Ok().json(serde_json::json!({"ok": true}))
}
