//! MCP Server process management

use crate::mcp::protocol::JsonRpcNotification;
use crate::utils::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use uuid::Uuid;

/// MCP Server status
#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Starting,
    Running,
    Stopped,
    Failed(String),
}

/// MCP Server configuration for spawning
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
}

/// Handle to a running MCP server process
pub struct McpServerHandle {
    pub id: String,
    pub config: McpServerConfig,
    pub status: ServerStatus,
    child: Option<Child>,
}

impl McpServerHandle {
    /// Check if the server process is still running
    pub fn is_running(&self) -> bool {
        matches!(self.status, ServerStatus::Running)
    }
}

/// Manager for MCP server processes
pub struct McpServerManager {
    servers: Arc<RwLock<HashMap<String, McpServerHandle>>>,
}

impl McpServerManager {
    /// Create a new server manager
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Spawn a new MCP server process
    pub async fn spawn_server(&self, config: McpServerConfig) -> Result<String, AppError> {
        let server_id = Uuid::new_v4().to_string();

        // Build the command
        let mut cmd = Command::new(&config.command);

        // Add arguments
        for arg in &config.args {
            cmd.arg(arg);
        }

        // Set environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Set working directory if specified
        if let Some(dir) = &config.working_dir {
            cmd.current_dir(dir);
        }

        // Configure pipes for stdin/stdout/stderr
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Kill process on drop
        cmd.kill_on_drop(true);

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| {
            AppError::Process(format!("Failed to spawn MCP server '{}': {}", config.name, e))
        })?;

        // Set up stderr logging
        if let Some(stderr) = child.stderr.take() {
            let server_id_clone = server_id.clone();
            let server_name = config.name.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!(
                        server_id = %server_id_clone,
                        server_name = %server_name,
                        "MCP server stderr: {}",
                        line
                    );
                }
            });
        }

        // Store the server handle
        let handle = McpServerHandle {
            id: server_id.clone(),
            config,
            status: ServerStatus::Running,
            child: Some(child),
        };

        self.servers.write().await.insert(server_id.clone(), handle);

        tracing::info!(server_id = %server_id, "MCP server spawned");

        Ok(server_id)
    }

    /// Stop a running MCP server
    pub async fn stop_server(&self, server_id: &str) -> Result<(), AppError> {
        let mut servers = self.servers.write().await;

        if let Some(mut handle) = servers.remove(server_id) {
            if let Some(mut child) = handle.child.take() {
                // Try graceful shutdown first
                if let Some(mut stdin) = child.stdin.take() {
                    // Send shutdown notification
                    let shutdown = JsonRpcNotification {
                        jsonrpc: "2.0".to_string(),
                        method: "shutdown".to_string(),
                        params: None,
                    };
                    let msg = serde_json::to_string(&shutdown).unwrap_or_default();
                    let _ = stdin.write_all(msg.as_bytes()).await;
                    let _ = stdin.write_all(b"\n").await;
                    let _ = stdin.flush().await;
                }

                // Wait for graceful shutdown with timeout
                let timeout = tokio::time::Duration::from_secs(5);
                match tokio::time::timeout(timeout, child.wait()).await {
                    Ok(Ok(status)) => {
                        tracing::info!(
                            server_id = %server_id,
                            status = ?status,
                            "MCP server stopped gracefully"
                        );
                    }
                    _ => {
                        // Force kill if graceful shutdown failed
                        let _ = child.kill().await;
                        tracing::warn!(server_id = %server_id, "MCP server force killed");
                    }
                }
            }

            handle.status = ServerStatus::Stopped;
        }

        Ok(())
    }

    /// Restart a server with the same configuration
    pub async fn restart_server(&self, server_id: &str) -> Result<String, AppError> {
        // Get the config before stopping
        let config = {
            let servers = self.servers.read().await;
            servers
                .get(server_id)
                .map(|h| h.config.clone())
                .ok_or_else(|| AppError::NotFound(format!("Server {} not found", server_id)))?
        };

        // Stop the server
        self.stop_server(server_id).await?;

        // Spawn new instance
        self.spawn_server(config).await
    }

    /// List all servers
    pub async fn list_servers(&self) -> Vec<ServerInfo> {
        let servers = self.servers.read().await;
        servers
            .values()
            .map(|handle| ServerInfo {
                id: handle.id.clone(),
                name: handle.config.name.clone(),
                status: handle.status.clone(),
            })
            .collect()
    }

    /// Get server info by ID
    pub async fn get_server(&self, server_id: &str) -> Option<ServerInfo> {
        let servers = self.servers.read().await;
        servers.get(server_id).map(|handle| ServerInfo {
            id: handle.id.clone(),
            name: handle.config.name.clone(),
            status: handle.status.clone(),
        })
    }

    /// Send a message to a server's stdin
    pub async fn send_message(&self, server_id: &str, message: &str) -> Result<(), AppError> {
        let mut servers = self.servers.write().await;

        let handle = servers
            .get_mut(server_id)
            .ok_or_else(|| AppError::NotFound(format!("Server {} not found", server_id)))?;

        if let Some(ref mut child) = handle.child {
            if let Some(ref mut stdin) = child.stdin {
                stdin
                    .write_all(message.as_bytes())
                    .await
                    .map_err(|e| AppError::Process(format!("Failed to write to server: {}", e)))?;
                stdin
                    .write_all(b"\n")
                    .await
                    .map_err(|e| AppError::Process(format!("Failed to write newline: {}", e)))?;
                stdin
                    .flush()
                    .await
                    .map_err(|e| AppError::Process(format!("Failed to flush: {}", e)))?;
            }
        }

        Ok(())
    }

    /// Monitor server health (run in background task)
    pub async fn monitor_servers(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        loop {
            interval.tick().await;

            let mut servers = self.servers.write().await;
            for (id, handle) in servers.iter_mut() {
                if let Some(ref mut child) = handle.child {
                    if let Ok(Some(status)) = child.try_wait() {
                        handle.status =
                            ServerStatus::Failed(format!("Process exited with status: {:?}", status));
                        tracing::error!(server_id = %id, status = ?status, "MCP server crashed");
                    }
                }
            }
        }
    }
}

impl Default for McpServerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Server info for external consumption
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub status: ServerStatus,
}
