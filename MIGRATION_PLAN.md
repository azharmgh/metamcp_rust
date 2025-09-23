# MetaMCP Backend JavaScript to Rust Migration Plan

## 1. Architecture Overview

### Migration Goal
**Build a headless, API-only backend for MetaMCP in pure Rust** - focusing on performance, type safety, and scalability without any frontend/UI components.

### Current JavaScript Stack Analysis
- **Framework**: Express 5.1
- **Database**: PostgreSQL with Drizzle ORM
- **Authentication**: better-auth
- **RPC**: tRPC
- **Protocol**: Model Context Protocol (MCP) SDK
- **Process Management**: spawn-rx, cross-spawn
- **Streaming**: SSE (Server-Sent Events)

### Target Rust Architecture (Headless API)
- **Framework**: Axum (recommended) - async, performant, type-safe
- **Database**: SQLx - compile-time checked queries ✅ DECIDED
- **Authentication**: JWT-based stateless auth with Rust crates ✅ DECIDED
- **API**: RESTful with OpenAPI documentation ✅ DECIDED
- **Process Management**: tokio::process for MCP servers ✅ DECIDED
- **Streaming**: Custom Tokio streams with Axum SSE/WebSocket ✅ DECIDED
- **Protocol**: Native Rust MCP implementation

## 2. Web Framework Selection

### Option A: Axum (Recommended)
**Pros:**
- Tower ecosystem integration
- Excellent async performance
- Type-safe routing
- Built-in extractors and middleware
- Strong community support

**Cons:**
- Steeper learning curve
- More boilerplate initially

### Option B: Actix-web
**Pros:**
- Mature, battle-tested
- Actor model for concurrency
- Extensive middleware ecosystem
- Excellent performance

**Cons:**
- Complex actor system might be overkill
- Larger binary size

### Option C: Rocket
**Pros:**
- Developer-friendly with macros
- Type-safe by default
- Built-in JSON handling

**Cons:**
- Slower compile times
- Less flexible than Axum

## 3. Database Layer Design Patterns

### Recommended: SQLx (Compile-time checked queries)

**SQLx is the recommended choice for MetaMCP** due to its unique combination of safety, performance, and migration-friendliness.

#### Why SQLx is the Best Choice:

1. **Compile-time SQL Validation**
   - Catches SQL errors at compile time, not runtime
   - Validates against your actual database schema
   - Critical for ensuring correctness during migration

2. **Pure Rust & Async-First**
   - Built for async from the ground up
   - No blocking I/O in hot paths
   - Perfect match with Axum/Tokio ecosystem

3. **Migration-Friendly**
   - Can reuse existing SQL queries with minimal changes
   - Built-in migration system similar to Drizzle
   - Direct SQL means easier porting from JavaScript

4. **Performance**
   - Zero-cost abstractions
   - Prepared statement caching
   - Connection pooling built-in
   - No ORM overhead

5. **Developer Experience**
   ```rust
   // Type-safe query with compile-time validation
   let user = sqlx::query_as!(
       User,
       r#"
       SELECT id, email, created_at
       FROM users
       WHERE email = $1
       "#,
       email
   )
   .fetch_one(&pool)
   .await?;
   ```

#### SQLx Implementation Strategy:

```rust
use sqlx::{PgPool, PgPoolOptions};
use std::time::Duration;

pub struct Database {
    pool: PgPool,
}

impl Database {
    // Connection pool setup
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(100)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    // Repository pattern for clean architecture
    pub fn users(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    pub fn sessions(&self) -> SessionRepository {
        SessionRepository::new(self.pool.clone())
    }
}

// Example repository
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create(&self, user: CreateUser) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, password_hash, created_at)
            VALUES ($1, $2, NOW())
            RETURNING *
            "#,
            user.email,
            user.password_hash
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}
```

#### Migration Path from Drizzle:

1. **Direct SQL Translation**
   - Drizzle's SQL-like syntax maps directly to SQLx raw queries
   - Team's existing SQL knowledge transfers immediately

2. **Type Safety Maintained**
   - Like Drizzle's type inference, SQLx provides compile-time guarantees
   - Struct mapping is automatic with `query_as!` macro

3. **Transaction Support**
   ```rust
   let mut tx = pool.begin().await?;

   // Multiple operations with transaction
   let user = create_user(&mut tx, data).await?;
   let session = create_session(&mut tx, user.id).await?;

   tx.commit().await?;
   ```

4. **Migration System**
   ```rust
   // Similar to Drizzle migrations
   sqlx::migrate!("./migrations")
       .run(&pool)
       .await?;
   ```

### Alternative Options (Not Recommended)

#### Option A: Diesel
- **Cons for MetaMCP**: Not async-native, requires learning DSL, more complex migration from JavaScript
- Use only if team has existing Diesel expertise

#### Option B: SeaORM
- **Cons for MetaMCP**: Additional abstraction overhead, heavier runtime cost
- Use only if you need ActiveRecord-style patterns

## 4. Authentication Strategy

### Recommended: Headless API Authentication with Rust Crates

Since the goal is to provide a **headless API backend** for MetaMCP, we'll use battle-tested Rust crates for a stateless, token-based authentication system.

#### Authentication Architecture:

1. **JWT-based Authentication** (Stateless)
   - No server-side sessions needed for headless API
   - Tokens can be validated without database lookups
   - Perfect for distributed/scaled deployments

2. **OAuth2/OIDC Support**
   - Support external identity providers
   - Maintain compatibility with existing OAuth flows

3. **API Key Authentication**
   - For service-to-service communication
   - Long-lived tokens for CI/CD and automation

#### Implementation with Rust Crates:

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use oauth2::{
    AuthorizationCode,
    AuthUrl,
    ClientId,
    ClientSecret,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenUrl,
};
use axum_extra::headers::authorization::Bearer;
use tower_http::auth::RequireAuthorizationLayer;

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // user id
    pub email: String,
    pub exp: usize,            // expiry
    pub iat: usize,            // issued at
    pub roles: Vec<String>,
    pub session_id: String,
}

// Authentication service
pub struct AuthService {
    jwt_secret: String,
    jwt_refresh_secret: String,
    argon2: Argon2<'static>,
    oauth_client: BasicClient,
}

impl AuthService {
    // Password hashing with Argon2
    pub async fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self.argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(hash)
    }

    // Verify password
    pub async fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(self.argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    // Generate JWT tokens
    pub async fn generate_tokens(&self, user_id: &str, email: &str) -> Result<TokenPair> {
        let now = Utc::now();

        // Access token (15 minutes)
        let access_claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: (now + Duration::minutes(15)).timestamp() as usize,
            iat: now.timestamp() as usize,
            roles: vec!["user".to_string()],
            session_id: Uuid::new_v4().to_string(),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes())
        )?;

        // Refresh token (7 days)
        let refresh_claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: (now + Duration::days(7)).timestamp() as usize,
            iat: now.timestamp() as usize,
            roles: vec!["refresh".to_string()],
            session_id: access_claims.session_id.clone(),
        };

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_refresh_secret.as_bytes())
        )?;

        Ok(TokenPair { access_token, refresh_token })
    }

    // Validate JWT token
    pub async fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default()
        )?;
        Ok(token_data.claims)
    }
}

// Axum middleware for authentication
pub async fn auth_middleware<B>(
    State(auth): State<Arc<AuthService>>,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let token = auth_header.token();

    match auth.validate_token(token).await {
        Ok(claims) => {
            // Attach claims to request extensions
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

// API Key authentication for service-to-service
pub struct ApiKeyAuth {
    valid_keys: HashMap<String, ApiKeyMetadata>,
}

impl ApiKeyAuth {
    pub async fn validate_api_key(&self, key: &str) -> Result<ApiKeyMetadata> {
        self.valid_keys
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("Invalid API key"))
    }
}
```

#### OAuth2 Implementation:

```rust
use oauth2::{
    basic::BasicClient,
    AuthorizationCode,
    TokenResponse,
};

pub struct OAuthService {
    clients: HashMap<String, BasicClient>,
}

impl OAuthService {
    pub fn new() -> Self {
        let mut clients = HashMap::new();

        // Configure OAuth providers
        let github_client = BasicClient::new(
            ClientId::new(env::var("GITHUB_CLIENT_ID")?),
            Some(ClientSecret::new(env::var("GITHUB_CLIENT_SECRET")?)),
            AuthUrl::new("https://github.com/login/oauth/authorize")?,
            Some(TokenUrl::new("https://github.com/login/oauth/access_token")?)
        )
        .set_redirect_uri(RedirectUrl::new("http://localhost:12009/oauth/callback")?);

        clients.insert("github".to_string(), github_client);

        Self { clients }
    }

    pub async fn handle_callback(&self, provider: &str, code: AuthorizationCode) -> Result<User> {
        let client = self.clients.get(provider)
            .ok_or_else(|| anyhow!("Unknown provider"))?;

        let token = client
            .exchange_code(code)
            .request_async(async_http_client)
            .await?;

        // Exchange token for user info
        let user_info = self.fetch_user_info(provider, token.access_token()).await?;

        // Create or update user in database
        let user = self.upsert_user(user_info).await?;

        Ok(user)
    }
}
```

#### Key Dependencies:

```toml
# Authentication
jsonwebtoken = "9.3"              # JWT token handling
argon2 = "0.5"                    # Password hashing
oauth2 = "4.4"                    # OAuth2 client
axum-extra = "0.9"                # Additional Axum extractors
tower = "0.4"                     # Middleware framework
tower-http = { version = "0.5", features = ["auth"] }

# Session store (optional, for refresh tokens)
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }
```

### Benefits of This Approach:

1. **Stateless & Scalable**: JWT tokens don't require server-side session storage
2. **Industry Standard**: Uses well-established authentication patterns
3. **Secure**: Argon2 for passwords, RS256 for JWT signing (optional)
4. **Flexible**: Supports multiple auth methods (password, OAuth, API keys)
5. **Headless-First**: No cookie/session management complexity
6. **Performance**: Token validation without database lookups

## 5. RPC/API Design Pattern

### Recommended: RESTful with OpenAPI

**RESTful API with OpenAPI documentation is the recommended choice** for MetaMCP's headless backend, providing the best balance of simplicity, tooling, and client compatibility.

#### Why RESTful with OpenAPI:

1. **Universal Client Compatibility**
   - Works with any HTTP client in any language
   - No special client libraries required
   - Easy integration with existing tools

2. **Excellent Documentation**
   - Auto-generated interactive API docs (Swagger UI)
   - OpenAPI spec for client code generation
   - Clear contract between backend and frontend teams

3. **Industry Standard**
   - Well-understood by all developers
   - Extensive tooling ecosystem
   - Easy to test with curl, Postman, etc.

4. **Simplicity**
   - Straightforward to implement and maintain
   - Clear HTTP semantics (GET, POST, PUT, DELETE)
   - Standard status codes and error handling

5. **Caching & CDN Support**
   - HTTP caching headers work out of the box
   - CDN-friendly for read endpoints
   - Better performance for global deployments

#### Implementation with Axum and OpenAPI:

```rust
use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use serde::{Deserialize, Serialize};

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        list_users,
        get_user,
        create_user,
        update_user,
        delete_user,
        list_mcp_servers,
        execute_mcp_tool
    ),
    components(
        schemas(User, CreateUserRequest, UpdateUserRequest, McpServer, McpToolRequest, McpToolResponse, ErrorResponse)
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "users", description = "User management"),
        (name = "mcp", description = "MCP server operations")
    ),
    info(
        title = "MetaMCP API",
        version = "1.0.0",
        description = "Headless API backend for MetaMCP",
        contact(
            name = "MetaMCP Team",
            email = "support@metamcp.io"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:12009", description = "Local development"),
        (url = "https://api.metamcp.io", description = "Production")
    )
)]
struct ApiDoc;

// Request/Response schemas
#[derive(Serialize, Deserialize, ToSchema)]
struct User {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    id: Uuid,
    #[schema(example = "user@example.com")]
    email: String,
    #[schema(example = "2024-01-01T00:00:00Z")]
    created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct CreateUserRequest {
    #[schema(example = "user@example.com")]
    email: String,
    #[schema(example = "securePassword123")]
    password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct ErrorResponse {
    #[schema(example = "Invalid request parameters")]
    error: String,
    #[schema(example = 400)]
    status: u16,
    #[schema(example = "2024-01-01T00:00:00Z")]
    timestamp: DateTime<Utc>,
}

// API Endpoints with OpenAPI documentation
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service unavailable", body = ErrorResponse)
    )
)]
async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    // Health check implementation
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 409, description = "User already exists", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    // User creation logic
    let user = state.db.users().create(payload).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    let user = state.db.users().find_by_id(id).await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(user))
}

// MCP-specific endpoints
#[utoipa::path(
    post,
    path = "/api/v1/mcp/servers/{server_id}/tools/{tool_name}/execute",
    tag = "mcp",
    params(
        ("server_id" = String, Path, description = "MCP Server ID"),
        ("tool_name" = String, Path, description = "Tool name to execute")
    ),
    request_body = McpToolRequest,
    responses(
        (status = 200, description = "Tool executed successfully", body = McpToolResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Server or tool not found", body = ErrorResponse),
        (status = 500, description = "Tool execution failed", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn execute_mcp_tool(
    State(state): State<AppState>,
    Path((server_id, tool_name)): Path<(String, String)>,
    Json(payload): Json<McpToolRequest>,
) -> Result<Json<McpToolResponse>, AppError> {
    // MCP tool execution logic
}

// Router setup
pub fn create_api_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))

        // User management
        .route("/api/v1/users", post(create_user).get(list_users))
        .route("/api/v1/users/:id", get(get_user).put(update_user).delete(delete_user))

        // MCP operations
        .route("/api/v1/mcp/servers", get(list_mcp_servers).post(create_mcp_server))
        .route("/api/v1/mcp/servers/:id", get(get_mcp_server).delete(delete_mcp_server))
        .route("/api/v1/mcp/servers/:server_id/tools", get(list_tools))
        .route("/api/v1/mcp/servers/:server_id/tools/:tool_name/execute", post(execute_mcp_tool))

        // SSE endpoint for real-time updates
        .route("/api/v1/events", get(sse_handler))

        // Serve OpenAPI documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))

        // Apply authentication middleware to protected routes
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

// Versioning strategy
pub fn create_versioned_api() -> Router {
    Router::new()
        .nest("/api/v1", v1::router())
        // Future versions can be added here
        // .nest("/api/v2", v2::router())
}
```

#### Key Dependencies for RESTful/OpenAPI:

```toml
# OpenAPI documentation
utoipa = { version = "4.2", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "6.0", features = ["axum"] }

# API versioning and validation
validator = { version = "0.18", features = ["derive"] }
```

### Alternative Options (Not Recommended for MetaMCP)

#### GraphQL
- **Cons**: Over-complicated for MetaMCP's use case, requires special clients
- Use only if you need flexible query composition

#### gRPC
- **Cons**: Binary protocol less suitable for web clients, requires protobuf
- Use only for internal service-to-service communication

#### Custom RPC (tRPC-like)
- **Cons**: Requires custom client generation, less standard
- Use only if maintaining exact TypeScript type safety is critical

## 6. Process Management Pattern

### Recommended: tokio::process

**tokio::process is the recommended choice** for managing MCP server processes, providing native async support and seamless integration with the Tokio runtime.

#### Why tokio::process:

1. **Native Tokio Integration**
   - Built into the Tokio ecosystem
   - No additional dependencies
   - Consistent with the rest of the async runtime

2. **Async/Await Support**
   - Non-blocking process spawning
   - Efficient resource utilization
   - Natural integration with async code

3. **Stream-based I/O**
   - Async stdout/stderr reading
   - Perfect for streaming MCP protocol messages
   - Backpressure handling

4. **Cross-platform**
   - Works on Linux, macOS, and Windows
   - Consistent behavior across platforms
   - Production-tested

#### Implementation for MCP Server Management:

```rust
use tokio::process::{Command, Child};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;

pub struct McpServerManager {
    servers: Arc<RwLock<HashMap<String, McpServerHandle>>>,
}

pub struct McpServerHandle {
    id: String,
    child: Child,
    config: McpServerConfig,
    status: ServerStatus,
}

#[derive(Clone, Debug)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ServerStatus {
    Starting,
    Running,
    Stopped,
    Failed(String),
}

impl McpServerManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn spawn_server(&self, config: McpServerConfig) -> Result<String> {
        let server_id = uuid::Uuid::new_v4().to_string();

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

        // Kill process group on drop (important for cleanup)
        cmd.kill_on_drop(true);

        // Spawn the process
        let mut child = cmd.spawn()?;

        // Set up stdout reader for MCP messages
        if let Some(stdout) = child.stdout.take() {
            let server_id_clone = server_id.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    // Process MCP protocol messages
                    if let Ok(message) = serde_json::from_str::<McpMessage>(&line) {
                        handle_mcp_message(&server_id_clone, message).await;
                    }
                }
            });
        }

        // Set up stderr reader for logging
        if let Some(stderr) = child.stderr.take() {
            let server_id_clone = server_id.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!(server_id = %server_id_clone, "Server stderr: {}", line);
                }
            });
        }

        // Store the server handle
        let handle = McpServerHandle {
            id: server_id.clone(),
            child,
            config,
            status: ServerStatus::Running,
        };

        self.servers.write().await.insert(server_id.clone(), handle);

        Ok(server_id)
    }

    pub async fn stop_server(&self, server_id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;

        if let Some(mut handle) = servers.remove(server_id) {
            // Try graceful shutdown first
            if let Some(stdin) = handle.child.stdin.take() {
                // Send shutdown message via MCP protocol
                let shutdown_msg = serde_json::to_string(&McpMessage::Shutdown)?;
                let _ = tokio::io::AsyncWriteExt::write_all(&mut stdin, shutdown_msg.as_bytes()).await;
            }

            // Wait for graceful shutdown (with timeout)
            let timeout = tokio::time::Duration::from_secs(5);
            let shutdown_result = tokio::time::timeout(timeout, handle.child.wait()).await;

            match shutdown_result {
                Ok(Ok(status)) => {
                    tracing::info!("Server {} stopped with status: {:?}", server_id, status);
                }
                _ => {
                    // Force kill if graceful shutdown failed
                    handle.child.kill().await?;
                    tracing::warn!("Server {} force killed", server_id);
                }
            }
        }

        Ok(())
    }

    pub async fn restart_server(&self, server_id: &str) -> Result<String> {
        // Get the config before stopping
        let config = {
            let servers = self.servers.read().await;
            servers.get(server_id)
                .ok_or_else(|| anyhow!("Server not found"))?
                .config
                .clone()
        };

        // Stop the server
        self.stop_server(server_id).await?;

        // Spawn new instance with same config
        self.spawn_server(config).await
    }

    pub async fn list_servers(&self) -> Vec<ServerInfo> {
        let servers = self.servers.read().await;
        servers.values()
            .map(|handle| ServerInfo {
                id: handle.id.clone(),
                name: handle.config.name.clone(),
                status: handle.status.clone(),
            })
            .collect()
    }

    pub async fn send_message(&self, server_id: &str, message: McpMessage) -> Result<()> {
        let mut servers = self.servers.write().await;

        let handle = servers.get_mut(server_id)
            .ok_or_else(|| anyhow!("Server not found"))?;

        if let Some(stdin) = &mut handle.child.stdin {
            let msg = serde_json::to_string(&message)?;
            tokio::io::AsyncWriteExt::write_all(stdin, msg.as_bytes()).await?;
            tokio::io::AsyncWriteExt::write_all(stdin, b"\n").await?;
            tokio::io::AsyncWriteExt::flush(stdin).await?;
        }

        Ok(())
    }

    // Monitor server health
    pub async fn monitor_servers(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        loop {
            interval.tick().await;

            let mut servers = self.servers.write().await;
            for (id, handle) in servers.iter_mut() {
                if let Ok(Some(status)) = handle.child.try_wait() {
                    // Process has exited
                    handle.status = ServerStatus::Failed(
                        format!("Process exited with status: {:?}", status)
                    );
                    tracing::error!("Server {} crashed", id);

                    // Could implement auto-restart logic here
                }
            }
        }
    }
}

// Graceful shutdown handler
impl Drop for McpServerManager {
    fn drop(&mut self) {
        // Schedule cleanup of all servers
        let servers = self.servers.clone();
        tokio::spawn(async move {
            let mut servers = servers.write().await;
            for (_, mut handle) in servers.drain() {
                let _ = handle.child.kill().await;
            }
        });
    }
}
```

#### Advanced Features:

```rust
// Process groups for better cleanup
pub async fn spawn_with_process_group(config: McpServerConfig) -> Result<Child> {
    let mut cmd = Command::new(&config.command);

    // Create new process group (Unix-like systems)
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    cmd.spawn()
}

// Resource limits
#[cfg(unix)]
pub fn set_resource_limits(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;

    cmd.pre_exec(|| {
        // Set memory limit (1GB)
        let limit = libc::rlimit {
            rlim_cur: 1024 * 1024 * 1024,
            rlim_max: 1024 * 1024 * 1024,
        };
        unsafe {
            libc::setrlimit(libc::RLIMIT_AS, &limit);
        }
        Ok(())
    });
}
```

### Alternative (Not Recommended)

#### async-process
- **Cons**: Additional dependency, less integrated with Tokio
- Use only if you need specific features not available in tokio::process

## 7. Streaming/SSE Implementation

### Recommended: Custom Implementation with Tokio Streams + Axum SSE

**A hybrid approach combining Axum's SSE support with custom Tokio streams** is recommended for MetaMCP, providing maximum flexibility for MCP protocol streaming while maintaining clean abstractions.

#### Why This Approach:

1. **Fine-grained Control**
   - Custom stream management for MCP events
   - Backpressure handling
   - Per-client stream customization

2. **Efficient Resource Management**
   - Automatic cleanup on disconnect
   - Memory-bounded channels
   - Connection pooling

3. **Protocol Flexibility**
   - Support SSE for web clients
   - WebSockets for bidirectional communication
   - Raw TCP streams for MCP servers

4. **Scalability**
   - Broadcast to multiple clients efficiently
   - Selective event filtering
   - Rate limiting per client

#### Implementation Architecture:

```rust
use axum::{
    response::sse::{Event, KeepAlive, Sse},
    extract::{State, Path, Query},
    http::StatusCode,
};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt, Stream};
use futures::stream::{self, StreamExt as _};
use std::{sync::Arc, collections::HashMap, time::Duration};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

// Event types for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    McpServerStarted { server_id: String, name: String },
    McpServerStopped { server_id: String, reason: String },
    McpToolExecuted { server_id: String, tool: String, status: String },
    McpMessage { server_id: String, message: serde_json::Value },
    SystemHealth { cpu: f32, memory: f32, active_servers: usize },
    Error { code: String, message: String },
}

// Stream manager for handling multiple clients
pub struct StreamManager {
    // Broadcast channel for system-wide events
    broadcast_tx: broadcast::Sender<StreamEvent>,

    // Per-client channels for targeted events
    client_channels: Arc<RwLock<HashMap<String, mpsc::Sender<StreamEvent>>>>,

    // Per-server channels for MCP server events
    server_channels: Arc<RwLock<HashMap<String, broadcast::Sender<StreamEvent>>>>,
}

impl StreamManager {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);

        Self {
            broadcast_tx,
            client_channels: Arc::new(RwLock::new(HashMap::new())),
            server_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Broadcast event to all connected clients
    pub async fn broadcast(&self, event: StreamEvent) {
        let _ = self.broadcast_tx.send(event);
    }

    // Send event to specific client
    pub async fn send_to_client(&self, client_id: &str, event: StreamEvent) {
        let channels = self.client_channels.read().await;
        if let Some(tx) = channels.get(client_id) {
            let _ = tx.send(event).await;
        }
    }

    // Create a stream for SSE endpoint
    pub async fn create_sse_stream(
        &self,
        client_id: String,
        filters: EventFilters,
    ) -> impl Stream<Item = Result<Event, axum::Error>> {
        // Create client-specific channel
        let (client_tx, mut client_rx) = mpsc::channel(256);
        self.client_channels.write().await.insert(client_id.clone(), client_tx);

        // Subscribe to broadcast channel
        let broadcast_rx = self.broadcast_tx.subscribe();
        let mut broadcast_stream = BroadcastStream::new(broadcast_rx);

        // Subscribe to specific server channels if requested
        let server_streams = self.create_server_streams(&filters.server_ids).await;

        // Combine all streams
        let client_id_clone = client_id.clone();
        let manager = Arc::new(self.clone());

        stream::unfold(
            (client_rx, broadcast_stream, server_streams, client_id_clone, manager, filters),
            |(mut client_rx, mut broadcast_stream, mut server_streams, client_id, manager, filters)| async move {
                loop {
                    tokio::select! {
                        // Client-specific events (highest priority)
                        Some(event) = client_rx.recv() => {
                            if filters.should_send(&event) {
                                let sse_event = Event::default()
                                    .json_data(event)
                                    .id(Uuid::new_v4().to_string());
                                return Some((Ok(sse_event), (client_rx, broadcast_stream, server_streams, client_id, manager, filters)));
                            }
                        }

                        // Broadcast events
                        Some(Ok(event)) = broadcast_stream.next() => {
                            if filters.should_send(&event) {
                                let sse_event = Event::default()
                                    .json_data(event)
                                    .id(Uuid::new_v4().to_string());
                                return Some((Ok(sse_event), (client_rx, broadcast_stream, server_streams, client_id, manager, filters)));
                            }
                        }

                        // Server-specific events
                        Some(event) = Self::poll_server_streams(&mut server_streams) => {
                            if filters.should_send(&event) {
                                let sse_event = Event::default()
                                    .json_data(event)
                                    .id(Uuid::new_v4().to_string());
                                return Some((Ok(sse_event), (client_rx, broadcast_stream, server_streams, client_id, manager, filters)));
                            }
                        }

                        // Cleanup on all streams closed
                        else => {
                            // Remove client channel on disconnect
                            manager.client_channels.write().await.remove(&client_id);
                            return None;
                        }
                    }
                }
            }
        )
    }

    // Helper to create server-specific streams
    async fn create_server_streams(
        &self,
        server_ids: &[String],
    ) -> Vec<BroadcastStream<StreamEvent>> {
        let channels = self.server_channels.read().await;
        server_ids
            .iter()
            .filter_map(|id| {
                channels.get(id).map(|tx| BroadcastStream::new(tx.subscribe()))
            })
            .collect()
    }

    // Helper to poll multiple server streams
    async fn poll_server_streams(
        streams: &mut Vec<BroadcastStream<StreamEvent>>,
    ) -> Option<StreamEvent> {
        for stream in streams.iter_mut() {
            if let Some(Ok(event)) = stream.next().await {
                return Some(event);
            }
        }
        None
    }
}

// Event filtering for clients
#[derive(Debug, Clone, Deserialize)]
pub struct EventFilters {
    pub event_types: Option<Vec<String>>,
    pub server_ids: Vec<String>,
    pub include_system: bool,
}

impl EventFilters {
    fn should_send(&self, event: &StreamEvent) -> bool {
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

        true
    }
}

// SSE endpoint handler
pub async fn sse_handler(
    State(manager): State<Arc<StreamManager>>,
    Query(filters): Query<EventFilters>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let client_id = Uuid::new_v4().to_string();
    let stream = manager.create_sse_stream(client_id, filters).await;

    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(30)))
}

// WebSocket support for bidirectional streaming
pub mod websocket {
    use axum::{
        extract::{WebSocketUpgrade, ws::{WebSocket, Message}},
        response::Response,
    };
    use super::*;

    pub async fn ws_handler(
        ws: WebSocketUpgrade,
        State(manager): State<Arc<StreamManager>>,
    ) -> Response {
        ws.on_upgrade(|socket| handle_socket(socket, manager))
    }

    async fn handle_socket(mut socket: WebSocket, manager: Arc<StreamManager>) {
        let client_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::channel(256);

        // Register client
        manager.client_channels.write().await.insert(client_id.clone(), tx);

        // Subscribe to broadcasts
        let mut broadcast_rx = manager.broadcast_tx.subscribe();

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                Some(msg) = socket.recv() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            // Parse and handle client commands
                            if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                                handle_client_command(&manager, &client_id, cmd).await;
                            }
                        }
                        Ok(Message::Close(_)) | Err(_) => break,
                        _ => {}
                    }
                }

                // Send events to client
                Ok(event) = broadcast_rx.recv() => {
                    let json = serde_json::to_string(&event).unwrap();
                    if socket.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }

                Some(event) = rx.recv() => {
                    let json = serde_json::to_string(&event).unwrap();
                    if socket.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }

        // Cleanup
        manager.client_channels.write().await.remove(&client_id);
    }

    #[derive(Deserialize)]
    #[serde(tag = "type")]
    enum ClientCommand {
        Subscribe { server_ids: Vec<String> },
        Unsubscribe { server_ids: Vec<String> },
        ExecuteTool { server_id: String, tool: String, args: serde_json::Value },
    }

    async fn handle_client_command(
        manager: &StreamManager,
        client_id: &str,
        cmd: ClientCommand,
    ) {
        match cmd {
            ClientCommand::Subscribe { server_ids } => {
                // Add client to server-specific subscriptions
                for server_id in server_ids {
                    // Implementation
                }
            }
            ClientCommand::Unsubscribe { .. } => {
                // Remove subscriptions
            }
            ClientCommand::ExecuteTool { .. } => {
                // Execute MCP tool and stream results
            }
        }
    }
}

// Integration with MCP server events
impl StreamManager {
    pub async fn handle_mcp_event(&self, server_id: String, event: StreamEvent) {
        // Send to server-specific channel
        let channels = self.server_channels.read().await;
        if let Some(tx) = channels.get(&server_id) {
            let _ = tx.send(event.clone());
        }

        // Also broadcast certain events
        match &event {
            StreamEvent::McpServerStarted { .. } |
            StreamEvent::McpServerStopped { .. } => {
                self.broadcast(event).await;
            }
            _ => {}
        }
    }

    // Create rate-limited stream for high-frequency events
    pub fn create_throttled_stream(
        &self,
        duration: Duration,
    ) -> impl Stream<Item = StreamEvent> {
        let rx = self.broadcast_tx.subscribe();
        BroadcastStream::new(rx)
            .filter_map(|r| async { r.ok() })
            .throttle(duration)
    }
}

// Axum route configuration
pub fn streaming_routes() -> Router {
    Router::new()
        .route("/api/v1/events", get(sse_handler))
        .route("/api/v1/ws", get(websocket::ws_handler))
        .route("/api/v1/events/health", get(health_stream))
}

// Specialized health monitoring stream
async fn health_stream(
    State(manager): State<Arc<StreamManager>>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let stream = stream::repeat_with(|| {
        Event::default()
            .json_data(StreamEvent::SystemHealth {
                cpu: get_cpu_usage(),
                memory: get_memory_usage(),
                active_servers: get_active_server_count(),
            })
    })
    .map(Ok)
    .throttle(Duration::from_secs(5));

    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(30)))
}
```

#### Key Dependencies:

```toml
# Streaming dependencies
tokio-stream = "0.1"
futures = "0.3"
axum = { version = "0.7", features = ["ws"] }

# Optional: for more advanced streaming
async-stream = "0.3"  # For creating streams with async/await syntax
pin-project = "1.1"   # For custom stream implementations
```

### Benefits of This Approach:

1. **Flexibility**: Supports SSE, WebSockets, and raw TCP streams
2. **Performance**: Efficient broadcasting with minimal memory overhead
3. **Scalability**: Can handle thousands of concurrent connections
4. **Control**: Fine-grained event filtering and rate limiting
5. **Integration**: Seamless integration with MCP server events
6. **Reliability**: Automatic cleanup and reconnection handling

## 8. Project Structure

```
metamcp_rust/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── handlers.rs
│   │   ├── middleware.rs
│   │   └── jwt.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── models/
│   │   ├── schema.rs
│   │   └── migrations/
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── proxy.rs
│   │   ├── protocol.rs
│   │   └── server_manager.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes/
│   │   ├── handlers/
│   │   └── middleware/
│   ├── streaming/
│   │   ├── mod.rs
│   │   ├── sse.rs
│   │   └── websocket.rs
│   └── utils/
│       ├── mod.rs
│       └── error.rs
├── migrations/
├── tests/
│   ├── integration/
│   └── unit/
└── benches/
```

## 9. Migration Phases

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Set up Rust project with chosen web framework
- [ ] Implement configuration management
- [ ] Set up database connection pool
- [ ] Create error handling framework
- [ ] Implement logging and tracing

### Phase 2: Database Layer (Week 2-3)
- [ ] Define database schema in Rust
- [ ] Migrate existing migrations to chosen ORM
- [ ] Implement repository pattern for data access
- [ ] Add transaction support
- [ ] Create database seeding scripts

### Phase 3: Authentication System (Week 3-4)
- [ ] Port OAuth2 endpoints
- [ ] Implement JWT token generation/validation
- [ ] Create session management
- [ ] Add API key authentication
- [ ] Implement user registration/login

### Phase 4: MCP Protocol Implementation (Week 4-6)
- [ ] Create MCP protocol types
- [ ] Implement MCP proxy router
- [ ] Add server spawning/management
- [ ] Handle bidirectional streaming
- [ ] Implement protocol message parsing

### Phase 5: API Endpoints (Week 6-7)
- [ ] Port tRPC-like functionality or create REST/GraphQL API
- [ ] Implement all existing endpoints
- [ ] Add request validation
- [ ] Create response serialization
- [ ] Add API documentation

### Phase 6: Streaming & Real-time (Week 7-8)
- [ ] Implement SSE endpoints
- [ ] Add WebSocket support (if needed)
- [ ] Create pub/sub system for real-time updates
- [ ] Handle long-polling fallback

### Phase 7: Testing & Optimization (Week 8-9)
- [ ] Write unit tests for all modules
- [ ] Create integration tests
- [ ] Add performance benchmarks
- [ ] Optimize hot paths
- [ ] Load testing

### Phase 8: Deployment & Migration (Week 9-10)
- [ ] Create Docker images
- [ ] Set up CI/CD pipeline
- [ ] Create migration scripts for data
- [ ] Implement blue-green deployment
- [ ] Monitor and rollback strategy

## 10. Key Design Decisions Required

### Questions for Architecture Selection:

1. **Web Framework**: Which option (Axum/Actix/Rocket) aligns best with your team's experience and performance requirements?

2. **Database Strategy** ✅ **DECIDED: SQLx**
   - SQLx chosen for compile-time SQL validation and seamless async support
   - Maintains schema compatibility with existing database
   - Direct SQL queries make migration from Drizzle straightforward

3. **Authentication Strategy** ✅ **DECIDED: Headless JWT-based**
   - Stateless JWT tokens for API authentication
   - Using battle-tested Rust crates (jsonwebtoken, argon2, oauth2)
   - Support for OAuth2, API keys, and password-based auth
   - No session/cookie management for true headless API

4. **API Protocol** ✅ **DECIDED: RESTful with OpenAPI**
   - RESTful API with OpenAPI/Swagger documentation
   - Universal client compatibility (any HTTP client works)
   - Auto-generated API documentation and client SDKs
   - Industry standard with excellent tooling

5. **Process Management** ✅ **DECIDED: tokio::process**
   - Native Tokio integration for async process spawning
   - Stream-based I/O for MCP message handling
   - Built-in support for graceful shutdown and resource limits
   - No additional dependencies needed

6. **Streaming/Real-time** ✅ **DECIDED: Custom Tokio Streams + Axum**
   - Hybrid approach with Tokio streams and Axum's SSE/WebSocket
   - Support for SSE, WebSockets, and raw TCP
   - Efficient broadcasting with event filtering
   - Automatic cleanup and backpressure handling

7. **MCP Protocol**:
   - Should we create FFI bindings to the existing SDK or implement the protocol natively in Rust?
   - What's the strategy for maintaining protocol compatibility?

8. **Concurrency Model**:
   - Tokio-based async/await throughout?
   - Actor model for certain components?
   - Thread pool for CPU-intensive tasks?

7. **Error Handling**:
   - `thiserror` + `anyhow` combination?
   - Custom error types with `From` implementations?

8. **Configuration**:
   - Environment variables with `dotenv`?
   - TOML/YAML configuration files?
   - Runtime reloading support?

## 11. Dependencies Recommendation

```toml
[dependencies]
# Web Framework
axum = { version = "0.7", features = ["ws"] }  # WebSocket support included

# Async Runtime
tokio = { version = "1.38", features = ["full"] }

# Streaming
tokio-stream = "0.1"
futures = "0.3"

# Database - SQLx (Recommended)
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "chrono", "migrate"] }
# Additional SQLx features for development
# sqlx-cli should be installed globally: cargo install sqlx-cli

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Authentication (Headless API)
jsonwebtoken = "9.3"              # JWT token handling
argon2 = "0.5"                    # Password hashing
oauth2 = "4.4"                    # OAuth2 client support
axum-extra = "0.9"                # Additional extractors for Bearer tokens
tower = "0.4"                     # Middleware framework
tower-http = { version = "0.5", features = ["auth", "cors"] }

# Error Handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Configuration
config = "0.14"
dotenv = "0.15"

# UUID and Time
uuid = { version = "1.10", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# HTTP Client
reqwest = { version = "0.12", features = ["json", "stream"] }

# Validation
validator = { version = "0.18", features = ["derive"] }

# OpenAPI/REST API Documentation
utoipa = { version = "4.2", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "6.0", features = ["axum"] }
```

## 12. Performance Considerations

### Memory Management
- Use `Arc<T>` for shared state
- Consider using `Cow<'a, str>` for string handling
- Implement connection pooling for database

### Concurrency
- Use channels for inter-task communication
- Implement backpressure for streaming
- Consider using `RwLock` vs `Mutex` appropriately

### Optimization Targets
- Sub-millisecond response time for cached queries
- Handle 10,000+ concurrent connections
- Minimize memory footprint per connection

## 13. Testing Strategy

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_creation() {
        // Test implementation
    }
}
```

### Integration Testing
- Database tests with test containers
- API endpoint testing with mock servers
- End-to-end protocol testing

### Performance Testing
- Use `criterion` for benchmarking
- Load testing with `drill` or `vegeta`

## 14. Monitoring & Observability

- OpenTelemetry integration
- Prometheus metrics
- Structured logging with `tracing`
- Health check endpoints
- Graceful shutdown handling

## 15. Security Considerations

- Input validation at all entry points
- SQL injection prevention (parameterized queries)
- Rate limiting
- CORS configuration
- Security headers (Helmet equivalent)
- Secrets management

## Next Steps

1. **Review and select architecture options**
2. **Create proof-of-concept for critical components**
3. **Set up development environment**
4. **Begin Phase 1 implementation**

## Questions for Decision Making

1. What's your team's Rust experience level?
2. Are there specific performance SLAs to meet?
3. Do you need to maintain backward compatibility with existing clients?
4. What's the deployment target (Docker, bare metal, cloud)?
5. Are there specific compliance requirements (GDPR, SOC2)?
6. What's the expected scale (users, requests/sec)?
7. Do you need real-time features beyond SSE?
8. Should the migration be incremental or a complete rewrite?