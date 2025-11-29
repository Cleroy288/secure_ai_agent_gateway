use std::env;

#[derive(Debug, Clone)]
pub struct Settings {
    // Server
    pub host: String,
    pub port: u16,

    // Security
    pub encryption_key: String,
    #[allow(dead_code)]
    pub session_secret: String,  // For future JWT sessions

    // Session management
    pub session_ttl_secs: u64,
    #[allow(dead_code)]
    pub token_refresh_buffer_secs: u64,

    // Paths
    pub services_config_path: String,
    pub credentials_path: String,
}

impl Settings {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a number"),
            encryption_key: env::var("ENCRYPTION_KEY")
                .expect("ENCRYPTION_KEY must be set"),
            session_secret: env::var("SESSION_SECRET")
                .expect("SESSION_SECRET must be set"),
            session_ttl_secs: env::var("SESSION_TTL_SECS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .expect("SESSION_TTL_SECS must be a number"),
            token_refresh_buffer_secs: env::var("TOKEN_REFRESH_BUFFER_SECS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .expect("TOKEN_REFRESH_BUFFER_SECS must be a number"),
            services_config_path: env::var("SERVICES_CONFIG_PATH")
                .unwrap_or_else(|_| "config/services.json".to_string()),
            credentials_path: env::var("CREDENTIALS_PATH")
                .unwrap_or_else(|_| "data/credentials.json".to_string()),
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
