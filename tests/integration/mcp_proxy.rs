//! Integration tests for MCP proxy functionality
//!
//! These tests verify the MCP proxy can correctly forward requests
//! to backend MCP servers.

use metamcp::mcp::{McpProxy, McpServerManager, McpServerConfig};
use metamcp::db::models::McpServer;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, body_json};

/// Create a mock MCP server for testing
fn create_mock_mcp_server(url: &str) -> McpServer {
    McpServer {
        id: Uuid::new_v4(),
        name: "Mock MCP Server".to_string(),
        url: url.to_string(),
        protocol: "http".to_string(),
        command: None,
        args: None,
        env: None,
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_mcp_proxy_list_tools() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response for tools/list
    Mock::given(method("POST"))
        .and(body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": null
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "tools": [
                    {
                        "name": "echo",
                        "description": "Echoes input",
                        "inputSchema": {"type": "object"}
                    },
                    {
                        "name": "add",
                        "description": "Adds numbers",
                        "inputSchema": {"type": "object"}
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    // Create proxy and server
    let proxy = McpProxy::new();
    let server = create_mock_mcp_server(&mock_server.uri());

    // List tools
    let tools = proxy.list_tools(&server).await.expect("Failed to list tools");

    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|t| t.get("name").and_then(|n| n.as_str()) == Some("echo")));
    assert!(tools.iter().any(|t| t.get("name").and_then(|n| n.as_str()) == Some("add")));
}

#[tokio::test]
async fn test_mcp_proxy_call_tool() {
    let mock_server = MockServer::start().await;

    // Set up mock response for tools/call
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "content": [{
                    "type": "text",
                    "text": "Hello, World!"
                }]
            }
        })))
        .mount(&mock_server)
        .await;

    let proxy = McpProxy::new();
    let server = create_mock_mcp_server(&mock_server.uri());

    let result = proxy
        .call_tool(&server, "echo", json!({"message": "Hello, World!"}))
        .await
        .expect("Failed to call tool");

    assert!(result.get("content").is_some());
}

#[tokio::test]
async fn test_mcp_proxy_list_resources() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "resources": [
                    {
                        "uri": "file:///test.txt",
                        "name": "Test File",
                        "mimeType": "text/plain"
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let proxy = McpProxy::new();
    let server = create_mock_mcp_server(&mock_server.uri());

    let resources = proxy
        .list_resources(&server)
        .await
        .expect("Failed to list resources");

    assert_eq!(resources.len(), 1);
}

#[tokio::test]
async fn test_mcp_proxy_list_prompts() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "prompts": [
                    {
                        "name": "code_review",
                        "description": "Review code"
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let proxy = McpProxy::new();
    let server = create_mock_mcp_server(&mock_server.uri());

    let prompts = proxy
        .list_prompts(&server)
        .await
        .expect("Failed to list prompts");

    assert_eq!(prompts.len(), 1);
}

#[tokio::test]
async fn test_mcp_proxy_error_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        })))
        .mount(&mock_server)
        .await;

    let proxy = McpProxy::new();
    let server = create_mock_mcp_server(&mock_server.uri());

    let result = proxy.list_tools(&server).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_proxy_connection_error() {
    let proxy = McpProxy::new();

    // Server that doesn't exist
    let server = create_mock_mcp_server("http://localhost:59999");

    let result = proxy.list_tools(&server).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_proxy_unsupported_protocol() {
    let proxy = McpProxy::new();

    let mut server = create_mock_mcp_server("http://localhost:3000");
    server.protocol = "sse".to_string(); // SSE not yet implemented

    let result = proxy.list_tools(&server).await;
    assert!(result.is_err());

    server.protocol = "stdio".to_string(); // stdio not yet implemented
    let result = proxy.list_tools(&server).await;
    assert!(result.is_err());
}

// MCP Server Manager tests
#[tokio::test]
async fn test_server_manager_new() {
    let manager = McpServerManager::new();
    let servers = manager.list_servers().await;
    assert!(servers.is_empty());
}

#[tokio::test]
async fn test_server_manager_spawn_invalid_command() {
    let manager = McpServerManager::new();

    let config = McpServerConfig {
        name: "Invalid Server".to_string(),
        command: "/nonexistent/command".to_string(),
        args: vec![],
        env: HashMap::new(),
        working_dir: None,
    };

    let result = manager.spawn_server(config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_server_manager_stop_nonexistent() {
    let manager = McpServerManager::new();

    // Stopping a non-existent server should succeed (no-op)
    let result = manager.stop_server("nonexistent-id").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_server_manager_get_nonexistent() {
    let manager = McpServerManager::new();

    let server = manager.get_server("nonexistent-id").await;
    assert!(server.is_none());
}

#[tokio::test]
async fn test_server_manager_restart_nonexistent() {
    let manager = McpServerManager::new();

    let result = manager.restart_server("nonexistent-id").await;
    assert!(result.is_err());
}

// Test with a simple echo command that should work on most systems
#[tokio::test]
#[cfg(unix)]
async fn test_server_manager_spawn_echo() {
    let manager = McpServerManager::new();

    let config = McpServerConfig {
        name: "Echo Server".to_string(),
        command: "cat".to_string(), // 'cat' will echo stdin to stdout
        args: vec![],
        env: HashMap::new(),
        working_dir: None,
    };

    // This might fail on some systems, so we just test that it doesn't panic
    let result = manager.spawn_server(config).await;

    if let Ok(server_id) = result {
        // Verify server is listed
        let servers = manager.list_servers().await;
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, server_id);

        // Get server info
        let info = manager.get_server(&server_id).await;
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, "Echo Server");

        // Stop the server
        manager.stop_server(&server_id).await.expect("Failed to stop server");

        // Verify server is removed
        let servers = manager.list_servers().await;
        assert!(servers.is_empty());
    }
}

#[tokio::test]
async fn test_server_manager_send_message_nonexistent() {
    let manager = McpServerManager::new();

    let result = manager.send_message("nonexistent", "test message").await;
    assert!(result.is_err());
}
