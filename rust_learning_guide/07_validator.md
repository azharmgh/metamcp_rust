# Validator - Input Validation

Validator is a Rust crate for validating struct fields using derive macros and custom validators.

## What is Validator?

Validator provides:
- **Derive macros** - Add validation rules to struct fields
- **Built-in validators** - Email, URL, length, range, and more
- **Custom validators** - Define your own validation logic
- **Async validation** - Support for async custom validators

## Why Validator?

Input validation is critical for:
- Security - Prevent malicious input
- Data integrity - Ensure data meets requirements
- User experience - Provide meaningful error messages

## Installation

```toml
[dependencies]
validator = { version = "0.20", features = ["derive"] }
```

## Basic Usage

### Derive Macro

```rust
use validator::Validate;

#[derive(Debug, Validate)]
struct CreateUser {
    #[validate(length(min = 1, max = 100))]
    name: String,

    #[validate(email)]
    email: String,

    #[validate(range(min = 18, max = 150))]
    age: u8,

    #[validate(url)]
    website: Option<String>,
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20validator%3A%3AValidate%3B%0A%0A%23%5Bderive(Debug%2C%20Validate)%5D%0Astruct%20CreateUser%20%7B%0A%20%20%20%20%23%5Bvalidate(length(min%20%3D%201%2C%20max%20%3D%20100))%5D%0A%20%20%20%20name%3A%20String%2C%0A%0A%20%20%20%20%23%5Bvalidate(email)%5D%0A%20%20%20%20email%3A%20String%2C%0A%0A%20%20%20%20%23%5Bvalidate(range(min%20%3D%2018%2C%20max%20%3D%20150))%5D%0A%20%20%20%20age%3A%20u8%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Valid%20user%0A%20%20%20%20let%20valid_user%20%3D%20CreateUser%20%7B%0A%20%20%20%20%20%20%20%20name%3A%20%22Alice%22.to_string()%2C%0A%20%20%20%20%20%20%20%20email%3A%20%22alice%40example.com%22.to_string()%2C%0A%20%20%20%20%20%20%20%20age%3A%2025%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20match%20valid_user.validate()%20%7B%0A%20%20%20%20%20%20%20%20Ok(_)%20%3D%3E%20println!(%22User%20is%20valid!%22)%2C%0A%20%20%20%20%20%20%20%20Err(e)%20%3D%3E%20println!(%22Validation%20errors%3A%20%7B%3A%3F%7D%22%2C%20e)%2C%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Invalid%20user%0A%20%20%20%20let%20invalid_user%20%3D%20CreateUser%20%7B%0A%20%20%20%20%20%20%20%20name%3A%20%22%22.to_string()%2C%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%2F%2F%20Too%20short%0A%20%20%20%20%20%20%20%20email%3A%20%22not-an-email%22.to_string()%2C%20%2F%2F%20Invalid%20email%0A%20%20%20%20%20%20%20%20age%3A%2015%2C%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%2F%2F%20Too%20young%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20match%20invalid_user.validate()%20%7B%0A%20%20%20%20%20%20%20%20Ok(_)%20%3D%3E%20println!(%22User%20is%20valid!%22)%2C%0A%20%20%20%20%20%20%20%20Err(e)%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Validation%20errors%3A%22)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20for%20(field%2C%20errors)%20in%20e.field_errors()%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20println!(%22%20%20%7B%7D%3A%20%7B%3A%3F%7D%22%2C%20field%2C%20errors)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D)

### Running Validation

```rust
use validator::Validate;

fn create_user(input: CreateUser) -> Result<User, ValidationErrors> {
    // Validate the input
    input.validate()?;

    // If we get here, validation passed
    Ok(User::from(input))
}
```

## Built-in Validators

### String Validators

```rust
#[derive(Validate)]
struct StringValidation {
    // Length constraints
    #[validate(length(min = 1))]
    required: String,

    #[validate(length(max = 100))]
    limited: String,

    #[validate(length(min = 8, max = 128))]
    password: String,

    #[validate(length(equal = 6))]
    pin_code: String,

    // Format validators
    #[validate(email)]
    email: String,

    #[validate(url)]
    website: String,

    // Regex pattern
    #[validate(regex(path = "PHONE_REGEX"))]
    phone: String,

    // Must contain only specific characters
    #[validate(contains(pattern = "@"))]
    username: String,

    // Must not contain
    #[validate(does_not_contain(pattern = "admin"))]
    safe_name: String,
}

// Define regex for phone validation
use once_cell::sync::Lazy;
use regex::Regex;

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap()
});
```

### Numeric Validators

```rust
#[derive(Validate)]
struct NumericValidation {
    #[validate(range(min = 0))]
    non_negative: i32,

    #[validate(range(max = 100))]
    percentage: u8,

    #[validate(range(min = 1, max = 10))]
    rating: u8,

    #[validate(range(exclusive_min = 0.0, exclusive_max = 1.0))]
    probability: f64,
}
```

### Collection Validators

```rust
#[derive(Validate)]
struct CollectionValidation {
    #[validate(length(min = 1, max = 10))]
    tags: Vec<String>,

    // Validate each item in collection
    #[validate]
    items: Vec<Item>,
}

#[derive(Validate)]
struct Item {
    #[validate(length(min = 1))]
    name: String,
}
```

### Nested Validation

```rust
#[derive(Validate)]
struct Order {
    #[validate(length(min = 1))]
    id: String,

    // Validate nested struct
    #[validate(nested)]
    customer: Customer,

    // Validate each item in vec
    #[validate(nested)]
    items: Vec<OrderItem>,
}

#[derive(Validate)]
struct Customer {
    #[validate(length(min = 1))]
    name: String,

    #[validate(email)]
    email: String,
}

#[derive(Validate)]
struct OrderItem {
    #[validate(range(min = 1))]
    quantity: u32,
}
```

[Run nested validation example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20validator%3A%3AValidate%3B%0A%0A%23%5Bderive(Debug%2C%20Validate)%5D%0Astruct%20Address%20%7B%0A%20%20%20%20%23%5Bvalidate(length(min%20%3D%201))%5D%0A%20%20%20%20street%3A%20String%2C%0A%20%20%20%20%0A%20%20%20%20%23%5Bvalidate(length(min%20%3D%201))%5D%0A%20%20%20%20city%3A%20String%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Validate)%5D%0Astruct%20Person%20%7B%0A%20%20%20%20%23%5Bvalidate(length(min%20%3D%201))%5D%0A%20%20%20%20name%3A%20String%2C%0A%20%20%20%20%0A%20%20%20%20%23%5Bvalidate(nested)%5D%0A%20%20%20%20address%3A%20Address%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20person%20%3D%20Person%20%7B%0A%20%20%20%20%20%20%20%20name%3A%20%22Alice%22.to_string()%2C%0A%20%20%20%20%20%20%20%20address%3A%20Address%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20street%3A%20%22%22.to_string()%2C%20%20%2F%2F%20Invalid%20-%20empty%0A%20%20%20%20%20%20%20%20%20%20%20%20city%3A%20%22NYC%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20match%20person.validate()%20%7B%0A%20%20%20%20%20%20%20%20Ok(_)%20%3D%3E%20println!(%22Valid!%22)%2C%0A%20%20%20%20%20%20%20%20Err(e)%20%3D%3E%20println!(%22Errors%3A%20%7B%3A%3F%7D%22%2C%20e)%2C%0A%20%20%20%20%7D%0A%7D)

## Custom Validators

### Function Validator

```rust
use validator::ValidationError;

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_username"))
    }
}

#[derive(Validate)]
struct User {
    #[validate(custom(function = "validate_username"))]
    username: String,
}
```

### Custom Validator with Message

```rust
fn validate_strong_password(password: &str) -> Result<(), ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if has_uppercase && has_lowercase && has_digit && has_special {
        Ok(())
    } else {
        let mut error = ValidationError::new("weak_password");
        error.message = Some("Password must contain uppercase, lowercase, digit, and special character".into());
        Err(error)
    }
}
```

[Run custom validator example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20validator%3A%3A%7BValidate%2C%20ValidationError%7D%3B%0A%0Afn%20validate_username(username%3A%20%26str)%20-%3E%20Result%3C()%2C%20ValidationError%3E%20%7B%0A%20%20%20%20%2F%2F%20Must%20be%20alphanumeric%20or%20underscore%0A%20%20%20%20if%20username.chars().all(%7Cc%7C%20c.is_alphanumeric()%20%7C%7C%20c%20%3D%3D%20%27_%27)%20%7B%0A%20%20%20%20%20%20%20%20%2F%2F%20Must%20start%20with%20letter%0A%20%20%20%20%20%20%20%20if%20username.chars().next().map(%7Cc%7C%20c.is_alphabetic()).unwrap_or(false)%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20return%20Ok(())%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20let%20mut%20err%20%3D%20ValidationError%3A%3Anew(%22invalid_username%22)%3B%0A%20%20%20%20err.message%20%3D%20Some(%22Username%20must%20start%20with%20letter%20and%20contain%20only%20alphanumeric%20or%20underscore%22.into())%3B%0A%20%20%20%20Err(err)%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Validate)%5D%0Astruct%20User%20%7B%0A%20%20%20%20%23%5Bvalidate(custom(function%20%3D%20%22validate_username%22))%5D%0A%20%20%20%20username%3A%20String%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20valid%20%3D%20User%20%7B%20username%3A%20%22alice_123%22.to_string()%20%7D%3B%0A%20%20%20%20println!(%22alice_123%3A%20%7B%3A%3F%7D%22%2C%20valid.validate())%3B%0A%20%20%20%20%0A%20%20%20%20let%20invalid1%20%3D%20User%20%7B%20username%3A%20%22123alice%22.to_string()%20%7D%3B%0A%20%20%20%20println!(%22123alice%3A%20%7B%3A%3F%7D%22%2C%20invalid1.validate())%3B%0A%20%20%20%20%0A%20%20%20%20let%20invalid2%20%3D%20User%20%7B%20username%3A%20%22alice%40home%22.to_string()%20%7D%3B%0A%20%20%20%20println!(%22alice%40home%3A%20%7B%3A%3F%7D%22%2C%20invalid2.validate())%3B%0A%7D)

## Integration with Axum

### Validating Request Bodies

```rust
use axum::{extract::Json, response::IntoResponse};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,
}

pub async fn create_user(
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate the input
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Process valid input
    Ok(Json(User::from(payload)))
}
```

### Custom Extractor with Validation

```rust
use axum::{
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| e.into_response())?;

        value.validate()
            .map_err(|e| {
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            })?;

        Ok(ValidatedJson(value))
    }
}

// Usage in handler
async fn create_user(
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> impl IntoResponse {
    // payload is already validated!
    Json(User::from(payload))
}
```

## Error Handling

### Working with ValidationErrors

```rust
use validator::{Validate, ValidationErrors};

fn format_validation_errors(errors: &ValidationErrors) -> Vec<String> {
    let mut messages = Vec::new();

    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = error.message
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| {
                    format!("{}: validation failed ({})", field, error.code)
                });
            messages.push(message);
        }
    }

    messages
}
```

## Best Practices

### DO

1. **Validate at boundaries** - Validate input as soon as it enters your system
2. **Use meaningful error messages** - Help users understand what's wrong
3. **Combine with serde** - Deserialize then validate
4. **Validate nested structures** - Use `#[validate(nested)]`
5. **Create custom validators** - For business logic rules

### DON'T

1. **Don't validate in domain logic** - Validate once at the boundary
2. **Don't expose internal errors** - Map to user-friendly messages
3. **Don't skip validation** - Even for "trusted" internal APIs
4. **Don't validate optional fields if None** - Use conditional validation

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Declarative** | Rules defined with attributes |
| **Composable** | Nested validation support |
| **Extensible** | Custom validators |
| **Standard** | Common validators built-in |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Compile time** | Derive macros add compile time |
| **Limited async** | Async validators less ergonomic |
| **Error messages** | Default messages not always clear |

## When to Use Validator

**Use Validator when:**
- Validating API request bodies
- Form input validation
- Configuration validation
- Any user-provided input

**Consider alternatives when:**
- Very simple validation (manual checks)
- Complex cross-field validation (custom logic)

## Further Learning

### Official Resources
- [Validator Documentation](https://docs.rs/validator)
- [Validator GitHub](https://github.com/Keats/validator)

### Practice
1. Create a registration form validator
2. Implement custom password strength validator
3. Build a validated JSON extractor for Axum

## Related Crates

- **garde** - Alternative validation library
- **serde** - Deserialization (use together)
- **thiserror** - Custom error types
