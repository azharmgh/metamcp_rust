//! API Key repository for database operations

use crate::db::models::ApiKey;
use crate::utils::AppResult;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for API key database operations
#[derive(Clone)]
pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    /// Create a new API key repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new API key
    pub async fn create(
        &self,
        name: &str,
        key_hash: &str,
        encrypted_key: Vec<u8>,
    ) -> AppResult<ApiKey> {
        let api_key = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (name, key_hash, encrypted_key, is_active, created_at)
            VALUES ($1, $2, $3, true, NOW())
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(key_hash)
        .bind(encrypted_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(api_key)
    }

    /// Find an API key by its hash
    pub async fn find_by_key_hash(&self, key_hash: &str) -> AppResult<Option<ApiKey>> {
        let api_key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(api_key)
    }

    /// Find an API key by ID
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<ApiKey>> {
        let api_key = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(api_key)
    }

    /// List all API keys
    pub async fn list_all(&self, include_inactive: bool) -> AppResult<Vec<ApiKey>> {
        let keys = if include_inactive {
            sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, ApiKey>(
                "SELECT * FROM api_keys WHERE is_active = true ORDER BY created_at DESC",
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(keys)
    }

    /// Update last used timestamp
    pub async fn update_last_used(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Set API key as inactive
    pub async fn set_inactive(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Set API key as active
    pub async fn set_active(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE api_keys SET is_active = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete an API key permanently
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
