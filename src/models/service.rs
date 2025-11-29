//! Service models for external API definitions (prepared for scope enforcement)

use serde::{Deserialize, Serialize};

use super::common::RateLimit;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalService {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub auth_type: ServiceAuthType,
    pub endpoints: Vec<Endpoint>,
    pub global_rate_limit: RateLimit,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceAuthType {
    None,
    ApiKey { header_name: String },
    BearerToken,
    OAuth2 { token_url: String, client_id: String },
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub path: String,
    pub method: String,
    pub required_scopes: Vec<String>,
}
