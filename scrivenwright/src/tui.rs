use crate::app::{App, AppResult};

use ratatui::backend::Backend;
use ratatui::Terminal;

#[derive(Debug)]
pub struct Tui<B>
where
    B: Backend,
{
    terminal: Terminal<B>,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>) -> Self {
        Self { terminal }
    }

    pub fn init(&mut self) -> AppResult<()> {
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> AppResult<()> {
        self.terminal.draw(|frame| app.render(frame))?;
        Ok(())
    }

    pub fn exit(&mut self) -> AppResult<()> {
        self.terminal.show_cursor()?;
        Ok(())
    }
}
