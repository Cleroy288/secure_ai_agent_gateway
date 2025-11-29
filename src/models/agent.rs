use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

use super::common::RateLimit;

/// Default lifespan for access keys: 30 days
#[allow(dead_code)]
const DEFAULT_LIFESPAN_DAYS: i64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub allowed_services: Vec<String>,
    pub scopes: Vec<String>,
    pub rate_limit: RateLimit,
    pub ip_allowlist: Option<Vec<IpAddr>>,
    // === Access Key Lifespan ===
    pub expires_at: DateTime<Utc>,           // When this access key expires
    pub lifespan_days: u32,                  // How long the key is valid (for rotation)
    // === Timestamps ===
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Agent {
    /// Create agent with default lifespan
    #[allow(dead_code)]
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            allowed_services: Vec::new(),
            scopes: Vec::new(),
            rate_limit: RateLimit::default(),
            ip_allowlist: None,
            expires_at: now + Duration::days(DEFAULT_LIFESPAN_DAYS),
            lifespan_days: DEFAULT_LIFESPAN_DAYS as u32,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create agent with custom lifespan
    pub fn with_lifespan(name: String, description: String, lifespan_days: u32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            allowed_services: Vec::new(),
            scopes: Vec::new(),
            rate_limit: RateLimit::default(),
            ip_allowlist: None,
            expires_at: now + Duration::days(lifespan_days as i64),
            lifespan_days,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn can_access_service(&self, service_id: &str) -> bool {
        self.allowed_services.contains(&service_id.to_string())
    }

    /// Check if the access key has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Add a service to allowed services
    pub fn add_service(&mut self, service_id: String) {
        if !self.allowed_services.contains(&service_id) {
            self.allowed_services.push(service_id);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a service from allowed services
    pub fn remove_service(&mut self, service_id: &str) -> bool {
        let initial_len = self.allowed_services.len();
        self.allowed_services.retain(|s| s != service_id);
        if self.allowed_services.len() != initial_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Rotate/regenerate the access key (extends expiration)
    pub fn rotate(&mut self) -> Uuid {
        let now = Utc::now();
        self.id = Uuid::new_v4();
        self.expires_at = now + Duration::days(self.lifespan_days as i64);
        self.updated_at = now;
        self.id
    }

    /// Days until expiration
    pub fn days_until_expiry(&self) -> i64 {
        (self.expires_at - Utc::now()).num_days()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub session_id: String,
    pub agent_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

impl AgentSession {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
