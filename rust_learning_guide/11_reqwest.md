# Reqwest - HTTP Client

Reqwest is Rust's most popular HTTP client library, providing an ergonomic, async-first interface for making HTTP requests.

## What is Reqwest?

Reqwest provides:
- **Async HTTP client** - Built on Tokio and Hyper
- **Connection pooling** - Efficient connection reuse
- **JSON support** - Automatic serialization/deserialization
- **TLS** - Secure connections by default
- **Streaming** - Handle large responses efficiently

## Why Reqwest?

Reqwest is the go-to HTTP client because:
- Easy to use API
- Excellent async support
- Feature-rich (cookies, redirects, proxies)
- Well-maintained and widely used

## Installation

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
```

Common features:
- `json` - JSON request/response bodies
- `stream` - Streaming responses
- `cookies` - Cookie jar support
- `gzip` / `brotli` - Compression
- `rustls-tls` - TLS via rustls (default is native-tls)

## Basic Usage

### Simple GET Request

```rust
use reqwest;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let response = reqwest::get("https://api.example.com/data")
        .await?;

    let body = response.text().await?;
    println!("{}", body);

    Ok(())
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=%2F%2F%20Note%3A%20Network%20requests%20don%27t%20work%20in%20playground%0A%2F%2F%20This%20demonstrates%20the%20pattern%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22Reqwest%20GET%20example%3A%22)%3B%0A%20%20%20%20println!(%22let%20response%20%3D%20reqwest%3A%3Aget(url).await%3F%3B%22)%3B%0A%20%20%20%20println!(%22let%20body%20%3D%20response.text().await%3F%3B%22)%3B%0A%7D)

### Creating a Client

For connection pooling and custom configuration:

```rust
use reqwest::Client;
use std::time::Duration;

// Create a reusable client
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .build()
    .expect("Failed to create HTTP client");

// Use the client for multiple requests
let response = client
    .get("https://api.example.com/data")
    .send()
    .await?;
```

## Real Examples from MetaMCP

### Client Configuration

From `src/mcp/proxy.rs`:

```rust
use reqwest::Client;
use std::time::Duration;

pub struct McpProxy {
    http_client: Client,
}

impl McpProxy {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { http_client: client }
    }
}
```

### POST Request with JSON

From `src/mcp/proxy.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

async fn forward_http(
    &self,
    server: &McpServer,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse, AppError> {
    let response = self
        .http_client
        .post(&server.url)
        .json(&request)  // Serialize request to JSON
        .send()
        .await
        .map_err(|e| {
            AppError::McpProtocol(format!("Failed to connect to MCP server: {}", e))
        })?;

    // Check response status
    if !response.status().is_success() {
        return Err(AppError::McpProtocol(format!(
            "MCP server returned error status: {}",
            response.status()
        )));
    }

    // Deserialize JSON response
    let json_response: JsonRpcResponse = response.json().await.map_err(|e| {
        AppError::McpProtocol(format!("Failed to parse MCP server response: {}", e))
    })?;

    Ok(json_response)
}
```

### Test Client Example

From `examples/test_client.rs`:

```rust
use reqwest::Client;
use std::time::Duration;

struct ApiClient {
    http_client: Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    fn new(base_url: &str) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            base_url: base_url.to_string(),
            token: None,
        }
    }

    fn auth_header(&self) -> Result<String, Box<dyn std::error::Error>> {
        match &self.token {
            Some(token) => Ok(format!("Bearer {}", token)),
            None => Err("No token available".into()),
        }
    }

    async fn authenticate(&mut self, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/auth/token", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .json(&serde_json::json!({ "api_key": api_key }))
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: AuthResponse = response.json().await?;
            self.token = Some(auth_response.access_token);
            Ok(())
        } else {
            Err(format!("Authentication failed: {}", response.status()).into())
        }
    }

    async fn list_servers(&self) -> Result<Vec<McpServerInfo>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/mcp/servers", self.base_url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if response.status().is_success() {
            let servers: ListServersResponse = response.json().await?;
            Ok(servers.servers)
        } else {
            Err(format!("Failed to list servers: {}", response.status()).into())
        }
    }
}
```

## Common Patterns

### Request Methods

```rust
// GET
let response = client.get("https://api.example.com/users").send().await?;

// POST
let response = client.post("https://api.example.com/users")
    .json(&new_user)
    .send()
    .await?;

// PUT
let response = client.put("https://api.example.com/users/123")
    .json(&updated_user)
    .send()
    .await?;

// DELETE
let response = client.delete("https://api.example.com/users/123")
    .send()
    .await?;

// PATCH
let response = client.patch("https://api.example.com/users/123")
    .json(&partial_update)
    .send()
    .await?;
```

### Headers

```rust
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

// Single header
let response = client
    .get("https://api.example.com/data")
    .header("X-API-Key", "my-api-key")
    .header(AUTHORIZATION, "Bearer token123")
    .send()
    .await?;

// Multiple headers
let mut headers = HeaderMap::new();
headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
headers.insert("X-Custom-Header", HeaderValue::from_static("value"));

let response = client
    .get("https://api.example.com/data")
    .headers(headers)
    .send()
    .await?;
```

### Query Parameters

```rust
// Using query method
let response = client
    .get("https://api.example.com/search")
    .query(&[("q", "rust"), ("page", "1")])
    .send()
    .await?;

// Or build URL manually
let url = format!("https://api.example.com/search?q={}&page={}", query, page);
let response = client.get(&url).send().await?;
```

### Response Handling

```rust
let response = client.get(url).send().await?;

// Check status
if response.status().is_success() {
    // Get body as text
    let text = response.text().await?;

    // Or as bytes
    let bytes = response.bytes().await?;

    // Or parse as JSON
    let data: MyStruct = response.json().await?;
}

// Access status code
let status = response.status();
println!("Status: {}", status);

// Access headers
if let Some(content_type) = response.headers().get("content-type") {
    println!("Content-Type: {:?}", content_type);
}
```

### Error Handling

```rust
use reqwest::StatusCode;

async fn fetch_user(client: &Client, id: u64) -> Result<User, AppError> {
    let url = format!("https://api.example.com/users/{}", id);

    let response = client.get(&url).send().await?;

    match response.status() {
        StatusCode::OK => {
            let user = response.json().await?;
            Ok(user)
        }
        StatusCode::NOT_FOUND => {
            Err(AppError::NotFound(format!("User {} not found", id)))
        }
        StatusCode::UNAUTHORIZED => {
            Err(AppError::Unauthorized("Invalid token".into()))
        }
        status => {
            let body = response.text().await.unwrap_or_default();
            Err(AppError::Internal(format!(
                "Unexpected status {}: {}",
                status, body
            )))
        }
    }
}
```

[Run error handling example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=%2F%2F%20Simulated%20response%20status%20handling%0A%0Aenum%20ApiError%20%7B%0A%20%20%20%20NotFound(String)%2C%0A%20%20%20%20Unauthorized%2C%0A%20%20%20%20ServerError(String)%2C%0A%7D%0A%0Afn%20handle_status(status_code%3A%20u16%2C%20body%3A%20%26str)%20-%3E%20Result%3CString%2C%20ApiError%3E%20%7B%0A%20%20%20%20match%20status_code%20%7B%0A%20%20%20%20%20%20%20%20200%20%3D%3E%20Ok(body.to_string())%2C%0A%20%20%20%20%20%20%20%20404%20%3D%3E%20Err(ApiError%3A%3ANotFound(body.to_string()))%2C%0A%20%20%20%20%20%20%20%20401%20%3D%3E%20Err(ApiError%3A%3AUnauthorized)%2C%0A%20%20%20%20%20%20%20%20_%20%3D%3E%20Err(ApiError%3A%3AServerError(format!(%22Status%20%7B%7D%3A%20%7B%7D%22%2C%20status_code%2C%20body)))%2C%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Simulate%20different%20responses%0A%20%20%20%20let%20responses%20%3D%20vec!%5B%0A%20%20%20%20%20%20%20%20(200%2C%20%22%7B%5C%22data%5C%22%3A%20%5C%22success%5C%22%7D%22)%2C%0A%20%20%20%20%20%20%20%20(404%2C%20%22User%20not%20found%22)%2C%0A%20%20%20%20%20%20%20%20(401%2C%20%22Unauthorized%22)%2C%0A%20%20%20%20%20%20%20%20(500%2C%20%22Internal%20error%22)%2C%0A%20%20%20%20%5D%3B%0A%20%20%20%20%0A%20%20%20%20for%20(status%2C%20body)%20in%20responses%20%7B%0A%20%20%20%20%20%20%20%20match%20handle_status(status%2C%20body)%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20Ok(data)%20%3D%3E%20println!(%22Success%3A%20%7B%7D%22%2C%20data)%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20Err(e)%20%3D%3E%20println!(%22Error%3A%20%7B%3A%3F%7D%22%2C%20e)%2C%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D)

### Streaming Responses

For large files or streaming APIs:

```rust
use futures::StreamExt;

async fn download_file(client: &Client, url: &str) -> Result<Vec<u8>> {
    let response = client.get(url).send().await?;

    let mut stream = response.bytes_stream();
    let mut data = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        data.extend_from_slice(&chunk);
    }

    Ok(data)
}
```

### Timeout and Retry

```rust
use std::time::Duration;

// Per-client timeout
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(5))
    .build()?;

// Per-request timeout
let response = client
    .get(url)
    .timeout(Duration::from_secs(10))
    .send()
    .await?;

// Simple retry logic
async fn fetch_with_retry(client: &Client, url: &str, retries: u32) -> Result<String> {
    let mut last_error = None;

    for attempt in 0..retries {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => {
                return response.text().await.map_err(Into::into);
            }
            Ok(response) => {
                last_error = Some(format!("Status: {}", response.status()));
            }
            Err(e) => {
                last_error = Some(e.to_string());
            }
        }

        if attempt < retries - 1 {
            tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
        }
    }

    Err(anyhow!("Failed after {} retries: {:?}", retries, last_error))
}
```

## Client Configuration

```rust
use reqwest::Client;
use std::time::Duration;

let client = Client::builder()
    // Timeouts
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(5))

    // Connection pooling
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))

    // TLS configuration
    .danger_accept_invalid_certs(false)  // Don't disable in production!
    .min_tls_version(reqwest::tls::Version::TLS_1_2)

    // Redirect policy
    .redirect(reqwest::redirect::Policy::limited(10))

    // User agent
    .user_agent("MyApp/1.0")

    // Default headers
    .default_headers({
        let mut headers = HeaderMap::new();
        headers.insert("X-API-Version", HeaderValue::from_static("1.0"));
        headers
    })

    .build()?;
```

## Best Practices

### DO

1. **Reuse the Client** - Create once, use many times
2. **Set timeouts** - Prevent hanging requests
3. **Handle all status codes** - Not just 200
4. **Use streaming for large responses** - Memory efficiency
5. **Add error context** - Know which request failed

### DON'T

1. **Don't create a new Client per request** - Loses connection pooling
2. **Don't ignore errors** - Handle them properly
3. **Don't hardcode URLs** - Use configuration
4. **Don't disable TLS verification in production**

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Ergonomic** | Easy-to-use API |
| **Async-first** | Built for async Rust |
| **Feature-rich** | Cookies, redirects, proxies |
| **Well-maintained** | Active development |
| **Connection pooling** | Efficient by default |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Compile time** | Many dependencies |
| **Binary size** | Adds to final binary |
| **TLS options** | native-tls vs rustls choice |

## When to Use

**Use reqwest when:**
- Making HTTP requests to external services
- Building API clients
- Downloading files
- Web scraping

**Consider alternatives when:**
- Need HTTP/3 (use `h3` crate)
- Minimal binary size (use `ureq` for sync)
- Low-level control (use `hyper` directly)

## Further Learning

### Official Resources
- [Reqwest Documentation](https://docs.rs/reqwest)
- [Reqwest GitHub](https://github.com/seanmonstar/reqwest)

### Practice
1. Build an API client wrapper
2. Implement retry with exponential backoff
3. Download large files with progress
4. Handle authentication headers

## Related Crates

- **hyper** - Lower-level HTTP library (reqwest is built on it)
- **ureq** - Blocking HTTP client (simpler, smaller)
- **surf** - Alternative async client
- **isahc** - Another async HTTP client
