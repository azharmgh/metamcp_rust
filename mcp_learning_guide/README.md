# MCP Protocol Learning Guide for Rust Developers

Welcome to the Model Context Protocol (MCP) learning guide! This guide is designed for developers who are new to both Rust and the MCP protocol.

## What is MCP?

The **Model Context Protocol (MCP)** is an open protocol that enables AI assistants (like Claude) to securely connect to external data sources and tools. Think of it as a standardized way for AI to interact with the outside world.

```
┌─────────────┐     MCP Protocol      ┌─────────────┐
│   Claude    │◄────────────────────►│  MCP Server │
│  (Client)   │   JSON-RPC over HTTP  │   (Tools)   │
└─────────────┘                       └─────────────┘
```

## Why MCP?

| Without MCP | With MCP |
|-------------|----------|
| Each AI needs custom integrations | Standardized protocol for all |
| Security is ad-hoc | Built-in authentication patterns |
| No discoverability | Tools, resources, prompts are discoverable |
| Tight coupling | Loose coupling via protocol |

## Learning Path

Follow these guides in order:

| # | Guide | Description | Difficulty |
|---|-------|-------------|------------|
| 1 | [Introduction to MCP](./01-introduction.md) | Core concepts and architecture | Beginner |
| 2 | [JSON-RPC Basics](./02-json-rpc-basics.md) | The foundation protocol | Beginner |
| 3 | [Protocol Types](./03-protocol-types.md) | Requests, responses, notifications | Beginner |
| 4 | [MCP Capabilities](./04-capabilities.md) | Tools, resources, and prompts | Intermediate |
| 5 | [HTTP Transport](./05-http-transport.md) | SSE and Streamable HTTP | Intermediate |
| 6 | [Authentication](./06-authentication.md) | JWT tokens and API keys | Intermediate |
| 7 | [Building an MCP Server](./07-building-server.md) | Step-by-step implementation | Advanced |
| 8 | [Best Practices](./08-best-practices.md) | Dos, don'ts, and patterns | Advanced |

## Prerequisites

- Basic programming knowledge
- Rust installed ([rustup.rs](https://rustup.rs))
- Familiarity with HTTP and JSON

## Quick Reference

### MCP Protocol Version
```
2025-03-26
```

### Key Endpoints
```
POST /mcp          - JSON-RPC requests
GET  /mcp          - SSE stream for server messages
GET  /mcp/health   - Health check
```

### Essential Crates
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
```

## Running Examples

All code examples in this guide can be run in the [Rust Playground](https://play.rust-lang.org). Look for the "Run in Playground" links.

For examples that require external dependencies or networking, clone this repository:

```bash
git clone https://github.com/azharmgh/metamcp_rust.git
cd metamcp_rust
cargo run --example backend_server_1
```

## Repository Structure

```
metamcp_rust/
├── src/
│   ├── mcp/
│   │   ├── protocol/types.rs   # MCP type definitions
│   │   ├── proxy.rs            # MCP client/proxy
│   │   └── server_manager.rs   # Server management
│   └── api/
│       └── handlers/
│           └── mcp_gateway.rs  # MCP HTTP handler
└── examples/
    ├── backend_server_1.rs     # Simple MCP server
    └── backend_server_2.rs     # Advanced MCP server
```

## Getting Help

- [MCP Specification](https://modelcontextprotocol.io/specification)
- [Rust Book](https://doc.rust-lang.org/book/)
- [This Repository Issues](https://github.com/azharmgh/metamcp_rust/issues)

---

Let's start with [Introduction to MCP](./01-introduction.md)!
