use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{Frame, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, Paragraph, List, ListItem, Gauge}};
use anyhow::Result;
use crate::{ui::{App, ScreenType}};

/// Main application screen with coupon management features
#[derive(Clone)]
pub struct MainScreen {
    pub selected_option: usize,
    pub show_coupons: bool,
    pub coupons: Vec<String>,
}

impl MainScreen {
    /// Create a new main screen instance
    pub fn new() -> Self {
        Self {
            selected_option: 0,
            show_coupons: false,
            coupons: Vec::new(),
        }
    }

    /// Handle keyboard input for the main screen
    pub async fn handle_key(mut self, key: KeyEvent, app: &mut App) -> Result<ScreenType> {
        match key.code {
            KeyCode::Up => {
                if self.selected_option > 0 {
                    self.selected_option -= 1;
                }
            },
            KeyCode::Down => {
                if self.selected_option < 2 {
                    self.selected_option += 1;
                }
            },
            KeyCode::Enter => {
                if let Some(new_screen) = self.handle_option_selection(app).await? {
                    return Ok(new_screen);
                }
            },
            KeyCode::Char('1') => {
                self.selected_option = 0;
                if let Some(new_screen) = self.handle_option_selection(app).await? {
                    return Ok(new_screen);
                }
            },
            KeyCode::Char('2') => {
                self.selected_option = 1;
                if let Some(new_screen) = self.handle_option_selection(app).await? {
                    return Ok(new_screen);
                }
            },
            KeyCode::Char('3') => {
                self.selected_option = 2;
                if let Some(new_screen) = self.handle_option_selection(app).await? {
                    return Ok(new_screen);
                }
            },
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.show_coupons = !self.show_coupons;
                if self.show_coupons {
                    self.load_coupons(app).await?;
                }
            },
            _ => {},
        }
        Ok(ScreenType::Main(self))
    }

    /// Handle option selection
    async fn handle_option_selection(&mut self, app: &mut App) -> Result<Option<ScreenType>> {
        match self.selected_option {
            0 => {
                self.claim_all_coupons(app).await?;
                Ok(None)
            },
            1 => {
                self.show_coupons = true;
                self.load_coupons(app).await?;
                Ok(None)
            },
            2 => {
                let new_screen = self.reset_token(app);
                Ok(Some(new_screen))
            },
            _ => {
                Ok(None)
            },
        }
    }

    /// Claim all available coupons
    async fn claim_all_coupons(&mut self, app: &mut App) -> Result<()> {
        // First, clone the client if it exists
        if let Some(client) = app.mcp_client.clone() {
            app.set_loading(true, 0);
            app.add_log("正在领取所有优惠券...".to_string());
            
            let result = client.lock().await.auto_bind_coupons().await;
            
            app.set_loading(false, 100);
            
            match result {
                Ok(response) => {
                    app.add_log("领取成功！".to_string());
                    // Response is markdown text, show first few lines as summary
                    for line in response.lines().take(5) {
                        if !line.trim().is_empty() {
                            app.add_log(line.to_string());
                        }
                    }
                },
                Err(e) => {
                    app.add_log(format!("领取失败: {}", e));
                },
            }
        }
        Ok(())
    }

    /// Load user's coupons
    async fn load_coupons(&mut self, app: &mut App) -> Result<()> {
        // First, clone the client if it exists
        if let Some(client) = app.mcp_client.clone() {
            app.set_loading(true, 0);
            app.add_log("正在加载已领取的优惠券...".to_string());
            
            let result = client.lock().await.get_my_coupons().await;
            
            app.set_loading(false, 100);
            
            match result {
                Ok(coupons_text) => {
                    self.coupons.clear();
                    // Response is markdown text, split by lines for display
                    let lines: Vec<&str> = coupons_text.lines().collect();
                    let coupon_count = lines.iter().filter(|l| l.starts_with("- ") || l.starts_with("* ")).count();
                    for line in lines {
                        if !line.trim().is_empty() {
                            self.coupons.push(line.to_string());
                        }
                    }
                    app.add_log(format!("已加载优惠券列表 (约 {} 项)", coupon_count));
                },
                Err(e) => {
                    app.add_log(format!("加载失败: {}", e));
                    self.coupons.push(format!("加载失败: {}", e));
                },
            }
        }
        Ok(())
    }

    /// Reset the token and return to token input screen
    fn reset_token(&mut self, app: &mut App) -> ScreenType {
        // Clear client and config
        app.mcp_client = None;
        
        // Remove token from config
        if let Ok(mut config) = crate::config::Config::load() {
            config.token = String::new();
            config.save().ok();
        }
        
        app.add_log("Token已重置".to_string());
        app.add_log("请输入新的MCP Token".to_string());
        
        // Return to token input screen
        ScreenType::TokenInput(crate::ui::screens::TokenInputScreen::new())
    }

    /// Render the main screen
    pub fn render(&self, f: &mut Frame<'_>, app: &App) {
        let size = f.size();
        
        // Create vertical layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(8),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(size);

        // Title
        let title = Paragraph::new("麦当劳优惠券自动领取工具")
            .block(Block::default().borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(title, main_layout[0]);

        // Main menu options
        let options = [
            "[1] 一键领取所有优惠券",
            "[2] 查看已领取优惠券",
            "[3] 重新设置Token",
        ];
        
        let items: Vec<ListItem> = options.iter()
            .enumerate()
            .map(|(i, option)| {
                let style = if i == self.selected_option {
                    ratatui::style::Style::default()
                        .bg(ratatui::style::Color::Green)
                        .fg(ratatui::style::Color::Black)
                        .add_modifier(ratatui::style::Modifier::BOLD)
                } else {
                    ratatui::style::Style::default()
                };
                ListItem::new(*option).style(style)
            })
            .collect();
        
        let menu = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("菜单选项"));
        
        f.render_widget(menu, main_layout[1]);

        // Content area - logs or coupons
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(main_layout[2]);
        
        // Logs panel
        let logs_block = Block::default()
            .borders(Borders::ALL)
            .title("操作日志");
        
        let log_items: Vec<ListItem> = app.logs.iter()
            .rev()
            .take(10)
            .map(|log| ListItem::new(log.clone()))
            .collect();
        
        let logs_list = List::new(log_items)
            .block(logs_block);
        
        f.render_widget(logs_list, content_layout[0]);
        
        // Coupons panel
        let coupons_block = Block::default()
            .borders(Borders::ALL)
            .title("我的优惠券");
        
        if self.show_coupons {
            let coupon_items: Vec<ListItem> = self.coupons.iter()
                .map(|coupon| ListItem::new(coupon.clone()))
                .collect();
            
            let coupons_list = List::new(coupon_items)
                .block(coupons_block);
            
            f.render_widget(coupons_list, content_layout[1]);
        } else {
            let hint = Paragraph::new("按 'c' 查看已领取的优惠券")
                .block(coupons_block)
                .alignment(ratatui::layout::Alignment::Center);
            
            f.render_widget(hint, content_layout[1]);
        }

        // Status bar
        let status_text = if app.is_loading {
            "加载中..."
        } else {
            "按 'q' 退出 | 按方向键选择选项 | 按 Enter 执行"
        };
        
        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status, main_layout[3]);
        
        // Progress bar if loading
        if app.is_loading {
            let progress_block = Block::default()
                .borders(Borders::NONE)
                .title("进度");
            
            let gauge = Gauge::default()
                .block(progress_block)
                .gauge_style(ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Green)
                    .bg(ratatui::style::Color::Black)
                    .add_modifier(ratatui::style::Modifier::BOLD))
                .percent(app.progress.into());
            
            let progress_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
                .split(main_layout[3]);
            
            f.render_widget(gauge, progress_layout[0]);
        }
    }
}
