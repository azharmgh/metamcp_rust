//! API middleware
//!
//! This module provides middleware for the MetaMCP API including:
//! - Authentication middleware
//! - Security headers middleware (OWASP API8:2023)
//! - Rate limiting headers (OWASP API4:2023)

pub mod security;

// Re-export auth middleware from auth module
pub use crate::auth::auth_middleware;

// Re-export security middleware
pub use security::{rate_limit_headers, security_headers};
