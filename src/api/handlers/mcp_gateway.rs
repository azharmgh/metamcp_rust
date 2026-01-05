//! MCP Gateway handler - implements the MCP protocol endpoint for Claude

use crate::api::AppState;
use crate::mcp::protocol::{
    InitializeResult, JsonRpcRequest, JsonRpcResponse, PromptsCapability, ResourcesCapability,
    ServerCapabilities, ServerInfo, ToolsCapability, MCP_PROTOCOL_VERSION,
};
use crate::mcp::McpProxy;
use crate::utils::AppError;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue},
    response::{sse::Event, IntoResponse, Response, Sse},
    Json,
};
use futures::stream::{self, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

/// MCP Gateway state
#[derive(Clone)]
pub struct McpGatewayState {
    pub db: crate::db::Database,
    pub proxy: Arc<McpProxy>,
}

/// Handle MCP protocol requests at /mcp endpoint
pub async fn mcp_gateway(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Response, AppError> {
    let proxy = McpProxy::new();

    tracing::debug!("MCP Gateway received: {} (id: {:?})", request.method, request.id);

    // Add MCP protocol headers
    let mut headers = HeaderMap::new();
    headers.insert(
        "mcp-protocol-version",
        HeaderValue::from_static(MCP_PROTOCOL_VERSION),
    );

    // Handle notifications (no id) - they don't expect a response
    if request.id.is_none() {
        match request.method.as_str() {
            "initialized" | "notifications/cancelled" => {
                // Return 202 Accepted for notifications
                return Ok((headers, axum::http::StatusCode::ACCEPTED).into_response());
            }
            _ => {
                // Unknown notification - still return 202
                return Ok((headers, axum::http::StatusCode::ACCEPTED).into_response());
            }
        }
    }

    // For requests with id, process and return response
    let id = request.id.unwrap();
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(id).await,
        "tools/list" => handle_tools_list(&state, &proxy, id).await,
        "tools/call" => handle_tools_call(&state, &proxy, id, request.params).await,
        "resources/list" => handle_resources_list(&state, &proxy, id).await,
        "resources/read" => handle_resources_read(&state, &proxy, id, request.params).await,
        "prompts/list" => handle_prompts_list(&state, &proxy, id).await,
        "prompts/get" => handle_prompts_get(&state, &proxy, id, request.params).await,
        "ping" => handle_ping(id).await,
        _ => JsonRpcResponse::error(
            id,
            -32601,
            &format!("Method not found: {}", request.method),
            None,
        ),
    };

    Ok((headers, Json(response)).into_response())
}

/// Handle initialize request
async fn handle_initialize(id: crate::mcp::protocol::RequestId) -> JsonRpcResponse {
    let result = InitializeResult {
        protocol_version: MCP_PROTOCOL_VERSION.to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: false }),
            resources: Some(ResourcesCapability {
                subscribe: false,
                list_changed: false,
            }),
            prompts: Some(PromptsCapability { list_changed: false }),
        },
        server_info: ServerInfo {
            name: "metamcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
}

/// Handle tools/list - aggregate tools from all backend servers
async fn handle_tools_list(
    state: &AppState,
    proxy: &McpProxy,
    id: crate::mcp::protocol::RequestId,
) -> JsonRpcResponse {
    let servers = match state.db.mcp_servers().list_all(false).await {
        Ok(s) => s,
        Err(e) => {
            return JsonRpcResponse::error(id, -32000, &format!("Database error: {}", e), None)
        }
    };

    let mut all_tools: Vec<Value> = Vec::new();
    let mut tool_server_map: HashMap<String, String> = HashMap::new();

    for server in &servers {
        match proxy.list_tools(server).await {
            Ok(tools) => {
                for tool in tools {
                    if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                        // Prefix tool name with server name to avoid collisions
                        let prefixed_name = format!("{}_{}", server.name, name);
                        let mut tool_with_prefix = tool.clone();
                        if let Some(obj) = tool_with_prefix.as_object_mut() {
                            obj.insert("name".to_string(), json!(prefixed_name.clone()));
                            // Add original name and server info for routing
                            obj.insert("_original_name".to_string(), json!(name));
                            obj.insert("_server_id".to_string(), json!(server.id.to_string()));
                        }
                        tool_server_map.insert(prefixed_name.clone(), server.id.to_string());
                        all_tools.push(tool_with_prefix);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to list tools from {}: {}", server.name, e);
            }
        }
    }

    JsonRpcResponse::success(id, json!({ "tools": all_tools }))
}

/// Handle tools/call - route to appropriate backend server
async fn handle_tools_call(
    state: &AppState,
    proxy: &McpProxy,
    id: crate::mcp::protocol::RequestId,
    params: Option<Value>,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => return JsonRpcResponse::error(id, -32602, "Missing params", None),
    };

    let tool_name = match params.get("name").and_then(|n| n.as_str()) {
        Some(n) => n,
        None => return JsonRpcResponse::error(id, -32602, "Missing tool name", None),
    };

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    // Parse the prefixed tool name to find server and original tool name
    // Format: servername_toolname
    let servers = match state.db.mcp_servers().list_all(false).await {
        Ok(s) => s,
        Err(e) => {
            return JsonRpcResponse::error(id, -32000, &format!("Database error: {}", e), None)
        }
    };

    // Find matching server by prefix
    for server in &servers {
        let prefix = format!("{}_", server.name);
        if tool_name.starts_with(&prefix) {
            let original_tool_name = &tool_name[prefix.len()..];
            match proxy.call_tool(server, original_tool_name, arguments).await {
                Ok(result) => return JsonRpcResponse::success(id, result),
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        -32000,
                        &format!("Tool call failed: {}", e),
                        None,
                    )
                }
            }
        }
    }

    JsonRpcResponse::error(id, -32602, &format!("Unknown tool: {}", tool_name), None)
}

/// Handle resources/list - aggregate resources from all backend servers
async fn handle_resources_list(
    state: &AppState,
    proxy: &McpProxy,
    id: crate::mcp::protocol::RequestId,
) -> JsonRpcResponse {
    let servers = match state.db.mcp_servers().list_all(false).await {
        Ok(s) => s,
        Err(e) => {
            return JsonRpcResponse::error(id, -32000, &format!("Database error: {}", e), None)
        }
    };

    let mut all_resources: Vec<Value> = Vec::new();

    for server in &servers {
        match proxy.list_resources(server).await {
            Ok(resources) => {
                for resource in resources {
                    // Add server prefix to URI to avoid collisions
                    let mut resource_with_server = resource.clone();
                    if let Some(obj) = resource_with_server.as_object_mut() {
                        let uri_opt = obj.get("uri").and_then(|u| u.as_str()).map(|s| s.to_string());
                        if let Some(uri) = uri_opt {
                            let prefixed_uri = format!("{}:{}", server.name, uri);
                            obj.insert("uri".to_string(), json!(prefixed_uri));
                            obj.insert("_original_uri".to_string(), json!(uri));
                            obj.insert("_server_id".to_string(), json!(server.id.to_string()));
                        }
                    }
                    all_resources.push(resource_with_server);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to list resources from {}: {}", server.name, e);
            }
        }
    }

    JsonRpcResponse::success(id, json!({ "resources": all_resources }))
}

/// Handle resources/read - route to appropriate backend server
async fn handle_resources_read(
    state: &AppState,
    proxy: &McpProxy,
    id: crate::mcp::protocol::RequestId,
    params: Option<Value>,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => return JsonRpcResponse::error(id, -32602, "Missing params", None),
    };

    let uri = match params.get("uri").and_then(|u| u.as_str()) {
        Some(u) => u,
        None => return JsonRpcResponse::error(id, -32602, "Missing uri", None),
    };

    let servers = match state.db.mcp_servers().list_all(false).await {
        Ok(s) => s,
        Err(e) => {
            return JsonRpcResponse::error(id, -32000, &format!("Database error: {}", e), None)
        }
    };

    // Parse prefixed URI: servername:original_uri
    for server in &servers {
        let prefix = format!("{}:", server.name);
        if uri.starts_with(&prefix) {
            let original_uri = &uri[prefix.len()..];

            let request = crate::mcp::protocol::JsonRpcRequest::new(
                1i64,
                "resources/read",
                Some(json!({ "uri": original_uri })),
            );

            match proxy.forward_request(server, request).await {
                Ok(response) => {
                    if let Some(result) = response.result {
                        return JsonRpcResponse::success(id, result);
                    } else if let Some(error) = response.error {
                        return JsonRpcResponse::error(id, error.code, &error.message, error.data);
                    }
                }
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        -32000,
                        &format!("Resource read failed: {}", e),
                        None,
                    )
                }
            }
        }
    }

    JsonRpcResponse::error(id, -32602, &format!("Unknown resource: {}", uri), None)
}

/// Handle prompts/list - aggregate prompts from all backend servers
async fn handle_prompts_list(
    state: &AppState,
    proxy: &McpProxy,
    id: crate::mcp::protocol::RequestId,
) -> JsonRpcResponse {
    let servers = match state.db.mcp_servers().list_all(false).await {
        Ok(s) => s,
        Err(e) => {
            return JsonRpcResponse::error(id, -32000, &format!("Database error: {}", e), None)
        }
    };

    let mut all_prompts: Vec<Value> = Vec::new();

    for server in &servers {
        match proxy.list_prompts(server).await {
            Ok(prompts) => {
                for prompt in prompts {
                    let mut prompt_with_server = prompt.clone();
                    if let Some(obj) = prompt_with_server.as_object_mut() {
                        let name_opt = obj.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
                        if let Some(name) = name_opt {
                            let prefixed_name = format!("{}_{}", server.name, name);
                            obj.insert("name".to_string(), json!(prefixed_name));
                            obj.insert("_original_name".to_string(), json!(name));
                            obj.insert("_server_id".to_string(), json!(server.id.to_string()));
                        }
                    }
                    all_prompts.push(prompt_with_server);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to list prompts from {}: {}", server.name, e);
            }
        }
    }

    JsonRpcResponse::success(id, json!({ "prompts": all_prompts }))
}

/// Handle prompts/get - route to appropriate backend server
async fn handle_prompts_get(
    state: &AppState,
    proxy: &McpProxy,
    id: crate::mcp::protocol::RequestId,
    params: Option<Value>,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => return JsonRpcResponse::error(id, -32602, "Missing params", None),
    };

    let prompt_name = match params.get("name").and_then(|n| n.as_str()) {
        Some(n) => n,
        None => return JsonRpcResponse::error(id, -32602, "Missing prompt name", None),
    };

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    let servers = match state.db.mcp_servers().list_all(false).await {
        Ok(s) => s,
        Err(e) => {
            return JsonRpcResponse::error(id, -32000, &format!("Database error: {}", e), None)
        }
    };

    // Parse prefixed prompt name
    for server in &servers {
        let prefix = format!("{}_", server.name);
        if prompt_name.starts_with(&prefix) {
            let original_name = &prompt_name[prefix.len()..];

            let request = crate::mcp::protocol::JsonRpcRequest::new(
                1i64,
                "prompts/get",
                Some(json!({
                    "name": original_name,
                    "arguments": arguments
                })),
            );

            match proxy.forward_request(server, request).await {
                Ok(response) => {
                    if let Some(result) = response.result {
                        return JsonRpcResponse::success(id, result);
                    } else if let Some(error) = response.error {
                        return JsonRpcResponse::error(id, error.code, &error.message, error.data);
                    }
                }
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        -32000,
                        &format!("Prompt get failed: {}", e),
                        None,
                    )
                }
            }
        }
    }

    JsonRpcResponse::error(id, -32602, &format!("Unknown prompt: {}", prompt_name), None)
}

/// Handle ping request
async fn handle_ping(id: crate::mcp::protocol::RequestId) -> JsonRpcResponse {
    JsonRpcResponse::success(id, json!({}))
}

/// MCP health check response
#[derive(serde::Serialize)]
pub struct McpHealthResponse {
    pub status: String,
    pub version: String,
}

/// Handle MCP health check at /mcp/health endpoint
/// Required by Claude Code's HTTP transport for health checks
pub async fn mcp_health() -> Json<McpHealthResponse> {
    Json(McpHealthResponse {
        status: "ok".to_string(),
        version: MCP_PROTOCOL_VERSION.to_string(),
    })
}

/// Handle GET requests to /mcp - returns persistent SSE stream for MCP protocol
/// This is required by Claude Code's HTTP transport for server-to-client messages
pub async fn mcp_gateway_sse(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Create a persistent SSE stream that stays open
    // First send an endpoint event, then keep the connection alive with periodic pings
    let endpoint_msg = json!({
        "jsonrpc": "2.0",
        "method": "endpoint",
        "params": {
            "uri": "/mcp"
        }
    });

    // Create a stream that sends the endpoint message first, then sends periodic keep-alive pings
    let initial = stream::once(async move {
        Ok::<_, Infallible>(Event::default().data(endpoint_msg.to_string()))
    });

    // Create a keep-alive stream that sends pings every 30 seconds to keep connection open
    let keep_alive = stream::unfold((), |_| async {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        Some((Ok::<_, Infallible>(Event::default().comment("ping")), ()))
    });

    let combined = initial.chain(keep_alive);

    // Add MCP protocol version header to SSE response
    let mut headers = HeaderMap::new();
    headers.insert(
        "mcp-protocol-version",
        HeaderValue::from_static(MCP_PROTOCOL_VERSION),
    );

    (headers, Sse::new(combined).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    ))
}
