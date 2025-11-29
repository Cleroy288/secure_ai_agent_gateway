use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::GatewayError;
use crate::models::{Agent, AgentSession, User};

// ============ Users Storage ============

#[derive(Debug, Serialize, Deserialize)]
struct UsersFile {
    users: Vec<User>,
}

#[derive(Clone)]
pub struct UserStore {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    users_by_email: Arc<RwLock<HashMap<String, Uuid>>>,
    file_path: String,
}

impl UserStore {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, GatewayError> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        let content = fs::read_to_string(&path).unwrap_or_else(|_| r#"{"users":[]}"#.to_string());

        let file: UsersFile = serde_json::from_str(&content)
            .map_err(|e| GatewayError::Internal(format!("Failed to parse users: {}", e)))?;

        let mut users = HashMap::new();
        let mut users_by_email = HashMap::new();

        for user in file.users {
            users_by_email.insert(user.email.clone(), user.id);
            users.insert(user.id, user);
        }

        Ok(Self {
            users: Arc::new(RwLock::new(users)),
            users_by_email: Arc::new(RwLock::new(users_by_email)),
            file_path: path_str,
        })
    }

    pub async fn create_user(&self, user: User) -> Result<User, GatewayError> {
        // Check if email already exists
        if self.users_by_email.read().await.contains_key(&user.email) {
            return Err(GatewayError::BadRequest("Email already registered".to_string()));
        }

        let mut users = self.users.write().await;
        let mut by_email = self.users_by_email.write().await;

        by_email.insert(user.email.clone(), user.id);
        users.insert(user.id, user.clone());

        self.save_to_file(&users).await?;
        Ok(user)
    }

    pub async fn get_user(&self, id: Uuid) -> Option<User> {
        self.users.read().await.get(&id).cloned()
    }

    /// Get user by email (alternative lookup method)
    #[allow(dead_code)]
    pub async fn get_user_by_email(&self, email: &str) -> Option<User> {
        let by_email = self.users_by_email.read().await;
        if let Some(id) = by_email.get(email) {
            return self.users.read().await.get(id).cloned();
        }
        None
    }

    pub async fn update_user(&self, user: User) -> Result<(), GatewayError> {
        let mut users = self.users.write().await;
        users.insert(user.id, user);
        self.save_to_file(&users).await
    }

    async fn save_to_file(&self, users: &HashMap<Uuid, User>) -> Result<(), GatewayError> {
        let file = UsersFile {
            users: users.values().cloned().collect(),
        };

        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| GatewayError::Internal(format!("Failed to serialize users: {}", e)))?;

        fs::write(&self.file_path, content)
            .map_err(|e| GatewayError::Internal(format!("Failed to write users: {}", e)))?;

        Ok(())
    }
}

// ============ Agents & Sessions Storage ============

#[derive(Debug, Serialize, Deserialize)]
struct AgentsFile {
    agents: Vec<Agent>,
    sessions: Vec<AgentSession>,
}

#[derive(Clone)]
pub struct AgentStore {
    agents: Arc<RwLock<HashMap<Uuid, Agent>>>,
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
    file_path: String,
}

impl AgentStore {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, GatewayError> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|_| r#"{"agents":[],"sessions":[]}"#.to_string());

        let file: AgentsFile = serde_json::from_str(&content)
            .map_err(|e| GatewayError::Internal(format!("Failed to parse agents: {}", e)))?;

        let agents = file.agents.into_iter().map(|a| (a.id, a)).collect();
        let sessions = file
            .sessions
            .into_iter()
            .map(|s| (s.session_id.clone(), s))
            .collect();

        Ok(Self {
            agents: Arc::new(RwLock::new(agents)),
            sessions: Arc::new(RwLock::new(sessions)),
            file_path: path_str,
        })
    }

    pub async fn create_agent(&self, agent: Agent) -> Result<Agent, GatewayError> {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id, agent.clone());
        self.save_to_file(&agents, &*self.sessions.read().await).await?;
        Ok(agent)
    }

    pub async fn get_agent(&self, id: Uuid) -> Option<Agent> {
        self.agents.read().await.get(&id).cloned()
    }

    pub async fn update_agent(&self, agent: Agent) -> Result<(), GatewayError> {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id, agent);
        self.save_to_file(&agents, &*self.sessions.read().await).await
    }

    /// Delete an agent (for future agent management)
    #[allow(dead_code)]
    pub async fn delete_agent(&self, id: Uuid) -> Result<bool, GatewayError> {
        let mut agents = self.agents.write().await;
        let removed = agents.remove(&id).is_some();
        if removed {
            self.save_to_file(&agents, &*self.sessions.read().await).await?;
        }
        Ok(removed)
    }

    pub async fn create_session(
        &self,
        agent_id: Uuid,
        ttl_secs: u64,
    ) -> Result<AgentSession, GatewayError> {
        let now = Utc::now();
        let session = AgentSession {
            session_id: Uuid::new_v4().to_string(),
            agent_id,
            created_at: now,
            expires_at: now + Duration::seconds(ttl_secs as i64),
            last_used_at: now,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session.clone());

        self.save_to_file(&*self.agents.read().await, &sessions).await?;
        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> Option<AgentSession> {
        self.sessions.read().await.get(session_id).cloned()
    }

    pub async fn validate_session(&self, session_id: &str) -> Result<(AgentSession, Agent), GatewayError> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or_else(|| GatewayError::Unauthorized("Invalid session".to_string()))?;

        if session.is_expired() {
            return Err(GatewayError::SessionExpired);
        }

        let agent = self
            .get_agent(session.agent_id)
            .await
            .ok_or_else(|| GatewayError::Internal("Agent not found".to_string()))?;

        Ok((session, agent))
    }

    async fn save_to_file(
        &self,
        agents: &HashMap<Uuid, Agent>,
        sessions: &HashMap<String, AgentSession>,
    ) -> Result<(), GatewayError> {
        let file = AgentsFile {
            agents: agents.values().cloned().collect(),
            sessions: sessions.values().cloned().collect(),
        };

        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| GatewayError::Internal(format!("Failed to serialize agents: {}", e)))?;

        fs::write(&self.file_path, content)
            .map_err(|e| GatewayError::Internal(format!("Failed to write agents: {}", e)))?;

        Ok(())
    }
}
