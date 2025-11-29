// === Sliding window rate limiter for agents and services ===

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::error::GatewayError;

// === Rate limit configuration ===
#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests: u32,
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests: 100,
            window: Duration::from_secs(60),
        }
    }
}

// === Rate limiter with sliding window ===
#[derive(Clone)]
pub struct RateLimiter {
    // Key: identifier (agent_id or service_id), Value: list of request timestamps
    windows: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    // Default limits (public for testing)
    pub agent_limit: RateLimitConfig,
    pub service_limits: HashMap<String, RateLimitConfig>,
}

impl RateLimiter {
    // === Create new rate limiter with hardcoded limits ===
    pub fn new() -> Self {
        let mut service_limits = HashMap::new();

        // Hardcoded service limits
        service_limits.insert(
            "payment".to_string(),
            RateLimitConfig {
                requests: 100,
                window: Duration::from_secs(60),
            },
        );
        service_limits.insert(
            "bank".to_string(),
            RateLimitConfig {
                requests: 50,
                window: Duration::from_secs(60),
            },
        );

        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
            agent_limit: RateLimitConfig {
                requests: 200,
                window: Duration::from_secs(60),
            },
            service_limits,
        }
    }

    // === Check if request is allowed for agent ===
    pub async fn check_agent(&self, agent_id: &str) -> Result<(), GatewayError> {
        self.check_limit(&format!("agent:{}", agent_id), &self.agent_limit)
            .await
    }

    // === Check if request is allowed for service ===
    pub async fn check_service(&self, service_id: &str) -> Result<(), GatewayError> {
        let limit = self
            .service_limits
            .get(service_id)
            .cloned()
            .unwrap_or_default();

        self.check_limit(&format!("service:{}", service_id), &limit)
            .await
    }

    // === Core rate limit check with sliding window ===
    async fn check_limit(&self, key: &str, config: &RateLimitConfig) -> Result<(), GatewayError> {
        let now = Instant::now();
        let window_start = now - config.window;

        let mut windows = self.windows.write().await;
        let timestamps = windows.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove expired timestamps
        timestamps.retain(|&t| t > window_start);

        // Check if limit exceeded
        if timestamps.len() >= config.requests as usize {
            return Err(GatewayError::RateLimitExceeded);
        }

        // Record this request
        timestamps.push(now);

        Ok(())
    }

    /// Get remaining requests for a key (for future rate limit monitoring)
    #[allow(dead_code)]
    pub async fn remaining(&self, key: &str, config: &RateLimitConfig) -> u32 {
        let now = Instant::now();
        let window_start = now - config.window;

        let windows = self.windows.read().await;
        let count = windows
            .get(key)
            .map(|ts| ts.iter().filter(|&&t| t > window_start).count())
            .unwrap_or(0);

        config.requests.saturating_sub(count as u32)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_allows_under_limit() {
        let limiter = RateLimiter::new();

        for _ in 0..5 {
            assert!(limiter.check_agent("test-agent").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_over_limit() {
        let mut limiter = RateLimiter::new();
        limiter.agent_limit = RateLimitConfig {
            requests: 3,
            window: Duration::from_secs(60),
        };

        assert!(limiter.check_agent("test-agent").await.is_ok());
        assert!(limiter.check_agent("test-agent").await.is_ok());
        assert!(limiter.check_agent("test-agent").await.is_ok());
        assert!(limiter.check_agent("test-agent").await.is_err());
    }
}
