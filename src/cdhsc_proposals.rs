/// CDHSC Proposals: Community-driven brain updates with approval workflow
/// Users can propose additions, modifications, or removals to the holographic brain
/// Community members vote to approve or deny proposals

use actix_web::{post, get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use serde_json::json;
use crate::AppState;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProposalRequest {
    pub proposal_type: String,  // add_node, remove_node, link_nodes, update_metadata
    pub title: String,
    pub description: Option<String>,
    pub node_id: Option<String>,
    pub fact: Option<String>,
    pub source: Option<String>,
    pub from_node_id: Option<String>,
    pub to_node_id: Option<String>,
    pub relation_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposalResponse {
    pub id: String,
    pub proposal_type: String,
    pub status: String,
    pub title: String,
    pub description: Option<String>,
    pub proposer_id: String,
    pub created_at: String,
    pub votes_for: i64,
    pub votes_against: i64,
    pub can_approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    pub vote_type: String,  // approve, deny
}

/// Create a proposal to modify the CDHSC brain
#[post("/api/brain/proposals")]
pub async fn create_proposal(
    data: web::Data<AppState>,
    req: web::Json<CreateProposalRequest>,
    session: actix_session::Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({
            "error": "Not authenticated"
        })),
    };

    let proposal_id = Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO cdhsc_proposals (id, proposal_type, status, title, description, proposer_id,
         created_at, updated_at, node_id, fact, source, from_node_id, to_node_id, relation_type)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&proposal_id)
    .bind(&req.proposal_type)
    .bind("pending")
    .bind(&req.title)
    .bind(&req.description)
    .bind(&username)
    .bind(&now)
    .bind(&now)
    .bind(&req.node_id)
    .bind(&req.fact)
    .bind(&req.source)
    .bind(&req.from_node_id)
    .bind(&req.to_node_id)
    .bind(&req.relation_type)
    .execute(&data.db)
    .await;

    match result {
        Ok(_) => {
            HttpResponse::Created().json(json!({
                "id": proposal_id,
                "status": "pending",
                "message": "Proposal created. Community will review and vote on it."
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to create proposal: {}", e)
            }))
        }
    }
}

/// List all proposals
#[get("/api/brain/proposals")]
pub async fn list_proposals(
    data: web::Data<AppState>,
    web::Query(status): web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let status_filter = status.get("status").map(|s| s.as_str()).unwrap_or("pending");

    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, i64, i64)>(
        "SELECT id, proposal_type, status, title, description, proposer_id, created_at, votes_for, votes_against
         FROM cdhsc_proposals WHERE status = ? ORDER BY created_at DESC LIMIT 50"
    )
    .bind(status_filter)
    .fetch_all(&data.db)
    .await
    .unwrap_or_default();

    let proposals: Vec<ProposalResponse> = rows.into_iter().map(|r| ProposalResponse {
        id: r.0,
        proposal_type: r.1,
        status: r.2,
        title: r.3,
        description: r.4,
        proposer_id: r.5,
        created_at: r.6,
        votes_for: r.7,
        votes_against: r.8,
        can_approve: false,
    }).collect();

    HttpResponse::Ok().json(proposals)
}

/// Vote on a proposal (approve or deny)
#[post("/api/brain/proposals/{proposal_id}/vote")]
pub async fn vote_proposal(
    data: web::Data<AppState>,
    proposal_id: web::Path<String>,
    req: web::Json<VoteRequest>,
    session: actix_session::Session,
) -> impl Responder {
    let username = match session.get::<String>("username") {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().json(json!({
            "error": "Not authenticated"
        })),
    };

    let proposal_id = proposal_id.into_inner();
    let vote_id = Uuid::new_v4().to_string();

    // Insert or replace vote
    match sqlx::query(
        "INSERT INTO cdhsc_proposal_votes (id, proposal_id, voter_id, vote_type, created_at)
         VALUES (?, ?, ?, ?, datetime('now'))
         ON CONFLICT(proposal_id, voter_id) DO UPDATE SET vote_type = ?"
    )
    .bind(&vote_id)
    .bind(&proposal_id)
    .bind(&username)
    .bind(&req.vote_type)
    .bind(&req.vote_type)
    .execute(&data.db)
    .await {
        Ok(_) => {
            // Update proposal vote counts
            let _ = update_proposal_vote_counts(&data.db, &proposal_id).await;
            HttpResponse::Ok().json(json!({
                "message": "Vote recorded",
                "proposal_id": proposal_id
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to vote: {}", e)
            }))
        }
    }
}

/// Approve a proposal (admin only) - implements the proposed change
#[post("/api/brain/proposals/{proposal_id}/approve")]
pub async fn approve_proposal(
    data: web::Data<AppState>,
    proposal_id: web::Path<String>,
    session: actix_session::Session,
) -> impl Responder {
    // Check if admin
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)) == Some(true);
    if !is_admin {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only admins can approve proposals"
        }));
    }

    let proposal_id = proposal_id.into_inner();

    // Get proposal details including type
    let proposal = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>(
        "SELECT proposal_type, node_id, fact, source, from_node_id, to_node_id FROM cdhsc_proposals WHERE id = ?"
    )
    .bind(&proposal_id)
    .fetch_optional(&data.db)
    .await
    .unwrap_or(None);

    match proposal {
        Some((proposal_type, node_id, fact, source, from_node, to_node)) => {
            // Update CDHSC based on proposal type
            let mut chdsc = data.chdsc.write().unwrap();

            match proposal_type.as_str() {
                "add_node" => {
                    if let (Some(nid), Some(f), Some(s)) = (node_id, fact, source) {
                        let node = crate::brain::coded_holographic_data_storage_container::create_node_from_fact(&nid, &f, &s);
                        chdsc.insert_node(node);
                    }
                }
                "link_nodes" => {
                    if let (Some(from), Some(to)) = (from_node, to_node) {
                        chdsc.link(&from, "community_link", &to);
                    }
                }
                _ => {}
            }

            // Save CDHSC
            let _ = chdsc.save(&data.db).await;

            // Update proposal status
            let _ = sqlx::query(
                "UPDATE cdhsc_proposals SET status = ?, updated_at = datetime('now') WHERE id = ?"
            )
            .bind("approved")
            .bind(&proposal_id)
            .execute(&data.db)
            .await;

            HttpResponse::Ok().json(json!({
                "message": "Proposal approved and implemented",
                "proposal_id": proposal_id
            }))
        }
        None => {
            HttpResponse::NotFound().json(json!({
                "error": "Proposal not found"
            }))
        }
    }
}

/// Deny a proposal with reason
#[post("/api/brain/proposals/{proposal_id}/deny")]
pub async fn deny_proposal(
    data: web::Data<AppState>,
    proposal_id: web::Path<String>,
    req: web::Json<std::collections::HashMap<String, String>>,
    session: actix_session::Session,
) -> impl Responder {
    let is_admin = session.get::<bool>("is_admin").unwrap_or(Some(false)) == Some(true);
    if !is_admin {
        return HttpResponse::Forbidden().json(json!({
            "error": "Only admins can deny proposals"
        }));
    }

    let proposal_id = proposal_id.into_inner();
    let reason = req.get("reason").map(|s| s.as_str());

    let _ = sqlx::query(
        "UPDATE cdhsc_proposals SET status = ?, denied_reason = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind("denied")
    .bind(reason)
    .bind(&proposal_id)
    .execute(&data.db)
    .await;

    HttpResponse::Ok().json(json!({
        "message": "Proposal denied",
        "proposal_id": proposal_id
    }))
}

/// Helper: Update vote counts on a proposal
async fn update_proposal_vote_counts(db: &SqlitePool, proposal_id: &str) -> Result<(), String> {
    let votes = sqlx::query_as::<_, (String, i64)>(
        "SELECT vote_type, COUNT(*) as count FROM cdhsc_proposal_votes WHERE proposal_id = ? GROUP BY vote_type"
    )
    .bind(proposal_id)
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    let mut votes_for = 0;
    let mut votes_against = 0;

    for (vote_type, count) in votes {
        if vote_type == "approve" {
            votes_for = count;
        } else {
            votes_against = count;
        }
    }

    sqlx::query(
        "UPDATE cdhsc_proposals SET votes_for = ?, votes_against = ? WHERE id = ?"
    )
    .bind(votes_for)
    .bind(votes_against)
    .bind(proposal_id)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}
