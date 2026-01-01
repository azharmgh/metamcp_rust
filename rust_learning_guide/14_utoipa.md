# Utoipa - OpenAPI Documentation

Utoipa is a library for generating OpenAPI (Swagger) documentation from Rust code using derive macros.

## What is Utoipa?

Utoipa provides:
- **OpenAPI generation** - Create OpenAPI 3.x specs from code
- **Derive macros** - Document types and endpoints declaratively
- **Swagger UI** - Integrate interactive API documentation
- **Type safety** - Documentation stays in sync with code

## Why Utoipa?

Documentation that lives with code:
- No separate OpenAPI files to maintain
- Changes to code automatically update docs
- Compile-time verification
- Integration with Axum, Actix, and more

## Installation

```toml
[dependencies]
utoipa = { version = "5", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "9", features = ["axum"] }
```

Features:
- `axum_extras` - Axum integration
- `uuid` - UUID type support
- `chrono` - DateTime type support

## Basic Concepts

### Schema (Data Types)

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct User {
    /// Unique user identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,

    /// User's display name
    #[schema(example = "Alice")]
    pub name: String,

    /// User's email address
    #[schema(example = "alice@example.com")]
    pub email: String,

    /// Whether the account is active
    #[schema(example = true)]
    pub active: bool,
}
```

### Path (API Endpoint)

```rust
use utoipa::path;

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found")
    ),
    tag = "users"
)]
async fn get_user(Path(id): Path<Uuid>) -> Result<Json<User>, AppError> {
    // ...
}
```

### OpenAPI Document

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_user,
        list_users,
        create_user,
    ),
    components(
        schemas(User, CreateUserRequest)
    ),
    tags(
        (name = "users", description = "User management")
    )
)]
pub struct ApiDoc;
```

## Real Example from MetaMCP

### OpenAPI Definition

From `src/api/mod.rs`:

```rust
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::health_check,
        handlers::auth::authenticate,
        handlers::mcp::list_mcp_servers,
        handlers::mcp::get_mcp_server,
        handlers::mcp::create_mcp_server,
        handlers::mcp::update_mcp_server,
        handlers::mcp::delete_mcp_server,
        handlers::mcp::execute_mcp_tool,
    ),
    components(
        schemas(
            handlers::health::HealthResponse,
            handlers::auth::AuthRequest,
            handlers::auth::AuthResponse,
            handlers::mcp::ListMcpServersResponse,
            handlers::mcp::McpServerInfo,
            handlers::mcp::CreateMcpServerRequest,
            handlers::mcp::UpdateMcpServerRequest,
            handlers::mcp::McpToolRequest,
            handlers::mcp::McpToolResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "mcp", description = "MCP server management")
    ),
    info(
        title = "MetaMCP API",
        version = "1.0.0",
        description = "Headless API backend for MetaMCP - MCP Protocol Proxy Server",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT"),
        contact(
            name = "MetaMCP Team",
            url = "https://github.com/azharmgh/metamcp_rust"
        )
    ),
)]
pub struct ApiDoc;
```

### Adding Swagger UI to Router

From `src/api/mod.rs`:

```rust
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(routes::public_routes())
        .merge(routes::protected_routes(state.clone()))
        // Add Swagger UI at /swagger-ui
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
```

### Request/Response Schemas

From `src/api/handlers/auth.rs`:

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthRequest {
    /// API key for authentication
    #[schema(example = "mcp_a1b2c3d4e5f67890abcdef1234567890")]
    pub api_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// JWT access token
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,

    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,

    /// Token expiration time in seconds
    #[schema(example = 900)]
    pub expires_in: u64,
}
```

### Endpoint Documentation

From `src/api/handlers/auth.rs`:

```rust
#[utoipa::path(
    post,
    path = "/api/v1/auth/token",
    tag = "auth",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "JWT token generated successfully", body = AuthResponse),
        (status = 401, description = "Invalid API key", body = ErrorResponse)
    )
)]
pub async fn authenticate(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
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

From `src/api/handlers/mcp.rs`:

```rust
#[utoipa::path(
    get,
    path = "/api/v1/mcp/servers",
    tag = "mcp",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List of MCP servers", body = ListMcpServersResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_mcp_servers(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
) -> Result<Json<ListMcpServersResponse>, AppError> {
    let servers = state.db.mcp_servers().list_all(false).await?;
    let server_infos: Vec<McpServerInfo> = servers.into_iter().map(Into::into).collect();
    Ok(Json(ListMcpServersResponse { servers: server_infos }))
}

#[utoipa::path(
    get,
    path = "/api/v1/mcp/servers/{server_id}",
    tag = "mcp",
    params(
        ("server_id" = Uuid, Path, description = "MCP server ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "MCP server details", body = McpServerInfo),
        (status = 404, description = "Server not found")
    )
)]
pub async fn get_mcp_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> Result<Json<McpServerInfo>, AppError> {
    let server = state.db.mcp_servers()
        .find_by_id(server_id)
        .await?
        .ok_or_else(|| AppError::NotFound("MCP server not found".to_string()))?;

    Ok(Json(server.into()))
}
```

## Schema Attributes

### Basic Types

```rust
#[derive(ToSchema)]
pub struct Example {
    // String with example
    #[schema(example = "example value")]
    field: String,

    // Number with min/max
    #[schema(minimum = 0, maximum = 100)]
    percentage: u8,

    // Optional field
    #[schema(nullable)]
    optional: Option<String>,

    // Array
    #[schema(example = json!(["a", "b", "c"]))]
    tags: Vec<String>,
}
```

### Enums

```rust
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

// Results in: { "enum": ["active", "inactive", "pending"] }
```

### Nested Types

```rust
#[derive(ToSchema)]
pub struct Order {
    id: Uuid,
    #[schema(inline)]  // Inline the schema instead of $ref
    customer: Customer,
}

#[derive(ToSchema)]
pub struct Customer {
    name: String,
    email: String,
}
```

## Path Attributes

### Parameters

```rust
#[utoipa::path(
    get,
    path = "/api/items/{id}",
    params(
        // Path parameter
        ("id" = u64, Path, description = "Item ID"),

        // Query parameters
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page"),

        // Header
        ("X-Request-ID" = Option<String>, Header, description = "Request ID"),
    ),
    // ...
)]
```

### Request Body

```rust
#[utoipa::path(
    post,
    path = "/api/items",
    request_body(
        content = CreateItemRequest,
        description = "Item to create",
        content_type = "application/json"
    ),
    // Or simply:
    // request_body = CreateItemRequest,
)]
```

### Responses

```rust
#[utoipa::path(
    get,
    path = "/api/items/{id}",
    responses(
        (status = 200, description = "Success", body = Item),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal error"),
    ),
)]
```

### Security

```rust
#[derive(OpenApi)]
#[openapi(
    // ... paths and components ...
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build()
            ),
        );
    }
}
```

[Run schema example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BSerialize%2C%20Deserialize%7D%3B%0A%0A%2F%2F%20Simulating%20ToSchema%20pattern%0A%23%5Bderive(Serialize%2C%20Deserialize%2C%20Debug)%5D%0Astruct%20User%20%7B%0A%20%20%20%20id%3A%20String%2C%0A%20%20%20%20name%3A%20String%2C%0A%20%20%20%20email%3A%20String%2C%0A%7D%0A%0A%23%5Bderive(Serialize)%5D%0Astruct%20OpenApiSchema%20%7B%0A%20%20%20%20%23%5Bserde(rename%20%3D%20%22type%22)%5D%0A%20%20%20%20schema_type%3A%20String%2C%0A%20%20%20%20properties%3A%20std%3A%3Acollections%3A%3AHashMap%3CString%2C%20Property%3E%2C%0A%7D%0A%0A%23%5Bderive(Serialize)%5D%0Astruct%20Property%20%7B%0A%20%20%20%20%23%5Bserde(rename%20%3D%20%22type%22)%5D%0A%20%20%20%20prop_type%3A%20String%2C%0A%20%20%20%20example%3A%20Option%3CString%3E%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20mut%20properties%20%3D%20std%3A%3Acollections%3A%3AHashMap%3A%3Anew()%3B%0A%20%20%20%20properties.insert(%22id%22.to_string()%2C%20Property%20%7B%0A%20%20%20%20%20%20%20%20prop_type%3A%20%22string%22.to_string()%2C%0A%20%20%20%20%20%20%20%20example%3A%20Some(%22550e8400-e29b-41d4-a716-446655440000%22.to_string())%2C%0A%20%20%20%20%7D)%3B%0A%20%20%20%20properties.insert(%22name%22.to_string()%2C%20Property%20%7B%0A%20%20%20%20%20%20%20%20prop_type%3A%20%22string%22.to_string()%2C%0A%20%20%20%20%20%20%20%20example%3A%20Some(%22Alice%22.to_string())%2C%0A%20%20%20%20%7D)%3B%0A%20%20%20%20%0A%20%20%20%20let%20schema%20%3D%20OpenApiSchema%20%7B%0A%20%20%20%20%20%20%20%20schema_type%3A%20%22object%22.to_string()%2C%0A%20%20%20%20%20%20%20%20properties%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Generated%20schema%3A%22)%3B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26schema).unwrap())%3B%0A%7D)

## Integration with Axum

### Complete Setup

```rust
use axum::{Router, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(health_check),
    components(schemas(HealthResponse))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, body = HealthResponse))
)]
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        );

    // ...
}
```

### Access Swagger UI

After running the server:
- Swagger UI: `http://localhost:8080/swagger-ui/`
- OpenAPI JSON: `http://localhost:8080/api-docs/openapi.json`

## Best Practices

### DO

1. **Document all public endpoints** - Complete API coverage
2. **Provide examples** - Help API consumers understand
3. **Use meaningful descriptions** - Doc comments become descriptions
4. **Group with tags** - Organize related endpoints
5. **Document errors** - Include error responses

### DON'T

1. **Don't skip optional fields** - Document them as nullable
2. **Don't forget security** - Add auth requirements
3. **Don't use vague descriptions** - Be specific
4. **Don't duplicate schemas** - Use references

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Code-first** | Docs live with code |
| **Type-safe** | Compile-time verification |
| **Auto-sync** | Docs update with code |
| **Interactive** | Swagger UI included |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Verbosity** | Many attributes needed |
| **Compile time** | Macros add overhead |
| **Learning curve** | OpenAPI concepts required |

## When to Use

**Use Utoipa when:**
- Building REST APIs
- Need API documentation
- Want Swagger UI
- Using Axum or Actix

**Consider alternatives when:**
- GraphQL (use async-graphql)
- Simple internal APIs
- Schema-first preferred (use openapi-generator)

## Further Learning

### Official Resources
- [Utoipa Documentation](https://docs.rs/utoipa)
- [Utoipa GitHub](https://github.com/juhaku/utoipa)
- [OpenAPI Specification](https://swagger.io/specification/)

### Practice
1. Document a CRUD API
2. Add authentication to docs
3. Create nested request/response types
4. Add example requests

## Related Crates

- **utoipa-swagger-ui** - Swagger UI integration
- **utoipa-redoc** - ReDoc alternative
- **utoipa-rapidoc** - RapiDoc alternative
- **paperclip** - Alternative OpenAPI generator
