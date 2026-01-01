//! MetaMCP - MCP Protocol Proxy Server
//!
//! A headless API backend for proxying Model Context Protocol (MCP) requests
//! to multiple backend MCP servers.

pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod mcp;
pub mod streaming;
pub mod utils;

// Re-export commonly used types
pub use api::{create_router, AppState};
pub use auth::AuthService;
pub use config::Config;
pub use db::Database;
pub use mcp::{McpProxy, McpServerManager};
pub use streaming::StreamManager;
pub use utils::{AppError, AppResult};
