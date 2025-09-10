use crate::app::App;
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::{prelude::*, widgets::*};
use std::cmp;

//TODO fix panic on end of short input
impl<'a> App<'a> {
    pub fn render(&self, frame: &mut Frame) {
        let state = &self.ui_state;
        let (start_line, start_offset) = state.line_offset_of_idx(self.text.sample_start_index);
        let (cur_line, cur_offset) =
            state.line_offset_of_idx(self.text.sample_start_index + self.text.cur_char);
        let (end_line, end_offset) =
            state.line_offset_of_idx(self.text.sample_start_index + self.text.sample_len);

        let num_rows = frame.size().height as usize - 2; //TODO fix crash
        let rows_to_center = num_rows / 2 - 2; //TODO fix crash

        let first_line = usize::checked_sub(cur_line, rows_to_center).unwrap_or(0);
        let first_row = usize::checked_sub(rows_to_center, cur_line).unwrap_or(0);
        

        let mut display_lines: Vec<Line> = Vec::new();
        for i in first_line..cmp::min(first_line + num_rows, state.lines.len()){
            let s = state.lines[i].1;
            if i == cur_line {
                if i == start_line && i == end_line {
                    display_lines.push(Line::from(vec![
                        s.chars().take(start_offset).collect::<String>().dim(),
                        s.chars()
                            .take(cur_offset)
                            .skip(start_offset)
                            .collect::<String>()
                            .white(),
                        s.chars()
                            .nth(cur_offset)
                            .unwrap()
                            .to_string()
                            .black()
                            .bg(Color::White),
                        s.chars()
                            .take(end_offset)
                            .skip(cur_offset + 1)
                            .collect::<String>()
                            .blue(),
                        s.chars().skip(end_offset).collect::<String>().dim(),
                    ]));
                } else if i == start_line {
                    display_lines.push(Line::from(vec![
                        s.chars().take(start_offset).collect::<String>().dim(),
                        s.chars()
                            .take(cur_offset)
                            .skip(start_offset)
                            .collect::<String>()
                            .white(),
                        s.chars()
                            .nth(cur_offset)
                            .unwrap()
                            .to_string()
                            .black()
                            .bg(Color::White),
                        s.chars().skip(cur_offset + 1).collect::<String>().blue(),
                    ]));
                } else if i == end_line {
                    display_lines.push(Line::from(vec![
                        s.chars().take(cur_offset).collect::<String>().white(),
                        s.chars()
                            .nth(cur_offset)
                            .unwrap()
                            .to_string()
                            .black()
                            .bg(Color::White),
                        s.chars()
                            .take(end_offset)
                            .skip(cur_offset + 1)
                            .collect::<String>()
                            .blue(),
                        s.chars().skip(end_offset).collect::<String>().dim(),
                    ]));
                } else {
                    display_lines.push(Line::from(vec![
                        s.chars().take(cur_offset).collect::<String>().white(),
                        s.chars()
                            .nth(cur_offset)
                            .unwrap()
                            .to_string()
                            .black()
                            .bg(Color::White),
                        s.chars().skip(cur_offset + 1).collect::<String>().blue(),
                    ]));
                }
            } else if i < cur_line {
                if i == start_line {
                    display_lines.push(Line::from(vec![
                        s.chars().take(start_offset).collect::<String>().dim(),
                        s.chars().skip(start_offset).collect::<String>().white(),
                    ]));
                } else if i < start_line {
                    display_lines.push(s.dim().into());
                } else {
                    display_lines.push(s.white().into());
                }
            } else {
                if i == end_line {
                    display_lines.push(Line::from(vec![
                        s.chars().take(end_offset).collect::<String>().blue(),
                        s.chars().skip(end_offset).collect::<String>().dim(),
                    ]));
                } else if i < end_line {
                    display_lines.push(s.blue().into());
                } else {
                    display_lines.push(s.dim().into());
                }
            }
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
                    block::Title::from(format!("{}", self.get_rolling_average().unwrap()))
                        .alignment(Alignment::Right),
                )
                .borders(Borders::ALL)
                .border_style(Style::new().white()),
            screen,
        );
    }
}
