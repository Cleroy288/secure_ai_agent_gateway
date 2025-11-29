use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::GatewayError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub auth_type: String,
    pub endpoints: Vec<EndpointConfig>,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub path: String,
    pub methods: Vec<String>,
    pub required_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests: u32,
    pub window_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServicesFile {
    services: Vec<ServiceConfig>,
}

#[derive(Debug, Clone)]
pub struct ServiceRegistry {
    services: HashMap<String, ServiceConfig>,
}

impl ServiceRegistry {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, GatewayError> {
        let content = fs::read_to_string(path)
            .map_err(|e| GatewayError::Internal(format!("Failed to read services config: {}", e)))?;

        let file: ServicesFile = serde_json::from_str(&content)
            .map_err(|e| GatewayError::Internal(format!("Failed to parse services config: {}", e)))?;

        let services = file
            .services
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();

        Ok(Self { services })
    }

    pub fn get(&self, service_id: &str) -> Option<&ServiceConfig> {
        self.services.get(service_id)
    }

    pub fn list(&self) -> Vec<&ServiceConfig> {
        self.services.values().collect()
    }

    pub fn exists(&self, service_id: &str) -> bool {
        self.services.contains_key(service_id)
    }
}
