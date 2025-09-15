use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode as CK, KeyEvent, KeyModifiers as CM,
};
use scrivenwright::app::AppResult;
use scrivenwright::handler::{KeyCode as K, KeyDown, KeyModifiers as M};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Key(KeyDown),
    Resize(u16, u16),
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
        CM::CONTROL => M::Ctrl,
        _ => M::Unimplemented,
    };
    KeyDown { code, mods }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender: mpsc::Sender<Event> = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("no events available") {
                        match event::read().expect("unable to read event") {
                            CrosstermEvent::Key(e) => sender.send(Event::Key(to_key_down(e))),
                            CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                            _ => Ok(()),
                        }
                        .expect("failed to send terminal event")
                    }

                    if last_tick.elapsed() >= tick_rate {
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub fn next(&self) -> AppResult<Event> {
        Ok(self.receiver.recv()?)
    }
}
