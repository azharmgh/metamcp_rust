//! MCP Server repository for database operations

use crate::db::models::{McpServer, CreateMcpServerRequest, UpdateMcpServerRequest};
use crate::utils::AppResult;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for MCP server database operations
#[derive(Clone)]
pub struct McpServerRepository {
    pool: PgPool,
}

impl McpServerRepository {
    /// Create a new MCP server repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new MCP server configuration
    pub async fn create(&self, request: &CreateMcpServerRequest) -> AppResult<McpServer> {
        let protocol = if request.protocol.is_empty() {
            "http"
        } else {
            &request.protocol
        };

        let args_json = request.args.as_ref().map(|a| serde_json::json!(a));
        let env_json = request.env.as_ref().map(|e| serde_json::json!(e));

        let server = sqlx::query_as::<_, McpServer>(
            r#"
            INSERT INTO mcp_servers (name, url, protocol, command, args, env, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(&request.name)
        .bind(&request.url)
        .bind(protocol)
        .bind(&request.command)
        .bind(args_json)
        .bind(env_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(server)
    }

    /// Find an MCP server by ID
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<McpServer>> {
        let server = sqlx::query_as::<_, McpServer>("SELECT * FROM mcp_servers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(server)
    }

    /// Find an MCP server by name
    pub async fn find_by_name(&self, name: &str) -> AppResult<Option<McpServer>> {
        let server = sqlx::query_as::<_, McpServer>(
            "SELECT * FROM mcp_servers WHERE name = $1 AND is_active = true",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(server)
    }

    /// List all MCP servers
    pub async fn list_all(&self, include_inactive: bool) -> AppResult<Vec<McpServer>> {
        let servers = if include_inactive {
            sqlx::query_as::<_, McpServer>("SELECT * FROM mcp_servers ORDER BY name")
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, McpServer>(
                "SELECT * FROM mcp_servers WHERE is_active = true ORDER BY name",
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(servers)
    }

    /// Update an MCP server configuration
    pub async fn update(&self, id: Uuid, request: &UpdateMcpServerRequest) -> AppResult<Option<McpServer>> {
        // Build dynamic update query
        let mut updates = Vec::new();
        let mut param_count = 1;

        if request.name.is_some() {
            updates.push(format!("name = ${}", param_count));
            param_count += 1;
        }
        if request.url.is_some() {
            updates.push(format!("url = ${}", param_count));
            param_count += 1;
        }
        if request.protocol.is_some() {
            updates.push(format!("protocol = ${}", param_count));
            param_count += 1;
        }
        if request.command.is_some() {
            updates.push(format!("command = ${}", param_count));
            param_count += 1;
        }
        if request.args.is_some() {
            updates.push(format!("args = ${}", param_count));
            param_count += 1;
        }
        if request.env.is_some() {
            updates.push(format!("env = ${}", param_count));
            param_count += 1;
        }
        if request.is_active.is_some() {
            updates.push(format!("is_active = ${}", param_count));
            param_count += 1;
        }

        if updates.is_empty() {
            return self.find_by_id(id).await;
        }

        updates.push("updated_at = NOW()".to_string());

        let query = format!(
            "UPDATE mcp_servers SET {} WHERE id = ${} RETURNING *",
            updates.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query_as::<_, McpServer>(&query);

        if let Some(ref name) = request.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(ref url) = request.url {
            query_builder = query_builder.bind(url);
        }
        if let Some(ref protocol) = request.protocol {
            query_builder = query_builder.bind(protocol);
        }
        if let Some(ref command) = request.command {
            query_builder = query_builder.bind(command);
        }
        if let Some(ref args) = request.args {
            query_builder = query_builder.bind(serde_json::json!(args));
        }
        if let Some(ref env) = request.env {
            query_builder = query_builder.bind(serde_json::json!(env));
        }
        if let Some(is_active) = request.is_active {
            query_builder = query_builder.bind(is_active);
        }

        query_builder = query_builder.bind(id);

        let server = query_builder.fetch_optional(&self.pool).await?;

        Ok(server)
    }

    /// Delete an MCP server configuration
    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM mcp_servers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Set MCP server as inactive
    pub async fn set_inactive(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE mcp_servers SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Set MCP server as active
    pub async fn set_active(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE mcp_servers SET is_active = true, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
