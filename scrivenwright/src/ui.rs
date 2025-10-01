use crate::app::{App, OpenText, PlatformAdapter};
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::{prelude::*, widgets::*};

impl<PA: PlatformAdapter> App<PA> {
    pub fn render(&self, text: &OpenText, frame: &mut Frame) {
        let line_width = self
            .settings
            .line_width((frame.size().width).saturating_sub(2));
        let cur_line = text.line_of_idx(text.focused_char, line_width);
        let num_rows = (frame.size().height as usize).saturating_sub(2);
        let rows_to_center = (num_rows / 2).saturating_sub(2);
        let first_line = cur_line.saturating_sub(rows_to_center);
        let first_row = rows_to_center.saturating_sub(cur_line);
        let num_lines = num_rows - first_row;

        let sidx = text.test.start_index;
        let cidx = sidx + text.test.cur_char;
        let eidx = sidx + text.test.length;
        let style_char = |idx: usize, c: char| -> Span {
            let s: String;
            if c == '\n' && idx >= sidx && idx < eidx {
                s = "↵".to_string();
            } else {
                s = c.to_string();
            }
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

        let display_lines: Vec<Line> = text
            .lines(line_width, first_line, num_lines)
            .iter()
            .map(|(sidx, l)| {
                let styled = l
                    .char_indices()
                    .map(|(idx, c)| style_char(sidx + idx, c))
                    .collect::<Vec<_>>();
                Line::from(styled)
            })
            .collect();

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
                    block::Title::from(format!("{}", text.get_rolling_average()))
                        .alignment(Alignment::Right),
                )
                .borders(Borders::ALL)
                .border_style(Style::new().white()),
            screen,
        );
    }
}
