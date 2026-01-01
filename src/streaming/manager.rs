//! Stream manager for handling client connections

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

/// Events that can be streamed to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// MCP server started
    McpServerStarted { server_id: String, name: String },
    /// MCP server stopped
    McpServerStopped { server_id: String, reason: String },
    /// MCP tool was executed
    McpToolExecuted {
        server_id: String,
        tool: String,
        status: String,
    },
    /// MCP message received
    McpMessage {
        server_id: String,
        message: serde_json::Value,
    },
    /// System health update
    SystemHealth {
        cpu: f32,
        memory: f32,
        active_servers: usize,
    },
    /// Error event
    Error { code: String, message: String },
}

/// Event filters for client subscriptions
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EventFilters {
    /// Filter by event types
    pub event_types: Option<Vec<String>>,
    /// Filter by server IDs
    #[serde(default)]
    pub server_ids: Vec<String>,
    /// Include system health events
    #[serde(default)]
    pub include_system: bool,
}

impl EventFilters {
    /// Check if an event should be sent based on filters
    pub fn should_send(&self, event: &StreamEvent) -> bool {
        // Filter by event type if specified
        if let Some(types) = &self.event_types {
            let event_type = match event {
                StreamEvent::McpServerStarted { .. } => "mcp_server_started",
                StreamEvent::McpServerStopped { .. } => "mcp_server_stopped",
                StreamEvent::McpToolExecuted { .. } => "mcp_tool_executed",
                StreamEvent::McpMessage { .. } => "mcp_message",
                StreamEvent::SystemHealth { .. } => "system_health",
                StreamEvent::Error { .. } => "error",
            };

            if !types.contains(&event_type.to_string()) {
                return false;
            }
        }

        // Filter system events
        if !self.include_system && matches!(event, StreamEvent::SystemHealth { .. }) {
            return false;
        }

        // Filter by server ID if applicable
        if !self.server_ids.is_empty() {
            let server_id = match event {
                StreamEvent::McpServerStarted { server_id, .. } => Some(server_id),
                StreamEvent::McpServerStopped { server_id, .. } => Some(server_id),
                StreamEvent::McpToolExecuted { server_id, .. } => Some(server_id),
                StreamEvent::McpMessage { server_id, .. } => Some(server_id),
                _ => None,
            };

            if let Some(id) = server_id {
                if !self.server_ids.contains(id) {
                    return false;
                }
            }
        }

        true
    }
}

/// Client connection info
struct ClientConnection {
    tx: mpsc::Sender<StreamEvent>,
    filters: EventFilters,
}

/// Stream manager for broadcasting events to connected clients
pub struct StreamManager {
    /// Broadcast channel for system-wide events
    broadcast_tx: broadcast::Sender<StreamEvent>,
    /// Per-client channels for targeted events
    client_channels: Arc<RwLock<HashMap<String, ClientConnection>>>,
    /// Per-server broadcast channels
    server_channels: Arc<RwLock<HashMap<String, broadcast::Sender<StreamEvent>>>>,
}

impl StreamManager {
    /// Create a new stream manager
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);

        Self {
            broadcast_tx,
            client_channels: Arc::new(RwLock::new(HashMap::new())),
            server_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Broadcast an event to all connected clients
    pub async fn broadcast(&self, event: StreamEvent) {
        let _ = self.broadcast_tx.send(event.clone());

        // Also send to individual clients based on their filters
        let clients = self.client_channels.read().await;
        for (_, client) in clients.iter() {
            if client.filters.should_send(&event) {
                let _ = client.tx.send(event.clone()).await;
            }
        }
    }

    /// Send an event to a specific client
    pub async fn send_to_client(&self, client_id: &str, event: StreamEvent) {
        let clients = self.client_channels.read().await;
        if let Some(client) = clients.get(client_id) {
            if client.filters.should_send(&event) {
                let _ = client.tx.send(event).await;
            }
        }
    }

    /// Register a new client connection
    pub async fn register_client(
        &self,
        filters: EventFilters,
    ) -> (String, mpsc::Receiver<StreamEvent>) {
        let client_id = Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::channel(256);

        let connection = ClientConnection { tx, filters };

        self.client_channels
            .write()
            .await
            .insert(client_id.clone(), connection);

        tracing::debug!(client_id = %client_id, "Client registered for streaming");

        (client_id, rx)
    }

    /// Unregister a client connection
    pub async fn unregister_client(&self, client_id: &str) {
        self.client_channels.write().await.remove(client_id);
        tracing::debug!(client_id = %client_id, "Client unregistered from streaming");
    }

    /// Subscribe to broadcast events
    pub fn subscribe(&self) -> broadcast::Receiver<StreamEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Register a server-specific broadcast channel
    pub async fn register_server(&self, server_id: String) {
        let (tx, _) = broadcast::channel(256);
        self.server_channels.write().await.insert(server_id, tx);
    }

    /// Unregister a server broadcast channel
    pub async fn unregister_server(&self, server_id: &str) {
        self.server_channels.write().await.remove(server_id);
    }

    /// Send an event to a server-specific channel
    pub async fn send_to_server(&self, server_id: &str, event: StreamEvent) {
        let channels = self.server_channels.read().await;
        if let Some(tx) = channels.get(server_id) {
            let _ = tx.send(event);
        }
    }

    /// Handle an MCP-related event
    pub async fn handle_mcp_event(&self, server_id: String, event: StreamEvent) {
        // Send to server-specific channel
        self.send_to_server(&server_id, event.clone()).await;

        // Broadcast certain events to all clients
        match &event {
            StreamEvent::McpServerStarted { .. } | StreamEvent::McpServerStopped { .. } => {
                self.broadcast(event).await;
            }
            _ => {}
        }
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        self.client_channels.read().await.len()
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared stream manager instance
pub type SharedStreamManager = Arc<StreamManager>;
