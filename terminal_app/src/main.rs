use scrivenwright::app::{App, AppResult, Test, KeyPress};
use scrivenwright::handler::handle_key_events;
use scrivenwright::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::{env, io, fs};

pub mod event;
pub mod file_sys;

use crate::event::*;

fn main() -> AppResult<()> {
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;

    let book_title = if let Some(arg_1) = env::args().collect::<Vec<_>>().get(1) {
        arg_1.clone()
    } else {
        println!("Please provide the name of a book in the ~/.booktyping directory");
        return Ok(());
    };

    let _ = fs::create_dir(
        dirs::home_dir()
            .unwrap()
            .join(".booktyping")
            .join(&book_title),
    );

    let book_text = file_sys::load_book(&book_title).expect("Failed to load book");

    let tests = file_sys::load_tests(&book_title).expect("Failed to load tests");

    let save = move |tests: Vec<Test>, keypresses : Vec<KeyPress>| {
        file_sys::save_tests(&book_title, &tests)?;
        file_sys::save_keypresses(&book_title, &keypresses)?;
        Ok(())
    };

    let mut app = App::new(terminal.size()?.width,  book_text, tests, save)?;

    let events = EventHandler::new(250);

    let mut tui = Tui::new(terminal);

    tui.init()?;
    tui.draw(&mut app)?; //Draw first frame

    // Start the main loop.
    while app.running {
        // Handle events.
        match events.next()? {
            Event::Key(key_event) => {
                handle_key_events(key_event, &mut app)?;
                tui.draw(&mut app)?;
            }
            Event::Resize(width, _) => {
                app.terminal_width = width;
                app.generate_lines();
                tui.draw(&mut app)?;
            }
        }
    }

    tui.exit()?;
    Ok(())
}