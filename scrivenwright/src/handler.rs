use crate::app::{App, AppResult, DEFAULT_TEXT_WIDTH_PERCENT, FULL_TEXT_WIDTH_PERCENT};
#[derive(Debug, Copy, Clone)]
pub struct KeyDown {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}
#[derive(Debug, Copy, Clone)]
pub enum KeyCode {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Esc,
    Unimplemented,
}
#[derive(Debug, Copy, Clone)]
pub enum KeyModifiers {
    Ctrl,
    Unimplemented,
}

impl App {
    pub fn handle_key_events(&mut self, key_press: KeyDown) -> AppResult<()> {
        use KeyCode as C;
        use KeyModifiers as M;
        match (key_press.mods, key_press.code) {
            (M::Ctrl, C::Char('c')) => self.quit()?,
            (M::Ctrl, C::Char('f')) => {
                self.full_text_width = !self.full_text_width;
                self.text_width_percent = if self.full_text_width {
                    FULL_TEXT_WIDTH_PERCENT
                } else {
                    DEFAULT_TEXT_WIDTH_PERCENT
                };
                self.generate_lines()
            }
            (_, C::Char(c)) => self.handle_char(c)?,
            (M::Ctrl, C::Up) => {
                self.display_line = self.display_line.checked_sub(10).unwrap_or_default();
            }
            (M::Ctrl, C::Down) => {
                self.display_line += 10;
            }
            (_, C::Up) => {
                self.display_line = self.display_line.checked_sub(1).unwrap_or_default();
            }
            (_, C::Down) => {
                self.display_line += 1;
            }
            (_, C::Esc) => {
                let &(cur_line, _) = self
                    .line_index
                    .get(self.sample_start_index + self.text.cur_char)
                    .unwrap();
                self.display_line = cur_line
            }
            _ => {}
        }
        Ok(())
    }
}
