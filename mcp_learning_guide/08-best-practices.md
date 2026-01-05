# MCP Best Practices

This guide covers dos and don'ts, common patterns, and anti-patterns for MCP implementation.

## Protocol Dos and Don'ts

### Requests and Responses

| Do | Don't |
|----|-------|
| Always include `jsonrpc: "2.0"` | Use other JSON-RPC versions |
| Match response `id` to request `id` | Generate new IDs for responses |
| Return `result` OR `error`, not both | Include both fields |
| Use standard error codes | Invent custom negative codes |

```rust
use serde_json::json;

// DO: Correct response
fn good_response(id: i64) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,  // Same as request
        "result": { "data": "success" }
    })
}

// DON'T: Wrong response
fn bad_response(id: i64) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id + 1,  // Wrong! Must match request
        "result": {},
        "error": {}    // Wrong! Can't have both
    })
}

fn main() {
    println!("Good: {}", serde_json::to_string_pretty(&good_response(1)).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3Ajson%3B%0A%0Afn%20good_response(id%3A%20i64)%20-%3E%20serde_json%3A%3AValue%20%7B%0A%20%20%20%20json!(%7B%0A%20%20%20%20%20%20%20%20%22jsonrpc%22%3A%20%222.0%22%2C%0A%20%20%20%20%20%20%20%20%22id%22%3A%20id%2C%0A%20%20%20%20%20%20%20%20%22result%22%3A%20%7B%20%22data%22%3A%20%22success%22%20%7D%0A%20%20%20%20%7D)%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26good_response(1)).unwrap())%3B%0A%7D)

### Notifications

| Do | Don't |
|----|-------|
| Return HTTP 202 for notifications | Return JSON response |
| Omit `id` field in notifications you send | Use `null` for id |
| Process notifications silently | Log errors for valid notifications |

```rust
fn handle_message(has_id: bool) -> &'static str {
    if has_id {
        // Request - must respond
        r#"{"jsonrpc":"2.0","id":1,"result":{}}"#
    } else {
        // Notification - no response body
        "HTTP 202 Accepted"
    }
}

fn main() {
    println!("Request response: {}", handle_message(true));
    println!("Notification response: {}", handle_message(false));
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=fn%20handle_message(has_id%3A%20bool)%20-%3E%20%26'static%20str%20%7B%0A%20%20%20%20if%20has_id%20%7B%0A%20%20%20%20%20%20%20%20r%23%22%7B%22jsonrpc%22%3A%222.0%22%2C%22id%22%3A1%2C%22result%22%3A%7B%7D%7D%22%23%0A%20%20%20%20%7D%20else%20%7B%0A%20%20%20%20%20%20%20%20%22HTTP%20202%20Accepted%22%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22Request%3A%20%7B%7D%22%2C%20handle_message(true))%3B%0A%20%20%20%20println!(%22Notification%3A%20%7B%7D%22%2C%20handle_message(false))%3B%0A%7D)

## Tool Design

### Good Tool Design

```rust
use serde_json::json;

// DO: Clear, focused, well-documented tool
fn good_tool() -> serde_json::Value {
    json!({
        "name": "send_email",
        "description": "Send an email to a recipient",
        "inputSchema": {
            "type": "object",
            "properties": {
                "to": {
                    "type": "string",
                    "description": "Recipient email address",
                    "format": "email"
                },
                "subject": {
                    "type": "string",
                    "description": "Email subject line",
                    "maxLength": 200
                },
                "body": {
                    "type": "string",
                    "description": "Email body content"
                }
            },
            "required": ["to", "subject", "body"]
        }
    })
}

// DON'T: Vague, undocumented, too broad
fn bad_tool() -> serde_json::Value {
    json!({
        "name": "do_stuff",  // Vague name
        "description": "",   // No description
        "inputSchema": {
            "type": "object"
            // No properties defined
        }
    })
}

fn main() {
    println!("Good:\n{}\n", serde_json::to_string_pretty(&good_tool()).unwrap());
    println!("Bad:\n{}", serde_json::to_string_pretty(&bad_tool()).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3Ajson%3B%0A%0Afn%20good_tool()%20-%3E%20serde_json%3A%3AValue%20%7B%0A%20%20%20%20json!(%7B%0A%20%20%20%20%20%20%20%20%22name%22%3A%20%22send_email%22%2C%0A%20%20%20%20%20%20%20%20%22description%22%3A%20%22Send%20an%20email%22%2C%0A%20%20%20%20%20%20%20%20%22inputSchema%22%3A%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22type%22%3A%20%22object%22%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%22properties%22%3A%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22to%22%3A%20%7B%22type%22%3A%22string%22%2C%22description%22%3A%22Email%20address%22%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22subject%22%3A%20%7B%22type%22%3A%22string%22%7D%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%22required%22%3A%20%5B%22to%22%2C%20%22subject%22%5D%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D)%0A%7D%0A%0Afn%20bad_tool()%20-%3E%20serde_json%3A%3AValue%20%7B%0A%20%20%20%20json!(%7B%22name%22%3A%20%22do_stuff%22%2C%20%22description%22%3A%20%22%22%2C%20%22inputSchema%22%3A%20%7B%7D%7D)%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22Good%3A%5Cn%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26good_tool()).unwrap())%3B%0A%7D)

### Tool Naming Conventions

| Pattern | Example | Use Case |
|---------|---------|----------|
| `verb_noun` | `send_email`, `create_user` | Actions |
| `get_noun` | `get_weather`, `get_file` | Read operations |
| `noun_verb` | `file_read`, `user_delete` | Resource-focused |

## Error Handling

### Proper Error Responses

```rust
use serde_json::json;

// Standard JSON-RPC error codes
const PARSE_ERROR: i32 = -32700;
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;

fn create_error(id: i64, code: i32, message: &str) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

// DO: Use appropriate error codes
fn good_error_handling(method: &str, id: i64) -> serde_json::Value {
    match method {
        "unknown" => create_error(id, METHOD_NOT_FOUND, "Method not found"),
        "bad_params" => create_error(id, INVALID_PARAMS, "Missing required parameter 'name'"),
        _ => create_error(id, INTERNAL_ERROR, "Unexpected error"),
    }
}

fn main() {
    println!("{}", serde_json::to_string_pretty(&good_error_handling("unknown", 1)).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3Ajson%3B%0A%0Aconst%20METHOD_NOT_FOUND%3A%20i32%20%3D%20-32601%3B%0Aconst%20INVALID_PARAMS%3A%20i32%20%3D%20-32602%3B%0Aconst%20INTERNAL_ERROR%3A%20i32%20%3D%20-32603%3B%0A%0Afn%20create_error(id%3A%20i64%2C%20code%3A%20i32%2C%20message%3A%20%26str)%20-%3E%20serde_json%3A%3AValue%20%7B%0A%20%20%20%20json!(%7B%22jsonrpc%22%3A%222.0%22%2C%22id%22%3Aid%2C%22error%22%3A%7B%22code%22%3Acode%2C%22message%22%3Amessage%7D%7D)%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26create_error(1%2C%20METHOD_NOT_FOUND%2C%20%22Not%20found%22)).unwrap())%3B%0A%7D)

### Tool Execution Errors

```rust
use serde_json::json;

// DO: Return isError: true for tool failures
fn tool_error(message: &str) -> serde_json::Value {
    json!({
        "content": [{
            "type": "text",
            "text": message
        }],
        "isError": true
    })
}

// DO: Return isError: false for success
fn tool_success(result: &str) -> serde_json::Value {
    json!({
        "content": [{
            "type": "text",
            "text": result
        }],
        "isError": false
    })
}

fn main() {
    println!("Error: {}", serde_json::to_string_pretty(&tool_error("File not found")).unwrap());
    println!("Success: {}", serde_json::to_string_pretty(&tool_success("Done")).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3Ajson%3B%0A%0Afn%20tool_error(message%3A%20%26str)%20-%3E%20serde_json%3A%3AValue%20%7B%0A%20%20%20%20json!(%7B%22content%22%3A%5B%7B%22type%22%3A%22text%22%2C%22text%22%3Amessage%7D%5D%2C%22isError%22%3Atrue%7D)%0A%7D%0A%0Afn%20tool_success(result%3A%20%26str)%20-%3E%20serde_json%3A%3AValue%20%7B%0A%20%20%20%20json!(%7B%22content%22%3A%5B%7B%22type%22%3A%22text%22%2C%22text%22%3Aresult%7D%5D%2C%22isError%22%3Afalse%7D)%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20println!(%22Error%3A%20%7B%7D%22%2C%20serde_json%3A%3Ato_string(&tool_error(%22Not%20found%22)).unwrap())%3B%0A%20%20%20%20println!(%22Success%3A%20%7B%7D%22%2C%20serde_json%3A%3Ato_string(&tool_success(%22Done%22)).unwrap())%3B%0A%7D)

## HTTP Transport Best Practices

### Headers

| Header | Requirement | Value |
|--------|-------------|-------|
| `Content-Type` | Required | `application/json` |
| `mcp-protocol-version` | Required | `2025-03-26` |
| `Cache-Control` | For SSE | `no-cache` |

### SSE Keep-Alive

```rust
// DO: Keep SSE connections alive
async fn sse_with_keepalive() {
    // Send ping every 30 seconds
    // This prevents connection timeouts
    loop {
        // send_ping().await;
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        println!("Sending keep-alive ping");
    }
}

// DON'T: Close SSE immediately after first message
async fn bad_sse() {
    // send_one_message().await;
    // Connection closes - client will fail!
    println!("Bad: SSE closed immediately");
}

#[tokio::main]
async fn main() {
    println!("SSE should stay open with periodic pings");
}
```

## Security Best Practices

### Token Handling

| Do | Don't |
|----|-------|
| Use short-lived tokens (15 min) | Use long-lived tokens (days) |
| Validate tokens on every request | Cache validation results |
| Use HTTPS in production | Use HTTP with sensitive data |
| Hash API keys with Argon2 | Store plain text keys |

### Input Validation

```rust
fn validate_tool_args(name: &str, args: &serde_json::Value) -> Result<(), String> {
    match name {
        "send_email" => {
            // Validate email format
            let to = args.get("to")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'to' field")?;

            if !to.contains('@') {
                return Err("Invalid email format".to_string());
            }

            // Validate subject length
            if let Some(subject) = args.get("subject").and_then(|v| v.as_str()) {
                if subject.len() > 200 {
                    return Err("Subject too long (max 200 chars)".to_string());
                }
            }

            Ok(())
        }
        _ => Ok(())
    }
}

fn main() {
    use serde_json::json;

    // Valid
    let result = validate_tool_args("send_email", &json!({"to": "user@example.com", "subject": "Hi"}));
    println!("Valid: {:?}", result);

    // Invalid email
    let result = validate_tool_args("send_email", &json!({"to": "invalid", "subject": "Hi"}));
    println!("Invalid email: {:?}", result);
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=fn%20validate_tool_args(name%3A%20%26str%2C%20args%3A%20%26serde_json%3A%3AValue)%20-%3E%20Result%3C()%2C%20String%3E%20%7B%0A%20%20%20%20match%20name%20%7B%0A%20%20%20%20%20%20%20%20%22send_email%22%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20to%20%3D%20args.get(%22to%22).and_then(%7Cv%7C%20v.as_str()).ok_or(%22Missing%20'to'%22)%3F%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20if%20!to.contains('%40')%20%7B%20return%20Err(%22Invalid%20email%22.to_string())%3B%20%7D%0A%20%20%20%20%20%20%20%20%20%20%20%20Ok(())%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20_%20%3D%3E%20Ok(())%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20use%20serde_json%3A%3Ajson%3B%0A%20%20%20%20println!(%22Valid%3A%20%7B%3A%3F%7D%22%2C%20validate_tool_args(%22send_email%22%2C%20%26json!(%7B%22to%22%3A%22a%40b.com%22%7D)))%3B%0A%20%20%20%20println!(%22Invalid%3A%20%7B%3A%3F%7D%22%2C%20validate_tool_args(%22send_email%22%2C%20%26json!(%7B%22to%22%3A%22bad%22%7D)))%3B%0A%7D)

## Common Patterns

### Gateway/Aggregator Pattern

When aggregating multiple MCP servers:

```rust
use std::collections::HashMap;
use serde_json::{json, Value};

fn aggregate_tools(servers: &HashMap<&str, Vec<Value>>) -> Vec<Value> {
    let mut all_tools = Vec::new();

    for (server_name, tools) in servers {
        for tool in tools {
            let mut t = tool.clone();
            if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                // Prefix to avoid collisions
                t["name"] = json!(format!("{}_{}", server_name, name));
                t["_server"] = json!(server_name);
                t["_original_name"] = json!(name);
            }
            all_tools.push(t);
        }
    }

    all_tools
}

fn route_tool_call(prefixed_name: &str, servers: &[&str]) -> Option<(&str, &str)> {
    for server in servers {
        let prefix = format!("{}_", server);
        if prefixed_name.starts_with(&prefix) {
            let original = &prefixed_name[prefix.len()..];
            return Some((server, original));
        }
    }
    None
}

fn main() {
    let mut servers = HashMap::new();
    servers.insert("math", vec![json!({"name": "add"}), json!({"name": "multiply"})]);
    servers.insert("text", vec![json!({"name": "uppercase"})]);

    let tools = aggregate_tools(&servers);
    println!("Aggregated tools:");
    for tool in &tools {
        println!("  - {}", tool.get("name").unwrap());
    }

    // Route a call
    if let Some((server, tool)) = route_tool_call("math_add", &["math", "text"]) {
        println!("\nRouting 'math_add' to server '{}', tool '{}'", server, tool);
    }
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20std%3A%3Acollections%3A%3AHashMap%3B%0Ause%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0Afn%20aggregate_tools(servers%3A%20%26HashMap%3C%26str%2C%20Vec%3CValue%3E%3E)%20-%3E%20Vec%3CValue%3E%20%7B%0A%20%20%20%20let%20mut%20all%20%3D%20Vec%3A%3Anew()%3B%0A%20%20%20%20for%20(name%2C%20tools)%20in%20servers%20%7B%0A%20%20%20%20%20%20%20%20for%20tool%20in%20tools%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20mut%20t%20%3D%20tool.clone()%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20if%20let%20Some(n)%20%3D%20tool.get(%22name%22).and_then(%7Cn%7Cn.as_str())%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20t%5B%22name%22%5D%20%3D%20json!(format!(%22%7B%7D_%7B%7D%22%2C%20name%2C%20n))%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20%20%20%20%20all.push(t)%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%20%20%20%20all%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20mut%20servers%20%3D%20HashMap%3A%3Anew()%3B%0A%20%20%20%20servers.insert(%22math%22%2C%20vec!%5Bjson!(%7B%22name%22%3A%22add%22%7D)%5D)%3B%0A%20%20%20%20servers.insert(%22text%22%2C%20vec!%5Bjson!(%7B%22name%22%3A%22upper%22%7D)%5D)%3B%0A%20%20%20%20for%20t%20in%20aggregate_tools(%26servers)%20%7B%0A%20%20%20%20%20%20%20%20println!(%22Tool%3A%20%7B%7D%22%2C%20t.get(%22name%22).unwrap())%3B%0A%20%20%20%20%7D%0A%7D)

## Checklist

### Before Deployment

- [ ] All requests return valid JSON-RPC responses
- [ ] Notifications return HTTP 202 with no body
- [ ] `mcp-protocol-version` header included
- [ ] SSE stream stays open with keep-alive
- [ ] `/mcp/health` endpoint works without auth
- [ ] All tools have descriptions and schemas
- [ ] Error responses use standard codes
- [ ] Input validation on all tool arguments
- [ ] Tokens validated on every request
- [ ] HTTPS enabled (production)

### Testing Checklist

```bash
# Health check
curl http://localhost:3000/mcp/health

# Initialize handshake
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'

# Notification (should return 202)
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"notifications/initialized"}'

# List tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

# SSE stream (should stay open)
curl -N http://localhost:3000/mcp \
  -H "Accept: text/event-stream"
```

## Summary

| Category | Key Points |
|----------|------------|
| **Protocol** | Use JSON-RPC 2.0, match IDs, handle notifications |
| **Tools** | Clear names, good descriptions, input schemas |
| **Errors** | Standard codes, helpful messages, `isError` flag |
| **HTTP** | Required headers, SSE keep-alive, health endpoint |
| **Security** | Short tokens, validate everything, use HTTPS |

---

Congratulations! You've completed the MCP Learning Guide. Check out the [MetaMCP repository](../) for a complete implementation example.
