# Secure AI Agent Gateway

A credential vault and proxy gateway built in Rust that enables AI agents to securely access third-party APIs without directly holding sensitive credentials.

![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)
![Axum](https://img.shields.io/badge/Axum-0.7-blue)

## Overview

The gateway acts as a secure intermediary between AI agents and external APIs. Agents authenticate with a simple session ID, while the gateway manages all OAuth tokens, API keys, and credentials internally.

```
┌─────────────┐         ┌─────────────────────┐         ┌──────────────┐
│  AI Agent   │────────►│   SECURE GATEWAY    │────────►│ External API │
│             │         │                     │         │              │
│ Session ID  │         │ • Credential Vault  │         │  (Stripe,    │
│ only        │         │ • Auto-refresh      │         │   GitHub,    │
└─────────────┘         │ • Rate limiting     │         │   OpenAI)    │
                        │ • Access control    │         └──────────────┘
                        └─────────────────────┘
```

## Features

- **Credential Vault** - Securely stores and manages API tokens
- **Automatic Token Refresh** - Refreshes OAuth tokens before expiry
- **Rate Limiting** - Sliding window rate limits per agent and service
- **Access Control** - Service-level permissions per agent
- **Key Rotation** - Built-in access key rotation with configurable lifespan
- **Audit Ready** - Request logging infrastructure

## Tech Stack

- **Rust** - Systems programming language
- **Axum** - Async web framework
- **Tokio** - Async runtime
- **OAuth2/OIDC** - External service authentication
- **AES-256-GCM** - Credential encryption (module ready)

## Quick Start

### Prerequisites

- Rust 1.70+
- Cargo

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/sec_ai_agent_gw.git
cd sec_ai_agent_gw

# Copy environment file
cp .env.example .env

# Edit .env with your secrets
# ENCRYPTION_KEY and SESSION_SECRET are required

# Build and run
cargo run
```

### Configuration

Edit `.env` to configure the gateway:

```env
HOST=0.0.0.0
PORT=3000
ENCRYPTION_KEY=your-32-char-encryption-key-here
SESSION_SECRET=your-session-signing-secret-here
SESSION_TTL_SECS=3600
```

## Usage

### 1. Register a User

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "john", "email": "john@example.com"}'
```

### 2. Create an Access Key

```bash
curl -X POST http://localhost:3000/auth/agent \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "YOUR_USER_ID",
    "agent_name": "My AI Agent",
    "agent_description": "Handles API operations",
    "services": ["payment", "bank"],
    "lifespan_days": 30
  }'
```

### 3. Make Proxied Requests

```bash
curl http://localhost:3000/api/payment/transactions \
  -H "X-Session-ID: YOUR_SESSION_ID"
```

The gateway automatically injects credentials and forwards the request.

## API Reference

| Endpoint                          | Method | Description                                |
| --------------------------------- | ------ | ------------------------------------------ |
| `/auth/register`                  | POST   | Register a new user                        |
| `/auth/agent`                     | POST   | Create access key with service permissions |
| `/auth/agent/{id}`                | GET    | Get access key info                        |
| `/auth/agent/{id}/rotate`         | POST   | Rotate access key                          |
| `/auth/agent/{id}/services`       | POST   | Grant service access                       |
| `/auth/agent/{id}/services/{svc}` | DELETE | Revoke service access                      |
| `/auth/services`                  | GET    | List available services                    |
| `/api/{service}/{path}`           | ANY    | Proxy request to external service          |

See [API Reference](doc/API_REFERENCE.md) for detailed documentation.

## Project Structure

```
sec_ai_agent_gw/
├── src/
│   ├── main.rs          # Entry point
│   ├── state.rs         # Application state
│   ├── config/          # Configuration & credentials
│   ├── models/          # Data models
│   ├── routes/          # HTTP handlers
│   ├── gateway/         # Proxy, rate limiting, encryption
│   ├── storage/         # Data persistence
│   └── error/           # Error handling
├── config/
│   └── services.json    # Service definitions
├── data/                # Runtime data storage
└── tests/               # Test suite
```

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

## Security Considerations

- Credentials are stored in `data/credentials.json` (encryption module available but not yet integrated)
- Session IDs should be treated as secrets
- Use HTTPS in production
- Rotate access keys regularly

## Roadmap

- [ ] Integrate credential encryption at rest
- [ ] OAuth2 device flow support
- [ ] Full audit logging with persistence
- [ ] Replay protection
- [ ] Database storage (SQLite/Postgres)
- [ ] Admin dashboard

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
