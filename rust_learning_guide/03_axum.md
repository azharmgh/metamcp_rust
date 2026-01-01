# Axum - Web Framework

Axum is a modern, ergonomic web framework built on top of Tokio and Tower. It's designed for building robust, type-safe web APIs in Rust.

## What is Axum?

Axum provides:
- **Routing** - Map URLs to handler functions
- **Extractors** - Type-safe request parsing
- **Middleware** - Request/response processing via Tower
- **State management** - Share application state across handlers

## Why Axum?

Axum was created by the Tokio team and offers:
- Deep Tokio integration
- Type-safe extractors (compile-time validation)
- Minimal boilerplate
- Full Tower middleware compatibility
- Great error messages

## Installation

```toml
[dependencies]
axum = { version = "0.8", features = ["http2"] }
axum-extra = { version = "0.12", features = ["typed-header"] }
tokio = { version = "1", features = ["full"] }
```

## Basic Concepts

### Hello World

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // Build the router
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20axum%3A%3A%7Brouting%3A%3Aget%2C%20Router%7D%3B%0A%0A%2F%2F%20Note%3A%20This%20won%27t%20actually%20run%20in%20playground%20due%20to%20network%20restrictions%0A%2F%2F%20but%20demonstrates%20the%20pattern%0A%0Aasync%20fn%20hello()%20-%3E%20%26%27static%20str%20%7B%0A%20%20%20%20%22Hello%2C%20World!%22%0A%7D%0A%0Afn%20create_app()%20-%3E%20Router%20%7B%0A%20%20%20%20Router%3A%3Anew()%0A%20%20%20%20%20%20%20%20.route(%22%2F%22%2C%20get(hello))%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20_app%20%3D%20create_app()%3B%0A%20%20%20%20println!(%22Router%20created%20successfully!%22)%3B%0A%7D)

### Handler Functions

Handlers are async functions that return something implementing `IntoResponse`:

```rust
use axum::response::Json;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

// Returns JSON
async fn get_user() -> Json<User> {
    Json(User {
        id: 1,
        name: "Alice".to_string(),
    })
}

// Returns plain text
async fn health() -> &'static str {
    "OK"
}

// Returns status code with body
async fn created() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "Resource created")
}
```

## Real Examples from MetaMCP

### Router Setup

From `src/api/mod.rs`:

```rust
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: AppState) -> Router {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the router
    Router::new()
        // Merge public routes (no auth required)
        .merge(routes::public_routes())
        // Merge protected routes (auth required)
        .merge(routes::protected_routes(state.clone()))
        // Add Swagger UI
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Attach shared state
        .with_state(state)
        // Add middleware layers
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
```

### Route Definitions

From `src/api/routes/mod.rs`:

```rust
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

// Public routes - no authentication
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/v1/auth/token", post(handlers::authenticate))
}

// Protected routes - require authentication
pub fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // MCP Server management
        .route("/api/v1/mcp/servers",
            get(handlers::list_mcp_servers)
                .post(handlers::create_mcp_server))
        .route("/api/v1/mcp/servers/{server_id}",
            get(handlers::get_mcp_server)
                .put(handlers::update_mcp_server)
                .delete(handlers::delete_mcp_server))
        // MCP Tool execution
        .route("/api/v1/mcp/servers/{server_id}/tools/{tool_name}",
            post(handlers::execute_mcp_tool))
        // Apply auth middleware to all routes in this router
        .layer(middleware::from_fn_with_state(
            state.auth.clone(),
            auth_middleware
        ))
}
```

### Extractors

Extractors parse request data into typed values.

#### State Extractor

From `src/api/handlers/mcp.rs`:

```rust
use axum::extract::State;

pub async fn list_mcp_servers(
    State(state): State<AppState>,  // Extract shared state
    _user: AuthenticatedUser,        // Extract authenticated user
) -> Result<Json<ListMcpServersResponse>, AppError> {
    let servers = state.db.mcp_servers().list_all(false).await?;
    let server_infos: Vec<McpServerInfo> = servers
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(ListMcpServersResponse { servers: server_infos }))
}
```

#### Path Extractor

```rust
use axum::extract::Path;
use uuid::Uuid;

pub async fn get_mcp_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,  // Extract from URL path
    _user: AuthenticatedUser,
) -> Result<Json<McpServerInfo>, AppError> {
    let server = state.db.mcp_servers()
        .find_by_id(server_id)
        .await?
        .ok_or_else(|| AppError::NotFound("MCP server not found".to_string()))?;

    Ok(Json(server.into()))
}
```

#### JSON Body Extractor

From `src/api/handlers/auth.rs`:

```rust
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub api_key: String,
}

pub async fn authenticate(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,  // Extract JSON body
) -> Result<Json<AuthResponse>, AppError> {
    let token = state.auth
        .authenticate_with_api_key(&payload.api_key)
        .await?;

    Ok(Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 900,
    }))
}
```

[Run extractor example in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%2F%2F%20Request%20body%20type%0A%23%5Bderive(Debug%2C%20Deserialize)%5D%0Astruct%20CreateUser%20%7B%0A%20%20%20%20name%3A%20String%2C%0A%20%20%20%20email%3A%20String%2C%0A%7D%0A%0A%2F%2F%20Response%20body%20type%0A%23%5Bderive(Debug%2C%20Serialize)%5D%0Astruct%20User%20%7B%0A%20%20%20%20id%3A%20u64%2C%0A%20%20%20%20name%3A%20String%2C%0A%20%20%20%20email%3A%20String%2C%0A%7D%0A%0A%2F%2F%20Simulated%20handler%20logic%0Afn%20create_user_logic(req%3A%20CreateUser)%20-%3E%20User%20%7B%0A%20%20%20%20User%20%7B%0A%20%20%20%20%20%20%20%20id%3A%201%2C%0A%20%20%20%20%20%20%20%20name%3A%20req.name%2C%0A%20%20%20%20%20%20%20%20email%3A%20req.email%2C%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Simulate%20JSON%20parsing%0A%20%20%20%20let%20json%20%3D%20r%23%22%7B%22name%22%3A%22Alice%22%2C%22email%22%3A%22alice%40example.com%22%7D%22%23%3B%0A%20%20%20%20let%20req%3A%20CreateUser%20%3D%20serde_json%3A%3Afrom_str(json).unwrap()%3B%0A%20%20%20%20%0A%20%20%20%20let%20user%20%3D%20create_user_logic(req)%3B%0A%20%20%20%20let%20response%20%3D%20serde_json%3A%3Ato_string_pretty(%26user).unwrap()%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Response%3A%5Cn%7B%7D%22%2C%20response)%3B%0A%7D)

### Custom Extractors

From `src/auth/middleware.rs` - custom `AuthenticatedUser` extractor:

```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct AuthenticatedUser {
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Look for claims in request extensions (set by middleware)
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(|claims| AuthenticatedUser { claims })
            .ok_or_else(|| {
                AppError::Unauthorized("Not authenticated".to_string())
            })
    }
}

// Now you can use AuthenticatedUser in any handler
async fn protected_handler(user: AuthenticatedUser) -> String {
    format!("Hello, user {}!", user.claims.sub)
}
```

### Error Handling

From `src/utils/error.rs` - implementing `IntoResponse` for custom errors:

```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match &self {
            AppError::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized", Some(msg.clone()))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "Not Found", Some(msg.clone()))
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "Bad Request", Some(msg.clone()))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error", None)
            }
            AppError::Database(err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error", None)
            }
            // ... more variants
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
            message: self.to_string(),
            details,
        });

        (status, body).into_response()
    }
}
```

[Run error handling example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3ASerialize%3B%0A%0A%23%5Bderive(Debug)%5D%0Aenum%20AppError%20%7B%0A%20%20%20%20NotFound(String)%2C%0A%20%20%20%20BadRequest(String)%2C%0A%20%20%20%20Internal(String)%2C%0A%7D%0A%0A%23%5Bderive(Serialize)%5D%0Astruct%20ErrorResponse%20%7B%0A%20%20%20%20status%3A%20u16%2C%0A%20%20%20%20error%3A%20String%2C%0A%20%20%20%20message%3A%20String%2C%0A%7D%0A%0Aimpl%20AppError%20%7B%0A%20%20%20%20fn%20to_response(%26self)%20-%3E%20ErrorResponse%20%7B%0A%20%20%20%20%20%20%20%20match%20self%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20AppError%3A%3ANotFound(msg)%20%3D%3E%20ErrorResponse%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20status%3A%20404%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20error%3A%20%22Not%20Found%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20message%3A%20msg.clone()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20AppError%3A%3ABadRequest(msg)%20%3D%3E%20ErrorResponse%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20status%3A%20400%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20error%3A%20%22Bad%20Request%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20message%3A%20msg.clone()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20AppError%3A%3AInternal(msg)%20%3D%3E%20ErrorResponse%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20status%3A%20500%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20error%3A%20%22Internal%20Error%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20message%3A%20msg.clone()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20error%20%3D%20AppError%3A%3ANotFound(%22User%20not%20found%22.to_string())%3B%0A%20%20%20%20let%20response%20%3D%20error.to_response()%3B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26response).unwrap())%3B%0A%7D)

### Middleware

From `src/auth/middleware.rs` - authentication middleware:

```rust
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware<B>(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            AppError::Unauthorized("Missing Authorization header".to_string())
        })?;

    // Parse Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            AppError::Unauthorized("Invalid Authorization format".to_string())
        })?;

    // Validate token
    let claims = auth_service.validate_token(token).await?;

    // Add claims to request extensions for handlers
    request.extensions_mut().insert(claims);

    // Continue to handler
    Ok(next.run(request).await)
}
```

### Application State

From `src/api/mod.rs` - defining shared state:

```rust
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub auth: Arc<AuthService>,
}

// State is cloned for each request, Arc ensures efficient sharing
```

## Common Patterns

### Multiple HTTP Methods on Same Route

```rust
Router::new()
    .route("/api/items",
        get(list_items)
            .post(create_item))
    .route("/api/items/{id}",
        get(get_item)
            .put(update_item)
            .delete(delete_item))
```

### Nested Routers

```rust
fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/users", user_routes())
        .nest("/products", product_routes())
        .nest("/orders", order_routes())
}

fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).delete(delete_user))
}
```

### Response Types

```rust
use axum::response::{Html, Redirect};

// Return HTML
async fn index() -> Html<&'static str> {
    Html("<h1>Welcome</h1>")
}

// Return redirect
async fn old_path() -> Redirect {
    Redirect::permanent("/new-path")
}

// Return custom headers
async fn with_headers() -> impl IntoResponse {
    (
        [("X-Custom-Header", "value")],
        "Response body"
    )
}
```

## Best Practices

### DO

1. **Use typed extractors** - Let the compiler catch errors
2. **Implement `IntoResponse` for errors** - Clean error handling
3. **Use `Arc` for expensive state** - Efficient cloning
4. **Layer middleware appropriately** - Order matters (last added runs first)
5. **Return `Result<T, AppError>`** - Propagate errors with `?`

### DON'T

1. **Don't use `unwrap()` in handlers** - Return errors properly
2. **Don't block in handlers** - Use `spawn_blocking` for CPU work
3. **Don't forget CORS** - Add it for browser clients
4. **Don't skip validation** - Use validator or custom checks
5. **Don't expose internal errors** - Map to user-friendly messages

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Type-safe** | Compile-time extraction validation |
| **Ergonomic** | Clean, intuitive API |
| **Fast** | Built on Hyper and Tokio |
| **Tower compatible** | Use any Tower middleware |
| **Good errors** | Helpful compile-time messages |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Learning curve** | Extractors take time to understand |
| **Evolving** | Breaking changes between major versions |
| **Less batteries** | Fewer built-in features than some frameworks |

## When to Use Axum

**Use Axum when:**
- Building REST APIs
- You want type-safe request handling
- You're already using Tokio
- You need Tower middleware compatibility
- You want minimal framework overhead

**Consider alternatives when:**
- You need a full-stack framework (consider Actix Web)
- You're not using async Rust
- You need GraphQL primarily (consider Juniper or async-graphql)

## Further Learning

### Official Resources
- [Axum Documentation](https://docs.rs/axum)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [Tokio Axum Tutorial](https://tokio.rs/blog/2021-07-announcing-axum)

### Books & Articles
- [Zero to Production in Rust](https://www.zero2prod.com/) - Uses Actix but concepts apply
- [Rust Web Development](https://www.manning.com/books/rust-web-development)

### Practice Projects
1. Build a TODO API with CRUD operations
2. Implement JWT authentication
3. Create a file upload service
4. Build a WebSocket chat server

## Related Crates

- **tower** - Middleware framework
- **tower-http** - HTTP-specific middleware
- **axum-extra** - Additional extractors and utilities
- **utoipa** - OpenAPI documentation
