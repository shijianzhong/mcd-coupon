use anyhow::Result;
use std::io::{self, Write};

// Import TUI dependencies
use crossterm::{terminal::{EnterAlternateScreen, LeaveAlternateScreen}, execute, event::{EnableMouseCapture, DisableMouseCapture}};
use ratatui::{backend::CrosstermBackend, Terminal};

mod config;
mod mcp;
mod mcp_server;
mod ui;
mod utils;
mod web;

/// Application mode
#[derive(Debug, Clone)]
enum Mode {
    /// Terminal User Interface mode
    Tui,
    /// HTML web interface mode
    Html,
    /// MCP Server mode
    McpServer,
}

fn main() -> Result<()> {
    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();

    let mode = if args.len() > 1 {
        // Parse command line argument
        match args[1].to_lowercase().as_str() {
            "tui" | "-tui" | "--tui" | "1" => Mode::Tui,
            "html" | "-html" | "--html" | "web" | "-web" | "--web" | "2" => Mode::Html,
            "mcpserver" | "-mcpserver" | "--mcpserver" | "mcp-server" | "3" => Mode::McpServer,
            "-h" | "--help" | "help" => {
                print_help();
                return Ok(());
            }
            _ => {
                println!("未知参数: {}", args[1]);
                print_help();
                return Ok(());
            }
        }
    } else {
        // No arguments - show interactive menu
        show_mode_menu()?
    };

    match mode {
        Mode::Tui => {
            run_tui_mode()?;
        },
        Mode::Html => {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(web::run())?;
        },
        Mode::McpServer => {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(run_mcp_server_mode())?;
        },
    }

    Ok(())
}

/// Print help information
fn print_help() {
    println!();
    println!("麦当劳优惠券自动领取工具");
    println!();
    println!("用法:");
    println!("  mcd-coupon          交互式选择模式");
    println!("  mcd-coupon tui      终端界面模式");
    println!("  mcd-coupon html     网页界面模式");
    println!("  mcd-coupon mcpserver MCP服务器模式");
    println!("  mcd-coupon --help   显示帮助信息");
    println!();
}

/// Show interactive mode selection menu
fn show_mode_menu() -> Result<Mode> {
    println!();
    println!("╔════════════════════════════════════════╗");
    println!("║    麦当劳优惠券自动领取工具            ║");
    println!("╠════════════════════════════════════════╣");
    println!("║                                        ║");
    println!("║  请选择运行模式:                       ║");
    println!("║                                        ║");
    println!("║  [1] 网页模式 (推荐小白用户)           ║");
    println!("║      浏览器打开，界面友好              ║");
    println!("║                                        ║");
    println!("║  [2] 终端模式 (TUI)                    ║");
    println!("║      在终端中运行，适合高级用户        ║");
    println!("║                                        ║");
    println!("║  [3] MCP服务器模式                     ║");
    println!("║      提供优惠券MCP工具服务             ║");
    println!("║                                        ║");
    println!("╚════════════════════════════════════════╝");
    println!();
    print!("请输入选项 [1/2/3] (默认1): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    match input {
        "" | "1" | "html" | "web" => {
            println!();
            println!("正在启动网页模式...");
            Ok(Mode::Html)
        }
        "2" | "tui" => {
            println!();
            println!("正在启动终端模式...");
            Ok(Mode::Tui)
        }
        "3" | "mcpserver" | "mcp-server" => {
            println!();
            println!("正在启动MCP服务器模式...");
            Ok(Mode::McpServer)
        }
        _ => {
            println!();
            println!("无效选项，默认启动网页模式...");
            Ok(Mode::Html)
        }
    }
}

/// Run the application in TUI mode
fn run_tui_mode() -> Result<()> {
    // Set up terminal
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    crossterm::terminal::enable_raw_mode()?;

    // Create backend and terminal
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load configuration
    let config = config::Config::load()?;

    // Initialize application
    let mut app = ui::App::new();

    // Set up MCP client if token exists
    if config.has_valid_token() {
        match mcp::McpClient::new(config.token.clone()) {
            Ok(client) => {
                app.mcp_client = Some(std::sync::Arc::new(tokio::sync::Mutex::new(client)));
                app.current_screen = ui::screens::ScreenType::Main(ui::screens::MainScreen::new());
                app.add_log("已加载保存的Token".to_string());
            },
            Err(e) => {
                app.add_log(format!("加载Token失败: {}", e));
            },
        }
    } else {
        // If no valid token, start with token input screen
        app.current_screen = ui::screens::ScreenType::TokenInput(ui::screens::TokenInputScreen::new());
    }

    // Run application
    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(app.run(&mut terminal));

    // Clean up
    crossterm::terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

/// Run the application in MCP Server mode
async fn run_mcp_server_mode() -> Result<()> {
    // Load configuration
    let config = config::Config::load()?;

    // Check if valid token exists
    if !config.has_valid_token() {
        println!("错误: 未找到有效的MCP Token");
        println!("请先在配置文件中设置有效的Token，或使用其他模式获取Token");
        println!("配置文件位置: {}", config::Config::get_config_path().display());
        return Ok(());
    }

    // Initialize MCP client
    let mcp_client = match mcp::McpClient::new(config.token.clone()) {
        Ok(client) => client,
        Err(e) => {
            println!("初始化MCP客户端失败: {}", e);
            return Ok(());
        },
    };

    // Start MCP server
    mcp_server::run_mcp_server(config, mcp_client).await?;

    Ok(())
}
