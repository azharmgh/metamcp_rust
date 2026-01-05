# Serde & Serde JSON - Serialization

Serde is Rust's most popular serialization/deserialization framework. It's a fundamental crate used in almost every Rust web application.

## What is Serde?

Serde provides:
- **Serialization** - Converting Rust data structures to formats like JSON, YAML, TOML
- **Deserialization** - Converting data formats back to Rust structures
- **Derive macros** - Automatic implementation via `#[derive(Serialize, Deserialize)]`

## Why Serde?

Serde is the de-facto standard for Rust serialization:
- Zero-copy deserialization when possible
- Compile-time code generation (no runtime reflection)
- Support for every common data format
- Extensive customization options

## Installation

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

The `derive` feature enables `#[derive(Serialize, Deserialize)]`.

## Basic Usage

### Derive Macros

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20User%20%7B%0A%20%20%20%20id%3A%20u64%2C%0A%20%20%20%20name%3A%20String%2C%0A%20%20%20%20email%3A%20String%2C%0A%20%20%20%20active%3A%20bool%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Create%20a%20user%0A%20%20%20%20let%20user%20%3D%20User%20%7B%0A%20%20%20%20%20%20%20%20id%3A%201%2C%0A%20%20%20%20%20%20%20%20name%3A%20%22Alice%22.to_string()%2C%0A%20%20%20%20%20%20%20%20email%3A%20%22alice%40example.com%22.to_string()%2C%0A%20%20%20%20%20%20%20%20active%3A%20true%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Serialize%20to%20JSON%0A%20%20%20%20let%20json%20%3D%20serde_json%3A%3Ato_string_pretty(%26user).unwrap()%3B%0A%20%20%20%20println!(%22Serialized%3A%5Cn%7B%7D%22%2C%20json)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Deserialize%20from%20JSON%0A%20%20%20%20let%20parsed%3A%20User%20%3D%20serde_json%3A%3Afrom_str(%26json).unwrap()%3B%0A%20%20%20%20println!(%22Deserialized%3A%20%7B%3A%3F%7D%22%2C%20parsed)%3B%0A%7D)

### JSON Operations

```rust
use serde_json::{json, Value};

// Serialize struct to JSON string
let json_string = serde_json::to_string(&user)?;

// Pretty print JSON
let pretty_json = serde_json::to_string_pretty(&user)?;

// Deserialize JSON string to struct
let user: User = serde_json::from_str(&json_string)?;

// Create JSON dynamically
let data = json!({
    "name": "Alice",
    "age": 30,
    "tags": ["rust", "developer"]
});

// Work with untyped JSON
let value: Value = serde_json::from_str(&json_string)?;
if let Some(name) = value.get("name") {
    println!("Name: {}", name);
}
```

[Run JSON examples](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Create%20JSON%20dynamically%20with%20json!%20macro%0A%20%20%20%20let%20data%20%3D%20json!(%7B%0A%20%20%20%20%20%20%20%20%22name%22%3A%20%22Alice%22%2C%0A%20%20%20%20%20%20%20%20%22age%22%3A%2030%2C%0A%20%20%20%20%20%20%20%20%22languages%22%3A%20%5B%22Rust%22%2C%20%22Python%22%5D%2C%0A%20%20%20%20%20%20%20%20%22metadata%22%3A%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22verified%22%3A%20true%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D)%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22JSON%3A%20%7B%7D%22%2C%20data)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Access%20nested%20values%0A%20%20%20%20if%20let%20Some(name)%20%3D%20data.get(%22name%22)%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Name%3A%20%7B%7D%22%2C%20name)%3B%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Parse%20JSON%20string%20to%20Value%0A%20%20%20%20let%20json_str%20%3D%20r%23%22%7B%22id%22%3A%201%2C%20%22active%22%3A%20true%7D%22%23%3B%0A%20%20%20%20let%20value%3A%20Value%20%3D%20serde_json%3A%3Afrom_str(json_str).unwrap()%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22ID%3A%20%7B%7D%22%2C%20value%5B%22id%22%5D)%3B%0A%20%20%20%20println!(%22Active%3A%20%7B%7D%22%2C%20value%5B%22active%22%5D)%3B%0A%7D)

## Real Examples from MetaMCP

### API Request/Response Types

From `src/api/handlers/auth.rs`:

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthRequest {
    #[schema(example = "mcp_a1b2c3d4e5f6")]
    pub api_key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    #[schema(example = "Bearer")]
    pub token_type: String,
    #[schema(example = 900)]
    pub expires_in: u64,
}
```

### Tagged Enums

From `src/streaming/manager.rs` - using `#[serde(tag)]` for polymorphic JSON:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    McpServerStarted { server_id: String, name: String },
    McpServerStopped { server_id: String, reason: String },
    McpToolExecuted { server_id: String, tool: String, status: String },
    McpMessage { server_id: String, message: serde_json::Value },
    SystemHealth { cpu: f32, memory: f32, active_servers: usize },
    Error { code: String, message: String },
}
```

This produces JSON like:
```json
{
    "type": "mcp_server_started",
    "server_id": "abc123",
    "name": "my-server"
}
```

[Run tagged enum example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(tag%20%3D%20%22type%22%2C%20rename_all%20%3D%20%22snake_case%22)%5D%0Aenum%20Event%20%7B%0A%20%20%20%20UserCreated%20%7B%20user_id%3A%20String%2C%20name%3A%20String%20%7D%2C%0A%20%20%20%20UserDeleted%20%7B%20user_id%3A%20String%20%7D%2C%0A%20%20%20%20SystemError%20%7B%20code%3A%20String%2C%20message%3A%20String%20%7D%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20events%20%3D%20vec!%5B%0A%20%20%20%20%20%20%20%20Event%3A%3AUserCreated%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20user_id%3A%20%22123%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20name%3A%20%22Alice%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20Event%3A%3AUserDeleted%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20user_id%3A%20%22456%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20Event%3A%3ASystemError%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20code%3A%20%22E001%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20message%3A%20%22Something%20went%20wrong%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%5D%3B%0A%20%20%20%20%0A%20%20%20%20for%20event%20in%20%26events%20%7B%0A%20%20%20%20%20%20%20%20let%20json%20%3D%20serde_json%3A%3Ato_string_pretty(event).unwrap()%3B%0A%20%20%20%20%20%20%20%20println!(%22%7B%7D%5Cn%22%2C%20json)%3B%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Deserialize%20back%0A%20%20%20%20let%20json%20%3D%20r%23%22%7B%22type%22%3A%20%22user_created%22%2C%20%22user_id%22%3A%20%22789%22%2C%20%22name%22%3A%20%22Bob%22%7D%22%23%3B%0A%20%20%20%20let%20parsed%3A%20Event%20%3D%20serde_json%3A%3Afrom_str(json).unwrap()%3B%0A%20%20%20%20println!(%22Parsed%3A%20%7B%3A%3F%7D%22%2C%20parsed)%3B%0A%7D)

### JWT Claims

From `src/auth/jwt.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // Subject (user ID)
    pub exp: usize,     // Expiration time
    pub iat: usize,     // Issued at
    pub jti: String,    // JWT ID
}
```

### Database Models with Custom Types

From `src/db/models/api_key.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    #[serde(skip_serializing)]  // Don't include in JSON output
    pub encrypted_key: Vec<u8>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}
```

## Common Serde Attributes

### Field Attributes

```rust
#[derive(Serialize, Deserialize)]
struct Example {
    // Rename field in JSON
    #[serde(rename = "firstName")]
    first_name: String,

    // Skip if None
    #[serde(skip_serializing_if = "Option::is_none")]
    middle_name: Option<String>,

    // Use default if missing during deserialization
    #[serde(default)]
    count: u32,

    // Skip this field entirely
    #[serde(skip)]
    internal_state: String,

    // Custom serialization
    #[serde(serialize_with = "serialize_date")]
    created_at: DateTime<Utc>,

    // Flatten nested struct
    #[serde(flatten)]
    metadata: Metadata,
}
```

[Run attributes example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Serialize%2C%20Deserialize)%5D%0Astruct%20User%20%7B%0A%20%20%20%20%2F%2F%20Rename%20to%20camelCase%20in%20JSON%0A%20%20%20%20%23%5Bserde(rename%20%3D%20%22firstName%22)%5D%0A%20%20%20%20first_name%3A%20String%2C%0A%20%20%20%20%0A%20%20%20%20%23%5Bserde(rename%20%3D%20%22lastName%22)%5D%0A%20%20%20%20last_name%3A%20String%2C%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Skip%20if%20None%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20nickname%3A%20Option%3CString%3E%2C%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Default%20value%20if%20missing%0A%20%20%20%20%23%5Bserde(default)%5D%0A%20%20%20%20age%3A%20u32%2C%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Never%20serialize%0A%20%20%20%20%23%5Bserde(skip)%5D%0A%20%20%20%20internal_id%3A%20u64%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20user%20%3D%20User%20%7B%0A%20%20%20%20%20%20%20%20first_name%3A%20%22Alice%22.to_string()%2C%0A%20%20%20%20%20%20%20%20last_name%3A%20%22Smith%22.to_string()%2C%0A%20%20%20%20%20%20%20%20nickname%3A%20None%2C%0A%20%20%20%20%20%20%20%20age%3A%2030%2C%0A%20%20%20%20%20%20%20%20internal_id%3A%2012345%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20let%20json%20%3D%20serde_json%3A%3Ato_string_pretty(%26user).unwrap()%3B%0A%20%20%20%20println!(%22Serialized%20(no%20nickname%2C%20no%20internal_id)%3A%5Cn%7B%7D%22%2C%20json)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Deserialize%20with%20missing%20age%20(uses%20default%200)%0A%20%20%20%20let%20json%20%3D%20r%23%22%7B%22firstName%22%3A%22Bob%22%2C%22lastName%22%3A%22Jones%22%7D%22%23%3B%0A%20%20%20%20let%20user2%3A%20User%20%3D%20serde_json%3A%3Afrom_str(json).unwrap()%3B%0A%20%20%20%20println!(%22%5CnDeserialized%20with%20default%20age%3A%20%7B%3A%3F%7D%22%2C%20user2)%3B%0A%7D)

### Container Attributes

```rust
// Rename all fields to camelCase
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    server_host: String,    // becomes "serverHost"
    server_port: u16,       // becomes "serverPort"
}

// Deny unknown fields during deserialization
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictInput {
    name: String,
}

// Use default for all missing fields
#[derive(Deserialize)]
#[serde(default)]
struct WithDefaults {
    enabled: bool,
    count: u32,
}
```

### Enum Representations

```rust
// Externally tagged (default)
#[derive(Serialize)]
enum Message {
    Request { id: u32 },
    Response { data: String },
}
// {"Request": {"id": 1}}

// Internally tagged
#[derive(Serialize)]
#[serde(tag = "type")]
enum Message {
    Request { id: u32 },
    Response { data: String },
}
// {"type": "Request", "id": 1}

// Adjacently tagged
#[derive(Serialize)]
#[serde(tag = "t", content = "c")]
enum Message {
    Request { id: u32 },
    Response { data: String },
}
// {"t": "Request", "c": {"id": 1}}

// Untagged
#[derive(Serialize)]
#[serde(untagged)]
enum Message {
    Request { id: u32 },
    Response { data: String },
}
// {"id": 1} (tries each variant)
```

[Run enum example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3ASerialize%3B%0A%0A%2F%2F%20Different%20enum%20tagging%20strategies%0A%0A%23%5Bderive(Serialize)%5D%0Aenum%20External%20%7B%0A%20%20%20%20Variant%20%7B%20value%3A%20i32%20%7D%2C%0A%7D%0A%0A%23%5Bderive(Serialize)%5D%0A%23%5Bserde(tag%20%3D%20%22type%22)%5D%0Aenum%20Internal%20%7B%0A%20%20%20%20Variant%20%7B%20value%3A%20i32%20%7D%2C%0A%7D%0A%0A%23%5Bderive(Serialize)%5D%0A%23%5Bserde(tag%20%3D%20%22t%22%2C%20content%20%3D%20%22c%22)%5D%0Aenum%20Adjacent%20%7B%0A%20%20%20%20Variant%20%7B%20value%3A%20i32%20%7D%2C%0A%7D%0A%0A%23%5Bderive(Serialize)%5D%0A%23%5Bserde(untagged)%5D%0Aenum%20Untagged%20%7B%0A%20%20%20%20Variant%20%7B%20value%3A%20i32%20%7D%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22External%3A%20%7B%7D%22%2C%0A%20%20%20%20%20%20%20%20serde_json%3A%3Ato_string(%26External%3A%3AVariant%20%7B%20value%3A%2042%20%7D).unwrap())%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Internal%3A%20%7B%7D%22%2C%0A%20%20%20%20%20%20%20%20serde_json%3A%3Ato_string(%26Internal%3A%3AVariant%20%7B%20value%3A%2042%20%7D).unwrap())%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Adjacent%3A%20%7B%7D%22%2C%0A%20%20%20%20%20%20%20%20serde_json%3A%3Ato_string(%26Adjacent%3A%3AVariant%20%7B%20value%3A%2042%20%7D).unwrap())%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Untagged%3A%20%7B%7D%22%2C%0A%20%20%20%20%20%20%20%20serde_json%3A%3Ato_string(%26Untagged%3A%3AVariant%20%7B%20value%3A%2042%20%7D).unwrap())%3B%0A%7D)

## Working with serde_json::Value

For dynamic JSON:

```rust
use serde_json::{json, Value, Map};

fn process_dynamic_json() {
    // Parse unknown JSON
    let data: Value = serde_json::from_str(r#"{"key": "value", "count": 42}"#)?;

    // Access values
    match &data {
        Value::Object(map) => {
            for (key, value) in map {
                println!("{}: {}", key, value);
            }
        }
        _ => {}
    }

    // Modify JSON
    let mut obj = json!({"name": "test"});
    obj["new_field"] = json!("new_value");
    obj["nested"] = json!({"a": 1, "b": 2});

    // Convert Value to typed struct
    #[derive(Deserialize)]
    struct Config {
        name: String,
    }
    let config: Config = serde_json::from_value(obj)?;
}
```

## Error Handling

```rust
use serde_json::Error;

fn parse_user(json: &str) -> Result<User, Error> {
    serde_json::from_str(json)
}

fn main() {
    match parse_user(r#"{"invalid": "json"}"#) {
        Ok(user) => println!("Parsed: {:?}", user),
        Err(e) => {
            eprintln!("Parse error: {}", e);
            // Detailed error info
            eprintln!("Line: {}, Column: {}", e.line(), e.column());
        }
    }
}
```

## Best Practices

### DO

1. **Use derive macros** - Automatic and efficient
2. **Define clear types** - Type-safe over `Value`
3. **Use `#[serde(rename_all)]`** - Consistent naming
4. **Handle `Option` properly** - Use `skip_serializing_if`
5. **Validate after deserializing** - Serde doesn't validate business logic

### DON'T

1. **Don't use `Value` everywhere** - Lose type safety
2. **Don't forget error handling** - Deserialization can fail
3. **Don't serialize sensitive data** - Use `#[serde(skip)]`
4. **Don't ignore versioning** - Add `#[serde(default)]` for new fields

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Fast** | Zero-copy where possible |
| **Type-safe** | Compile-time validation |
| **Flexible** | Many formats supported |
| **Customizable** | Extensive attributes |
| **Ecosystem** | Used everywhere |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Compile time** | Derive macros add compile time |
| **Complex errors** | Macro errors can be cryptic |
| **Learning curve** | Many attributes to learn |

## When to Use

**Use Serde when:**
- Working with JSON APIs
- Serializing to/from files
- Database communication (via other crates)
- Any data format conversion

**Consider alternatives when:**
- Simple CSV (use `csv` crate directly)
- Protocol Buffers (use `prost`)
- Very performance-critical binary (use `bincode` or `rkyv`)

## Further Learning

### Official Resources
- [Serde Documentation](https://serde.rs/)
- [Serde JSON Documentation](https://docs.rs/serde_json)
- [Serde Attributes Reference](https://serde.rs/attributes.html)

### Practice
1. Create an API request/response pair
2. Implement custom serialization
3. Handle nested JSON structures
4. Work with enum variants

## Related Crates

- **serde_yaml** - YAML support
- **toml** - TOML support
- **serde_urlencoded** - URL query strings
- **csv** - CSV files
- **bincode** - Compact binary format
