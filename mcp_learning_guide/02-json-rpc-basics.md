# JSON-RPC Basics

MCP is built on top of **JSON-RPC 2.0**, a lightweight remote procedure call protocol. Understanding JSON-RPC is essential for implementing MCP.

## What is JSON-RPC?

JSON-RPC is a stateless, transport-agnostic protocol that uses JSON for encoding. It defines three types of messages:

| Message Type | Has ID? | Expects Response? |
|--------------|---------|-------------------|
| **Request** | Yes | Yes |
| **Notification** | No | No |
| **Response** | Yes (matches request) | N/A |

## Message Structures

### Request

A request expects a response from the server.

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,        // Always "2.0"
    id: i64,                // Unique identifier
    method: String,         // Method to call
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,  // Optional parameters
}

fn main() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 42,
        method: "tools/list".to_string(),
        params: None,
    };

    println!("{}", serde_json::to_string_pretty(&request).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3AValue%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20JsonRpcRequest%20%7B%0A%20%20%20%20jsonrpc%3A%20String%2C%0A%20%20%20%20id%3A%20i64%2C%0A%20%20%20%20method%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20params%3A%20Option%3CValue%3E%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20request%20%3D%20JsonRpcRequest%20%7B%0A%20%20%20%20%20%20%20%20jsonrpc%3A%20%222.0%22.to_string()%2C%0A%20%20%20%20%20%20%20%20id%3A%2042%2C%0A%20%20%20%20%20%20%20%20method%3A%20%22tools%2Flist%22.to_string()%2C%0A%20%20%20%20%20%20%20%20params%3A%20None%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26request).unwrap())%3B%0A%7D)

### Notification

A notification is a request without an `id`. The server should NOT respond.

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
    // Note: NO id field!
}

fn main() {
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "notifications/initialized".to_string(),
        params: None,
    };

    println!("{}", serde_json::to_string_pretty(&notification).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3AValue%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20JsonRpcNotification%20%7B%0A%20%20%20%20jsonrpc%3A%20String%2C%0A%20%20%20%20method%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20params%3A%20Option%3CValue%3E%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20notification%20%3D%20JsonRpcNotification%20%7B%0A%20%20%20%20%20%20%20%20jsonrpc%3A%20%222.0%22.to_string()%2C%0A%20%20%20%20%20%20%20%20method%3A%20%22notifications%2Finitialized%22.to_string()%2C%0A%20%20%20%20%20%20%20%20params%3A%20None%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26notification).unwrap())%3B%0A%7D)

### Response

A response is sent back for each request. It contains either `result` OR `error`, never both.

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

fn main() {
    // Success response
    let success = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 42,
        result: Some(json!({"tools": []})),
        error: None,
    };

    // Error response
    let error = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 43,
        result: None,
        error: Some(JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }),
    };

    println!("Success:\n{}\n", serde_json::to_string_pretty(&success).unwrap());
    println!("Error:\n{}", serde_json::to_string_pretty(&error).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20JsonRpcResponse%20%7B%0A%20%20%20%20jsonrpc%3A%20String%2C%0A%20%20%20%20id%3A%20i64%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20result%3A%20Option%3CValue%3E%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20error%3A%20Option%3CJsonRpcError%3E%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20JsonRpcError%20%7B%0A%20%20%20%20code%3A%20i32%2C%0A%20%20%20%20message%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20data%3A%20Option%3CValue%3E%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20success%20%3D%20JsonRpcResponse%20%7B%0A%20%20%20%20%20%20%20%20jsonrpc%3A%20%222.0%22.to_string()%2C%0A%20%20%20%20%20%20%20%20id%3A%2042%2C%0A%20%20%20%20%20%20%20%20result%3A%20Some(json!(%7B%22tools%22%3A%20%5B%5D%7D))%2C%0A%20%20%20%20%20%20%20%20error%3A%20None%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20let%20error%20%3D%20JsonRpcResponse%20%7B%0A%20%20%20%20%20%20%20%20jsonrpc%3A%20%222.0%22.to_string()%2C%0A%20%20%20%20%20%20%20%20id%3A%2043%2C%0A%20%20%20%20%20%20%20%20result%3A%20None%2C%0A%20%20%20%20%20%20%20%20error%3A%20Some(JsonRpcError%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20code%3A%20-32601%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20message%3A%20%22Method%20not%20found%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20data%3A%20None%2C%0A%20%20%20%20%20%20%20%20%7D)%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Success%3A%5Cn%7B%7D%5Cn%22%2C%20serde_json%3A%3Ato_string_pretty(%26success).unwrap())%3B%0A%20%20%20%20println!(%22Error%3A%5Cn%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26error).unwrap())%3B%0A%7D)

## Standard Error Codes

JSON-RPC defines standard error codes:

| Code | Message | Meaning |
|------|---------|---------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Not a valid request object |
| -32601 | Method not found | Method doesn't exist |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Internal JSON-RPC error |

## Handling Both Requests and Notifications

In MCP, the same endpoint receives both requests (with id) and notifications (without id). Here's how to handle both:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Use Option<i64> for id to handle both cases
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcMessage {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,  // None for notifications
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

fn handle_message(msg: &JsonRpcMessage) -> Option<String> {
    match &msg.id {
        Some(id) => {
            // This is a request - we MUST respond
            println!("Request {} for method: {}", id, msg.method);
            Some(format!(r#"{{"jsonrpc":"2.0","id":{},"result":{{}}}}"#, id))
        }
        None => {
            // This is a notification - we MUST NOT respond
            println!("Notification for method: {}", msg.method);
            None
        }
    }
}

fn main() {
    let request = JsonRpcMessage {
        jsonrpc: "2.0".to_string(),
        id: Some(1),
        method: "initialize".to_string(),
        params: None,
    };

    let notification = JsonRpcMessage {
        jsonrpc: "2.0".to_string(),
        id: None,
        method: "notifications/initialized".to_string(),
        params: None,
    };

    if let Some(response) = handle_message(&request) {
        println!("Response: {}", response);
    }

    if let Some(response) = handle_message(&notification) {
        println!("Response: {}", response);
    } else {
        println!("No response needed for notification");
    }
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3AValue%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20JsonRpcMessage%20%7B%0A%20%20%20%20jsonrpc%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20id%3A%20Option%3Ci64%3E%2C%0A%20%20%20%20method%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20params%3A%20Option%3CValue%3E%2C%0A%7D%0A%0Afn%20handle_message(msg%3A%20%26JsonRpcMessage)%20-%3E%20Option%3CString%3E%20%7B%0A%20%20%20%20match%20%26msg.id%20%7B%0A%20%20%20%20%20%20%20%20Some(id)%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Request%20%7B%7D%20for%20method%3A%20%7B%7D%22%2C%20id%2C%20msg.method)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20Some(format!(r%23%22%7B%7B%22jsonrpc%22%3A%222.0%22%2C%22id%22%3A%7B%7D%2C%22result%22%3A%7B%7D%7D%7D%22%23%2C%20id))%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20None%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Notification%20for%20method%3A%20%7B%7D%22%2C%20msg.method)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20None%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20request%20%3D%20JsonRpcMessage%20%7B%0A%20%20%20%20%20%20%20%20jsonrpc%3A%20%222.0%22.to_string()%2C%0A%20%20%20%20%20%20%20%20id%3A%20Some(1)%2C%0A%20%20%20%20%20%20%20%20method%3A%20%22initialize%22.to_string()%2C%0A%20%20%20%20%20%20%20%20params%3A%20None%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20let%20notification%20%3D%20JsonRpcMessage%20%7B%0A%20%20%20%20%20%20%20%20jsonrpc%3A%20%222.0%22.to_string()%2C%0A%20%20%20%20%20%20%20%20id%3A%20None%2C%0A%20%20%20%20%20%20%20%20method%3A%20%22notifications%2Finitialized%22.to_string()%2C%0A%20%20%20%20%20%20%20%20params%3A%20None%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20if%20let%20Some(response)%20%3D%20handle_message(%26request)%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Response%3A%20%7B%7D%22%2C%20response)%3B%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20if%20let%20Some(response)%20%3D%20handle_message(%26notification)%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Response%3A%20%7B%7D%22%2C%20response)%3B%0A%20%20%20%20%7D%20else%20%7B%0A%20%20%20%20%20%20%20%20println!(%22No%20response%20needed%20for%20notification%22)%3B%0A%20%20%20%20%7D%0A%7D)

## Dos and Don'ts

### Do

- Always include `jsonrpc: "2.0"` in every message
- Match response `id` exactly to request `id`
- Return either `result` OR `error`, never both
- Use standard error codes when applicable

### Don't

- Don't respond to notifications (messages without id)
- Don't include `id` in notifications you send
- Don't use `null` for `id` - omit it entirely for notifications
- Don't include both `result` and `error` in a response

## Real-World Example from MetaMCP

Here's how MetaMCP handles the request/notification distinction:

```rust
// From src/api/handlers/mcp_gateway.rs (simplified)

// Handle notifications (no id) - they don't expect a response
if request.id.is_none() {
    match request.method.as_str() {
        "initialized" | "notifications/cancelled" => {
            // Return HTTP 202 Accepted for notifications
            return Ok(StatusCode::ACCEPTED);
        }
        _ => {
            return Ok(StatusCode::ACCEPTED);
        }
    }
}

// For requests with id, process and return response
let id = request.id.unwrap();
let response = match request.method.as_str() {
    "initialize" => handle_initialize(id).await,
    "tools/list" => handle_tools_list(id).await,
    // ... other methods
    _ => JsonRpcResponse::error(id, -32601, "Method not found"),
};
```

Source: [`src/api/handlers/mcp_gateway.rs`](../src/api/handlers/mcp_gateway.rs)

---

Next: [Protocol Types](./03-protocol-types.md)
