//! JWT token generation and validation (prepared for JWT-based auth)

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::GatewayError;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,     // agent_id
    pub session: String, // session_id
    pub exp: usize,      // expiration timestamp
    pub iat: usize,      // issued at
}

#[allow(dead_code)]
pub fn generate_session_token(
    agent_id: Uuid,
    session_id: &str,
    secret: &str,
    ttl_secs: u64,
) -> Result<String, GatewayError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ttl_secs as i64);

    let claims = Claims {
        sub: agent_id.to_string(),
        session: session_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| GatewayError::TokenError(e.to_string()))
}

#[allow(dead_code)]
pub fn validate_session_token(token: &str, secret: &str) -> Result<Claims, GatewayError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| GatewayError::TokenError(e.to_string()))
}
