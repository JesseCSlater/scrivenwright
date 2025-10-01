use crate::app::{
    App, OpenText, PlatformAdapter, DEFAULT_TEXT_WIDTH_PERCENT, FULL_TEXT_WIDTH_PERCENT,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct KeyDown {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyCode {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Esc,
    Unimplemented,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyModifiers {
    Ctrl,
    Unimplemented,
}

//TODO extract behavior into functions so this is just a key map.
impl<PA: PlatformAdapter> App<PA> {
    pub fn handle_key_events(
        &mut self,
        key_press: KeyDown,
        text: &mut OpenText,
        terminal_width: u16,
    ) {
        use KeyCode as C;
        use KeyModifiers as M;
        match (key_press.mods, key_press.code) {
            (M::Ctrl, C::Char('c')) => self.quit(),
            (M::Ctrl, C::Char('f')) => {
                self.settings.full_text_width = !self.settings.full_text_width;
                self.settings.text_width_percent = if self.settings.full_text_width {
                    FULL_TEXT_WIDTH_PERCENT
                } else {
                    DEFAULT_TEXT_WIDTH_PERCENT
                };
            }
            (_, C::Char(c)) => text.handle_char(c),
            (M::Ctrl, C::Up) => {
                text.scroll(self.settings.line_width(terminal_width), -10);
            }
            (M::Ctrl, C::Down) => {
                text.scroll(self.settings.line_width(terminal_width), 10);
            }
            (_, C::Up) => {
                text.scroll(self.settings.line_width(terminal_width), -1);
            }
            (_, C::Down) => {
                text.scroll(self.settings.line_width(terminal_width), 1);
            }
            (_, C::Esc) => text.snap_to_cursor(),
            _ => {}
        }
    }
}
