//! API module
//!
//! This module provides the REST API for MetaMCP with security hardening
//! based on OWASP API Security Top 10 guidelines.

pub mod handlers;
pub mod middleware;
pub mod routes;

use crate::auth::AuthService;
use crate::db::Database;
use axum::{
    http::{header, Method, StatusCode},
    response::{IntoResponse, Json},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
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
///
/// # Security
/// This router includes several OWASP API Security Top 10 mitigations:
/// - OWASP API8:2023 - Security headers middleware
/// - OWASP API8:2023 - Restricted CORS configuration
/// - OWASP API4:2023 - Rate limiting headers
pub fn create_router(state: AppState) -> Router {
    // OWASP API8:2023 - Security Misconfiguration
    // CORS configuration - restrict to specific origins in production
    // For development, we allow common localhost ports
    // In production, replace with actual allowed origins
    let cors = CorsLayer::new()
        // OWASP API8:2023 - Restrict origins instead of allowing all (*)
        // Allow localhost for development, add production domains as needed
        .allow_origin([
            "http://localhost:3000".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(),  // Vite dev server
            "http://localhost:8080".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
            "http://127.0.0.1:5173".parse().unwrap(),
            "http://127.0.0.1:8080".parse().unwrap(),
        ])
        // OWASP API8:2023 - Only allow necessary HTTP methods
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // OWASP API8:2023 - Only allow necessary headers
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
            header::HeaderName::from_static("x-requested-with"),
        ])
        // Allow credentials for authenticated requests
        .allow_credentials(true)
        // Cache preflight requests for 1 hour
        .max_age(std::time::Duration::from_secs(3600));

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
        // OWASP API8:2023 - Add security headers to all responses
        .layer(axum::middleware::from_fn(middleware::security_headers))
        // OWASP API4:2023 - Add rate limiting headers
        .layer(axum::middleware::from_fn(middleware::rate_limit_headers))
        // Add tracing
        .layer(TraceLayer::new_for_http())
        // Add CORS
        .layer(cors)
}
