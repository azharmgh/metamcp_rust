//! API route definitions

use crate::api::handlers;
use crate::api::AppState;
use crate::auth::auth_middleware;
use axum::{middleware, routing::{get, post}, Router};

/// Create the public routes (no authentication required)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/v1/auth/token", post(handlers::authenticate))
        // MCP health check endpoint (required by Claude Code's HTTP transport)
        // Must be public as health checks may not include auth headers
        .route("/mcp/health", get(handlers::mcp_gateway::mcp_health))
}

/// Create the protected routes (authentication required)
pub fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // MCP Gateway endpoint (for Claude and other MCP clients)
        // Support both GET (for SSE/info) and POST (for JSON-RPC)
        .route("/mcp", get(handlers::mcp_gateway::mcp_gateway_sse).post(handlers::mcp_gateway))
        // MCP server management
        .route(
            "/api/v1/mcp/servers",
            get(handlers::list_mcp_servers).post(handlers::create_mcp_server),
        )
        .route(
            "/api/v1/mcp/servers/{server_id}",
            get(handlers::get_mcp_server)
                .put(handlers::update_mcp_server)
                .delete(handlers::delete_mcp_server),
        )
        // MCP tool execution
        .route(
            "/api/v1/mcp/servers/{server_id}/tools/{tool_name}/execute",
            post(handlers::execute_mcp_tool),
        )
        // Apply authentication middleware
        .layer(middleware::from_fn_with_state(state.auth.clone(), auth_middleware))
}
