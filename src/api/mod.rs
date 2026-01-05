//! API module

pub mod handlers;
pub mod middleware;
pub mod routes;

use crate::auth::AuthService;
use crate::db::Database;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub auth: Arc<AuthService>,
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::health_check,
        handlers::auth::authenticate,
        handlers::mcp::list_mcp_servers,
        handlers::mcp::get_mcp_server,
        handlers::mcp::create_mcp_server,
        handlers::mcp::update_mcp_server,
        handlers::mcp::delete_mcp_server,
        handlers::mcp::execute_mcp_tool,
    ),
    components(
        schemas(
            handlers::health::HealthResponse,
            handlers::auth::AuthRequest,
            handlers::auth::AuthResponse,
            handlers::mcp::ListMcpServersResponse,
            handlers::mcp::McpToolRequest,
            handlers::mcp::McpToolResponse,
            handlers::mcp::CreateMcpServerSchema,
            handlers::mcp::UpdateMcpServerSchema,
            crate::db::models::McpServerInfo,
            crate::utils::ErrorResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "mcp", description = "MCP server management")
    ),
    info(
        title = "MetaMCP API",
        version = "1.0.0",
        description = "Headless API backend for MetaMCP - MCP Protocol Proxy",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:12009", description = "Local development")
    )
)]
pub struct ApiDoc;

/// Fallback handler for 404 errors - returns JSON response
async fn fallback_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "not_found",
            "error_description": "The requested endpoint does not exist"
        })),
    )
}

/// Create the API router with all routes
pub fn create_router(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Public routes
        .merge(routes::public_routes())
        // Protected routes
        .merge(routes::protected_routes(state.clone()))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Add state
        .with_state(state)
        // Fallback for unmatched routes (returns JSON 404)
        .fallback(fallback_handler)
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
