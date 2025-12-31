//! Backend MCP Server #2 - Advanced Server with Resources and Prompts
//!
//! A more comprehensive MCP backend server that provides:
//! - Tools: file operations, JSON parsing, base64 encoding/decoding
//! - Resources: virtual file system, configuration values
//! - Prompts: code review, summarization templates
//!
//! This server demonstrates the full MCP protocol capabilities.
//!
//! Usage:
//!   cargo run --example backend_server_2 -- --port 3002
//!
//! The server will listen on http://localhost:3002

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

/// JSON-RPC version
const JSONRPC_VERSION: &str = "2.0";

/// MCP Protocol version
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Server info
const SERVER_NAME: &str = "backend-server-2";
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

/// Resource definition
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Resource {
    uri: String,
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
}

/// Prompt definition
#[derive(Debug, Serialize)]
struct Prompt {
    name: String,
    description: String,
    arguments: Vec<PromptArgument>,
}

/// Prompt argument
#[derive(Debug, Serialize)]
struct PromptArgument {
    name: String,
    description: String,
    required: bool,
}

/// Server state with virtual file system
struct ServerState {
    name: String,
    version: String,
    virtual_files: RwLock<HashMap<String, String>>,
    config: HashMap<String, String>,
}

impl ServerState {
    fn new() -> Self {
        let mut virtual_files = HashMap::new();
        virtual_files.insert(
            "readme.txt".to_string(),
            "Welcome to Backend Server #2!\n\nThis is a virtual file system for testing.".to_string(),
        );
        virtual_files.insert(
            "config.json".to_string(),
            r#"{"version": "1.0", "debug": true}"#.to_string(),
        );
        virtual_files.insert(
            "notes.md".to_string(),
            "# Notes\n\n- Item 1\n- Item 2\n- Item 3".to_string(),
        );

        let mut config = HashMap::new();
        config.insert("server.name".to_string(), SERVER_NAME.to_string());
        config.insert("server.version".to_string(), SERVER_VERSION.to_string());
        config.insert("debug.enabled".to_string(), "true".to_string());
        config.insert("max_connections".to_string(), "100".to_string());

        Self {
            name: SERVER_NAME.to_string(),
            version: SERVER_VERSION.to_string(),
            virtual_files: RwLock::new(virtual_files),
            config,
        }
    }
}

/// Get available tools
fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "read_file".to_string(),
            description: "Reads a file from the virtual file system".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    }
                },
                "required": ["path"]
            }),
        },
        Tool {
            name: "write_file".to_string(),
            description: "Writes content to a file in the virtual file system".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        Tool {
            name: "list_files".to_string(),
            description: "Lists all files in the virtual file system".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "parse_json".to_string(),
            description: "Parses a JSON string and returns formatted output".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "json_string": {
                        "type": "string",
                        "description": "JSON string to parse"
                    }
                },
                "required": ["json_string"]
            }),
        },
        Tool {
            name: "base64_encode".to_string(),
            description: "Encodes text to base64".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to encode"
                    }
                },
                "required": ["text"]
            }),
        },
        Tool {
            name: "base64_decode".to_string(),
            description: "Decodes base64 to text".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "encoded": {
                        "type": "string",
                        "description": "Base64 encoded string"
                    }
                },
                "required": ["encoded"]
            }),
        },
        Tool {
            name: "word_count".to_string(),
            description: "Counts words, characters, and lines in text".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to analyze"
                    }
                },
                "required": ["text"]
            }),
        },
        Tool {
            name: "get_config".to_string(),
            description: "Gets a configuration value".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Configuration key"
                    }
                },
                "required": ["key"]
            }),
        },
    ]
}

/// Get available resources
fn get_resources(state: &ServerState) -> Vec<Resource> {
    let mut resources = vec![
        Resource {
            uri: "config://server".to_string(),
            name: "Server Configuration".to_string(),
            description: "Server configuration values".to_string(),
            mime_type: Some("application/json".to_string()),
        },
        Resource {
            uri: "config://all".to_string(),
            name: "All Configuration".to_string(),
            description: "All configuration key-value pairs".to_string(),
            mime_type: Some("application/json".to_string()),
        },
    ];

    // Add virtual files as resources
    if let Ok(files) = state.virtual_files.try_read() {
        for (name, _) in files.iter() {
            let mime_type = if name.ends_with(".json") {
                Some("application/json".to_string())
            } else if name.ends_with(".md") {
                Some("text/markdown".to_string())
            } else {
                Some("text/plain".to_string())
            };

            resources.push(Resource {
                uri: format!("file://{}", name),
                name: name.clone(),
                description: format!("Virtual file: {}", name),
                mime_type,
            });
        }
    }

    resources
}

/// Get available prompts
fn get_prompts() -> Vec<Prompt> {
    vec![
        Prompt {
            name: "code_review".to_string(),
            description: "Generate a code review for the provided code".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "code".to_string(),
                    description: "The code to review".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "language".to_string(),
                    description: "Programming language of the code".to_string(),
                    required: false,
                },
            ],
        },
        Prompt {
            name: "summarize".to_string(),
            description: "Summarize the provided text".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "text".to_string(),
                    description: "The text to summarize".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "max_length".to_string(),
                    description: "Maximum length of summary in words".to_string(),
                    required: false,
                },
            ],
        },
        Prompt {
            name: "explain".to_string(),
            description: "Explain a concept in simple terms".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "topic".to_string(),
                    description: "The topic to explain".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "audience".to_string(),
                    description: "Target audience (beginner, intermediate, expert)".to_string(),
                    required: false,
                },
            ],
        },
    ]
}

/// Execute a tool
async fn execute_tool(state: &Arc<ServerState>, name: &str, arguments: &Value) -> Result<Value, String> {
    match name {
        "read_file" => {
            let path = arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path' argument")?;

            let files = state.virtual_files.read().await;
            match files.get(path) {
                Some(content) => Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": content
                    }]
                })),
                None => Err(format!("File not found: {}", path)),
            }
        }
        "write_file" => {
            let path = arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path' argument")?;
            let content = arguments
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'content' argument")?;

            let mut files = state.virtual_files.write().await;
            files.insert(path.to_string(), content.to_string());

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("File '{}' written successfully ({} bytes)", path, content.len())
                }]
            }))
        }
        "list_files" => {
            let files = state.virtual_files.read().await;
            let file_list: Vec<String> = files.keys().cloned().collect();

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": file_list.join("\n")
                }]
            }))
        }
        "parse_json" => {
            let json_string = arguments
                .get("json_string")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'json_string' argument")?;

            match serde_json::from_str::<Value>(json_string) {
                Ok(parsed) => Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&parsed).unwrap_or_default()
                    }]
                })),
                Err(e) => Err(format!("Invalid JSON: {}", e)),
            }
        }
        "base64_encode" => {
            let text = arguments
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'text' argument")?;

            use std::io::Write;
            let mut buf = Vec::new();
            {
                let mut encoder = Base64Encoder::new(&mut buf);
                encoder.write_all(text.as_bytes()).unwrap();
                encoder.finish().unwrap();
            }
            let encoded = String::from_utf8(buf).unwrap();

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": encoded
                }]
            }))
        }
        "base64_decode" => {
            let encoded = arguments
                .get("encoded")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'encoded' argument")?;

            match base64_decode(encoded) {
                Ok(decoded) => Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": decoded
                    }]
                })),
                Err(e) => Err(format!("Invalid base64: {}", e)),
            }
        }
        "word_count" => {
            let text = arguments
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'text' argument")?;

            let words = text.split_whitespace().count();
            let chars = text.chars().count();
            let lines = text.lines().count();

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Words: {}\nCharacters: {}\nLines: {}", words, chars, lines)
                }]
            }))
        }
        "get_config" => {
            let key = arguments
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'key' argument")?;

            match state.config.get(key) {
                Some(value) => Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": value
                    }]
                })),
                None => Err(format!("Configuration key not found: {}", key)),
            }
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

/// Read a resource
async fn read_resource(state: &Arc<ServerState>, uri: &str) -> Result<Value, String> {
    if uri == "config://server" {
        return Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": json!({
                    "name": state.name,
                    "version": state.version
                }).to_string()
            }]
        }));
    }

    if uri == "config://all" {
        return Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(&state.config).unwrap_or_default()
            }]
        }));
    }

    if uri.starts_with("file://") {
        let path = &uri[7..];
        let files = state.virtual_files.read().await;

        match files.get(path) {
            Some(content) => {
                let mime_type = if path.ends_with(".json") {
                    "application/json"
                } else if path.ends_with(".md") {
                    "text/markdown"
                } else {
                    "text/plain"
                };

                return Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": mime_type,
                        "text": content
                    }]
                }));
            }
            None => return Err(format!("Resource not found: {}", uri)),
        }
    }

    Err(format!("Unknown resource URI: {}", uri))
}

/// Get a prompt
fn get_prompt(name: &str, arguments: &Value) -> Result<Value, String> {
    match name {
        "code_review" => {
            let code = arguments
                .get("code")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'code' argument")?;
            let language = arguments
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            Ok(json!({
                "messages": [{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Please review the following {} code and provide feedback on:\n\
                            1. Code quality\n\
                            2. Potential bugs\n\
                            3. Performance issues\n\
                            4. Best practices\n\n\
                            Code:\n```{}\n{}\n```",
                            language, language, code
                        )
                    }
                }]
            }))
        }
        "summarize" => {
            let text = arguments
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'text' argument")?;
            let max_length = arguments
                .get("max_length")
                .and_then(|v| v.as_u64())
                .unwrap_or(100);

            Ok(json!({
                "messages": [{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Please summarize the following text in {} words or less:\n\n{}",
                            max_length, text
                        )
                    }
                }]
            }))
        }
        "explain" => {
            let topic = arguments
                .get("topic")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'topic' argument")?;
            let audience = arguments
                .get("audience")
                .and_then(|v| v.as_str())
                .unwrap_or("beginner");

            Ok(json!({
                "messages": [{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Explain '{}' in simple terms for a {} audience.",
                            topic, audience
                        )
                    }
                }]
            }))
        }
        _ => Err(format!("Unknown prompt: {}", name)),
    }
}

/// Simple base64 encoder
struct Base64Encoder<W: std::io::Write> {
    writer: W,
    buffer: Vec<u8>,
}

impl<W: std::io::Write> Base64Encoder<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: Vec::new(),
        }
    }

    fn finish(mut self) -> std::io::Result<()> {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let chunks = self.buffer.chunks(3);
        for chunk in chunks {
            let mut n = 0u32;
            for (i, &byte) in chunk.iter().enumerate() {
                n |= (byte as u32) << (16 - i * 8);
            }

            let padding = 3 - chunk.len();
            for i in 0..(4 - padding) {
                let idx = ((n >> (18 - i * 6)) & 0x3F) as usize;
                self.writer.write_all(&[ALPHABET[idx]])?;
            }
            for _ in 0..padding {
                self.writer.write_all(b"=")?;
            }
        }
        Ok(())
    }
}

impl<W: std::io::Write> std::io::Write for Base64Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Simple base64 decoder
fn base64_decode(encoded: &str) -> Result<String, &'static str> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let encoded = encoded.trim_end_matches('=');
    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;

    for c in encoded.bytes() {
        let value = ALPHABET.iter().position(|&x| x == c)
            .ok_or("Invalid base64 character")? as u32;
        buffer = (buffer << 6) | value;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    String::from_utf8(result).map_err(|_| "Invalid UTF-8 in decoded data")
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
            JsonRpcResponse::success(
                request.id,
                json!({
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        },
                        "resources": {
                            "subscribe": false,
                            "listChanged": false
                        },
                        "prompts": {
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

            match execute_tool(&state, tool_name, &arguments).await {
                Ok(result) => JsonRpcResponse::success(request.id, result),
                Err(e) => JsonRpcResponse::error(request.id, -32000, &e),
            }
        }
        "resources/list" => {
            let resources = get_resources(&state);
            JsonRpcResponse::success(request.id, json!({ "resources": resources }))
        }
        "resources/read" => {
            let params = request.params.unwrap_or(json!({}));
            let uri = params
                .get("uri")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match read_resource(&state, uri).await {
                Ok(result) => JsonRpcResponse::success(request.id, result),
                Err(e) => JsonRpcResponse::error(request.id, -32000, &e),
            }
        }
        "prompts/list" => {
            let prompts = get_prompts();
            JsonRpcResponse::success(request.id, json!({ "prompts": prompts }))
        }
        "prompts/get" => {
            let params = request.params.unwrap_or(json!({}));
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            match get_prompt(name, &arguments) {
                Ok(result) => JsonRpcResponse::success(request.id, result),
                Err(e) => JsonRpcResponse::error(request.id, -32000, &e),
            }
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
    let mut port = 3002u16;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(3002);
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("Backend MCP Server #2 - Advanced Server");
                println!();
                println!("Usage: backend_server_2 [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -p, --port <PORT>  Port to listen on (default: 3002)");
                println!("  -h, --help         Show this help message");
                println!();
                println!("Available tools:");
                for tool in get_tools() {
                    println!("  {} - {}", tool.name, tool.description);
                }
                println!();
                println!("Available prompts:");
                for prompt in get_prompts() {
                    println!("  {} - {}", prompt.name, prompt.description);
                }
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
    println!("  Backend MCP Server #2 - Advanced Server");
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
    println!("Available resources:");
    for resource in get_resources(&state) {
        println!("  - {}: {}", resource.uri, resource.description);
    }
    println!();
    println!("Available prompts:");
    for prompt in get_prompts() {
        println!("  - {}: {}", prompt.name, prompt.description);
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
    fn test_base64_encode_decode() {
        let original = "Hello, World!";

        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut encoder = Base64Encoder::new(&mut buf);
            encoder.write_all(original.as_bytes()).unwrap();
            encoder.finish().unwrap();
        }
        let encoded = String::from_utf8(buf).unwrap();

        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_get_tools() {
        let tools = get_tools();
        assert!(tools.len() >= 5);
        assert!(tools.iter().any(|t| t.name == "read_file"));
        assert!(tools.iter().any(|t| t.name == "write_file"));
        assert!(tools.iter().any(|t| t.name == "base64_encode"));
    }

    #[test]
    fn test_get_prompts() {
        let prompts = get_prompts();
        assert_eq!(prompts.len(), 3);
        assert!(prompts.iter().any(|p| p.name == "code_review"));
        assert!(prompts.iter().any(|p| p.name == "summarize"));
        assert!(prompts.iter().any(|p| p.name == "explain"));
    }

    #[test]
    fn test_get_prompt_code_review() {
        let args = json!({"code": "fn main() {}", "language": "rust"});
        let result = get_prompt("code_review", &args).unwrap();
        assert!(result.get("messages").is_some());
    }

    #[test]
    fn test_get_prompt_summarize() {
        let args = json!({"text": "This is a test.", "max_length": 50});
        let result = get_prompt("summarize", &args).unwrap();
        assert!(result.get("messages").is_some());
    }
}
