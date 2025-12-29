//! Authentication service

use crate::auth::{ApiKeyEncryption, Claims, JwtService};
use crate::db::Database;
use crate::utils::AppError;
use uuid::Uuid;

/// Authentication service combining API key and JWT functionality
pub struct AuthService {
    jwt_service: JwtService,
    encryption: ApiKeyEncryption,
    db: Database,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(jwt_secret: String, encryption_key: &[u8; 32], db: Database) -> Self {
        Self {
            jwt_service: JwtService::new(&jwt_secret),
            encryption: ApiKeyEncryption::new(encryption_key),
            db,
        }
    }

    /// Generate a new API key
    pub async fn generate_api_key(
        &self,
        name: String,
    ) -> Result<(String, crate::db::models::ApiKey), AppError> {
        // Generate random API key
        let raw_key = ApiKeyEncryption::generate_api_key();

        // Hash for database lookup
        let key_hash = ApiKeyEncryption::hash_api_key(&raw_key)?;

        // Encrypt for storage
        let encrypted_key = self.encryption.encrypt(&raw_key)?;

        // Store in database
        let api_key = self
            .db
            .api_keys()
            .create(&name, &key_hash, encrypted_key)
            .await?;

        Ok((raw_key, api_key))
    }

    /// Authenticate with an API key and return a JWT token
    pub async fn authenticate_with_api_key(&self, api_key: &str) -> Result<String, AppError> {
        // Get all active keys and check against each hash
        // (This is necessary because argon2 hashes include random salt)
        let keys = self.db.api_keys().list_all(false).await?;

        let mut found_key = None;
        for key in keys {
            if ApiKeyEncryption::verify_api_key(api_key, &key.key_hash)? {
                found_key = Some(key);
                break;
            }
        }

        let stored_key = found_key.ok_or_else(|| AppError::Unauthorized("Invalid API key".to_string()))?;

        if !stored_key.is_active {
            return Err(AppError::Unauthorized("API key is inactive".to_string()));
        }

        // Update last used timestamp
        self.db.api_keys().update_last_used(stored_key.id).await?;

        // Generate JWT token
        self.generate_jwt_for_key(stored_key.id)
    }

    /// Generate JWT token for an API key ID
    pub fn generate_jwt_for_key(&self, key_id: Uuid) -> Result<String, AppError> {
        self.jwt_service.generate_token(key_id)
    }

    /// Validate a JWT token
    pub async fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let claims = self.jwt_service.validate_token(token)?;

        // Verify API key is still active
        let key_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;

        let api_key = self
            .db
            .api_keys()
            .find_by_id(key_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("API key not found".to_string()))?;

        if !api_key.is_active {
            return Err(AppError::Unauthorized("API key has been revoked".to_string()));
        }

        Ok(claims)
    }

    /// Revoke an API key
    pub async fn revoke_api_key(&self, key_id: Uuid) -> Result<(), AppError> {
        self.db.api_keys().set_inactive(key_id).await
    }

    /// Get token duration in seconds
    pub fn token_duration_seconds(&self) -> u64 {
        self.jwt_service.token_duration_seconds()
    }

    /// Get database reference (for CLI operations)
    pub fn database(&self) -> &Database {
        &self.db
    }
}
