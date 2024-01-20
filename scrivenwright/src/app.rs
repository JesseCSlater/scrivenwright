use chrono::{serde::ts_microseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error;

pub const DEFAULT_TEXT_WIDTH_PERCENT: u16 = 60;
pub const FULL_TEXT_WIDTH_PERCENT: u16 = 95;
const STARTING_SAMPLE_SIZE: usize = 100;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App {
    pub running: bool,
    pub book_lines: Vec<String>,
    pub line_index: Vec<(usize, usize)>,
    pub sample_start_index: usize,
    pub sample_len: usize,
    start_time: DateTime<Utc>,
    pub display_line: usize,
    pub text_width_percent: u16,
    pub terminal_width: u16,
    pub full_text_width: bool,
    pub text: Text,
}

pub struct Text {
    pub text: String,
    pub cur_char: usize,
    sample_log: Vec<Test>,
    keypress_log: Vec<KeyPress>,
    save: Box<dyn Fn(Vec<Test>, Vec<KeyPress>) -> AppResult<()>>,
}

impl Text {
    pub fn new<F>(text: String, sample_log: Vec<Test>, save: F, cur_char: usize) -> Self
    where
        F: Fn(Vec<Test>, Vec<KeyPress>) -> AppResult<()> + 'static,
    {
        Self {
            text,
            cur_char,
            sample_log,
            save: Box::new(save),
            keypress_log: Default::default(),
        }
    }
}

impl App {
    pub fn new(terminal_width: u16, text: Text) -> Self {
        let mut ret = Self {
            text,
            terminal_width,
            running: true,
            text_width_percent: DEFAULT_TEXT_WIDTH_PERCENT,
            full_text_width: false,
            start_time: Default::default(),
            sample_len: Default::default(),
            sample_start_index: Default::default(),
            book_lines: Default::default(),
            line_index: Default::default(),
            display_line: Default::default(),
        };

        let _ = ret.get_next_sample();
        ret.generate_lines();

        ret
    }

    pub fn quit(&mut self) -> AppResult<()> {
        self.running = false;
        (self.text.save)(self.text.sample_log.clone(), self.text.keypress_log.clone())
    }

    pub fn handle_char(&mut self, c: char) -> AppResult<()> {
        let correct = c
            == self
                .text
                .text
                .chars()
                .nth(self.sample_start_index + self.text.cur_char)
                .unwrap();

        if correct {
            self.text.cur_char += 1
        }
        if !correct || self.text.cur_char == self.sample_len {
            self.text.sample_log.push(Test {
                succeeded: correct,
                start_index: self.sample_start_index,
                end_index: self.sample_start_index + self.text.cur_char,
                started: self.start_time,
                completed: Utc::now(),
            });
            self.start_time = Utc::now();
            self.get_next_sample()?;

            self.text.cur_char = 0;
        }

        let log_entry = &KeyPress {
            correct,
            key: c,
            time: Utc::now(),
        };
        self.text.keypress_log.push(log_entry.clone());

        let &(cur_line, _) = self
            .line_index
            .get(self.sample_start_index + self.text.cur_char)
            .unwrap();
        self.display_line = cur_line;
        Ok(())
    }

    pub fn generate_lines(&mut self) {
        let max_line_len =
            (self.terminal_width as f64 * (self.text_width_percent as f64 / 100.0)) as usize;
        let mut lines = Vec::new();
        let mut line_index: Vec<(usize, usize)> = Vec::new();
        let mut line = "".to_owned();
        let mut word = "".to_owned();
        let mut row_i = 0;
        let mut column_i = 0;

        for c in self.text.text.chars() {
            word.push(c);
            if c == ' ' {
                if line.len() + word.len() < max_line_len {
                    line.push_str(&word);
                } else {
                    lines.push(line);
                    line = word.to_owned();
                    row_i += 1;
                    column_i = 0;
                }
                for _ in 0..word.len() {
                    line_index.push((row_i, column_i));
                    column_i += 1;
                }
                word = "".to_owned();
            }
        }
        if line.len() + word.len() < max_line_len {
            line.push_str(&word);
            lines.push(line);
        } else {
            lines.push(line);
            lines.push(word.clone());
            row_i += 1;
        }
        for _ in 0..word.len() {
            line_index.push((row_i, column_i));
            column_i += 1;
        }

        self.book_lines = lines;
        self.display_line = line_index.get(self.sample_start_index).unwrap().0; //TODO allow for resize while scrolled
        self.line_index = line_index;
    }

    fn get_next_sample(&mut self) -> AppResult<()> {
        let tests = &self.text.sample_log;

        let mut start_index = 0;
        for t in tests {
            if t.succeeded && t.end_index > start_index {
                start_index = t.end_index;
            }
        }

        let avg_50 = tests
            .iter()
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .rev()
            .take(50)
            .sum::<usize>()
            / 50;
        let max_10 = tests
            .iter()
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .rev()
            .take(10)
            .max()
            .unwrap_or(STARTING_SAMPLE_SIZE);
        let best = usize::max(avg_50, max_10) + 5;

        let wrong_num = tests
            .iter()
            .rev()
            .take_while(|t| !t.succeeded)
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .count();

        let full = self
            .text
            .text
            .chars()
            .skip(start_index)
            .take(best)
            .collect::<String>();

        let len = full
            .split_whitespace()
            .rev()
            .skip(usize::max(wrong_num, 1))
            .collect::<Vec<_>>()
            .join(" ")
            .len()
            + 1;

        self.sample_start_index = usize::min(start_index, self.text.text.len() - 1);
        self.sample_len = usize::min(len, self.text.text.len() - start_index - 1);
        self.start_time = Utc::now();
        Ok(())
    }

    pub fn get_rolling_average(&self) -> AppResult<usize> {
        Ok(self
            .text
            .sample_log
            .iter()
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .rev()
            .take(10)
            .sum::<usize>()
            / 10)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KeyPress {
    correct: bool,
    key: char,
    #[serde(with = "ts_microseconds")]
    time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Test {
    succeeded: bool,
    start_index: usize,
    end_index: usize,
    #[serde(with = "ts_microseconds")]
    started: DateTime<Utc>,
    #[serde(with = "ts_microseconds")]
    completed: DateTime<Utc>,
}
