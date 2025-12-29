//! Application settings and configuration

use crate::utils::AppError;
use std::env;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,

    /// JWT secret for token signing
    pub jwt_secret: String,

    /// Encryption key for API keys at rest (32 bytes)
    pub encryption_key: [u8; 32],

    /// Server host address
    pub server_host: String,

    /// Server port
    pub server_port: u16,

    /// Log level
    pub log_level: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("DATABASE_URL is required".to_string()))?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| AppError::Config("JWT_SECRET is required".to_string()))?;

        let encryption_key_hex = env::var("ENCRYPTION_KEY")
            .map_err(|_| AppError::Config("ENCRYPTION_KEY is required".to_string()))?;

        let encryption_key_bytes = hex::decode(&encryption_key_hex)
            .map_err(|_| AppError::Config("ENCRYPTION_KEY must be valid hex".to_string()))?;

        let encryption_key: [u8; 32] = encryption_key_bytes
            .try_into()
            .map_err(|_| AppError::Config("ENCRYPTION_KEY must be 32 bytes (64 hex chars)".to_string()))?;

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "12009".to_string())
            .parse::<u16>()
            .map_err(|_| AppError::Config("SERVER_PORT must be a valid port number".to_string()))?;

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info,metamcp=debug".to_string());

        Ok(Self {
            database_url,
            jwt_secret,
            encryption_key,
            server_host,
            server_port,
            log_level,
        })
    }

    /// Get the server bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
