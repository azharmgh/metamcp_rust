//! MCP Server configuration model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// MCP Server protocol type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum McpProtocol {
    Http,
    Sse,
    Stdio,
}

impl Default for McpProtocol {
    fn default() -> Self {
        Self::Http
    }
}

/// MCP Server configuration stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct McpServer {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub protocol: String,
    pub command: Option<String>,
    pub args: Option<serde_json::Value>,
    pub env: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new MCP server configuration
#[derive(Debug, Deserialize)]
pub struct CreateMcpServerRequest {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub protocol: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

/// Request to update an MCP server configuration
#[derive(Debug, Deserialize)]
pub struct UpdateMcpServerRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub protocol: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub is_active: Option<bool>,
}

/// MCP Server info for API responses
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct McpServerInfo {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub protocol: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<McpServer> for McpServerInfo {
    fn from(server: McpServer) -> Self {
        Self {
            id: server.id,
            name: server.name,
            url: server.url,
            protocol: server.protocol,
            is_active: server.is_active,
            created_at: server.created_at,
            updated_at: server.updated_at,
        }
    }
}
