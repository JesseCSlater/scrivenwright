use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use scrivenwright::app::{App, AppResult, KeyPress, Test, Text};
use std::panic;
use std::{env, io};

pub mod event;
pub mod file_sys;

use crate::event::*;

fn main() -> AppResult<()> {
    let backend = CrosstermBackend::new(io::stderr());
    let mut terminal = Terminal::new(backend)?;

    let book_title = if let Some(arg_1) = env::args().collect::<Vec<_>>().get(1) {
        arg_1.clone()
    } else {
        println!("Please provide the name of a book in the ~/scrivenwright directory");
        return Ok(());
    };

    crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal::enable_raw_mode()?;

    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture);
        panic_hook(panic);
    }));

    let _ = file_sys::create_book_dir(&book_title);

    let book_text = file_sys::load_book(&book_title).expect("Failed to load book");

    let tests = file_sys::load_tests(&book_title).expect("Failed to load tests");

    let save = move |tests: Vec<Test>, keypresses: Vec<KeyPress>| {
        file_sys::save_tests(&book_title, &tests)?;
        file_sys::save_keypresses(&book_title, &keypresses)?;
        Ok(())
    };

    let text = Text::new(&book_text, tests, save, 0, 0, 10); //TODO sample length generation should
                                                             //be based only on the settings, the text, and the sample history
    let mut app = App::new(terminal.size()?.width, text);

    let events = EventHandler::new(250);

    terminal.hide_cursor()?;
    terminal.clear()?;
    terminal.draw(|frame| app.render(frame))?;

    // Start the main loop.
    while app.running {
        // Handle events.
        match events.next()? {
            Event::Key(key_event) => {
                app.handle_key_events(key_event)?;
            }
            Event::Resize(width, _) => {
                app.terminal_width = width;
                app.rewrap();
            }
        }
        terminal.draw(|frame| app.render(frame))?;
    }

    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
