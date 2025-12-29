//! JWT token generation and validation

use crate::utils::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (API key ID)
    pub sub: String,
    /// Expiry timestamp
    pub exp: usize,
    /// Issued at timestamp
    pub iat: usize,
    /// JWT ID for tracking
    pub jti: String,
}

/// JWT service for token operations
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    /// Token validity duration in minutes
    token_duration_minutes: i64,
}

impl JwtService {
    /// Create a new JWT service
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            token_duration_minutes: 15,
        }
    }

    /// Create with custom token duration
    pub fn with_duration(secret: &str, duration_minutes: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            token_duration_minutes: duration_minutes,
        }
    }

    /// Generate a JWT token for an API key
    pub fn generate_token(&self, api_key_id: Uuid) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(self.token_duration_minutes);

        let claims = Claims {
            sub: api_key_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))
    }

    /// Validate a JWT token and return claims
    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let validation = Validation::default();

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;

        Ok(token_data.claims)
    }

    /// Get the token duration in seconds
    pub fn token_duration_seconds(&self) -> u64 {
        (self.token_duration_minutes * 60) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_token() {
        let service = JwtService::new("test_secret");
        let api_key_id = Uuid::new_v4();

        let token = service.generate_token(api_key_id).unwrap();
        let claims = service.validate_token(&token).unwrap();

        assert_eq!(claims.sub, api_key_id.to_string());
    }

    #[test]
    fn test_invalid_token() {
        let service = JwtService::new("test_secret");
        let result = service.validate_token("invalid_token");
        assert!(result.is_err());
    }
}
