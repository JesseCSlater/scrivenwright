use crate::settings::Settings;
use chrono::{DateTime, Utc};
use std::error;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct TextHandle {
    pub name: String,
    pub length: usize,
    pub last_opened: DateTime<Utc>,
    open: Box<dyn FnOnce()>,
}

pub trait PlatformAdapter {
    fn get_texts(&self) -> Vec<TextHandle>;
}

impl PlatformAdapter for () {
    fn get_texts(&self) -> Vec<TextHandle> {
        Vec::new()
    }
}

pub struct App<PA: PlatformAdapter> {
    pub running: bool,
    pub adapter: PA,
    pub settings: Settings,
}

impl<PA: PlatformAdapter> App<PA> {
    pub fn new(adapter: PA) -> Self {
        Self {
            adapter,
            settings: Settings::default(),
            running: true,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
