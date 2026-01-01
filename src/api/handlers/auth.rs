//! Authentication handlers

use crate::api::AppState;
use crate::utils::AppError;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to authenticate with an API key
#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthRequest {
    /// The API key to authenticate with
    #[schema(example = "mcp_a1b2c3d4e5f6")]
    pub api_key: String,
}

/// Authentication response with JWT token
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// JWT access token
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,
    /// Token validity in seconds
    #[schema(example = 900)]
    pub expires_in: u64,
}

/// Authenticate with API key and get JWT token
#[utoipa::path(
    post,
    path = "/api/v1/auth/token",
    tag = "auth",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "JWT token generated successfully", body = AuthResponse),
        (status = 401, description = "Invalid API key")
    )
)]
pub async fn authenticate(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let token = state
        .auth
        .authenticate_with_api_key(&payload.api_key)
        .await?;

    Ok(Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: state.auth.token_duration_seconds(),
    }))
}
