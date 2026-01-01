# Error Handling (thiserror, anyhow)

This guide covers two essential error handling crates that work together to provide idiomatic error handling in Rust.

## Overview

| Crate | Purpose | Use Case |
|-------|---------|----------|
| **thiserror** | Define custom error types | Libraries, specific errors |
| **anyhow** | Easy error propagation | Applications, quick prototyping |

## Installation

```toml
[dependencies]
thiserror = "2"
anyhow = "1"
```

---

## thiserror - Custom Error Types

`thiserror` provides derive macros to easily create custom error types that implement `std::error::Error`.

### Basic Usage

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Database error")]
    Database(#[from] sqlx::Error),
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20std%3A%3Afmt%3B%0A%0A%2F%2F%20Simplified%20version%20of%20thiserror%20functionality%0A%23%5Bderive(Debug)%5D%0Aenum%20MyError%20%7B%0A%20%20%20%20InvalidInput(String)%2C%0A%20%20%20%20NotFound%20%7B%20resource%3A%20String%20%7D%2C%0A%20%20%20%20IoError(std%3A%3Aio%3A%3AError)%2C%0A%7D%0A%0Aimpl%20fmt%3A%3ADisplay%20for%20MyError%20%7B%0A%20%20%20%20fn%20fmt(%26self%2C%20f%3A%20%26mut%20fmt%3A%3AFormatter%3C%27_%3E)%20-%3E%20fmt%3A%3AResult%20%7B%0A%20%20%20%20%20%20%20%20match%20self%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20MyError%3A%3AInvalidInput(msg)%20%3D%3E%20write!(f%2C%20%22Invalid%20input%3A%20%7B%7D%22%2C%20msg)%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20MyError%3A%3ANotFound%20%7B%20resource%20%7D%20%3D%3E%20write!(f%2C%20%22Not%20found%3A%20%7B%7D%22%2C%20resource)%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20MyError%3A%3AIoError(e)%20%3D%3E%20write!(f%2C%20%22IO%20error%3A%20%7B%7D%22%2C%20e)%2C%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D%0A%0Aimpl%20std%3A%3Aerror%3A%3AError%20for%20MyError%20%7B%7D%0A%0A%2F%2F%20Automatic%20From%20implementation%0Aimpl%20From%3Cstd%3A%3Aio%3A%3AError%3E%20for%20MyError%20%7B%0A%20%20%20%20fn%20from(err%3A%20std%3A%3Aio%3A%3AError)%20-%3E%20Self%20%7B%0A%20%20%20%20%20%20%20%20MyError%3A%3AIoError(err)%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20errors%20%3D%20vec!%5B%0A%20%20%20%20%20%20%20%20MyError%3A%3AInvalidInput(%22bad%20data%22.to_string())%2C%0A%20%20%20%20%20%20%20%20MyError%3A%3ANotFound%20%7B%20resource%3A%20%22user_123%22.to_string()%20%7D%2C%0A%20%20%20%20%5D%3B%0A%20%20%20%20%0A%20%20%20%20for%20err%20in%20errors%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Error%3A%20%7B%7D%22%2C%20err)%3B%0A%20%20%20%20%7D%0A%7D)

### Real Example from MetaMCP

From `src/utils/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    // Automatic conversion from sqlx::Error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // Automatic conversion from JWT errors
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Process error: {0}")]
    Process(String),
}
```

### Key Features

#### Error Messages

```rust
#[derive(Error, Debug)]
pub enum ApiError {
    // Simple message
    #[error("Something went wrong")]
    Generic,

    // With field interpolation
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    // With named fields
    #[error("User {user_id} not found in {database}")]
    UserNotFound {
        user_id: String,
        database: String,
    },

    // Display the source error
    #[error("Failed to parse config")]
    ConfigParse(#[source] std::io::Error),

    // Transparent - delegates Display to source
    #[error(transparent)]
    Other(#[from] std::io::Error),
}
```

#### From Implementation

The `#[from]` attribute automatically implements `From`:

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

// This is automatically generated:
// impl From<sqlx::Error> for AppError {
//     fn from(err: sqlx::Error) -> Self {
//         AppError::Database(err)
//     }
// }

// Now you can use ? operator
async fn get_user(pool: &PgPool, id: Uuid) -> Result<User, AppError> {
    let user = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;  // Automatically converts sqlx::Error to AppError

    Ok(user)
}
```

#### Source Errors

```rust
#[derive(Error, Debug)]
pub enum MyError {
    // #[source] preserves the error chain
    #[error("Failed to read file")]
    ReadFile(#[source] std::io::Error),

    // #[from] implies #[source]
    #[error("Parse error")]
    Parse(#[from] serde_json::Error),
}

// Access the source
fn handle_error(err: MyError) {
    println!("Error: {}", err);
    if let Some(source) = err.source() {
        println!("Caused by: {}", source);
    }
}
```

### Implementing IntoResponse for Axum

From `src/utils/error.rs`:

```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match &self {
            AppError::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized", Some(msg.clone()))
            }
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, "Forbidden", Some(msg.clone()))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "Not Found", Some(msg.clone()))
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "Bad Request", Some(msg.clone()))
            }
            AppError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "Validation Error", Some(msg.clone()))
            }
            AppError::Internal(msg) => {
                // Log internal errors, don't expose to client
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error", None)
            }
            AppError::Database(err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error", None)
            }
            AppError::Jwt(err) => {
                tracing::warn!("JWT error: {}", err);
                (StatusCode::UNAUTHORIZED, "Invalid Token", Some(err.to_string()))
            }
            // ... other variants
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
            message: self.to_string(),
            details,
        });

        (status, body).into_response()
    }
}
```

[Run error response example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3ASerialize%3B%0A%0A%23%5Bderive(Debug)%5D%0Aenum%20AppError%20%7B%0A%20%20%20%20NotFound(String)%2C%0A%20%20%20%20BadRequest(String)%2C%0A%20%20%20%20Internal(String)%2C%0A%7D%0A%0Aimpl%20std%3A%3Afmt%3A%3ADisplay%20for%20AppError%20%7B%0A%20%20%20%20fn%20fmt(%26self%2C%20f%3A%20%26mut%20std%3A%3Afmt%3A%3AFormatter)%20-%3E%20std%3A%3Afmt%3A%3AResult%20%7B%0A%20%20%20%20%20%20%20%20match%20self%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20AppError%3A%3ANotFound(s)%20%3D%3E%20write!(f%2C%20%22Not%20found%3A%20%7B%7D%22%2C%20s)%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20AppError%3A%3ABadRequest(s)%20%3D%3E%20write!(f%2C%20%22Bad%20request%3A%20%7B%7D%22%2C%20s)%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20AppError%3A%3AInternal(s)%20%3D%3E%20write!(f%2C%20%22Internal%3A%20%7B%7D%22%2C%20s)%2C%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D%0A%0A%23%5Bderive(Serialize)%5D%0Astruct%20ErrorResponse%20%7B%0A%20%20%20%20status%3A%20u16%2C%0A%20%20%20%20error%3A%20String%2C%0A%20%20%20%20message%3A%20String%2C%0A%7D%0A%0Afn%20to_response(err%3A%20%26AppError)%20-%3E%20ErrorResponse%20%7B%0A%20%20%20%20match%20err%20%7B%0A%20%20%20%20%20%20%20%20AppError%3A%3ANotFound(msg)%20%3D%3E%20ErrorResponse%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20status%3A%20404%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20error%3A%20%22Not%20Found%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20message%3A%20msg.clone()%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20AppError%3A%3ABadRequest(msg)%20%3D%3E%20ErrorResponse%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20status%3A%20400%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20error%3A%20%22Bad%20Request%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20message%3A%20msg.clone()%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20AppError%3A%3AInternal(_)%20%3D%3E%20ErrorResponse%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20status%3A%20500%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20error%3A%20%22Internal%20Error%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20message%3A%20%22Something%20went%20wrong%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20errors%20%3D%20vec!%5B%0A%20%20%20%20%20%20%20%20AppError%3A%3ANotFound(%22User%20123%22.to_string())%2C%0A%20%20%20%20%20%20%20%20AppError%3A%3ABadRequest(%22Invalid%20email%22.to_string())%2C%0A%20%20%20%20%20%20%20%20AppError%3A%3AInternal(%22DB%20connection%20failed%22.to_string())%2C%0A%20%20%20%20%5D%3B%0A%20%20%20%20%0A%20%20%20%20for%20err%20in%20%26errors%20%7B%0A%20%20%20%20%20%20%20%20let%20response%20%3D%20to_response(err)%3B%0A%20%20%20%20%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26response).unwrap())%3B%0A%20%20%20%20%20%20%20%20println!()%3B%0A%20%20%20%20%7D%0A%7D)

---

## anyhow - Easy Error Propagation

`anyhow` provides a simple `Error` type that can hold any error, making it easy to propagate errors without defining custom types.

### Basic Usage

From `src/main.rs`:

```rust
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    let db = Database::new(&config.database_url).await?;
    db.run_migrations().await?;

    // ... rest of app

    Ok(())
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20anyhow%3A%3A%7BResult%2C%20Context%2C%20anyhow%7D%3B%0A%0Afn%20load_config()%20-%3E%20Result%3CString%3E%20%7B%0A%20%20%20%20%2F%2F%20Simulating%20file%20read%20that%20might%20fail%0A%20%20%20%20let%20content%20%3D%20std%3A%3Afs%3A%3Aread_to_string(%22config.toml%22)%0A%20%20%20%20%20%20%20%20.context(%22Failed%20to%20read%20config%20file%22)%3F%3B%0A%20%20%20%20Ok(content)%0A%7D%0A%0Afn%20parse_port(s%3A%20%26str)%20-%3E%20Result%3Cu16%3E%20%7B%0A%20%20%20%20s.parse()%0A%20%20%20%20%20%20%20%20.context(format!(%22Invalid%20port%20number%3A%20%7B%7D%22%2C%20s))%0A%7D%0A%0Afn%20main()%20-%3E%20Result%3C()%3E%20%7B%0A%20%20%20%20%2F%2F%20This%20will%20fail%20with%20nice%20context%0A%20%20%20%20match%20load_config()%20%7B%0A%20%20%20%20%20%20%20%20Ok(config)%20%3D%3E%20println!(%22Config%3A%20%7B%7D%22%2C%20config)%2C%0A%20%20%20%20%20%20%20%20Err(e)%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Error%3A%20%7B%3A%23%7D%22%2C%20e)%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Create%20error%20directly%0A%20%20%20%20let%20err%20%3D%20anyhow!(%22Something%20went%20wrong%22)%3B%0A%20%20%20%20println!(%22Error%3A%20%7B%7D%22%2C%20err)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Parse%20with%20context%0A%20%20%20%20match%20parse_port(%22not_a_number%22)%20%7B%0A%20%20%20%20%20%20%20%20Ok(port)%20%3D%3E%20println!(%22Port%3A%20%7B%7D%22%2C%20port)%2C%0A%20%20%20%20%20%20%20%20Err(e)%20%3D%3E%20println!(%22Error%3A%20%7B%3A%23%7D%22%2C%20e)%2C%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20Ok(())%0A%7D)

### Key Features

#### Context

Add context to errors:

```rust
use anyhow::{Context, Result};

fn read_config(path: &str) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .context("Failed to read config file")?;

    let config: Config = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse config from {}", path))?;

    Ok(config)
}
```

#### Creating Errors

```rust
use anyhow::{anyhow, bail, ensure, Result};

fn validate_input(value: i32) -> Result<()> {
    // bail! for early return with error
    if value < 0 {
        bail!("Value must be non-negative, got {}", value);
    }

    // ensure! for assertions
    ensure!(value <= 100, "Value must be at most 100, got {}", value);

    Ok(())
}

fn process() -> Result<()> {
    // anyhow! creates an error inline
    return Err(anyhow!("Something unexpected happened"));
}
```

#### Error Chain

```rust
use anyhow::{Context, Result};
use std::fs;

fn load_user_config() -> Result<UserConfig> {
    let path = get_config_path()
        .context("Failed to determine config path")?;

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let config = toml::from_str(&contents)
        .context("Failed to parse config")?;

    Ok(config)
}

// Error chain example output:
// Error: Failed to read /home/user/.config/app/config.toml
//
// Caused by:
//     0: No such file or directory (os error 2)
```

### anyhow vs thiserror

Use both together - `thiserror` for defining errors, `anyhow` for propagating:

```rust
// In your library/domain code: use thiserror
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid email format")]
    InvalidEmail,
}

// In your application main: use anyhow
use anyhow::Result;

fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to initialize application")?;

    run_server(config)?;

    Ok(())
}
```

## Type Alias Pattern

From `src/utils/error.rs`:

```rust
// Define a Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

// Use throughout the codebase
pub async fn get_user(id: Uuid) -> AppResult<User> {
    // ...
}
```

## Best Practices

### DO

1. **Use thiserror for libraries** - Callers need specific error types
2. **Use anyhow for applications** - Simpler error propagation
3. **Add context to errors** - Help with debugging
4. **Map errors appropriately** - Don't expose internal errors
5. **Log errors at boundaries** - Before converting to responses

### DON'T

1. **Don't use `.unwrap()` in production** - Handle errors properly
2. **Don't lose error context** - Use `?` with context
3. **Don't expose internal errors** - Map to user-friendly messages
4. **Don't panic in libraries** - Return errors instead

## Error Handling Patterns

### Converting Between Error Types

```rust
// Using map_err
let result = some_operation()
    .map_err(|e| AppError::Internal(e.to_string()))?;

// Using From implementation (automatic with #[from])
let result = some_operation()?;  // If From is implemented

// Adding context with anyhow
let result = some_operation()
    .context("Operation failed")?;
```

### Handling Multiple Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

// Now any of these can use ?
fn process() -> Result<(), AppError> {
    let file = std::fs::read_to_string("data.json")?;  // io::Error -> AppError
    let data: Data = serde_json::from_str(&file)?;     // serde::Error -> AppError
    save_to_db(&data).await?;                          // sqlx::Error -> AppError
    Ok(())
}
```

## Pros and Cons

### thiserror

| Pros | Cons |
|------|------|
| Type-safe error handling | More boilerplate |
| Clear error variants | Must define all errors upfront |
| Works well with match | Can lead to large error enums |

### anyhow

| Pros | Cons |
|------|------|
| Very ergonomic | Loses type information |
| Great for applications | Not ideal for libraries |
| Easy error context | Harder to pattern match |

## When to Use

**Use thiserror when:**
- Writing a library
- Callers need to handle specific errors
- You want exhaustive error handling

**Use anyhow when:**
- Writing an application
- Errors are logged/displayed, not handled specifically
- Prototyping or quick scripts

**Use both when:**
- Application with domain-specific errors
- You want type safety internally but easy propagation

## Further Learning

### Official Resources
- [thiserror Documentation](https://docs.rs/thiserror)
- [anyhow Documentation](https://docs.rs/anyhow)
- [Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

### Practice
1. Create a custom error type for a domain
2. Implement `IntoResponse` for Axum
3. Use anyhow in a CLI application
4. Chain errors with context

## Related Crates

- **eyre** - Alternative to anyhow with better reports
- **color-eyre** - Colorful error reports
- **snafu** - Alternative to thiserror
- **displaydoc** - Simpler Display derive
