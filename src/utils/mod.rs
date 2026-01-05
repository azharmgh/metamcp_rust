//! Utility modules

pub mod error;
pub mod security;

pub use error::{AppError, AppResult, ErrorResponse};
pub use security::{validate_url_for_ssrf, UrlValidationError};
