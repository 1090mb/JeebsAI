use crate::state::AppState;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;
use sysinfo::System;

#[get("/api/admin/status")]
pub async fn get_system_status(data: web::Data<AppState>, session: Session) -> impl Responder {
    if !crate::auth::is_root_admin_session(&session) {
        return HttpResponse::Forbidden()
            .json(json!({"error": "Restricted to 1090mb admin account"}));
    }

    let mut sys = data.sys.lock().unwrap();
    sys.refresh_memory();

    let used_memory = sys.used_memory();
    let total_memory = sys.total_memory();
    let available_memory = sys.available_memory();
    let uptime = System::uptime();
    let used_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    HttpResponse::Ok().json(json!({
        "used_memory": used_memory,
        "total_memory": total_memory,
        "available_memory": available_memory,
        "used_memory_mb": bytes_to_mb(used_memory),
        "total_memory_mb": bytes_to_mb(total_memory),
        "available_memory_mb": bytes_to_mb(available_memory),
        "used_percent": (used_percent * 10.0).round() / 10.0,
        "uptime": uptime,
        "uptime_formatted": format_uptime(uptime)
    }))
}

fn bytes_to_mb(bytes: u64) -> f64 {
    let mb = bytes as f64 / 1024.0 / 1024.0;
    (mb * 100.0).round() / 100.0
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    format!("{days}d {hours}h {minutes}m")
}
