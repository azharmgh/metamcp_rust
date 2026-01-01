# SQLx - Database Access

SQLx is a compile-time verified, pure Rust SQL toolkit that provides type-safe database access without an ORM.

## What is SQLx?

SQLx provides:
- **Compile-time query verification** - SQL errors caught at build time
- **Async-first** - Built for async Rust from the ground up
- **Type-safe** - Automatic type mapping between SQL and Rust
- **Multiple databases** - PostgreSQL, MySQL, SQLite, MSSQL

## Why SQLx?

SQLx offers a unique combination:
- Write raw SQL (no ORM abstraction)
- Get compile-time checking
- Zero runtime cost for query parsing
- Works with existing databases

## Installation

```toml
[dependencies]
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid", "chrono", "migrate"] }
```

Common features:
- `postgres` / `mysql` / `sqlite` - Database driver
- `runtime-tokio` - Tokio async runtime
- `uuid` - UUID type support
- `chrono` - DateTime type support
- `migrate` - Database migrations

## Setup

### Environment Variable

```bash
export DATABASE_URL="postgres://user:password@localhost/mydb"
```

### Connection Pool

From `src/db/mod.rs`:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            // Maximum connections in pool
            .max_connections(100)
            // Timeout for acquiring a connection
            .acquire_timeout(Duration::from_secs(3))
            // Connect to database
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }
}
```

[Run conceptual example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=%2F%2F%20Conceptual%20example%20-%20SQLx%20requires%20database%20connection%0A%2F%2F%20This%20shows%20the%20pattern%20without%20actual%20connection%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Connection%20pool%20configuration%0A%20%20%20%20let%20config%20%3D%20PoolConfig%20%7B%0A%20%20%20%20%20%20%20%20max_connections%3A%20100%2C%0A%20%20%20%20%20%20%20%20acquire_timeout_secs%3A%203%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Pool%20config%3A%20max%3D%7B%7D%2C%20timeout%3D%7B%7Ds%22%2C%0A%20%20%20%20%20%20%20%20config.max_connections%2C%0A%20%20%20%20%20%20%20%20config.acquire_timeout_secs)%3B%0A%7D%0A%0Astruct%20PoolConfig%20%7B%0A%20%20%20%20max_connections%3A%20u32%2C%0A%20%20%20%20acquire_timeout_secs%3A%20u64%2C%0A%7D)

## Basic Queries

### Fetching Data

```rust
// Fetch one row (returns Option)
let user: Option<User> = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE id = $1"
)
.bind(user_id)
.fetch_optional(&pool)
.await?;

// Fetch one row (returns Error if not found)
let user: User = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE id = $1"
)
.bind(user_id)
.fetch_one(&pool)
.await?;

// Fetch all rows
let users: Vec<User> = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE active = $1"
)
.bind(true)
.fetch_all(&pool)
.await?;
```

### Inserting Data

```rust
// Insert and return the inserted row
let user: User = sqlx::query_as::<_, User>(
    r#"
    INSERT INTO users (name, email)
    VALUES ($1, $2)
    RETURNING *
    "#
)
.bind(&name)
.bind(&email)
.fetch_one(&pool)
.await?;

// Insert without returning
sqlx::query(
    "INSERT INTO logs (message, level) VALUES ($1, $2)"
)
.bind(&message)
.bind(&level)
.execute(&pool)
.await?;
```

### Updating Data

```rust
let result = sqlx::query(
    "UPDATE users SET name = $1 WHERE id = $2"
)
.bind(&new_name)
.bind(user_id)
.execute(&pool)
.await?;

println!("Rows affected: {}", result.rows_affected());
```

### Deleting Data

```rust
sqlx::query("DELETE FROM users WHERE id = $1")
    .bind(user_id)
    .execute(&pool)
    .await?;
```

## Real Examples from MetaMCP

### Model Definition with FromRow

From `src/db/models/api_key.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub encrypted_key: Vec<u8>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}
```

The `#[derive(FromRow)]` macro automatically maps database columns to struct fields.

### Repository Pattern

From `src/db/repositories/api_key.rs`:

```rust
use sqlx::PgPool;

pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        name: &str,
        key_hash: &str,
        encrypted_key: Vec<u8>,
    ) -> AppResult<ApiKey> {
        let api_key = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (name, key_hash, encrypted_key, is_active, created_at)
            VALUES ($1, $2, $3, true, NOW())
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(key_hash)
        .bind(encrypted_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(api_key)
    }

    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<ApiKey>> {
        let api_key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(api_key)
    }

    pub async fn find_by_hash(&self, key_hash: &str) -> AppResult<Option<ApiKey>> {
        let api_key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true"
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(api_key)
    }

    pub async fn list_all(&self, include_inactive: bool) -> AppResult<Vec<ApiKey>> {
        let keys = if include_inactive {
            sqlx::query_as::<_, ApiKey>(
                "SELECT * FROM api_keys ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ApiKey>(
                "SELECT * FROM api_keys WHERE is_active = true ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(keys)
    }

    pub async fn update_last_used(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE api_keys SET last_used_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn set_active(&self, id: Uuid, active: bool) -> AppResult<()> {
        sqlx::query(
            "UPDATE api_keys SET is_active = $1 WHERE id = $2"
        )
        .bind(active)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
```

### Running Migrations

From `src/db/mod.rs`:

```rust
impl Database {
    pub async fn run_migrations(&self) -> Result<(), sqlx::migrate::MigrateError> {
        // Runs all pending migrations from ./migrations folder
        sqlx::migrate!("./migrations").run(&self.pool).await
    }
}
```

Migration files are typically named:
```
migrations/
├── 20240101000000_create_api_keys.sql
├── 20240102000000_create_mcp_servers.sql
└── 20240103000000_add_indexes.sql
```

## Compile-Time Verification

SQLx can verify queries at compile time using the `query!` macro:

```rust
// This is checked against your actual database at compile time!
let user = sqlx::query!(
    r#"
    SELECT id, name, email
    FROM users
    WHERE id = $1
    "#,
    user_id
)
.fetch_one(&pool)
.await?;

// Fields are typed based on database schema
let id: i32 = user.id;
let name: String = user.name;
let email: Option<String> = user.email;  // Nullable column
```

### Setting Up Compile-Time Checking

1. Set `DATABASE_URL` environment variable
2. Run `cargo sqlx prepare` to generate query metadata
3. Commit `.sqlx` folder to version control

```bash
# Generate query metadata for offline builds
cargo sqlx prepare

# Check queries without database
cargo sqlx prepare --check
```

## Transactions

```rust
// Using transaction
let mut tx = pool.begin().await?;

sqlx::query("INSERT INTO users (name) VALUES ($1)")
    .bind("Alice")
    .execute(&mut *tx)
    .await?;

sqlx::query("INSERT INTO profiles (user_id, bio) VALUES ($1, $2)")
    .bind(user_id)
    .bind("Hello!")
    .execute(&mut *tx)
    .await?;

// Commit the transaction
tx.commit().await?;

// Or rollback on error (automatic if tx is dropped without commit)
```

[Run transaction pattern](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=%2F%2F%20Transaction%20pattern%20demonstration%0A%2F%2F%20Actual%20SQLx%20requires%20database%20connection%0A%0Astruct%20MockTransaction%20%7B%0A%20%20%20%20committed%3A%20bool%2C%0A%7D%0A%0Aimpl%20MockTransaction%20%7B%0A%20%20%20%20fn%20new()%20-%3E%20Self%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Transaction%20started%22)%3B%0A%20%20%20%20%20%20%20%20Self%20%7B%20committed%3A%20false%20%7D%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20fn%20execute(%26self%2C%20query%3A%20%26str)%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Executing%3A%20%7B%7D%22%2C%20query)%3B%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20fn%20commit(%26mut%20self)%20%7B%0A%20%20%20%20%20%20%20%20self.committed%20%3D%20true%3B%0A%20%20%20%20%20%20%20%20println!(%22Transaction%20committed%22)%3B%0A%20%20%20%20%7D%0A%7D%0A%0Aimpl%20Drop%20for%20MockTransaction%20%7B%0A%20%20%20%20fn%20drop(%26mut%20self)%20%7B%0A%20%20%20%20%20%20%20%20if%20!self.committed%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20println!(%22Transaction%20rolled%20back%20(not%20committed)%22)%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Successful%20transaction%0A%20%20%20%20%7B%0A%20%20%20%20%20%20%20%20let%20mut%20tx%20%3D%20MockTransaction%3A%3Anew()%3B%0A%20%20%20%20%20%20%20%20tx.execute(%22INSERT%20INTO%20users%20...%22)%3B%0A%20%20%20%20%20%20%20%20tx.execute(%22INSERT%20INTO%20profiles%20...%22)%3B%0A%20%20%20%20%20%20%20%20tx.commit()%3B%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20println!(%22%22)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Failed%20transaction%20(automatic%20rollback)%0A%20%20%20%20%7B%0A%20%20%20%20%20%20%20%20let%20tx%20%3D%20MockTransaction%3A%3Anew()%3B%0A%20%20%20%20%20%20%20%20tx.execute(%22INSERT%20INTO%20users%20...%22)%3B%0A%20%20%20%20%20%20%20%20%2F%2F%20Error%20occurs%2C%20tx%20dropped%20without%20commit%0A%20%20%20%20%7D%0A%7D)

## Advanced Patterns

### Dynamic Queries

```rust
use sqlx::QueryBuilder;

pub async fn search_users(
    pool: &PgPool,
    name_filter: Option<&str>,
    active_only: bool,
    limit: i64,
) -> Result<Vec<User>, sqlx::Error> {
    let mut builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1");

    if let Some(name) = name_filter {
        builder.push(" AND name ILIKE ");
        builder.push_bind(format!("%{}%", name));
    }

    if active_only {
        builder.push(" AND is_active = true");
    }

    builder.push(" ORDER BY created_at DESC LIMIT ");
    builder.push_bind(limit);

    let query = builder.build_query_as::<User>();
    query.fetch_all(pool).await
}
```

### Streaming Results

```rust
use futures::TryStreamExt;

// For large result sets, stream instead of loading all at once
let mut stream = sqlx::query_as::<_, User>("SELECT * FROM users")
    .fetch(&pool);

while let Some(user) = stream.try_next().await? {
    process_user(user);
}
```

### Custom Type Mapping

```rust
use sqlx::{Type, Decode, Encode};

// For PostgreSQL enums
#[derive(Debug, Type)]
#[sqlx(type_name = "user_status")]  // Must match PostgreSQL type name
pub enum UserStatus {
    Active,
    Inactive,
    Pending,
}

// Use in queries
let users: Vec<User> = sqlx::query_as(
    "SELECT * FROM users WHERE status = $1"
)
.bind(UserStatus::Active)
.fetch_all(&pool)
.await?;
```

## Database Migrations

### Creating Migrations

```bash
# Create a new migration
sqlx migrate add create_users_table
```

This creates `migrations/20240101120000_create_users_table.sql`:

```sql
-- Add migration script here
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
```

### Running Migrations

```bash
# Run all pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

Or programmatically:
```rust
sqlx::migrate!("./migrations").run(&pool).await?;
```

## Best Practices

### DO

1. **Use connection pooling** - Never create connections per request
2. **Use `query_as` with `FromRow`** - Type-safe result mapping
3. **Use transactions** - For multi-statement operations
4. **Use migrations** - Version control your schema
5. **Use `RETURNING *`** - Get inserted/updated data efficiently

### DON'T

1. **Don't interpolate strings** - Use `.bind()` to prevent SQL injection
2. **Don't fetch more than needed** - Use `LIMIT` and pagination
3. **Don't ignore connection limits** - Configure pool size appropriately
4. **Don't block on DB operations** - Always use async

### SQL Injection Prevention

```rust
// WRONG - SQL injection vulnerability!
let query = format!("SELECT * FROM users WHERE name = '{}'", user_input);
sqlx::query(&query).fetch_all(&pool).await?;

// CORRECT - Use parameter binding
sqlx::query("SELECT * FROM users WHERE name = $1")
    .bind(&user_input)
    .fetch_all(&pool)
    .await?;
```

## Pros and Cons

### Pros

| Advantage | Description |
|-----------|-------------|
| **Type-safe** | Compile-time query verification |
| **No ORM overhead** | Write raw SQL, no abstraction penalty |
| **Async-native** | Built for async from the start |
| **Multiple DBs** | PostgreSQL, MySQL, SQLite, MSSQL |
| **Migrations** | Built-in migration support |

### Cons

| Disadvantage | Description |
|--------------|-------------|
| **Compile time** | Query checking adds to build time |
| **Setup complexity** | Need DATABASE_URL for compile-time checks |
| **Raw SQL** | No query builder (by design) |
| **Learning curve** | Must know SQL well |

## When to Use SQLx

**Use SQLx when:**
- You're comfortable with SQL
- You want compile-time safety
- You need async database access
- You want to avoid ORM overhead

**Consider alternatives when:**
- You want a full ORM (use Diesel or SeaORM)
- You need complex query building (use Diesel)
- You're doing simple CRUD only (any ORM)

## Further Learning

### Official Resources
- [SQLx Documentation](https://docs.rs/sqlx)
- [SQLx GitHub](https://github.com/launchbadge/sqlx)
- [SQLx Examples](https://github.com/launchbadge/sqlx/tree/main/examples)

### Practice
1. Set up a PostgreSQL database
2. Create and run migrations
3. Implement a CRUD repository
4. Use transactions for multi-step operations

## Related Crates

- **sqlx-cli** - Command-line tool for migrations
- **diesel** - Full ORM alternative
- **sea-orm** - Active Record ORM built on SQLx
- **deadpool-postgres** - Alternative connection pooling
