# Rust Learning Guide for MetaMCP

Welcome to the Rust Learning Guide! This guide is designed for developers who are new to Rust and want to learn by exploring real-world examples from the MetaMCP project.

## About This Guide

This learning guide covers all the major dependencies used in the MetaMCP project - a production-grade MCP (Model Context Protocol) proxy server written in Rust. Each guide includes:

- Explanation of what the dependency does
- Real code examples from this repository
- Runnable examples in Rust Playground
- Best practices and when to use/avoid the dependency
- Pros and cons
- Further learning resources

## Prerequisites

Before diving in, you should have:
- Basic understanding of programming concepts
- Rust installed on your system (`rustup` recommended)
- Familiarity with command line tools

## Table of Contents

### Core Async Runtime
1. [Tokio - Async Runtime](./01_tokio.md) - The foundation for async Rust
2. [Async Utilities (futures, tokio-stream, async-trait)](./02_async_utilities.md) - Working with async patterns

### Web Framework
3. [Axum - Web Framework](./03_axum.md) - Building web APIs
4. [Tower & Tower-HTTP - Middleware](./04_tower.md) - Request/response middleware

### Data Handling
5. [Serde & Serde JSON - Serialization](./05_serde.md) - JSON and data serialization
6. [SQLx - Database Access](./06_sqlx.md) - Type-safe SQL queries
7. [Validator - Input Validation](./07_validator.md) - Request validation

### Security & Authentication
8. [Authentication (jsonwebtoken, argon2, chacha20poly1305)](./08_authentication.md) - Security patterns

### Error Handling
9. [Error Handling (thiserror, anyhow)](./09_error_handling.md) - Idiomatic error management

### Observability
10. [Tracing - Logging & Observability](./10_tracing.md) - Structured logging

### HTTP Client
11. [Reqwest - HTTP Client](./11_reqwest.md) - Making HTTP requests

### Utility Crates
12. [Utility Crates (uuid, chrono, dotenvy, hex)](./12_utilities.md) - Common utilities

### CLI & Documentation
13. [Clap - Command Line Parsing](./13_clap.md) - Building CLI tools
14. [Utoipa - OpenAPI Documentation](./14_utoipa.md) - API documentation

## Recommended Learning Path

For beginners, we recommend following this order:

### Week 1: Foundations
1. Start with **Tokio** to understand async programming in Rust
2. Learn **Serde** for data serialization (used everywhere)
3. Study **Error Handling** patterns

### Week 2: Web Development
4. Explore **Axum** for building web APIs
5. Learn **Tower** middleware concepts
6. Understand **Tracing** for logging

### Week 3: Data & Security
7. Study **SQLx** for database operations
8. Learn **Authentication** patterns
9. Explore **Validator** for input validation

### Week 4: Advanced Topics
10. Master **Async Utilities** for complex async patterns
11. Build CLI tools with **Clap**
12. Document APIs with **Utoipa**
13. Use **Reqwest** for HTTP clients
14. Learn utility crates

## How to Use This Guide

### Reading Code Examples

Each guide contains code examples from this repository. Look for:

```rust
// File: src/example.rs (Lines 10-25)
// This comment tells you where to find the original code
```

### Running Examples in Rust Playground

Most examples include a link like:

> [Run in Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=fn%20main()%20%7B%7D)

Click these links to experiment with the code interactively.

### Testing Locally

To test examples from this project:

```bash
# Clone the repository
git clone https://github.com/azharmgh/metamcp_rust

# Run tests
cargo test

# Run specific examples
cargo run --example test_client
```

## Project Structure

Understanding the MetaMCP project structure helps contextualize the examples:

```
metamcp_rust/
├── src/
│   ├── main.rs           # Application entry point
│   ├── api/              # HTTP routes and handlers
│   ├── auth/             # Authentication logic
│   ├── config/           # Configuration management
│   ├── db/               # Database access layer
│   ├── mcp/              # MCP protocol implementation
│   ├── streaming/        # Event streaming
│   └── utils/            # Utilities and error handling
├── examples/             # Example programs
├── tests/                # Integration tests
├── migrations/           # Database migrations
└── rust_learning_guide/  # This guide!
```

## Key Concepts

Before diving into specific crates, understand these Rust concepts:

### Ownership & Borrowing
Rust's memory safety comes from its ownership system. You'll see patterns like:
- `&self` - borrowing
- `self` - taking ownership
- `Clone` - explicit copying
- `Arc<T>` - shared ownership across threads

### Traits
Traits define shared behavior. Key traits you'll encounter:
- `Clone`, `Debug`, `Default`
- `Serialize`, `Deserialize` (from Serde)
- `FromRow` (from SQLx)
- `Error` (from thiserror)

### Async/Await
Most code in this project is async. Key patterns:
- `async fn` - async function
- `.await` - waiting for async operations
- `tokio::spawn` - spawning background tasks

## Getting Help

- [The Rust Book](https://doc.rust-lang.org/book/) - Official Rust learning resource
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Learn through examples
- [Rust Discord](https://discord.gg/rust-lang) - Community help
- [This Week in Rust](https://this-week-in-rust.org/) - Weekly newsletter

## Contributing

Found an error or want to improve the guide? Contributions are welcome!

---

Happy learning! Rust has a steep learning curve, but understanding these dependencies will give you a solid foundation for building production-grade applications.
