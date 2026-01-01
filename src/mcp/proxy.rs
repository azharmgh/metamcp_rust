//! MCP Proxy for routing requests to backend servers

use crate::db::models::McpServer;
use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::utils::AppError;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

/// MCP Proxy for forwarding requests to backend servers
pub struct McpProxy {
    http_client: Client,
}

impl McpProxy {
    /// Create a new MCP proxy
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client: client,
        }
    }

    /// Forward a JSON-RPC request to a backend MCP server
    pub async fn forward_request(
        &self,
        server: &McpServer,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, AppError> {
        match server.protocol.as_str() {
            "http" => self.forward_http(server, request).await,
            "sse" => {
                // SSE support is planned for v2
                Err(AppError::McpProtocol(
                    "SSE protocol not yet implemented".to_string(),
                ))
            }
            "stdio" => {
                // stdio support is planned for v2
                Err(AppError::McpProtocol(
                    "stdio protocol not yet implemented".to_string(),
                ))
            }
            _ => Err(AppError::McpProtocol(format!(
                "Unknown protocol: {}",
                server.protocol
            ))),
        }
    }

    /// Forward request via HTTP
    async fn forward_http(
        &self,
        server: &McpServer,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, AppError> {
        let response = self
            .http_client
            .post(&server.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AppError::McpProtocol(format!("Failed to connect to MCP server: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(AppError::McpProtocol(format!(
                "MCP server returned error status: {}",
                response.status()
            )));
        }

        let json_response: JsonRpcResponse = response.json().await.map_err(|e| {
            AppError::McpProtocol(format!("Failed to parse MCP server response: {}", e))
        })?;

        Ok(json_response)
    }

    /// List tools from a backend server
    pub async fn list_tools(&self, server: &McpServer) -> Result<Vec<serde_json::Value>, AppError> {
        let request = JsonRpcRequest::new(1i64, "tools/list", None);
        let response = self.forward_request(server, request).await?;

        if let Some(error) = response.error {
            return Err(AppError::McpProtocol(format!(
                "MCP error: {} (code: {})",
                error.message, error.code
            )));
        }

        let result = response
            .result
            .ok_or_else(|| AppError::McpProtocol("Empty response from MCP server".to_string()))?;

        let tools = result
            .get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(tools)
    }

    /// Call a tool on a backend server
    pub async fn call_tool(
        &self,
        server: &McpServer,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        });

        let request = JsonRpcRequest::new(1i64, "tools/call", Some(params));
        let response = self.forward_request(server, request).await?;

        if let Some(error) = response.error {
            return Err(AppError::McpProtocol(format!(
                "Tool execution failed: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
            .ok_or_else(|| AppError::McpProtocol("Empty response from tool call".to_string()))
    }

    /// List resources from a backend server
    pub async fn list_resources(
        &self,
        server: &McpServer,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let request = JsonRpcRequest::new(1i64, "resources/list", None);
        let response = self.forward_request(server, request).await?;

        if let Some(error) = response.error {
            return Err(AppError::McpProtocol(format!(
                "MCP error: {} (code: {})",
                error.message, error.code
            )));
        }

        let result = response
            .result
            .ok_or_else(|| AppError::McpProtocol("Empty response from MCP server".to_string()))?;

        let resources = result
            .get("resources")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(resources)
    }

    /// List prompts from a backend server
    pub async fn list_prompts(
        &self,
        server: &McpServer,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let request = JsonRpcRequest::new(1i64, "prompts/list", None);
        let response = self.forward_request(server, request).await?;

        if let Some(error) = response.error {
            return Err(AppError::McpProtocol(format!(
                "MCP error: {} (code: {})",
                error.message, error.code
            )));
        }

        let result = response
            .result
            .ok_or_else(|| AppError::McpProtocol("Empty response from MCP server".to_string()))?;

        let prompts = result
            .get("prompts")
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(prompts)
    }
}

impl Default for McpProxy {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared MCP proxy instance
pub type SharedMcpProxy = Arc<McpProxy>;
