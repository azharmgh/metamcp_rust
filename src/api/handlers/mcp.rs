//! MCP server management handlers

use crate::api::AppState;
use crate::auth::AuthenticatedUser;
use crate::db::models::{CreateMcpServerRequest, McpServerInfo, UpdateMcpServerRequest};
use crate::utils::AppError;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// List MCP servers response
#[derive(Debug, Serialize, ToSchema)]
pub struct ListMcpServersResponse {
    pub servers: Vec<McpServerInfo>,
}

/// List all MCP servers
#[utoipa::path(
    get,
    path = "/api/v1/mcp/servers",
    tag = "mcp",
    responses(
        (status = 200, description = "List of MCP servers", body = ListMcpServersResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_mcp_servers(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<ListMcpServersResponse>, AppError> {
    let servers = state.db.mcp_servers().list_all(false).await?;
    let server_infos: Vec<McpServerInfo> = servers.into_iter().map(Into::into).collect();

    Ok(Json(ListMcpServersResponse {
        servers: server_infos,
    }))
}

/// Get a specific MCP server
#[utoipa::path(
    get,
    path = "/api/v1/mcp/servers/{server_id}",
    tag = "mcp",
    params(
        ("server_id" = Uuid, Path, description = "MCP Server ID")
    ),
    responses(
        (status = 200, description = "MCP server details", body = McpServerInfo),
        (status = 404, description = "Server not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_mcp_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<McpServerInfo>, AppError> {
    let server = state
        .db
        .mcp_servers()
        .find_by_id(server_id)
        .await?
        .ok_or_else(|| AppError::NotFound("MCP server not found".to_string()))?;

    Ok(Json(server.into()))
}

/// Create MCP server request schema for OpenAPI
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMcpServerSchema {
    /// Server name
    #[schema(example = "my-mcp-server")]
    pub name: String,
    /// Server URL
    #[schema(example = "http://localhost:3001")]
    pub url: String,
    /// Protocol type (http, sse, stdio)
    #[schema(example = "http")]
    pub protocol: Option<String>,
    /// Command for stdio servers
    pub command: Option<String>,
    /// Command arguments for stdio servers
    pub args: Option<Vec<String>>,
    /// Environment variables
    pub env: Option<std::collections::HashMap<String, String>>,
}

/// Create a new MCP server
#[utoipa::path(
    post,
    path = "/api/v1/mcp/servers",
    tag = "mcp",
    request_body = CreateMcpServerSchema,
    responses(
        (status = 201, description = "MCP server created", body = McpServerInfo),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_mcp_server(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(payload): Json<CreateMcpServerRequest>,
) -> Result<Json<McpServerInfo>, AppError> {
    let server = state.db.mcp_servers().create(&payload).await?;
    Ok(Json(server.into()))
}

/// Update MCP server request schema for OpenAPI
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMcpServerSchema {
    /// Server name
    pub name: Option<String>,
    /// Server URL
    pub url: Option<String>,
    /// Protocol type
    pub protocol: Option<String>,
    /// Command for stdio servers
    pub command: Option<String>,
    /// Command arguments
    pub args: Option<Vec<String>>,
    /// Environment variables
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Whether the server is active
    pub is_active: Option<bool>,
}

/// Update an MCP server
#[utoipa::path(
    put,
    path = "/api/v1/mcp/servers/{server_id}",
    tag = "mcp",
    params(
        ("server_id" = Uuid, Path, description = "MCP Server ID")
    ),
    request_body = UpdateMcpServerSchema,
    responses(
        (status = 200, description = "MCP server updated", body = McpServerInfo),
        (status = 404, description = "Server not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_mcp_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    _user: AuthenticatedUser,
    Json(payload): Json<UpdateMcpServerRequest>,
) -> Result<Json<McpServerInfo>, AppError> {
    let server = state
        .db
        .mcp_servers()
        .update(server_id, &payload)
        .await?
        .ok_or_else(|| AppError::NotFound("MCP server not found".to_string()))?;

    Ok(Json(server.into()))
}

/// Delete an MCP server
#[utoipa::path(
    delete,
    path = "/api/v1/mcp/servers/{server_id}",
    tag = "mcp",
    params(
        ("server_id" = Uuid, Path, description = "MCP Server ID")
    ),
    responses(
        (status = 204, description = "MCP server deleted"),
        (status = 404, description = "Server not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_mcp_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<(), AppError> {
    let deleted = state.db.mcp_servers().delete(server_id).await?;

    if !deleted {
        return Err(AppError::NotFound("MCP server not found".to_string()));
    }

    Ok(())
}

/// MCP tool execution request
#[derive(Debug, Deserialize, ToSchema)]
pub struct McpToolRequest {
    /// Tool arguments
    pub arguments: serde_json::Value,
}

/// MCP tool execution response
#[derive(Debug, Serialize, ToSchema)]
pub struct McpToolResponse {
    /// Tool execution result
    pub result: serde_json::Value,
}

/// Execute a tool on an MCP server
#[utoipa::path(
    post,
    path = "/api/v1/mcp/servers/{server_id}/tools/{tool_name}/execute",
    tag = "mcp",
    params(
        ("server_id" = Uuid, Path, description = "MCP Server ID"),
        ("tool_name" = String, Path, description = "Tool name to execute")
    ),
    request_body = McpToolRequest,
    responses(
        (status = 200, description = "Tool executed successfully", body = McpToolResponse),
        (status = 404, description = "Server or tool not found"),
        (status = 500, description = "Tool execution failed"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn execute_mcp_tool(
    State(state): State<AppState>,
    Path((server_id, tool_name)): Path<(Uuid, String)>,
    _user: AuthenticatedUser,
    Json(payload): Json<McpToolRequest>,
) -> Result<Json<McpToolResponse>, AppError> {
    // Verify server exists
    let _server = state
        .db
        .mcp_servers()
        .find_by_id(server_id)
        .await?
        .ok_or_else(|| AppError::NotFound("MCP server not found".to_string()))?;

    // TODO: Implement actual MCP tool execution via MCP proxy
    // For now, return a placeholder response
    tracing::info!("Executing tool '{}' on server {}", tool_name, server_id);

    Ok(Json(McpToolResponse {
        result: serde_json::json!({
            "status": "not_implemented",
            "message": format!("Tool execution for '{}' is not yet implemented", tool_name),
            "arguments": payload.arguments
        }),
    }))
}
