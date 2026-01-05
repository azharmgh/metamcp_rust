# HTTP Transport

MCP uses HTTP as its transport layer with support for both standard request/response and Server-Sent Events (SSE) for real-time communication.

## Transport Overview

| Endpoint | Method | Purpose | Content-Type |
|----------|--------|---------|--------------|
| `/mcp` | POST | Send JSON-RPC requests | `application/json` |
| `/mcp` | GET | Open SSE stream | `text/event-stream` |
| `/mcp/health` | GET | Health check | `application/json` |

## POST: Sending Requests

All MCP requests use POST with JSON-RPC format:

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

// Simulated HTTP handler
fn handle_post(body: &str) -> String {
    let request: JsonRpcRequest = serde_json::from_str(body).unwrap();

    // Check if it's a notification (no id)
    if request.id.is_none() {
        // Notifications return 202 Accepted with no body
        return "HTTP 202 Accepted".to_string();
    }

    let id = request.id.unwrap();
    let response = match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2025-03-26",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "demo", "version": "1.0" }
            })),
            error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({ "tools": [] })),
            error: None,
        },
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
            }),
        },
    };

    serde_json::to_string_pretty(&response).unwrap()
}

fn main() {
    // Request with id
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    println!("Request:\n{}\n", request);
    println!("Response:\n{}\n", handle_post(request));

    // Notification (no id)
    let notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    println!("Notification:\n{}\n", notification);
    println!("Response: {}", handle_post(notification));
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20JsonRpcRequest%20%7B%0A%20%20%20%20jsonrpc%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20id%3A%20Option%3Ci64%3E%2C%0A%20%20%20%20method%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20params%3A%20Option%3CValue%3E%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20JsonRpcResponse%20%7B%0A%20%20%20%20jsonrpc%3A%20String%2C%0A%20%20%20%20id%3A%20i64%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20result%3A%20Option%3CValue%3E%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20error%3A%20Option%3CJsonRpcError%3E%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20JsonRpcError%20%7B%20code%3A%20i32%2C%20message%3A%20String%20%7D%0A%0Afn%20handle_post(body%3A%20%26str)%20-%3E%20String%20%7B%0A%20%20%20%20let%20request%3A%20JsonRpcRequest%20%3D%20serde_json%3A%3Afrom_str(body).unwrap()%3B%0A%20%20%20%20if%20request.id.is_none()%20%7B%20return%20%22HTTP%20202%20Accepted%22.to_string()%3B%20%7D%0A%20%20%20%20let%20id%20%3D%20request.id.unwrap()%3B%0A%20%20%20%20let%20response%20%3D%20match%20request.method.as_str()%20%7B%0A%20%20%20%20%20%20%20%20%22initialize%22%20%3D%3E%20JsonRpcResponse%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20jsonrpc%3A%20%222.0%22.to_string()%2C%20id%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20result%3A%20Some(json!(%7B%22protocolVersion%22%3A%222025-03-26%22%7D))%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20error%3A%20None%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20_%20%3D%3E%20JsonRpcResponse%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20jsonrpc%3A%20%222.0%22.to_string()%2C%20id%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20result%3A%20None%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20error%3A%20Some(JsonRpcError%20%7B%20code%3A%20-32601%2C%20message%3A%20%22Not%20found%22.to_string()%20%7D)%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20serde_json%3A%3Ato_string_pretty(%26response).unwrap()%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20req%20%3D%20r%23%22%7B%22jsonrpc%22%3A%222.0%22%2C%22id%22%3A1%2C%22method%22%3A%22initialize%22%7D%22%23%3B%0A%20%20%20%20println!(%22Response%3A%5Cn%7B%7D%22%2C%20handle_post(req))%3B%0A%7D)

## GET: SSE Stream

The GET endpoint opens a persistent SSE connection for server-to-client messages:

```
Client                              Server
  │                                   │
  │──── GET /mcp ────────────────────►│
  │     Accept: text/event-stream     │
  │                                   │
  │◄──── HTTP 200 ────────────────────│
  │      Content-Type: text/event-stream
  │                                   │
  │◄──── data: {"method":"endpoint"}──│
  │                                   │
  │◄──── : ping ──────────────────────│  (keep-alive)
  │                                   │
  │◄──── data: {...} ─────────────────│  (server messages)
  │                                   │
```

### SSE Format

```
data: {"jsonrpc":"2.0","method":"endpoint","params":{"uri":"/mcp"}}

: ping

data: {"jsonrpc":"2.0","method":"notifications/tools/list_changed"}
```

## Required Headers

### Request Headers (Client → Server)

| Header | Value | Required |
|--------|-------|----------|
| `Content-Type` | `application/json` | Yes (POST) |
| `Accept` | `application/json, text/event-stream` | Yes |
| `Authorization` | `Bearer <token>` | If authenticated |
| `MCP-Protocol-Version` | `2025-03-26` | Recommended |

### Response Headers (Server → Client)

| Header | Value | Required |
|--------|-------|----------|
| `Content-Type` | `application/json` or `text/event-stream` | Yes |
| `mcp-protocol-version` | `2025-03-26` | Yes |
| `Cache-Control` | `no-cache` | For SSE |

## Implementing with Axum

Here's how MetaMCP implements the HTTP transport:

```rust
// Simplified from src/api/handlers/mcp_gateway.rs

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json,
};
use futures::stream::{self, StreamExt};
use std::convert::Infallible;

const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

// POST handler for JSON-RPC requests
async fn mcp_post_handler(
    Json(request): Json<serde_json::Value>,
) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        "mcp-protocol-version",
        HeaderValue::from_static(MCP_PROTOCOL_VERSION),
    );

    // Check if notification (no id)
    if request.get("id").is_none() {
        return (headers, StatusCode::ACCEPTED).into_response();
    }

    // Process request and return response
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request.get("id"),
        "result": {}
    });

    (headers, Json(response)).into_response()
}

// GET handler for SSE stream
async fn mcp_sse_handler() -> impl IntoResponse {
    let endpoint_msg = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "endpoint",
        "params": { "uri": "/mcp" }
    });

    // Initial message
    let initial = stream::once(async move {
        Ok::<_, Infallible>(Event::default().data(endpoint_msg.to_string()))
    });

    // Keep-alive pings every 30 seconds
    let keep_alive = stream::unfold((), |_| async {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        Some((Ok::<_, Infallible>(Event::default().comment("ping")), ()))
    });

    let stream = initial.chain(keep_alive);

    let mut headers = HeaderMap::new();
    headers.insert(
        "mcp-protocol-version",
        HeaderValue::from_static(MCP_PROTOCOL_VERSION),
    );

    (headers, Sse::new(stream))
}
```

Source: [`src/api/handlers/mcp_gateway.rs`](../src/api/handlers/mcp_gateway.rs)

## Health Check

The health endpoint is crucial for clients to verify server availability:

```rust
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

fn health_check() -> HealthResponse {
    HealthResponse {
        status: "ok".to_string(),
        version: "2025-03-26".to_string(),
    }
}

fn main() {
    let health = health_check();
    println!("{}", serde_json::to_string_pretty(&health).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3ASerialize%3B%0A%0A%23%5Bderive(Serialize)%5D%0Astruct%20HealthResponse%20%7B%0A%20%20%20%20status%3A%20String%2C%0A%20%20%20%20version%3A%20String%2C%0A%7D%0A%0Afn%20health_check()%20-%3E%20HealthResponse%20%7B%0A%20%20%20%20HealthResponse%20%7B%0A%20%20%20%20%20%20%20%20status%3A%20%22ok%22.to_string()%2C%0A%20%20%20%20%20%20%20%20version%3A%20%222025-03-26%22.to_string()%2C%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20health%20%3D%20health_check()%3B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26health).unwrap())%3B%0A%7D)

## Testing with curl

```bash
# Health check
curl http://localhost:12009/mcp/health

# Initialize
curl -X POST http://localhost:12009/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'

# SSE stream (will stay open)
curl -N http://localhost:12009/mcp \
  -H "Accept: text/event-stream" \
  -H "Authorization: Bearer $TOKEN"
```

## Dos and Don'ts

### Do

| Practice | Reason |
|----------|--------|
| Include `mcp-protocol-version` header | Required for protocol negotiation |
| Return 202 for notifications | Notifications don't expect body |
| Keep SSE connections alive | Use ping/keep-alive |
| Validate `Content-Type` | Ensure correct request format |

### Don't

| Anti-Pattern | Why |
|--------------|-----|
| Close SSE immediately | Clients expect persistent connection |
| Return JSON for notifications | Return 202 Accepted with no body |
| Ignore protocol version | May cause compatibility issues |
| Block on long operations | Use async/await properly |

## Error Handling

```rust
use serde_json::json;

fn create_error_response(id: i64, code: i32, message: &str) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })).unwrap()
}

fn main() {
    // Standard error codes
    println!("Parse error: {}", create_error_response(1, -32700, "Parse error"));
    println!("Method not found: {}", create_error_response(2, -32601, "Method not found"));
    println!("Invalid params: {}", create_error_response(3, -32602, "Invalid params"));
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3Ajson%3B%0A%0Afn%20create_error_response(id%3A%20i64%2C%20code%3A%20i32%2C%20message%3A%20%26str)%20-%3E%20String%20%7B%0A%20%20%20%20serde_json%3A%3Ato_string(%26json!(%7B%0A%20%20%20%20%20%20%20%20%22jsonrpc%22%3A%20%222.0%22%2C%0A%20%20%20%20%20%20%20%20%22id%22%3A%20id%2C%0A%20%20%20%20%20%20%20%20%22error%22%3A%20%7B%20%22code%22%3A%20code%2C%20%22message%22%3A%20message%20%7D%0A%20%20%20%20%7D)).unwrap()%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22Parse%20error%3A%20%7B%7D%22%2C%20create_error_response(1%2C%20-32700%2C%20%22Parse%20error%22))%3B%0A%20%20%20%20println!(%22Method%20not%20found%3A%20%7B%7D%22%2C%20create_error_response(2%2C%20-32601%2C%20%22Method%20not%20found%22))%3B%0A%7D)

---

Next: [Authentication](./06-authentication.md)
