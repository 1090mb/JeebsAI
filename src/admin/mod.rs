// Admin module for user management

use actix_web::{get, post, delete, web, Responder, HttpResponse};
use actix_session::Session;
use serde::Serialize;
use serde_json::json;

use crate::AppState;

#[derive(Serialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub is_admin: bool,
}

#[get("/api/admin/users")]
pub async fn admin_list_users(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }
    let db = &data.db;
    // Further modularized: user management is now in the user submodule
    pub mod user;
    pub use user::*;
            if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
