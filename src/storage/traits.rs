//! Storage traits for future database abstraction

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::GatewayError;
use crate::models::{Agent, AgentSession, ServiceCredential};

#[allow(dead_code)]
#[async_trait]
pub trait AgentStoreTrait: Send + Sync {
    async fn get_agent(&self, id: Uuid) -> Result<Option<Agent>, GatewayError>;
    async fn create_agent(&self, agent: Agent) -> Result<Agent, GatewayError>;
    async fn delete_agent(&self, id: Uuid) -> Result<(), GatewayError>;
}

#[allow(dead_code)]
#[async_trait]
pub trait SessionStoreTrait: Send + Sync {
    async fn get_session(&self, session_id: &str) -> Result<Option<AgentSession>, GatewayError>;
    async fn create_session(&self, session: AgentSession) -> Result<AgentSession, GatewayError>;
    async fn delete_session(&self, session_id: &str) -> Result<(), GatewayError>;
}

#[allow(dead_code)]
#[async_trait]
pub trait CredentialStoreTrait: Send + Sync {
    async fn get_credential(
        &self,
        agent_id: Uuid,
        service_id: &str,
    ) -> Result<Option<ServiceCredential>, GatewayError>;
    async fn store_credential(&self, credential: ServiceCredential) -> Result<(), GatewayError>;
    async fn delete_credential(&self, agent_id: Uuid, service_id: &str) -> Result<(), GatewayError>;
}
