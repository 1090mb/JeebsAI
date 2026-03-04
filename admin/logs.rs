use actix_session::Session;
use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

#[get("/api/admin/logs")]
pub async fn get_logs(session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    // let buffer = get_log_buffer();
    // let logs: Vec<String> = buffer.lock().unwrap().iter().cloned().collect();
    let logs: Vec<String> = Vec::new();
    HttpResponse::Ok().json(logs)
}
