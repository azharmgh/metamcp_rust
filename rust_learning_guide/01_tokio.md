# Tokio - Async Runtime

Tokio is Rust's most popular asynchronous runtime. It provides the foundation for writing non-blocking, concurrent applications in Rust.

## What is Tokio?

Tokio is an asynchronous runtime that provides:
- **Async I/O** - Non-blocking network and file operations
- **Task scheduling** - Efficient management of thousands of concurrent tasks
- **Synchronization primitives** - Channels, mutexes, semaphores for async code
- **Timers** - Delays, intervals, and timeouts

## Why Use Tokio?

In traditional synchronous programming, operations block the thread:

```rust
// Synchronous - blocks the thread
let data = read_file("data.txt");  // Thread waits here
let response = http_request();      // Thread waits here too
```

With Tokio's async/await:

```rust
// Asynchronous - thread can do other work while waiting
let data = read_file("data.txt").await;  // Other tasks run while waiting
let response = http_request().await;      // Same here
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

Common feature flags:
- `full` - All features (recommended for learning)
- `rt-multi-thread` - Multi-threaded runtime
- `macros` - The `#[tokio::main]` and `#[tokio::test]` macros
- `sync` - Synchronization primitives (channels, mutex)
- `time` - Timers and timeouts
- `net` - TCP/UDP networking
- `io-util` - I/O utilities

## Basic Usage

### The #[tokio::main] Macro

Every async Rust program needs a runtime. The `#[tokio::main]` macro sets this up:

```rust
// File: src/main.rs (Lines 8-56)
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Your async code here
    println!("Hello from async Rust!");
    Ok(())
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20std%3A%3Aerror%3A%3AError%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20-%3E%20Result%3C()%2C%20Box%3Cdyn%20Error%3E%3E%20%7B%0A%20%20%20%20println!(%22Hello%20from%20async%20Rust!%22)%3B%0A%20%20%20%20Ok(())%0A%7D)

### Async Functions

Declare async functions with the `async` keyword:

```rust
async fn fetch_data(url: &str) -> Result<String, Error> {
    // This function can use .await
    let response = make_request(url).await?;
    Ok(response)
}
```

## Real Examples from MetaMCP

### Starting an Async Server

From `src/main.rs`:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize database (async operation)
    let db = Database::new(&config.database_url).await?;
    db.run_migrations().await?;

    // Create application state
    let auth_service = Arc::new(AuthService::new(
        config.jwt_secret.clone(),
        &config.encryption_key,
        db.clone(),
    ));
    let state = api::AppState { db, auth: auth_service };

    // Build the web application
    let app = api::create_router(state);

    // Bind to address and start server
    let bind_addr = config.bind_address();
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    tracing::info!("Server listening on {}", bind_addr);

    // Run the server (this blocks until shutdown)
    axum::serve(listener, app).await?;

    Ok(())
}
```

[Run simplified version in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Simulate%20async%20database%20connection%0A%20%20%20%20let%20db%20%3D%20connect_database().await%3B%0A%20%20%20%20println!(%22Database%20connected%3A%20%7B%7D%22%2C%20db)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Simulate%20async%20server%20start%0A%20%20%20%20let%20addr%20%3D%20%22127.0.0.1%3A8080%22%3B%0A%20%20%20%20println!(%22Server%20would%20listen%20on%20%7B%7D%22%2C%20addr)%3B%0A%7D%0A%0Aasync%20fn%20connect_database()%20-%3E%20String%20%7B%0A%20%20%20%20%2F%2F%20Simulate%20async%20delay%0A%20%20%20%20tokio%3A%3Atime%3A%3Asleep(std%3A%3Atime%3A%3ADuration%3A%3Afrom_millis(100)).await%3B%0A%20%20%20%20%22PostgreSQL%22.to_string()%0A%7D)

### Spawning Background Tasks

From `src/mcp/server_manager.rs` - spawning a task to read stderr:

```rust
use tokio::io::{AsyncBufReadExt, BufReader};

// Spawn a background task to read server stderr
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
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20std%3A%3Atime%3A%3ADuration%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Spawn%20a%20background%20task%0A%20%20%20%20let%20handle%20%3D%20tokio%3A%3Aspawn(async%20%7B%0A%20%20%20%20%20%20%20%20for%20i%20in%201..%3D5%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Background%20task%3A%20count%20%7B%7D%22%2C%20i)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(100)).await%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20%22Task%20completed%22%0A%20%20%20%20%7D)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Main%20task%20continues%0A%20%20%20%20println!(%22Main%20task%20started%22)%3B%0A%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(250)).await%3B%0A%20%20%20%20println!(%22Main%20task%20middle%22)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Wait%20for%20background%20task%0A%20%20%20%20let%20result%20%3D%20handle.await.unwrap()%3B%0A%20%20%20%20println!(%22Result%3A%20%7B%7D%22%2C%20result)%3B%0A%7D)

### Synchronization Primitives

From `src/streaming/manager.rs` - using channels for communication:

```rust
use tokio::sync::{broadcast, mpsc, RwLock};
use std::sync::Arc;
use std::collections::HashMap;

pub struct StreamManager {
    // Broadcast channel - one sender, many receivers
    broadcast_tx: broadcast::Sender<StreamEvent>,

    // Per-client channels stored in a thread-safe map
    client_channels: Arc<RwLock<HashMap<String, ClientConnection>>>,

    // Per-server channels
    server_channels: Arc<RwLock<HashMap<String, broadcast::Sender<StreamEvent>>>>,
}

impl StreamManager {
    pub fn new() -> Self {
        // Create broadcast channel with buffer of 1024 messages
        let (broadcast_tx, _) = broadcast::channel(1024);

        Self {
            broadcast_tx,
            client_channels: Arc::new(RwLock::new(HashMap::new())),
            server_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Register a client and get a receiver channel
    pub async fn register_client(
        &self,
        filters: EventFilters,
    ) -> (String, mpsc::Receiver<StreamEvent>) {
        let client_id = Uuid::new_v4().to_string();

        // Create MPSC channel (multi-producer, single-consumer)
        let (tx, rx) = mpsc::channel(256);

        let connection = ClientConnection { tx, filters };

        // Write lock to modify the map
        self.client_channels
            .write()
            .await
            .insert(client_id.clone(), connection);

        (client_id, rx)
    }

    // Broadcast an event to all clients
    pub async fn broadcast(&self, event: StreamEvent) {
        // Send to global broadcast channel
        let _ = self.broadcast_tx.send(event.clone());

        // Also send to individual clients based on filters
        let clients = self.client_channels.read().await;
        for (_, client) in clients.iter() {
            if client.filters.should_send(&event) {
                let _ = client.tx.send(event.clone()).await;
            }
        }
    }
}
```

[Run simplified channel example in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20tokio%3A%3Async%3A%3Ampsc%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Create%20a%20channel%20with%20buffer%20size%2032%0A%20%20%20%20let%20(tx%2C%20mut%20rx)%20%3D%20mpsc%3A%3Achannel%3A%3A%3Ci32%3E(32)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Clone%20sender%20for%20multiple%20producers%0A%20%20%20%20let%20tx2%20%3D%20tx.clone()%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Spawn%20producer%201%0A%20%20%20%20tokio%3A%3Aspawn(async%20move%20%7B%0A%20%20%20%20%20%20%20%20for%20i%20in%200..5%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20tx.send(i).await.unwrap()%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Spawn%20producer%202%0A%20%20%20%20tokio%3A%3Aspawn(async%20move%20%7B%0A%20%20%20%20%20%20%20%20for%20i%20in%20100..105%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20tx2.send(i).await.unwrap()%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Receive%20all%20messages%0A%20%20%20%20while%20let%20Some(msg)%20%3D%20rx.recv().await%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Received%3A%20%7B%7D%22%2C%20msg)%3B%0A%20%20%20%20%7D%0A%7D)

### Timers and Intervals

From `src/mcp/server_manager.rs` - periodic monitoring:

```rust
pub async fn monitor_servers(&self) {
    // Create an interval that ticks every 10 seconds
    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_secs(10)
    );

    loop {
        // Wait for next tick
        interval.tick().await;

        // Check server health
        let mut servers = self.servers.write().await;
        for (id, handle) in servers.iter_mut() {
            if let Some(ref mut child) = handle.child {
                // Non-blocking check if process has exited
                if let Ok(Some(status)) = child.try_wait() {
                    handle.status = ServerStatus::Failed(
                        format!("Process exited: {:?}", status)
                    );
                    tracing::error!(
                        server_id = %id,
                        status = ?status,
                        "MCP server crashed"
                    );
                }
            }
        }
    }
}
```

[Run timer example in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20std%3A%3Atime%3A%3ADuration%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20%2F%2F%20One-shot%20delay%0A%20%20%20%20println!(%22Waiting%201%20second...%22)%3B%0A%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_secs(1)).await%3B%0A%20%20%20%20println!(%22Done%20waiting!%22)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Interval%20timer%0A%20%20%20%20let%20mut%20interval%20%3D%20tokio%3A%3Atime%3A%3Ainterval(Duration%3A%3Afrom_millis(200))%3B%0A%20%20%20%20%0A%20%20%20%20for%20i%20in%200..5%20%7B%0A%20%20%20%20%20%20%20%20interval.tick().await%3B%0A%20%20%20%20%20%20%20%20println!(%22Tick%20%7B%7D%22%2C%20i)%3B%0A%20%20%20%20%7D%0A%7D)

### TCP Listener

From `src/main.rs` - binding a TCP listener:

```rust
// Create async TCP listener
let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

tracing::info!("Server listening on {}", bind_addr);

// Serve accepts connections in a loop
axum::serve(listener, app).await?;
```

## Common Patterns

### Concurrent Operations

Run multiple async operations concurrently:

```rust
use tokio::join;

async fn fetch_all_data() -> (UserData, OrderData, InventoryData) {
    // All three run concurrently
    let (users, orders, inventory) = join!(
        fetch_users(),
        fetch_orders(),
        fetch_inventory()
    );

    (users, orders, inventory)
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20std%3A%3Atime%3A%3ADuration%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20let%20start%20%3D%20std%3A%3Atime%3A%3AInstant%3A%3Anow()%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Run%20concurrently%20-%20takes%20~300ms%20total%0A%20%20%20%20let%20(a%2C%20b%2C%20c)%20%3D%20tokio%3A%3Ajoin!(%0A%20%20%20%20%20%20%20%20fetch_a()%2C%0A%20%20%20%20%20%20%20%20fetch_b()%2C%0A%20%20%20%20%20%20%20%20fetch_c()%0A%20%20%20%20)%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Results%3A%20%7B%7D%2C%20%7B%7D%2C%20%7B%7D%22%2C%20a%2C%20b%2C%20c)%3B%0A%20%20%20%20println!(%22Time%3A%20%7B%3A%3F%7D%22%2C%20start.elapsed())%3B%0A%7D%0A%0Aasync%20fn%20fetch_a()%20-%3E%20i32%20%7B%0A%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(300)).await%3B%0A%20%20%20%201%0A%7D%0A%0Aasync%20fn%20fetch_b()%20-%3E%20i32%20%7B%0A%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(200)).await%3B%0A%20%20%20%202%0A%7D%0A%0Aasync%20fn%20fetch_c()%20-%3E%20i32%20%7B%0A%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(100)).await%3B%0A%20%20%20%203%0A%7D)

### Select (First Completion)

Wait for the first of multiple operations:

```rust
use tokio::select;

async fn with_timeout<T>(
    future: impl Future<Output = T>,
    timeout: Duration,
) -> Option<T> {
    select! {
        result = future => Some(result),
        _ = tokio::time::sleep(timeout) => None,
    }
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20std%3A%3Atime%3A%3ADuration%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Race%20two%20operations%0A%20%20%20%20tokio%3A%3Aselect!%20%7B%0A%20%20%20%20%20%20%20%20val%20%3D%20slow_operation()%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Operation%20completed%3A%20%7B%7D%22%2C%20val)%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20_%20%3D%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(100))%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Timeout!%22)%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D%0A%0Aasync%20fn%20slow_operation()%20-%3E%20i32%20%7B%0A%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(500)).await%3B%0A%20%20%20%2042%0A%7D)

### Shared State with RwLock

From MetaMCP - thread-safe shared state:

```rust
use tokio::sync::RwLock;
use std::sync::Arc;

struct AppState {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl AppState {
    // Read access - multiple readers allowed
    async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }

    // Write access - exclusive access
    async fn set(&self, key: String, value: String) {
        let mut data = self.data.write().await;
        data.insert(key, value);
    }
}
```

## Best Practices

### DO

1. **Use `#[tokio::main]`** for async entry points
2. **Prefer channels over shared state** for communication between tasks
3. **Use `Arc<RwLock<T>>`** for shared state that needs both read and write access
4. **Spawn long-running tasks** with `tokio::spawn`
5. **Use timeouts** for operations that might hang

### DON'T

1. **Don't block in async code** - use `tokio::task::spawn_blocking` for CPU-intensive work
2. **Don't hold locks across `.await`** points (use `tokio::sync` not `std::sync`)
3. **Don't create a new runtime inside an async context**
4. **Don't ignore JoinHandle results** - errors in spawned tasks won't propagate otherwise

### Blocking Code in Async Context

```rust
// WRONG - blocks the async runtime
async fn process_file() {
    let contents = std::fs::read_to_string("file.txt").unwrap();  // Blocks!
}

// RIGHT - use async version
async fn process_file() {
    let contents = tokio::fs::read_to_string("file.txt").await.unwrap();
}

// RIGHT - offload blocking work
async fn cpu_intensive_work(data: Vec<u8>) -> Vec<u8> {
    tokio::task::spawn_blocking(move || {
        // CPU-intensive computation here
        expensive_computation(data)
    })
    .await
    .unwrap()
}
```

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Performance** | Handle thousands of concurrent connections efficiently |
| **Ecosystem** | Most async Rust libraries built on Tokio |
| **Mature** | Battle-tested in production at large scale |
| **Full-featured** | Timers, channels, I/O, networking all included |
| **Good docs** | Excellent documentation and tutorials |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Complexity** | Async Rust has a learning curve |
| **Binary size** | Adds to compile time and binary size |
| **Debugging** | Async stack traces can be confusing |
| **Colored functions** | Async/sync boundary requires care |

## When to Use Tokio

**Use Tokio when:**
- Building web servers or APIs
- Writing network clients
- Handling many concurrent I/O operations
- Building real-time applications
- You need timers, channels, or other async primitives

**Consider alternatives when:**
- Your app is purely CPU-bound (use `rayon` for parallelism)
- Simple scripts with minimal I/O
- Embedded systems with limited resources

## Further Learning

### Official Resources
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) - Official getting started guide
- [Tokio API Documentation](https://docs.rs/tokio) - Complete API reference
- [Tokio Mini-Redis](https://github.com/tokio-rs/mini-redis) - Learning project

### Books & Articles
- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/) - The official async book
- [Tokio Internals](https://tokio.rs/blog/2019-10-scheduler) - How the scheduler works

### Practice Projects
1. Build a simple chat server
2. Create an async file processor
3. Build a rate-limited HTTP client
4. Implement a task queue with workers

## Related Crates in This Project

- **tokio-stream** - Stream utilities for async iterators
- **futures** - Additional async utilities
- **axum** - Web framework built on Tokio
- **sqlx** - Async database driver using Tokio
- **reqwest** - Async HTTP client using Tokio
