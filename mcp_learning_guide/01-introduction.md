# Introduction to MCP

## What is the Model Context Protocol?

MCP (Model Context Protocol) is an open standard that allows AI assistants to securely interact with external systems. It defines how AI clients can:

- **Discover** available tools, resources, and prompts
- **Execute** tools with parameters
- **Read** resources (files, configs, data)
- **Use** prompt templates

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                         AI Client                             │
│                    (Claude, GPT, etc.)                        │
└──────────────────────────┬───────────────────────────────────┘
                           │ MCP Protocol
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                      MCP Gateway                              │
│                 (Aggregates multiple servers)                 │
└───────┬──────────────────┬──────────────────┬────────────────┘
        │                  │                  │
        ▼                  ▼                  ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│  MCP Server 1 │  │  MCP Server 2 │  │  MCP Server 3 │
│   (Database)  │  │    (APIs)     │  │   (Files)     │
└───────────────┘  └───────────────┘  └───────────────┘
```

## Core Concepts

### 1. Clients and Servers

| Component | Role | Example |
|-----------|------|---------|
| **Client** | Initiates requests, consumes capabilities | Claude, your AI app |
| **Server** | Exposes capabilities, handles requests | Your Rust backend |
| **Gateway** | Aggregates multiple servers | MetaMCP |

### 2. Capabilities

MCP servers expose three types of capabilities:

```rust
// From src/mcp/protocol/types.rs
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,      // Executable functions
    pub resources: Option<ResourcesCapability>, // Readable data
    pub prompts: Option<PromptsCapability>,  // Template prompts
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=a1b2c3d4e5f6) - *Note: Simplified version*

### 3. The Handshake

Every MCP session starts with a handshake:

```
Client                              Server
  │                                   │
  │──── initialize (request) ────────►│
  │                                   │
  │◄─── InitializeResult ─────────────│
  │                                   │
  │──── initialized (notification) ──►│
  │                                   │
  │         Session Active            │
  │                                   │
```

## Why Use MCP?

### Pros

| Advantage | Description |
|-----------|-------------|
| **Standardization** | One protocol, many integrations |
| **Discoverability** | Clients can discover what servers offer |
| **Security** | Built-in authentication patterns |
| **Flexibility** | Tools, resources, prompts - cover all use cases |
| **Language Agnostic** | JSON-RPC works with any language |

### Cons

| Disadvantage | Mitigation |
|--------------|------------|
| **Complexity** | Start with simple tools, add complexity later |
| **Overhead** | HTTP/JSON overhead for simple ops | Use efficient serialization |
| **Learning Curve** | Follow this guide step by step |

## MCP vs REST vs GraphQL

| Feature | MCP | REST | GraphQL |
|---------|-----|------|---------|
| **Purpose** | AI-tool integration | General APIs | Flexible queries |
| **Discovery** | Built-in | External (OpenAPI) | Introspection |
| **Bidirectional** | Yes (SSE) | No | Subscriptions |
| **Schema** | JSON-RPC | Varies | Strong typing |

## Simple Example: What MCP Communication Looks Like

```rust
// A simple MCP request to call a tool
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: i64,
    method: String,
    params: Option<serde_json::Value>,
}

fn main() {
    // Client sends this to call the "add" tool
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "add",
            "arguments": {
                "a": 5,
                "b": 3
            }
        })),
    };

    println!("{}", serde_json::to_string_pretty(&request).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3Ajson%3B%0A%0A%23%5Bderive(Serialize%2C%20Deserialize)%5D%0Astruct%20JsonRpcRequest%20%7B%0A%20%20%20%20jsonrpc%3A%20String%2C%0A%20%20%20%20id%3A%20i64%2C%0A%20%20%20%20method%3A%20String%2C%0A%20%20%20%20params%3A%20Option%3Cserde_json%3A%3AValue%3E%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20request%20%3D%20JsonRpcRequest%20%7B%0A%20%20%20%20%20%20%20%20jsonrpc%3A%20%222.0%22.to_string()%2C%0A%20%20%20%20%20%20%20%20id%3A%201%2C%0A%20%20%20%20%20%20%20%20method%3A%20%22tools%2Fcall%22.to_string()%2C%0A%20%20%20%20%20%20%20%20params%3A%20Some(json!(%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22name%22%3A%20%22add%22%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%22arguments%22%3A%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22a%22%3A%205%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22b%22%3A%203%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20%7D))%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26request).unwrap())%3B%0A%7D)

**Output:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "add",
    "arguments": {
      "a": 5,
      "b": 3
    }
  }
}
```

## Key Takeaways

1. **MCP is a protocol**, not a library - you implement it
2. **JSON-RPC 2.0** is the message format
3. **Three capabilities**: Tools (actions), Resources (data), Prompts (templates)
4. **Handshake first**: Always initialize before using capabilities
5. **HTTP transport**: POST for requests, GET for SSE streams

---

Next: [JSON-RPC Basics](./02-json-rpc-basics.md)
