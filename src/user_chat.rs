use actix_session::Session;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::state::AppState;
use crate::logging;

#[derive(Deserialize)]
pub struct UserChatRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct UserChatResponse {
    pub response: String,
    pub username: String,
    pub is_admin: bool,
}

/// Check if user is authenticated (logged in)
fn is_user_authenticated(session: &Session) -> bool {
    session
        .get::<bool>("logged_in")
        .ok()
        .flatten()
        .unwrap_or(false)
}

/// Check if user is root admin
fn is_root_admin(session: &Session) -> bool {
    let logged_in = is_user_authenticated(session);
    if !logged_in {
        return false;
    }

    let is_admin = session
        .get::<bool>("is_admin")
        .ok()
        .flatten()
        .unwrap_or(false);
    if !is_admin {
        return false;
    }

    session
        .get::<String>("username")
        .ok()
        .flatten()
        .map(|u| u == crate::auth::ROOT_ADMIN_USERNAME)
        .unwrap_or(false)
}

/// Get username from session
fn get_username(session: &Session) -> Option<String> {
    session.get::<String>("username").ok().flatten()
}

/// Get peer IP address
fn peer_addr(http_req: &HttpRequest) -> String {
    http_req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// User-friendly chat endpoint (requires authentication)
#[post("/api/chat")]
pub async fn user_chat(
    data: web::Data<AppState>,
    req: web::Json<UserChatRequest>,
    session: Session,
    http_req: HttpRequest,
) -> impl Responder {
    // Verify user is authenticated
    if !is_user_authenticated(&session) {
        logging::log(
            &data.db,
            "WARN",
            "CHAT",
            &format!(
                "Rejected chat request from unauthenticated user ip={}",
                peer_addr(&http_req)
            ),
        )
        .await;
        return HttpResponse::Unauthorized().json(json!({
            "error": "Not logged in. Please register and log in using PGP authentication."
        }));
    }

    let username = match get_username(&session) {
        Some(u) => u,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unable to retrieve username"
            }));
        }
    };

    let is_admin = is_root_admin(&session);
    let message = req.message.trim();

    if message.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Message cannot be empty"
        }));
    }

    // Log chat message
    logging::log(
        &data.db,
        "INFO",
        "CHAT",
        &format!("User {} sent message: {}", username, message),
    )
    .await;

    // Get response from Jeebs
    let response = crate::cortex::custom_ai_logic_with_context(
        message,
        &data.db,
        &[],
        Some(&username),
        Some(&username),
    )
    .await;

    HttpResponse::Ok().json(UserChatResponse {
        response,
        username,
        is_admin,
    })
}

/// Get chat status (check if user is authenticated)
#[get("/api/chat/status")]
pub async fn chat_status(
    session: Session,
) -> impl Responder {
    if !is_user_authenticated(&session) {
        return HttpResponse::Ok().json(json!({
            "authenticated": false,
            "username": null,
            "message": "Not logged in"
        }));
    }

    let username = get_username(&session);
    let is_admin = is_root_admin(&session);

    HttpResponse::Ok().json(json!({
        "authenticated": true,
        "username": username,
        "is_admin": is_admin,
        "message": "Ready to chat!"
    }))
}
