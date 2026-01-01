//! Integration tests for authentication flow
//!
//! These tests require a running database. They test the complete
//! authentication flow from API key creation to JWT token validation.

use metamcp::auth::{AuthService, JwtService, ApiKeyEncryption};
use metamcp::db::Database;
use std::sync::Arc;

/// Test helper to create a test database connection
async fn setup_test_db() -> Option<Database> {
    // Try to connect to test database
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("TEST_DATABASE_URL"))
        .ok()?;

    Database::new(&database_url).await.ok()
}

/// Test helper to create auth service
fn create_auth_service(db: Database) -> Arc<AuthService> {
    let jwt_secret = "test_jwt_secret_for_integration_tests_12345".to_string();
    let encryption_key = [0u8; 32]; // Test key

    Arc::new(AuthService::new(jwt_secret, &encryption_key, db))
}

#[tokio::test]
async fn test_full_auth_flow() {
    let db = match setup_test_db().await {
        Some(db) => db,
        None => {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }
    };

    // Run migrations
    if db.run_migrations().await.is_err() {
        eprintln!("Skipping test: Failed to run migrations");
        return;
    }

    let auth = create_auth_service(db.clone());

    // Step 1: Generate API key
    let (raw_key, stored_key) = auth
        .generate_api_key("Integration Test Key".to_string())
        .await
        .expect("Failed to generate API key");

    assert!(raw_key.starts_with("mcp_"));
    assert!(!stored_key.id.is_nil());
    assert_eq!(stored_key.name, "Integration Test Key");
    assert!(stored_key.is_active);

    // Step 2: Authenticate with API key
    let token = auth
        .authenticate_with_api_key(&raw_key)
        .await
        .expect("Failed to authenticate");

    assert!(!token.is_empty());

    // Step 3: Validate JWT token
    let claims = auth
        .validate_token(&token)
        .await
        .expect("Failed to validate token");

    assert_eq!(claims.sub, stored_key.id.to_string());

    // Step 4: Verify last_used_at was updated
    let updated_key = db
        .api_keys()
        .find_by_id(stored_key.id)
        .await
        .expect("Failed to fetch key")
        .expect("Key not found");

    assert!(updated_key.last_used_at.is_some());

    // Cleanup: Delete the test key
    db.api_keys()
        .delete(stored_key.id)
        .await
        .expect("Failed to delete test key");
}

#[tokio::test]
async fn test_invalid_api_key_authentication() {
    let db = match setup_test_db().await {
        Some(db) => db,
        None => {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }
    };

    if db.run_migrations().await.is_err() {
        eprintln!("Skipping test: Failed to run migrations");
        return;
    }

    let auth = create_auth_service(db);

    // Try to authenticate with invalid API key
    let result = auth.authenticate_with_api_key("mcp_invalid_key_12345").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_revoked_api_key_authentication() {
    let db = match setup_test_db().await {
        Some(db) => db,
        None => {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }
    };

    if db.run_migrations().await.is_err() {
        eprintln!("Skipping test: Failed to run migrations");
        return;
    }

    let auth = create_auth_service(db.clone());

    // Generate API key
    let (raw_key, stored_key) = auth
        .generate_api_key("Revoke Test Key".to_string())
        .await
        .expect("Failed to generate API key");

    // Revoke the key
    db.api_keys()
        .set_inactive(stored_key.id)
        .await
        .expect("Failed to revoke key");

    // Try to authenticate with revoked key
    let result = auth.authenticate_with_api_key(&raw_key).await;
    assert!(result.is_err());

    // Cleanup
    db.api_keys()
        .delete(stored_key.id)
        .await
        .expect("Failed to delete test key");
}

#[tokio::test]
async fn test_jwt_token_for_revoked_key() {
    let db = match setup_test_db().await {
        Some(db) => db,
        None => {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }
    };

    if db.run_migrations().await.is_err() {
        eprintln!("Skipping test: Failed to run migrations");
        return;
    }

    let auth = create_auth_service(db.clone());

    // Generate API key and get token
    let (raw_key, stored_key) = auth
        .generate_api_key("Token Revoke Test".to_string())
        .await
        .expect("Failed to generate API key");

    let token = auth
        .authenticate_with_api_key(&raw_key)
        .await
        .expect("Failed to authenticate");

    // Token should be valid initially
    let result = auth.validate_token(&token).await;
    assert!(result.is_ok());

    // Revoke the key
    db.api_keys()
        .set_inactive(stored_key.id)
        .await
        .expect("Failed to revoke key");

    // Token should now be invalid (key is revoked)
    let result = auth.validate_token(&token).await;
    assert!(result.is_err());

    // Cleanup
    db.api_keys()
        .delete(stored_key.id)
        .await
        .expect("Failed to delete test key");
}

#[tokio::test]
async fn test_api_key_rotation() {
    let db = match setup_test_db().await {
        Some(db) => db,
        None => {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }
    };

    if db.run_migrations().await.is_err() {
        eprintln!("Skipping test: Failed to run migrations");
        return;
    }

    let auth = create_auth_service(db.clone());

    // Generate first API key
    let (old_key, old_stored) = auth
        .generate_api_key("Rotation Test".to_string())
        .await
        .expect("Failed to generate API key");

    // Verify old key works
    let old_token = auth
        .authenticate_with_api_key(&old_key)
        .await
        .expect("Old key should work");
    assert!(!old_token.is_empty());

    // "Rotate" by creating new key and inactivating old
    let (new_key, new_stored) = auth
        .generate_api_key("Rotation Test (rotated)".to_string())
        .await
        .expect("Failed to generate new API key");

    db.api_keys()
        .set_inactive(old_stored.id)
        .await
        .expect("Failed to inactivate old key");

    // Old key should not work
    let result = auth.authenticate_with_api_key(&old_key).await;
    assert!(result.is_err());

    // New key should work
    let new_token = auth
        .authenticate_with_api_key(&new_key)
        .await
        .expect("New key should work");
    assert!(!new_token.is_empty());

    // Cleanup
    db.api_keys()
        .delete(old_stored.id)
        .await
        .expect("Failed to delete old key");
    db.api_keys()
        .delete(new_stored.id)
        .await
        .expect("Failed to delete new key");
}

#[tokio::test]
async fn test_multiple_api_keys() {
    let db = match setup_test_db().await {
        Some(db) => db,
        None => {
            eprintln!("Skipping test: DATABASE_URL not set");
            return;
        }
    };

    if db.run_migrations().await.is_err() {
        eprintln!("Skipping test: Failed to run migrations");
        return;
    }

    let auth = create_auth_service(db.clone());

    // Generate multiple API keys
    let (key1, stored1) = auth
        .generate_api_key("Multi Test 1".to_string())
        .await
        .expect("Failed to generate key 1");
    let (key2, stored2) = auth
        .generate_api_key("Multi Test 2".to_string())
        .await
        .expect("Failed to generate key 2");
    let (key3, stored3) = auth
        .generate_api_key("Multi Test 3".to_string())
        .await
        .expect("Failed to generate key 3");

    // All keys are different
    assert_ne!(key1, key2);
    assert_ne!(key2, key3);
    assert_ne!(stored1.id, stored2.id);
    assert_ne!(stored2.id, stored3.id);

    // All keys work
    let token1 = auth.authenticate_with_api_key(&key1).await.expect("Key 1 failed");
    let token2 = auth.authenticate_with_api_key(&key2).await.expect("Key 2 failed");
    let token3 = auth.authenticate_with_api_key(&key3).await.expect("Key 3 failed");

    // All tokens are different
    assert_ne!(token1, token2);
    assert_ne!(token2, token3);

    // Cleanup
    db.api_keys().delete(stored1.id).await.expect("Failed to delete key 1");
    db.api_keys().delete(stored2.id).await.expect("Failed to delete key 2");
    db.api_keys().delete(stored3.id).await.expect("Failed to delete key 3");
}
