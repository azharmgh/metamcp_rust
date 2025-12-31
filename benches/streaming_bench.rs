//! Benchmarks for streaming operations

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use metamcp::streaming::{StreamManager, StreamEvent, EventFilters};
use serde_json::json;
use tokio::runtime::Runtime;

/// Create a test runtime for async benchmarks
fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

/// Benchmark stream event serialization
fn bench_event_serialization(c: &mut Criterion) {
    let events = vec![
        StreamEvent::McpServerStarted {
            server_id: "srv-12345678".to_string(),
            name: "Test Server".to_string(),
        },
        StreamEvent::McpServerStopped {
            server_id: "srv-12345678".to_string(),
            reason: "Shutdown requested".to_string(),
        },
        StreamEvent::McpToolExecuted {
            server_id: "srv-12345678".to_string(),
            tool: "echo".to_string(),
            status: "success".to_string(),
        },
        StreamEvent::McpMessage {
            server_id: "srv-12345678".to_string(),
            message: json!({"method": "ping", "params": {}}),
        },
        StreamEvent::SystemHealth {
            cpu: 45.5,
            memory: 60.2,
            active_servers: 5,
        },
        StreamEvent::Error {
            code: "E001".to_string(),
            message: "Something went wrong".to_string(),
        },
    ];

    let mut group = c.benchmark_group("event_serialization");

    for (i, event) in events.iter().enumerate() {
        let name = match event {
            StreamEvent::McpServerStarted { .. } => "server_started",
            StreamEvent::McpServerStopped { .. } => "server_stopped",
            StreamEvent::McpToolExecuted { .. } => "tool_executed",
            StreamEvent::McpMessage { .. } => "message",
            StreamEvent::SystemHealth { .. } => "health",
            StreamEvent::Error { .. } => "error",
        };

        group.bench_with_input(BenchmarkId::new("serialize", name), event, |b, event| {
            b.iter(|| serde_json::to_string(black_box(event)).unwrap())
        });
    }

    group.finish();
}

/// Benchmark event filter evaluation
fn bench_event_filter_evaluation(c: &mut Criterion) {
    let event = StreamEvent::McpServerStarted {
        server_id: "srv-12345678".to_string(),
        name: "Test Server".to_string(),
    };

    let filters = vec![
        ("no_filter", EventFilters::default()),
        ("type_filter", EventFilters {
            event_types: Some(vec!["mcp_server_started".to_string()]),
            server_ids: vec![],
            include_system: false,
        }),
        ("server_filter", EventFilters {
            event_types: None,
            server_ids: vec!["srv-12345678".to_string()],
            include_system: false,
        }),
        ("combined_filter", EventFilters {
            event_types: Some(vec!["mcp_server_started".to_string()]),
            server_ids: vec!["srv-12345678".to_string()],
            include_system: true,
        }),
    ];

    let mut group = c.benchmark_group("filter_evaluation");

    for (name, filter) in filters.iter() {
        group.bench_with_input(BenchmarkId::new("should_send", *name), filter, |b, filter| {
            b.iter(|| filter.should_send(black_box(&event)))
        });
    }

    group.finish();
}

/// Benchmark stream manager client registration
fn bench_stream_manager_registration(c: &mut Criterion) {
    let rt = create_runtime();

    c.bench_function("manager_register_client", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = StreamManager::new();
                let filters = EventFilters::default();
                let (_id, _rx) = manager.register_client(black_box(filters)).await;
            })
        })
    });
}

/// Benchmark stream manager broadcast to multiple clients
fn bench_stream_manager_broadcast(c: &mut Criterion) {
    let rt = create_runtime();

    let mut group = c.benchmark_group("manager_broadcast");

    for num_clients in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_clients),
            num_clients,
            |b, &num_clients| {
                b.iter(|| {
                    rt.block_on(async {
                        let manager = StreamManager::new();

                        // Register clients
                        let mut _receivers = Vec::new();
                        for _ in 0..num_clients {
                            let (_id, rx) = manager.register_client(EventFilters::default()).await;
                            _receivers.push(rx);
                        }

                        // Broadcast event
                        let event = StreamEvent::McpServerStarted {
                            server_id: "srv-1".to_string(),
                            name: "Test".to_string(),
                        };
                        manager.broadcast(black_box(event)).await;
                    })
                })
            },
        );
    }

    group.finish();
}

/// Benchmark stream manager with filtered broadcasts
fn bench_stream_manager_filtered_broadcast(c: &mut Criterion) {
    let rt = create_runtime();

    c.bench_function("manager_filtered_broadcast", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = StreamManager::new();

                // Register clients with different filters
                let _rx1 = manager.register_client(EventFilters {
                    event_types: Some(vec!["mcp_server_started".to_string()]),
                    server_ids: vec![],
                    include_system: false,
                }).await;

                let _rx2 = manager.register_client(EventFilters {
                    event_types: None,
                    server_ids: vec!["srv-1".to_string()],
                    include_system: false,
                }).await;

                let _rx3 = manager.register_client(EventFilters {
                    event_types: Some(vec!["mcp_server_stopped".to_string()]),
                    server_ids: vec!["srv-2".to_string()],
                    include_system: true,
                }).await;

                // Broadcast events
                let event1 = StreamEvent::McpServerStarted {
                    server_id: "srv-1".to_string(),
                    name: "Test".to_string(),
                };
                manager.broadcast(black_box(event1)).await;

                let event2 = StreamEvent::McpToolExecuted {
                    server_id: "srv-1".to_string(),
                    tool: "echo".to_string(),
                    status: "success".to_string(),
                };
                manager.broadcast(black_box(event2)).await;
            })
        })
    });
}

/// Benchmark stream manager server registration
fn bench_stream_manager_server_registration(c: &mut Criterion) {
    let rt = create_runtime();

    c.bench_function("manager_register_server", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = StreamManager::new();
                manager.register_server(black_box("srv-12345678".to_string())).await;
            })
        })
    });
}

/// Benchmark stream manager handle MCP event
fn bench_stream_manager_handle_mcp_event(c: &mut Criterion) {
    let rt = create_runtime();

    c.bench_function("manager_handle_mcp_event", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = StreamManager::new();
                manager.register_server("srv-1".to_string()).await;

                let event = StreamEvent::McpToolExecuted {
                    server_id: "srv-1".to_string(),
                    tool: "echo".to_string(),
                    status: "success".to_string(),
                };

                manager.handle_mcp_event(
                    black_box("srv-1".to_string()),
                    black_box(event),
                ).await;
            })
        })
    });
}

/// Benchmark stream event deserialization
fn bench_event_deserialization(c: &mut Criterion) {
    let json_events = vec![
        (
            "server_started",
            r#"{"type":"mcp_server_started","server_id":"srv-1","name":"Test"}"#,
        ),
        (
            "server_stopped",
            r#"{"type":"mcp_server_stopped","server_id":"srv-1","reason":"Done"}"#,
        ),
        (
            "tool_executed",
            r#"{"type":"mcp_tool_executed","server_id":"srv-1","tool":"echo","status":"ok"}"#,
        ),
        (
            "health",
            r#"{"type":"system_health","cpu":45.5,"memory":60.2,"active_servers":3}"#,
        ),
    ];

    let mut group = c.benchmark_group("event_deserialization");

    for (name, json) in json_events.iter() {
        group.bench_with_input(BenchmarkId::new("deserialize", *name), json, |b, json| {
            b.iter(|| serde_json::from_str::<StreamEvent>(black_box(json)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_event_serialization,
    bench_event_filter_evaluation,
    bench_stream_manager_registration,
    bench_stream_manager_broadcast,
    bench_stream_manager_filtered_broadcast,
    bench_stream_manager_server_registration,
    bench_stream_manager_handle_mcp_event,
    bench_event_deserialization,
);

criterion_main!(benches);
