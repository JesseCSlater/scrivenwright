use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use scrivenwright::app::{App, AppResult};
use scrivenwright::text::{KeyPress, OpenText, TestResult};
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

    let test_log = file_sys::load_tests(&book_title).expect("Failed to load tests");

    let save = move |test: TestResult, keypresses: Vec<KeyPress>| {
        file_sys::save_test(&book_title, &test).unwrap();
        file_sys::save_keypresses(&book_title, &keypresses).unwrap();
    };

    let mut text = OpenText::new(book_text, test_log, save);
    let adapter = ();

    let mut app = App::new(adapter);
    let mut width = terminal.size()?.width;

    let events = EventHandler::new(250);

    terminal.hide_cursor()?;
    terminal.clear()?;
    terminal.draw(|frame| app.render(&mut text, frame))?;

    // Start the main loop.
    while app.running {
        // Handle events.
        match events.next() {
            Event::Key(key_event) => {
                app.handle_key_events(key_event, &mut text, width - 2);
            }
            Event::Resize(w, _) => {
                width = w;
            }
        }
        terminal.draw(|frame| app.render(&mut text, frame))?;
    }

    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
