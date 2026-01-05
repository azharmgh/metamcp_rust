# OWASP Top 10 API Security Risks 2023

This document details the OWASP Top 10 API Security Risks (2023 edition) and evaluates MetaMCP APIs against each risk.

## Overview

The OWASP API Security Top 10 is a standard awareness document for developers and security professionals. It represents a broad consensus about the most critical security risks to APIs.

| Risk | Name | MetaMCP Status |
|------|------|----------------|
| API1:2023 | Broken Object Level Authorization | Mitigated |
| API2:2023 | Broken Authentication | Mitigated |
| API3:2023 | Broken Object Property Level Authorization | Mitigated |
| API4:2023 | Unrestricted Resource Consumption | Partially Mitigated |
| API5:2023 | Broken Function Level Authorization | Mitigated |
| API6:2023 | Unrestricted Access to Sensitive Business Flows | Mitigated |
| API7:2023 | Server Side Request Forgery | Needs Review |
| API8:2023 | Security Misconfiguration | Partially Mitigated |
| API9:2023 | Improper Inventory Management | Mitigated |
| API10:2023 | Unsafe Consumption of APIs | Needs Review |

---

## API1:2023 - Broken Object Level Authorization (BOLA)

### Description
APIs tend to expose endpoints that handle object identifiers, creating a wide attack surface of Object Level Access Control issues. Authorization checks should be considered in every function that accesses a data source using input from the user.

### Attack Scenario
An attacker manipulates object IDs in API requests to access resources belonging to other users:
```
GET /api/v1/mcp-servers/123  # User's own server
GET /api/v1/mcp-servers/456  # Another user's server (unauthorized)
```

### MetaMCP Evaluation

**Current Implementation:**
- All MCP server endpoints require JWT authentication
- Server queries are filtered by `user_id` from the authenticated JWT token
- Database queries use parameterized queries preventing SQL injection

**Code Reference:** `src/api/handlers/mcp_servers.rs`
```rust
// Servers are always filtered by user_id from JWT claims
let servers = sqlx::query_as!(
    McpServer,
    "SELECT * FROM mcp_servers WHERE user_id = $1",
    claims.sub  // User ID from JWT
)
```

**Status:** Mitigated

**Recommendations:**
- Add integration tests that verify cross-user access is blocked
- Consider adding resource-level audit logging

---

## API2:2023 - Broken Authentication

### Description
Authentication mechanisms are often implemented incorrectly, allowing attackers to compromise authentication tokens or exploit implementation flaws to assume other users' identities.

### Attack Scenarios
1. Credential stuffing with leaked username/password pairs
2. Brute force attacks on login endpoints
3. Weak or predictable API keys
4. Token theft through insecure transmission

### MetaMCP Evaluation

**Current Implementation:**
- API key authentication with secure random generation
- JWT tokens with configurable expiration
- Password hashing with Argon2 (industry standard)
- All endpoints use HTTPS (configured at deployment)

**Code Reference:** `src/auth/middleware.rs`
```rust
// JWT validation with expiration check
pub async fn validate_jwt(token: &str) -> Result<Claims, AuthError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;
    // Expiration is automatically validated
    Ok(token_data.claims)
}
```

**Status:** Mitigated

**Recommendations:**
- Implement rate limiting on authentication endpoints
- Add account lockout after failed attempts
- Consider implementing refresh token rotation

---

## API3:2023 - Broken Object Property Level Authorization

### Description
This combines two former risks: Excessive Data Exposure and Mass Assignment. APIs may expose object properties that should be restricted, or accept properties that should not be modifiable by the user.

### Attack Scenarios

**Excessive Data Exposure:**
```json
// Response includes sensitive fields
{
  "id": 123,
  "name": "My Server",
  "api_key": "secret_key_exposed",  // Should not be returned
  "internal_config": {...}          // Should not be returned
}
```

**Mass Assignment:**
```json
// Attacker adds admin field to request
{
  "name": "My Server",
  "is_admin": true  // Should not be assignable
}
```

### MetaMCP Evaluation

**Current Implementation:**
- DTOs (Data Transfer Objects) define explicit fields for requests/responses
- Sensitive fields are excluded from responses
- Serde's `skip_serializing` used for internal fields

**Code Reference:** `src/api/handlers/mcp_servers.rs`
```rust
#[derive(Serialize)]
pub struct McpServerResponse {
    pub id: i32,
    pub name: String,
    pub url: String,
    // api_key is NOT included in response
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub url: String,
    // Only allowed fields can be set
}
```

**Status:** Mitigated

**Recommendations:**
- Add schema validation tests to ensure DTOs match expected structure
- Consider using OpenAPI spec to enforce response schemas

---

## API4:2023 - Unrestricted Resource Consumption

### Description
APIs that do not limit resource consumption can lead to Denial of Service (DoS) or increased operational costs.

### Attack Scenarios
1. Sending large payloads to crash the server
2. Making excessive API requests (rate limiting bypass)
3. Requesting large data sets without pagination
4. Triggering expensive operations repeatedly

### MetaMCP Evaluation

**Current Implementation:**
- Request body size limits via Axum configuration
- Database connection pooling to prevent exhaustion
- Pagination on list endpoints

**Gaps Identified:**
- No rate limiting middleware currently implemented
- No request timeout configuration for long-running operations

**Code Reference:** `src/main.rs`
```rust
// Body size limit
let app = Router::new()
    .layer(DefaultBodyLimit::max(1024 * 1024)); // 1MB limit
```

**Status:** Partially Mitigated

**Recommendations:**
- Implement rate limiting middleware (e.g., tower-governor)
- Add request timeouts for all endpoints
- Implement per-user quotas for API calls
- Add circuit breaker for downstream MCP server calls

---

## API5:2023 - Broken Function Level Authorization

### Description
Complex access control policies with different hierarchies, groups, and roles can lead to authorization flaws. Attackers may access administrative functions.

### Attack Scenarios
```
# Normal user endpoint
GET /api/v1/mcp-servers

# Admin endpoint accessible to normal users
DELETE /api/v1/admin/users/123
POST /api/v1/admin/config
```

### MetaMCP Evaluation

**Current Implementation:**
- Routes are separated into public and protected categories
- JWT claims include user role information
- Admin endpoints (if any) require admin role validation

**Code Reference:** `src/api/routes/mod.rs`
```rust
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/v1/auth/token", post(handlers::authenticate))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/mcp-servers", get(handlers::list_servers))
        // All routes require authentication
        .layer(middleware::from_fn(auth_middleware))
}
```

**Status:** Mitigated

**Recommendations:**
- Add role-based access control (RBAC) middleware
- Implement function-level authorization tests
- Document all admin-only endpoints clearly

---

## API6:2023 - Unrestricted Access to Sensitive Business Flows

### Description
APIs that expose business flows without understanding how to protect them from excessive use may be abused. This is not necessarily a bug—the attacker exploits legitimate functionality.

### Attack Scenarios
1. Automated ticket purchasing (scalping)
2. Mass account creation for spam
3. Automated referral code abuse
4. Resource hoarding

### MetaMCP Evaluation

**Current Implementation:**
- MCP server registration requires authentication
- Tool execution flows through authenticated gateway
- No public registration endpoints

**Sensitive Flows:**
- MCP server registration
- Tool execution via gateway
- Authentication token generation

**Status:** Mitigated

**Recommendations:**
- Implement CAPTCHA for account registration
- Add velocity checks for MCP server creation
- Monitor for unusual tool execution patterns

---

## API7:2023 - Server Side Request Forgery (SSRF)

### Description
SSRF occurs when an API fetches a remote resource without validating the user-supplied URL. This enables attackers to make the application send requests to unexpected destinations.

### Attack Scenarios
```json
// Attacker provides internal URL
{
  "mcp_server_url": "http://169.254.169.254/latest/meta-data/"
}

// Attacker scans internal network
{
  "mcp_server_url": "http://192.168.1.1:22"
}
```

### MetaMCP Evaluation

**Current Implementation:**
- MCP gateway forwards requests to user-configured MCP server URLs
- No URL validation/allowlisting currently implemented

**Potential Risk:**
- Users can register MCP servers with any URL
- Gateway will make requests to those URLs
- Could be used to scan internal networks or access cloud metadata

**Status:** Needs Review

**Recommendations:**
- Implement URL allowlist/blocklist
- Block private IP ranges (10.x, 172.16-31.x, 192.168.x, 169.254.x)
- Block localhost and loopback addresses
- Validate URL scheme (only allow http/https)
- Consider using a proxy for outbound requests
- Add DNS rebinding protection

**Suggested Implementation:**
```rust
fn validate_mcp_server_url(url: &str) -> Result<(), ValidationError> {
    let parsed = Url::parse(url)?;

    // Only allow http/https
    if !["http", "https"].contains(&parsed.scheme()) {
        return Err(ValidationError::InvalidScheme);
    }

    // Block private IPs
    if let Some(host) = parsed.host() {
        if is_private_ip(host) {
            return Err(ValidationError::PrivateIpBlocked);
        }
    }

    Ok(())
}
```

---

## API8:2023 - Security Misconfiguration

### Description
APIs and supporting systems often contain misconfigurations that can be exploited. This includes missing security headers, unnecessary features enabled, or insecure default configurations.

### Common Misconfigurations
1. Missing security headers (CORS, CSP, etc.)
2. Verbose error messages exposing internals
3. Default credentials not changed
4. Unnecessary HTTP methods enabled
5. TLS/SSL misconfiguration

### MetaMCP Evaluation

**Current Implementation:**
- CORS configuration present
- Structured error responses (no stack traces in production)
- Environment-based configuration

**Gaps Identified:**
- Security headers could be more comprehensive
- Debug logging may expose sensitive data

**Code Reference:** `src/api/routes/mod.rs`
```rust
// CORS configuration
let cors = CorsLayer::new()
    .allow_origin(Any)  // Consider restricting in production
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(Any);
```

**Status:** Partially Mitigated

**Recommendations:**
- Restrict CORS origins in production
- Add security headers middleware:
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `X-XSS-Protection: 1; mode=block`
  - `Strict-Transport-Security` (HSTS)
- Disable verbose error messages in production
- Implement secrets management (not env vars)
- Regular security configuration audits

---

## API9:2023 - Improper Inventory Management

### Description
APIs tend to expose more endpoints than traditional web applications. Proper documentation and inventory of hosts and API versions is important. Old API versions and debug endpoints often remain exposed.

### Issues
1. Undocumented endpoints
2. Old API versions still accessible
3. Debug/test endpoints in production
4. Shadow APIs (unknown to security team)

### MetaMCP Evaluation

**Current Implementation:**
- Single API version (`/api/v1/`)
- Routes defined centrally in `src/api/routes/mod.rs`
- Health check endpoints documented

**API Inventory:**
| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/health` | GET | No | Health check |
| `/mcp/health` | GET | No | MCP gateway health |
| `/api/v1/auth/token` | POST | No | Get JWT token |
| `/api/v1/mcp-servers` | GET | Yes | List MCP servers |
| `/api/v1/mcp-servers` | POST | Yes | Create MCP server |
| `/api/v1/mcp-servers/:id` | GET | Yes | Get MCP server |
| `/api/v1/mcp-servers/:id` | PUT | Yes | Update MCP server |
| `/api/v1/mcp-servers/:id` | DELETE | Yes | Delete MCP server |
| `/mcp` | POST | Yes | MCP JSON-RPC |
| `/mcp` | GET | Yes | MCP SSE stream |

**Status:** Mitigated

**Recommendations:**
- Generate OpenAPI/Swagger documentation automatically
- Implement API versioning strategy for future
- Add deprecation headers when retiring endpoints
- Regular endpoint inventory audits

---

## API10:2023 - Unsafe Consumption of APIs

### Description
Developers tend to trust data received from third-party APIs more than user input. Attackers may target integrated services to compromise the API.

### Attack Scenarios
1. Malicious data from compromised third-party API
2. Man-in-the-middle on API-to-API communication
3. Injection via third-party data

### MetaMCP Evaluation

**Current Implementation:**
- MetaMCP consumes responses from registered MCP servers
- MCP server responses are parsed and forwarded to clients

**Potential Risks:**
- MCP servers could return malicious payloads
- Tool execution results are not sanitized
- Resource content is not validated

**Status:** Needs Review

**Recommendations:**
- Validate/sanitize data from MCP servers
- Implement response size limits
- Add timeout for MCP server requests
- Consider sandboxing tool execution results
- Validate JSON-RPC response structure strictly
- Log anomalous responses from MCP servers

**Suggested Implementation:**
```rust
fn validate_mcp_response(response: &JsonRpcResponse) -> Result<(), ValidationError> {
    // Validate response structure
    if response.jsonrpc != "2.0" {
        return Err(ValidationError::InvalidJsonRpc);
    }

    // Limit response size
    let serialized = serde_json::to_string(response)?;
    if serialized.len() > MAX_RESPONSE_SIZE {
        return Err(ValidationError::ResponseTooLarge);
    }

    Ok(())
}
```

---

## Security Testing Checklist

Use the `api_security_test` binary to verify these security controls:

### Authentication Tests
- [ ] Unauthenticated requests are rejected (401)
- [ ] Invalid tokens are rejected (401)
- [ ] Expired tokens are rejected (401)
- [ ] Malformed tokens are rejected (401)

### Authorization Tests
- [ ] Users cannot access other users' MCP servers (403)
- [ ] Object IDs cannot be manipulated for unauthorized access
- [ ] Admin endpoints require admin role

### Input Validation Tests
- [ ] SQL injection attempts are blocked
- [ ] XSS payloads are sanitized/escaped
- [ ] Large payloads are rejected (413)
- [ ] Invalid JSON is rejected (400)

### Rate Limiting Tests
- [ ] Excessive requests are throttled (429)
- [ ] Authentication endpoints have stricter limits

### SSRF Tests
- [ ] Private IP addresses are blocked
- [ ] Localhost URLs are blocked
- [ ] Cloud metadata URLs are blocked

---

## References

- [OWASP API Security Top 10 2023](https://owasp.org/API-Security/editions/2023/en/0x11-t10/)
- [OWASP API Security Project](https://owasp.org/www-project-api-security/)
- [OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/)
