//! MCP (Model Context Protocol) module

pub mod protocol;
pub mod proxy;
pub mod server_manager;

pub use protocol::*;
pub use proxy::{McpProxy, SharedMcpProxy};
pub use server_manager::{McpServerConfig, McpServerManager, ServerInfo, ServerStatus};
