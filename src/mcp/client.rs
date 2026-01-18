use crate::mcp::types::*;
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Duration;

const MCP_SERVER_URL: &str = "https://mcp.mcd.cn/mcp-servers/mcd-mcp";
const TIMEOUT: Duration = Duration::from_secs(30);

/// MCP Client for interacting with McDonald's MCP Server
#[derive(Debug, Clone)]
pub struct McpClient {
    client: Client,
    token: String,
    url: String,
}

impl McpClient {
    /// Create a new MCP client with the given token
    pub fn new(token: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(TIMEOUT)
            .build()?;

        Ok(Self {
            client,
            token,
            url: MCP_SERVER_URL.to_string(),
        })
    }

    /// Set a custom MCP server URL
    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    /// Validate if the token is valid by making a test request
    pub async fn validate_token(&self) -> Result<bool, String> {
        // Instead of using 'test' method, use a simple RPC request with a known structure
        // Even if method doesn't exist, we can still check authorization status
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "system.listMethods",
            "params": {},
            "id": 1
        });
        
        match self.client
            .post(&self.url)
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json")
            .json(&rpc_request)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                
                // If we get 401 Unauthorized, token is definitely invalid
                if status == reqwest::StatusCode::UNAUTHORIZED {
                    Ok(false)
                } else {
                    // Any other status means token is probably valid
                    // Don't log the response body to avoid showing method not found errors
                    Ok(true)
                }
            },
            Err(e) => {
                let error_msg = format!("网络请求失败: {}", e);
                Err(error_msg)
            }
        }
    }

    /// Call an MCP tool with the given parameters
    pub async fn call_tool(&self, tool_name: &str, params: serde_json::Value) -> Result<String> {
        // Build MCP tools/call request per MCP 2025-06-18 spec
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: McpToolCallParams {
                name: tool_name.to_string(),
                arguments: if params.is_null() || params == serde_json::json!({}) {
                    None
                } else {
                    Some(params)
                },
            },
            id: 1,
        };

        let response = self.client
            .post(&self.url)
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        
        if !status.is_success() {
            return Err(anyhow!("MCP Server error: {} - {}", status, body));
        }

        // Parse MCP response
        let mcp_response: McpResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow!("Failed to parse MCP response: {} - body: {}", e, body))?;

        // Check for JSON-RPC error
        if let Some(error) = mcp_response.error {
            return Err(anyhow!("MCP error {}: {}", error.code, error.message));
        }

        // Extract result
        let result = mcp_response.result
            .ok_or_else(|| anyhow!("MCP response missing result"))?;

        if result.is_error {
            // Collect error text from content
            let error_text: String = result.content.iter()
                .filter_map(|c| c.text.as_ref())
                .cloned()
                .collect::<Vec<_>>()
                .join("\n");
            return Err(anyhow!("MCP tool error: {}", error_text));
        }

        // Collect text content from result
        let text: String = result.content.iter()
            .filter(|c| c.content_type == "text")
            .filter_map(|c| c.text.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }

    /// Get all available coupons for the user (returns markdown text)
    pub async fn get_available_coupons(&self) -> Result<String> {
        self.call_tool("available-coupons", serde_json::json!({})).await
    }

    /// Auto-bind (claim) all available coupons (returns markdown summary)
    pub async fn auto_bind_coupons(&self) -> Result<String> {
        self.call_tool("auto-bind-coupons", serde_json::json!({})).await
    }

    /// Get all coupons that the user currently has (returns markdown text)
    pub async fn get_my_coupons(&self) -> Result<String> {
        self.call_tool("my-coupons", serde_json::json!({})).await
    }

    /// Get current time information from the server
    pub async fn get_current_time(&self) -> Result<String> {
        self.call_tool("now-time-info", serde_json::json!({})).await
    }
}

