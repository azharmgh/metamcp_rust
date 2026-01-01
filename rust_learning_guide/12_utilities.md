# Utility Crates (uuid, chrono, dotenvy, hex)

This guide covers four essential utility crates commonly used in Rust applications.

## Overview

| Crate | Purpose |
|-------|---------|
| **uuid** | Generate and parse UUIDs |
| **chrono** | Date and time handling |
| **dotenvy** | Load environment variables from .env files |
| **hex** | Hex encoding/decoding |

---

## uuid - Universally Unique Identifiers

UUIDs are 128-bit identifiers that are globally unique without a central authority.

### Installation

```toml
[dependencies]
uuid = { version = "1", features = ["v4", "serde"] }
```

Features:
- `v4` - Random UUIDs (most common)
- `v7` - Time-ordered UUIDs
- `serde` - Serialization support

### Basic Usage

```rust
use uuid::Uuid;

// Generate a random UUID (v4)
let id = Uuid::new_v4();
println!("New UUID: {}", id);
// Output: 550e8400-e29b-41d4-a716-446655440000

// Parse from string
let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;

// Convert to string
let id_string = id.to_string();
let id_hyphenated = id.hyphenated().to_string();
let id_simple = id.simple().to_string();  // No hyphens
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20uuid%3A%3AUuid%3B%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Generate%20random%20UUID%0A%20%20%20%20let%20id%20%3D%20Uuid%3A%3Anew_v4()%3B%0A%20%20%20%20println!(%22New%20UUID%3A%20%7B%7D%22%2C%20id)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Different%20string%20formats%0A%20%20%20%20println!(%22Hyphenated%3A%20%7B%7D%22%2C%20id.hyphenated())%3B%0A%20%20%20%20println!(%22Simple%3A%20%7B%7D%22%2C%20id.simple())%3B%0A%20%20%20%20println!(%22URN%3A%20%7B%7D%22%2C%20id.urn())%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Parse%20from%20string%0A%20%20%20%20let%20parsed%20%3D%20Uuid%3A%3Aparse_str(%22550e8400-e29b-41d4-a716-446655440000%22).unwrap()%3B%0A%20%20%20%20println!(%22Parsed%3A%20%7B%7D%22%2C%20parsed)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Nil%20UUID%20(all%20zeros)%0A%20%20%20%20let%20nil%20%3D%20Uuid%3A%3Anil()%3B%0A%20%20%20%20println!(%22Nil%3A%20%7B%7D%2C%20is_nil%3A%20%7B%7D%22%2C%20nil%2C%20nil.is_nil())%3B%0A%7D)

### Real Examples from MetaMCP

From `src/auth/api_key.rs` - generating API keys:

```rust
use uuid::Uuid;

pub fn generate_api_key() -> String {
    // Create API key with prefix and UUID
    format!("mcp_{}", Uuid::new_v4().simple())
}
// Result: "mcp_550e8400e29b41d4a716446655440000"
```

From `src/auth/jwt.rs` - JWT ID:

```rust
let claims = Claims {
    sub: api_key_id.to_string(),
    exp: exp.timestamp() as usize,
    iat: now.timestamp() as usize,
    jti: Uuid::new_v4().to_string(),  // Unique JWT identifier
};
```

From `src/streaming/manager.rs` - client IDs:

```rust
let client_id = Uuid::new_v4().to_string();
```

### With Serde

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct User {
    id: Uuid,  // Automatically serializes to/from string
    name: String,
}
```

---

## chrono - Date and Time

Chrono provides comprehensive date and time handling for Rust.

### Installation

```toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
```

### Basic Usage

```rust
use chrono::{DateTime, Utc, Duration, Local, NaiveDateTime};

// Current UTC time
let now = Utc::now();
println!("UTC now: {}", now);

// Current local time
let local_now = Local::now();
println!("Local now: {}", local_now);

// Create specific datetime
let dt = Utc.with_ymd_and_hms(2024, 1, 15, 12, 30, 0).unwrap();

// Add/subtract time
let tomorrow = now + Duration::days(1);
let last_week = now - Duration::weeks(1);

// Format datetime
let formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20chrono%3A%3A%7BUtc%2C%20Duration%2C%20Local%7D%3B%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Current%20time%0A%20%20%20%20let%20now%20%3D%20Utc%3A%3Anow()%3B%0A%20%20%20%20println!(%22UTC%20now%3A%20%7B%7D%22%2C%20now)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Local%20time%0A%20%20%20%20let%20local%20%3D%20Local%3A%3Anow()%3B%0A%20%20%20%20println!(%22Local%3A%20%7B%7D%22%2C%20local)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Time%20arithmetic%0A%20%20%20%20let%20tomorrow%20%3D%20now%20%2B%20Duration%3A%3Adays(1)%3B%0A%20%20%20%20let%20one_hour_ago%20%3D%20now%20-%20Duration%3A%3Ahours(1)%3B%0A%20%20%20%20println!(%22Tomorrow%3A%20%7B%7D%22%2C%20tomorrow)%3B%0A%20%20%20%20println!(%22One%20hour%20ago%3A%20%7B%7D%22%2C%20one_hour_ago)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Formatting%0A%20%20%20%20println!(%22ISO%208601%3A%20%7B%7D%22%2C%20now.to_rfc3339())%3B%0A%20%20%20%20println!(%22Custom%3A%20%7B%7D%22%2C%20now.format(%22%25Y-%25m-%25d%20%25H%3A%25M%3A%25S%22))%3B%0A%20%20%20%20println!(%22Date%20only%3A%20%7B%7D%22%2C%20now.format(%22%25B%20%25d%2C%20%25Y%22))%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Unix%20timestamp%0A%20%20%20%20println!(%22Timestamp%3A%20%7B%7D%22%2C%20now.timestamp())%3B%0A%7D)

### Real Examples from MetaMCP

From `src/auth/jwt.rs` - token expiration:

```rust
use chrono::{Duration, Utc};

pub fn generate_token(&self, api_key_id: Uuid) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now + Duration::minutes(self.token_duration_minutes);

    let claims = Claims {
        sub: api_key_id.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
        jti: Uuid::new_v4().to_string(),
    };

    // ...
}
```

From `src/api/handlers/health.rs` - timestamp in response:

```rust
use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
```

From `src/db/models/api_key.rs` - database timestamps:

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}
```

### Common Operations

```rust
use chrono::{DateTime, Utc, Duration, TimeZone, Datelike, Timelike};

// Parse from string
let dt = DateTime::parse_from_rfc3339("2024-01-15T12:30:00Z")?;
let dt = Utc.datetime_from_str("2024-01-15 12:30:00", "%Y-%m-%d %H:%M:%S")?;

// Access components
let now = Utc::now();
println!("Year: {}", now.year());
println!("Month: {}", now.month());
println!("Day: {}", now.day());
println!("Hour: {}", now.hour());
println!("Minute: {}", now.minute());
println!("Second: {}", now.second());

// Compare times
let is_future = dt > Utc::now();
let is_expired = now > expiration_time;

// Duration between times
let duration = end_time - start_time;
println!("Elapsed: {} seconds", duration.num_seconds());
```

---

## dotenvy - Environment Configuration

Dotenvy loads environment variables from `.env` files (fork of dotenv).

### Installation

```toml
[dependencies]
dotenvy = "0.15"
```

### Basic Usage

```rust
use std::env;

fn main() {
    // Load .env file (optional - won't fail if missing)
    dotenvy::dotenv().ok();

    // Now use environment variables
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string());
}
```

### Real Example from MetaMCP

From `src/config/settings.rs`:

```rust
use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub encryption_key: [u8; 32],
    pub server_host: String,
    pub server_port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        // Load .env file (ignore if missing)
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("DATABASE_URL is required".to_string()))?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| AppError::Config("JWT_SECRET is required".to_string()))?;

        let encryption_key_hex = env::var("ENCRYPTION_KEY")
            .map_err(|_| AppError::Config("ENCRYPTION_KEY is required".to_string()))?;

        let encryption_key_bytes = hex::decode(&encryption_key_hex)
            .map_err(|_| AppError::Config("ENCRYPTION_KEY must be valid hex".to_string()))?;

        let encryption_key: [u8; 32] = encryption_key_bytes
            .try_into()
            .map_err(|_| AppError::Config("ENCRYPTION_KEY must be 32 bytes".to_string()))?;

        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "12009".to_string())
            .parse::<u16>()
            .map_err(|_| AppError::Config("SERVER_PORT must be a valid port".to_string()))?;

        let log_level = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info,metamcp=debug".to_string());

        Ok(Self {
            database_url,
            jwt_secret,
            encryption_key,
            server_host,
            server_port,
            log_level,
        })
    }
}
```

### .env File Example

```bash
# .env
DATABASE_URL=postgres://user:password@localhost/mydb
JWT_SECRET=your-super-secret-key-here
ENCRYPTION_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=info,metamcp=debug
```

[Run dotenvy concept](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20std%3A%3Aenv%3B%0A%0A%2F%2F%20Simulating%20dotenvy%20pattern%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20In%20real%20code%3A%20dotenvy%3A%3Adotenv().ok()%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Required%20variable%0A%20%20%20%20let%20database_url%20%3D%20env%3A%3Avar(%22DATABASE_URL%22)%0A%20%20%20%20%20%20%20%20.unwrap_or_else(%7C_%7C%20%22postgres%3A%2F%2Flocalhost%2Ftest%22.to_string())%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Optional%20with%20default%0A%20%20%20%20let%20port%20%3D%20env%3A%3Avar(%22PORT%22)%0A%20%20%20%20%20%20%20%20.unwrap_or_else(%7C_%7C%20%228080%22.to_string())%0A%20%20%20%20%20%20%20%20.parse%3A%3A%3Cu16%3E()%0A%20%20%20%20%20%20%20%20.expect(%22PORT%20must%20be%20a%20number%22)%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Database%3A%20%7B%7D%22%2C%20database_url)%3B%0A%20%20%20%20println!(%22Port%3A%20%7B%7D%22%2C%20port)%3B%0A%7D)

---

## hex - Hexadecimal Encoding

Hex provides functions to encode bytes to hexadecimal strings and vice versa.

### Installation

```toml
[dependencies]
hex = "0.4"
```

### Basic Usage

```rust
use hex;

// Encode bytes to hex string
let bytes = [0x12, 0x34, 0xAB, 0xCD];
let hex_string = hex::encode(&bytes);
println!("{}", hex_string);  // "1234abcd"

// Decode hex string to bytes
let decoded = hex::decode("1234abcd")?;
assert_eq!(decoded, vec![0x12, 0x34, 0xAB, 0xCD]);

// Upper case encoding
let upper = hex::encode_upper(&bytes);
println!("{}", upper);  // "1234ABCD"
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=fn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Encode%20bytes%20to%20hex%0A%20%20%20%20let%20bytes%20%3D%20%5B0x12%2C%200x34%2C%200xAB%2C%200xCD%5D%3B%0A%20%20%20%20let%20hex_string%20%3D%20hex%3A%3Aencode(%26bytes)%3B%0A%20%20%20%20println!(%22Hex%3A%20%7B%7D%22%2C%20hex_string)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Decode%20hex%20to%20bytes%0A%20%20%20%20let%20decoded%20%3D%20hex%3A%3Adecode(%221234abcd%22).unwrap()%3B%0A%20%20%20%20println!(%22Decoded%3A%20%7B%3A%3F%7D%22%2C%20decoded)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Upper%20case%0A%20%20%20%20let%20upper%20%3D%20hex%3A%3Aencode_upper(%26bytes)%3B%0A%20%20%20%20println!(%22Upper%3A%20%7B%7D%22%2C%20upper)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Generate%2032%20random%20bytes%20and%20encode%0A%20%20%20%20let%20key%20%3D%20%5B0u8%3B%2032%5D%3B%20%2F%2F%20In%20real%20code%2C%20use%20random%20bytes%0A%20%20%20%20let%20key_hex%20%3D%20hex%3A%3Aencode(%26key)%3B%0A%20%20%20%20println!(%22Key%20length%3A%20%7B%7D%20bytes%20%3D%20%7B%7D%20hex%20chars%22%2C%20%0A%20%20%20%20%20%20%20%20key.len()%2C%20key_hex.len())%3B%0A%7D)

### Real Example from MetaMCP

From `src/config/settings.rs` - decoding encryption key:

```rust
let encryption_key_hex = env::var("ENCRYPTION_KEY")
    .map_err(|_| AppError::Config("ENCRYPTION_KEY is required".to_string()))?;

// Decode from hex string to bytes
let encryption_key_bytes = hex::decode(&encryption_key_hex)
    .map_err(|_| AppError::Config("ENCRYPTION_KEY must be valid hex".to_string()))?;

// Convert to fixed-size array
let encryption_key: [u8; 32] = encryption_key_bytes
    .try_into()
    .map_err(|_| AppError::Config(
        "ENCRYPTION_KEY must be 32 bytes (64 hex chars)".to_string()
    ))?;
```

### Generating a Hex Key

```rust
use argon2::password_hash::rand_core::{OsRng, RngCore};

fn generate_hex_key() -> String {
    let mut key = [0u8; 32];  // 256 bits
    OsRng.fill_bytes(&mut key);
    hex::encode(key)  // 64 character hex string
}

fn main() {
    let key = generate_hex_key();
    println!("ENCRYPTION_KEY={}", key);
}
```

## Best Practices

### UUID

| Do | Don't |
|----|-------|
| Use v4 for random IDs | Use v1 (contains MAC address) |
| Store as native UUID in databases | Store as string unnecessarily |
| Use `simple()` for URL-safe strings | Parse user input without validation |

### Chrono

| Do | Don't |
|----|-------|
| Store in UTC | Store in local time |
| Use `DateTime<Utc>` in APIs | Use `NaiveDateTime` for timestamps |
| Parse with explicit format | Rely on implicit parsing |

### Dotenvy

| Do | Don't |
|----|-------|
| Use `.ok()` to ignore missing .env | Require .env in production |
| Validate required variables | Assume variables exist |
| Document required variables | Commit .env with secrets |

### Hex

| Do | Don't |
|----|-------|
| Validate hex strings before decode | Assume all input is valid hex |
| Use for display/config of binary data | Use for serialization (use base64) |

## Pros and Cons Summary

| Crate | Pros | Cons |
|-------|------|------|
| **uuid** | Standard format, unique, fast | 128-bit might be overkill |
| **chrono** | Feature-complete, timezone support | Compile time, size |
| **dotenvy** | Simple, 12-factor app compatible | Limited to .env format |
| **hex** | Simple, standard | Base64 more compact |

## Further Learning

- [UUID crate docs](https://docs.rs/uuid)
- [Chrono crate docs](https://docs.rs/chrono)
- [Dotenvy crate docs](https://docs.rs/dotenvy)
- [Hex crate docs](https://docs.rs/hex)

## Related Crates

- **time** - Alternative to chrono
- **base64** - Alternative encoding
- **once_cell** / **lazy_static** - For config singletons
- **config** - More powerful configuration
