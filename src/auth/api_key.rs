//! API Key generation and encryption

use crate::utils::AppError;
use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use uuid::Uuid;

/// API Key encryption service
pub struct ApiKeyEncryption {
    cipher: ChaCha20Poly1305,
}

impl ApiKeyEncryption {
    /// Create a new encryption service with the given key
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new(key.into()),
        }
    }

    /// Generate a new random API key
    pub fn generate_api_key() -> String {
        format!("mcp_{}", Uuid::new_v4().simple())
    }

    /// Hash an API key for database lookup
    pub fn hash_api_key(api_key: &str) -> Result<String, AppError> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2
            .hash_password(api_key.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Failed to hash API key: {}", e)))?
            .to_string();
        Ok(hash)
    }

    /// Verify an API key against a hash
    pub fn verify_api_key(api_key: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("Invalid hash format: {}", e)))?;

        Ok(Argon2::default()
            .verify_password(api_key.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Encrypt an API key for storage
    pub fn encrypt(&self, api_key: &str) -> Result<Vec<u8>, AppError> {
        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the API key
        let ciphertext = self
            .cipher
            .encrypt(nonce, api_key.as_bytes())
            .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    /// Decrypt an API key from storage
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<String, AppError> {
        if encrypted.len() < 12 {
            return Err(AppError::Internal("Invalid encrypted data".to_string()));
        }

        // Extract nonce and ciphertext
        let nonce = Nonce::from_slice(&encrypted[..12]);
        let ciphertext = &encrypted[12..];

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::Internal(format!("Invalid UTF-8 in decrypted data: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key = ApiKeyEncryption::generate_api_key();
        assert!(key.starts_with("mcp_"));
        assert_eq!(key.len(), 36); // "mcp_" + 32 hex chars
    }

    #[test]
    fn test_hash_and_verify() {
        let api_key = "mcp_test_key_123";
        let hash = ApiKeyEncryption::hash_api_key(api_key).unwrap();
        assert!(ApiKeyEncryption::verify_api_key(api_key, &hash).unwrap());
        assert!(!ApiKeyEncryption::verify_api_key("wrong_key", &hash).unwrap());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32];
        let encryption = ApiKeyEncryption::new(&key);

        let api_key = "mcp_test_key_123";
        let encrypted = encryption.encrypt(api_key).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();

        assert_eq!(api_key, decrypted);
    }
}
