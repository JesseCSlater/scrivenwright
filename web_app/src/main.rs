#![warn(rust_2018_idioms)]
#![deny(
    rustdoc::broken_intra_doc_links,
    unreachable_pub,
    unreachable_patterns,
    unused,
    unused_qualifications,
    while_true,
    trivial_casts,
    trivial_bounds,
    trivial_numeric_casts,
    unconditional_panic,
    clippy::all
)]

use std::{
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard, OnceLock},
};

use app::TermApp;
use ratatui::prelude::*;
use send_wrapper::SendWrapper;
use terminal::WebTerm;

pub mod app;
pub mod terminal;

pub static TERMINAL: Renderer = Renderer::new();

pub struct Renderer(OnceLock<Mutex<SendWrapper<Terminal<WebTerm>>>>);

impl Renderer {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn load(&self) {
        self.0
            .set(Mutex::new(SendWrapper::new(
                Terminal::new(WebTerm::new()).unwrap(),
            )))
            .unwrap();
    }

    pub fn term(&'static self) -> impl 'static + DerefMut<Target = Terminal<WebTerm>> {
        TermDeref(self.0.get().unwrap().lock().unwrap())
    }
}

struct TermDeref<'a>(MutexGuard<'a, SendWrapper<Terminal<WebTerm>>>);

impl<'a> Deref for TermDeref<'a> {
    type Target = Terminal<WebTerm>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for TermDeref<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn main() {
    TERMINAL.load();
    yew::Renderer::<TermApp>::new().render();
}
