//! Session creation utilities (prepared for session management)

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::models::AgentSession;

#[allow(dead_code)]
pub fn create_session(agent_id: Uuid, ttl_secs: u64) -> AgentSession {
    let now = Utc::now();
    AgentSession {
        session_id: Uuid::new_v4().to_string(),
        agent_id,
        created_at: now,
        expires_at: now + Duration::seconds(ttl_secs as i64),
        last_used_at: now,
    }
}
