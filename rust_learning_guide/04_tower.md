# Tower & Tower-HTTP - Middleware

Tower is a modular and reusable components framework for building robust networking clients and servers. Tower-HTTP provides HTTP-specific middleware built on Tower.

## What is Tower?

Tower provides:
- **Service trait** - A universal interface for request-response operations
- **Layer trait** - A way to wrap services with additional functionality
- **Middleware** - Reusable components like timeouts, rate limiting, etc.

## Why Tower?

Tower is the foundation for Rust's async networking ecosystem:
- Used by Axum, Hyper, Tonic, and more
- Middleware can be shared across frameworks
- Composable and testable design

## Installation

```toml
[dependencies]
tower = "0.5"
tower-http = { version = "0.6", features = ["auth", "cors", "trace"] }
```

Common tower-http features:
- `cors` - Cross-Origin Resource Sharing
- `trace` - Request/response tracing
- `auth` - Authentication utilities
- `compression` - Response compression
- `timeout` - Request timeouts

## Core Concepts

### The Service Trait

A Service is anything that takes a request and returns a future of a response:

```rust
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

### The Layer Trait

A Layer wraps a service to add functionality:

```rust
pub trait Layer<S> {
    type Service;
    fn layer(&self, service: S) -> Self::Service;
}
```

[Run conceptual example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=%2F%2F%20Conceptual%20example%20of%20Tower%20Service%20pattern%0A%2F%2F%20Tower%20uses%20async%20traits%2C%20this%20is%20simplified%0A%0Atrait%20SimpleService%20%7B%0A%20%20%20%20fn%20call(%26self%2C%20request%3A%20String)%20-%3E%20String%3B%0A%7D%0A%0A%2F%2F%20Basic%20service%0Astruct%20HelloService%3B%0A%0Aimpl%20SimpleService%20for%20HelloService%20%7B%0A%20%20%20%20fn%20call(%26self%2C%20request%3A%20String)%20-%3E%20String%20%7B%0A%20%20%20%20%20%20%20%20format!(%22Hello%2C%20%7B%7D!%22%2C%20request)%0A%20%20%20%20%7D%0A%7D%0A%0A%2F%2F%20Logging%20%22middleware%22%0Astruct%20LoggingService%3CS%3E%20%7B%0A%20%20%20%20inner%3A%20S%2C%0A%7D%0A%0Aimpl%3CS%3A%20SimpleService%3E%20SimpleService%20for%20LoggingService%3CS%3E%20%7B%0A%20%20%20%20fn%20call(%26self%2C%20request%3A%20String)%20-%3E%20String%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Request%3A%20%7B%7D%22%2C%20request)%3B%0A%20%20%20%20%20%20%20%20let%20response%20%3D%20self.inner.call(request)%3B%0A%20%20%20%20%20%20%20%20println!(%22Response%3A%20%7B%7D%22%2C%20response)%3B%0A%20%20%20%20%20%20%20%20response%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20service%20%3D%20LoggingService%20%7B%0A%20%20%20%20%20%20%20%20inner%3A%20HelloService%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20service.call(%22World%22.to_string())%3B%0A%7D)

## Real Examples from MetaMCP

### CORS Configuration

From `src/api/mod.rs`:

```rust
use tower_http::cors::{Any, CorsLayer};

pub fn create_router(state: AppState) -> Router {
    // Configure CORS middleware
    let cors = CorsLayer::new()
        // Allow requests from any origin
        .allow_origin(Any)
        // Allow any HTTP method
        .allow_methods(Any)
        // Allow any headers
        .allow_headers(Any);

    Router::new()
        .merge(routes::public_routes())
        .merge(routes::protected_routes(state.clone()))
        .with_state(state)
        // Apply CORS layer
        .layer(cors)
}
```

### Request Tracing

From `src/api/mod.rs`:

```rust
use tower_http::trace::TraceLayer;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(routes::public_routes())
        .merge(routes::protected_routes(state.clone()))
        .with_state(state)
        // Add tracing for all requests
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
```

The `TraceLayer` automatically logs:
- Request method and URI
- Response status code
- Request duration
- Headers (configurable)

### Custom Authentication Middleware

From `src/auth/middleware.rs`:

```rust
use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware<B>(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // Extract the Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            AppError::Unauthorized("Missing Authorization header".to_string())
        })?;

    // Parse the Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            AppError::Unauthorized("Invalid Authorization format".to_string())
        })?;

    // Validate the token and get claims
    let claims = auth_service.validate_token(token).await?;

    // Insert claims into request extensions for handlers
    request.extensions_mut().insert(claims);

    // Call the next layer/handler
    Ok(next.run(request).await)
}
```

### Applying Middleware to Routes

From `src/api/routes/mod.rs`:

```rust
use axum::middleware;

pub fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/mcp/servers",
            get(handlers::list_mcp_servers)
                .post(handlers::create_mcp_server))
        .route("/api/v1/mcp/servers/{server_id}",
            get(handlers::get_mcp_server)
                .put(handlers::update_mcp_server)
                .delete(handlers::delete_mcp_server))
        // Apply auth middleware to all routes in this router
        .layer(middleware::from_fn_with_state(
            state.auth.clone(),
            auth_middleware
        ))
}
```

## Common Tower-HTTP Middleware

### Timeout

```rust
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

let app = Router::new()
    .route("/", get(handler))
    .layer(TimeoutLayer::new(Duration::from_secs(30)));
```

### Compression

```rust
use tower_http::compression::CompressionLayer;

let app = Router::new()
    .route("/", get(handler))
    .layer(CompressionLayer::new());
```

### Request Body Limit

```rust
use tower_http::limit::RequestBodyLimitLayer;

let app = Router::new()
    .route("/upload", post(upload_handler))
    // Limit request body to 10MB
    .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024));
```

### Sensitive Headers

```rust
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use http::header::{AUTHORIZATION, COOKIE};

let app = Router::new()
    .route("/", get(handler))
    // Don't log these headers
    .layer(SetSensitiveRequestHeadersLayer::new([
        AUTHORIZATION,
        COOKIE,
    ]));
```

## Layer Ordering

Layers are applied in reverse order - the last one added runs first:

```rust
Router::new()
    .route("/", get(handler))
    .layer(layer_a)  // Runs 3rd
    .layer(layer_b)  // Runs 2nd
    .layer(layer_c)  // Runs 1st (outermost)
```

Request flow: `layer_c → layer_b → layer_a → handler`
Response flow: `handler → layer_a → layer_b → layer_c`

[Run layer ordering example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=%2F%2F%20Demonstrates%20middleware%20ordering%20concept%0A%0Afn%20layer_a(next%3A%20impl%20Fn(%26str)%20-%3E%20String)%20-%3E%20impl%20Fn(%26str)%20-%3E%20String%20%7B%0A%20%20%20%20move%20%7Creq%7C%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Layer%20A%3A%20before%22)%3B%0A%20%20%20%20%20%20%20%20let%20res%20%3D%20next(req)%3B%0A%20%20%20%20%20%20%20%20println!(%22Layer%20A%3A%20after%22)%3B%0A%20%20%20%20%20%20%20%20res%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20layer_b(next%3A%20impl%20Fn(%26str)%20-%3E%20String)%20-%3E%20impl%20Fn(%26str)%20-%3E%20String%20%7B%0A%20%20%20%20move%20%7Creq%7C%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Layer%20B%3A%20before%22)%3B%0A%20%20%20%20%20%20%20%20let%20res%20%3D%20next(req)%3B%0A%20%20%20%20%20%20%20%20println!(%22Layer%20B%3A%20after%22)%3B%0A%20%20%20%20%20%20%20%20res%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20handler(req%3A%20%26str)%20-%3E%20String%20%7B%0A%20%20%20%20println!(%22Handler%3A%20processing%20%7B%7D%22%2C%20req)%3B%0A%20%20%20%20%22Response%22.to_string()%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Build%20the%20%22stack%22%3A%20B%20wraps%20A%20wraps%20handler%0A%20%20%20%20let%20with_a%20%3D%20layer_a(handler)%3B%0A%20%20%20%20let%20with_b%20%3D%20layer_b(with_a)%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22%3D%3D%3D%20Request%20flow%20%3D%3D%3D%22)%3B%0A%20%20%20%20let%20result%20%3D%20with_b(%22GET%20%2F%22)%3B%0A%20%20%20%20println!(%22Final%20result%3A%20%7B%7D%22%2C%20result)%3B%0A%7D)

## Creating Custom Middleware

### Function-Based (Recommended for Axum)

```rust
use axum::{http::Request, middleware::Next, response::Response};

async fn timing_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    tracing::info!("Request took {:?}", duration);

    response
}

// Apply it
let app = Router::new()
    .route("/", get(handler))
    .layer(middleware::from_fn(timing_middleware));
```

### Tower Service (For Complex Cases)

```rust
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;

// The layer that produces the service
#[derive(Clone)]
pub struct LogLayer;

impl<S> Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(&self, service: S) -> Self::Service {
        LogService { inner: service }
    }
}

// The actual middleware service
#[derive(Clone)]
pub struct LogService<S> {
    inner: S,
}

impl<S, Request> Service<Request> for LogService<S>
where
    S: Service<Request>,
    Request: std::fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        tracing::info!("Request: {:?}", request);
        self.inner.call(request)
    }
}
```

## Best Practices

### DO

1. **Use tower-http for common needs** - Don't reinvent the wheel
2. **Order layers carefully** - Outer layers run first
3. **Use `from_fn` for simple middleware** - Less boilerplate
4. **Add tracing early** - Essential for debugging
5. **Configure CORS properly** - Security matters

### DON'T

1. **Don't block in middleware** - Use async operations
2. **Don't ignore backpressure** - Implement `poll_ready` properly
3. **Don't leak sensitive data** - Mark headers as sensitive
4. **Don't forget error handling** - Middleware can fail too

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Composable** | Mix and match middleware freely |
| **Reusable** | Same middleware works across frameworks |
| **Type-safe** | Compile-time checking |
| **Async-native** | Built for async Rust |
| **Well-tested** | Production-proven |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Complex types** | Can get verbose |
| **Learning curve** | Service/Layer patterns take time |
| **Debugging** | Stack traces can be deep |

## When to Use

**Use Tower/Tower-HTTP when:**
- Building web services with Axum, Hyper, or Tonic
- You need common HTTP middleware (CORS, compression, tracing)
- You want to share middleware across services
- You need fine-grained control over request/response processing

**Consider alternatives when:**
- Simple applications without complex middleware needs
- You're using a framework with built-in middleware (Actix Web)

## Further Learning

### Official Resources
- [Tower Documentation](https://docs.rs/tower)
- [Tower-HTTP Documentation](https://docs.rs/tower-http)
- [Tower GitHub](https://github.com/tower-rs/tower)

### Articles
- [Inventing the Service Trait](https://tokio.rs/blog/2021-05-14-inventing-the-service-trait)
- [Tower Deep Dive](https://docs.rs/tower/latest/tower/)

### Practice
1. Implement a rate limiting middleware
2. Build a request ID middleware
3. Create a caching layer
4. Implement circuit breaker pattern

## Related Crates

- **axum** - Web framework built on Tower
- **hyper** - HTTP library using Tower
- **tonic** - gRPC framework using Tower
- **tower-service** - Core Service trait
