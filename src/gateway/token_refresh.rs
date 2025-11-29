// === Token refresh logic for service credentials ===

use chrono::{Duration, Utc};

use crate::config::StoredCredential;

// === Refresh buffer: 6 hours before expiry ===
const REFRESH_BUFFER_HOURS: i64 = 6;

// === Check if credential needs refresh ===
pub fn needs_refresh(credential: &StoredCredential) -> bool {
    match credential.expires_at {
        Some(expires_at) => {
            let buffer = Duration::hours(REFRESH_BUFFER_HOURS);
            Utc::now() + buffer > expires_at
        }
        None => false, // No expiry = no refresh needed
    }
}

/// Check if credential is expired (alternative expiry check)
#[allow(dead_code)]
pub fn is_expired(credential: &StoredCredential) -> bool {
    match credential.expires_at {
        Some(expires_at) => Utc::now() > expires_at,
        None => false,
    }
}

// === Simulate token refresh (in production, call OAuth2 token endpoint) ===
pub async fn refresh_token(credential: &StoredCredential) -> Option<StoredCredential> {
    // In production: use oauth2 crate to call token_url with refresh_token
    // For now: extend expiry by 1 hour (simulation)

    credential.refresh_token.as_ref()?;

    let mut refreshed = credential.clone();
    refreshed.expires_at = Some(Utc::now() + Duration::hours(1));

    tracing::info!(
        service_id = %credential.service_id,
        "Token refreshed (simulated)"
    );

    Some(refreshed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_credential(hours_until_expiry: i64) -> StoredCredential {
        StoredCredential {
            service_id: "test".to_string(),
            access_token: "token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: Some(Utc::now() + Duration::hours(hours_until_expiry)),
            scopes: vec![],
        }
    }

    #[test]
    fn test_needs_refresh_when_close_to_expiry() {
        // Expires in 5 hours (< 6 hour buffer)
        let cred = make_credential(5);
        assert!(needs_refresh(&cred));
    }

    #[test]
    fn test_no_refresh_when_far_from_expiry() {
        // Expires in 24 hours (> 6 hour buffer)
        let cred = make_credential(24);
        assert!(!needs_refresh(&cred));
    }

    #[test]
    fn test_is_expired() {
        // Already expired
        let cred = make_credential(-1);
        assert!(is_expired(&cred));
    }

    #[tokio::test]
    async fn test_refresh_extends_expiry() {
        let cred = make_credential(1);
        let refreshed = refresh_token(&cred).await.unwrap();

        assert!(refreshed.expires_at.unwrap() > cred.expires_at.unwrap());
    }
}
