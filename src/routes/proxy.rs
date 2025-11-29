// === Proxy routes with rate limiting and token refresh ===

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Method},
    Json, Router,
    routing::any,
};
use serde_json::Value;

use crate::error::GatewayError;
use crate::gateway::{needs_refresh, refresh_token, ProxyClient};
use crate::state::AppState;

const SESSION_HEADER: &str = "x-session-id";

pub fn proxy_routes() -> Router<AppState> {
    Router::new().route("/:service/*path", any(proxy_request))
}

// === Main proxy handler ===
async fn proxy_request(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    Path((service, path)): Path<(String, String)>,
    body: Option<Bytes>,
) -> Result<Json<Value>, GatewayError> {
    // === Extract and validate session ===
    let session_id = headers
        .get(SESSION_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| GatewayError::Unauthorized("Missing X-Session-ID header".to_string()))?;

    let (session, agent) = state.agents.validate_session(session_id).await?;

    // === Check if access key has expired ===
    if agent.is_expired() {
        return Err(GatewayError::Unauthorized(
            "Access key has expired. Please rotate your key.".to_string(),
        ));
    }

    // === Check agent has access to service ===
    if !agent.can_access_service(&service) {
        return Err(GatewayError::ServiceNotAllowed(service.clone()));
    }

    // === Rate limiting ===
    state.rate_limiter.check_agent(&agent.id.to_string()).await?;
    state.rate_limiter.check_service(&service).await?;

    // === Get service config ===
    let service_config = state
        .services
        .get(&service)
        .ok_or_else(|| GatewayError::NotFound(format!("Service '{}' not found", service)))?;

    // === Get and refresh credentials if needed ===
    let mut credential = state
        .credentials
        .get(&service)
        .await
        .ok_or_else(|| GatewayError::CredentialNotFound(service.clone()))?;

    if needs_refresh(&credential) {
        if let Some(refreshed) = refresh_token(&credential).await {
            state.credentials.update(refreshed.clone()).await?;
            credential = refreshed;
            tracing::info!(service = %service, "Token refreshed before proxy");
        }
    }

    // === Parse body if present ===
    let json_body: Option<Value> = body.and_then(|b| serde_json::from_slice(&b).ok());

    // === Forward request ===
    let proxy = ProxyClient::new();
    let (status, response_body) = proxy
        .forward(
            &service_config.base_url,
            &path,
            method,
            headers,
            json_body,
            &credential,
        )
        .await?;

    tracing::info!(
        agent_id = %agent.id,
        session_id = %session.session_id,
        service = %service,
        path = %path,
        status = status,
        "Request proxied"
    );

    Ok(Json(response_body))
}
