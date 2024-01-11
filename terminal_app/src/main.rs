use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyCode as CK, KeyEvent, KeyModifiers as CM,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use scrivenwright::app::{App, AppResult, KeyPress, Test};
use scrivenwright::handler::{handle_key_events, KeyCode as K, KeyDown, KeyModifiers as M};
use scrivenwright::tui::Tui;
use std::panic;
use std::{env, fs, io};

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

    crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal::enable_raw_mode()?;

    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture);
        panic_hook(panic);
    }));

    let _ = fs::create_dir(
        dirs::home_dir()
            .unwrap()
            .join(".booktyping")
            .join(&book_title),
    );

    let book_text = file_sys::load_book(&book_title).expect("Failed to load book");

    let tests = file_sys::load_tests(&book_title).expect("Failed to load tests");

    let save = move |tests: Vec<Test>, keypresses: Vec<KeyPress>| {
        file_sys::save_tests(&book_title, &tests)?;
        file_sys::save_keypresses(&book_title, &keypresses)?;
        Ok(())
    };

    let mut app = App::new(terminal.size()?.width, book_text, tests, save)?;

    let events = EventHandler::new(250);

    let mut tui = Tui::new(terminal);

    tui.init()?;
    tui.draw(&mut app)?; //Draw first frame

    // Start the main loop.
    while app.running {
        // Handle events.
        match events.next()? {
            Event::Key(key_event) => {
                handle_key_events(to_key_down(key_event), &mut app)?;
                tui.draw(&mut app)?;
            }
            Event::Resize(width, _) => {
                app.terminal_width = width;
                app.generate_lines();
                tui.draw(&mut app)?;
            }
        }
    }

    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
    tui.exit()?;
    Ok(())
}

fn to_key_down(event: KeyEvent) -> KeyDown {
    let code = match event.code {
        CK::Char(c) => K::Char(c),
        CK::Esc => K::Esc,
        CK::Up => K::Up,
        CK::Down => K::Down,
        CK::Right => K::Right,
        CK::Left => K::Left,
        _ => K::Unimplemented,
    };
    let mods = match event.modifiers {
        CM::SHIFT => M::Shift,
        CM::CONTROL => M::Ctrl,
        CM::ALT => M::Alt,
        _ => M::Unimplemented,
    };
    KeyDown { code, mods }
}
