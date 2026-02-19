#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo "$(pwd)")"
cd "$REPO_ROOT"

ts=$(date +%s)
backup_file() {
  local f="$1"
  if [ -f "$f" ]; then
    cp "$f" "${f}.bak.$ts"
    echo "Backed up $f -> ${f}.bak.$ts"
  fi
}

echo "Removing password hashing and hardcoded passwords..."

# auth handler: disable password login and instruct clients to use PGP
auth_file="src/auth/mod.rs"
backup_file "$auth_file"
cat > "$auth_file" <<'EOF'
use actix_web::{web, HttpResponse, Responder, HttpRequest, post};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::state::AppState;
use sqlx::Row;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[post("/api/login")]
pub async fn login(
    _data: web::Data<AppState>,
    _req: web::Json<LoginRequest>,
    _session: Session,
) -> impl Responder {
    // Password-based logins removed. All accounts must use PGP auth.
    HttpResponse::BadRequest().json(json!({"error": "Password login disabled. Use PGP authentication", "use_pgp": true}))
}
EOF

echo "Wrote $auth_file"

# admin user reset: clear any password and mark as PGP-only
admin_user_file="src/admin/user/mod.rs"
backup_file "$admin_user_file"
cat > "$admin_user_file" <<'EOF'
use crate::state::AppState;
use actix_session::Session;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;

#[derive(Serialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub role: String,
}

#[get("/api/admin/users")]
pub async fn admin_list_users(data: web::Data<AppState>, session: Session) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let rows = sqlx::query("SELECT key, value FROM jeebs_store WHERE key LIKE 'user:%'")
        .fetch_all(&data.db)
        .await
        .unwrap_or_default();

    let mut users = Vec::new();
    for row in rows {
        let key: String = row.get(0);
        let val: Vec<u8> = row.get(1);
        if let Ok(user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            let username = key.strip_prefix("user:").unwrap_or(&key).to_string();
            let email = user_json["email"].as_str().unwrap_or("").to_string();
            let role = user_json["role"].as_str().unwrap_or("user").to_string();
            let is_admin = role == "admin";
            users.push(UserInfo {
                username,
                email,
                is_admin,
                role,
            });
        }
    }
    HttpResponse::Ok().json(users)
}

#[delete("/api/admin/user/{username}")]
pub async fn admin_delete_user(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: Session,
) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }
    let username = path.into_inner();
    if username == "admin" {
        return HttpResponse::BadRequest().json(json!({"error": "Cannot delete root admin"}));
    }

    let user_key = format!("user:{username}");
    sqlx::query("DELETE FROM jeebs_store WHERE key = ?")
        .bind(user_key)
        .execute(&data.db)
        .await
        .unwrap();
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub username: String,
    pub new_password: String,
}

#[post("/api/admin/user/reset_password")]
pub async fn admin_reset_user_password(
    data: web::Data<AppState>,
    req: web::Json<ResetPasswordRequest>,
    session: Session,
) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    let user_key = format!("user:{}", req.username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(mut user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            // Clear any stored password and mark the account as PGP-only
            user_json["password"] = serde_json::Value::String("".to_string());
            user_json["auth_type"] = serde_json::Value::String("pgp".to_string());

            sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                .bind(&user_key)
                .bind(serde_json::to_vec(&user_json).unwrap())
                .execute(&data.db)
                .await
                .unwrap();

            return HttpResponse::Ok().json(json!({"ok": true}));
        }
    }
    HttpResponse::NotFound().json(json!({"error": "User not found"}))
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub username: String,
    pub role: String,
}

#[post("/api/admin/user/role")]
pub async fn admin_update_user_role(
    data: web::Data<AppState>,
    req: web::Json<UpdateRoleRequest>,
    session: Session,
) -> impl Responder {
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);
    if !is_admin {
        return HttpResponse::Unauthorized().json(json!({"error": "Admin only"}));
    }

    if req.username == "admin" {
        return HttpResponse::BadRequest().json(json!({"error": "Cannot change root admin role"}));
    }

    let user_key = format!("user:{}", req.username);
    if let Ok(Some(row)) = sqlx::query("SELECT value FROM jeebs_store WHERE key = ?")
        .bind(&user_key)
        .fetch_optional(&data.db)
        .await
    {
        let val: Vec<u8> = row.get(0);
        if let Ok(mut user_json) = serde_json::from_slice::<serde_json::Value>(&val) {
            user_json["role"] = serde_json::Value::String(req.role.clone());

            sqlx::query("INSERT OR REPLACE INTO jeebs_store (key, value) VALUES (?, ?)")
                .bind(&user_key)
                .bind(serde_json::to_vec(&user_json).unwrap())
                .execute(&data.db)
                .await
                .unwrap();

            return HttpResponse::Ok().json(json!({"ok": true}));
        }
    }
    HttpResponse::NotFound().json(json!({"error": "User not found"}))
}
EOF

echo "Wrote $admin_user_file"

# Remove argon2 dependency line from Cargo.toml if present
if [ -f Cargo.toml ]; then
  if grep -q '^argon2' Cargo.toml; then
    backup_file Cargo.toml
    # delete lines starting with argon2
    sed -i.bak '/^argon2/Id' Cargo.toml && rm -f Cargo.toml.bak
    echo "Removed argon2 entry from Cargo.toml"
  else
    echo "No argon2 entry in Cargo.toml"
  fi
fi

# Optional: remove any argon2 imports in other Rust files (non-destructive, creates backups)
rg --hidden --glob '!target' -n "use argon2::" || true
for f in $(rg --hidden --glob '!target' -l "use argon2::" 2>/dev/null || true); do
  backup_file "$f"
  sed -E -e '/use argon2::/Id' -e '/argon2::password_hash::SaltString/Id' "$f" > "${f}.tmp" && mv "${f}.tmp" "$f"
  echo "Stripped argon2 imports from $f"
done

echo "Done. Backups stored with .bak.$ts suffix."

echo "Next steps: run 'cargo build' to verify and run your PGP login flow against /api/login_pgp."
