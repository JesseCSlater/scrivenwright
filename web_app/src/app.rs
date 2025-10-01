use crate::TERMINAL;
use js_sys::Function;
//use ratatui::layout::Rect;
use ratatui::Frame;
use scrivenwright::app::{App, KeyPress, OpenText, TestResult};
use scrivenwright::handler::{KeyCode as K, KeyDown, KeyModifiers as M};
use std::panic;
use wasm_bindgen::prelude::Closure;
use yew::prelude::*;

pub struct TermApp {
    app: App<()>,
    text: OpenText,
}

#[derive(Debug)]
pub enum TermAppMsg {
    Resized,
    KeyDown(KeyDown),
}

fn to_key_down(event: KeyboardEvent) -> KeyDown {
    let code = match event.key().as_str() {
        "Escape" => K::Esc,
        "ArrowUp" => K::Up,
        "ArrowDown" => K::Down,
        "ArrowRight" => K::Right,
        "ArrowLeft" => K::Left,
        s => {
            if s.len() == 1 {
                K::Char(s.chars().next().unwrap())
            } else {
                K::Unimplemented
            }
        }
    };
    let mods = if event.ctrl_key() {
        M::Ctrl
    } else {
        M::Unimplemented
    };

    // Don't send keys which were part of a shortcut
    let ignored = [
        (K::Char(')'), M::Ctrl),
        (K::Char('0'), M::Ctrl),
        (K::Char('-'), M::Ctrl),
        (K::Char('_'), M::Ctrl),
        (K::Char('+'), M::Ctrl),
        (K::Char('='), M::Ctrl),
    ];
    if ignored.contains(&(code, mods)) {
        return KeyDown {
            code: K::Unimplemented,
            mods: M::Unimplemented,
        };
    }

    KeyDown { code, mods }
}

impl TermApp {
    fn draw(&self, frame: &mut Frame<'_>) {
        self.app.render(&self.text, frame)
    }
}

impl Component for TermApp {
    type Message = TermAppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let window = web_sys::window().unwrap();

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            _ = web_sys::window().unwrap().alert_with_message(
                format!("Panicked, please reload. Error: {}", panic.to_string()).as_str(),
            );
            panic_hook(panic);
        }));

        let cb = ctx.link().callback(|()| TermAppMsg::Resized);
        let func: Function = Closure::<dyn 'static + Fn()>::new(move || cb.emit(()))
            .into_js_value()
            .into();
        window.set_onresize(Some(&func));

        let cb: Callback<KeyboardEvent> = ctx
            .link()
            .callback(|e: KeyboardEvent| TermAppMsg::KeyDown(to_key_down(e)));
        let func: Function =
            Closure::<dyn 'static + Fn(KeyboardEvent)>::new(move |e: KeyboardEvent| cb.emit(e))
                .into_js_value()
                .into();
        window.set_onkeydown(Some(&func));

        let book_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras ultrices imperdiet augue et facilisis. Duis dignissim libero eros, eu sagittis purus suscipit at. Vivamus sit amet bibendum ex. Ut convallis velit id odio tincidunt fringilla. Mauris interdum eleifend sapien, vitae luctus sem. Sed suscipit ultrices metus, ut iaculis urna sagittis vel. Ut elementum nisi ac diam mattis, non condimentum urna pretium. Proin hendrerit metus sed pretium lacinia. Praesent a purus rhoncus odio imperdiet blandit quis quis risus. Aliquam euismod, eros at congue laoreet, sem mi pellentesque augue, et dictum magna augue eget lectus. Nam ultrices justo justo, quis gravida justo semper eget. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Nunc hendrerit massa sed fringilla laoreet. In id quam tincidunt sem laoreet aliquet molestie a lacus. Donec felis dui, tempus tincidunt laoreet ut, convallis quis mauris. ".into();

        let tests = Vec::new();

        let save = move |_tests: Vec<TestResult>, _keypresses: Vec<KeyPress>| ();

        let text = OpenText::new(book_text, tests, save);
        let app = App::new(());

        Self { app, text }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TermAppMsg::Resized => {
                TERMINAL.term().backend_mut().resize_buffer();
            }
            TermAppMsg::KeyDown(event) => {
                self.app.handle_key_events(
                    event,
                    &mut self.text,
                    TERMINAL.term().size().unwrap().width - 2,
                );
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut term = TERMINAL.term();
        term.draw(|frame: &mut Frame<'_>| self.draw(frame)).unwrap();
        term.backend_mut().render()
    }
}
