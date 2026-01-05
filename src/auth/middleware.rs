//! Authentication middleware

use crate::auth::{AuthService, Claims};
use crate::utils::AppError;
use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::sync::Arc;

/// JSON error response for authentication failures
fn auth_error_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "unauthorized",
            "error_description": message
        })),
    )
        .into_response()
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(request: &Request<Body>) -> Option<&str> {
    request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

/// Authentication middleware
pub async fn auth_middleware(
    State(auth): State<Arc<AuthService>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let token = match extract_bearer_token(&request) {
        Some(t) => t,
        None => {
            tracing::warn!("Missing authorization header");
            return auth_error_response("Missing or invalid Authorization header");
        }
    };

    match auth.validate_token(token).await {
        Ok(claims) => {
            // Attach claims to request extensions for handlers to use
            request.extensions_mut().insert(claims);
            next.run(request).await
        }
        Err(e) => {
            tracing::warn!("Authentication failed: {}", e);
            auth_error_response("Invalid or expired token")
        }
    }
}

/// Extract claims from request extensions
pub fn get_claims(request: &Request<Body>) -> Option<&Claims> {
    request.extensions().get::<Claims>()
}

/// Extractor for claims in handlers
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub claims: Claims,
}

impl AuthenticatedUser {
    /// Get the API key ID from claims
    pub fn api_key_id(&self) -> Result<uuid::Uuid, AppError> {
        uuid::Uuid::parse_str(&self.claims.sub)
            .map_err(|_| AppError::Internal("Invalid API key ID in token".to_string()))
    }
}

/// Axum extractor for authenticated user
impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(|claims| AuthenticatedUser { claims })
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))
    }
}
