// === HTTP proxy with credential injection ===

use axum::http::{HeaderMap, Method};
use reqwest::Client;
use serde_json::Value;

use crate::config::StoredCredential;
use crate::error::GatewayError;

// === Proxy client for forwarding requests ===
#[derive(Clone)]
pub struct ProxyClient {
    client: Client,
}

impl ProxyClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    // === Forward request to external service with injected credentials ===
    pub async fn forward(
        &self,
        base_url: &str,
        path: &str,
        method: Method,
        headers: HeaderMap,
        body: Option<Value>,
        credential: &StoredCredential,
    ) -> Result<(u16, Value), GatewayError> {
        let url = format!("{}/{}", base_url.trim_end_matches('/'), path);

        // Build request
        let mut request = match method {
            Method::GET => self.client.get(&url),
            Method::POST => self.client.post(&url),
            Method::PUT => self.client.put(&url),
            Method::DELETE => self.client.delete(&url),
            Method::PATCH => self.client.patch(&url),
            _ => return Err(GatewayError::BadRequest("Unsupported method".to_string())),
        };

        // Inject authorization header
        request = request.header("Authorization", format!("Bearer {}", credential.access_token));

        // Forward relevant headers (skip hop-by-hop headers)
        for (name, value) in headers.iter() {
            let name_str = name.as_str().to_lowercase();
            if !is_hop_by_hop(&name_str) && name_str != "host" && name_str != "authorization" {
                if let Ok(v) = value.to_str() {
                    request = request.header(name.as_str(), v);
                }
            }
        }

        // Add body if present
        if let Some(json_body) = body {
            request = request.json(&json_body);
        }

        // Execute request
        let response = request
            .send()
            .await
            .map_err(|e| GatewayError::UpstreamError(format!("Request failed: {}", e)))?;

        let status = response.status().as_u16();

        // Parse response body
        let body: Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"raw": "non-json response"}));

        Ok((status, body))
    }
}

impl Default for ProxyClient {
    fn default() -> Self {
        Self::new()
    }
}

// === Check if header is hop-by-hop (should not be forwarded) ===
fn is_hop_by_hop(name: &str) -> bool {
    matches!(
        name,
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
    )
}
