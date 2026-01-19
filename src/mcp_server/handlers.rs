use axum::{extract::State, response::{Json, Response}, routing::{post, get}, Router, http::{HeaderMap, StatusCode, header}, body::Body};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::{mcp::McpClient, config::Config, mcp_server::types::*};

/// MCP server state
pub struct McpServerState {
    pub mcp_client: Arc<Mutex<McpClient>>,
    pub config: Config,
}

impl McpServerState {
    pub fn new(mcp_client: McpClient, config: Config) -> Self {
        Self {
            mcp_client: Arc::new(Mutex::new(mcp_client)),
            config,
        }
    }
}

/// Handle MCP JSON-RPC requests
async fn handle_mcp_request(
    State(state): State<Arc<Mutex<McpServerState>>>,
    Json(request): Json<McpRequest>,
) -> Response<Body> {
    // Handle notifications (requests without id) - don't send response
    if request.id.is_none() {
        // For notifications, we don't send a response per JSON-RPC 2.0 spec
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(""))
            .unwrap();
    }
    
    let id = request.id.unwrap();
    let response: McpResponse = match request.method.as_str() {
        // Standard MCP initialization method
        "initialize" => handle_initialize(&state, id).await.0,
        // Standard MCP methods
        "tools/list" => handle_tools_list(&state, id).await.0,
        "tools/call" => handle_tools_call(&state, &request).await.0,
        "system.listMethods" => handle_list_methods(&state, id).await.0,
        "system.describeMethod" => handle_describe_method(&state, &request).await.0,
        _ => McpResponse::error(
            id,
            -32601,
            &format!("Method not found: {}", request.method),
        ),
    };
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&response).unwrap_or_default()))
        .unwrap()
}

/// Handle initialize method - required for MCP protocol
async fn handle_initialize(
    _state: &Arc<Mutex<McpServerState>>,
    id: u32,
) -> Json<McpResponse> {
    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "mcd-coupon",
            "version": "0.1.0"
        }
    });
    
    Json(McpResponse::success(id, result))
}

/// Handle tools/list method - returns list of available tools
async fn handle_tools_list(
    _state: &Arc<Mutex<McpServerState>>,
    id: u32,
) -> Json<McpResponse> {
    let tools = vec![
        serde_json::json!({
            "name": "available-coupons",
            "description": "获取所有可用的麦当劳优惠券",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        serde_json::json!({
            "name": "auto-bind-coupons",
            "description": "一键领取所有可用的麦当劳优惠券",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        serde_json::json!({
            "name": "my-coupons",
            "description": "查看已领取的麦当劳优惠券",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        serde_json::json!({
            "name": "now-time-info",
            "description": "获取当前时间信息",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
    ];
    
    let result = serde_json::json!({
        "tools": tools
    });
    
    Json(McpResponse::success(id, result))
}

/// Handle tools/call method
async fn handle_tools_call(
    state: &Arc<Mutex<McpServerState>>,
    request: &McpRequest,
) -> Json<McpResponse> {
    // request.id should always be Some at this point (checked in handle_mcp_request)
    let id = request.id.unwrap_or(0);
    
    // Parse params as McpToolCallParams
    let tool_params = match &request.params {
        Some(params) => serde_json::from_value(params.clone()),
        None => Err(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing params"))),
    };

    let tool_params: McpToolCallParams = match tool_params {
        Ok(params) => params,
        Err(e) => {
            return Json(McpResponse::error(
                id,
                -32602,
                &format!("Invalid params: {}", e),
            ));
        }
    };

    // Handle the tool call based on tool name
    match tool_params.name.as_str() {
        "available-coupons" => handle_available_coupons(&state, id).await,
        "auto-bind-coupons" => handle_auto_bind_coupons(&state, id).await,
        "my-coupons" => handle_my_coupons(&state, id).await,
        "now-time-info" => handle_current_time(&state, id).await,
        _ => Json(McpResponse::error(
            id,
            -32601,
            &format!("Tool not found: {}", tool_params.name),
        )),
    }
}

/// Handle system.listMethods method
async fn handle_list_methods(
    _state: &Arc<Mutex<McpServerState>>,
    id: u32,
) -> Json<McpResponse> {
    let mut all_methods = vec![
        "initialize".to_string(),
        "tools/list".to_string(),
        "tools/call".to_string(),
        "system.listMethods".to_string(),
        "system.describeMethod".to_string(),
    ];

    // Add all tools as "tools/call:{tool_name}" format
    let tools = vec![
        "available-coupons",
        "auto-bind-coupons",
        "my-coupons",
        "now-time-info",
    ];

    all_methods.extend(tools.iter().map(|tool| format!("tools/call:{}", tool)));

    Json(McpResponse::success(id, all_methods))
}

/// Handle system.describeMethod method
async fn handle_describe_method(
    _state: &Arc<Mutex<McpServerState>>,
    request: &McpRequest,
) -> Json<McpResponse> {
    // request.id should always be Some at this point (checked in handle_mcp_request)
    let id = request.id.unwrap_or(0);
    
    // Parse params as McpDescribeMethodParams
    let describe_params = match &request.params {
        Some(params) => serde_json::from_value(params.clone()),
        None => Err(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing params"))),
    };

    let describe_params: McpDescribeMethodParams = match describe_params {
        Ok(params) => params,
        Err(e) => {
            return Json(McpResponse::error(
                id,
                -32602,
                &format!("Invalid params: {}", e),
            ));
        }
    };

    // Handle method description based on method name
    let description = match describe_params.name.as_str() {
        "initialize" => describe_initialize(),
        "tools/list" => describe_tools_list(),
        "tools/call" => describe_tools_call(),
        "system.listMethods" => describe_list_methods(),
        "system.describeMethod" => describe_describe_method(),
        "available-coupons" | "tools/call:available-coupons" => describe_available_coupons_tool(),
        "auto-bind-coupons" | "tools/call:auto-bind-coupons" => describe_auto_bind_coupons_tool(),
        "my-coupons" | "tools/call:my-coupons" => describe_my_coupons_tool(),
        "now-time-info" | "tools/call:now-time-info" => describe_current_time_tool(),
        _ => {
            return Json(McpResponse::error(
                id,
                -32601,
                &format!("Method not found: {}", describe_params.name),
            ));
        }
    };

    Json(McpResponse::success(id, description))
}

/// Handle available-coupons tool
async fn handle_available_coupons(
    state: &Arc<Mutex<McpServerState>>,
    id: u32,
) -> Json<McpResponse> {
    let state = state.lock().await;
    let client = state.mcp_client.lock().await;

    match client.get_available_coupons().await {
        Ok(result) => {
            let content = vec![McpContent::text(&result)];
            Json(McpResponse::success_tool_result(id, content))
        }
        Err(e) => Json(McpResponse::tool_error(id, &e.to_string())),
    }
}

/// Handle auto-bind-coupons tool
async fn handle_auto_bind_coupons(
    state: &Arc<Mutex<McpServerState>>,
    id: u32,
) -> Json<McpResponse> {
    let state = state.lock().await;
    let client = state.mcp_client.lock().await;

    match client.auto_bind_coupons().await {
        Ok(result) => {
            let content = vec![McpContent::text(&result)];
            Json(McpResponse::success_tool_result(id, content))
        }
        Err(e) => Json(McpResponse::tool_error(id, &e.to_string())),
    }
}

/// Handle my-coupons tool
async fn handle_my_coupons(
    state: &Arc<Mutex<McpServerState>>,
    id: u32,
) -> Json<McpResponse> {
    let state = state.lock().await;
    let client = state.mcp_client.lock().await;

    match client.get_my_coupons().await {
        Ok(result) => {
            let content = vec![McpContent::text(&result)];
            Json(McpResponse::success_tool_result(id, content))
        }
        Err(e) => Json(McpResponse::tool_error(id, &e.to_string())),
    }
}

/// Handle now-time-info tool
async fn handle_current_time(
    state: &Arc<Mutex<McpServerState>>,
    id: u32,
) -> Json<McpResponse> {
    let state = state.lock().await;
    let client = state.mcp_client.lock().await;

    match client.get_current_time().await {
        Ok(result) => {
            let content = vec![McpContent::text(&result)];
            Json(McpResponse::success_tool_result(id, content))
        }
        Err(e) => Json(McpResponse::tool_error(id, &e.to_string())),
    }
}

/// Describe initialize method
fn describe_initialize() -> McpToolDescription {
    McpToolDescription {
        name: "initialize".to_string(),
        description: "初始化MCP连接".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "protocolVersion": {
                    "type": "string",
                    "description": "协议版本"
                },
                "capabilities": {
                    "type": "object",
                    "description": "客户端能力"
                },
                "clientInfo": {
                    "type": "object",
                    "description": "客户端信息"
                }
            }
        }),
        returns: serde_json::json!({
            "type": "object",
            "properties": {
                "protocolVersion": {
                    "type": "string"
                },
                "capabilities": {
                    "type": "object"
                },
                "serverInfo": {
                    "type": "object"
                }
            }
        }),
        tags: vec!["system".to_string(), "initialization".to_string()],
        examples: None,
    }
}

/// Describe tools/list method
fn describe_tools_list() -> McpToolDescription {
    McpToolDescription {
        name: "tools/list".to_string(),
        description: "列出所有可用的MCP工具".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
        returns: serde_json::json!({
            "type": "object",
            "properties": {
                "tools": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "description": {"type": "string"},
                            "inputSchema": {"type": "object"}
                        }
                    }
                }
            }
        }),
        tags: vec!["tools".to_string(), "introspection".to_string()],
        examples: None,
    }
}

/// Describe tools/call method
fn describe_tools_call() -> McpToolDescription {
    McpToolDescription {
        name: "tools/call".to_string(),
        description: "调用指定的MCP工具".to_string(),
        parameters: serde_json::Value::Object(serde_json::Map::new()),
        returns: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec!["system".to_string(), "tools".to_string()],
        examples: None,
    }
}

/// Describe system.listMethods method
fn describe_list_methods() -> McpToolDescription {
    McpToolDescription {
        name: "system.listMethods".to_string(),
        description: "列出所有可用的MCP方法".to_string(),
        parameters: serde_json::Value::Object(serde_json::Map::new()),
        returns: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec!["system".to_string(), "introspection".to_string()],
        examples: None,
    }
}

/// Describe system.describeMethod method
fn describe_describe_method() -> McpToolDescription {
    McpToolDescription {
        name: "system.describeMethod".to_string(),
        description: "获取指定MCP方法的详细描述".to_string(),
        parameters: serde_json::Value::Object(serde_json::Map::new()),
        returns: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec!["system".to_string(), "introspection".to_string()],
        examples: None,
    }
}

/// Describe available-coupons tool
fn describe_available_coupons_tool() -> McpToolDescription {
    McpToolDescription {
        name: "available-coupons".to_string(),
        description: "获取所有可用的麦当劳优惠券".to_string(),
        parameters: serde_json::Value::Object(serde_json::Map::new()),
        returns: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec!["coupons".to_string(), "available".to_string()],
        examples: None,
    }
}

/// Describe auto-bind-coupons tool
fn describe_auto_bind_coupons_tool() -> McpToolDescription {
    McpToolDescription {
        name: "auto-bind-coupons".to_string(),
        description: "一键领取所有可用的麦当劳优惠券".to_string(),
        parameters: serde_json::Value::Object(serde_json::Map::new()),
        returns: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec!["coupons".to_string(), "claim".to_string()],
        examples: None,
    }
}

/// Describe my-coupons tool
fn describe_my_coupons_tool() -> McpToolDescription {
    McpToolDescription {
        name: "my-coupons".to_string(),
        description: "查看已领取的麦当劳优惠券".to_string(),
        parameters: serde_json::Value::Object(serde_json::Map::new()),
        returns: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec!["coupons".to_string(), "my".to_string()],
        examples: None,
    }
}

/// Describe now-time-info tool
fn describe_current_time_tool() -> McpToolDescription {
    McpToolDescription {
        name: "now-time-info".to_string(),
        description: "获取当前时间信息".to_string(),
        parameters: serde_json::Value::Object(serde_json::Map::new()),
        returns: serde_json::Value::Object(serde_json::Map::new()),
        tags: vec!["time".to_string()],
        examples: None,
    }
}

/// Handle MCP GET requests for SSE/streamable connections
/// For SSE: GET request establishes the connection, responses come via POST
/// For streamable HTTP: GET request is just a health check
async fn handle_mcp_get_request(
    headers: HeaderMap,
    State(_state): State<Arc<Mutex<McpServerState>>>,
) -> Response<Body> {
    // Check if this is an SSE request by looking for Accept header
    if let Some(accept) = headers.get(header::ACCEPT) {
        if accept.to_str().unwrap_or("").contains("text/event-stream") {
            // For SSE connections, establish the connection but don't send a response yet
            // The client will send POST requests for actual JSON-RPC calls
            // We need to keep the connection open and wait for POST requests
            // However, axum doesn't support bidirectional SSE easily, so we'll just
            // return an empty SSE stream that the client can use
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/event-stream")
                .header(header::CONNECTION, "keep-alive")
                .header(header::CACHE_CONTROL, "no-cache")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(": connected\n\n"))
                .unwrap();
        }
    }
    
    // For normal streamable HTTP GET requests, return a simple health check response
    // This is not a JSON-RPC response, just a simple status
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"status":"ok"}"#))
        .unwrap()
}

/// Run the MCP server
pub async fn run_mcp_server(config: Config, mcp_client: McpClient) -> Result<()> {
    let port = config.mcp_server_port.unwrap_or(8080);
    let state = Arc::new(Mutex::new(McpServerState::new(mcp_client, config.clone())));

    // Create router with MCP endpoints
    // POST for JSON-RPC 2.0 requests
    // GET for SSE/streamable connections
    let app = Router::new()
        .route("/", post(handle_mcp_request))
        .route("/", get(handle_mcp_get_request))
        .with_state(state);

    // Start server
    println!("MCP server starting on port {}", port);
    axum::serve(
        tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?,
        app.into_make_service()
    ).await?;

    Ok(())
}
