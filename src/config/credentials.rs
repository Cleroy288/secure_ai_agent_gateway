use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::GatewayError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub service_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CredentialsFile {
    credentials: Vec<StoredCredential>,
}

#[derive(Debug, Clone)]
pub struct CredentialManager {
    credentials: Arc<RwLock<HashMap<String, StoredCredential>>>,
    file_path: String,
}

impl CredentialManager {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, GatewayError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let content = fs::read_to_string(&path)
            .map_err(|e| GatewayError::Internal(format!("Failed to read credentials: {}", e)))?;

        let file: CredentialsFile = serde_json::from_str(&content)
            .map_err(|e| GatewayError::Internal(format!("Failed to parse credentials: {}", e)))?;

        let credentials = file
            .credentials
            .into_iter()
            .map(|c| (c.service_id.clone(), c))
            .collect();

        Ok(Self {
            credentials: Arc::new(RwLock::new(credentials)),
            file_path: path_str,
        })
    }

    pub async fn get(&self, service_id: &str) -> Option<StoredCredential> {
        self.credentials.read().await.get(service_id).cloned()
    }

    pub async fn update(&self, credential: StoredCredential) -> Result<(), GatewayError> {
        let mut creds = self.credentials.write().await;
        creds.insert(credential.service_id.clone(), credential);
        
        // Persist to file
        self.save_to_file(&creds).await
    }

    /// Check if credential needs refresh (for future proactive refresh)
    #[allow(dead_code)]
    pub async fn needs_refresh(&self, service_id: &str, buffer_secs: i64) -> bool {
        if let Some(cred) = self.credentials.read().await.get(service_id) {
            if let Some(expires_at) = cred.expires_at {
                let buffer = chrono::Duration::seconds(buffer_secs);
                return Utc::now() + buffer > expires_at;
            }
        }
        false
    }

    async fn save_to_file(&self, creds: &HashMap<String, StoredCredential>) -> Result<(), GatewayError> {
        let file = CredentialsFile {
            credentials: creds.values().cloned().collect(),
        };

        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| GatewayError::Internal(format!("Failed to serialize credentials: {}", e)))?;

        fs::write(&self.file_path, content)
            .map_err(|e| GatewayError::Internal(format!("Failed to write credentials: {}", e)))?;

        Ok(())
    }
}
