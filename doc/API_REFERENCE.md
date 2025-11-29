# API Reference

## Overview

Base URL: `http://localhost:3000`

All requests return JSON. Errors follow this format:
```json
{
  "error": "error_type",
  "message": "Human readable message"
}
```

---

## Authentication

### Register User

```http
POST /auth/register
Content-Type: application/json
```

**Request:**
```json
{
  "username": "john",
  "email": "john@example.com"
}
```

**Response:** `200 OK`
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john",
  "email": "john@example.com",
  "message": "Registration successful"
}
```

---

### Create Access Key

```http
POST /auth/agent
Content-Type: application/json
```

**Request:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "agent_name": "My AI Agent",
  "agent_description": "Handles payment operations",
  "services": ["payment", "bank"],
  "lifespan_days": 30
}
```

**Response:** `200 OK`
```json
{
  "agent_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "agent_name": "My AI Agent",
  "allowed_services": ["payment", "bank"],
  "expires_in_secs": 3600,
  "key_expires_at": "2025-12-29T17:00:00Z",
  "lifespan_days": 30
}
```

---

### Get Access Key Info

```http
GET /auth/agent/{agent_id}
```

**Response:** `200 OK`
```json
{
  "agent_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "name": "My AI Agent",
  "description": "Handles payment operations",
  "allowed_services": ["payment", "bank"],
  "rate_limit": { "requests": 100, "window_secs": 60 },
  "expires_at": "2025-12-29T17:00:00Z",
  "lifespan_days": 30,
  "days_until_expiry": 25,
  "is_expired": false
}
```

---

### Rotate Access Key

```http
POST /auth/agent/{agent_id}/rotate
```

**Response:** `200 OK`
```json
{
  "agent_id": "new-uuid-after-rotation",
  "new_session_id": "new-session-id",
  "expires_at": "2025-12-29T17:00:00Z",
  "message": "Access key rotated successfully"
}
```

---

### Grant Service Access

```http
POST /auth/agent/{agent_id}/services
Content-Type: application/json
```

**Request:**
```json
{
  "service_id": "payment"
}
```

**Response:** `200 OK`
```json
{
  "agent_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "service_id": "payment",
  "allowed_services": ["payment", "bank"],
  "message": "Service access granted"
}
```

---

### Revoke Service Access

```http
DELETE /auth/agent/{agent_id}/services/{service_id}
```

**Response:** `200 OK`
```json
{
  "agent_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "service_id": "payment",
  "allowed_services": ["bank"],
  "message": "Service access revoked"
}
```

---

### List Services

```http
GET /auth/services
```

**Response:** `200 OK`
```json
{
  "services": [
    {
      "id": "payment",
      "name": "Payment Service",
      "description": "Payment processing API"
    },
    {
      "id": "bank",
      "name": "Bank Service",
      "description": "Banking API"
    }
  ]
}
```

---

## Proxy

### Proxy Request

```http
ANY /api/{service}/{path}
X-Session-ID: your-session-id
```

Proxies the request to the external service with credential injection.

**Flow:**
1. Validate session
2. Check access key expiration
3. Verify service access permission
4. Apply rate limiting
5. Inject credentials
6. Forward to external service
7. Return response

**Example:**
```bash
curl http://localhost:3000/api/payment/transactions \
  -H "X-Session-ID: a1b2c3d4-e5f6-7890-abcd-ef1234567890"
```

---

## Error Codes

| Status | Error Type | Description |
|--------|------------|-------------|
| 400 | `bad_request` | Invalid input |
| 401 | `unauthorized` | Missing/invalid session |
| 401 | `session_expired` | Session has expired |
| 403 | `service_not_allowed` | No access to service |
| 404 | `not_found` | Resource not found |
| 429 | `rate_limit_exceeded` | Too many requests |
| 502 | `upstream_error` | External service error |

---

## cURL Examples

```bash
# 1. Register user
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "john", "email": "john@example.com"}'

# 2. Create access key
curl -X POST http://localhost:3000/auth/agent \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "YOUR_USER_ID",
    "agent_name": "My AI Agent",
    "agent_description": "Test agent",
    "services": ["payment"],
    "lifespan_days": 30
  }'

# 3. Proxy request
curl http://localhost:3000/api/payment/transactions \
  -H "X-Session-ID: YOUR_SESSION_ID"

# 4. List services
curl http://localhost:3000/auth/services

# 5. Rotate key
curl -X POST http://localhost:3000/auth/agent/YOUR_AGENT_ID/rotate

# 6. Grant service
curl -X POST http://localhost:3000/auth/agent/YOUR_AGENT_ID/services \
  -H "Content-Type: application/json" \
  -d '{"service_id": "bank"}'

# 7. Revoke service
curl -X DELETE http://localhost:3000/auth/agent/YOUR_AGENT_ID/services/bank
```
