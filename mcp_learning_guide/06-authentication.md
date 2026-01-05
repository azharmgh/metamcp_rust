# Authentication

MCP servers typically use JWT (JSON Web Tokens) for authentication. This guide covers API key-based authentication with JWT tokens.

## Authentication Flow

```
┌────────┐                    ┌────────────┐
│ Client │                    │ MCP Server │
└────┬───┘                    └─────┬──────┘
     │                              │
     │─── POST /auth/token ────────►│
     │    {"api_key": "mcp_xxx"}    │
     │                              │
     │◄─── JWT Token ───────────────│
     │    {"access_token": "eyJ..."}│
     │                              │
     │─── POST /mcp ───────────────►│
     │    Authorization: Bearer eyJ..
     │                              │
     │◄─── MCP Response ────────────│
     │                              │
```

## JWT Structure

A JWT has three parts: Header, Payload, and Signature.

```rust
use serde::{Deserialize, Serialize};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

#[derive(Debug, Serialize, Deserialize)]
struct JwtHeader {
    typ: String,
    alg: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    sub: String,      // Subject (user/key ID)
    exp: u64,         // Expiration time
    iat: u64,         // Issued at
    jti: String,      // JWT ID (unique identifier)
}

fn decode_jwt_parts(token: &str) {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        println!("Invalid JWT format");
        return;
    }

    // Decode header
    if let Ok(header_bytes) = URL_SAFE_NO_PAD.decode(parts[0]) {
        if let Ok(header_str) = String::from_utf8(header_bytes) {
            println!("Header: {}", header_str);
        }
    }

    // Decode payload
    if let Ok(payload_bytes) = URL_SAFE_NO_PAD.decode(parts[1]) {
        if let Ok(payload_str) = String::from_utf8(payload_bytes) {
            println!("Payload: {}", payload_str);
        }
    }

    println!("Signature: {} (base64)", parts[2]);
}

fn main() {
    // Example JWT (DO NOT use in production - for demo only)
    let example_jwt = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.\
                       eyJzdWIiOiJ1c2VyMTIzIiwiZXhwIjoxNzA0MDY3MjAwLCJpYXQiOjE3MDQwNjM2MDAsImp0aSI6ImFiYzEyMyJ9.\
                       signature_here";

    decode_jwt_parts(example_jwt);
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20base64%3A%3A%7BEngine%2C%20engine%3A%3Ageneral_purpose%3A%3AURL_SAFE_NO_PAD%7D%3B%0A%0Afn%20decode_jwt_parts(token%3A%20%26str)%20%7B%0A%20%20%20%20let%20parts%3A%20Vec%3C%26str%3E%20%3D%20token.split('.').collect()%3B%0A%20%20%20%20if%20parts.len()%20!%3D%203%20%7B%20println!(%22Invalid%20JWT%22)%3B%20return%3B%20%7D%0A%20%20%20%20%0A%20%20%20%20if%20let%20Ok(header_bytes)%20%3D%20URL_SAFE_NO_PAD.decode(parts%5B0%5D)%20%7B%0A%20%20%20%20%20%20%20%20if%20let%20Ok(header_str)%20%3D%20String%3A%3Afrom_utf8(header_bytes)%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Header%3A%20%7B%7D%22%2C%20header_str)%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20if%20let%20Ok(payload_bytes)%20%3D%20URL_SAFE_NO_PAD.decode(parts%5B1%5D)%20%7B%0A%20%20%20%20%20%20%20%20if%20let%20Ok(payload_str)%20%3D%20String%3A%3Afrom_utf8(payload_bytes)%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Payload%3A%20%7B%7D%22%2C%20payload_str)%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20jwt%20%3D%20%22eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyMTIzIn0.sig%22%3B%0A%20%20%20%20decode_jwt_parts(jwt)%3B%0A%7D)

## API Key Generation

API keys should be securely generated and stored:

```rust
use uuid::Uuid;

fn generate_api_key() -> String {
    // Format: mcp_<32 hex chars>
    format!("mcp_{}", Uuid::new_v4().simple())
}

fn main() {
    let key = generate_api_key();
    println!("Generated API Key: {}", key);
    println!("Length: {} chars", key.len());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=fn%20generate_api_key()%20-%3E%20String%20%7B%0A%20%20%20%20%2F%2F%20Simulating%20UUID%20generation%20(simplified)%0A%20%20%20%20use%20std%3A%3Atime%3A%3A%7BSystemTime%2C%20UNIX_EPOCH%7D%3B%0A%20%20%20%20let%20now%20%3D%20SystemTime%3A%3Anow().duration_since(UNIX_EPOCH).unwrap()%3B%0A%20%20%20%20format!(%22mcp_%7B%3A032x%7D%22%2C%20now.as_nanos())%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20key%20%3D%20generate_api_key()%3B%0A%20%20%20%20println!(%22Generated%20API%20Key%3A%20%7B%7D%22%2C%20key)%3B%0A%20%20%20%20println!(%22Length%3A%20%7B%7D%20chars%22%2C%20key.len())%3B%0A%7D)

## Token Exchange

The client exchanges an API key for a JWT token:

```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
struct AuthRequest {
    api_key: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

fn authenticate(api_key: &str) -> Result<AuthResponse, String> {
    // In production: validate against database
    if !api_key.starts_with("mcp_") {
        return Err("Invalid API key format".to_string());
    }

    // Generate JWT (simplified - use jsonwebtoken crate in production)
    let token = format!("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.{}.signature",
        base64_encode(&json!({
            "sub": "user-id-123",
            "exp": current_time() + 900,  // 15 minutes
            "iat": current_time(),
        }).to_string())
    );

    Ok(AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 900,
    })
}

fn base64_encode(s: &str) -> String {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD.encode(s)
}

fn current_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn main() {
    match authenticate("mcp_test123") {
        Ok(response) => {
            println!("{}", serde_json::to_string_pretty(&response).unwrap());
        }
        Err(e) => println!("Error: {}", e),
    }
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3Ajson%3B%0Ause%20base64%3A%3A%7BEngine%2C%20engine%3A%3Ageneral_purpose%3A%3AURL_SAFE_NO_PAD%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize)%5D%0Astruct%20AuthResponse%20%7B%0A%20%20%20%20access_token%3A%20String%2C%0A%20%20%20%20token_type%3A%20String%2C%0A%20%20%20%20expires_in%3A%20u64%2C%0A%7D%0A%0Afn%20authenticate(api_key%3A%20%26str)%20-%3E%20Result%3CAuthResponse%2C%20String%3E%20%7B%0A%20%20%20%20if%20!api_key.starts_with(%22mcp_%22)%20%7B%0A%20%20%20%20%20%20%20%20return%20Err(%22Invalid%20API%20key%22.to_string())%3B%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20let%20payload%20%3D%20json!(%7B%22sub%22%3A%22user123%22%2C%22exp%22%3A9999999999u64%7D)%3B%0A%20%20%20%20let%20token%20%3D%20format!(%22eyJhbGciOiJIUzI1NiJ9.%7B%7D.sig%22%2C%20%0A%20%20%20%20%20%20%20%20URL_SAFE_NO_PAD.encode(payload.to_string()))%3B%0A%20%20%20%20%0A%20%20%20%20Ok(AuthResponse%20%7B%0A%20%20%20%20%20%20%20%20access_token%3A%20token%2C%0A%20%20%20%20%20%20%20%20token_type%3A%20%22Bearer%22.to_string()%2C%0A%20%20%20%20%20%20%20%20expires_in%3A%20900%2C%0A%20%20%20%20%7D)%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20match%20authenticate(%22mcp_test123%22)%20%7B%0A%20%20%20%20%20%20%20%20Ok(r)%20%3D%3E%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26r).unwrap())%2C%0A%20%20%20%20%20%20%20%20Err(e)%20%3D%3E%20println!(%22Error%3A%20%7B%7D%22%2C%20e)%2C%0A%20%20%20%20%7D%0A%7D)

## Middleware Pattern

Validate tokens on every request:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
}

fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ")
}

fn validate_token(token: &str) -> Result<Claims, String> {
    // In production: use jsonwebtoken crate
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid token format".to_string());
    }

    // Decode and validate (simplified)
    // Real implementation would verify signature and expiration
    Ok(Claims {
        sub: "user-123".to_string(),
        exp: 9999999999,
    })
}

fn auth_middleware(auth_header: Option<&str>) -> Result<Claims, String> {
    let header = auth_header.ok_or("Missing Authorization header")?;
    let token = extract_bearer_token(header).ok_or("Invalid Authorization format")?;
    validate_token(token)
}

fn main() {
    // Valid token
    let result = auth_middleware(Some("Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.payload.sig"));
    println!("Valid: {:?}", result);

    // Missing header
    let result = auth_middleware(None);
    println!("Missing: {:?}", result);

    // Invalid format
    let result = auth_middleware(Some("InvalidFormat"));
    println!("Invalid: {:?}", result);
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20Claims%20%7B%20sub%3A%20String%2C%20exp%3A%20u64%20%7D%0A%0Afn%20extract_bearer_token(auth_header%3A%20%26str)%20-%3E%20Option%3C%26str%3E%20%7B%0A%20%20%20%20auth_header.strip_prefix(%22Bearer%20%22)%0A%7D%0A%0Afn%20validate_token(token%3A%20%26str)%20-%3E%20Result%3CClaims%2C%20String%3E%20%7B%0A%20%20%20%20let%20parts%3A%20Vec%3C%26str%3E%20%3D%20token.split('.').collect()%3B%0A%20%20%20%20if%20parts.len()%20!%3D%203%20%7B%20return%20Err(%22Invalid%20format%22.to_string())%3B%20%7D%0A%20%20%20%20Ok(Claims%20%7B%20sub%3A%20%22user-123%22.to_string()%2C%20exp%3A%209999999999%20%7D)%0A%7D%0A%0Afn%20auth_middleware(auth_header%3A%20Option%3C%26str%3E)%20-%3E%20Result%3CClaims%2C%20String%3E%20%7B%0A%20%20%20%20let%20header%20%3D%20auth_header.ok_or(%22Missing%20header%22)%3F%3B%0A%20%20%20%20let%20token%20%3D%20extract_bearer_token(header).ok_or(%22Invalid%20format%22)%3F%3B%0A%20%20%20%20validate_token(token)%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22Valid%3A%20%7B%3A%3F%7D%22%2C%20auth_middleware(Some(%22Bearer%20a.b.c%22)))%3B%0A%20%20%20%20println!(%22Missing%3A%20%7B%3A%3F%7D%22%2C%20auth_middleware(None))%3B%0A%20%20%20%20println!(%22Invalid%3A%20%7B%3A%3F%7D%22%2C%20auth_middleware(Some(%22Bad%22)))%3B%0A%7D)

## Security Best Practices

### Do

| Practice | Implementation |
|----------|----------------|
| Use HTTPS | TLS for all connections |
| Short token expiry | 15 minutes default |
| Secure key storage | Hash API keys with Argon2 |
| Validate on every request | Middleware pattern |

### Don't

| Anti-Pattern | Risk |
|--------------|------|
| Long-lived tokens | Increased exposure window |
| Storing plain API keys | Database breach exposure |
| Skipping validation | Unauthorized access |
| Logging tokens | Credential leakage |

## Token Expiration Handling

```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn is_token_expired(exp: u64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    now >= exp
}

fn time_until_expiry(exp: u64) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    exp as i64 - now as i64
}

fn main() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let expired_token_exp = now - 100;  // 100 seconds ago
    let valid_token_exp = now + 900;    // 15 minutes from now

    println!("Expired token: expired={}", is_token_expired(expired_token_exp));
    println!("Valid token: expired={}", is_token_expired(valid_token_exp));
    println!("Time until expiry: {} seconds", time_until_expiry(valid_token_exp));
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20std%3A%3Atime%3A%3A%7BSystemTime%2C%20UNIX_EPOCH%7D%3B%0A%0Afn%20is_token_expired(exp%3A%20u64)%20-%3E%20bool%20%7B%0A%20%20%20%20let%20now%20%3D%20SystemTime%3A%3Anow().duration_since(UNIX_EPOCH).unwrap().as_secs()%3B%0A%20%20%20%20now%20%3E%3D%20exp%0A%7D%0A%0Afn%20time_until_expiry(exp%3A%20u64)%20-%3E%20i64%20%7B%0A%20%20%20%20let%20now%20%3D%20SystemTime%3A%3Anow().duration_since(UNIX_EPOCH).unwrap().as_secs()%3B%0A%20%20%20%20exp%20as%20i64%20-%20now%20as%20i64%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20now%20%3D%20SystemTime%3A%3Anow().duration_since(UNIX_EPOCH).unwrap().as_secs()%3B%0A%20%20%20%20%0A%20%20%20%20let%20expired%20%3D%20now%20-%20100%3B%0A%20%20%20%20let%20valid%20%3D%20now%20%2B%20900%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Expired%3A%20%7B%7D%22%2C%20is_token_expired(expired))%3B%0A%20%20%20%20println!(%22Valid%3A%20%7B%7D%22%2C%20is_token_expired(valid))%3B%0A%20%20%20%20println!(%22Time%20left%3A%20%7B%7Ds%22%2C%20time_until_expiry(valid))%3B%0A%7D)

## Using with Claude CLI

```bash
# Get a token
TOKEN=$(curl -s -X POST http://localhost:12009/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"api_key": "mcp_your_key_here"}' | jq -r '.access_token')

# Configure Claude CLI
claude mcp add metamcp --transport http http://localhost:12009/mcp \
  --header "Authorization: Bearer $TOKEN"
```

---

Next: [Building an MCP Server](./07-building-server.md)
