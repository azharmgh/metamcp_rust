//! Test MCP Client
//!
//! A test client for MetaMCP server that demonstrates:
//! - API key authentication
//! - JWT token retrieval
//! - MCP server management
//! - Tool execution
//!
//! Usage:
//!   cargo run --example test_client -- --api-key <API_KEY>
//!
//! Or set METAMCP_API_KEY environment variable.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// MetaMCP server URL
const DEFAULT_SERVER_URL: &str = "http://localhost:12009";

/// Authentication request
#[derive(Debug, Serialize)]
struct AuthRequest {
    api_key: String,
}

/// Authentication response
#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

/// MCP Server info
#[derive(Debug, Deserialize)]
struct McpServerInfo {
    id: String,
    name: String,
    url: String,
    protocol: String,
    is_active: bool,
}

/// List servers response
#[derive(Debug, Deserialize)]
struct ListServersResponse {
    servers: Vec<McpServerInfo>,
}

/// Create server request
#[derive(Debug, Serialize)]
struct CreateServerRequest {
    name: String,
    url: String,
    protocol: String,
}

/// Tool execution request
#[derive(Debug, Serialize)]
struct ToolRequest {
    arguments: serde_json::Value,
}

/// Tool execution response
#[derive(Debug, Deserialize)]
struct ToolResponse {
    result: serde_json::Value,
}

/// Health check response
#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

/// Test client for MetaMCP
struct TestClient {
    http_client: Client,
    base_url: String,
    token: Option<String>,
}

impl TestClient {
    /// Create a new test client
    fn new(base_url: &str) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            base_url: base_url.to_string(),
            token: None,
        }
    }

    /// Check server health
    async fn health_check(&self) -> Result<HealthResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/health", self.base_url);
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Health check failed: {}", response.status()).into());
        }

        Ok(response.json().await?)
    }

    /// Authenticate with API key and get JWT token
    async fn authenticate(&mut self, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/auth/token", self.base_url);
        let request = AuthRequest {
            api_key: api_key.to_string(),
        };

        let response = self.http_client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Authentication failed: {} - {}", status, body).into());
        }

        let auth_response: AuthResponse = response.json().await?;
        println!("  Token type: {}", auth_response.token_type);
        println!("  Expires in: {} seconds", auth_response.expires_in);

        self.token = Some(auth_response.access_token);
        Ok(())
    }

    /// Get authorization header
    fn auth_header(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.token
            .as_ref()
            .map(|t| format!("Bearer {}", t))
            .ok_or_else(|| "Not authenticated".into())
    }

    /// List MCP servers
    async fn list_servers(&self) -> Result<Vec<McpServerInfo>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/mcp/servers", self.base_url);
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("List servers failed: {} - {}", status, body).into());
        }

        let list_response: ListServersResponse = response.json().await?;
        Ok(list_response.servers)
    }

    /// Create an MCP server
    async fn create_server(
        &self,
        name: &str,
        url: &str,
        protocol: &str,
    ) -> Result<McpServerInfo, Box<dyn std::error::Error>> {
        let api_url = format!("{}/api/v1/mcp/servers", self.base_url);
        let request = CreateServerRequest {
            name: name.to_string(),
            url: url.to_string(),
            protocol: protocol.to_string(),
        };

        let response = self
            .http_client
            .post(&api_url)
            .header("Authorization", self.auth_header()?)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Create server failed: {} - {}", status, body).into());
        }

        Ok(response.json().await?)
    }

    /// Get an MCP server by ID
    async fn get_server(&self, server_id: &str) -> Result<McpServerInfo, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/mcp/servers/{}", self.base_url, server_id);
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Get server failed: {} - {}", status, body).into());
        }

        Ok(response.json().await?)
    }

    /// Delete an MCP server
    async fn delete_server(&self, server_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/mcp/servers/{}", self.base_url, server_id);
        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", self.auth_header()?)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Delete server failed: {} - {}", status, body).into());
        }

        Ok(())
    }

    /// Execute a tool on an MCP server
    async fn execute_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/api/v1/mcp/servers/{}/tools/{}/execute",
            self.base_url, server_id, tool_name
        );
        let request = ToolRequest { arguments };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.auth_header()?)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Execute tool failed: {} - {}", status, body).into());
        }

        let tool_response: ToolResponse = response.json().await?;
        Ok(tool_response.result)
    }
}

/// Parse command line arguments
fn parse_args() -> (String, String) {
    let args: Vec<String> = std::env::args().collect();
    let mut api_key = std::env::var("METAMCP_API_KEY").ok();
    let mut server_url = std::env::var("METAMCP_URL").unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string());

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--api-key" | "-k" => {
                if i + 1 < args.len() {
                    api_key = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--url" | "-u" => {
                if i + 1 < args.len() {
                    server_url = args[i + 1].clone();
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("MetaMCP Test Client");
                println!();
                println!("Usage: test_client [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -k, --api-key <KEY>  API key for authentication");
                println!("  -u, --url <URL>      MetaMCP server URL (default: {})", DEFAULT_SERVER_URL);
                println!("  -h, --help           Show this help message");
                println!();
                println!("Environment variables:");
                println!("  METAMCP_API_KEY      API key (alternative to --api-key)");
                println!("  METAMCP_URL          Server URL (alternative to --url)");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    let api_key = api_key.unwrap_or_else(|| {
        eprintln!("Error: API key required. Use --api-key or set METAMCP_API_KEY");
        std::process::exit(1);
    });

    (api_key, server_url)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (api_key, server_url) = parse_args();

    println!("===========================================");
    println!("  MetaMCP Test Client");
    println!("===========================================");
    println!();
    println!("Server: {}", server_url);
    println!();

    let mut client = TestClient::new(&server_url);

    // Step 1: Health check
    println!("[1/6] Health Check");
    println!("-------------------------------------------");
    match client.health_check().await {
        Ok(health) => {
            println!("  Status: {}", health.status);
            println!("  Version: {}", health.version);
        }
        Err(e) => {
            eprintln!("  Failed: {}", e);
            eprintln!("  Is the MetaMCP server running?");
            return Err(e);
        }
    }
    println!();

    // Step 2: Authenticate
    println!("[2/6] Authentication");
    println!("-------------------------------------------");
    match client.authenticate(&api_key).await {
        Ok(()) => println!("  Authentication successful!"),
        Err(e) => {
            eprintln!("  Authentication failed: {}", e);
            return Err(e);
        }
    }
    println!();

    // Step 3: List servers
    println!("[3/6] List MCP Servers");
    println!("-------------------------------------------");
    let servers = client.list_servers().await?;
    if servers.is_empty() {
        println!("  No servers configured");
    } else {
        for server in &servers {
            println!(
                "  - {} ({}): {} [{}]",
                server.name,
                server.id,
                server.url,
                if server.is_active { "active" } else { "inactive" }
            );
        }
    }
    println!();

    // Step 4: Create a test server
    println!("[4/6] Create Test Server");
    println!("-------------------------------------------");
    let test_server = client
        .create_server(
            "test-backend-1",
            "http://localhost:3001",
            "http",
        )
        .await?;
    println!("  Created: {} ({})", test_server.name, test_server.id);
    println!();

    // Step 5: Execute a tool (placeholder)
    println!("[5/6] Execute Tool");
    println!("-------------------------------------------");
    let result = client
        .execute_tool(
            &test_server.id,
            "echo",
            serde_json::json!({"message": "Hello from test client!"}),
        )
        .await?;
    println!("  Result: {}", serde_json::to_string_pretty(&result)?);
    println!();

    // Step 6: Clean up - delete test server
    println!("[6/6] Cleanup");
    println!("-------------------------------------------");
    client.delete_server(&test_server.id).await?;
    println!("  Deleted test server: {}", test_server.id);
    println!();

    println!("===========================================");
    println!("  All tests completed successfully!");
    println!("===========================================");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_request_serialization() {
        let request = AuthRequest {
            api_key: "test_key".to_string(),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test_key"));
    }

    #[test]
    fn test_tool_request_serialization() {
        let request = ToolRequest {
            arguments: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("key"));
        assert!(json.contains("value"));
    }
}
