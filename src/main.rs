use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod models;
mod auth;
mod gateway;
mod audit;
mod routes;
mod storage;
mod error;
mod state;

use config::Settings;
use routes::{admin_routes, auth_routes, credential_routes, proxy_routes};
use state::AppState;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .init();

    // Load configuration
    let settings = Settings::from_env();
    let addr = settings.addr();

    tracing::info!("Starting Secure AI Agent Gateway on {}", addr);

    // Initialize application state
    let state = AppState::new(settings).expect("Failed to initialize application state");

    tracing::info!(
        services = state.services.list().len(),
        "Loaded services configuration"
    );

    // Build router with state
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_routes())
        .nest("/credentials", credential_routes())
        .nest("/api", proxy_routes())
        .nest("/admin", admin_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    tracing::info!("Listening on {}", addr);
    tracing::info!("Available endpoints:");
    tracing::info!("  POST /auth/register     - Register new user");
    tracing::info!("  POST /auth/agent        - Create agent with service access");
    tracing::info!("  GET  /auth/services     - List available services");
    tracing::info!("  ANY  /api/{{service}}/{{path}} - Proxy to external service");

    axum::serve(listener, app).await.expect("Server failed");
}

async fn health_check() -> &'static str {
    "OK"
}
