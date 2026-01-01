//! API handlers

pub mod auth;
pub mod health;
pub mod mcp;

pub use auth::{authenticate, AuthRequest, AuthResponse};
pub use health::{health_check, HealthResponse};
pub use mcp::{
    create_mcp_server, delete_mcp_server, execute_mcp_tool, get_mcp_server, list_mcp_servers,
    update_mcp_server, ListMcpServersResponse, McpToolRequest, McpToolResponse,
};
