pub const DEFAULT_TEXT_WIDTH_PERCENT: u16 = 60;
pub const FULL_TEXT_WIDTH_PERCENT: u16 = 96;

pub struct Settings {
    pub text_width_percent: u16,
    pub full_text_width: bool,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            text_width_percent: DEFAULT_TEXT_WIDTH_PERCENT,
            full_text_width: false,
        }
    }

    pub(crate) fn line_width(&self, terminal_width: u16) -> u16 {
        ((terminal_width as usize) * (self.text_width_percent as usize) / 100) as u16
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
