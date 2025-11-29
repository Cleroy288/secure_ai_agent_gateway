use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum GatewayError {
    // Auth errors
    Unauthorized(String),
    SessionExpired,
    TokenError(String),

    // Access errors (Forbidden for future authorization)
    #[allow(dead_code)]
    Forbidden(String),
    ServiceNotAllowed(String),
    RateLimitExceeded,

    // Request errors
    BadRequest(String),
    #[allow(dead_code)]
    ReplayDetected,

    // Proxy errors
    UpstreamError(String),
    CredentialNotFound(String),
    #[allow(dead_code)]
    TokenRefreshFailed(String),

    // Internal errors
    Internal(String),
    NotFound(String),
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            GatewayError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            GatewayError::SessionExpired => {
                (StatusCode::UNAUTHORIZED, "session_expired", "Session has expired".to_string())
            }
            GatewayError::TokenError(msg) => (StatusCode::UNAUTHORIZED, "token_error", msg),
            GatewayError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            GatewayError::ServiceNotAllowed(svc) => {
                (StatusCode::FORBIDDEN, "service_not_allowed", format!("Access to {} not permitted", svc))
            }
            GatewayError::RateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded", "Rate limit exceeded".to_string())
            }
            GatewayError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            GatewayError::ReplayDetected => {
                (StatusCode::BAD_REQUEST, "replay_detected", "Replay attack detected".to_string())
            }
            GatewayError::UpstreamError(msg) => {
                (StatusCode::BAD_GATEWAY, "upstream_error", msg)
            }
            GatewayError::CredentialNotFound(svc) => {
                (StatusCode::NOT_FOUND, "credential_not_found", format!("No credentials for {}", svc))
            }
            GatewayError::TokenRefreshFailed(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "token_refresh_failed", msg)
            }
            GatewayError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg)
            }
            GatewayError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
        };

        let body = Json(json!({
            "error": error_type,
            "message": message,
        }));

        (status, body).into_response()
    }
}
