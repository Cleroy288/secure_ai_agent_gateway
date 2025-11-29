use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::GatewayError;
use crate::gateway::{decrypt, encrypt};

/// Credential as stored in JSON file (tokens are encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedCredential {
    pub service_id: String,
    pub access_token: String,           // Encrypted, base64
    pub refresh_token: Option<String>,  // Encrypted, base64
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
    #[serde(default)]
    pub encrypted: bool,                // Flag to detect plaintext migration
}

/// Credential in memory (tokens are decrypted)
#[derive(Debug, Clone)]
pub struct StoredCredential {
    pub service_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CredentialsFile {
    credentials: Vec<EncryptedCredential>,
}

#[derive(Clone)]
pub struct CredentialManager {
    credentials: Arc<RwLock<HashMap<String, StoredCredential>>>,
    file_path: String,
    encryption_key: String,
}

impl CredentialManager {
    /// Load credentials from file, decrypting tokens
    pub fn load_from_file<P: AsRef<Path>>(path: P, encryption_key: &str) -> Result<Self, GatewayError> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        let content = fs::read_to_string(&path)
            .map_err(|e| GatewayError::Internal(format!("Failed to read credentials: {}", e)))?;

        let file: CredentialsFile = serde_json::from_str(&content)
            .map_err(|e| GatewayError::Internal(format!("Failed to parse credentials: {}", e)))?;

        let mut credentials = HashMap::new();
        let mut needs_migration = false;

        for enc_cred in file.credentials {
            let decrypted = if enc_cred.encrypted {
                // Decrypt tokens
                let access_token = decrypt(&enc_cred.access_token, encryption_key)?;
                let refresh_token = match &enc_cred.refresh_token {
                    Some(rt) => Some(decrypt(rt, encryption_key)?),
                    None => None,
                };
                StoredCredential {
                    service_id: enc_cred.service_id,
                    access_token,
                    refresh_token,
                    expires_at: enc_cred.expires_at,
                    scopes: enc_cred.scopes,
                }
            } else {
                // Plaintext migration: mark for re-save
                needs_migration = true;
                tracing::warn!(
                    service_id = %enc_cred.service_id,
                    "Found unencrypted credential, will encrypt on next save"
                );
                StoredCredential {
                    service_id: enc_cred.service_id,
                    access_token: enc_cred.access_token,
                    refresh_token: enc_cred.refresh_token,
                    expires_at: enc_cred.expires_at,
                    scopes: enc_cred.scopes,
                }
            };
            credentials.insert(decrypted.service_id.clone(), decrypted);
        }

        // Auto-migrate plaintext credentials to encrypted (before wrapping in Arc)
        if needs_migration {
            let mut encrypted_creds = Vec::new();
            let mut migration_error: Option<GatewayError> = None;

            for c in credentials.values() {
                match encrypt(&c.access_token, encryption_key) {
                    Ok(access_token) => {
                        let refresh_token = match &c.refresh_token {
                            Some(rt) => match encrypt(rt, encryption_key) {
                                Ok(enc) => Some(enc),
                                Err(e) => {
                                    migration_error = Some(e);
                                    break;
                                }
                            },
                            None => None,
                        };
                        encrypted_creds.push(EncryptedCredential {
                            service_id: c.service_id.clone(),
                            access_token,
                            refresh_token,
                            expires_at: c.expires_at,
                            scopes: c.scopes.clone(),
                            encrypted: true,
                        });
                    }
                    Err(e) => {
                        migration_error = Some(e);
                        break;
                    }
                }
            }

            if let Some(e) = migration_error {
                tracing::error!("Failed to migrate credentials: {:?}", e);
            } else {
                let file = CredentialsFile { credentials: encrypted_creds };
                match serde_json::to_string_pretty(&file) {
                    Ok(content) => {
                        if let Err(e) = fs::write(&path_str, content) {
                            tracing::error!("Failed to write migrated credentials: {}", e);
                        } else {
                            tracing::info!("Migrated credentials to encrypted format");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to serialize migrated credentials: {}", e);
                    }
                }
            }
        }

        Ok(Self {
            credentials: Arc::new(RwLock::new(credentials)),
            file_path: path_str,
            encryption_key: encryption_key.to_string(),
        })
    }

    pub async fn get(&self, service_id: &str) -> Option<StoredCredential> {
        self.credentials.read().await.get(service_id).cloned()
    }

    pub async fn update(&self, credential: StoredCredential) -> Result<(), GatewayError> {
        let mut creds = self.credentials.write().await;
        creds.insert(credential.service_id.clone(), credential);
        self.save_to_file(&creds).await
    }

    /// Check if credential needs refresh
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

    /// Save credentials to file with encryption
    async fn save_to_file(&self, creds: &HashMap<String, StoredCredential>) -> Result<(), GatewayError> {
        let encrypted_creds: Result<Vec<_>, _> = creds
            .values()
            .map(|c| self.encrypt_credential(c))
            .collect();

        let file = CredentialsFile {
            credentials: encrypted_creds?,
        };

        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| GatewayError::Internal(format!("Failed to serialize credentials: {}", e)))?;

        fs::write(&self.file_path, content)
            .map_err(|e| GatewayError::Internal(format!("Failed to write credentials: {}", e)))?;

        Ok(())
    }

    /// Encrypt a credential for storage
    fn encrypt_credential(&self, cred: &StoredCredential) -> Result<EncryptedCredential, GatewayError> {
        let access_token = encrypt(&cred.access_token, &self.encryption_key)?;
        let refresh_token = match &cred.refresh_token {
            Some(rt) => Some(encrypt(rt, &self.encryption_key)?),
            None => None,
        };

        Ok(EncryptedCredential {
            service_id: cred.service_id.clone(),
            access_token,
            refresh_token,
            expires_at: cred.expires_at,
            scopes: cred.scopes.clone(),
            encrypted: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = "test-encryption-key-32-chars!!!";
        
        // Create plaintext credentials file
        let plaintext_json = r#"{
            "credentials": [
                {
                    "service_id": "test-service",
                    "access_token": "secret_token_123",
                    "refresh_token": "refresh_456",
                    "expires_at": "2025-12-31T23:59:59Z",
                    "scopes": ["read", "write"],
                    "encrypted": false
                }
            ]
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(plaintext_json.as_bytes()).unwrap();
        let path = file.path().to_string_lossy().to_string();

        // Load (should auto-migrate to encrypted)
        let manager = CredentialManager::load_from_file(&path, key).unwrap();

        // Verify in-memory credential is decrypted
        let cred = manager.credentials.blocking_read();
        let stored = cred.get("test-service").unwrap();
        assert_eq!(stored.access_token, "secret_token_123");
        assert_eq!(stored.refresh_token, Some("refresh_456".to_string()));

        // Verify file is now encrypted
        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.contains("secret_token_123")); // Token should be encrypted
        assert!(content.contains("\"encrypted\": true"));
    }

    #[test]
    fn test_load_encrypted_credentials() {
        let key = "test-encryption-key-32-chars!!!";
        
        // Pre-encrypt tokens
        let enc_access = encrypt("my_secret_token", key).unwrap();
        let enc_refresh = encrypt("my_refresh_token", key).unwrap();

        let encrypted_json = format!(r#"{{
            "credentials": [
                {{
                    "service_id": "encrypted-service",
                    "access_token": "{}",
                    "refresh_token": "{}",
                    "expires_at": "2025-12-31T23:59:59Z",
                    "scopes": [],
                    "encrypted": true
                }}
            ]
        }}"#, enc_access, enc_refresh);

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(encrypted_json.as_bytes()).unwrap();
        let path = file.path().to_string_lossy().to_string();

        // Load encrypted credentials
        let manager = CredentialManager::load_from_file(&path, key).unwrap();

        // Verify decryption
        let cred = manager.credentials.blocking_read();
        let stored = cred.get("encrypted-service").unwrap();
        assert_eq!(stored.access_token, "my_secret_token");
        assert_eq!(stored.refresh_token, Some("my_refresh_token".to_string()));
    }
}
