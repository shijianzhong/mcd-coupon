use axum::{extract::State, response::{Html, IntoResponse, Json}, routing::{get, post}, Router};
use handlebars::Handlebars;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::{mcp::McpClient, config::Config};

/// Coupon structure for template rendering
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Coupon {
    pub title: String,
    pub price: String,
    pub expiry: String,
    pub receive_time: String,
    pub tags: String,
    pub image_url: String,
}

/// Serializable view of the application state for templates
#[derive(Debug, Serialize)]
pub struct AppStateView {
    pub has_token: bool,
}

impl AppStateView {
    fn from_state(state: &WebAppState) -> Self {
        Self {
            has_token: state.mcp_client.is_some(),
        }
    }
}

/// API Response structure
#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupons: Option<Vec<Coupon>>,
}

/// Web application state
pub struct WebAppState {
    pub mcp_client: Option<Arc<Mutex<McpClient>>>,
    pub config: Config,
    pub logs: Vec<String>,
    pub coupons: Vec<Coupon>,
    pub handlebars: Handlebars<'static>,
}

impl WebAppState {
    pub fn new(config: Config, handlebars: Handlebars<'static>) -> Self {
        Self {
            mcp_client: None,
            config,
            logs: vec!["应用已启动...".to_string()],
            coupons: Vec::new(),
            handlebars,
        }
    }

    pub fn add_log(&mut self, message: String) {
        println!("[LOG] {}", message);
        self.logs.push(message);
        // Keep only the last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub async fn init_mcp_client(&mut self, token: String) -> Result<()> {
        let client = McpClient::new(token)?;
        self.mcp_client = Some(Arc::new(Mutex::new(client)));
        Ok(())
    }
}

/// Initialize the web application
pub async fn run() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Set up Handlebars template engine
    let mut handlebars = Handlebars::new();

    // Embed template into binary (no external file dependency)
    const INDEX_TEMPLATE: &str = include_str!("templates/index.html");
    handlebars.register_template_string("index", INDEX_TEMPLATE)?;

    // Create application state
    let app_state = Arc::new(Mutex::new(WebAppState::new(config, handlebars)));

    // Check if token exists and initialize MCP client
    {
        let mut state = app_state.lock().await;
        if state.config.has_valid_token() {
            let token = state.config.token.clone();
            match state.init_mcp_client(token).await {
                Ok(_) => {
                    state.add_log("已加载保存的Token".to_string());
                },
                Err(e) => {
                    state.add_log(format!("加载Token失败: {}", e));
                },
            }
        }
    }

    // Build the router
    let app = Router::new()
        // Main page
        .route("/", get(index_handler))
        // API routes
        .route("/api/token", post(api_token_handler))
        .route("/api/coupons", get(api_coupons_handler))
        .route("/api/claim", post(api_claim_handler))
        .route("/api/reset", post(api_reset_handler))
        // Add state
        .with_state(app_state);

    // Try to bind to a port, starting from 8080
    let mut port = 8080u16;
    let listener = loop {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => break listener,
            Err(_) => {
                port += 1;
                if port > 9000 {
                    return Err(anyhow::anyhow!("无法找到可用端口 (8080-9000)"));
                }
            }
        }
    };

    let url = format!("http://127.0.0.1:{}", port);
    println!("HTML模式已启动，访问地址: {}", url);

    // Open browser in incognito/private mode
    open_browser_incognito(&url);

    // Serve the app
    axum::serve(listener, app).await?;

    Ok(())
}

/// Open browser in incognito/private mode
fn open_browser_incognito(url: &str) {
    #[cfg(target_os = "macos")]
    {
        // Try Chrome first, then Firefox, then Safari
        // Format: open "URL" -a "Browser" --args --incognito
        let browsers = [
            ("Google Chrome", "--incognito"),
            ("Google Chrome Canary", "--incognito"),
            ("Chromium", "--incognito"),
            ("Firefox", "--private-window"),
        ];

        for (browser, flag) in browsers {
            let result = std::process::Command::new("open")
                .arg(url)
                .arg("-a")
                .arg(browser)
                .arg("--args")
                .arg(flag)
                .spawn();

            if result.is_ok() {
                println!("已在 {} 中打开 (无痕模式)", browser);
                return;
            }
        }

        // Fallback: just open with default browser
        if std::process::Command::new("open").arg(url).spawn().is_ok() {
            println!("已在默认浏览器中打开");
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: Try to find browsers in common locations
        let chrome_paths = [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ];
        let edge_paths = [
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        ];
        let firefox_paths = [
            r"C:\Program Files\Mozilla Firefox\firefox.exe",
            r"C:\Program Files (x86)\Mozilla Firefox\firefox.exe",
        ];

        // Try Chrome
        for path in chrome_paths {
            if std::path::Path::new(path).exists() {
                if std::process::Command::new(path)
                    .args(["--incognito", url])
                    .spawn()
                    .is_ok()
                {
                    println!("已在 Chrome 中打开 (无痕模式)");
                    return;
                }
            }
        }

        // Try Edge (default on Windows 10/11)
        for path in edge_paths {
            if std::path::Path::new(path).exists() {
                if std::process::Command::new(path)
                    .args(["--inprivate", url])
                    .spawn()
                    .is_ok()
                {
                    println!("已在 Edge 中打开 (InPrivate 模式)");
                    return;
                }
            }
        }

        // Try Firefox
        for path in firefox_paths {
            if std::path::Path::new(path).exists() {
                if std::process::Command::new(path)
                    .args(["--private-window", url])
                    .spawn()
                    .is_ok()
                {
                    println!("已在 Firefox 中打开 (隐私模式)");
                    return;
                }
            }
        }

        // Fallback: open with default browser using start command
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
        println!("已在默认浏览器中打开");
    }

    #[cfg(target_os = "linux")]
    {
        let browsers = [
            ("google-chrome", "--incognito"),
            ("chromium-browser", "--incognito"),
            ("firefox", "--private-window"),
        ];

        for (browser, flag) in browsers {
            if std::process::Command::new(browser)
                .args([flag, url])
                .spawn()
                .is_ok()
            {
                println!("已在浏览器中打开 (无痕模式)");
                return;
            }
        }
        // Fallback
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        println!("已在默认浏览器中打开");
    }
}

/// Handler for the index page
async fn index_handler(State(state): State<Arc<Mutex<WebAppState>>>) -> impl IntoResponse {
    let state = state.lock().await;

    // Render main page with has_token flag
    let view = AppStateView::from_state(&state);
    match state.handlebars.render("index", &view) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("模板渲染错误: {}", e);
            Html(format!("<h1>Error</h1><p>{}</p>", e)).into_response()
        }
    }
}

/// API handler for token submission
async fn api_token_handler(
    State(state): State<Arc<Mutex<WebAppState>>>,
    Json(payload): Json<TokenPayload>,
) -> impl IntoResponse {
    let mut state = state.lock().await;

    // Format token if needed
    let formatted_token = if payload.token.starts_with("Bearer ") {
        payload.token.clone()
    } else {
        format!("Bearer {}", payload.token)
    };

    // Validate token
    match McpClient::new(formatted_token.clone()) {
        Ok(client) => {
            match client.validate_token().await {
                Ok(true) => {
                    // Save token
                    state.config.token = formatted_token.clone();
                    state.config.save().ok();

                    // Initialize MCP client
                    state.mcp_client = Some(Arc::new(Mutex::new(client)));

                    // Add logs
                    state.add_log("Token验证成功！".to_string());
                    state.add_log("配置已保存到当前目录".to_string());

                    Json(ApiResponse {
                        success: true,
                        message: "Token验证成功！".to_string(),
                        coupons: None,
                    })
                },
                Ok(false) => {
                    state.add_log("Token无效，请重新输入".to_string());
                    Json(ApiResponse {
                        success: false,
                        message: "Token无效，请重新输入".to_string(),
                        coupons: None,
                    })
                },
                Err(e) => {
                    state.add_log(format!("验证失败: {}", e));
                    Json(ApiResponse {
                        success: false,
                        message: format!("验证失败: {}", e),
                        coupons: None,
                    })
                }
            }
        },
        Err(e) => {
            state.add_log(format!("创建客户端失败: {}", e));
            Json(ApiResponse {
                success: false,
                message: format!("创建客户端失败: {}", e),
                coupons: None,
            })
        }
    }
}

/// Parse markdown text to extract coupons
fn parse_coupons_from_markdown(text: &str) -> Vec<Coupon> {
    let mut coupons = Vec::new();
    let mut current_title = String::new();
    let mut current_price = String::new();
    let mut current_expiry = String::new();
    let mut current_receive_time = String::new();
    let mut current_tags = String::new();
    let mut current_image_url = String::new();

    for line in text.lines() {
        let line = line.trim();

        // Skip empty lines and header
        if line.is_empty() || line.starts_with("# ") || line.starts_with("共 ") {
            continue;
        }

        // New coupon title (## 标题)
        if line.starts_with("## ") {
            // Save previous coupon if exists
            if !current_title.is_empty() {
                coupons.push(Coupon {
                    title: current_title.clone(),
                    price: current_price.clone(),
                    expiry: current_expiry.clone(),
                    receive_time: current_receive_time.clone(),
                    tags: current_tags.clone(),
                    image_url: current_image_url.clone(),
                });
            }
            // Start new coupon
            current_title = line.trim_start_matches("## ").to_string();
            current_price.clear();
            current_expiry.clear();
            current_receive_time.clear();
            current_tags.clear();
            current_image_url.clear();
        }
        // Parse coupon details
        else if line.starts_with("- **优惠**:") {
            current_price = line.trim_start_matches("- **优惠**:").trim().to_string();
        }
        else if line.starts_with("- **有效期**:") {
            current_expiry = line.trim_start_matches("- **有效期**:").trim().to_string();
        }
        else if line.starts_with("- **领取时间**:") {
            current_receive_time = line.trim_start_matches("- **领取时间**:").trim().to_string();
        }
        else if line.starts_with("- **标签**:") {
            current_tags = line.trim_start_matches("- **标签**:").trim().to_string();
        }
        // Parse image URL
        else if line.starts_with("<img") {
            // Extract src from <img src="..." ...>
            if let Some(start) = line.find("src=\"") {
                let rest = &line[start + 5..];
                if let Some(end) = rest.find('"') {
                    current_image_url = rest[..end].to_string();
                }
            }
        }
    }

    // Don't forget the last coupon
    if !current_title.is_empty() {
        coupons.push(Coupon {
            title: current_title,
            price: current_price,
            expiry: current_expiry,
            receive_time: current_receive_time,
            tags: current_tags,
            image_url: current_image_url,
        });
    }

    coupons
}

/// API handler for getting coupons
async fn api_coupons_handler(State(state): State<Arc<Mutex<WebAppState>>>) -> impl IntoResponse {
    let mut state = state.lock().await;

    // If no token, return error
    if state.mcp_client.is_none() {
        return Json(ApiResponse {
            success: false,
            message: "请先设置Token".to_string(),
            coupons: None,
        });
    }

    // Load coupons
    state.add_log("正在加载已领取的优惠券...".to_string());
    if let Some(client) = state.mcp_client.clone() {
        match client.lock().await.get_my_coupons().await {
            Ok(coupons_text) => {
                state.add_log(format!("原始数据: {}", coupons_text));

                // Parse markdown text to extract coupons
                let coupons = parse_coupons_from_markdown(&coupons_text);
                let coupon_count = coupons.len();

                if coupon_count > 0 {
                    state.add_log(format!("优惠券加载成功！共找到 {} 张优惠券", coupon_count));
                    state.coupons = coupons.clone();
                    return Json(ApiResponse {
                        success: true,
                        message: format!("共找到 {} 张优惠券", coupon_count),
                        coupons: Some(coupons),
                    });
                } else {
                    state.add_log("未解析到优惠券数据".to_string());
                    return Json(ApiResponse {
                        success: true,
                        message: "暂无优惠券".to_string(),
                        coupons: Some(vec![]),
                    });
                }
            },
            Err(e) => {
                state.add_log(format!("优惠券加载失败: {}", e));
                return Json(ApiResponse {
                    success: false,
                    message: format!("优惠券加载失败: {}", e),
                    coupons: None,
                });
            }
        }
    }

    Json(ApiResponse {
        success: false,
        message: "未知错误".to_string(),
        coupons: None,
    })
}

/// API handler for claiming all coupons
async fn api_claim_handler(State(state): State<Arc<Mutex<WebAppState>>>) -> impl IntoResponse {
    let mut state = state.lock().await;

    // If no token, return error
    if state.mcp_client.is_none() {
        return Json(ApiResponse {
            success: false,
            message: "请先设置Token".to_string(),
            coupons: None,
        });
    }

    // Claim all coupons
    state.add_log("正在领取所有优惠券...".to_string());
    if let Some(client) = state.mcp_client.clone() {
        match client.lock().await.auto_bind_coupons().await {
            Ok(result) => {
                state.add_log("领取成功！".to_string());
                // Add result to logs
                for line in result.lines().take(5) {
                    if !line.trim().is_empty() {
                        state.add_log(line.to_string());
                    }
                }
                // Clear cached coupons so they will be reloaded
                state.coupons.clear();
                return Json(ApiResponse {
                    success: true,
                    message: "领取成功！".to_string(),
                    coupons: None,
                });
            },
            Err(e) => {
                state.add_log(format!("领取失败: {}", e));
                return Json(ApiResponse {
                    success: false,
                    message: format!("领取失败: {}", e),
                    coupons: None,
                });
            }
        }
    }

    Json(ApiResponse {
        success: false,
        message: "未知错误".to_string(),
        coupons: None,
    })
}

/// API handler for resetting token
async fn api_reset_handler(State(state): State<Arc<Mutex<WebAppState>>>) -> impl IntoResponse {
    let mut state = state.lock().await;

    // Clear token
    state.mcp_client = None;

    // Remove token from config
    state.config.token = String::new();
    state.config.save().ok();

    // Clear coupons
    state.coupons.clear();

    // Add logs
    state.add_log("Token已重置".to_string());
    state.add_log("请输入新的MCP Token".to_string());

    Json(ApiResponse {
        success: true,
        message: "Token已重置".to_string(),
        coupons: None,
    })
}

/// Payload for token API
#[derive(Debug, Deserialize)]
pub struct TokenPayload {
    pub token: String,
}
