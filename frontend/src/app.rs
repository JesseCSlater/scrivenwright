use std::cmp::Ordering;

use yew::{Component, Context, Properties};
use yew_router::scope_ext::RouterScopeExt;

use crate::{
    home::{Home, HomeMessage},
    palette::{GruvboxColor, GruvboxExt},
    terminal::{DehydratedSpan, NeedsHydration},
    utils::{padded_title, ScrollRef},
    Route, TERMINAL,
};
use derive_more::From;
use js_sys::Function;
use ratatui::{prelude::*, widgets::*};
use wasm_bindgen::{prelude::Closure, JsValue};
use yew::prelude::*;

/// This module contains all of the machinery to run the UI app. The UI app is a single page
/// application consisting of the header, body, and footer. The body is changed when switching
/// between tabs/"pages". The app holds all of the logic for interacting with the browser window,
/// including switching between tabs and tracking the cursor.

pub struct TermApp {
    /// The body of the UI
    body: AppBody,
}

/// The body used for the app on construction.
#[derive(Debug, Properties, PartialEq, Clone)]
pub struct TermAppProps {
    pub body: AppBodyProps,
}

pub struct AppBody {
    inner: AppBodyInner,
    // TODO: Rename to scroll
    scroll: ScrollRef,
}

/// The different main sections the user might find themselves in.
#[derive(Debug, PartialEq, From)]
enum AppBodyInner {
    Home(Home),
}

impl AppBody {
    fn new<I: Into<AppBodyInner>>(inner: I) -> Self {
        Self {
            inner: inner.into(),
            scroll: ScrollRef::new(0, 0),
        }
    }

    fn draw(&self, chunk: Rect, frame: &mut Frame<'_>) {
        self.inner.draw(&self.scroll, chunk, frame);
        self.scroll.render_scroll(
            frame,
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            chunk,
        );
    }

    fn hydrate(&self, ctx: &Context<TermApp>, span: &mut DehydratedSpan) {
        self.inner.hydrate(ctx, span)
    }

    fn update(&mut self, ctx: &Context<TermApp>, msg: ComponentMsg) {
        self.inner.update(ctx, msg)
    }

    fn handle_scroll(&mut self, dir: bool) {
        self.inner.handle_scroll(dir);
        if dir {
            self.scroll.next()
        } else {
            self.scroll.prev()
        }
    }
}

impl AppBodyInner {
    fn draw(&self, scroll: &ScrollRef, chunk: Rect, frame: &mut Frame<'_>) {
        match self {
            Self::Home(home) => home.draw(scroll, chunk, frame),
        }
    }

    fn hydrate(&self, ctx: &Context<TermApp>, span: &mut DehydratedSpan) {
        match self {
            Self::Home(home) => home.hydrate(ctx, span),
        }
    }

    fn update(&mut self, _ctx: &Context<TermApp>, msg: ComponentMsg) {
        match (self, msg) {
            (Self::Home(body), ComponentMsg::Home(msg)) => body.update(msg),
        }
    }

    fn handle_scroll(&mut self, dir: bool) {
        match self {
            Self::Home(home) => home.handle_scroll(dir),
        }
    }
}

/// The different main sections the user might find themselves in.
#[derive(Debug, PartialEq, Clone)]
pub enum AppBodyProps {
    Home,
}

impl AppBodyProps {
    fn create_body(self, ctx: &Context<TermApp>) -> AppBody {
        let inner = match self {
            AppBodyProps::Home => AppBodyInner::Home(Home::create(ctx)),
        };
        AppBody::new(inner)
    }
}

#[derive(Debug, From)]
pub enum TermAppMsg {
    Resized,
    Clicked(AppBodyProps),
    // TODO: Replace bool with "up" or "down"
    // true = up, down = false
    Scrolled(bool),
    ComponentMsg(ComponentMsg),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Motion {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, From)]
pub enum ComponentMsg {
    Home(HomeMessage),
}

impl TermApp {
    fn draw(&self, area: Rect, frame: &mut Frame<'_>) {
        let chunks = Layout::new(
            Direction::Vertical,
            [
                Constraint::Min(3),
                Constraint::Percentage(100),
                Constraint::Min(3),
            ],
        )
        .split(area);
        self.draw_header(chunks[0], frame);
        self.body.draw(chunks[1], frame);
        self.draw_footer(chunks[2], frame);
    }

    fn draw_header(&self, rect: Rect, frame: &mut Frame<'_>) {
        let titles: Vec<Line<'_>> = vec![
            Line::styled("Home", GruvboxColor::teal().fg_style().to_hydrate()),
            Line::styled("Projects", GruvboxColor::teal().fg_style().to_hydrate()),
            Line::styled("Blog", GruvboxColor::teal().fg_style().to_hydrate()),
        ];
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(padded_title(
                        "The Avid Rustacean".to_owned(),
                        GruvboxColor::burnt_orange().fg_style(),
                    ))
                    .title_alignment(Alignment::Center),
            )
            .style(GruvboxColor::orange().full_style(GruvboxColor::dark_2()));
        frame.render_widget(tabs, rect);
    }

    fn draw_footer(&self, rect: Rect, frame: &mut Frame<'_>) {
        let line = Line::from(vec![
            Span::styled("Email", GruvboxColor::blue().fg_style().to_hydrate()),
            Span::from(" | "),
            Span::styled("GitHub", GruvboxColor::blue().fg_style().to_hydrate()),
            Span::from(" | "),
            Span::styled("LinkdIn", GruvboxColor::blue().fg_style().to_hydrate()),
            Span::from(" "),
        ])
        .alignment(Alignment::Right);
        let tabs = Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL))
            .style(GruvboxColor::orange().fg_style());
        frame.render_widget(tabs, rect);
    }
}

impl Component for TermApp {
    type Message = TermAppMsg;
    type Properties = TermAppProps;

    fn create(ctx: &Context<Self>) -> Self {
        let window = web_sys::window().unwrap();
        // Bind a function to the "on-resize" window event
        let cb = ctx.link().callback(|()| TermAppMsg::Resized);
        let func = move || cb.emit(());
        let func: Function = Closure::<dyn 'static + Fn()>::new(func)
            .into_js_value()
            .into();
        window.set_onresize(Some(&func));
        // Bind a function to the "on-wheel" window event
        let cb = ctx.link().callback(|msg: TermAppMsg| msg);
        let func = move |event: JsValue| {
            let event: WheelEvent = event.into();
            match event.delta_y().partial_cmp(&0.0) {
                Some(Ordering::Less) => cb.emit(TermAppMsg::Scrolled(false)),
                Some(Ordering::Greater) => cb.emit(TermAppMsg::Scrolled(true)),
                _ => {}
            }
        };
        let func: Function = Closure::<dyn 'static + Fn(JsValue)>::new(func)
            .into_js_value()
            .into();
        window.set_onwheel(Some(&func));
        // Create the viewer
        let body = ctx.props().body.clone().create_body(ctx);
        Self { body }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TermAppMsg::Resized => TERMINAL.term().backend_mut().resize_buffer(),
            TermAppMsg::ComponentMsg(msg) => self.body.update(ctx, msg),
            TermAppMsg::Scrolled(b) => self.body.handle_scroll(b),
            TermAppMsg::Clicked(page) => {
                match &page {
                    AppBodyProps::Home => ctx.link().navigator().unwrap().push(&Route::Home),
                }
                self.body = page.create_body(ctx);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut term = TERMINAL.term();
        let area = term.size().unwrap();
        term.draw(|frame| self.draw(area, frame)).unwrap();
        term.backend_mut().hydrate(|span| match span.text().trim() {
            "Home" => span.on_click(ctx.link().callback(|_| AppBodyProps::Home)),
            "Email" => span.hyperlink("mailto:tylerbloom2222@gmail.com".to_owned()),
            "GitHub" => span.hyperlink("https://github.com/TylerBloom".to_owned()),
            "LinkdIn" => {
                span.hyperlink("https://www.linkedin.com/in/tyler-bloom-aba0a4156/".to_owned())
            }
            _ => self.body.hydrate(ctx, span),
        })
    }
}