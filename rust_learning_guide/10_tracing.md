# Tracing - Logging & Observability

Tracing is Rust's modern observability framework that provides structured, contextual logging with support for distributed tracing.

## What is Tracing?

Tracing provides:
- **Structured logging** - Key-value pairs instead of plain text
- **Spans** - Track execution context across async operations
- **Events** - Log individual occurrences within spans
- **Subscribers** - Pluggable output (console, files, services)

## Why Tracing over log?

| Feature | log | tracing |
|---------|-----|---------|
| Structured data | Limited | Full support |
| Async support | Poor | Excellent |
| Span context | No | Yes |
| Performance | Good | Better |

## Installation

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

## Basic Setup

From `src/main.rs`:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing(log_level: &str) {
    tracing_subscriber::registry()
        // Filter by log level from environment or config
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        // Format and output logs
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing("info,metamcp=debug");

    tracing::info!("Starting MetaMCP server...");

    // ... rest of application
}
```

[Run tracing example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20tracing%3A%3A%7Binfo%2C%20warn%2C%20error%2C%20debug%2C%20trace%2C%20instrument%7D%3B%0Ause%20tracing_subscriber%3B%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Initialize%20simple%20subscriber%0A%20%20%20%20tracing_subscriber%3A%3Afmt%3A%3Ainit()%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Log%20at%20different%20levels%0A%20%20%20%20trace!(%22Very%20detailed%20info%22)%3B%0A%20%20%20%20debug!(%22Debug%20information%22)%3B%0A%20%20%20%20info!(%22Normal%20operation%22)%3B%0A%20%20%20%20warn!(%22Something%20might%20be%20wrong%22)%3B%0A%20%20%20%20error!(%22Something%20failed!%22)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Structured%20logging%0A%20%20%20%20let%20user_id%20%3D%20123%3B%0A%20%20%20%20let%20action%20%3D%20%22login%22%3B%0A%20%20%20%20info!(user_id%2C%20action%2C%20%22User%20performed%20action%22)%3B%0A%7D)

## Log Levels

```rust
use tracing::{trace, debug, info, warn, error};

trace!("Very detailed debugging info");    // TRACE
debug!("Useful for debugging");            // DEBUG
info!("Normal operational messages");       // INFO
warn!("Something unexpected happened");     // WARN
error!("Something failed!");               // ERROR
```

## Structured Logging

### Basic Fields

```rust
// Named fields
info!(user_id = 123, action = "login", "User logged in");

// Display formatting
info!(user = %user.name, "Processing user");

// Debug formatting
info!(request = ?request, "Received request");

// Multiple fields
info!(
    user_id = user.id,
    email = %user.email,
    status = ?user.status,
    "User status changed"
);
```

### Real Examples from MetaMCP

From `src/mcp/server_manager.rs`:

```rust
// Log with multiple structured fields
tracing::warn!(
    server_id = %server_id_clone,
    server_name = %server_name,
    "MCP server stderr: {}",
    line
);

// Log server events
tracing::info!(server_id = %server_id, "MCP server spawned");

// Error with context
tracing::error!(
    server_id = %id,
    status = ?status,
    "MCP server crashed"
);
```

From `src/utils/error.rs`:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                // ...
            }
            AppError::Database(err) => {
                tracing::error!("Database error: {}", err);
                // ...
            }
            AppError::Jwt(err) => {
                tracing::warn!("JWT error: {}", err);
                // ...
            }
            // ...
        }
    }
}
```

From `src/streaming/manager.rs`:

```rust
tracing::debug!(client_id = %client_id, "Client registered for streaming");
tracing::debug!(client_id = %client_id, "Client unregistered from streaming");
```

## Spans

Spans track execution context:

```rust
use tracing::{info_span, span, Level, Instrument};

// Create a span
let span = info_span!("process_request", request_id = %id);

// Enter the span (sync)
let _guard = span.enter();
info!("Processing...");  // This log is inside the span

// Async with .instrument()
async fn process_request(id: u64) {
    let span = info_span!("process_request", request_id = id);

    async {
        info!("Starting processing");
        do_work().await;
        info!("Finished processing");
    }
    .instrument(span)
    .await;
}
```

### #[instrument] Macro

```rust
use tracing::instrument;

#[instrument]
async fn get_user(user_id: u64) -> Result<User, Error> {
    // Automatically creates a span with function name and arguments
    info!("Fetching user");
    db.get_user(user_id).await
}

// With custom span name and skipping fields
#[instrument(name = "fetch_user", skip(db), fields(user_id = %id))]
async fn get_user(db: &Database, id: u64) -> Result<User, Error> {
    db.get_user(id).await
}
```

[Run span example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20tracing%3A%3A%7Binfo%2C%20info_span%2C%20instrument%7D%3B%0A%0A%23%5Binstrument%5D%0Afn%20outer_function(value%3A%20i32)%20%7B%0A%20%20%20%20info!(%22Starting%20outer%22)%3B%0A%20%20%20%20inner_function(value%20*%202)%3B%0A%20%20%20%20info!(%22Finished%20outer%22)%3B%0A%7D%0A%0A%23%5Binstrument%5D%0Afn%20inner_function(value%3A%20i32)%20%7B%0A%20%20%20%20info!(%22Processing%20value%22)%3B%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20tracing_subscriber%3A%3Afmt%3A%3Ainit()%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Manual%20span%0A%20%20%20%20let%20span%20%3D%20info_span!(%22main_operation%22%2C%20request_id%20%3D%2042)%3B%0A%20%20%20%20let%20_guard%20%3D%20span.enter()%3B%0A%20%20%20%20%0A%20%20%20%20info!(%22Inside%20main%20span%22)%3B%0A%20%20%20%20outer_function(10)%3B%0A%20%20%20%20info!(%22Leaving%20main%20span%22)%3B%0A%7D)

## Subscriber Configuration

### Environment Filter

```rust
use tracing_subscriber::EnvFilter;

// From environment variable RUST_LOG
let filter = EnvFilter::from_default_env();

// Or set directly
let filter = EnvFilter::new("info,metamcp=debug,sqlx=warn");

// Filter syntax:
// - "info" - Set default level to info
// - "myapp=debug" - Set myapp module to debug
// - "sqlx=warn" - Only warn and above from sqlx
// - "[request_id]" - Filter by span field
```

### JSON Output

For production/log aggregation:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, fmt};

tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())
    .with(fmt::layer().json())  // JSON format
    .init();
```

Output:
```json
{"timestamp":"2024-01-15T10:30:00.000Z","level":"INFO","target":"metamcp","message":"Server started","port":8080}
```

### Pretty Console Output

For development:

```rust
tracing_subscriber::fmt()
    .pretty()                    // Pretty format
    .with_target(true)          // Show module path
    .with_thread_names(true)    // Show thread names
    .with_file(true)            // Show file name
    .with_line_number(true)     // Show line numbers
    .init();
```

### Multiple Layers

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, fmt, Layer};

tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())
    // Console output (human readable)
    .with(
        fmt::layer()
            .with_target(true)
            .with_filter(EnvFilter::new("info"))
    )
    // File output (JSON for log aggregation)
    .with(
        fmt::layer()
            .json()
            .with_writer(file_appender)
            .with_filter(EnvFilter::new("debug"))
    )
    .init();
```

## Tower-HTTP Integration

From `src/api/mod.rs`:

```rust
use tower_http::trace::TraceLayer;

Router::new()
    .route("/api/v1/users", get(list_users))
    // Automatically trace all HTTP requests
    .layer(TraceLayer::new_for_http())
```

This automatically logs:
- Request method and path
- Response status code
- Request duration
- Error details

## Common Patterns

### Request ID Propagation

```rust
use uuid::Uuid;

pub async fn request_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    let request_id = Uuid::new_v4();

    // Create span with request ID
    let span = info_span!(
        "http_request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri()
    );

    async move {
        info!("Request started");
        let response = next.run(request).await;
        info!(status = %response.status(), "Request completed");
        response
    }
    .instrument(span)
    .await
}
```

### Database Query Logging

```rust
#[instrument(skip(pool), fields(query = %"SELECT * FROM users"))]
async fn get_users(pool: &PgPool) -> Result<Vec<User>> {
    let start = Instant::now();

    let users = sqlx::query_as("SELECT * FROM users")
        .fetch_all(pool)
        .await?;

    debug!(
        rows = users.len(),
        duration_ms = start.elapsed().as_millis(),
        "Query completed"
    );

    Ok(users)
}
```

### Error Logging

```rust
async fn process_request(id: u64) -> Result<Response, AppError> {
    let result = fetch_data(id).await;

    match &result {
        Ok(data) => {
            info!(data_size = data.len(), "Data fetched successfully");
        }
        Err(e) => {
            error!(
                error = %e,
                error_type = std::any::type_name_of_val(e),
                "Failed to fetch data"
            );
        }
    }

    result
}
```

## Best Practices

### DO

1. **Use structured fields** - `info!(user_id = 123)` not `info!("user_id: 123")`
2. **Use appropriate levels** - Don't log everything at INFO
3. **Include context** - Request IDs, user IDs, etc.
4. **Use spans for async** - Track execution across await points
5. **Filter in production** - Don't log DEBUG in production

### DON'T

1. **Don't log sensitive data** - Passwords, API keys, PII
2. **Don't log too much** - It hurts performance
3. **Don't use string formatting for fields** - Use structured fields
4. **Don't ignore errors in logging** - Log them properly

### Sensitive Data

```rust
// BAD - exposes password
info!(password = %user_password, "Login attempt");

// GOOD - redact sensitive data
info!(
    username = %username,
    password_length = user_password.len(),
    "Login attempt"
);

// GOOD - skip sensitive fields
#[instrument(skip(password))]
async fn login(username: &str, password: &str) -> Result<Token> {
    // password is not logged
}
```

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Structured** | Key-value logging for analysis |
| **Async-friendly** | Spans work across await points |
| **Performant** | Disabled levels have zero cost |
| **Flexible** | Multiple subscribers/outputs |
| **Ecosystem** | Integrates with many tools |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Learning curve** | More complex than simple logging |
| **Compile time** | Macros add to compile time |
| **Configuration** | Subscriber setup can be complex |

## When to Use

**Use tracing when:**
- Building async applications
- Need structured logging
- Want distributed tracing
- Building production services

**Consider alternatives when:**
- Simple scripts (use `println!`)
- Legacy codebase using `log` (use `tracing-log` bridge)

## Further Learning

### Official Resources
- [Tracing Documentation](https://docs.rs/tracing)
- [Tracing Subscriber](https://docs.rs/tracing-subscriber)
- [Tokio Tracing Tutorial](https://tokio.rs/tokio/topics/tracing)

### Practice
1. Add tracing to an existing application
2. Create custom spans for database queries
3. Set up JSON logging for production
4. Add request ID propagation

## Related Crates

- **tracing-log** - Bridge to `log` crate
- **tracing-opentelemetry** - OpenTelemetry integration
- **tracing-appender** - Non-blocking file writer
- **tower-http** - HTTP middleware with tracing
- **tracing-error** - Error context from spans
