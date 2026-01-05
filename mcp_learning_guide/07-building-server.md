# Building an MCP Server

This guide walks through building a complete MCP server in Rust step by step.

## Project Setup

```bash
# Create new project
cargo new my-mcp-server
cd my-mcp-server

# Add dependencies to Cargo.toml
```

```toml
[package]
name = "my-mcp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower-http = { version = "0.5", features = ["cors"] }
futures = "0.3"
```

## Step 1: Define Protocol Types

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn success(id: i64, result: Value) -> Self {
        Self { jsonrpc: "2.0".to_string(), id, result: Some(result), error: None }
    }

    pub fn error(id: i64, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message: message.to_string() }),
        }
    }
}

fn main() {
    let response = JsonRpcResponse::success(1, serde_json::json!({"status": "ok"}));
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3AValue%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0Apub%20struct%20JsonRpcResponse%20%7B%0A%20%20%20%20pub%20jsonrpc%3A%20String%2C%0A%20%20%20%20pub%20id%3A%20i64%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20result%3A%20Option%3CValue%3E%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20error%3A%20Option%3CJsonRpcError%3E%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0Apub%20struct%20JsonRpcError%20%7B%20pub%20code%3A%20i32%2C%20pub%20message%3A%20String%20%7D%0A%0Aimpl%20JsonRpcResponse%20%7B%0A%20%20%20%20pub%20fn%20success(id%3A%20i64%2C%20result%3A%20Value)%20-%3E%20Self%20%7B%0A%20%20%20%20%20%20%20%20Self%20%7B%20jsonrpc%3A%20%222.0%22.to_string()%2C%20id%2C%20result%3A%20Some(result)%2C%20error%3A%20None%20%7D%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20r%20%3D%20JsonRpcResponse%3A%3Asuccess(1%2C%20serde_json%3A%3Ajson!(%7B%22status%22%3A%22ok%22%7D))%3B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26r).unwrap())%3B%0A%7D)

## Step 2: Define Tools

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "echo".to_string(),
            description: Some("Echoes back the message".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string", "description": "Message to echo" }
                },
                "required": ["message"]
            }),
        },
        Tool {
            name: "add".to_string(),
            description: Some("Adds two numbers".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "a": { "type": "number" },
                    "b": { "type": "number" }
                },
                "required": ["a", "b"]
            }),
        },
        Tool {
            name: "uppercase".to_string(),
            description: Some("Converts text to uppercase".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string" }
                },
                "required": ["text"]
            }),
        },
    ]
}

fn main() {
    let tools = get_tools();
    println!("{}", serde_json::to_string_pretty(&json!({"tools": tools})).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20Tool%20%7B%0A%20%20%20%20pub%20name%3A%20String%2C%0A%20%20%20%20pub%20description%3A%20Option%3CString%3E%2C%0A%20%20%20%20pub%20input_schema%3A%20Value%2C%0A%7D%0A%0Afn%20get_tools()%20-%3E%20Vec%3CTool%3E%20%7B%0A%20%20%20%20vec!%5B%0A%20%20%20%20%20%20%20%20Tool%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20name%3A%20%22echo%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20description%3A%20Some(%22Echoes%20back%22.to_string())%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20input_schema%3A%20json!(%7B%22type%22%3A%22object%22%2C%22properties%22%3A%7B%22message%22%3A%7B%22type%22%3A%22string%22%7D%7D%7D)%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20Tool%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20name%3A%20%22add%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20description%3A%20Some(%22Adds%20numbers%22.to_string())%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20input_schema%3A%20json!(%7B%22type%22%3A%22object%22%2C%22properties%22%3A%7B%22a%22%3A%7B%22type%22%3A%22number%22%7D%2C%22b%22%3A%7B%22type%22%3A%22number%22%7D%7D%7D)%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%5D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26json!(%7B%22tools%22%3A%20get_tools()%7D)).unwrap())%3B%0A%7D)

## Step 3: Implement Tool Execution

```rust
use serde_json::{json, Value};

fn execute_tool(name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "echo" => {
            let message = args.get("message")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'message' argument")?;
            Ok(json!({
                "content": [{"type": "text", "text": message}],
                "isError": false
            }))
        }
        "add" => {
            let a = args.get("a").and_then(|v| v.as_f64()).ok_or("Missing 'a'")?;
            let b = args.get("b").and_then(|v| v.as_f64()).ok_or("Missing 'b'")?;
            Ok(json!({
                "content": [{"type": "text", "text": format!("{}", a + b)}],
                "isError": false
            }))
        }
        "uppercase" => {
            let text = args.get("text")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'text'")?;
            Ok(json!({
                "content": [{"type": "text", "text": text.to_uppercase()}],
                "isError": false
            }))
        }
        _ => Err(format!("Unknown tool: {}", name))
    }
}

fn main() {
    // Test echo
    let result = execute_tool("echo", &json!({"message": "Hello!"}));
    println!("Echo: {:?}\n", result);

    // Test add
    let result = execute_tool("add", &json!({"a": 10, "b": 5}));
    println!("Add: {:?}\n", result);

    // Test uppercase
    let result = execute_tool("uppercase", &json!({"text": "hello world"}));
    println!("Uppercase: {:?}", result);
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0Afn%20execute_tool(name%3A%20%26str%2C%20args%3A%20%26Value)%20-%3E%20Result%3CValue%2C%20String%3E%20%7B%0A%20%20%20%20match%20name%20%7B%0A%20%20%20%20%20%20%20%20%22echo%22%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20msg%20%3D%20args.get(%22message%22).and_then(%7Cv%7C%20v.as_str()).ok_or(%22Missing%20message%22)%3F%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20Ok(json!(%7B%22content%22%3A%5B%7B%22type%22%3A%22text%22%2C%22text%22%3Amsg%7D%5D%7D))%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20%22add%22%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20a%20%3D%20args.get(%22a%22).and_then(%7Cv%7C%20v.as_f64()).ok_or(%22Missing%20a%22)%3F%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20b%20%3D%20args.get(%22b%22).and_then(%7Cv%7C%20v.as_f64()).ok_or(%22Missing%20b%22)%3F%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20Ok(json!(%7B%22content%22%3A%5B%7B%22type%22%3A%22text%22%2C%22text%22%3Aformat!(%22%7B%7D%22%2Ca%2Bb)%7D%5D%7D))%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20%22uppercase%22%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20text%20%3D%20args.get(%22text%22).and_then(%7Cv%7C%20v.as_str()).ok_or(%22Missing%20text%22)%3F%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20Ok(json!(%7B%22content%22%3A%5B%7B%22type%22%3A%22text%22%2C%22text%22%3Atext.to_uppercase()%7D%5D%7D))%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20_%20%3D%3E%20Err(format!(%22Unknown%20tool%3A%20%7B%7D%22%2C%20name))%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22Echo%3A%20%7B%3A%3F%7D%22%2C%20execute_tool(%22echo%22%2C%20%26json!(%7B%22message%22%3A%22Hi!%22%7D)))%3B%0A%20%20%20%20println!(%22Add%3A%20%7B%3A%3F%7D%22%2C%20execute_tool(%22add%22%2C%20%26json!(%7B%22a%22%3A10%2C%22b%22%3A5%7D)))%3B%0A%20%20%20%20println!(%22Upper%3A%20%7B%3A%3F%7D%22%2C%20execute_tool(%22uppercase%22%2C%20%26json!(%7B%22text%22%3A%22hello%22%7D)))%3B%0A%7D)

## Step 4: Request Handler

```rust
use serde_json::{json, Value};

const MCP_VERSION: &str = "2025-03-26";

fn handle_request(method: &str, id: i64, params: Option<&Value>) -> Value {
    match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": MCP_VERSION,
                "capabilities": {
                    "tools": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "my-mcp-server",
                    "version": "1.0.0"
                }
            }
        }),
        "tools/list" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    {"name": "echo", "description": "Echo message", "inputSchema": {"type": "object"}},
                    {"name": "add", "description": "Add numbers", "inputSchema": {"type": "object"}}
                ]
            }
        }),
        "tools/call" => {
            let params = params.unwrap_or(&json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").unwrap_or(&json!({}));

            // Execute tool (simplified)
            let result = match tool_name {
                "echo" => {
                    let msg = args.get("message").and_then(|v| v.as_str()).unwrap_or("");
                    json!({"content": [{"type": "text", "text": msg}], "isError": false})
                }
                "add" => {
                    let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    json!({"content": [{"type": "text", "text": format!("{}", a + b)}], "isError": false})
                }
                _ => json!({"content": [{"type": "text", "text": "Unknown tool"}], "isError": true})
            };

            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            })
        },
        "ping" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {}
        }),
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": "Method not found" }
        })
    }
}

fn main() {
    // Test various methods
    println!("Initialize:\n{}\n",
        serde_json::to_string_pretty(&handle_request("initialize", 1, None)).unwrap());

    println!("Tools list:\n{}\n",
        serde_json::to_string_pretty(&handle_request("tools/list", 2, None)).unwrap());

    let call_params = json!({"name": "add", "arguments": {"a": 5, "b": 3}});
    println!("Tool call:\n{}\n",
        serde_json::to_string_pretty(&handle_request("tools/call", 3, Some(&call_params))).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0Aconst%20MCP_VERSION%3A%20%26str%20%3D%20%222025-03-26%22%3B%0A%0Afn%20handle_request(method%3A%20%26str%2C%20id%3A%20i64%2C%20params%3A%20Option%3C%26Value%3E)%20-%3E%20Value%20%7B%0A%20%20%20%20match%20method%20%7B%0A%20%20%20%20%20%20%20%20%22initialize%22%20%3D%3E%20json!(%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22jsonrpc%22%3A%20%222.0%22%2C%20%22id%22%3A%20id%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%22result%22%3A%20%7B%22protocolVersion%22%3A%20MCP_VERSION%2C%20%22capabilities%22%3A%20%7B%7D%7D%0A%20%20%20%20%20%20%20%20%7D)%2C%0A%20%20%20%20%20%20%20%20%22tools%2Flist%22%20%3D%3E%20json!(%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22jsonrpc%22%3A%20%222.0%22%2C%20%22id%22%3A%20id%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%22result%22%3A%20%7B%22tools%22%3A%20%5B%7B%22name%22%3A%22echo%22%7D%5D%7D%0A%20%20%20%20%20%20%20%20%7D)%2C%0A%20%20%20%20%20%20%20%20%22tools%2Fcall%22%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20p%20%3D%20params.unwrap_or(%26json!(%7B%7D))%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20name%20%3D%20p.get(%22name%22).and_then(%7Cv%7Cv.as_str()).unwrap_or(%22%22)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20args%20%3D%20p.get(%22arguments%22).unwrap_or(%26json!(%7B%7D))%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20result%20%3D%20if%20name%20%3D%3D%20%22add%22%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20let%20a%20%3D%20args.get(%22a%22).and_then(%7Cv%7Cv.as_f64()).unwrap_or(0.0)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20let%20b%20%3D%20args.get(%22b%22).and_then(%7Cv%7Cv.as_f64()).unwrap_or(0.0)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20json!(%7B%22content%22%3A%5B%7B%22type%22%3A%22text%22%2C%22text%22%3Aformat!(%22%7B%7D%22%2Ca%2Bb)%7D%5D%7D)%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%20else%20%7B%20json!(%7B%22content%22%3A%5B%5D%7D)%20%7D%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20json!(%7B%22jsonrpc%22%3A%222.0%22%2C%22id%22%3Aid%2C%22result%22%3Aresult%7D)%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20_%20%3D%3E%20json!(%7B%22jsonrpc%22%3A%222.0%22%2C%22id%22%3Aid%2C%22error%22%3A%7B%22code%22%3A-32601%7D%7D)%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26handle_request(%22initialize%22%2C1%2CNone)).unwrap())%3B%0A%7D)

## Step 5: Complete Axum Server

```rust
// main.rs - Complete MCP Server
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response, sse::{Event, Sse}},
    routing::{get, post},
    Json, Router,
};
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{convert::Infallible, sync::Arc};
use tower_http::cors::CorsLayer;

const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

// Types
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<i64>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

// Handlers
async fn mcp_post(Json(request): Json<JsonRpcRequest>) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("mcp-protocol-version", HeaderValue::from_static(MCP_PROTOCOL_VERSION));

    // Handle notifications (no id)
    if request.id.is_none() {
        return (headers, StatusCode::ACCEPTED).into_response();
    }

    let id = request.id.unwrap();
    let response = match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "my-mcp-server", "version": "1.0.0" }
            })),
            error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({ "tools": get_tools() })),
            error: None,
        },
        "tools/call" => handle_tool_call(id, request.params),
        "ping" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        },
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        },
    };

    (headers, Json(response)).into_response()
}

async fn mcp_sse() -> impl IntoResponse {
    let initial = stream::once(async {
        Ok::<_, Infallible>(Event::default().data(json!({
            "jsonrpc": "2.0",
            "method": "endpoint",
            "params": { "uri": "/mcp" }
        }).to_string()))
    });

    let keep_alive = stream::unfold((), |_| async {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        Some((Ok::<_, Infallible>(Event::default().comment("ping")), ()))
    });

    let mut headers = HeaderMap::new();
    headers.insert("mcp-protocol-version", HeaderValue::from_static(MCP_PROTOCOL_VERSION));

    (headers, Sse::new(initial.chain(keep_alive)))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": MCP_PROTOCOL_VERSION }))
}

// Tool helpers
fn get_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "echo",
            "description": "Echo a message",
            "inputSchema": { "type": "object", "properties": { "message": { "type": "string" } } }
        }),
        json!({
            "name": "add",
            "description": "Add two numbers",
            "inputSchema": { "type": "object", "properties": { "a": { "type": "number" }, "b": { "type": "number" } } }
        }),
    ]
}

fn handle_tool_call(id: i64, params: Option<Value>) -> JsonRpcResponse {
    let params = params.unwrap_or(json!({}));
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let args = params.get("arguments").unwrap_or(&json!({}));

    let result = match name {
        "echo" => {
            let msg = args.get("message").and_then(|v| v.as_str()).unwrap_or("");
            json!({ "content": [{ "type": "text", "text": msg }], "isError": false })
        }
        "add" => {
            let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            json!({ "content": [{ "type": "text", "text": format!("{}", a + b) }], "isError": false })
        }
        _ => json!({ "content": [{ "type": "text", "text": "Unknown tool" }], "isError": true }),
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/mcp", post(mcp_post).get(mcp_sse))
        .route("/mcp/health", get(health))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("MCP Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
```

## Testing Your Server

```bash
# Health check
curl http://localhost:3000/mcp/health

# Initialize
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'

# List tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

# Call tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"add","arguments":{"a":5,"b":3}}}'
```

## See Working Examples

Check out the example servers in this repository:
- [`examples/backend_server_1.rs`](../examples/backend_server_1.rs) - Simple tools server
- [`examples/backend_server_2.rs`](../examples/backend_server_2.rs) - Advanced server with resources and prompts

---

Next: [Best Practices](./08-best-practices.md)
