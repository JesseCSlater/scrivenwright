use crate::app::{App, AppResult, DEFAULT_TEXT_WIDTH_PERCENT, FULL_TEXT_WIDTH_PERCENT};
pub struct KeyDown {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}
pub enum KeyCode {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Esc,
    Unimplemented,
}
pub enum KeyModifiers {
    Shift,
    Ctrl,
    Alt,
    Unimplemented,
}

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_press: KeyDown, app: &mut App) -> AppResult<()> {
    use KeyCode as C;
    use KeyModifiers as M;
    match (key_press.mods, key_press.code) {
        (M::Ctrl, C::Char('c')) => app.quit()?,
        (M::Ctrl, C::Char('f')) => {
            app.full_text_width = !app.full_text_width;
            app.text_width_percent = if app.full_text_width {
                FULL_TEXT_WIDTH_PERCENT
            } else {
                DEFAULT_TEXT_WIDTH_PERCENT
            };
            app.generate_lines()
        }
        (_, C::Char(c)) => app.handle_char(c)?,
        (M::Ctrl, C::Up) => {
            app.following_typing = false;
            app.display_line = app.display_line.checked_sub(10).unwrap_or_default();
        }
        (M::Ctrl, C::Down) => {
            app.following_typing = false;
            app.display_line += 10;
        }
        (_, C::Up) => {
            app.following_typing = false;
            app.display_line = app.display_line.checked_sub(1).unwrap_or_default();
        }
        (_, C::Down) => {
            app.following_typing = false;
            app.display_line += 1;
        }
        (_, C::Esc) => {
            app.following_typing = true;
        }
        _ => {}
    }
    Ok(())
}
