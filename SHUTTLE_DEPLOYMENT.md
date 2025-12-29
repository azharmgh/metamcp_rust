# MetaMCP Deployment with Shuttle.dev

This guide covers deploying MetaMCP to [Shuttle.dev](https://shuttle.dev), a Rust-native cloud development platform that simplifies deployment and infrastructure management.

## Overview

Shuttle provides:
- **Infrastructure from Code**: Provision databases and secrets via Rust macros
- **Native Axum Support**: First-class integration with the Axum framework
- **Managed PostgreSQL**: Automatic database provisioning
- **Secrets Management**: Secure storage for API keys and configuration
- **Zero DevOps**: No Docker, Kubernetes, or cloud configuration required

## Prerequisites

- Rust toolchain installed
- Shuttle CLI installed
- Shuttle account (free tier available)

## 1. Install Shuttle CLI

### Linux/macOS (Recommended)

```bash
curl -sSfL https://www.shuttle.dev/install | bash
```

### Windows (PowerShell)

```powershell
iwr https://www.shuttle.dev/install -iex
```

### Alternative Methods

```bash
# Using cargo-binstall (faster)
cargo binstall cargo-shuttle

# Using Homebrew (macOS)
brew install cargo-shuttle

# Building from source (slower)
cargo install cargo-shuttle
```

### Verify Installation

```bash
shuttle --version
```

## 2. Authenticate with Shuttle

```bash
shuttle login
```

This opens your browser to sign in via Google, GitHub, or email.

## 3. Project Configuration

### Update Cargo.toml

Add Shuttle dependencies alongside existing ones:

```toml
[dependencies]
# Existing dependencies...
axum = { version = "0.7", features = ["http2"] }
tokio = { version = "1.38", features = ["full"] }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "chrono", "migrate"] }

# Shuttle-specific dependencies
shuttle-runtime = "0.49"
shuttle-axum = "0.49"
shuttle-shared-db = { version = "0.49", features = ["postgres", "sqlx"] }
shuttle-secrets = "0.49"
```

### Create Shuttle Entry Point

Create or modify `src/main.rs` for Shuttle deployment:

```rust
use axum::{Router, routing::get};
use sqlx::PgPool;
use shuttle_runtime::SecretStore;
use std::sync::Arc;

// Import your application modules
use metamcp::{
    api::create_api_router,
    auth::AuthService,
    config::Config,
    db::Database,
};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Load secrets
    let jwt_secret = secrets
        .get("JWT_SECRET")
        .expect("JWT_SECRET not found in Secrets.toml");

    let encryption_key_hex = secrets
        .get("ENCRYPTION_KEY")
        .expect("ENCRYPTION_KEY not found in Secrets.toml");

    let encryption_key = hex::decode(&encryption_key_hex)
        .expect("Invalid ENCRYPTION_KEY hex string");

    let encryption_key: [u8; 32] = encryption_key
        .try_into()
        .expect("ENCRYPTION_KEY must be 32 bytes");

    // Initialize services
    let db = Database::from_pool(pool);
    let auth_service = Arc::new(AuthService::new(
        jwt_secret,
        &encryption_key,
        db.clone(),
    ));

    // Build application state
    let app_state = AppState {
        db,
        auth: auth_service,
    };

    // Create router
    let router = create_api_router(app_state);

    Ok(router.into())
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub auth: Arc<AuthService>,
}
```

### Configure Secrets

Create `Secrets.toml` in your project root:

```toml
# Secrets.toml - DO NOT COMMIT TO VERSION CONTROL

# JWT Secret for token signing (generate with: openssl rand -base64 32)
JWT_SECRET = "your-jwt-secret-here"

# Encryption key for API keys at rest (generate with: openssl rand -hex 32)
ENCRYPTION_KEY = "your-32-byte-hex-encryption-key-here"
```

Create `Secrets.dev.toml` for local development (optional):

```toml
# Secrets.dev.toml - Local development secrets

JWT_SECRET = "dev-jwt-secret-not-for-production"
ENCRYPTION_KEY = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
```

**Important**: Add secrets files to `.gitignore`:

```gitignore
# Shuttle secrets
Secrets.toml
Secrets*.toml
```

### Generate Secrets

```bash
# Generate JWT secret
echo "JWT_SECRET = '$(openssl rand -base64 32)'"

# Generate encryption key (32 bytes = 64 hex chars)
echo "ENCRYPTION_KEY = '$(openssl rand -hex 32)'"
```

## 4. Database Configuration

### Shuttle Shared Database

Shuttle automatically provisions a PostgreSQL 16 database. The connection pool is injected via macro:

```rust
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    // pool is ready to use
}
```

### Local Development with Custom Database

For local development, you can override with your own database:

```rust
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "{secrets.DATABASE_URL}"
    )] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // Uses DATABASE_URL from Secrets.dev.toml locally
    // Uses Shuttle's managed DB in production
}
```

Add to `Secrets.dev.toml`:

```toml
DATABASE_URL = "postgresql://metamcp:metamcp_dev_password@localhost:5432/metamcp_dev"
```

### Migrations

Migrations run automatically on startup. Ensure your `migrations/` directory is included:

```rust
sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .expect("Failed to run migrations");
```

## 5. Local Development

Test your application locally before deploying:

```bash
# Start local development server
shuttle run
```

This will:
1. Start a local PostgreSQL container (or use your custom `local_uri`)
2. Load secrets from `Secrets.dev.toml` (or `Secrets.toml`)
3. Run your application on `http://localhost:8000`

### Local Development with Docker PostgreSQL

If you prefer using your existing Docker PostgreSQL setup:

1. Ensure PostgreSQL is running:
   ```bash
   docker-compose up -d postgres
   ```

2. Add `DATABASE_URL` to `Secrets.dev.toml`:
   ```toml
   DATABASE_URL = "postgresql://metamcp:metamcp_dev_password@localhost:5432/metamcp_dev"
   ```

3. Run with Shuttle:
   ```bash
   shuttle run
   ```

## 6. Deployment

### Initialize Shuttle Project (First Time)

```bash
# Initialize in existing project
shuttle init --from .

# Or create new project
shuttle init --name metamcp
```

### Deploy to Production

```bash
shuttle deploy
```

First deployment may take several minutes as Shuttle:
1. Uploads your code
2. Builds a Docker image
3. Provisions database
4. Configures infrastructure
5. Assigns HTTPS URL

### Deployment Output

After successful deployment, you'll receive:

```
Service Name:  metamcp
Deployment ID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
Status:        running
Last Updated:  2024-01-15T10:30:00Z
URI:           https://metamcp.shuttle.app
```

## 7. Project Structure for Shuttle

Recommended structure with Shuttle support:

```
metamcp_rust/
├── Cargo.toml
├── Shuttle.toml              # Optional: Shuttle configuration
├── Secrets.toml              # Production secrets (DO NOT COMMIT)
├── Secrets.dev.toml          # Development secrets (DO NOT COMMIT)
├── .gitignore
│
├── src/
│   ├── main.rs               # Shuttle entry point
│   ├── lib.rs                # Library exports
│   │
│   ├── bin/
│   │   └── metamcp-cli.rs    # CLI tool (runs separately)
│   │
│   └── ... (other modules)
│
└── migrations/
    ├── 20240101000001_create_api_keys.sql
    └── 20240101000002_create_mcp_servers.sql
```

### Optional: Shuttle.toml

Create `Shuttle.toml` for additional configuration:

```toml
name = "metamcp"

[deploy]
# Include additional files in deployment
include = ["migrations/*"]

# Exclude files from deployment
exclude = ["tests/*", "benches/*", "examples/*"]
```

## 8. CLI Tool Deployment

The `metamcp-cli` tool for API key management runs separately from the web server. Options for CLI deployment:

### Option A: Run CLI Locally Against Production DB

```bash
# Set production DATABASE_URL
export DATABASE_URL="<shuttle-database-url>"

# Run CLI locally
cargo run --bin metamcp-cli keys create --name "Production Client"
```

To get your Shuttle database URL:
```bash
shuttle resource list
```

### Option B: Create Separate CLI Deployment

For a fully cloud-based CLI, consider creating a separate Shuttle project or using a different deployment method for the CLI binary.

### Option C: Admin API Endpoints (Alternative)

Add protected admin endpoints for key management (requires additional authentication):

```rust
// Only if you need web-based key management
// Ensure proper authorization checks
.route("/admin/keys", post(create_api_key).get(list_api_keys))
```

## 9. Managing Resources

### View Project Status

```bash
shuttle status
```

### View Logs

```bash
# Stream logs
shuttle logs --follow

# Get recent logs
shuttle logs --tail 100
```

### View Resources

```bash
shuttle resource list
```

### Delete Secrets

```bash
shuttle resource delete secrets
```

### Stop Deployment

```bash
shuttle stop
```

### Delete Project

```bash
shuttle project delete
```

## 10. Environment-Specific Configuration

### Feature Flags for Shuttle

Use Cargo feature flags to conditionally compile Shuttle-specific code:

```toml
# Cargo.toml
[features]
default = []
shuttle = [
    "shuttle-runtime",
    "shuttle-axum",
    "shuttle-shared-db",
    "shuttle-secrets"
]

[dependencies]
shuttle-runtime = { version = "0.49", optional = true }
shuttle-axum = { version = "0.49", optional = true }
shuttle-shared-db = { version = "0.49", features = ["postgres", "sqlx"], optional = true }
shuttle-secrets = { version = "0.49", optional = true }
```

### Conditional Main Function

```rust
// src/main.rs

#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // Shuttle deployment
    let router = setup_app(pool, secrets).await;
    Ok(router.into())
}

#[cfg(not(feature = "shuttle"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Standard deployment (Docker, bare metal, etc.)
    dotenv::dotenv().ok();

    let config = Config::from_env()?;
    let pool = PgPool::connect(&config.database_url).await?;

    let router = setup_app_from_config(pool, config).await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:12009").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
```

Build commands:
```bash
# Standard build
cargo build --release

# Shuttle build
cargo build --release --features shuttle
```

## 11. Production Considerations

### Shuttle Shared Database Limitations

The shared PostgreSQL database is suitable for:
- Development and testing
- Small to medium production workloads
- Prototyping

For high-traffic production:
- Consider Shuttle's dedicated AWS RDS option
- Performance may vary during peak usage periods

### Scaling

Shuttle handles scaling automatically. For specific requirements, contact Shuttle support or consider their paid tiers.

### Custom Domains

Configure custom domains through the Shuttle Console or CLI:

```bash
shuttle domain add api.yourdomain.com
```

### Health Checks

Shuttle performs automatic health checks. Ensure your `/health` endpoint responds correctly:

```rust
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}
```

## 12. Troubleshooting

### Build Failures

```bash
# Check build logs
shuttle logs --latest

# Build locally first to catch errors
cargo build --release --features shuttle
```

### Database Connection Issues

```bash
# Check database resource
shuttle resource list

# View connection details
shuttle resource show database
```

### Secrets Not Loading

```bash
# Verify secrets are deployed
shuttle resource list

# Redeploy with secrets
shuttle deploy
```

### Migration Failures

Ensure migrations are idempotent and handle existing tables:

```sql
CREATE TABLE IF NOT EXISTS api_keys (...);
```

## 13. Comparison: Shuttle vs. Traditional Deployment

| Aspect | Shuttle | Traditional (Docker/K8s) |
|--------|---------|-------------------------|
| Setup Time | Minutes | Hours to Days |
| Database | Automatic | Manual provisioning |
| Secrets | Built-in `Secrets.toml` | External secret manager |
| Scaling | Automatic | Manual configuration |
| Cost | Pay-per-use | Fixed infrastructure |
| Control | Limited | Full control |
| Best For | Rapid deployment, MVPs | Production at scale |

## 14. Quick Reference

```bash
# Installation
curl -sSfL https://www.shuttle.dev/install | bash

# Authentication
shuttle login

# Initialize project
shuttle init

# Local development
shuttle run

# Deploy
shuttle deploy

# View logs
shuttle logs --follow

# Check status
shuttle status

# List resources
shuttle resource list

# Stop service
shuttle stop

# Delete project
shuttle project delete
```

## Resources

- [Shuttle Documentation](https://docs.shuttle.dev)
- [Shuttle Examples](https://github.com/shuttle-hq/shuttle-examples)
- [Shuttle Discord](https://discord.gg/shuttle)
- [Axum Integration Guide](https://docs.shuttle.dev/frameworks/axum)
