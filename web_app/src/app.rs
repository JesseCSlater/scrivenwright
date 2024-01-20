use crate::{terminal::get_window_size, TERMINAL};
use js_sys::Function;
use ratatui::layout::Rect;
use ratatui::Frame;
use scrivenwright::app::{App, KeyPress, Test, Text};
use scrivenwright::handler::{KeyCode as K, KeyDown, KeyModifiers as M};
use std::panic;
use wasm_bindgen::prelude::Closure;
use yew::prelude::*;

pub struct TermApp {
    app: App,
}

#[derive(Debug)]
pub enum TermAppMsg {
    Resized,
    KeyDown(KeyDown),
}

fn to_key_down(event: KeyboardEvent) -> KeyDown {
    let code = match event.key().as_str() {
        "Escape" => K::Esc,
        "Up" => K::Up,
        "Down" => K::Down,
        "Right" => K::Right,
        "Left" => K::Left,
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

    KeyDown { code, mods }
}

impl TermApp {
    fn draw(&self, _area: Rect, frame: &mut Frame<'_>) {
        self.app.render(frame)
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

        let save = move |_tests: Vec<Test>, _keypresses: Vec<KeyPress>| Ok(());

        let text = Text::new(book_text, tests, save, 0);
        let app = App::new(get_window_size().1, text);

        Self { app }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TermAppMsg::Resized => TERMINAL.term().backend_mut().resize_buffer(),
            TermAppMsg::KeyDown(event) => {
                self.app
                    .handle_key_events(event)
                    .expect("Failed to handle char");
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut term = TERMINAL.term();
        let area = term.size().unwrap();
        term.draw(|frame: &mut Frame<'_>| self.draw(area, frame))
            .unwrap();
        term.backend_mut().render()
    }
}
