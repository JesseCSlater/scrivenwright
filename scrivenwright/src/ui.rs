use crate::app::App;
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::{prelude::*, widgets::*};
use std::cmp;

impl<'a> App<'a> {
    pub fn render(&self, frame: &mut Frame) {
        let state = &self.ui_state;
        let cur_line =
            state.line_of_idx(state.cursor_line);
        
        let num_rows = (frame.size().height as usize).saturating_sub(2);
        let rows_to_center = (num_rows / 2).saturating_sub(2);
        let first_line = cur_line.saturating_sub(rows_to_center);
        let first_row = rows_to_center.saturating_sub(cur_line);

        let sidx = self.text.sample_start_index;
        let cidx = sidx + self.text.cur_char;
        let eidx = sidx + self.text.sample_len;
        let style = |idx: usize, c: char| -> Span {
            let s = c.to_string(); 
            if idx < sidx || idx >= eidx {
                s.dim()
            } else if idx < cidx {
                s.white()
            } else if idx == cidx {
                s.black().bg(Color::White)
            } else {
                s.blue()
            }
        };

        let mut display_lines: Vec<Line> = Vec::new();
        for i in first_line..cmp::min(first_line + num_rows, state.lines.len()){
            let t = state.lines[i].1
                .chars()
                .enumerate()
                .map(|(idx, c)| style(state.lines[i].0 + idx, c))
                .collect::<Vec<_>>();
            display_lines.push(Line::from(t));
        }

        let graph = Paragraph::new::<Text>(display_lines.into()).style(Style::default());

        let screen = Rect::new(0, 0, frame.size().width, frame.size().height);

        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(first_row as u16 + 1),
                Constraint::Percentage(100),
            ])
            .split(screen);
        let horiz = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - self.settings.text_width_percent) / 2),
                Constraint::Percentage(self.settings.text_width_percent),
                Constraint::Percentage((100 - self.settings.text_width_percent) / 2),
            ])
            .split(vert[1])[1];

        // Render into the second chunk of the layout.
        frame.render_widget(graph, horiz);
        frame.render_widget(
            Block::default()
                .title("Scrivenwright")
                .title(
                    block::Title::from(format!("{}", self.get_rolling_average()))
                        .alignment(Alignment::Right),
                )
                .borders(Borders::ALL)
                .border_style(Style::new().white()),
            screen,
        );
    }
}
