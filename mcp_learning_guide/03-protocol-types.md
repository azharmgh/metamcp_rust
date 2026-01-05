# MCP Protocol Types

This guide covers the core data types used in the MCP protocol. Understanding these types is essential for implementing MCP in Rust.

## Request ID Types

MCP allows request IDs to be either strings or numbers. Here's how to handle both:

```rust
use serde::{Deserialize, Serialize};

/// Request ID can be a string or number
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

// Convenient conversions
impl From<String> for RequestId {
    fn from(s: String) -> Self {
        RequestId::String(s)
    }
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        RequestId::Number(n)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        RequestId::String(s.to_string())
    }
}

fn main() {
    // Both are valid
    let id1: RequestId = 42i64.into();
    let id2: RequestId = "request-abc".into();

    println!("Number ID: {:?}", id1);
    println!("String ID: {:?}", id2);

    // Serialize to JSON
    println!("JSON: {}", serde_json::to_string(&id1).unwrap());
    println!("JSON: {}", serde_json::to_string(&id2).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize%2C%20PartialEq%2C%20Eq%2C%20Hash)%5D%0A%23%5Bserde(untagged)%5D%0Apub%20enum%20RequestId%20%7B%0A%20%20%20%20String(String)%2C%0A%20%20%20%20Number(i64)%2C%0A%7D%0A%0Aimpl%20From%3CString%3E%20for%20RequestId%20%7B%0A%20%20%20%20fn%20from(s%3A%20String)%20-%3E%20Self%20%7B%20RequestId%3A%3AString(s)%20%7D%0A%7D%0A%0Aimpl%20From%3Ci64%3E%20for%20RequestId%20%7B%0A%20%20%20%20fn%20from(n%3A%20i64)%20-%3E%20Self%20%7B%20RequestId%3A%3ANumber(n)%20%7D%0A%7D%0A%0Aimpl%20From%3C%26str%3E%20for%20RequestId%20%7B%0A%20%20%20%20fn%20from(s%3A%20%26str)%20-%3E%20Self%20%7B%20RequestId%3A%3AString(s.to_string())%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20id1%3A%20RequestId%20%3D%2042i64.into()%3B%0A%20%20%20%20let%20id2%3A%20RequestId%20%3D%20%22request-abc%22.into()%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Number%20ID%3A%20%7B%3A%3F%7D%22%2C%20id1)%3B%0A%20%20%20%20println!(%22String%20ID%3A%20%7B%3A%3F%7D%22%2C%20id2)%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22JSON%3A%20%7B%7D%22%2C%20serde_json%3A%3Ato_string(%26id1).unwrap())%3B%0A%20%20%20%20println!(%22JSON%3A%20%7B%7D%22%2C%20serde_json%3A%3Ato_string(%26id2).unwrap())%3B%0A%7D)

## Server Capabilities

When a server responds to `initialize`, it declares its capabilities:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    #[serde(default)]
    pub list_changed: bool,  // Can notify when tools list changes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    #[serde(default)]
    pub subscribe: bool,     // Can subscribe to resource updates
    #[serde(default)]
    pub list_changed: bool,  // Can notify when resources list changes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptsCapability {
    #[serde(default)]
    pub list_changed: bool,  // Can notify when prompts list changes
}

fn main() {
    // Server that only supports tools
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: false }),
        resources: None,
        prompts: None,
    };

    println!("{}", serde_json::to_string_pretty(&capabilities).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Default%2C%20Serialize%2C%20Deserialize)%5D%0Apub%20struct%20ServerCapabilities%20%7B%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20tools%3A%20Option%3CToolsCapability%3E%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20resources%3A%20Option%3CResourcesCapability%3E%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20prompts%3A%20Option%3CPromptsCapability%3E%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20ToolsCapability%20%7B%0A%20%20%20%20%23%5Bserde(default)%5D%0A%20%20%20%20pub%20list_changed%3A%20bool%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20ResourcesCapability%20%7B%0A%20%20%20%20%23%5Bserde(default)%5D%0A%20%20%20%20pub%20subscribe%3A%20bool%2C%0A%20%20%20%20%23%5Bserde(default)%5D%0A%20%20%20%20pub%20list_changed%3A%20bool%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20PromptsCapability%20%7B%0A%20%20%20%20%23%5Bserde(default)%5D%0A%20%20%20%20pub%20list_changed%3A%20bool%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20capabilities%20%3D%20ServerCapabilities%20%7B%0A%20%20%20%20%20%20%20%20tools%3A%20Some(ToolsCapability%20%7B%20list_changed%3A%20false%20%7D)%2C%0A%20%20%20%20%20%20%20%20resources%3A%20None%2C%0A%20%20%20%20%20%20%20%20prompts%3A%20None%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26capabilities).unwrap())%3B%0A%7D)

## Initialize Result

The complete initialization response:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

// Using previous definitions...
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    pub list_changed: bool,
}

fn main() {
    let result = InitializeResult {
        protocol_version: "2025-03-26".to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: false }),
        },
        server_info: ServerInfo {
            name: "my-mcp-server".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20InitializeResult%20%7B%0A%20%20%20%20pub%20protocol_version%3A%20String%2C%0A%20%20%20%20pub%20capabilities%3A%20ServerCapabilities%2C%0A%20%20%20%20pub%20server_info%3A%20ServerInfo%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0Apub%20struct%20ServerInfo%20%7B%0A%20%20%20%20pub%20name%3A%20String%2C%0A%20%20%20%20pub%20version%3A%20String%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Default%2C%20Serialize%2C%20Deserialize)%5D%0Apub%20struct%20ServerCapabilities%20%7B%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20tools%3A%20Option%3CToolsCapability%3E%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20ToolsCapability%20%7B%0A%20%20%20%20pub%20list_changed%3A%20bool%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20result%20%3D%20InitializeResult%20%7B%0A%20%20%20%20%20%20%20%20protocol_version%3A%20%222025-03-26%22.to_string()%2C%0A%20%20%20%20%20%20%20%20capabilities%3A%20ServerCapabilities%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20tools%3A%20Some(ToolsCapability%20%7B%20list_changed%3A%20false%20%7D)%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20server_info%3A%20ServerInfo%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20name%3A%20%22my-mcp-server%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20version%3A%20%221.0.0%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26result).unwrap())%3B%0A%7D)

## Tool Definition

Tools are the most common capability. Here's how to define them:

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: Value,  // JSON Schema
}

fn main() {
    let add_tool = Tool {
        name: "add".to_string(),
        description: Some("Add two numbers together".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "a": {
                    "type": "number",
                    "description": "First number"
                },
                "b": {
                    "type": "number",
                    "description": "Second number"
                }
            },
            "required": ["a", "b"]
        }),
    };

    println!("{}", serde_json::to_string_pretty(&add_tool).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20Tool%20%7B%0A%20%20%20%20pub%20name%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20description%3A%20Option%3CString%3E%2C%0A%20%20%20%20pub%20input_schema%3A%20Value%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20add_tool%20%3D%20Tool%20%7B%0A%20%20%20%20%20%20%20%20name%3A%20%22add%22.to_string()%2C%0A%20%20%20%20%20%20%20%20description%3A%20Some(%22Add%20two%20numbers%20together%22.to_string())%2C%0A%20%20%20%20%20%20%20%20input_schema%3A%20json!(%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22type%22%3A%20%22object%22%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%22properties%22%3A%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22a%22%3A%20%7B%20%22type%22%3A%20%22number%22%2C%20%22description%22%3A%20%22First%20number%22%20%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22b%22%3A%20%7B%20%22type%22%3A%20%22number%22%2C%20%22description%22%3A%20%22Second%20number%22%20%7D%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%22required%22%3A%20%5B%22a%22%2C%20%22b%22%5D%0A%20%20%20%20%20%20%20%20%7D)%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26add_tool).unwrap())%3B%0A%7D)

## Tool Call Result

When a tool is executed, it returns content:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    pub content: Vec<Content>,
    #[serde(default)]
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Content {
    Text { text: String },
    Image { data: String, mime_type: String },
}

fn main() {
    // Successful result
    let success = ToolCallResult {
        content: vec![Content::Text {
            text: "The result is 8".to_string(),
        }],
        is_error: false,
    };

    // Error result
    let error = ToolCallResult {
        content: vec![Content::Text {
            text: "Division by zero".to_string(),
        }],
        is_error: true,
    };

    println!("Success:\n{}\n", serde_json::to_string_pretty(&success).unwrap());
    println!("Error:\n{}", serde_json::to_string_pretty(&error).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20ToolCallResult%20%7B%0A%20%20%20%20pub%20content%3A%20Vec%3CContent%3E%2C%0A%20%20%20%20%23%5Bserde(default)%5D%0A%20%20%20%20pub%20is_error%3A%20bool%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(tag%20%3D%20%22type%22%2C%20rename_all%20%3D%20%22lowercase%22)%5D%0Apub%20enum%20Content%20%7B%0A%20%20%20%20Text%20%7B%20text%3A%20String%20%7D%2C%0A%20%20%20%20Image%20%7B%20data%3A%20String%2C%20mime_type%3A%20String%20%7D%2C%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20success%20%3D%20ToolCallResult%20%7B%0A%20%20%20%20%20%20%20%20content%3A%20vec!%5BContent%3A%3AText%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20text%3A%20%22The%20result%20is%208%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%7D%5D%2C%0A%20%20%20%20%20%20%20%20is_error%3A%20false%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20let%20error%20%3D%20ToolCallResult%20%7B%0A%20%20%20%20%20%20%20%20content%3A%20vec!%5BContent%3A%3AText%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20text%3A%20%22Division%20by%20zero%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%7D%5D%2C%0A%20%20%20%20%20%20%20%20is_error%3A%20true%2C%0A%20%20%20%20%7D%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22Success%3A%5Cn%7B%7D%5Cn%22%2C%20serde_json%3A%3Ato_string_pretty(%26success).unwrap())%3B%0A%20%20%20%20println!(%22Error%3A%5Cn%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26error).unwrap())%3B%0A%7D)

## Key Serde Attributes

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `#[serde(rename_all = "camelCase")]` | Convert field names | `list_changed` → `listChanged` |
| `#[serde(skip_serializing_if = "...")]` | Omit None/empty values | Skip `None` options |
| `#[serde(default)]` | Use Default if missing | `false` for bool |
| `#[serde(untagged)]` | No type tag in JSON | For enums like RequestId |
| `#[serde(tag = "type")]` | Add type discriminator | For Content enum |

## Complete Type Reference

See the full implementation in [`src/mcp/protocol/types.rs`](../src/mcp/protocol/types.rs).

---

Next: [MCP Capabilities](./04-capabilities.md)
