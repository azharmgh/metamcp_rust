//! Unit tests for authentication module

use metamcp::auth::{Claims, JwtService};

#[test]
fn test_jwt_service_creation() {
    let service = JwtService::new("test_secret");
    assert!(service.token_duration_seconds() > 0);
}

#[test]
fn test_jwt_token_generation_and_validation() {
    let service = JwtService::new("test_secret_key_12345");
    let api_key_id = uuid::Uuid::new_v4();

    // Generate token
    let token = service.generate_token(api_key_id).expect("Failed to generate token");
    assert!(!token.is_empty());

    // Validate token
    let claims = service.validate_token(&token).expect("Failed to validate token");
    assert_eq!(claims.sub, api_key_id.to_string());
}

#[test]
fn test_jwt_token_invalid_signature() {
    let service1 = JwtService::new("secret1");
    let service2 = JwtService::new("secret2");
    let api_key_id = uuid::Uuid::new_v4();

    // Generate token with service1
    let token = service1.generate_token(api_key_id).expect("Failed to generate token");

    // Try to validate with service2 (different secret)
    let result = service2.validate_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_jwt_token_malformed() {
    let service = JwtService::new("test_secret");

    // Try to validate malformed token
    let result = service.validate_token("not.a.valid.token");
    assert!(result.is_err());
}

#[test]
fn test_claims_api_key_id() {
    let service = JwtService::new("test_secret");
    let api_key_id = uuid::Uuid::new_v4();

    let token = service.generate_token(api_key_id).expect("Failed to generate token");
    let claims = service.validate_token(&token).expect("Failed to validate token");

    let parsed_id: uuid::Uuid = claims.sub.parse().expect("Failed to parse API key ID");
    assert_eq!(parsed_id, api_key_id);
}

#[cfg(test)]
mod api_key_encryption_tests {
    use metamcp::auth::ApiKeyEncryption;

    fn create_test_encryption() -> ApiKeyEncryption {
        let key = [0u8; 32]; // Test key (all zeros - don't use in production!)
        ApiKeyEncryption::new(&key)
    }

    #[test]
    fn test_generate_api_key() {
        let key = ApiKeyEncryption::generate_api_key();

        assert!(key.starts_with("mcp_"));
        assert!(key.len() > 10);
    }

    #[test]
    fn test_hash_api_key() {
        let key = "mcp_test_key_12345";

        let hash = ApiKeyEncryption::hash_api_key(key).unwrap();
        assert!(!hash.is_empty());

        // Verify the key against the hash
        let verified = ApiKeyEncryption::verify_api_key(key, &hash).unwrap();
        assert!(verified);

        // Different key should not verify
        let wrong_verified = ApiKeyEncryption::verify_api_key("mcp_different_key", &hash).unwrap();
        assert!(!wrong_verified);
    }

    #[test]
    fn test_encrypt_decrypt_api_key() {
        let encryption = create_test_encryption();
        let original_key = "mcp_test_secret_key_12345";

        // Encrypt
        let encrypted = encryption
            .encrypt(original_key)
            .expect("Failed to encrypt");
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, original_key.as_bytes());

        // Decrypt
        let decrypted = encryption
            .decrypt(&encrypted)
            .expect("Failed to decrypt");
        assert_eq!(decrypted, original_key);
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let encryption = create_test_encryption();

        // Try to decrypt invalid data
        let result = encryption.decrypt(&[1, 2, 3, 4, 5]);
        assert!(result.is_err());
    }
}
