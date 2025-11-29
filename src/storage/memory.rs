use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::GatewayError;
use crate::models::{Agent, AgentSession, ServiceCredential};
use super::traits::{AgentStoreTrait, CredentialStoreTrait, SessionStoreTrait};

/// In-memory storage for development/testing
#[allow(dead_code)]
#[derive(Default)]
pub struct InMemoryStore {
    agents: Arc<RwLock<HashMap<Uuid, Agent>>>,
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
    credentials: Arc<RwLock<HashMap<(Uuid, String), ServiceCredential>>>,
}

#[allow(dead_code)]
impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl AgentStoreTrait for InMemoryStore {
    async fn get_agent(&self, id: Uuid) -> Result<Option<Agent>, GatewayError> {
        Ok(self.agents.read().await.get(&id).cloned())
    }

    async fn create_agent(&self, agent: Agent) -> Result<Agent, GatewayError> {
        self.agents.write().await.insert(agent.id, agent.clone());
        Ok(agent)
    }

    async fn delete_agent(&self, id: Uuid) -> Result<(), GatewayError> {
        self.agents.write().await.remove(&id);
        Ok(())
    }
}

#[async_trait]
impl SessionStoreTrait for InMemoryStore {
    async fn get_session(&self, session_id: &str) -> Result<Option<AgentSession>, GatewayError> {
        Ok(self.sessions.read().await.get(session_id).cloned())
    }

    async fn create_session(&self, session: AgentSession) -> Result<AgentSession, GatewayError> {
        self.sessions.write().await.insert(session.session_id.clone(), session.clone());
        Ok(session)
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), GatewayError> {
        self.sessions.write().await.remove(session_id);
        Ok(())
    }
}

#[async_trait]
impl CredentialStoreTrait for InMemoryStore {
    async fn get_credential(
        &self,
        agent_id: Uuid,
        service_id: &str,
    ) -> Result<Option<ServiceCredential>, GatewayError> {
        let key = (agent_id, service_id.to_string());
        Ok(self.credentials.read().await.get(&key).cloned())
    }

    async fn store_credential(&self, credential: ServiceCredential) -> Result<(), GatewayError> {
        let key = (credential.agent_id, credential.service_id.clone());
        self.credentials.write().await.insert(key, credential);
        Ok(())
    }

    async fn delete_credential(&self, agent_id: Uuid, service_id: &str) -> Result<(), GatewayError> {
        let key = (agent_id, service_id.to_string());
        self.credentials.write().await.remove(&key);
        Ok(())
    }
}
