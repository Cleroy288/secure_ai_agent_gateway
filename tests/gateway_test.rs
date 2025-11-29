use std::time::Duration;

// ===================================================================
// TEST: Token Refresh - Credential close to expiry triggers refresh
// ===================================================================
#[test]
fn test_token_needs_refresh_when_close_to_expiry() {
    use chrono::{Duration as ChronoDuration, Utc};
    use sec_ai_agent_gw::config::StoredCredential;
    use sec_ai_agent_gw::gateway::needs_refresh;

    // Credential expires in 5 hours (< 6 hour buffer)
    let credential = StoredCredential {
        service_id: "payment".to_string(),
        access_token: "token123".to_string(),
        refresh_token: Some("refresh123".to_string()),
        expires_at: Some(Utc::now() + ChronoDuration::hours(5)),
        scopes: vec!["read".to_string()],
    };

    assert!(needs_refresh(&credential));
}

// ===================================================================
// TEST: Token Refresh - Credential far from expiry does not refresh
// ===================================================================
#[test]
fn test_token_no_refresh_when_far_from_expiry() {
    use chrono::{Duration as ChronoDuration, Utc};
    use sec_ai_agent_gw::config::StoredCredential;
    use sec_ai_agent_gw::gateway::needs_refresh;

    // Credential expires in 24 hours (> 6 hour buffer)
    let credential = StoredCredential {
        service_id: "payment".to_string(),
        access_token: "token123".to_string(),
        refresh_token: Some("refresh123".to_string()),
        expires_at: Some(Utc::now() + ChronoDuration::hours(24)),
        scopes: vec!["read".to_string()],
    };

    assert!(!needs_refresh(&credential));
}

// ===================================================================
// TEST: Rate Limiter - Allows requests under limit
// ===================================================================
#[tokio::test]
async fn test_rate_limiter_allows_under_limit() {
    use sec_ai_agent_gw::gateway::RateLimiter;

    let limiter = RateLimiter::new();

    // Should allow multiple requests under the limit
    for i in 0..10 {
        let result = limiter.check_agent("test-agent-1").await;
        assert!(result.is_ok(), "Request {} should be allowed", i);
    }
}

// ===================================================================
// TEST: Rate Limiter - Blocks requests over limit
// ===================================================================
#[tokio::test]
async fn test_rate_limiter_blocks_over_limit() {
    use sec_ai_agent_gw::gateway::{RateLimitConfig, RateLimiter};

    // Create limiter with very low limit for testing
    let mut limiter = RateLimiter::new();
    limiter.agent_limit = RateLimitConfig {
        requests: 3,
        window: Duration::from_secs(60),
    };

    // First 3 requests should succeed
    assert!(limiter.check_agent("test-agent-2").await.is_ok());
    assert!(limiter.check_agent("test-agent-2").await.is_ok());
    assert!(limiter.check_agent("test-agent-2").await.is_ok());

    // 4th request should fail
    let result = limiter.check_agent("test-agent-2").await;
    assert!(result.is_err());
}

// ===================================================================
// TEST: Rate Limiter - Different agents have separate limits
// ===================================================================
#[tokio::test]
async fn test_rate_limiter_separate_agent_limits() {
    use sec_ai_agent_gw::gateway::{RateLimitConfig, RateLimiter};

    let mut limiter = RateLimiter::new();
    limiter.agent_limit = RateLimitConfig {
        requests: 2,
        window: Duration::from_secs(60),
    };

    // Agent A uses its limit
    assert!(limiter.check_agent("agent-a").await.is_ok());
    assert!(limiter.check_agent("agent-a").await.is_ok());
    assert!(limiter.check_agent("agent-a").await.is_err());

    // Agent B still has its own limit
    assert!(limiter.check_agent("agent-b").await.is_ok());
    assert!(limiter.check_agent("agent-b").await.is_ok());
}

// ===================================================================
// TEST: Encryption - Round trip encrypt/decrypt
// ===================================================================
#[test]
fn test_encryption_roundtrip() {
    use sec_ai_agent_gw::gateway::{decrypt, encrypt};

    let key = "my-super-secret-encryption-key!!";
    let plaintext = "sk_live_secret_token_12345";

    let encrypted = encrypt(plaintext, key).unwrap();
    let decrypted = decrypt(&encrypted, key).unwrap();

    assert_eq!(plaintext, decrypted);
    assert_ne!(plaintext, encrypted); // Encrypted should be different
}

// ===================================================================
// TEST: Encryption - Different keys produce different ciphertext
// ===================================================================
#[test]
fn test_encryption_different_keys() {
    use sec_ai_agent_gw::gateway::encrypt;

    let plaintext = "secret_data";
    let key1 = "key-one-32-characters-long!!!!!!";
    let key2 = "key-two-32-characters-long!!!!!!";

    let encrypted1 = encrypt(plaintext, key1).unwrap();
    let encrypted2 = encrypt(plaintext, key2).unwrap();

    // Same plaintext with different keys should produce different ciphertext
    // (Note: due to random nonce, even same key produces different ciphertext)
    assert_ne!(encrypted1, encrypted2);
}
