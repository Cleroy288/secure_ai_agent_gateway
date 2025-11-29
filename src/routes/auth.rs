use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::GatewayError;
use crate::models::{Agent, User};
use crate::state::AppState;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_user))
        .route("/agent", post(create_agent_access))
        .route("/agent/{agent_id}", get(get_agent_info))
        .route("/agent/{agent_id}/rotate", post(rotate_agent_key))
        .route("/agent/{agent_id}/services", post(grant_service_access))
        .route("/agent/{agent_id}/services/{service_id}", delete(revoke_service_access))
        .route("/services", get(list_available_services))
}

// ============ Request/Response Types ============

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub user_id: Uuid,
    pub agent_name: String,
    pub agent_description: String,
    pub services: Vec<String>,
    #[serde(default = "default_lifespan")]
    pub lifespan_days: u32,
}

fn default_lifespan() -> u32 { 30 }

#[derive(Debug, Serialize)]
pub struct CreateAgentResponse {
    pub agent_id: Uuid,
    pub session_id: String,
    pub agent_name: String,
    pub allowed_services: Vec<String>,
    pub expires_in_secs: u64,
    pub key_expires_at: String,
    pub lifespan_days: u32,
}

#[derive(Debug, Serialize)]
pub struct AgentInfoResponse {
    pub agent_id: Uuid,
    pub name: String,
    pub description: String,
    pub allowed_services: Vec<String>,
    pub rate_limit: crate::models::RateLimit,
    pub expires_at: String,
    pub lifespan_days: u32,
    pub days_until_expiry: i64,
    pub is_expired: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct GrantServiceRequest {
    pub service_id: String,
}

#[derive(Debug, Serialize)]
pub struct GrantServiceResponse {
    pub agent_id: Uuid,
    pub service_id: String,
    pub allowed_services: Vec<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct RotateKeyResponse {
    pub agent_id: Uuid,
    pub new_session_id: String,
    pub expires_at: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct AvailableServicesResponse {
    pub services: Vec<ServiceInfo>,
}

// ============ Handlers ============

/// POST /auth/register
/// Register a new user with username and email
async fn register_user(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, GatewayError> {
    // Validate input
    if req.username.trim().is_empty() {
        return Err(GatewayError::BadRequest("Username cannot be empty".to_string()));
    }
    if req.email.trim().is_empty() || !req.email.contains('@') {
        return Err(GatewayError::BadRequest("Invalid email".to_string()));
    }

    // Create user
    let user = User::new(req.username.clone(), req.email.clone());
    let user = state.users.create_user(user).await?;

    tracing::info!(user_id = %user.id, username = %user.username, "User registered");

    Ok(Json(RegisterResponse {
        user_id: user.id,
        username: user.username,
        email: user.email,
        message: "Registration successful. Use your user_id to create agent access.".to_string(),
    }))
}

/// POST /auth/agent
/// Create an agent with access to specified services, returns session_id
async fn create_agent_access(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<CreateAgentResponse>, GatewayError> {
    // Verify user exists
    let mut user = state
        .users
        .get_user(req.user_id)
        .await
        .ok_or_else(|| GatewayError::NotFound("User not found".to_string()))?;

    // Validate requested services exist
    let mut valid_services = Vec::new();
    for service_id in &req.services {
        if state.services.exists(service_id) {
            valid_services.push(service_id.clone());
        } else {
            return Err(GatewayError::BadRequest(format!(
                "Service '{}' does not exist",
                service_id
            )));
        }
    }

    if valid_services.is_empty() {
        return Err(GatewayError::BadRequest(
            "At least one valid service must be specified".to_string(),
        ));
    }

    // Create agent with lifespan
    let mut agent = Agent::with_lifespan(
        req.agent_name.clone(),
        req.agent_description,
        req.lifespan_days,
    );
    agent.allowed_services = valid_services.clone();

    let agent = state.agents.create_agent(agent.clone()).await?;

    // Link agent to user
    user.add_agent(agent.id);
    state.users.update_user(user).await?;

    // Create session
    let session = state
        .agents
        .create_session(agent.id, state.settings.session_ttl_secs)
        .await?;

    tracing::info!(
        agent_id = %agent.id,
        session_id = %session.session_id,
        services = ?valid_services,
        lifespan_days = req.lifespan_days,
        "Agent access created"
    );

    Ok(Json(CreateAgentResponse {
        agent_id: agent.id,
        session_id: session.session_id,
        agent_name: agent.name,
        allowed_services: valid_services,
        expires_in_secs: state.settings.session_ttl_secs,
        key_expires_at: agent.expires_at.to_rfc3339(),
        lifespan_days: agent.lifespan_days,
    }))
}

/// GET /auth/agent/{agent_id}
/// Get agent information including expiration status
async fn get_agent_info(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<AgentInfoResponse>, GatewayError> {
    let agent = state
        .agents
        .get_agent(agent_id)
        .await
        .ok_or_else(|| GatewayError::NotFound("Agent not found".to_string()))?;

    let days_until_expiry = agent.days_until_expiry();
    let is_expired = agent.is_expired();

    Ok(Json(AgentInfoResponse {
        agent_id: agent.id,
        name: agent.name.clone(),
        description: agent.description.clone(),
        allowed_services: agent.allowed_services.clone(),
        rate_limit: agent.rate_limit,
        expires_at: agent.expires_at.to_rfc3339(),
        lifespan_days: agent.lifespan_days,
        days_until_expiry,
        is_expired,
        created_at: agent.created_at.to_rfc3339(),
        updated_at: agent.updated_at.to_rfc3339(),
    }))
}

/// POST /auth/agent/{agent_id}/rotate
/// Rotate/regenerate the access key (extends expiration)
async fn rotate_agent_key(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<RotateKeyResponse>, GatewayError> {
    let mut agent = state
        .agents
        .get_agent(agent_id)
        .await
        .ok_or_else(|| GatewayError::NotFound("Agent not found".to_string()))?;

    // Rotate the key
    let new_id = agent.rotate();
    state.agents.update_agent(agent.clone()).await?;

    // Create new session for the rotated key
    let session = state
        .agents
        .create_session(new_id, state.settings.session_ttl_secs)
        .await?;

    tracing::info!(
        old_agent_id = %agent_id,
        new_agent_id = %new_id,
        "Agent key rotated"
    );

    Ok(Json(RotateKeyResponse {
        agent_id: new_id,
        new_session_id: session.session_id,
        expires_at: agent.expires_at.to_rfc3339(),
        message: "Access key rotated successfully. Use new session_id for requests.".to_string(),
    }))
}

/// POST /auth/agent/{agent_id}/services
/// Grant service access to an agent
async fn grant_service_access(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<GrantServiceRequest>,
) -> Result<Json<GrantServiceResponse>, GatewayError> {
    // Verify service exists
    if !state.services.exists(&req.service_id) {
        return Err(GatewayError::BadRequest(format!(
            "Service '{}' does not exist",
            req.service_id
        )));
    }

    let mut agent = state
        .agents
        .get_agent(agent_id)
        .await
        .ok_or_else(|| GatewayError::NotFound("Agent not found".to_string()))?;

    // Check if already has access
    if agent.can_access_service(&req.service_id) {
        return Err(GatewayError::BadRequest(format!(
            "Agent already has access to service '{}'",
            req.service_id
        )));
    }

    // Grant access
    agent.add_service(req.service_id.clone());
    state.agents.update_agent(agent.clone()).await?;

    tracing::info!(
        agent_id = %agent_id,
        service_id = %req.service_id,
        "Service access granted"
    );

    Ok(Json(GrantServiceResponse {
        agent_id,
        service_id: req.service_id,
        allowed_services: agent.allowed_services,
        message: "Service access granted successfully".to_string(),
    }))
}

/// DELETE /auth/agent/{agent_id}/services/{service_id}
/// Revoke service access from an agent
async fn revoke_service_access(
    State(state): State<AppState>,
    Path((agent_id, service_id)): Path<(Uuid, String)>,
) -> Result<Json<GrantServiceResponse>, GatewayError> {
    let mut agent = state
        .agents
        .get_agent(agent_id)
        .await
        .ok_or_else(|| GatewayError::NotFound("Agent not found".to_string()))?;

    // Remove access
    if !agent.remove_service(&service_id) {
        return Err(GatewayError::BadRequest(format!(
            "Agent does not have access to service '{}'",
            service_id
        )));
    }

    state.agents.update_agent(agent.clone()).await?;

    tracing::info!(
        agent_id = %agent_id,
        service_id = %service_id,
        "Service access revoked"
    );

    Ok(Json(GrantServiceResponse {
        agent_id,
        service_id,
        allowed_services: agent.allowed_services,
        message: "Service access revoked successfully".to_string(),
    }))
}

/// GET /auth/services
/// List all available services (requires valid session)
async fn list_available_services(
    State(state): State<AppState>,
) -> Result<Json<AvailableServicesResponse>, GatewayError> {
    let services: Vec<ServiceInfo> = state
        .services
        .list()
        .iter()
        .map(|s| ServiceInfo {
            id: s.id.clone(),
            name: s.name.clone(),
            description: s.description.clone(),
        })
        .collect();

    Ok(Json(AvailableServicesResponse { services }))
}
