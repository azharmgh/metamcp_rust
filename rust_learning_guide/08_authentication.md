# Authentication (jsonwebtoken, argon2, chacha20poly1305)

This guide covers three security-focused crates used for authentication in Rust applications.

## Overview

| Crate | Purpose |
|-------|---------|
| **jsonwebtoken** | Create and validate JWT tokens |
| **argon2** | Secure password hashing |
| **chacha20poly1305** | Symmetric encryption |

## Installation

```toml
[dependencies]
jsonwebtoken = { version = "10", features = ["rust_crypto"] }
argon2 = "0.5"
chacha20poly1305 = "0.10"
```

---

## jsonwebtoken - JWT Tokens

JSON Web Tokens (JWT) are a standard for securely transmitting information between parties as a JSON object.

### How JWTs Work

A JWT consists of three parts:
1. **Header** - Algorithm and token type
2. **Payload** - Claims (data)
3. **Signature** - Verification signature

```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.     <- Header
eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6...   <- Payload
SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c  <- Signature
```

### Real Example from MetaMCP

From `src/auth/jwt.rs`:

```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // Subject (usually user ID)
    pub exp: usize,     // Expiration time (Unix timestamp)
    pub iat: usize,     // Issued at
    pub jti: String,    // JWT ID (unique identifier)
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    token_duration_minutes: i64,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            token_duration_minutes: 15,
        }
    }

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
            .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let validation = Validation::default();

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;

        Ok(token_data.claims)
    }
}
```

[Run JWT example in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20Claims%20%7B%0A%20%20%20%20sub%3A%20String%2C%20%20%20%20%2F%2F%20Subject%0A%20%20%20%20exp%3A%20usize%2C%20%20%20%20%2F%2F%20Expiration%0A%20%20%20%20iat%3A%20usize%2C%20%20%20%20%2F%2F%20Issued%20at%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Create%20claims%0A%20%20%20%20let%20claims%20%3D%20Claims%20%7B%0A%20%20%20%20%20%20%20%20sub%3A%20%22user_123%22.to_string()%2C%0A%20%20%20%20%20%20%20%20exp%3A%201735689600%2C%20%20%2F%2F%20Future%20timestamp%0A%20%20%20%20%20%20%20%20iat%3A%201704067200%2C%20%20%2F%2F%20Now%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Serialize%20to%20JSON%20(payload%20part%20of%20JWT)%0A%20%20%20%20let%20json%20%3D%20serde_json%3A%3Ato_string(%26claims).unwrap()%3B%0A%20%20%20%20println!(%22Claims%20JSON%3A%20%7B%7D%22%2C%20json)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Base64%20encode%20(simplified%20-%20real%20JWT%20uses%20base64url)%0A%20%20%20%20use%20std%3A%3Acollections%3A%3Ahash_map%3A%3ADefaultHasher%3B%0A%20%20%20%20use%20std%3A%3Ahash%3A%3A%7BHash%2C%20Hasher%7D%3B%0A%20%20%20%20%0A%20%20%20%20let%20mut%20hasher%20%3D%20DefaultHasher%3A%3Anew()%3B%0A%20%20%20%20json.hash(%26mut%20hasher)%3B%0A%20%20%20%20let%20signature%20%3D%20hasher.finish()%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Simulated%20signature%3A%20%7B%3Ax%7D%22%2C%20signature)%3B%0A%7D)

### Common JWT Operations

```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};

// Generate token with custom header
let header = Header::new(Algorithm::HS512);
let token = encode(&header, &claims, &encoding_key)?;

// Validate with custom options
let mut validation = Validation::new(Algorithm::HS256);
validation.leeway = 60; // Allow 60 seconds clock skew
validation.validate_exp = true;

let token_data = decode::<Claims>(&token, &decoding_key, &validation)?;

// Access claims
println!("User ID: {}", token_data.claims.sub);
println!("Expires: {}", token_data.claims.exp);
```

---

## argon2 - Password Hashing

Argon2 is a secure password hashing algorithm that won the Password Hashing Competition. It's designed to be memory-hard, making brute-force attacks expensive.

### Why Argon2?

- **Memory-hard** - Requires significant memory, preventing GPU attacks
- **Configurable** - Adjust time/memory/parallelism trade-offs
- **Modern** - Designed for current security threats

### Real Example from MetaMCP

From `src/auth/api_key.rs`:

```rust
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

/// Hash an API key for secure storage
pub fn hash_api_key(api_key: &str) -> Result<String, AppError> {
    let argon2 = Argon2::default();

    // Generate a random salt
    let salt = SaltString::generate(&mut OsRng);

    // Hash the API key
    let hash = argon2
        .hash_password(api_key.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash API key: {}", e)))?
        .to_string();

    Ok(hash)
}

/// Verify an API key against a stored hash
pub fn verify_api_key(api_key: &str, hash: &str) -> Result<bool, AppError> {
    // Parse the stored hash
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid hash format: {}", e)))?;

    // Verify the API key
    Ok(Argon2::default()
        .verify_password(api_key.as_bytes(), &parsed_hash)
        .is_ok())
}
```

[Run Argon2 conceptual example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=%2F%2F%20Argon2%20conceptual%20example%0A%2F%2F%20Real%20argon2%20crate%20requires%20crypto%20features%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20password%20%3D%20%22my_secure_password%22%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Simulate%20hashing%20(real%20argon2%20is%20much%20more%20complex)%0A%20%20%20%20let%20salt%20%3D%20%22random_salt_123%22%3B%0A%20%20%20%20let%20hash%20%3D%20format!(%22%24argon2id%24v%3D19%24m%3D4096%2Ct%3D3%2Cp%3D1%24%7B%7D%24simulated_hash%22%2C%20salt)%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Password%3A%20%7B%7D%22%2C%20password)%3B%0A%20%20%20%20println!(%22Hash%20format%3A%20%7B%7D%22%2C%20hash)%3B%0A%20%20%20%20println!()%3B%0A%20%20%20%20println!(%22Hash%20components%3A%22)%3B%0A%20%20%20%20println!(%22%20%20Algorithm%3A%20argon2id%22)%3B%0A%20%20%20%20println!(%22%20%20Version%3A%2019%22)%3B%0A%20%20%20%20println!(%22%20%20Memory%3A%204096%20KB%22)%3B%0A%20%20%20%20println!(%22%20%20Iterations%3A%203%22)%3B%0A%20%20%20%20println!(%22%20%20Parallelism%3A%201%22)%3B%0A%7D)

### Argon2 Configuration

```rust
use argon2::{Argon2, Algorithm, Version, Params};

// Custom configuration for high-security applications
let params = Params::new(
    65536,  // Memory cost (64 MB)
    3,      // Time cost (iterations)
    4,      // Parallelism
    None,   // Output length (default 32)
).unwrap();

let argon2 = Argon2::new(
    Algorithm::Argon2id,  // Hybrid algorithm (recommended)
    Version::V0x13,       // Version 1.3
    params,
);
```

---

## chacha20poly1305 - Encryption

ChaCha20-Poly1305 is an authenticated encryption algorithm that provides both confidentiality and integrity.

### Why ChaCha20-Poly1305?

- **Fast** - Efficient on systems without AES hardware
- **Secure** - Used by TLS 1.3, WireGuard, and more
- **Authenticated** - Detects tampering automatically

### Real Example from MetaMCP

From `src/auth/api_key.rs`:

```rust
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use argon2::password_hash::rand_core::{OsRng, RngCore};

pub struct ApiKeyEncryption {
    cipher: ChaCha20Poly1305,
}

impl ApiKeyEncryption {
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new(key.into()),
        }
    }

    /// Encrypt an API key
    pub fn encrypt(&self, api_key: &str) -> Result<Vec<u8>, AppError> {
        // Generate a random 12-byte nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the API key
        let ciphertext = self
            .cipher
            .encrypt(nonce, api_key.as_bytes())
            .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext for storage
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    /// Decrypt an API key
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<String, AppError> {
        // Ensure we have at least nonce + some data
        if encrypted.len() < 12 {
            return Err(AppError::Internal("Invalid encrypted data".to_string()));
        }

        // Extract nonce (first 12 bytes) and ciphertext
        let nonce = Nonce::from_slice(&encrypted[..12]);
        let ciphertext = &encrypted[12..];

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;

        // Convert to string
        String::from_utf8(plaintext)
            .map_err(|e| AppError::Internal(format!("Invalid UTF-8: {}", e)))
    }
}
```

[Run encryption concept example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=%2F%2F%20Encryption%20concept%20demonstration%0A%2F%2F%20Real%20chacha20poly1305%20requires%20crypto%20features%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20plaintext%20%3D%20%22mcp_a1b2c3d4e5f6%22%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Simulated%20encryption%20components%0A%20%20%20%20let%20key%20%3D%20%5B0u8%3B%2032%5D%3B%20%20%20%20%20%20%20%20%20%20%20%20%2F%2F%2032-byte%20key%0A%20%20%20%20let%20nonce%20%3D%20%5B1u8%3B%2012%5D%3B%20%20%20%20%20%20%20%20%20%20%20%2F%2F%2012-byte%20nonce%0A%20%20%20%20%0A%20%20%20%20println!(%22Original%3A%20%7B%7D%22%2C%20plaintext)%3B%0A%20%20%20%20println!(%22Key%20size%3A%20%7B%7D%20bytes%20(256%20bits)%22%2C%20key.len())%3B%0A%20%20%20%20println!(%22Nonce%20size%3A%20%7B%7D%20bytes%20(96%20bits)%22%2C%20nonce.len())%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Simulated%20result%20format%0A%20%20%20%20println!()%3B%0A%20%20%20%20println!(%22Encrypted%20format%3A%20%5Bnonce%5D%5Bciphertext%5D%5Btag%5D%22)%3B%0A%20%20%20%20println!(%22%20%20Nonce%3A%2012%20bytes%20(stored%20with%20ciphertext)%22)%3B%0A%20%20%20%20println!(%22%20%20Ciphertext%3A%20same%20length%20as%20plaintext%22)%3B%0A%20%20%20%20println!(%22%20%20Tag%3A%2016%20bytes%20(authentication)%22)%3B%0A%7D)

### Generating Encryption Keys

```rust
use hex;

// Generate a secure 32-byte key
fn generate_encryption_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

// Convert to hex for storage in environment variable
let key = generate_encryption_key();
let hex_key = hex::encode(key);
println!("ENCRYPTION_KEY={}", hex_key);
```

From `src/config/settings.rs` - loading key from environment:

```rust
let encryption_key_hex = env::var("ENCRYPTION_KEY")
    .map_err(|_| AppError::Config("ENCRYPTION_KEY is required".to_string()))?;

let encryption_key_bytes = hex::decode(&encryption_key_hex)
    .map_err(|_| AppError::Config("ENCRYPTION_KEY must be valid hex".to_string()))?;

let encryption_key: [u8; 32] = encryption_key_bytes
    .try_into()
    .map_err(|_| AppError::Config("ENCRYPTION_KEY must be 32 bytes (64 hex chars)".to_string()))?;
```

## Complete Authentication Flow

Here's how these crates work together in MetaMCP:

```rust
// 1. User provides an API key
let api_key = "mcp_a1b2c3d4e5f6";

// 2. Hash the API key for secure comparison
let key_hash = hash_api_key(api_key)?;

// 3. Encrypt the API key for recovery (optional)
let encrypted_key = encryption.encrypt(api_key)?;

// 4. Store hash and encrypted key in database
db.api_keys().create(
    "My API Key",
    &key_hash,
    encrypted_key,
).await?;

// 5. User authenticates - verify the hash
let stored_key = db.api_keys().find_by_id(key_id).await?;
if verify_api_key(provided_key, &stored_key.key_hash)? {
    // 6. Generate JWT token
    let token = jwt_service.generate_token(stored_key.id)?;
    return Ok(token);
}
```

## API Key Generation

From `src/auth/api_key.rs`:

```rust
pub fn generate_api_key() -> String {
    format!("mcp_{}", Uuid::new_v4().simple())
}

// Result: "mcp_a1b2c3d4e5f67890abcdef1234567890"
```

## Best Practices

### JWT

| Do | Don't |
|----|-------|
| Use short expiration times | Store sensitive data in payload |
| Validate all claims | Use weak secrets |
| Use HTTPS only | Expose tokens in URLs |
| Implement refresh tokens | Ignore expiration |

### Argon2

| Do | Don't |
|----|-------|
| Use default parameters for most cases | Use plain hashes (SHA-256) |
| Store the full hash string | Implement your own hashing |
| Use unique salts (auto-generated) | Reuse salts |

### ChaCha20-Poly1305

| Do | Don't |
|----|-------|
| Generate random nonces | Reuse nonces with same key |
| Store nonce with ciphertext | Use predictable nonces |
| Use secure key generation | Hardcode keys in source |

## Pros and Cons

### jsonwebtoken

| Pros | Cons |
|------|------|
| Stateless authentication | Tokens can't be revoked easily |
| Standard format | Payload is readable (base64) |
| Good library support | Must handle token refresh |

### argon2

| Pros | Cons |
|------|------|
| Very secure | Slower than bcrypt |
| Memory-hard | Higher memory usage |
| Modern standard | More configuration options |

### chacha20poly1305

| Pros | Cons |
|------|------|
| Fast without hardware support | Nonce management required |
| Authenticated encryption | Key must be kept secret |
| Used in TLS 1.3 | Fixed key size (256-bit) |

## When to Use

**JWT (jsonwebtoken):**
- Stateless API authentication
- Single sign-on (SSO)
- Authorization between services

**Argon2:**
- Password storage
- API key storage
- Any secret that needs verification

**ChaCha20-Poly1305:**
- Encrypting sensitive data at rest
- Secure data transfer
- Key recovery scenarios

## Further Learning

### JWT
- [JWT.io](https://jwt.io/) - JWT debugger and info
- [jsonwebtoken docs](https://docs.rs/jsonwebtoken)

### Argon2
- [Argon2 specification](https://github.com/P-H-C/phc-winner-argon2)
- [OWASP Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)

### ChaCha20-Poly1305
- [RFC 8439](https://tools.ietf.org/html/rfc8439)
- [RustCrypto AEAD](https://github.com/RustCrypto/AEADs)

## Related Crates

- **uuid** - Generate unique identifiers
- **hex** - Encode/decode hex strings
- **ring** - Alternative crypto library
- **bcrypt** - Alternative password hashing
