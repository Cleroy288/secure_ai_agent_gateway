use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests: u32,
    pub window_secs: u64,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests: 100,
            window_secs: 60,
        }
    }
}
