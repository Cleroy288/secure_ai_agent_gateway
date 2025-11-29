use std::sync::Arc;

use crate::config::{CredentialManager, ServiceRegistry, Settings};
use crate::error::GatewayError;
use crate::gateway::RateLimiter;
use crate::storage::{AgentStore, UserStore};

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub users: UserStore,
    pub agents: AgentStore,
    pub services: Arc<ServiceRegistry>,
    pub credentials: Arc<CredentialManager>,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(settings: Settings) -> Result<Self, GatewayError> {
        let services = ServiceRegistry::load_from_file(&settings.services_config_path)?;
        let credentials = CredentialManager::load_from_file(&settings.credentials_path)?;
        let users = UserStore::load_from_file("data/users.json")?;
        let agents = AgentStore::load_from_file("data/agents.json")?;
        let rate_limiter = RateLimiter::new();

        Ok(Self {
            settings: Arc::new(settings),
            users,
            agents,
            services: Arc::new(services),
            credentials: Arc::new(credentials),
            rate_limiter,
        })
    }
}
