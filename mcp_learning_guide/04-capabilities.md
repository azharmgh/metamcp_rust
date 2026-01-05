# MCP Capabilities

MCP servers expose three types of capabilities: **Tools**, **Resources**, and **Prompts**. This guide explains each with practical examples.

## Overview

| Capability | Purpose | Methods | Example |
|------------|---------|---------|---------|
| **Tools** | Execute actions | `tools/list`, `tools/call` | Calculator, API calls |
| **Resources** | Read data | `resources/list`, `resources/read` | Files, configs |
| **Prompts** | Template prompts | `prompts/list`, `prompts/get` | Code review template |

## Tools

Tools are executable functions that the AI can call. They're the most common capability.

### Defining Tools

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

fn create_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "echo".to_string(),
            description: Some("Echoes back the input message".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo"
                    }
                },
                "required": ["message"]
            }),
        },
        Tool {
            name: "add".to_string(),
            description: Some("Adds two numbers".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "a": { "type": "number" },
                    "b": { "type": "number" }
                },
                "required": ["a", "b"]
            }),
        },
    ]
}

fn main() {
    let tools = create_tools();
    let response = json!({ "tools": tools });
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20Tool%20%7B%0A%20%20%20%20pub%20name%3A%20String%2C%0A%20%20%20%20pub%20description%3A%20Option%3CString%3E%2C%0A%20%20%20%20pub%20input_schema%3A%20Value%2C%0A%7D%0A%0Afn%20create_tools()%20-%3E%20Vec%3CTool%3E%20%7B%0A%20%20%20%20vec!%5B%0A%20%20%20%20%20%20%20%20Tool%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20name%3A%20%22echo%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20description%3A%20Some(%22Echoes%20back%20the%20input%22.to_string())%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20input_schema%3A%20json!(%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22type%22%3A%20%22object%22%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22properties%22%3A%20%7B%20%22message%22%3A%20%7B%20%22type%22%3A%20%22string%22%20%7D%20%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22required%22%3A%20%5B%22message%22%5D%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D)%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20Tool%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20name%3A%20%22add%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20description%3A%20Some(%22Adds%20two%20numbers%22.to_string())%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20input_schema%3A%20json!(%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22type%22%3A%20%22object%22%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22properties%22%3A%20%7B%20%22a%22%3A%20%7B%22type%22%3A%22number%22%7D%2C%20%22b%22%3A%20%7B%22type%22%3A%22number%22%7D%20%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22required%22%3A%20%5B%22a%22%2C%20%22b%22%5D%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D)%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%5D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20tools%20%3D%20create_tools()%3B%0A%20%20%20%20let%20response%20%3D%20json!(%7B%20%22tools%22%3A%20tools%20%7D)%3B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26response).unwrap())%3B%0A%7D)

### Executing Tools

```rust
use serde_json::{json, Value};

fn execute_tool(name: &str, arguments: &Value) -> Result<Value, String> {
    match name {
        "echo" => {
            let message = arguments.get("message")
                .and_then(|v| v.as_str())
                .ok_or("Missing message")?;
            Ok(json!({
                "content": [{ "type": "text", "text": message }],
                "isError": false
            }))
        }
        "add" => {
            let a = arguments.get("a")
                .and_then(|v| v.as_f64())
                .ok_or("Missing 'a'")?;
            let b = arguments.get("b")
                .and_then(|v| v.as_f64())
                .ok_or("Missing 'b'")?;
            Ok(json!({
                "content": [{ "type": "text", "text": format!("{}", a + b) }],
                "isError": false
            }))
        }
        _ => Err(format!("Unknown tool: {}", name))
    }
}

fn main() {
    // Execute echo
    let result = execute_tool("echo", &json!({"message": "Hello!"}));
    println!("Echo: {}", serde_json::to_string_pretty(&result.unwrap()).unwrap());

    // Execute add
    let result = execute_tool("add", &json!({"a": 5, "b": 3}));
    println!("\nAdd: {}", serde_json::to_string_pretty(&result.unwrap()).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0Afn%20execute_tool(name%3A%20%26str%2C%20arguments%3A%20%26Value)%20-%3E%20Result%3CValue%2C%20String%3E%20%7B%0A%20%20%20%20match%20name%20%7B%0A%20%20%20%20%20%20%20%20%22echo%22%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20message%20%3D%20arguments.get(%22message%22)%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20.and_then(%7Cv%7C%20v.as_str())%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20.ok_or(%22Missing%20message%22)%3F%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20Ok(json!(%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22content%22%3A%20%5B%7B%20%22type%22%3A%20%22text%22%2C%20%22text%22%3A%20message%20%7D%5D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22isError%22%3A%20false%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D))%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20%22add%22%20%3D%3E%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20a%20%3D%20arguments.get(%22a%22).and_then(%7Cv%7C%20v.as_f64()).ok_or(%22Missing%20'a'%22)%3F%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20b%20%3D%20arguments.get(%22b%22).and_then(%7Cv%7C%20v.as_f64()).ok_or(%22Missing%20'b'%22)%3F%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20Ok(json!(%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22content%22%3A%20%5B%7B%20%22type%22%3A%20%22text%22%2C%20%22text%22%3A%20format!(%22%7B%7D%22%2C%20a%20%2B%20b)%20%7D%5D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%22isError%22%3A%20false%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D))%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20_%20%3D%3E%20Err(format!(%22Unknown%20tool%3A%20%7B%7D%22%2C%20name))%0A%20%20%20%20%7D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20result%20%3D%20execute_tool(%22echo%22%2C%20%26json!(%7B%22message%22%3A%20%22Hello!%22%7D))%3B%0A%20%20%20%20println!(%22Echo%3A%20%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26result.unwrap()).unwrap())%3B%0A%20%20%20%20%0A%20%20%20%20let%20result%20%3D%20execute_tool(%22add%22%2C%20%26json!(%7B%22a%22%3A%205%2C%20%22b%22%3A%203%7D))%3B%0A%20%20%20%20println!(%22%5CnAdd%3A%20%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26result.unwrap()).unwrap())%3B%0A%7D)

## Resources

Resources provide read-only access to data like files, configurations, or external content.

### Defining Resources

```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

fn list_resources() -> Vec<Resource> {
    vec![
        Resource {
            uri: "config://server".to_string(),
            name: "Server Configuration".to_string(),
            description: Some("Current server settings".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        Resource {
            uri: "file://readme.txt".to_string(),
            name: "README".to_string(),
            description: Some("Project readme file".to_string()),
            mime_type: Some("text/plain".to_string()),
        },
    ]
}

fn main() {
    let resources = list_resources();
    println!("{}", serde_json::to_string_pretty(&json!({"resources": resources})).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3Ajson%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0A%23%5Bserde(rename_all%20%3D%20%22camelCase%22)%5D%0Apub%20struct%20Resource%20%7B%0A%20%20%20%20pub%20uri%3A%20String%2C%0A%20%20%20%20pub%20name%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20description%3A%20Option%3CString%3E%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20mime_type%3A%20Option%3CString%3E%2C%0A%7D%0A%0Afn%20list_resources()%20-%3E%20Vec%3CResource%3E%20%7B%0A%20%20%20%20vec!%5B%0A%20%20%20%20%20%20%20%20Resource%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20uri%3A%20%22config%3A%2F%2Fserver%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20name%3A%20%22Server%20Configuration%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20description%3A%20Some(%22Current%20server%20settings%22.to_string())%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20mime_type%3A%20Some(%22application%2Fjson%22.to_string())%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%20%20%20%20Resource%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20uri%3A%20%22file%3A%2F%2Freadme.txt%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20name%3A%20%22README%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20description%3A%20Some(%22Project%20readme%22.to_string())%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20mime_type%3A%20Some(%22text%2Fplain%22.to_string())%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%5D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20resources%20%3D%20list_resources()%3B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26json!(%7B%22resources%22%3A%20resources%7D)).unwrap())%3B%0A%7D)

### Reading Resources

```rust
use serde_json::{json, Value};
use std::collections::HashMap;

fn read_resource(uri: &str, store: &HashMap<String, String>) -> Result<Value, String> {
    let content = store.get(uri).ok_or(format!("Resource not found: {}", uri))?;

    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "text/plain",
            "text": content
        }]
    }))
}

fn main() {
    // Simulate a resource store
    let mut store = HashMap::new();
    store.insert(
        "file://readme.txt".to_string(),
        "Welcome to MetaMCP!".to_string()
    );
    store.insert(
        "config://server".to_string(),
        r#"{"port": 12009, "host": "localhost"}"#.to_string()
    );

    // Read a resource
    let result = read_resource("file://readme.txt", &store);
    println!("{}", serde_json::to_string_pretty(&result.unwrap()).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0Ause%20std%3A%3Acollections%3A%3AHashMap%3B%0A%0Afn%20read_resource(uri%3A%20%26str%2C%20store%3A%20%26HashMap%3CString%2C%20String%3E)%20-%3E%20Result%3CValue%2C%20String%3E%20%7B%0A%20%20%20%20let%20content%20%3D%20store.get(uri).ok_or(format!(%22Resource%20not%20found%3A%20%7B%7D%22%2C%20uri))%3F%3B%0A%20%20%20%20%0A%20%20%20%20Ok(json!(%7B%0A%20%20%20%20%20%20%20%20%22contents%22%3A%20%5B%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%22uri%22%3A%20uri%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%22mimeType%22%3A%20%22text%2Fplain%22%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%22text%22%3A%20content%0A%20%20%20%20%20%20%20%20%7D%5D%0A%20%20%20%20%7D))%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20mut%20store%20%3D%20HashMap%3A%3Anew()%3B%0A%20%20%20%20store.insert(%22file%3A%2F%2Freadme.txt%22.to_string()%2C%20%22Welcome%20to%20MetaMCP!%22.to_string())%3B%0A%20%20%20%20store.insert(%22config%3A%2F%2Fserver%22.to_string()%2C%20r%23%22%7B%22port%22%3A%2012009%7D%22%23.to_string())%3B%0A%20%20%20%20%0A%20%20%20%20let%20result%20%3D%20read_resource(%22file%3A%2F%2Freadme.txt%22%2C%20%26store)%3B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26result.unwrap()).unwrap())%3B%0A%7D)

## Prompts

Prompts are reusable templates that the AI can use to structure its responses.

### Defining Prompts

```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

fn list_prompts() -> Vec<Prompt> {
    vec![
        Prompt {
            name: "code_review".to_string(),
            description: Some("Generate a code review".to_string()),
            arguments: vec![
                PromptArgument {
                    name: "code".to_string(),
                    description: Some("The code to review".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language".to_string()),
                    required: false,
                },
            ],
        },
        Prompt {
            name: "summarize".to_string(),
            description: Some("Summarize text".to_string()),
            arguments: vec![PromptArgument {
                name: "text".to_string(),
                description: Some("Text to summarize".to_string()),
                required: true,
            }],
        },
    ]
}

fn main() {
    let prompts = list_prompts();
    println!("{}", serde_json::to_string_pretty(&json!({"prompts": prompts})).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde%3A%3A%7BDeserialize%2C%20Serialize%7D%3B%0Ause%20serde_json%3A%3Ajson%3B%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0Apub%20struct%20Prompt%20%7B%0A%20%20%20%20pub%20name%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20description%3A%20Option%3CString%3E%2C%0A%20%20%20%20%23%5Bserde(default%2C%20skip_serializing_if%20%3D%20%22Vec%3A%3Ais_empty%22)%5D%0A%20%20%20%20pub%20arguments%3A%20Vec%3CPromptArgument%3E%2C%0A%7D%0A%0A%23%5Bderive(Debug%2C%20Clone%2C%20Serialize%2C%20Deserialize)%5D%0Apub%20struct%20PromptArgument%20%7B%0A%20%20%20%20pub%20name%3A%20String%2C%0A%20%20%20%20%23%5Bserde(skip_serializing_if%20%3D%20%22Option%3A%3Ais_none%22)%5D%0A%20%20%20%20pub%20description%3A%20Option%3CString%3E%2C%0A%20%20%20%20%23%5Bserde(default)%5D%0A%20%20%20%20pub%20required%3A%20bool%2C%0A%7D%0A%0Afn%20list_prompts()%20-%3E%20Vec%3CPrompt%3E%20%7B%0A%20%20%20%20vec!%5B%0A%20%20%20%20%20%20%20%20Prompt%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20name%3A%20%22code_review%22.to_string()%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20description%3A%20Some(%22Generate%20a%20code%20review%22.to_string())%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20arguments%3A%20vec!%5B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20PromptArgument%20%7B%20name%3A%20%22code%22.to_string()%2C%20description%3A%20Some(%22The%20code%22.to_string())%2C%20required%3A%20true%20%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20PromptArgument%20%7B%20name%3A%20%22language%22.to_string()%2C%20description%3A%20Some(%22Language%22.to_string())%2C%20required%3A%20false%20%7D%2C%0A%20%20%20%20%20%20%20%20%20%20%20%20%5D%2C%0A%20%20%20%20%20%20%20%20%7D%2C%0A%20%20%20%20%5D%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20prompts%20%3D%20list_prompts()%3B%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26json!(%7B%22prompts%22%3A%20prompts%7D)).unwrap())%3B%0A%7D)

## Comparison: When to Use What

| Use Case | Capability | Why |
|----------|------------|-----|
| Execute an action | **Tool** | Tools perform operations |
| Read data | **Resource** | Resources are read-only |
| Provide a template | **Prompt** | Prompts structure output |
| Make an API call | **Tool** | Tools can have side effects |
| Serve a file | **Resource** | Files are static data |
| Format a response | **Prompt** | Prompts guide AI output |

## Aggregating Capabilities (Gateway Pattern)

MetaMCP aggregates capabilities from multiple servers by prefixing names:

```rust
use serde_json::{json, Value};

fn aggregate_tools(servers: Vec<(&str, Vec<Value>)>) -> Vec<Value> {
    let mut all_tools = Vec::new();

    for (server_name, tools) in servers {
        for tool in tools {
            let mut prefixed_tool = tool.clone();
            if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                // Prefix with server name: "server1_toolname"
                let prefixed_name = format!("{}_{}", server_name, name);
                prefixed_tool["name"] = json!(prefixed_name);
                prefixed_tool["_original_name"] = json!(name);
                prefixed_tool["_server"] = json!(server_name);
            }
            all_tools.push(prefixed_tool);
        }
    }

    all_tools
}

fn main() {
    let server1_tools = vec![
        json!({"name": "add", "description": "Add numbers"}),
        json!({"name": "subtract", "description": "Subtract numbers"}),
    ];

    let server2_tools = vec![
        json!({"name": "echo", "description": "Echo message"}),
    ];

    let aggregated = aggregate_tools(vec![
        ("math", server1_tools),
        ("utils", server2_tools),
    ]);

    println!("{}", serde_json::to_string_pretty(&aggregated).unwrap());
}
```

[Run in Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&code=use%20serde_json%3A%3A%7Bjson%2C%20Value%7D%3B%0A%0Afn%20aggregate_tools(servers%3A%20Vec%3C(%26str%2C%20Vec%3CValue%3E)%3E)%20-%3E%20Vec%3CValue%3E%20%7B%0A%20%20%20%20let%20mut%20all_tools%20%3D%20Vec%3A%3Anew()%3B%0A%20%20%20%20%0A%20%20%20%20for%20(server_name%2C%20tools)%20in%20servers%20%7B%0A%20%20%20%20%20%20%20%20for%20tool%20in%20tools%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20let%20mut%20prefixed_tool%20%3D%20tool.clone()%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20if%20let%20Some(name)%20%3D%20tool.get(%22name%22).and_then(%7Cn%7C%20n.as_str())%20%7B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20let%20prefixed_name%20%3D%20format!(%22%7B%7D_%7B%7D%22%2C%20server_name%2C%20name)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20prefixed_tool%5B%22name%22%5D%20%3D%20json!(prefixed_name)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20prefixed_tool%5B%22_original_name%22%5D%20%3D%20json!(name)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20prefixed_tool%5B%22_server%22%5D%20%3D%20json!(server_name)%3B%0A%20%20%20%20%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%20%20%20%20%20%20%20%20all_tools.push(prefixed_tool)%3B%0A%20%20%20%20%20%20%20%20%7D%0A%20%20%20%20%7D%0A%20%20%20%20%0A%20%20%20%20all_tools%0A%7D%0A%0Afn%20main()%20%7B%0A%20%20%20%20let%20server1_tools%20%3D%20vec!%5B%0A%20%20%20%20%20%20%20%20json!(%7B%22name%22%3A%20%22add%22%2C%20%22description%22%3A%20%22Add%20numbers%22%7D)%2C%0A%20%20%20%20%20%20%20%20json!(%7B%22name%22%3A%20%22subtract%22%2C%20%22description%22%3A%20%22Subtract%22%7D)%2C%0A%20%20%20%20%5D%3B%0A%20%20%20%20%0A%20%20%20%20let%20server2_tools%20%3D%20vec!%5B%0A%20%20%20%20%20%20%20%20json!(%7B%22name%22%3A%20%22echo%22%2C%20%22description%22%3A%20%22Echo%22%7D)%2C%0A%20%20%20%20%5D%3B%0A%20%20%20%20%0A%20%20%20%20let%20aggregated%20%3D%20aggregate_tools(vec!%5B%0A%20%20%20%20%20%20%20%20(%22math%22%2C%20server1_tools)%2C%0A%20%20%20%20%20%20%20%20(%22utils%22%2C%20server2_tools)%2C%0A%20%20%20%20%5D)%3B%0A%20%20%20%20%0A%20%20%20%20println!(%22%7B%7D%22%2C%20serde_json%3A%3Ato_string_pretty(%26aggregated).unwrap())%3B%0A%7D)

---

Next: [HTTP Transport](./05-http-transport.md)
