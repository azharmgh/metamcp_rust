//! Unit tests for MCP protocol types

use serde_json::json;

// Test JSON-RPC structures
#[cfg(test)]
mod jsonrpc_tests {
    use super::*;
    use metamcp::mcp::protocol::{
        JsonRpcRequest, JsonRpcResponse, JsonRpcError, RequestId,
        JSONRPC_VERSION, MCP_PROTOCOL_VERSION,
    };

    #[test]
    fn test_jsonrpc_version() {
        assert_eq!(JSONRPC_VERSION, "2.0");
    }

    #[test]
    fn test_mcp_protocol_version() {
        assert_eq!(MCP_PROTOCOL_VERSION, "2024-11-05");
    }

    #[test]
    fn test_request_id_from_string() {
        let id: RequestId = "test-id".to_string().into();
        match id {
            RequestId::String(s) => assert_eq!(s, "test-id"),
            _ => panic!("Expected string ID"),
        }
    }

    #[test]
    fn test_request_id_from_number() {
        let id: RequestId = 42i64.into();
        match id {
            RequestId::Number(n) => assert_eq!(n, 42),
            _ => panic!("Expected number ID"),
        }
    }

    #[test]
    fn test_jsonrpc_request_new() {
        let request = JsonRpcRequest::new(1i64, "test/method", None);

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test/method");
        assert!(request.params.is_none());
    }

    #[test]
    fn test_jsonrpc_request_with_params() {
        let params = json!({"key": "value"});
        let request = JsonRpcRequest::new("req-1".to_string(), "test/method", Some(params.clone()));

        assert_eq!(request.method, "test/method");
        assert_eq!(request.params, Some(params));
    }

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest::new(1i64, "tools/list", None);
        let json = serde_json::to_string(&request).expect("Failed to serialize");

        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tools/list\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_jsonrpc_request_deserialization() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tools/list");
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let result = json!({"tools": []});
        let response = JsonRpcResponse::success(RequestId::Number(1), result.clone());

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.result, Some(result));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let response = JsonRpcResponse::error(
            RequestId::Number(1),
            -32601,
            "Method not found",
            None,
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());

        let error = response.error.expect("Expected error");
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse::success(
            RequestId::Number(1),
            json!({"status": "ok"}),
        );
        let json = serde_json::to_string(&response).expect("Failed to serialize");

        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }
}

// Test MCP capability structures
#[cfg(test)]
mod capability_tests {
    use super::*;
    use metamcp::mcp::protocol::{
        ClientCapabilities, ServerCapabilities,
        ToolsCapability, ResourcesCapability, PromptsCapability,
        RootsCapability, SamplingCapability,
    };

    #[test]
    fn test_client_capabilities_default() {
        let caps = ClientCapabilities::default();
        assert!(caps.roots.is_none());
        assert!(caps.sampling.is_none());
    }

    #[test]
    fn test_server_capabilities_default() {
        let caps = ServerCapabilities::default();
        assert!(caps.tools.is_none());
        assert!(caps.resources.is_none());
        assert!(caps.prompts.is_none());
    }

    #[test]
    fn test_tools_capability_serialization() {
        let caps = ToolsCapability { list_changed: true };
        let json = serde_json::to_string(&caps).expect("Failed to serialize");
        assert!(json.contains("\"listChanged\":true"));
    }

    #[test]
    fn test_resources_capability() {
        let caps = ResourcesCapability {
            subscribe: true,
            list_changed: false,
        };
        let json = serde_json::to_string(&caps).expect("Failed to serialize");
        assert!(json.contains("\"subscribe\":true"));
        assert!(json.contains("\"listChanged\":false"));
    }

    #[test]
    fn test_prompts_capability() {
        let caps = PromptsCapability { list_changed: true };
        let json = serde_json::to_string(&caps).expect("Failed to serialize");
        assert!(json.contains("\"listChanged\":true"));
    }
}

// Test MCP tool structures
#[cfg(test)]
mod tool_tests {
    use super::*;
    use metamcp::mcp::protocol::{Tool, ToolCallParams, ToolCallResult, Content};

    #[test]
    fn test_tool_definition() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "arg1": {"type": "string"}
                }
            }),
        };

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, Some("A test tool".to_string()));
    }

    #[test]
    fn test_tool_serialization() {
        let tool = Tool {
            name: "echo".to_string(),
            description: Some("Echo tool".to_string()),
            input_schema: json!({"type": "object"}),
        };

        let json = serde_json::to_string(&tool).expect("Failed to serialize");
        assert!(json.contains("\"name\":\"echo\""));
        assert!(json.contains("\"description\":\"Echo tool\""));
        assert!(json.contains("\"inputSchema\""));
    }

    #[test]
    fn test_tool_call_params() {
        let params = ToolCallParams {
            name: "test".to_string(),
            arguments: json!({"key": "value"}),
        };

        let json = serde_json::to_string(&params).expect("Failed to serialize");
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"arguments\""));
    }

    #[test]
    fn test_content_text() {
        let content = Content::Text {
            text: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&content).expect("Failed to serialize");
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"text\":\"Hello, world!\""));
    }

    #[test]
    fn test_tool_call_result() {
        let result = ToolCallResult {
            content: vec![Content::Text {
                text: "Result".to_string(),
            }],
            is_error: false,
        };

        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
    }
}

// Test MCP resource and prompt structures
#[cfg(test)]
mod resource_prompt_tests {
    use super::*;
    use metamcp::mcp::protocol::{
        Resource, ResourceContent, ResourcesListResult,
        Prompt, PromptArgument, PromptsListResult,
    };

    #[test]
    fn test_resource_definition() {
        let resource = Resource {
            uri: "file:///test.txt".to_string(),
            name: "Test File".to_string(),
            description: Some("A test file".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        assert_eq!(resource.uri, "file:///test.txt");
        assert_eq!(resource.name, "Test File");
    }

    #[test]
    fn test_resource_content() {
        let content = ResourceContent {
            uri: "file:///test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("File content".to_string()),
            blob: None,
        };

        assert_eq!(content.text, Some("File content".to_string()));
        assert!(content.blob.is_none());
    }

    #[test]
    fn test_resources_list_result() {
        let result = ResourcesListResult {
            resources: vec![
                Resource {
                    uri: "file:///a.txt".to_string(),
                    name: "A".to_string(),
                    description: None,
                    mime_type: None,
                },
                Resource {
                    uri: "file:///b.txt".to_string(),
                    name: "B".to_string(),
                    description: None,
                    mime_type: None,
                },
            ],
        };

        assert_eq!(result.resources.len(), 2);
    }

    #[test]
    fn test_prompt_definition() {
        let prompt = Prompt {
            name: "test_prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: vec![
                PromptArgument {
                    name: "arg1".to_string(),
                    description: Some("First argument".to_string()),
                    required: true,
                },
            ],
        };

        assert_eq!(prompt.name, "test_prompt");
        assert_eq!(prompt.arguments.len(), 1);
        assert!(prompt.arguments[0].required);
    }

    #[test]
    fn test_prompts_list_result() {
        let result = PromptsListResult {
            prompts: vec![
                Prompt {
                    name: "prompt1".to_string(),
                    description: None,
                    arguments: vec![],
                },
            ],
        };

        assert_eq!(result.prompts.len(), 1);
    }
}

// Test initialize structures
#[cfg(test)]
mod initialize_tests {
    use super::*;
    use metamcp::mcp::protocol::{
        InitializeParams, InitializeResult,
        ClientInfo, ServerInfo as McpServerInfo,
        ClientCapabilities, ServerCapabilities,
        MCP_PROTOCOL_VERSION,
    };

    #[test]
    fn test_initialize_params() {
        let params = InitializeParams {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        assert_eq!(params.protocol_version, MCP_PROTOCOL_VERSION);
        assert_eq!(params.client_info.name, "test-client");
    }

    #[test]
    fn test_initialize_result() {
        let result = InitializeResult {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities::default(),
            server_info: McpServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        assert_eq!(result.protocol_version, MCP_PROTOCOL_VERSION);
        assert_eq!(result.server_info.name, "test-server");
    }

    #[test]
    fn test_initialize_params_serialization() {
        let params = InitializeParams {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
        };

        let json = serde_json::to_string(&params).expect("Failed to serialize");
        assert!(json.contains("\"protocolVersion\""));
        assert!(json.contains("\"clientInfo\""));
    }
}
