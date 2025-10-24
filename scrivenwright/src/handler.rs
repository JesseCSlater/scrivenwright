use crate::app::{App, PlatformAdapter};
use crate::settings::{DEFAULT_TEXT_WIDTH_PERCENT, FULL_TEXT_WIDTH_PERCENT};
use crate::text::OpenText;
use crate::text_wrapper::Dir;

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
            (_, C::Up) => {
                text.move_cursor(self.settings.line_width(terminal_width), Dir::Up);
            }
            (_, C::Down) => {
                text.move_cursor(self.settings.line_width(terminal_width), Dir::Down);
            }
            (_, C::Left) => {
                text.move_cursor(self.settings.line_width(terminal_width), Dir::Left);
            }
            (_, C::Right) => {
                text.move_cursor(self.settings.line_width(terminal_width), Dir::Right);
            }
            (_, C::Esc) => text.snap_to_cursor(),
            _ => {}
        }
    }
}
