use serde::{Deserialize, Serialize};

/// MCP JSON-RPC request structure for tools/call method
#[derive(Debug, Serialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: McpToolCallParams,
    pub id: u32,
}

/// MCP tools/call parameters - uses name and arguments per MCP spec
#[derive(Debug, Serialize)]
pub struct McpToolCallParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// MCP JSON-RPC response structure
#[derive(Debug, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub result: Option<McpToolResult>,
    pub error: Option<McpError>,
    pub id: u32,
}

/// MCP error structure
#[derive(Debug, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

/// MCP tool result with content array per MCP spec
#[derive(Debug, Deserialize)]
pub struct McpToolResult {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

/// MCP content item - can be text, image, etc.
#[derive(Debug, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

/// Coupon information structure
#[derive(Debug, Deserialize)]
pub struct Coupon {
    pub name: String,
    pub coupon_id: String,
    pub validity: String,
    pub description: Option<String>,
    pub available: bool,
}

/// My coupons response
#[derive(Debug, Deserialize)]
pub struct MyCouponsResponse {
    pub coupons: Vec<Coupon>,
}

/// Auto-bind coupons response
#[derive(Debug, Deserialize)]
pub struct AutoBindCouponsResponse {
    pub coupons_bound: Vec<String>,
    pub message: Option<String>,
}
