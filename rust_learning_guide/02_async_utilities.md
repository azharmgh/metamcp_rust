# Async Utilities (futures, tokio-stream, async-trait)

This guide covers three important crates that extend Rust's async capabilities: `futures`, `tokio-stream`, and `async-trait`.

## Overview

| Crate | Purpose |
|-------|---------|
| **futures** | Core async abstractions and utilities |
| **tokio-stream** | Stream adapters for Tokio |
| **async-trait** | Async methods in traits |

## Installation

```toml
[dependencies]
futures = "0.3"
tokio-stream = "0.1"
async-trait = "0.1"
```

---

## futures Crate

The `futures` crate provides foundational async abstractions used throughout the Rust async ecosystem.

### Key Concepts

#### Future Trait
A `Future` represents a value that will be available eventually:

```rust
use std::future::Future;

// Any async fn returns an impl Future
async fn get_data() -> String {
    "Hello".to_string()
}
```

#### Stream Trait
A `Stream` is like an async iterator - it yields multiple values over time:

```rust
use futures::Stream;

// Streams yield items asynchronously
// Similar to Iterator but with async
```

### Common Utilities

#### FutureExt - Future Extensions

```rust
use futures::FutureExt;

async fn example() {
    // Map the result
    let future = async { 42 };
    let doubled = future.map(|x| x * 2).await;

    // Box the future for dynamic dispatch
    let boxed: Pin<Box<dyn Future<Output = i32>>> =
        async { 42 }.boxed();
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20futures%3A%3AFutureExt%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Map%20transforms%20the%20result%0A%20%20%20%20let%20result%20%3D%20async%20%7B%2042%20%7D.map(%7Cx%7C%20x%20*%202).await%3B%0A%20%20%20%20println!(%22Result%3A%20%7B%7D%22%2C%20result)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Then%20chains%20futures%0A%20%20%20%20let%20chained%20%3D%20async%20%7B%2010%20%7D%0A%20%20%20%20%20%20%20%20.then(%7Cx%7C%20async%20move%20%7B%20x%20%2B%205%20%7D)%0A%20%20%20%20%20%20%20%20.await%3B%0A%20%20%20%20println!(%22Chained%3A%20%7B%7D%22%2C%20chained)%3B%0A%7D)

#### StreamExt - Stream Extensions

```rust
use futures::StreamExt;

async fn process_stream() {
    let stream = futures::stream::iter(vec![1, 2, 3, 4, 5]);

    // Collect all items
    let items: Vec<i32> = stream.collect().await;

    // Process items with for_each
    let stream = futures::stream::iter(vec![1, 2, 3]);
    stream.for_each(|item| async move {
        println!("Got: {}", item);
    }).await;
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20futures%3A%3AStreamExt%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20%2F%2F%20Create%20a%20stream%20from%20an%20iterator%0A%20%20%20%20let%20stream%20%3D%20futures%3A%3Astream%3A%3Aiter(1..%3D5)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Map%20each%20item%0A%20%20%20%20let%20doubled%3A%20Vec%3Ci32%3E%20%3D%20stream%0A%20%20%20%20%20%20%20%20.map(%7Cx%7C%20x%20*%202)%0A%20%20%20%20%20%20%20%20.collect()%0A%20%20%20%20%20%20%20%20.await%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Doubled%3A%20%7B%3A%3F%7D%22%2C%20doubled)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Filter%20items%0A%20%20%20%20let%20stream%20%3D%20futures%3A%3Astream%3A%3Aiter(1..%3D10)%3B%0A%20%20%20%20let%20evens%3A%20Vec%3Ci32%3E%20%3D%20stream%0A%20%20%20%20%20%20%20%20.filter(%7Cx%7C%20futures%3A%3Afuture%3A%3Aready(x%20%25%202%20%3D%3D%200))%0A%20%20%20%20%20%20%20%20.collect()%0A%20%20%20%20%20%20%20%20.await%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Evens%3A%20%7B%3A%3F%7D%22%2C%20evens)%3B%0A%7D)

#### join! and try_join!

Run multiple futures concurrently:

```rust
use futures::join;
use futures::try_join;

async fn example() -> Result<(), Error> {
    // Run all futures, wait for all to complete
    let (a, b, c) = join!(
        fetch_a(),
        fetch_b(),
        fetch_c()
    );

    // For Result-returning futures, fail fast on first error
    let (a, b, c) = try_join!(
        fetch_a_result(),
        fetch_b_result(),
        fetch_c_result()
    )?;

    Ok(())
}
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20futures%3A%3Ajoin%3B%0Ause%20std%3A%3Atime%3A%3ADuration%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20let%20start%20%3D%20std%3A%3Atime%3A%3AInstant%3A%3Anow()%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20All%20run%20concurrently%0A%20%20%20%20let%20(a%2C%20b%2C%20c)%20%3D%20join!(%0A%20%20%20%20%20%20%20%20async%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(100)).await%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22A%22%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20async%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(200)).await%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22B%22%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20async%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20tokio%3A%3Atime%3A%3Asleep(Duration%3A%3Afrom_millis(150)).await%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22C%22%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20)%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Results%3A%20%7B%7D%2C%20%7B%7D%2C%20%7B%7D%22%2C%20a%2C%20b%2C%20c)%3B%0A%20%20%20%20println!(%22Total%20time%3A%20%7B%3A%3F%7D%20(~200ms%2C%20not%20~450ms)%22%2C%20start.elapsed())%3B%0A%7D)

---

## tokio-stream Crate

`tokio-stream` provides utilities for working with streams in Tokio applications.

### Creating Streams

```rust
use tokio_stream::{self as stream, StreamExt};

#[tokio::main]
async fn main() {
    // From an iterator
    let mut stream = stream::iter(vec![1, 2, 3]);

    while let Some(item) = stream.next().await {
        println!("Got: {}", item);
    }
}
```

### Stream from Channel Receiver

From `src/streaming/manager.rs`:

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

// Create MPSC channel
let (tx, rx) = mpsc::channel(256);

// Convert receiver to a stream
let stream = ReceiverStream::new(rx);
```

[Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20tokio%3A%3Async%3A%3Ampsc%3B%0Ause%20tokio_stream%3A%3Awrappers%3A%3AReceiverStream%3B%0Ause%20tokio_stream%3A%3AStreamExt%3B%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20let%20(tx%2C%20rx)%20%3D%20mpsc%3A%3Achannel(32)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Spawn%20producer%0A%20%20%20%20tokio%3A%3Aspawn(async%20move%20%7B%0A%20%20%20%20%20%20%20%20for%20i%20in%201..%3D5%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20tx.send(i).await.unwrap()%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Convert%20to%20stream%0A%20%20%20%20let%20mut%20stream%20%3D%20ReceiverStream%3A%3Anew(rx)%3B%0A%20%20%20%20%0A%20%20%20%20%2F%2F%20Process%20as%20stream%0A%20%20%20%20while%20let%20Some(item)%20%3D%20stream.next().await%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Received%3A%20%7B%7D%22%2C%20item)%3B%0A%20%20%20%20%7D%0A%7D)

### Stream Adapters

```rust
use tokio_stream::StreamExt;

async fn process() {
    let stream = tokio_stream::iter(1..=100);

    // Take first 10
    let first_ten: Vec<_> = stream.take(10).collect().await;

    // With timeout per item
    let stream = tokio_stream::iter(vec![1, 2, 3]);
    let mut timed = stream.timeout(Duration::from_secs(1));

    while let Some(result) = timed.next().await {
        match result {
            Ok(item) => println!("Got: {}", item),
            Err(_) => println!("Timeout!"),
        }
    }
}
```

---

## async-trait Crate

Rust doesn't natively support async methods in traits. The `async-trait` macro provides this functionality.

### The Problem

```rust
// This doesn't compile in standard Rust!
trait MyTrait {
    async fn do_something(&self) -> Result<(), Error>;
}
```

### The Solution

```rust
use async_trait::async_trait;

#[async_trait]
trait MyTrait {
    async fn do_something(&self) -> Result<(), Error>;
}

#[async_trait]
impl MyTrait for MyStruct {
    async fn do_something(&self) -> Result<(), Error> {
        // Async implementation
        Ok(())
    }
}
```

### Real Example from MetaMCP

From `src/auth/middleware.rs` - implementing Axum's `FromRequestParts`:

```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct AuthenticatedUser {
    pub claims: Claims,
}

// Axum extractors need FromRequestParts which has an async method
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    // This is an async fn in a trait - works because Axum uses async-trait internally
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Get claims from request extensions (set by middleware)
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(|claims| AuthenticatedUser { claims })
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))
    }
}
```

### Creating Your Own Async Trait

```rust
use async_trait::async_trait;

// Define the trait
#[async_trait]
trait Repository {
    async fn find_by_id(&self, id: u64) -> Option<Entity>;
    async fn save(&self, entity: &Entity) -> Result<(), Error>;
    async fn delete(&self, id: u64) -> Result<(), Error>;
}

// Implement for a specific type
struct PostgresRepository {
    pool: PgPool,
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn find_by_id(&self, id: u64) -> Option<Entity> {
        sqlx::query_as("SELECT * FROM entities WHERE id = $1")
            .bind(id as i64)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
    }

    async fn save(&self, entity: &Entity) -> Result<(), Error> {
        sqlx::query("INSERT INTO entities (name) VALUES ($1)")
            .bind(&entity.name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: u64) -> Result<(), Error> {
        sqlx::query("DELETE FROM entities WHERE id = $1")
            .bind(id as i64)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
```

[Run simplified example in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20async_trait%3A%3Aasync_trait%3B%0A%0A%2F%2F%20Define%20an%20async%20trait%0A%23%5Basync_trait%5D%0Atrait%20DataFetcher%20%7B%0A%20%20%20%20async%20fn%20fetch(%26self%2C%20id%3A%20u64)%20-%3E%20Option%3CString%3E%3B%0A%7D%0A%0A%2F%2F%20Implement%20for%20a%20struct%0Astruct%20MockFetcher%3B%0A%0A%23%5Basync_trait%5D%0Aimpl%20DataFetcher%20for%20MockFetcher%20%7B%0A%20%20%20%20async%20fn%20fetch(%26self%2C%20id%3A%20u64)%20-%3E%20Option%3CString%3E%20%7B%0A%20%20%20%20%20%20%20%20%2F%2F%20Simulate%20async%20work%0A%20%20%20%20%20%20%20%20tokio%3A%3Atime%3A%3Asleep(std%3A%3Atime%3A%3ADuration%3A%3Afrom_millis(10)).await%3B%0A%20%20%20%20%20%20%20%20Some(format!(%22Data%20for%20%7B%7D%22%2C%20id))%0A%20%20%20%20%7D%0A%7D%0A%0A%23%5Btokio%3A%3Amain%5D%0Aasync%20fn%20main()%20%7B%0A%20%20%20%20let%20fetcher%20%3D%20MockFetcher%3B%0A%20%20%20%20%0A%20%20%20%20if%20let%20Some(data)%20%3D%20fetcher.fetch(42).await%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Fetched%3A%20%7B%7D%22%2C%20data)%3B%0A%20%20%20%20%7D%0A%7D)

### With ?Send Annotation

By default, async-trait requires futures to be `Send`. For single-threaded contexts:

```rust
#[async_trait(?Send)]
trait LocalOnlyTrait {
    async fn process(&self);
}
```

## Best Practices

### futures

| Do | Don't |
|----|-------|
| Use `join!` for concurrent operations | Use `join!` when order matters |
| Use `try_join!` for fallible operations | Ignore errors in concurrent operations |
| Use `StreamExt` for stream processing | Manually poll streams |

### tokio-stream

| Do | Don't |
|----|-------|
| Use `ReceiverStream` to convert channels | Create unnecessary intermediate collections |
| Use `timeout` for bounded waits | Let streams run indefinitely without bounds |
| Use `take` to limit stream items | Process infinite streams without limits |

### async-trait

| Do | Don't |
|----|-------|
| Use for trait objects with async methods | Use when you can avoid trait objects |
| Add `?Send` when futures don't need to be Send | Forget that it adds heap allocation |
| Keep async trait methods focused | Put heavy logic in trait methods |

## Pros and Cons

### futures

| Pros | Cons |
|------|------|
| Standard async abstractions | Large API surface |
| Widely used ecosystem | Can be confusing for beginners |
| Good stream utilities | Some overlap with tokio |

### tokio-stream

| Pros | Cons |
|------|------|
| Integrates well with Tokio | Tokio-specific |
| Useful wrappers for channels | Limited compared to futures-rs |
| Good timeout support | |

### async-trait

| Pros | Cons |
|------|------|
| Enables async methods in traits | Heap allocation overhead |
| Simple to use | Compile time increase |
| Well-maintained | May become unnecessary with future Rust |

## When to Use

**futures:**
- When you need stream combinators
- When you need `join!` or `try_join!`
- For core async trait implementations

**tokio-stream:**
- When converting Tokio channels to streams
- When you need stream timeouts
- For Tokio-specific stream utilities

**async-trait:**
- When defining traits with async methods
- When you need dynamic dispatch with async
- For repository/service patterns

## Further Learning

### futures
- [futures-rs Documentation](https://docs.rs/futures)
- [Async Book - Streams](https://rust-lang.github.io/async-book/05_streams/01_chapter.html)

### tokio-stream
- [tokio-stream Documentation](https://docs.rs/tokio-stream)
- [Tokio Tutorial - Streams](https://tokio.rs/tokio/tutorial/streams)

### async-trait
- [async-trait Documentation](https://docs.rs/async-trait)
- [Async Rust Patterns](https://rust-lang.github.io/async-book/)

## Related Crates

- **tokio** - The async runtime these crates build upon
- **pin-project** - Safe pin projections (used internally)
- **futures-util** - Additional future utilities
