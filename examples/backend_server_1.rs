//! Backend MCP Server #1 - Simple Tools Server
//!
//! A simple MCP backend server that provides basic tools:
//! - echo: Echoes back the input message
//! - add: Adds two numbers
//! - uppercase: Converts text to uppercase
//! - reverse: Reverses a string
//! - timestamp: Returns current timestamp
//!
//! This server implements the MCP protocol over HTTP for testing purposes.
//!
//! Usage:
//!   cargo run --example backend_server_1 -- --port 3001
//!
//! The server will listen on http://localhost:3001

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;

/// JSON-RPC version
const JSONRPC_VERSION: &str = "2.0";

/// MCP Protocol version
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Server info
const SERVER_NAME: &str = "backend-server-1";
const SERVER_VERSION: &str = "1.0.0";

/// JSON-RPC Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

/// JSON-RPC Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}

/// Tool definition
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Tool {
    name: String,
    description: String,
    input_schema: Value,
}

/// Content item for tool results
#[derive(Debug, Serialize)]
struct TextContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Server state
struct ServerState {
    name: String,
    version: String,
}

impl ServerState {
    fn new() -> Self {
        Self {
            name: SERVER_NAME.to_string(),
            version: SERVER_VERSION.to_string(),
        }
    }
}

/// Get available tools
fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "echo".to_string(),
            description: "Echoes back the input message".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to echo back"
                    }
                },
                "required": ["message"]
            }),
        },
        Tool {
            name: "add".to_string(),
            description: "Adds two numbers together".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "a": {
                        "type": "number",
                        "description": "First number"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second number"
                    }
                },
                "required": ["a", "b"]
            }),
        },
        Tool {
            name: "uppercase".to_string(),
            description: "Converts text to uppercase".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to convert to uppercase"
                    }
                },
                "required": ["text"]
            }),
        },
        Tool {
            name: "reverse".to_string(),
            description: "Reverses a string".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to reverse"
                    }
                },
                "required": ["text"]
            }),
        },
        Tool {
            name: "timestamp".to_string(),
            description: "Returns the current Unix timestamp".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

/// Execute a tool
fn execute_tool(name: &str, arguments: &Value) -> Result<Value, String> {
    match name {
        "echo" => {
            let message = arguments
                .get("message")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'message' argument")?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": message
                }]
            }))
        }
        "add" => {
            let a = arguments
                .get("a")
                .and_then(|v| v.as_f64())
                .ok_or("Missing or invalid 'a' argument")?;
            let b = arguments
                .get("b")
                .and_then(|v| v.as_f64())
                .ok_or("Missing or invalid 'b' argument")?;
            let result = a + b;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("{}", result)
                }]
            }))
        }
        "uppercase" => {
            let text = arguments
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'text' argument")?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text.to_uppercase()
                }]
            }))
        }
        "reverse" => {
            let text = arguments
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'text' argument")?;
            let reversed: String = text.chars().rev().collect();
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": reversed
                }]
            }))
        }
        "timestamp" => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("{}", now)
                }]
            }))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

/// Handle JSON-RPC requests
async fn handle_rpc(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    println!(
        "[{}] Received request: {} (id: {})",
        state.name, request.method, request.id
    );

    let response = match request.method.as_str() {
        "initialize" => {
            // Handle initialize request
            JsonRpcResponse::success(
                request.id,
                json!({
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": state.name,
                        "version": state.version
                    }
                }),
            )
        }
        "initialized" => {
            // Handle initialized notification (should not have a response, but we're lenient)
            JsonRpcResponse::success(request.id, json!({}))
        }
        "tools/list" => {
            let tools = get_tools();
            JsonRpcResponse::success(request.id, json!({ "tools": tools }))
        }
        "tools/call" => {
            let params = request.params.unwrap_or(json!({}));
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            match execute_tool(tool_name, &arguments) {
                Ok(result) => JsonRpcResponse::success(request.id, result),
                Err(e) => JsonRpcResponse::error(request.id, -32000, &e),
            }
        }
        "resources/list" => {
            // This server doesn't provide resources
            JsonRpcResponse::success(request.id, json!({ "resources": [] }))
        }
        "prompts/list" => {
            // This server doesn't provide prompts
            JsonRpcResponse::success(request.id, json!({ "prompts": [] }))
        }
        "ping" => JsonRpcResponse::success(request.id, json!({})),
        _ => JsonRpcResponse::error(
            request.id,
            -32601,
            &format!("Method not found: {}", request.method),
        ),
    };

    Json(response)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Parse command line arguments
fn parse_args() -> u16 {
    let args: Vec<String> = std::env::args().collect();
    let mut port = 3001u16;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(3001);
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("Backend MCP Server #1 - Simple Tools Server");
                println!();
                println!("Usage: backend_server_1 [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -p, --port <PORT>  Port to listen on (default: 3001)");
                println!("  -h, --help         Show this help message");
                println!();
                println!("Available tools:");
                println!("  echo      - Echoes back the input message");
                println!("  add       - Adds two numbers together");
                println!("  uppercase - Converts text to uppercase");
                println!("  reverse   - Reverses a string");
                println!("  timestamp - Returns current Unix timestamp");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    port
}

#[tokio::main]
async fn main() {
    let port = parse_args();
    let state = Arc::new(ServerState::new());

    println!("===========================================");
    println!("  Backend MCP Server #1 - Simple Tools");
    println!("===========================================");
    println!();
    println!("Server: {}", SERVER_NAME);
    println!("Version: {}", SERVER_VERSION);
    println!("Protocol: MCP {}", MCP_PROTOCOL_VERSION);
    println!();
    println!("Available tools:");
    for tool in get_tools() {
        println!("  - {}: {}", tool.name, tool.description);
    }
    println!();

    let app = Router::new()
        .route("/", post(handle_rpc))
        .route("/health", axum::routing::get(health_check))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("Listening on http://{}", addr);
    println!();

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_tool() {
        let args = json!({"message": "Hello, World!"});
        let result = execute_tool("echo", &args).unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_add_tool() {
        let args = json!({"a": 5, "b": 3});
        let result = execute_tool("add", &args).unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "8");
    }

    #[test]
    fn test_uppercase_tool() {
        let args = json!({"text": "hello"});
        let result = execute_tool("uppercase", &args).unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "HELLO");
    }

    #[test]
    fn test_reverse_tool() {
        let args = json!({"text": "hello"});
        let result = execute_tool("reverse", &args).unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "olleh");
    }

    #[test]
    fn test_timestamp_tool() {
        let args = json!({});
        let result = execute_tool("timestamp", &args).unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        let timestamp: u64 = text.parse().unwrap();
        assert!(timestamp > 0);
    }

    #[test]
    fn test_unknown_tool() {
        let args = json!({});
        let result = execute_tool("unknown", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_tools() {
        let tools = get_tools();
        assert_eq!(tools.len(), 5);
        assert!(tools.iter().any(|t| t.name == "echo"));
        assert!(tools.iter().any(|t| t.name == "add"));
        assert!(tools.iter().any(|t| t.name == "uppercase"));
        assert!(tools.iter().any(|t| t.name == "reverse"));
        assert!(tools.iter().any(|t| t.name == "timestamp"));
    }
}
