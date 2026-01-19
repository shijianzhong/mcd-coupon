use serde::{Deserialize, Serialize};

/// MCP JSON-RPC request structure
/// Per JSON-RPC 2.0 spec: id can be string, number, or null (for notifications)
/// Notifications don't have an id field
#[derive(Debug, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id: Option<u32>,
}

/// Deserialize optional ID field - supports number, string, null, or missing
fn deserialize_optional_id<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(n)) => {
            n.as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .map(Some)
                .ok_or_else(|| serde::de::Error::custom("Invalid ID: number out of range"))
        }
        Some(serde_json::Value::String(s)) => {
            s.parse::<u32>()
                .map(Some)
                .map_err(|_| serde::de::Error::custom("Invalid ID: string is not a valid number"))
        }
        _ => Err(serde::de::Error::custom("Invalid ID: must be number, string, or null")),
    }
}

/// MCP tools/call parameters - uses name and arguments per MCP spec
#[derive(Debug, Deserialize)]
pub struct McpToolCallParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// MCP system.listMethods parameters
#[derive(Debug, Deserialize)]
pub struct McpListMethodsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

/// MCP system.describeMethod parameters
#[derive(Debug, Deserialize)]
pub struct McpDescribeMethodParams {
    pub name: String,
}

/// MCP JSON-RPC response structure
/// Per JSON-RPC 2.0 spec: response must have either "result" or "error", but not both
#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
    pub id: u32,
}

/// MCP error structure
#[derive(Debug, Serialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP tool result with content array per MCP spec
#[derive(Debug, Serialize)]
pub struct McpToolResult {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

/// MCP content item - can be text, image, etc.
#[derive(Debug, Serialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP tool description for system.describeMethod
#[derive(Debug, Serialize)]
pub struct McpToolDescription {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub returns: serde_json::Value,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<McpToolExample>>,
}

/// MCP tool example
#[derive(Debug, Serialize)]
pub struct McpToolExample {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub returns: serde_json::Value,
}

impl McpResponse {
    /// Create a success response with tool result
    pub fn success_tool_result(id: u32, content: Vec<McpContent>) -> Self {
        let tool_result = McpToolResult {
            content,
            is_error: false,
        };
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(tool_result).unwrap()),
            error: None,
            id,
        }
    }
    
    /// Create a generic success response
    pub fn success<T: Serialize>(id: u32, data: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(data).unwrap()),
            error: None,
            id,
        }
    }
    
    /// Create an error response
    pub fn error(id: u32, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(McpError {
                code,
                message: message.to_string(),
                data: None,
            }),
            id,
        }
    }
    
    /// Create an error response with additional data
    pub fn error_with_data(id: u32, code: i32, message: &str, data: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(McpError {
                code,
                message: message.to_string(),
                data: Some(data),
            }),
            id,
        }
    }
    
    /// Create a tool error response
    pub fn tool_error(id: u32, message: &str) -> Self {
        let tool_result = McpToolResult {
            content: vec![McpContent {
                content_type: "text".to_string(),
                text: Some(message.to_string()),
                data: None,
            }],
            is_error: true,
        };
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(tool_result).unwrap()),
            error: None,
            id,
        }
    }
}

impl McpContent {
    /// Create a text content item
    pub fn text(content: &str) -> Self {
        Self {
            content_type: "text".to_string(),
            text: Some(content.to_string()),
            data: None,
        }
    }
    
    /// Create a text content item with additional data
    pub fn text_with_data(content: &str, data: serde_json::Value) -> Self {
        Self {
            content_type: "text".to_string(),
            text: Some(content.to_string()),
            data: Some(data),
        }
    }
    
    /// Create a structured data content item
    pub fn data(data: serde_json::Value) -> Self {
        Self {
            content_type: "data".to_string(),
            text: None,
            data: Some(data),
        }
    }
}