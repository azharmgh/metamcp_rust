//! Database module

pub mod models;
pub mod repositories;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub use models::{
    ApiKey, ApiKeyInfo, CreateApiKeyRequest, CreateMcpServerRequest, McpProtocol, McpServer,
    McpServerInfo, UpdateMcpServerRequest,
};
pub use repositories::{ApiKeyRepository, McpServerRepository};

/// Database connection wrapper
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(100)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Create from an existing pool (useful for Shuttle)
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get API key repository
    pub fn api_keys(&self) -> ApiKeyRepository {
        ApiKeyRepository::new(self.pool.clone())
    }

    /// Get MCP server repository
    pub fn mcp_servers(&self) -> McpServerRepository {
        McpServerRepository::new(self.pool.clone())
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }
}
