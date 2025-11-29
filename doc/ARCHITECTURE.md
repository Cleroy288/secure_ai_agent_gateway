# Architecture

## Overview

The Secure AI Agent Gateway is a credential vault and proxy that enables AI agents to access third-party APIs without holding sensitive credentials directly.

```
┌─────────────┐         ┌─────────────────────┐         ┌──────────────┐
│  AI Agent   │────────►│   SECURE GATEWAY    │────────►│ External API │
│             │         │                     │         │              │
│ Session ID  │         │ • Credential Vault  │         │  (Stripe,    │
│ only        │         │ • Token Refresh     │         │   GitHub,    │
└─────────────┘         │ • Rate Limiting     │         │   OpenAI)    │
                        │ • Access Control    │         └──────────────┘
                        └─────────────────────┘
```

## Request Flow

1. Agent sends request with `X-Session-ID` header
2. Gateway validates session → retrieves agent
3. Gateway checks service access permissions
4. Gateway applies rate limiting
5. Gateway retrieves credentials, refreshes if needed
6. Gateway injects credentials and proxies request
7. Response flows back to agent

## Security Layers

```
┌────────────────────────────────────────────────┐
│ Layer 1: Session Validation                    │
│   • Validate session exists and not expired    │
├────────────────────────────────────────────────┤
│ Layer 2: Access Control                        │
│   • Agent can only access allowed services     │
│   • Access key expiration check                │
├────────────────────────────────────────────────┤
│ Layer 3: Rate Limiting                         │
│   • Per-agent limits (200 req/min default)     │
│   • Per-service limits (configurable)          │
├────────────────────────────────────────────────┤
│ Layer 4: Credential Injection                  │
│   • Retrieve stored credentials                │
│   • Auto-refresh expired tokens                │
│   • Inject Authorization header                │
└────────────────────────────────────────────────┘
```

## Data Models

### User
- `id`: UUID
- `username`: String
- `email`: String (unique)
- `agents`: Vec<UUID>

### Agent (Access Key)
- `id`: UUID
- `name`: String
- `description`: String
- `allowed_services`: Vec<String>
- `rate_limit`: RateLimit
- `expires_at`: DateTime
- `lifespan_days`: u32

### AgentSession
- `session_id`: String
- `agent_id`: UUID
- `expires_at`: DateTime

### StoredCredential
- `service_id`: String
- `access_token`: String
- `refresh_token`: Option<String>
- `expires_at`: Option<DateTime>

## Project Structure

```
sec_ai_agent_gw/
├── src/
│   ├── main.rs              # Entry point
│   ├── state.rs             # AppState
│   ├── config/
│   │   ├── settings.rs      # Environment config
│   │   ├── services.rs      # Service registry
│   │   └── credentials.rs   # Credential manager
│   ├── models/
│   │   ├── user.rs          # User model
│   │   ├── agent.rs         # Agent, Session
│   │   └── audit.rs         # Audit log
│   ├── routes/
│   │   ├── auth.rs          # /auth/* endpoints
│   │   ├── proxy.rs         # /api/* proxy
│   │   └── admin.rs         # /admin/* endpoints
│   ├── gateway/
│   │   ├── proxy.rs         # HTTP proxy client
│   │   ├── rate_limiter.rs  # Rate limiting
│   │   ├── token_refresh.rs # Token refresh
│   │   └── encryption.rs    # AES-256-GCM
│   ├── storage/
│   │   ├── file_store.rs    # File-based storage
│   │   └── traits.rs        # Storage traits
│   └── error/
│       └── types.rs         # Error types
├── config/
│   └── services.json        # Service definitions
├── data/
│   ├── users.json           # User storage
│   ├── agents.json          # Agent storage
│   └── credentials.json     # Credentials
└── tests/
    ├── gateway_test.rs
    └── user_test.rs
```

## Technology Stack

| Component | Technology |
|-----------|------------|
| Runtime | Tokio |
| Web Framework | Axum |
| HTTP Client | Reqwest |
| Serialization | Serde |
| JWT | jsonwebtoken |
| Encryption | aes-gcm |
| Logging | tracing |

## Configuration

Environment variables (`.env`):

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server host | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `ENCRYPTION_KEY` | AES encryption key | Required |
| `SESSION_SECRET` | Session signing secret | Required |
| `SESSION_TTL_SECS` | Session lifetime | `3600` |
| `SERVICES_CONFIG_PATH` | Services config file | `config/services.json` |
| `CREDENTIALS_PATH` | Credentials file | `data/credentials.json` |
