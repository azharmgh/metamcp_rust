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
- **Framework**: Axum - async, performant, type-safe ✅ DECIDED
- **Database**: SQLx - compile-time checked queries ✅ DECIDED
- **Authentication**: API Key + JWT tokens ✅ DECIDED
- **Client Communication**: Streaming HTTP ONLY ✅ DECIDED
- **Backend MCP Servers**: SSE or stdio (protocol translation) ✅ DECIDED
- **Process Management**: tokio::process for MCP servers ✅ DECIDED
- **Protocol**: Native Rust MCP implementation with translation layer ✅ DECIDED

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
    pub fn api_keys(&self) -> ApiKeyRepository {
        ApiKeyRepository::new(self.pool.clone())
    }

    pub fn mcp_servers(&self) -> McpServerRepository {
        McpServerRepository::new(self.pool.clone())
    }
}

// API Key repository for authentication
pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub async fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let api_key = sqlx::query_as!(
            ApiKey,
            "SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true",
            key_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(api_key)
    }

    pub async fn create(&self, name: &str, key_hash: &str) -> Result<ApiKey> {
        let api_key = sqlx::query_as!(
            ApiKey,
            r#"
            INSERT INTO api_keys (name, key_hash, is_active, created_at)
            VALUES ($1, $2, true, NOW())
            RETURNING *
            "#,
            name,
            key_hash
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(api_key)
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
   let api_key = create_api_key(&mut tx, name, key_hash).await?;
   let server_config = create_server_config(&mut tx, api_key.id, config).await?;

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

### API Key + JWT Authentication (Simplified)

**MetaMCP uses a simple API key-based authentication system**. There are NO user accounts, NO OAuth, NO session management - just API keys for client authentication.

#### Authentication Architecture:

1. **API Key Management**
   - API keys are persistent and stored encrypted in database
   - Each API key has a unique identifier and metadata (name, created_at, last_used)
   - Keys can be revoked by marking them as inactive

2. **JWT Token Generation**
   - Clients obtain an API key (out of band - e.g., via CLI tool or admin interface)
   - Clients generate a JWT token based on their API key
   - JWT tokens are short-lived (15 minutes) for security
   - Token contains API key ID and expiry time

3. **No User Persistence**
   - No user tables, no user registration/login flows
   - No email, password, or OAuth providers
   - Pure API key to JWT workflow

#### Implementation:

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum_extra::headers::authorization::Bearer;
use tower_http::auth::RequireAuthorizationLayer;
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};

// API Key model (stored in database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,        // Hashed version for lookup
    pub encrypted_key: Vec<u8>,  // Encrypted actual key
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

// JWT Claims structure (simplified - no user info)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // API key ID
    pub exp: usize,       // expiry
    pub iat: usize,       // issued at
    pub jti: String,      // JWT ID for tracking
}

// Authentication service
pub struct AuthService {
    jwt_secret: String,
    encryption_key: ChaCha20Poly1305,
    db: Database,
}

impl AuthService {
    pub fn new(jwt_secret: String, encryption_key: &[u8; 32], db: Database) -> Self {
        Self {
            jwt_secret,
            encryption_key: ChaCha20Poly1305::new(encryption_key.into()),
            db,
        }
    }

    // Generate new API key
    pub async fn generate_api_key(&self, name: String) -> Result<(String, ApiKey)> {
        // Generate random API key
        let raw_key = format!("mcp_{}", Uuid::new_v4().simple());

        // Hash for database lookup
        let key_hash = self.hash_key(&raw_key)?;

        // Encrypt for storage
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let encrypted_key = self.encryption_key
            .encrypt(&nonce, raw_key.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Store in database
        let api_key = self.db.api_keys().create(&name, &key_hash, encrypted_key).await?;

        Ok((raw_key, api_key))
    }

    // Hash API key for database lookup
    fn hash_key(&self, key: &str) -> Result<String> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2
            .hash_password(key.as_bytes(), &salt)?
            .to_string();
        Ok(hash)
    }

    // Validate API key and generate JWT
    pub async fn authenticate_with_api_key(&self, api_key: &str) -> Result<String> {
        // Hash the provided key
        let key_hash = self.hash_key(api_key)?;

        // Look up in database
        let stored_key = self.db.api_keys()
            .find_by_key_hash(&key_hash)
            .await?
            .ok_or_else(|| anyhow!("Invalid API key"))?;

        if !stored_key.is_active {
            return Err(anyhow!("API key is inactive"));
        }

        // Update last used timestamp
        self.db.api_keys().update_last_used(stored_key.id).await?;

        // Generate JWT token
        self.generate_jwt_for_key(stored_key.id).await
    }

    // Generate JWT token for an API key
    pub async fn generate_jwt_for_key(&self, key_id: Uuid) -> Result<String> {
        let now = Utc::now();

        let claims = Claims {
            sub: key_id.to_string(),
            exp: (now + Duration::minutes(15)).timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes())
        )?;

        Ok(token)
    }

    // Validate JWT token
    pub async fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default()
        )?;

        // Verify API key is still active
        let key_id = Uuid::parse_str(&token_data.claims.sub)?;
        let api_key = self.db.api_keys()
            .find_by_id(key_id)
            .await?
            .ok_or_else(|| anyhow!("API key not found"))?;

        if !api_key.is_active {
            return Err(anyhow!("API key has been revoked"));
        }

        Ok(token_data.claims)
    }

    // Revoke API key
    pub async fn revoke_api_key(&self, key_id: Uuid) -> Result<()> {
        self.db.api_keys().set_inactive(key_id).await
    }
}

// Axum middleware for JWT authentication
pub async fn auth_middleware(
    State(auth): State<Arc<AuthService>>,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = auth_header.token();

    match auth.validate_token(token).await {
        Ok(claims) => {
            // Attach claims to request extensions
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(e) => {
            tracing::warn!("Authentication failed: {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
```

#### Key Dependencies:

```toml
# Authentication
jsonwebtoken = "9.3"              # JWT token handling
argon2 = "0.5"                    # API key hashing
chacha20poly1305 = "0.10"         # Encryption for API keys
axum-extra = "0.9"                # Additional Axum extractors
tower = "0.4"                     # Middleware framework
tower-http = { version = "0.5", features = ["auth"] }
```

### Benefits of This Approach:

1. **Simplicity**: No user management complexity, just API keys
2. **Secure**: Keys are hashed and encrypted at rest
3. **Stateless**: JWT tokens don't require server lookups once validated
4. **Scalable**: No session storage needed
5. **Headless-First**: Perfect for API-only service
6. **Performance**: Fast authentication with minimal database queries

## 5. CLI Interface for API Key Management

### MetaMCP CLI Tool

**API keys are managed via a dedicated CLI tool**, not through API endpoints. This ensures that only administrators with direct access to the server can create and manage API keys.

#### Why CLI for API Key Management:

1. **Security**: No public API endpoint for key generation prevents unauthorized key creation
2. **Access Control**: Only users with server/database access can manage keys
3. **Audit Trail**: CLI operations can be logged separately from API requests
4. **Simplicity**: No need to bootstrap authentication for key management

#### CLI Commands

```bash
# List all API keys
metamcp-cli keys list

# Create a new API key
metamcp-cli keys create --name "Production Client" --description "Main production MCP client"

# Show API key details (without revealing the key)
metamcp-cli keys show <key-id>

# Inactivate an API key (soft delete - can be reactivated)
metamcp-cli keys inactivate <key-id>

# Reactivate an inactive key
metamcp-cli keys activate <key-id>

# Delete an API key permanently (hard delete)
metamcp-cli keys delete <key-id> --confirm

# Rotate an API key (creates new key, marks old as inactive)
metamcp-cli keys rotate <key-id>
```

#### Implementation

```rust
// src/bin/metamcp-cli.rs

use clap::{Parser, Subcommand};
use sqlx::PgPool;
use metamcp::{auth::AuthService, db::Database};

#[derive(Parser)]
#[command(name = "metamcp-cli")]
#[command(about = "MetaMCP CLI for API key management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage API keys
    Keys {
        #[command(subcommand)]
        action: KeyActions,
    },
}

#[derive(Subcommand)]
enum KeyActions {
    /// List all API keys
    List {
        /// Show inactive keys
        #[arg(long)]
        include_inactive: bool,
    },

    /// Create a new API key
    Create {
        /// Name for the API key
        #[arg(short, long)]
        name: String,

        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Show API key details
    Show {
        /// API key ID
        key_id: String,
    },

    /// Inactivate an API key
    Inactivate {
        /// API key ID
        key_id: String,
    },

    /// Activate an inactive API key
    Activate {
        /// API key ID
        key_id: String,
    },

    /// Delete an API key permanently
    Delete {
        /// API key ID
        key_id: String,

        /// Confirm deletion
        #[arg(long)]
        confirm: bool,
    },

    /// Rotate an API key (create new, inactivate old)
    Rotate {
        /// API key ID to rotate
        key_id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Load configuration
    let config = metamcp::config::load_config()?;

    // Connect to database
    let db = Database::new(&config.database_url).await?;
    let auth_service = AuthService::new(
        config.jwt_secret,
        &config.encryption_key,
        db.clone()
    );

    match cli.command {
        Commands::Keys { action } => handle_key_commands(action, &db, &auth_service).await?,
    }

    Ok(())
}

async fn handle_key_commands(
    action: KeyActions,
    db: &Database,
    auth: &AuthService,
) -> anyhow::Result<()> {
    match action {
        KeyActions::List { include_inactive } => {
            let keys = db.api_keys().list_all(include_inactive).await?;

            println!("\n{:<36} {:<30} {:<10} {:<20}", "ID", "Name", "Status", "Created");
            println!("{}", "-".repeat(100));

            for key in keys {
                let status = if key.is_active { "Active" } else { "Inactive" };
                println!(
                    "{:<36} {:<30} {:<10} {}",
                    key.id,
                    key.name,
                    status,
                    key.created_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
            println!();
        }

        KeyActions::Create { name, description } => {
            let (api_key, stored_key) = auth.generate_api_key(name.clone()).await?;

            println!("\n✅ API Key created successfully!");
            println!("\nKey ID: {}", stored_key.id);
            println!("Name: {}", stored_key.name);
            if let Some(desc) = description {
                println!("Description: {}", desc);
            }
            println!("\n⚠️  IMPORTANT: Save this API key now. It won't be shown again!");
            println!("\nAPI Key: {}\n", api_key);
        }

        KeyActions::Show { key_id } => {
            let key_uuid = uuid::Uuid::parse_str(&key_id)?;
            let key = db.api_keys().find_by_id(key_uuid).await?
                .ok_or_else(|| anyhow::anyhow!("API key not found"))?;

            println!("\nAPI Key Details:");
            println!("ID: {}", key.id);
            println!("Name: {}", key.name);
            println!("Status: {}", if key.is_active { "Active" } else { "Inactive" });
            println!("Created: {}", key.created_at.format("%Y-%m-%d %H:%M:%S"));
            if let Some(last_used) = key.last_used_at {
                println!("Last Used: {}", last_used.format("%Y-%m-%d %H:%M:%S"));
            } else {
                println!("Last Used: Never");
            }
            println!();
        }

        KeyActions::Inactivate { key_id } => {
            let key_uuid = uuid::Uuid::parse_str(&key_id)?;
            db.api_keys().set_inactive(key_uuid).await?;
            println!("\n✅ API key inactivated successfully\n");
        }

        KeyActions::Activate { key_id } => {
            let key_uuid = uuid::Uuid::parse_str(&key_id)?;
            db.api_keys().set_active(key_uuid).await?;
            println!("\n✅ API key activated successfully\n");
        }

        KeyActions::Delete { key_id, confirm } => {
            if !confirm {
                eprintln!("\n❌ Error: Must use --confirm flag to delete an API key\n");
                std::process::exit(1);
            }

            let key_uuid = uuid::Uuid::parse_str(&key_id)?;
            db.api_keys().delete(key_uuid).await?;
            println!("\n✅ API key deleted permanently\n");
        }

        KeyActions::Rotate { key_id } => {
            let key_uuid = uuid::Uuid::parse_str(&key_id)?;
            let old_key = db.api_keys().find_by_id(key_uuid).await?
                .ok_or_else(|| anyhow::anyhow!("API key not found"))?;

            // Create new key with same name (appended with timestamp)
            let new_name = format!("{} (rotated {})", old_key.name, chrono::Utc::now().format("%Y-%m-%d"));
            let (new_api_key, new_stored_key) = auth.generate_api_key(new_name).await?;

            // Inactivate old key
            db.api_keys().set_inactive(key_uuid).await?;

            println!("\n✅ API key rotated successfully!");
            println!("\nOld Key ID: {} (now inactive)", old_key.id);
            println!("New Key ID: {}", new_stored_key.id);
            println!("\n⚠️  IMPORTANT: Save this new API key now!");
            println!("\nNew API Key: {}\n", new_api_key);
        }
    }

    Ok(())
}
```

#### Key Dependencies for CLI

```toml
# In Cargo.toml
[[bin]]
name = "metamcp-cli"
path = "src/bin/metamcp-cli.rs"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
```

### Benefits of CLI Approach

1. **Secure**: No public endpoint for key creation
2. **Simple**: Standard CLI tool, works like other database admin tools
3. **Flexible**: Easy to add more admin commands later
4. **Scriptable**: Can be used in deployment scripts
5. **Auditable**: Can integrate with system logging

## 6. RPC/API Design Pattern

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
        authenticate,
        list_mcp_servers,
        get_mcp_server,
        create_mcp_server,
        delete_mcp_server,
        execute_mcp_tool,
        stream_mcp_messages
    ),
    components(
        schemas(McpServer, McpToolRequest, McpToolResponse, AuthRequest, AuthResponse, ErrorResponse)
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "JWT token authentication"),
        (name = "mcp", description = "MCP server operations")
    ),
    info(
        title = "MetaMCP API",
        version = "1.0.0",
        description = "Headless API backend for MetaMCP - MCP Protocol Proxy with streaming HTTP",
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
struct AuthRequest {
    #[schema(example = "mcp_a1b2c3d4e5f6")]
    api_key: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct AuthResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    access_token: String,
    #[schema(example = "Bearer")]
    token_type: String,
    #[schema(example = 900)]
    expires_in: u64,
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
    path = "/api/v1/auth/token",
    tag = "auth",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "JWT token generated", body = AuthResponse),
        (status = 401, description = "Invalid API key", body = ErrorResponse)
    )
)]
async fn authenticate(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let token = state.auth.authenticate_with_api_key(&payload.api_key).await?;

    Ok(Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 900, // 15 minutes
    }))
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
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/auth/token", post(authenticate));  // Get JWT from API key

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // MCP server management
        .route("/api/v1/mcp/servers", get(list_mcp_servers).post(create_mcp_server))
        .route("/api/v1/mcp/servers/:id", get(get_mcp_server).delete(delete_mcp_server))
        .route("/api/v1/mcp/servers/:server_id/tools", get(list_tools))
        .route("/api/v1/mcp/servers/:server_id/tools/:tool_name/execute", post(execute_mcp_tool))

        // Streaming HTTP endpoint for MCP communication
        .route("/api/v1/mcp/stream", post(stream_mcp_messages))

        // Apply authentication middleware
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // Serve OpenAPI documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
```

**Note**: API keys are created via the `metamcp-cli` tool, not through API endpoints. This ensures secure key management.
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

## 7. Streaming Architecture & Protocol Translation

### Client Communication: Streaming HTTP ONLY

**MetaMCP clients communicate with the server using ONLY streaming HTTP** (not SSE, not WebSockets). This uses standard HTTP with chunked transfer encoding for bidirectional streaming.

### Backend MCP Servers: SSE or stdio

**Backend MCP servers can use:**
1. **Streaming HTTP** - for HTTP-based MCP servers (initial version)
2. **SSE (Server-Sent Events)** - for HTTP-based MCP servers (future)
3. **stdio** - for process-based MCP servers (future)

**Initial Version**: Only streaming HTTP backend servers supported. SSE and stdio translation will be added later.

### Protocol Translation Architecture

```
┌─────────────┐                    ┌──────────────┐                    ┌─────────────┐
│             │  Streaming HTTP    │              │   Streaming HTTP   │             │
│  MCP Client ├───────────────────►│   MetaMCP    ├───────────────────►│ MCP Backend │
│             │◄───────────────────┤    Server    │◄───────────────────┤   Server    │
└─────────────┘  HTTP Chunks       │              │   SSE/stdio (v2)   └─────────────┘
                                    │              │
                                    │  Protocol    │
                                    │  Translation │
                                    └──────────────┘
```

#### Why Streaming HTTP for Clients:

1. **Simplicity**: Standard HTTP with chunked transfer encoding
2. **Universal Support**: Works with any HTTP client library
3. **No Special Protocols**: No WebSocket or SSE dependencies
4. **Bidirectional**: Request/response streaming
5. **Firewall Friendly**: Pure HTTP/HTTPS traffic

#### Implementation with Tokio Streams

Internal implementation uses Tokio streams for efficient message handling, even though the external API is streaming HTTP.

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
├── Cargo.toml                     # Define both server and CLI binaries
├── Cargo.lock
├── README.md
├── MIGRATION_PLAN.md
│
├── src/
│   ├── main.rs                    # Main server entry point
│   ├── lib.rs                     # Library exports (shared by server and CLI)
│   │
│   ├── bin/
│   │   └── metamcp-cli.rs         # CLI tool for API key management
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs            # Configuration management
│   │
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── api_key.rs             # API key management
│   │   ├── jwt.rs                 # JWT token handling
│   │   └── middleware.rs          # Auth middleware
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── api_key.rs         # API key model
│   │   │   └── mcp_server.rs      # MCP server config model
│   │   ├── repositories/
│   │   │   ├── mod.rs
│   │   │   ├── api_key.rs         # API key repository
│   │   │   └── mcp_server.rs      # MCP server repository
│   │   └── migrations/
│   │
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── protocol/
│   │   │   ├── mod.rs
│   │   │   ├── types.rs           # MCP protocol types
│   │   │   └── translator.rs      # Protocol translation (HTTP <-> SSE/stdio)
│   │   ├── proxy.rs               # MCP proxy/router
│   │   ├── server_manager.rs      # Backend MCP server management
│   │   └── client_handler.rs      # Client connection handler
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs            # Auth endpoints
│   │   │   ├── mcp.rs             # MCP operations
│   │   │   └── health.rs          # Health check
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   └── mcp.rs
│   │   └── middleware/
│   │       ├── mod.rs
│   │       └── auth.rs
│   │
│   ├── streaming/
│   │   ├── mod.rs
│   │   ├── http_stream.rs         # Streaming HTTP implementation
│   │   └── manager.rs             # Stream manager
│   │
│   └── utils/
│       ├── mod.rs
│       └── error.rs               # Error types
│
├── migrations/
│   ├── 001_create_api_keys.sql
│   └── 002_create_mcp_servers.sql
│
├── examples/
│   ├── test_client.rs             # Test MCP client
│   ├── backend_server_1.rs        # Test backend MCP server #1
│   └── backend_server_2.rs        # Test backend MCP server #2
│
├── tests/
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── auth_flow.rs           # API key + JWT flow tests
│   │   ├── cli_tests.rs           # CLI tool integration tests
│   │   ├── mcp_proxy.rs           # MCP proxying tests
│   │   └── end_to_end.rs          # Full client -> MetaMCP -> backend tests
│   └── unit/
│       ├── mod.rs
│       ├── auth.rs
│       ├── protocol.rs
│       └── streaming.rs
│
└── benches/
    ├── auth_bench.rs
    └── streaming_bench.rs
```

## 9. Migration Phases

### Phase 1: Core Infrastructure ✅ COMPLETED
- [x] Set up Rust project with Axum web framework (`src/main.rs`, `Cargo.toml`)
- [x] Implement configuration management (`src/config/mod.rs`, `src/config/settings.rs`)
- [x] Set up database connection pool with SQLx (`src/db/mod.rs`)
- [x] Create error handling framework with thiserror + anyhow (`src/utils/error.rs`)
- [x] Implement logging and tracing with tracing-subscriber

### Phase 2: Database Layer & API Keys ✅ COMPLETED
- [x] Define API keys schema (`migrations/20240101000001_create_api_keys.sql`)
- [x] Define MCP server configuration schema (`migrations/20240101000002_create_mcp_servers.sql`)
- [x] Create SQLx migrations with proper indexes and constraints
- [x] Implement API key repository (`src/db/repositories/api_key.rs`)
- [x] Implement MCP server repository (`src/db/repositories/mcp_server.rs`)
- [x] Add encryption for API keys at rest using ChaCha20Poly1305 (`src/auth/api_key.rs`)

### Phase 3: Authentication System & CLI Tool ✅ COMPLETED
- [x] Implement API key generation and storage (`src/auth/service.rs`)
- [x] Implement JWT token generation from API keys (`src/auth/jwt.rs`)
- [x] Create JWT validation middleware (`src/auth/middleware.rs`)
- [x] Add API key revocation support
- [x] Create auth endpoint `/api/v1/auth/token` (`src/api/handlers/auth.rs`)
- [x] **Build CLI tool (metamcp-cli)** (`src/bin/metamcp-cli.rs`)
  - [x] Implement `keys list` command with `--include-inactive` flag
  - [x] Implement `keys create` command with `--name` option
  - [x] Implement `keys show` command with detailed output
  - [x] Implement `keys inactivate` command
  - [x] Implement `keys activate` command
  - [x] Implement `keys delete` command with `--confirm` safety flag
  - [x] Implement `keys rotate` command (creates new key, inactivates old)
- [x] CLI tool tested with database operations

### Phase 4: MCP Protocol Implementation ✅ COMPLETED
- [x] Define MCP protocol types (`src/mcp/protocol/types.rs`)
  - [x] JSON-RPC request/response/notification structures
  - [x] MCP capabilities (tools, resources, prompts)
  - [x] Tool, Resource, and Prompt definitions
  - [x] Content types (text, image, resource)
- [x] Create MCP proxy/router structure (`src/mcp/proxy.rs`)
- [x] Add backend server management (`src/mcp/server_manager.rs`)
  - [x] Spawn server processes with tokio::process
  - [x] Stop server with graceful shutdown + force kill fallback
  - [x] Monitor server health in background task
  - [x] Send messages to server stdin
  - [x] Handle server stderr logging
- [x] Implement protocol message parsing and validation

### Phase 5: API Endpoints ✅ COMPLETED
- [x] Implement MCP server management endpoints (`src/api/handlers/mcp.rs`)
  - [x] `GET /api/v1/mcp/servers` - List all servers
  - [x] `GET /api/v1/mcp/servers/{server_id}` - Get server details
  - [x] `POST /api/v1/mcp/servers` - Create new server
  - [x] `PUT /api/v1/mcp/servers/{server_id}` - Update server
  - [x] `DELETE /api/v1/mcp/servers/{server_id}` - Delete server
- [x] Implement MCP tool execution endpoint
  - [x] `POST /api/v1/mcp/servers/{server_id}/tools/{tool_name}/execute`
- [x] Add request validation with validator crate
- [x] Create OpenAPI documentation with utoipa (`src/api/mod.rs`)
- [x] Add Swagger UI at `/swagger-ui`

### Phase 6: Streaming HTTP Implementation 🔄 IN PROGRESS
- [x] Create stream manager for connection handling (`src/streaming/manager.rs`)
  - [x] StreamEvent types for various events
  - [x] EventFilters for client subscriptions
  - [x] Broadcast channels for system-wide events
  - [x] Per-client channels for targeted events
  - [x] Client registration/unregistration
- [ ] Implement streaming HTTP endpoint in routes
- [ ] Add chunked transfer encoding for responses
- [ ] Implement bidirectional streaming support
- [ ] Add backpressure and flow control
- [ ] Add connection cleanup on disconnect

### Phase 7: Testing Components ⬜ NOT STARTED
- [ ] Create test MCP client (`examples/test_client.rs`)
- [ ] Create test backend MCP server #1 (`examples/backend_server_1.rs`)
- [ ] Create test backend MCP server #2 (`examples/backend_server_2.rs`)
- [ ] Write unit tests for all modules
- [ ] Create integration tests (auth flow, MCP proxy, end-to-end)
- [ ] Add performance benchmarks

### Phase 8: Optimization & Documentation ⬜ NOT STARTED
- [ ] Optimize hot paths (streaming, protocol translation)
- [ ] Add comprehensive error messages
- [ ] Update README with setup instructions
- [ ] Document API key generation process
- [ ] Add examples for client usage
- [ ] Load testing and performance tuning

### Phase 9: Future Enhancements (Post-MVP) ⬜ NOT STARTED
- [ ] Add SSE support for backend MCP servers
- [ ] Add stdio support for backend MCP servers
- [ ] Implement full protocol translation (HTTP <-> SSE/stdio)
- [ ] Add metrics and monitoring (Prometheus)
- [ ] Add distributed tracing (OpenTelemetry)
- [ ] Implement rate limiting per API key
- [ ] Add API key usage analytics

### Implementation Summary

| Phase | Status | Key Files |
|-------|--------|-----------|
| Phase 1: Core Infrastructure | ✅ Complete | `main.rs`, `config/`, `utils/error.rs` |
| Phase 2: Database Layer | ✅ Complete | `db/`, `migrations/` |
| Phase 3: Authentication & CLI | ✅ Complete | `auth/`, `bin/metamcp-cli.rs` |
| Phase 4: MCP Protocol | ✅ Complete | `mcp/protocol/`, `mcp/server_manager.rs` |
| Phase 5: API Endpoints | ✅ Complete | `api/handlers/`, `api/routes/` |
| Phase 6: Streaming HTTP | 🔄 Partial | `streaming/manager.rs` |
| Phase 7: Testing | ⬜ Not Started | `examples/`, `tests/` |
| Phase 8: Optimization | ⬜ Not Started | - |
| Phase 9: Future | ⬜ Not Started | - |

## 10. Key Design Decisions

### Architecture Decisions Made ✅

1. **Web Framework** ✅ **DECIDED: Axum**
   - Type-safe routing with excellent async performance
   - Tower ecosystem integration for middleware
   - Native support for streaming HTTP
   - Strong community and production-proven

2. **Database Strategy** ✅ **DECIDED: SQLx**
   - SQLx chosen for compile-time SQL validation and seamless async support
   - No ORM overhead, direct SQL queries
   - Built-in connection pooling and migration support
   - Perfect for API key and server config persistence

3. **Authentication Strategy** ✅ **DECIDED: API Key + JWT**
   - NO user accounts, NO OAuth, NO sessions
   - Simple API key generation and storage (encrypted)
   - Short-lived JWT tokens generated from API keys
   - Stateless authentication perfect for API-only service

4. **API Protocol** ✅ **DECIDED: RESTful with OpenAPI**
   - RESTful API with OpenAPI/Swagger documentation
   - Universal client compatibility (any HTTP client works)
   - Auto-generated API documentation
   - Industry standard with excellent tooling

5. **Client Communication** ✅ **DECIDED: Streaming HTTP Only**
   - Clients use ONLY streaming HTTP (chunked transfer encoding)
   - NO SSE or WebSockets for clients
   - Simplifies client implementation
   - Standard HTTP/HTTPS traffic

6. **Backend MCP Servers** ✅ **DECIDED: Streaming HTTP (v1), SSE/stdio (v2)**
   - Initial version: Only streaming HTTP backend servers
   - Future versions: Add SSE and stdio support with protocol translation
   - MetaMCP acts as protocol translator

7. **Process Management** ✅ **DECIDED: tokio::process**
   - Native Tokio integration for async process spawning
   - Stream-based I/O for MCP message handling
   - Built-in support for graceful shutdown
   - Used for spawning backend MCP server processes

8. **MCP Protocol** ✅ **DECIDED: Native Rust Implementation**
   - Implement MCP protocol natively in Rust
   - No FFI bindings to JavaScript SDK
   - Full control over protocol implementation
   - Protocol translation layer for SSE/stdio (future)

9. **Error Handling** ✅ **DECIDED: thiserror + anyhow**
   - `thiserror` for library errors (structured)
   - `anyhow` for application errors (contextual)
   - Custom error types with `From` implementations

10. **Configuration** ✅ **DECIDED: Environment Variables + TOML**
    - Environment variables for secrets and deployment config
    - TOML file for application settings
    - No runtime reloading initially (restart required)

## 11. Dependencies Recommendation

```toml
[dependencies]
# Web Framework
axum = { version = "0.7", features = ["http2"] }  # HTTP/2 for streaming

# Async Runtime
tokio = { version = "1.38", features = ["full"] }

# Streaming
tokio-stream = "0.1"
futures = "0.3"

# Database - SQLx
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "chrono", "migrate"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Authentication (API Key + JWT only)
jsonwebtoken = "9.3"              # JWT token handling
argon2 = "0.5"                    # API key hashing
chacha20poly1305 = "0.10"         # API key encryption at rest
axum-extra = "0.9"                # Additional extractors for Bearer tokens
tower = "0.4"                     # Middleware framework
tower-http = { version = "0.5", features = ["auth", "cors"] }

# Error Handling
thiserror = "1.0"                 # Structured errors
anyhow = "1.0"                    # Context-rich errors

# Logging & Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Configuration
config = "0.14"                   # TOML configuration
dotenv = "0.15"                   # Environment variables

# UUID and Time
uuid = { version = "1.10", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# HTTP Client (for backend MCP servers)
reqwest = { version = "0.12", features = ["json", "stream"] }

# Validation
validator = { version = "0.18", features = ["derive"] }

# OpenAPI/REST API Documentation
utoipa = { version = "4.2", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "6.0", features = ["axum"] }

# CLI Tool
clap = { version = "4.5", features = ["derive"] }

[dev-dependencies]
# Testing
tokio-test = "0.4"
wiremock = "0.6"                  # HTTP mocking for tests
criterion = "0.5"                 # Benchmarking
```

### Global Tools

```bash
# SQLx CLI for migrations
cargo install sqlx-cli --no-default-features --features postgres

# Development tools
cargo install cargo-watch          # Auto-rebuild on changes
cargo install cargo-nextest        # Better test runner
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

## 16. Testing Strategy

### Test Components

1. **Test MCP Client** (`examples/test_client.rs`)
   - Implements MCP client using streaming HTTP
   - Tests API key authentication and JWT token generation
   - Sends various MCP protocol messages (tools.list, tools.call, etc.)
   - Validates responses from MetaMCP server

2. **Backend Test Server #1** (`examples/backend_server_1.rs`)
   - Simple MCP server providing basic tools (echo, math operations)
   - Implements streaming HTTP MCP protocol
   - Used for basic end-to-end testing

3. **Backend Test Server #2** (`examples/backend_server_2.rs`)
   - More complex MCP server with resources and prompts
   - Implements streaming HTTP MCP protocol
   - Used for testing advanced MCP features

### End-to-End Test Flow

```
Test Client  →  MetaMCP Server  →  Backend Server #1
                     ↓
                 API Key DB
                     ↓
              Backend Server #2
```

1. Test client obtains API key
2. Test client generates JWT token
3. Test client connects to MetaMCP via streaming HTTP
4. MetaMCP validates JWT and proxies to backend servers
5. Responses flow back through MetaMCP to client
6. All communication verified and logged

## 17. Development Environment Setup

### PostgreSQL in Docker

For development, run PostgreSQL in a Docker container for easy setup and isolation.

**Configuration Strategy:**
All database configuration is centralized in a `.env` file, which is:
- Used by Docker Compose for container configuration
- Used by SQLx CLI for running migrations
- Used by the MetaMCP application for database connections

This approach:
- ✅ Keeps credentials in one place
- ✅ Makes it easy to change environments (dev/staging/prod)
- ✅ Follows 12-factor app principles
- ✅ Prevents hardcoding credentials in docker-compose.yml

#### Option 1: Docker Compose (Recommended)

Create a `docker-compose.yml` in the project root:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    container_name: metamcp-postgres
    restart: unless-stopped
    environment:
      # Uses variables from .env file with defaults as fallback
      POSTGRES_USER: ${POSTGRES_USER:-metamcp}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-metamcp_dev_password}
      POSTGRES_DB: ${POSTGRES_DB:-metamcp_dev}
      POSTGRES_INITDB_ARGS: "-E UTF8"
    ports:
      - "${POSTGRES_PORT:-5432}:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-metamcp} -d ${POSTGRES_DB:-metamcp_dev}"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
    driver: local
```

**How it works:**
- Docker Compose automatically reads `.env` file from the project root
- Variables like `${POSTGRES_USER:-metamcp}` use `.env` values or defaults
- All database configuration comes from a single `.env` file
- Easy to change credentials without editing `docker-compose.yml`

**Usage:**

```bash
# Start PostgreSQL
docker-compose up -d postgres

# Check status
docker-compose ps

# View logs
docker-compose logs -f postgres

# Stop PostgreSQL
docker-compose down

# Stop and remove data (fresh start)
docker-compose down -v
```

#### Option 2: Docker Run Command

```bash
# Create a docker volume for data persistence
docker volume create metamcp-postgres-data

# Run PostgreSQL container
docker run -d \
  --name metamcp-postgres \
  -e POSTGRES_USER=metamcp \
  -e POSTGRES_PASSWORD=metamcp_dev_password \
  -e POSTGRES_DB=metamcp_dev \
  -p 5432:5432 \
  -v metamcp-postgres-data:/var/lib/postgresql/data \
  --restart unless-stopped \
  postgres:16-alpine

# Check container is running
docker ps | grep metamcp-postgres

# View logs
docker logs -f metamcp-postgres

# Stop container
docker stop metamcp-postgres

# Start container
docker start metamcp-postgres

# Remove container (keeps data in volume)
docker rm metamcp-postgres

# Remove container and data
docker rm metamcp-postgres
docker volume rm metamcp-postgres-data
```

#### Connection Configuration

Create a `.env` file in the project root (copy from `.env.example`):

```bash
cp .env.example .env
```

The `.env` file should contain:

```env
# ============================================================================
# Database Configuration (used by both Docker and application)
# ============================================================================

# PostgreSQL Docker container settings
POSTGRES_USER=metamcp
POSTGRES_PASSWORD=metamcp_dev_password
POSTGRES_DB=metamcp_dev
POSTGRES_HOST=localhost
POSTGRES_PORT=5432

# Database connection string (must match above variables)
DATABASE_URL=postgresql://metamcp:metamcp_dev_password@localhost:5432/metamcp_dev

# ============================================================================
# Security Configuration
# ============================================================================

# JWT Secret (generate with: openssl rand -base64 32)
JWT_SECRET=your-secret-key-here

# Encryption Key for API Keys (generate with: openssl rand -hex 32)
ENCRYPTION_KEY=your-encryption-key-here

# ============================================================================
# Server Configuration
# ============================================================================

SERVER_HOST=127.0.0.1
SERVER_PORT=12009

# ============================================================================
# Logging Configuration
# ============================================================================

RUST_LOG=info,metamcp=debug
```

**Generate secure secrets:**

```bash
# Generate and update secrets in .env
JWT_SECRET=$(openssl rand -base64 32)
ENCRYPTION_KEY=$(openssl rand -hex 32)

echo "JWT_SECRET=${JWT_SECRET}"
echo "ENCRYPTION_KEY=${ENCRYPTION_KEY}"

# Update .env file with these values
```

**Important:** `.env` is already in `.gitignore` to prevent committing secrets to version control.

#### Verify Database Connection

```bash
# Load environment variables
source .env

# Using psql (install postgresql-client if needed)
psql $DATABASE_URL

# Or using Docker with environment variables
docker exec -it metamcp-postgres psql -U $POSTGRES_USER -d $POSTGRES_DB

# Or with hardcoded values (less flexible)
docker exec -it metamcp-postgres psql -U metamcp -d metamcp_dev

# Test queries inside psql
postgres=# SELECT version();
postgres=# \l  -- List databases
postgres=# \dt  -- List tables (after migrations)
postgres=# \q  -- Quit
```

#### Running Migrations

After setting up the database, run SQLx migrations:

```bash
# Install SQLx CLI if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Create migration files (examples)
sqlx migrate add create_api_keys
sqlx migrate add create_mcp_servers

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

#### Database Schema Setup

Create migration files in `migrations/` directory:

**`migrations/20240101000001_create_api_keys.sql`:**

```sql
-- Create API keys table
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    encrypted_key BYTEA NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,

    CONSTRAINT api_keys_name_check CHECK (char_length(name) > 0)
);

-- Index for fast lookup by hash
CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash) WHERE is_active = true;

-- Index for listing active keys
CREATE INDEX idx_api_keys_active ON api_keys(is_active, created_at DESC);
```

**`migrations/20240101000002_create_mcp_servers.sql`:**

```sql
-- Create MCP servers configuration table
CREATE TABLE IF NOT EXISTS mcp_servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    url VARCHAR(512) NOT NULL,
    protocol VARCHAR(50) NOT NULL DEFAULT 'http', -- 'http', 'sse', 'stdio'
    command TEXT,  -- For stdio-based servers
    args JSONB,    -- Command arguments for stdio servers
    env JSONB,     -- Environment variables
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT mcp_servers_name_check CHECK (char_length(name) > 0),
    CONSTRAINT mcp_servers_protocol_check CHECK (protocol IN ('http', 'sse', 'stdio'))
);

-- Index for listing active servers
CREATE INDEX idx_mcp_servers_active ON mcp_servers(is_active, name);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_mcp_servers_updated_at
    BEFORE UPDATE ON mcp_servers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

#### SQLx Offline Mode (Optional)

For CI/CD or offline compilation, prepare SQLx metadata:

```bash
# Set DATABASE_URL in .env first
export DATABASE_URL=postgresql://metamcp:metamcp_dev_password@localhost:5432/metamcp_dev

# Prepare metadata (after migrations are run)
cargo sqlx prepare

# This creates .sqlx/ directory with query metadata
# Commit .sqlx/ to version control for offline builds
```

In `Cargo.toml`, enable offline mode:

```toml
[dependencies]
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "chrono", "migrate", "offline"] }
```

#### Docker Compose Full Stack (Optional)

For a complete development environment with pgAdmin, the `docker-compose.yml` already includes pgAdmin (commented out). To enable it:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    container_name: metamcp-postgres
    restart: unless-stopped
    environment:
      POSTGRES_USER: ${POSTGRES_USER:-metamcp}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-metamcp_dev_password}
      POSTGRES_DB: ${POSTGRES_DB:-metamcp_dev}
      POSTGRES_INITDB_ARGS: "-E UTF8"
    ports:
      - "${POSTGRES_PORT:-5432}:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-metamcp} -d ${POSTGRES_DB:-metamcp_dev}"]
      interval: 10s
      timeout: 5s
      retries: 5

  # Optional: pgAdmin for database management UI
  pgadmin:
    image: dpage/pgadmin4:latest
    container_name: metamcp-pgadmin
    restart: unless-stopped
    environment:
      PGADMIN_DEFAULT_EMAIL: ${PGADMIN_EMAIL:-admin@metamcp.local}
      PGADMIN_DEFAULT_PASSWORD: ${PGADMIN_PASSWORD:-admin}
      PGADMIN_CONFIG_SERVER_MODE: 'False'
    ports:
      - "5050:80"
    depends_on:
      - postgres
    volumes:
      - pgadmin_data:/var/lib/pgadmin

volumes:
  postgres_data:
    driver: local
  pgadmin_data:
    driver: local
```

**Start full stack:**
```bash
docker-compose up -d postgres pgadmin
```

**Access pgAdmin:**
- URL: http://localhost:5050
- Email: `admin@metamcp.local` (or from `PGADMIN_EMAIL` in `.env`)
- Password: `admin` (or from `PGADMIN_PASSWORD` in `.env`)

**Connect pgAdmin to PostgreSQL:**
1. Click "Add New Server"
2. General tab: Name = "MetaMCP Local"
3. Connection tab:
   - Host: `postgres` (Docker service name)
   - Port: `5432`
   - Username: Value of `POSTGRES_USER` from `.env`
   - Password: Value of `POSTGRES_PASSWORD` from `.env`
   - Database: Value of `POSTGRES_DB` from `.env`

#### Troubleshooting

**Port already in use:**
```bash
# Check what's using port 5432
lsof -i :5432

# Stop local PostgreSQL if running
sudo systemctl stop postgresql  # Linux
brew services stop postgresql   # macOS

# Or change port in .env file
echo "POSTGRES_PORT=5433" >> .env
# Then restart: docker-compose down && docker-compose up -d postgres
```

**Connection refused:**
```bash
# Check Docker container is running
docker ps | grep postgres

# Check container logs
docker logs metamcp-postgres

# Verify network connectivity (using .env variables)
source .env
docker exec metamcp-postgres pg_isready -U $POSTGRES_USER -d $POSTGRES_DB
```

**Database credentials mismatch:**
```bash
# Ensure all variables in .env are consistent
source .env
echo "User: $POSTGRES_USER"
echo "DB: $POSTGRES_DB"
echo "URL: $DATABASE_URL"

# DATABASE_URL should match: postgresql://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST:$POSTGRES_PORT/$POSTGRES_DB
```

**Reset database:**
```bash
# Stop and remove everything (WARNING: deletes all data)
docker-compose down -v

# Start fresh
docker-compose up -d postgres

# Run migrations
sqlx migrate run
```

**Docker Compose not reading .env:**
```bash
# Verify .env file exists in project root
ls -la .env

# Verify .env has proper format (no spaces around =)
# Good: POSTGRES_USER=metamcp
# Bad:  POSTGRES_USER = metamcp

# Test variable expansion
docker-compose config | grep POSTGRES_USER
```

## 18. Next Steps

### Quick Start Guide

1. **Initialize Rust project**
   ```bash
   cargo new --bin metamcp_rust
   cd metamcp_rust
   ```

2. **Configure environment** (FIRST - before Docker)
   ```bash
   # Copy environment template
   cp .env.example .env

   # Generate and set secrets
   JWT_SECRET=$(openssl rand -base64 32)
   ENCRYPTION_KEY=$(openssl rand -hex 32)

   # Update .env file
   sed -i.bak "s|JWT_SECRET=.*|JWT_SECRET=${JWT_SECRET}|" .env
   sed -i.bak "s|ENCRYPTION_KEY=.*|ENCRYPTION_KEY=${ENCRYPTION_KEY}|" .env

   # Verify .env contains database credentials
   cat .env | grep POSTGRES_
   ```

3. **Set up PostgreSQL with Docker** (See Section 17 above)
   ```bash
   # Docker Compose will automatically read .env file
   docker-compose up -d postgres

   # Verify connection (using credentials from .env)
   source .env
   docker exec -it metamcp-postgres psql -U $POSTGRES_USER -d $POSTGRES_DB
   ```

4. **Set up database schema**
   ```bash
   # Install SQLx CLI
   cargo install sqlx-cli --no-default-features --features postgres

   # Create and run migrations
   sqlx migrate add create_api_keys
   sqlx migrate add create_mcp_servers
   sqlx migrate run
   ```

5. **Begin Phase 1 implementation**
   - Core infrastructure
   - Configuration management
   - Database connection pool
   - Error handling framework

6. **Build CLI tool early (Phase 3)**
   - Implement metamcp-cli for API key management
   - Test with database operations
   - Use CLI to create first API key for development

7. **Create test components**
   - Test client for development workflow
   - Basic backend servers for integration testing
   - Use CLI-generated API keys for testing

### Example Workflow

```bash
# After Phase 3 is complete:

# 1. Create an API key using CLI
cargo run --bin metamcp-cli keys create --name "Dev Client"

# 2. Save the API key (shown only once)
# API Key: mcp_a1b2c3d4e5f6...

# 3. Start the server
cargo run --bin metamcp

# 4. Get JWT token using API key
curl -X POST http://localhost:12009/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"api_key": "mcp_a1b2c3d4e5f6..."}'

# 5. Use JWT token for authenticated requests
curl http://localhost:12009/api/v1/mcp/servers \
  -H "Authorization: Bearer <jwt-token>"
```

## 19. Summary

This migration plan outlines a **simplified, focused architecture** for MetaMCP in Rust:

### Key Simplifications
- ✅ **NO user management** - Only API keys
- ✅ **NO OAuth/sessions** - Simple JWT from API keys
- ✅ **NO WebSockets/SSE for clients** - Only streaming HTTP
- ✅ **NO complex authentication** - Just API key + JWT

### Core Focus
- 🎯 **MCP protocol proxy** with streaming HTTP
- 🎯 **Protocol translation** (HTTP ↔ SSE/stdio in future)
- 🎯 **High performance** with Rust/Tokio
- 🎯 **Type safety** with SQLx and strong typing

### MVP Deliverables
1. **CLI tool** for secure API key management (list, create, inactivate, delete, rotate)
2. JWT token authentication endpoint
3. MCP server configuration and management
4. Streaming HTTP for client communication
5. MCP protocol message routing
6. Test client and backend servers
7. OpenAPI documentation

### Key Components

1. **MetaMCP Server** - Main Axum-based server for MCP proxying
2. **MetaMCP CLI** - Command-line tool for API key administration
3. **Test Client** - Example MCP client using streaming HTTP
4. **Backend Test Servers** - Two example MCP servers for testing

The architecture is designed to be:
- **Simple**: Minimal moving parts, clear responsibilities
- **Secure**: API keys managed via CLI, not API endpoints
- **Performant**: Rust + Tokio + streaming HTTP
- **Extensible**: Easy to add SSE/stdio translation later
- **Testable**: Test components built from the start