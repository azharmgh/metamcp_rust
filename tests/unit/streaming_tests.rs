//! Unit tests for streaming module

use metamcp::streaming::{StreamEvent, EventFilters, StreamManager};
use serde_json::json;

#[test]
fn test_stream_event_mcp_server_started() {
    let event = StreamEvent::McpServerStarted {
        server_id: "srv-1".to_string(),
        name: "Test Server".to_string(),
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("\"type\":\"mcp_server_started\""));
    assert!(json.contains("\"server_id\":\"srv-1\""));
    assert!(json.contains("\"name\":\"Test Server\""));
}

#[test]
fn test_stream_event_mcp_server_stopped() {
    let event = StreamEvent::McpServerStopped {
        server_id: "srv-1".to_string(),
        reason: "Shutdown requested".to_string(),
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("\"type\":\"mcp_server_stopped\""));
    assert!(json.contains("\"reason\":\"Shutdown requested\""));
}

#[test]
fn test_stream_event_mcp_tool_executed() {
    let event = StreamEvent::McpToolExecuted {
        server_id: "srv-1".to_string(),
        tool: "echo".to_string(),
        status: "success".to_string(),
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("\"type\":\"mcp_tool_executed\""));
    assert!(json.contains("\"tool\":\"echo\""));
}

#[test]
fn test_stream_event_mcp_message() {
    let event = StreamEvent::McpMessage {
        server_id: "srv-1".to_string(),
        message: json!({"method": "ping"}),
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("\"type\":\"mcp_message\""));
    assert!(json.contains("\"message\""));
}

#[test]
fn test_stream_event_system_health() {
    let event = StreamEvent::SystemHealth {
        cpu: 45.5,
        memory: 60.2,
        active_servers: 3,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("\"type\":\"system_health\""));
    assert!(json.contains("\"cpu\":45.5"));
    assert!(json.contains("\"memory\":60.2"));
    assert!(json.contains("\"active_servers\":3"));
}

#[test]
fn test_stream_event_error() {
    let event = StreamEvent::Error {
        code: "E001".to_string(),
        message: "Something went wrong".to_string(),
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("\"type\":\"error\""));
    assert!(json.contains("\"code\":\"E001\""));
}

#[test]
fn test_event_filters_default() {
    let filters = EventFilters::default();

    assert!(filters.event_types.is_none());
    assert!(filters.server_ids.is_empty());
    assert!(!filters.include_system);
}

#[test]
fn test_event_filters_should_send_no_filters() {
    let filters = EventFilters::default();

    // With no filters, all events except system should pass
    let event = StreamEvent::McpServerStarted {
        server_id: "srv-1".to_string(),
        name: "Test".to_string(),
    };
    assert!(filters.should_send(&event));

    // System events should not pass without include_system
    let system_event = StreamEvent::SystemHealth {
        cpu: 50.0,
        memory: 50.0,
        active_servers: 1,
    };
    assert!(!filters.should_send(&system_event));
}

#[test]
fn test_event_filters_include_system() {
    let filters = EventFilters {
        event_types: None,
        server_ids: vec![],
        include_system: true,
    };

    let event = StreamEvent::SystemHealth {
        cpu: 50.0,
        memory: 50.0,
        active_servers: 1,
    };
    assert!(filters.should_send(&event));
}

#[test]
fn test_event_filters_by_event_type() {
    let filters = EventFilters {
        event_types: Some(vec!["mcp_server_started".to_string()]),
        server_ids: vec![],
        include_system: false,
    };

    // Should match
    let started_event = StreamEvent::McpServerStarted {
        server_id: "srv-1".to_string(),
        name: "Test".to_string(),
    };
    assert!(filters.should_send(&started_event));

    // Should not match
    let stopped_event = StreamEvent::McpServerStopped {
        server_id: "srv-1".to_string(),
        reason: "Done".to_string(),
    };
    assert!(!filters.should_send(&stopped_event));
}

#[test]
fn test_event_filters_by_server_id() {
    let filters = EventFilters {
        event_types: None,
        server_ids: vec!["srv-1".to_string()],
        include_system: false,
    };

    // Should match (correct server ID)
    let event1 = StreamEvent::McpServerStarted {
        server_id: "srv-1".to_string(),
        name: "Test".to_string(),
    };
    assert!(filters.should_send(&event1));

    // Should not match (different server ID)
    let event2 = StreamEvent::McpServerStarted {
        server_id: "srv-2".to_string(),
        name: "Test".to_string(),
    };
    assert!(!filters.should_send(&event2));

    // Error events should pass (no server_id filtering for them)
    let error_event = StreamEvent::Error {
        code: "E001".to_string(),
        message: "Error".to_string(),
    };
    assert!(filters.should_send(&error_event));
}

#[test]
fn test_event_filters_combined() {
    let filters = EventFilters {
        event_types: Some(vec![
            "mcp_server_started".to_string(),
            "mcp_tool_executed".to_string(),
        ]),
        server_ids: vec!["srv-1".to_string()],
        include_system: false,
    };

    // Should match (correct type and server)
    let event1 = StreamEvent::McpServerStarted {
        server_id: "srv-1".to_string(),
        name: "Test".to_string(),
    };
    assert!(filters.should_send(&event1));

    // Should not match (correct type, wrong server)
    let event2 = StreamEvent::McpServerStarted {
        server_id: "srv-2".to_string(),
        name: "Test".to_string(),
    };
    assert!(!filters.should_send(&event2));

    // Should not match (wrong type, correct server)
    let event3 = StreamEvent::McpServerStopped {
        server_id: "srv-1".to_string(),
        reason: "Done".to_string(),
    };
    assert!(!filters.should_send(&event3));
}

#[test]
fn test_stream_manager_new() {
    let manager = StreamManager::new();
    // Just verify it can be created
    assert!(true);
}

#[tokio::test]
async fn test_stream_manager_register_client() {
    let manager = StreamManager::new();
    let filters = EventFilters::default();

    let (client_id, _rx) = manager.register_client(filters).await;
    assert!(!client_id.is_empty());
    assert_eq!(manager.client_count().await, 1);
}

#[tokio::test]
async fn test_stream_manager_unregister_client() {
    let manager = StreamManager::new();
    let filters = EventFilters::default();

    let (client_id, _rx) = manager.register_client(filters).await;
    assert_eq!(manager.client_count().await, 1);

    manager.unregister_client(&client_id).await;
    assert_eq!(manager.client_count().await, 0);
}

#[tokio::test]
async fn test_stream_manager_multiple_clients() {
    let manager = StreamManager::new();

    let (id1, _rx1) = manager.register_client(EventFilters::default()).await;
    let (id2, _rx2) = manager.register_client(EventFilters::default()).await;
    let (id3, _rx3) = manager.register_client(EventFilters::default()).await;

    assert_eq!(manager.client_count().await, 3);
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
}

#[tokio::test]
async fn test_stream_manager_subscribe() {
    let manager = StreamManager::new();
    let _rx = manager.subscribe();
    // Verify subscription works without panic
    assert!(true);
}

#[tokio::test]
async fn test_stream_manager_server_registration() {
    let manager = StreamManager::new();

    manager.register_server("srv-1".to_string()).await;
    manager.register_server("srv-2".to_string()).await;

    // Unregister one
    manager.unregister_server("srv-1").await;

    // Verify no panic
    assert!(true);
}

#[tokio::test]
async fn test_stream_manager_broadcast() {
    let manager = StreamManager::new();
    let filters = EventFilters {
        event_types: None,
        server_ids: vec![],
        include_system: false,
    };

    let (_client_id, mut rx) = manager.register_client(filters).await;

    // Broadcast an event
    let event = StreamEvent::McpServerStarted {
        server_id: "srv-1".to_string(),
        name: "Test".to_string(),
    };
    manager.broadcast(event.clone()).await;

    // Try to receive (with timeout)
    let received = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        rx.recv(),
    ).await;

    assert!(received.is_ok());
    if let Ok(Some(StreamEvent::McpServerStarted { server_id, name })) = received {
        assert_eq!(server_id, "srv-1");
        assert_eq!(name, "Test");
    } else {
        panic!("Did not receive expected event");
    }
}

#[tokio::test]
async fn test_stream_manager_send_to_client() {
    let manager = StreamManager::new();
    let filters = EventFilters::default();

    let (client_id, mut rx) = manager.register_client(filters).await;

    // Send directly to client
    let event = StreamEvent::Error {
        code: "E001".to_string(),
        message: "Test error".to_string(),
    };
    manager.send_to_client(&client_id, event).await;

    // Try to receive
    let received = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        rx.recv(),
    ).await;

    assert!(received.is_ok());
}

#[tokio::test]
async fn test_stream_manager_handle_mcp_event() {
    let manager = StreamManager::new();

    // Register a server
    manager.register_server("srv-1".to_string()).await;

    // Handle an MCP event
    let event = StreamEvent::McpToolExecuted {
        server_id: "srv-1".to_string(),
        tool: "echo".to_string(),
        status: "success".to_string(),
    };
    manager.handle_mcp_event("srv-1".to_string(), event).await;

    // Verify no panic
    assert!(true);
}
