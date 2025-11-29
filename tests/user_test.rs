use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use sec_ai_agent_gw::config::Settings;
use sec_ai_agent_gw::routes::auth_routes;
use sec_ai_agent_gw::state::AppState;

fn setup_test_app() -> axum::Router {
    std::env::set_var("ENCRYPTION_KEY", "test-encryption-key-32chars!!");
    std::env::set_var("SESSION_SECRET", "test-session-secret");
    std::env::set_var("SERVICES_CONFIG_PATH", "config/services.json");
    std::env::set_var("CREDENTIALS_PATH", "data/credentials.json");

    let settings = Settings::from_env();
    let state = AppState::new(settings).expect("Failed to create test state");

    auth_routes().with_state(state)
}

// === Generate unique email for each test run ===
fn unique_email() -> String {
    format!("test-{}@example.com", Uuid::new_v4())
}

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or(json!({}));

    (status, json)
}

// ===================================================================
// TEST: User Registration
// Creates a new user with username and email.
// Expects: 200 OK with user_id, username, email in response.
// ===================================================================
#[tokio::test]
async fn test_user_registration() {
    let app = setup_test_app();
    let email = unique_email();

    let (status, body) = post_json(
        app,
        "/register",
        json!({
            "username": "testuser",
            "email": email
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.get("user_id").is_some());
    assert_eq!(body["username"], "testuser");
}

// ===================================================================
// TEST: Agent Access Creation
// Creates an agent with access to specific services for a user.
// Expects: 200 OK with agent_id, session_id, allowed_services.
// ===================================================================
#[tokio::test]
async fn test_agent_access_creation() {
    let app = setup_test_app();
    let email = unique_email();

    // First register a user
    let (status, user_response) = post_json(
        app.clone(),
        "/register",
        json!({
            "username": "agentowner",
            "email": email
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let user_id = user_response["user_id"].as_str().unwrap();

    // Then create an agent for that user
    let (status, agent_response) = post_json(
        app,
        "/agent",
        json!({
            "user_id": user_id,
            "agent_name": "Test Agent",
            "agent_description": "A test AI agent",
            "services": ["payment", "bank"]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(agent_response.get("agent_id").is_some());
    assert!(agent_response.get("session_id").is_some());
    assert_eq!(agent_response["agent_name"], "Test Agent");
    assert_eq!(agent_response["allowed_services"], json!(["payment", "bank"]));
}

// ===================================================================
// TEST: Agent Creation with Invalid User
// Attempts to create an agent for a non-existent user.
// Expects: 404 Not Found error.
// ===================================================================
#[tokio::test]
async fn test_agent_creation_invalid_user() {
    let app = setup_test_app();

    let (status, body) = post_json(
        app,
        "/agent",
        json!({
            "user_id": "00000000-0000-0000-0000-000000000000",
            "agent_name": "Ghost Agent",
            "agent_description": "Should fail",
            "services": ["payment"]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.get("error").is_some());
}
