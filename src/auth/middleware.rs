use axum::{extract::Request, middleware::Next, response::Response};

use crate::error::GatewayError;

const SESSION_HEADER: &str = "X-Session-ID";

/// Session authentication middleware (for future use with Tower middleware)
#[allow(dead_code)]
pub async fn session_auth(
    request: Request,
    next: Next,
) -> Result<Response, GatewayError> {
    let session_id = request
        .headers()
        .get(SESSION_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| GatewayError::Unauthorized("Missing session ID".to_string()))?;

    // TODO: Validate session against storage
    // TODO: Inject agent into request extensions

    tracing::debug!(session_id = %session_id, "Session validated");

    Ok(next.run(request).await)
}
