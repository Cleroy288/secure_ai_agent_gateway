//! Service credential model (prepared for per-agent credential storage)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCredential {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub service_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl ServiceCredential {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() > exp,
            None => false,
        }
    }

    pub fn needs_refresh(&self, buffer_secs: i64) -> bool {
        match self.expires_at {
            Some(exp) => {
                let buffer = chrono::Duration::seconds(buffer_secs);
                Utc::now() + buffer > exp
            }
            None => false,
        }
    }
}
