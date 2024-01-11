use crate::{console_log, TERMINAL, terminal::get_window_size};
use js_sys::Function;
use ratatui::prelude::*;
use wasm_bindgen::prelude::Closure;
use yew::prelude::*;
use scrivenwright::app::{App, Test, KeyPress};


pub struct TermApp {
    app : App
}

#[derive(Debug)]
pub enum TermAppMsg {
    Resized,
    KeyDown(KeyboardEvent),
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

        let cb = ctx.link().callback(|()| TermAppMsg::Resized);
        let func: Function = Closure::<dyn 'static + Fn()>::new(move || cb.emit(()))
            .into_js_value()
            .into();
        window.set_onresize(Some(&func));

        let cb: Callback<KeyboardEvent> = ctx
            .link()
            .callback(|e: KeyboardEvent| TermAppMsg::KeyDown(e));
        let func: Function =
            Closure::<dyn 'static + Fn(KeyboardEvent)>::new(move |e: KeyboardEvent| cb.emit(e))
                .into_js_value()
                .into();
        window.set_onkeydown(Some(&func));

        let book_text = "Text".into();

        let tests = Vec::new();
    
        let save = move |_tests: Vec<Test>, _keypresses: Vec<KeyPress>| {
            Ok(())
        };
    
        let app = App::new(get_window_size().1, book_text, tests, save).expect("Failed to initialize");

        Self {app}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TermAppMsg::Resized => TERMINAL.term().backend_mut().resize_buffer(),
            TermAppMsg::KeyDown(event) => {
                console_log(format!("{}, {}", event.key(), event.ctrl_key()))
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut term = TERMINAL.term();
        let area = term.size().unwrap();
        term.draw(|frame: &mut Frame<'_>| self.draw(area, frame)).unwrap();
        term.backend_mut().render()
    }
}
