// User management submodule for admin

use actix_web::{get, delete, post, web, Responder, HttpResponse};
use actix_session::Session;
use serde::{Serialize, Deserialize};
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
    let mut users = Vec::new();
    for item in db.scan_prefix("user:") {
        if let Ok((key, val)) = item {
            if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
                let username = key.strip_prefix(b"user:").map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
                let email = user_json["email"].as_str().unwrap_or("").to_string();
                let is_admin = username == "admin";
                users.push(UserInfo { username, email, is_admin });
            }
        }
    }
    HttpResponse::Ok().json(users)
}

#[delete("/api/admin/user/{username}")]
pub async fn admin_delete_user(data: web::Data<AppState>, session: Session, path: web::Path<String>) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }
    let username = path.into_inner();
    if username == "admin" {
        return HttpResponse::BadRequest().json(json!({"error": "Cannot delete admin user"}));
    }
    let db = &data.db;
    let user_key = format!("user:{}", username);
    db.remove(user_key).unwrap();
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[derive(Deserialize)]
pub struct NewPasswordRequest {
    pub username: String,
    pub new_password: String,
}

#[post("/api/admin/user/reset_password")]
pub async fn admin_reset_user_password(
    data: web::Data<AppState>,
    session: Session,
    req: web::Json<NewPasswordRequest>,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)).unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }
    let username = req.username.trim().to_lowercase();
    let user_key = format!("user:{}", username);
    let db = &data.db;
    if let Some(user_val) = db.get(&user_key).unwrap() {
        let mut user_json: serde_json::Value = serde_json::from_slice(&user_val).unwrap_or_default();
        // ...existing code for password reset...
        user_json["password"] = serde_json::Value::String(req.new_password.clone());
        db.insert(user_key, serde_json::to_vec(&user_json).unwrap()).unwrap();
        HttpResponse::Ok().json(json!({"ok": true}))
    } else {
        HttpResponse::NotFound().json(json!({"error": "User not found"}))
    }
}
