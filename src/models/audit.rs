//! Audit log model (prepared for request audit trail)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub session_id: String,
    pub service_id: String,
    pub endpoint: String,
    pub method: String,
    pub status_code: u16,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: u64,
    pub ip_address: Option<IpAddr>,
}

#[allow(dead_code)]
impl AuditLog {
    pub fn new(
        agent_id: Uuid,
        session_id: String,
        service_id: String,
        endpoint: String,
        method: String,
        request_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            session_id,
            service_id,
            endpoint,
            method,
            status_code: 0,
            request_id,
            timestamp: Utc::now(),
            response_time_ms: 0,
            ip_address: None,
        }
    }
}
