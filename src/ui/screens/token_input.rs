use ratatui::{widgets::*, style::*, layout::*};
use ratatui::{Frame, backend::Backend};
use crate::ui::{app::App, screens::ScreenType};
use anyhow::Result;
use crate::config::Config;

/// Token input screen
#[derive(Clone)]
pub struct TokenInputScreen {
    pub input: String,
    pub error_message: Option<String>,
}

impl TokenInputScreen {
    /// Create a new token input screen
    pub fn new() -> Self {
        Self {
            input: String::new(),
            error_message: None,
        }
    }
    
    /// Handle keyboard input
    pub async fn handle_key(mut self, key: crossterm::event::KeyEvent, app: &mut App) -> Result<ScreenType> {
        match key.code {
            crossterm::event::KeyCode::Char(c) => {
                self.input.push(c);
                Ok(ScreenType::TokenInput(self))
            },
            crossterm::event::KeyCode::Backspace => {
                self.input.pop();
                Ok(ScreenType::TokenInput(self))
            },
            crossterm::event::KeyCode::Enter => {
                // Validate input
                if self.input.is_empty() {
                    self.error_message = Some("Token不能为空".to_string());
                    return Ok(ScreenType::TokenInput(self));
                }
                
                // Format token with Bearer prefix if needed
                let formatted_token = if self.input.starts_with("Bearer ") {
                    self.input.to_string()
                } else {
                    format!("Bearer {}", self.input)
                };
                
                // Validate token
                app.set_loading(true, 50);
                
                let client = crate::mcp::McpClient::new(formatted_token.clone())?;
                let validation_result = client.validate_token().await;
                
                app.set_loading(false, 0);

                match validation_result {
                    Ok(true) => {
                        // Save token to config
                        let mut config = Config::load()?;
                        config.token = formatted_token.clone();
                        config.save()?;

                        // Initialize MCP client
                        app.init_mcp_client(formatted_token)?;
                        app.add_log("Token验证成功！".to_string());
                        app.add_log("配置已保存到当前目录".to_string());

                        // Switch to main screen
                        Ok(ScreenType::Main(crate::ui::screens::MainScreen::new()))
                    }
                    Ok(false) => {
                        self.error_message = Some("Token无效，请重新输入".to_string());
                        Ok(ScreenType::TokenInput(self))
                    }
                    Err(e) => {
                        self.error_message = Some(format!("验证失败: {}", e));
                        Ok(ScreenType::TokenInput(self))
                    }
                }
            },
            crossterm::event::KeyCode::Esc => {
                // Exit application
                // app.running is not available, we'll exit through the main loop
                std::process::exit(0);
            },
            _ => Ok(ScreenType::TokenInput(self)),
        }
    }
    
    /// Render the token input screen
    pub fn render(&self, f: &mut Frame<'_>, app: &App) {
        let size = f.size();
        
        // Create vertical layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(size);
        
        // Title
        let title = Paragraph::new("欢迎使用麦当劳优惠券自动领取工具")
            .block(Block::default().borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(title, layout[0]);
        
        // Subtitle
        let subtitle = Paragraph::new("请输入您的MCP Token:")
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(subtitle, layout[1]);
        
        // Token input field
        let input_field = Paragraph::new(self.input.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .title("MCP Token")
                .style(Style::default().fg(Color::Cyan)))
            .style(Style::default().fg(Color::White));
        f.render_widget(input_field, layout[2]);
        
        // Error message
        if let Some(ref error) = self.error_message {
            let error_widget = Paragraph::new(error.as_str())
                .block(Block::default().borders(Borders::NONE))
                .style(Style::default().fg(Color::Red))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(error_widget, layout[3]);
        }
        
        // Help text
        let help_text = Paragraph::new("按 Enter 确认，Esc 退出")
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::Yellow))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(help_text, layout[4]);
        
        // Logs area
        let logs_title = Paragraph::new("日志信息")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Green))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(logs_title, layout[5]);
    }
}