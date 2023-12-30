use std::collections::HashMap;

use avid_rustacean_model::{GruvboxColor, HomePage};
use gloo_net::http::Request;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};
use yew::Context;

use crate::{
    app::{ComponentMsg, TermApp},
    palette::GruvboxExt,
    terminal::DehydratedSpan,
    utils::{padded_title, ScrollRef},
};

#[derive(Debug, PartialEq)]
pub struct Home {
    data: Paragraph<'static>,
    links: HashMap<String, String>,
    scroll: u16,
}

#[derive(Debug, PartialEq)]
pub enum HomeMessage {
    Data(HomePage),
}

impl Home {
    pub fn create(ctx: &Context<TermApp>) -> Self {
        ctx.link().send_future(async move {
            let home = match Request::get("/api/v1/home").send().await {
                Ok(resp) => resp.json().await.unwrap_or_default(),
                Err(_) => HomePage::default(),
            };
            ComponentMsg::Home(HomeMessage::Data(home))
        });
        Self {
            data: Paragraph::default(),
            links: HashMap::new(),
            scroll: 0,
        }
    }

    pub fn handle_scroll(&mut self, dir: bool) {
        if dir {
            self.scroll = self.scroll.saturating_add(1);
        } else {
            self.scroll = self.scroll.saturating_sub(1);
        }
    }

    pub fn hydrate(&self, _ctx: &Context<TermApp>, _span: &mut DehydratedSpan) {}

    pub fn update(&mut self, msg: HomeMessage) {
        match msg {
            HomeMessage::Data(data) => {
                self.data = Paragraph::new(data.body)
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::new()
                            .title(padded_title(
                                "Home".into(),
                                GruvboxColor::green().full_style(GruvboxColor::dark_4()),
                            ))
                            .borders(Borders::ALL)
                            .padding(Padding::horizontal(10)),
                    )
                    .alignment(Alignment::Center);
            }
        }
    }

    pub fn draw(&self, scroll: &ScrollRef, rect: Rect, frame: &mut Frame<'_>) {
        scroll.set_content_length(self.data.line_count(rect.width.saturating_sub(2)));
        frame.render_widget(
            self.data.clone().scroll((scroll.view_start() as u16, 0)),
            rect,
        );
    }
}
