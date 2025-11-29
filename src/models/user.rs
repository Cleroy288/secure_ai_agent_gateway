use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub agents: Vec<Uuid>,  // List of agent IDs owned by this user
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            agents: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_agent(&mut self, agent_id: Uuid) {
        if !self.agents.contains(&agent_id) {
            self.agents.push(agent_id);
            self.updated_at = Utc::now();
        }
    }
}
