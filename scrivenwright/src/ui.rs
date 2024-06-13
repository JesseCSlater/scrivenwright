use crate::app::App;
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::{prelude::*, widgets::*};
use textwrap::WordSeparator::AsciiSpace;
use textwrap::{
    core::{break_words, Fragment, Word},
    wrap_algorithms::{wrap_optimal_fit, Penalties},
};

#[derive(Debug, Clone)]
struct WhitespaceWord<'a> {
    word: &'a str,
    whitespace: &'a str,
}

impl<'a> WhitespaceWord<'a> {
    pub fn new(word: Word<'a>) -> Self {
        WhitespaceWord {
            word: word.word,
            whitespace: word.whitespace,
        }
    }
    pub fn len(&self) -> usize {
        self.word.len() + self.word.len()
    }
}

impl Fragment for WhitespaceWord<'_> {
    fn width(&self) -> f64 {
        self.len() as f64
    }
    fn whitespace_width(&self) -> f64 {
        0.0
    }
    fn penalty_width(&self) -> f64 {
        0.0
    }
}

//TODO fix panic on end of short input
impl App {
    pub fn wrap<'a>(text: &'a str, line_width: usize) -> (Vec<Vec<&str>>, Vec<(usize, usize)>) {
        let long_words = AsciiSpace.find_words(text);
        let broken_words = break_words(long_words, line_width);
        let words: Vec<WhitespaceWord<'a>> = broken_words
            .into_iter()
            .map(|w| WhitespaceWord::new(w))
            .collect();
        let wrapped_words = wrap_optimal_fit(&words, &[line_width as f64], &Penalties::default())
            .unwrap()
            .iter()
            .map(|s| {
                s.iter()
                    .flat_map(|s| [s.word, s.whitespace])
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut line_index = Vec::new();
        let mut l: usize = 0;
        let mut c: usize = 0;
        for line in &wrapped_words {
            for word in line {
                for _ in 0..word.len() {
                    line_index.push((l, c));
                    c += 1;
                }
            }
            c = 0;
            l += 1;
        }
        (wrapped_words, line_index)
    }

    pub fn render(&self, frame: &mut Frame) {
        let (mut lines, line_index) = Self::wrap(&self.text.text, self.terminal_width as usize);

        //TODO fail gracefully
        let &(start_line, start_offset) = line_index.get(self.text.sample_start_index).unwrap();
        let &(cur_line, cur_offset) = line_index
            .get(self.text.sample_start_index + self.text.cur_char)
            .unwrap();
        let &(end_line, end_offset) = self
            .line_index
            .get(self.text.sample_start_index + self.text.sample_len)
            .unwrap();
        let num_rows = frame.size().height as usize - 2; //TODO fix crash
        let rows_to_center = num_rows / 2 - 2; //TODO fix crash

        let first_row = usize::checked_sub(rows_to_center, cur_line).unwrap_or(0);

        let num_skipped_lines = usize::checked_sub(cur_line, rows_to_center).unwrap_or(0);
        lines = lines.split_off(usize::min(num_skipped_lines, lines.len()));
        lines.truncate(num_rows - first_row);

        let mut display_lines: Vec<Line> = Vec::new();
        for (mut i, ss) in lines.iter().enumerate() {
            let s = ss.iter().flat_map(|st| st.chars()).collect::<String>();
            i += num_skipped_lines;
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
                    display_lines.push(s.clone().dim().into());
                } else {
                    display_lines.push(s.clone().white().into());
                }
            } else {
                if i == end_line {
                    display_lines.push(Line::from(vec![
                        s.chars().take(end_offset).collect::<String>().blue(),
                        s.chars().skip(end_offset).collect::<String>().dim(),
                    ]));
                } else if i < end_line {
                    display_lines.push(s.clone().blue().into());
                } else {
                    display_lines.push(s.clone().dim().into());
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
