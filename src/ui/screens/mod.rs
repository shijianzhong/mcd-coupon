use ratatui::{Frame, backend::Backend};
use crate::ui::app::App;

/// Trait that all screens must implement
pub trait Screen {
    /// Handle keyboard input
    async fn handle_key(self, key: crossterm::event::KeyEvent, app: &mut App) -> anyhow::Result<ScreenType>;
    
    /// Render the screen
    fn render(&self, f: &mut Frame<'_>, app: &App);
}

/// Enum representing all possible screen types
#[derive(Clone)]
pub enum ScreenType {
    TokenInput(TokenInputScreen),
    Main(MainScreen),
}

/// Implement Screen trait for ScreenType
impl Screen for ScreenType {
    async fn handle_key(self, key: crossterm::event::KeyEvent, app: &mut App) -> anyhow::Result<ScreenType> {
        match self {
            ScreenType::TokenInput(screen) => screen.handle_key(key, app).await,
            ScreenType::Main(screen) => screen.handle_key(key, app).await,
        }
    }
    
    fn render(&self, f: &mut Frame<'_>, app: &App) {
        match self {
            ScreenType::TokenInput(screen) => screen.render(f, app),
            ScreenType::Main(screen) => screen.render(f, app),
        }
    }
}

pub mod main_screen;
pub mod token_input;

pub use main_screen::MainScreen;
pub use token_input::TokenInputScreen;