use std::sync::Mutex;

use ratatui::{prelude::*, widgets::*};
use yew::Context;

use crate::{
    app::{CursorMap, Motion, TermApp},
    console_log,
    palette::GruvboxColor,
    HOST_ADDRESS,
};

static SCROLL_STATE: Mutex<Option<ScrollbarState>> = Mutex::new(None);

#[derive(Debug, PartialEq)]
pub struct Blog {
    summaries: Vec<(String, String, bool)>,
    scroll: u16,
}

#[derive(Debug)]
pub enum BlogMessage {
    PostSummaries(Vec<(String, String)>),
}

impl Blog {
    pub fn create(ctx: &Context<TermApp>, map: &mut CursorMap) -> Self {
        ctx.link().send_future(async move {
            let summaries = match reqwest::get(format!("http{HOST_ADDRESS}/api/v1/posts")).await {
                Ok(resp) => resp.json().await.unwrap_or_default(),
                Err(e) => Vec::new(),
            };
            BlogMessage::PostSummaries(summaries)
        });
        Self {
            scroll: 0,
            summaries: Vec::new(),
        }
    }

    pub fn handle_scroll(&mut self, dir: bool) {
        if dir {
            self.scroll = self.scroll.saturating_add(1);
        } else {
            self.scroll = self.scroll.saturating_sub(1);
        }
    }

    pub fn update(&mut self, msg: BlogMessage, map: &mut CursorMap) {
        map.clear_after(1);
        match msg {
            BlogMessage::PostSummaries(summaries) => {
                self.summaries = summaries.into_iter().map(|(t, s)| (t, s, false)).collect()
            }
        }
        for (title, _, _) in self.summaries.iter() {
            map.append_and_push(title.clone());
        }
    }

    pub fn handle_motion(&mut self, motion: Motion, map: &CursorMap) {
        match map.get_position() {
            (0, y) if y > 0 && y <= self.summaries.len() => {
                self.summaries
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, (_, _, sel))| *sel = i + 1 == y);
            }
            _ => {
                self.summaries
                    .iter_mut()
                    .for_each(|(_, _, sel)| *sel = false);
            }
        }
    }

    pub fn draw(&self, mut rect: Rect, frame: &mut Frame) -> Rect {
        let widget = Paragraph::new(
            self.summaries
                .iter()
                .map(|(t, _, sel)| get_line(t, *sel))
                .collect::<Vec<_>>(),
        )
        .alignment(Alignment::Center)
        .block(Block::new().borders(Borders::all()));
        frame.render_widget(widget, rect);
        let mut state = SCROLL_STATE.lock().unwrap();
        state.insert(ScrollbarState::new(100));
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            rect,
            state.as_mut().unwrap(),
        );
        rect.y += rect.height;
        rect
    }
}

fn get_line(s: &str, selected: bool) -> Line {
    let style = if selected {
        GruvboxColor::green().full_style(GruvboxColor::dark_3())
    } else {
        GruvboxColor::default_style()
    };
    Line::styled(s, style)
}