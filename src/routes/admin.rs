use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::state::AppState;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/agents", get(list_agents))
        .route("/audit", get(query_audit))
        .route("/services", get(list_services))
}

#[derive(Serialize)]
struct AgentInfo {
    id: String,
    name: String,
    description: String,
    allowed_services: Vec<String>,
}

async fn list_agents(State(_state): State<AppState>) -> Json<Vec<AgentInfo>> {
    // TODO: Implement full agent listing from storage
    Json(vec![])
}

async fn query_audit() -> &'static str {
    // TODO: Implement audit log query
    "[]"
}

async fn list_services(State(state): State<AppState>) -> Json<serde_json::Value> {
    let services: Vec<_> = state
        .services
        .list()
        .iter()
        .map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "description": s.description,
            "base_url": s.base_url,
        }))
        .collect();

    Json(serde_json::json!({ "services": services }))
}
