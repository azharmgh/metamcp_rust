//! Application error types and handling
//!
//! This module provides centralized error handling for the MetaMCP API,
//! including security-related errors for OWASP compliance.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

use super::security::UrlValidationError;

/// Application-level errors
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Process error: {0}")]
    Process(String),

    /// OWASP API7:2023 - Server Side Request Forgery (SSRF)
    /// Security violation errors for blocked URLs and other security issues
    #[error("Security violation: {0}")]
    SecurityViolation(String),
}

/// Convert URL validation errors to AppError
/// OWASP API7:2023 - SSRF Prevention
impl From<UrlValidationError> for AppError {
    fn from(err: UrlValidationError) -> Self {
        // Log security violations for monitoring
        tracing::warn!("OWASP API7:2023 - SSRF attempt blocked: {}", err);
        AppError::SecurityViolation(err.to_string())
    }
}

/// Error response body
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match &self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "Unauthorized", Some(msg.clone())),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "Forbidden", Some(msg.clone())),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "Not Found", Some(msg.clone())),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "Bad Request", Some(msg.clone())),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "Conflict", Some(msg.clone())),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error", None)
            }
            AppError::Database(err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error", None)
            }
            AppError::Jwt(err) => {
                tracing::warn!("JWT error: {}", err);
                (StatusCode::UNAUTHORIZED, "Invalid Token", Some(err.to_string()))
            }
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "Validation Error", Some(msg.clone())),
            AppError::Config(msg) => {
                tracing::error!("Configuration error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration Error", None)
            }
            AppError::McpProtocol(msg) => (StatusCode::BAD_REQUEST, "MCP Protocol Error", Some(msg.clone())),
            AppError::Process(msg) => {
                tracing::error!("Process error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Process Error", None)
            }
            // OWASP API7:2023 - Security violations return 422 Unprocessable Entity
            // to indicate the request was understood but cannot be processed for security reasons
            AppError::SecurityViolation(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Security Violation", Some(msg.clone()))
            }
        };

        let body = ErrorResponse {
            error: error_message.to_string(),
            status: status.as_u16(),
            details,
        };

        (status, Json(body)).into_response()
    }
}

/// Result type alias for application operations
pub type AppResult<T> = Result<T, AppError>;
