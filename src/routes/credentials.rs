use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::state::AppState;

pub fn credential_routes() -> Router<AppState> {
    Router::new()
        .route("/{service}", post(store_credential))
        .route("/{service}", delete(remove_credential))
        .route("/", get(list_credentials))
}

async fn store_credential() -> &'static str {
    // TODO: Implement credential storage
    "stored"
}

async fn remove_credential() -> &'static str {
    // TODO: Implement credential removal
    "removed"
}

async fn list_credentials() -> &'static str {
    // TODO: Implement credential listing (no secrets)
    "[]"
}
