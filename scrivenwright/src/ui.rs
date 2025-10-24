use crate::app::{App, PlatformAdapter};
use crate::text::OpenText;
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
        let cur_line = text
            .line_offset_of_idx(text.focused_glyph, line_width)
            .expect("Focused glyph outside")
            .0;
        let num_rows = (frame.size().height as usize).saturating_sub(2);
        let rows_to_center = (num_rows / 2).saturating_sub(2);
        let first_line = cur_line.saturating_sub(rows_to_center);
        let first_row = rows_to_center.saturating_sub(cur_line);
        let num_lines = num_rows - first_row;

        let sidx = text
            .test
            .map(|t| t.start_index)
            .unwrap_or(text.focused_glyph);
        let cidx = sidx + text.test.map(|t| t.cur_char).unwrap_or(0);
        let eidx = sidx + text.test.map(|t| t.length).unwrap_or(0);
        let style_char = |idx: usize, c: &str| -> Span {
            let mut s: Span<'_>;
            if c == "\n" && idx >= sidx && idx < eidx {
                s = Span::raw("↵");
            } else if c == "\n" {
                s = Span::raw(" ");
            } else {
                s = Span::raw(c.to_string());
            }

            s = if idx < sidx || idx >= eidx {
                s.dark_gray()
            } else if idx < cidx {
                s.white()
            } else if idx == cidx {
                s.black().bg(Color::Blue)
            } else {
                s.blue()
            };
            if idx == text.focused_glyph {
                s.black().bg(Color::White)
            } else {
                s
            }
        };

        let display_lines: Vec<Line> = text
            .lines(line_width, first_line, num_lines)
            .iter()
            .map(|&(sidx, eidx)| {
                let styled = text
                    .text
                    .glyphs()
                    .skip(sidx)
                    .take(eidx - sidx)
                    .enumerate()
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
