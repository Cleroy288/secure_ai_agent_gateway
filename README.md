# Secure AI Agent Gateway

A credential vault and proxy gateway built in Rust that enables AI agents to securely access third-party APIs without directly holding sensitive credentials.

## How It Works

```
┌─────────────┐         ┌─────────────────────┐         ┌──────────────┐
│  AI Agent   │────────►│   SECURE GATEWAY    │────────►│ External API │
│             │         │                     │         │              │
│ Session ID  │         │ • Credential Vault  │         │  (Stripe,    │
│ only        │         │ • Auto-refresh      │         │   GitHub,    │
└─────────────┘         │ • Rate limiting     │         │   OpenAI)    │
                        └─────────────────────┘         └──────────────┘
```

Agents authenticate with a simple session ID. The gateway manages all OAuth tokens, API keys, and credentials internally. When an agent makes a request, the gateway injects the appropriate credentials and proxies the request to the external service.

## Features

- **Credential Vault** — Securely stores API tokens encrypted at rest with AES-256-GCM
- **Automatic Token Refresh** — Refreshes OAuth tokens before they expire
- **Rate Limiting** — Sliding window rate limits per agent and per service
- **Access Control** — Service-level permissions for each agent
- **Key Rotation** — Built-in access key rotation with configurable lifespan
- **Session Management** — Time-limited sessions with automatic expiration

## Tech Stack

Rust, Axum, Tokio, OAuth2, AES-256-GCM

## Quick Start

```bash
# Clone and setup
git clone https://github.com/yourusername/sec_ai_agent_gw.git
cd sec_ai_agent_gw
cp .env.example .env

# Run
cargo run
```

## Usage

**1. Register a user**
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "john", "email": "john@example.com"}'
```

**2. Create an access key**
```bash
curl -X POST http://localhost:3000/auth/agent \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "YOUR_USER_ID",
    "agent_name": "My AI Agent",
    "services": ["payment", "bank"],
    "lifespan_days": 30
  }'
```

**3. Make proxied requests**
```bash
curl http://localhost:3000/api/payment/transactions \
  -H "X-Session-ID: YOUR_SESSION_ID"
```

The gateway automatically injects credentials and forwards the request.

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/auth/register` | POST | Register a new user |
| `/auth/agent` | POST | Create access key |
| `/auth/agent/{id}` | GET | Get access key info |
| `/auth/agent/{id}/rotate` | POST | Rotate access key |
| `/auth/agent/{id}/services` | POST | Grant service access |
| `/auth/agent/{id}/services/{svc}` | DELETE | Revoke service access |
| `/auth/services` | GET | List available services |
| `/api/{service}/{path}` | ANY | Proxy to external service |

See [doc/API_REFERENCE.md](doc/API_REFERENCE.md) for detailed documentation.
