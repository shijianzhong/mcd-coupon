use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{backend::Backend, Frame, Terminal};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

use crate::{mcp::McpClient, ui::screens::{Screen, ScreenType, TokenInputScreen}};

/// Application state and logic
pub struct App {
    pub current_screen: ScreenType,
    pub mcp_client: Option<Arc<Mutex<McpClient>>>,
    pub logs: Vec<String>,
    pub progress: u16,
    pub is_loading: bool,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            current_screen: ScreenType::TokenInput(TokenInputScreen::new()),
            mcp_client: None,
            logs: vec!["应用已启动...".to_string()],
            progress: 0,
            is_loading: false,
        }
    }

    /// Run the application main loop
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // Render current screen
            terminal.draw(|f| self.render(f))?;

            // Handle events
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            break;
                        }
                    }
                    _ => {
                        self.current_screen = self.current_screen.clone().handle_key(key, self).await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Render the current screen
    fn render(&self, f: &mut Frame<'_>) {
        self.current_screen.render(f, self);
    }

    /// Add a log message
    pub fn add_log(&mut self, message: String) {
        self.logs.push(message);
        // Keep only the last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool, progress: u16) {
        self.is_loading = loading;
        self.progress = progress;
    }

    /// Initialize MCP client with token
    pub fn init_mcp_client(&mut self, token: String) -> Result<()> {
        let client = McpClient::new(token)?;
        self.mcp_client = Some(Arc::new(Mutex::new(client)));
        Ok(())
    }
}
