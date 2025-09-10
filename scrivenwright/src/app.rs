use chrono::{serde::ts_microseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::error;
use textwrap::{Options};

pub const DEFAULT_TEXT_WIDTH_PERCENT: u16 = 60;
pub const FULL_TEXT_WIDTH_PERCENT: u16 = 96;
const STARTING_SAMPLE_SIZE: usize = 100;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App<'a> {
    pub running: bool,
    pub terminal_width: u16,
    start_time: DateTime<Utc>,

    pub settings: Settings,
    pub text: Text<'a>,
    pub ui_state: UIState<'a>,
}

pub struct UIState<'a> {
    pub lines: Vec<(usize, &'a str)>,
    pub cursor_pos: usize,
}

impl<'a> UIState<'a> {
    pub fn new(text: &'a str, line_width: u16, cursor_pos: usize) -> Self {
        Self {
            lines: Self::wrap(text, line_width),
            cursor_pos,
        }
    }

    pub fn wrap(text: &'a str, line_width: u16) -> Vec<(usize, &'a str)> {
        let options: Options = Options::new(line_width as usize)
            .break_words(false)
            .word_splitter(textwrap::WordSplitter::NoHyphenation)
            .preserve_trailing_space(true);
        let wrapped_lines: Vec<Cow<'a, str>> = textwrap::wrap(text, &options);

        let mut lines = Vec::new();
        let mut prefix_sum: usize = 0;
        for cow in wrapped_lines {
            let line = match cow {
                Cow::Borrowed(s) => s,
                Cow::Owned(_) => panic! {"Jesse has misunderstood the textwrap library."},
            };
            lines.push((prefix_sum, line));
            prefix_sum += line.len();
        }
        lines
    }

    pub fn line_offset_of_idx(&self, idx: usize) -> (usize, usize) {
        let res = self
            .lines
            .binary_search_by_key(&idx, |(start_idx, _)| *start_idx);
        let line = match res {
            Ok(l) => l,
            Err(l) => l - 1,
        };
        (line, idx - self.lines[line].0)
    }
}

pub struct Settings {
    pub text_width_percent: u16,
    pub full_text_width: bool,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            text_width_percent: DEFAULT_TEXT_WIDTH_PERCENT,
            full_text_width: false,
        }
    }

    pub fn line_width(&self, terminal_width: u16) -> u16 {
        terminal_width * self.text_width_percent / 100
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Text<'a> {
    pub text: &'a str,
    pub cur_char: usize,
    pub sample_start_index: usize,
    pub sample_len: usize,
    sample_log: Vec<Test>,
    keypress_log: Vec<KeyPress>,
    save: Box<dyn Fn(Vec<Test>, Vec<KeyPress>) -> AppResult<()>>,
}

impl<'a> Text<'a> {
    pub fn new<F>(
        text: &'a str,
        sample_log: Vec<Test>,
        save: F,
        cur_char: usize,
        sample_start_index: usize,
        sample_len: usize,
    ) -> Self
    where
        F: Fn(Vec<Test>, Vec<KeyPress>) -> AppResult<()> + 'static,
    {
        Self {
            text,
            cur_char,
            sample_start_index,
            sample_len,
            sample_log,
            save: Box::new(save),
            keypress_log: Default::default(),
        }
    }
}

impl<'a> App<'a> {
    pub fn new(terminal_width: u16, text: Text<'a>) -> Self {
        let settings = Settings::default();

        let mut ret = Self {
            ui_state: UIState::new(
                &text.text,
                settings.line_width(terminal_width),
                text.cur_char,
            ),
            text,
            settings,
            terminal_width,
            running: true,
            start_time: Default::default(),
        };

        let _ = ret.get_next_sample();

        ret
    }

    pub fn line_width(&self) -> u16 {
        self.settings.line_width(self.terminal_width)
    }

    pub fn rewrap(&mut self) {
        self.ui_state.lines = UIState::wrap(self.text.text, self.line_width())
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
                .nth(self.text.sample_start_index + self.text.cur_char)
                .unwrap();

        if correct {
            self.text.cur_char += 1
        }
        if !correct || self.text.cur_char == self.text.sample_len {
            self.text.sample_log.push(Test {
                succeeded: correct,
                start_index: self.text.sample_start_index,
                end_index: self.text.sample_start_index + self.text.cur_char,
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

        Ok(())
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

        self.text.sample_start_index = usize::min(start_index, self.text.text.len() - 1);
        self.text.sample_len = usize::min(len, self.text.text.len() - start_index - 1);
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
