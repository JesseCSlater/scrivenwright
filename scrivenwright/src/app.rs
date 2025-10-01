use chrono::{serde::ts_microseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::min;
use std::error;
use textwrap::Options;

pub const DEFAULT_TEXT_WIDTH_PERCENT: u16 = 60;
pub const FULL_TEXT_WIDTH_PERCENT: u16 = 96;
const STARTING_SAMPLE_SIZE: usize = 100;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App<PA: PlatformAdapter> {
    pub running: bool,
    pub adapter: PA,
    pub settings: Settings,
}

pub struct OpenText {
    text: String,
    pub test: Test,
    pub focused_char: usize,
    wrapper: RefCell<TextWrapper>,
    test_log: Vec<TestResult>,
    keypress_log: Vec<KeyPress>,
    save: Box<dyn Fn(Vec<TestResult>, Vec<KeyPress>)>,
}

pub struct Test {
    pub start_index: usize,
    pub length: usize,
    pub cur_char: usize,
    start_time: DateTime<Utc>,
}

pub struct TextWrapper {
    cur_width: u16,
    // Beginning of line, length of line
    _lines: Vec<(usize, usize)>,
}

pub struct Settings {
    pub text_width_percent: u16,
    pub full_text_width: bool,
}

impl TextWrapper {
    fn new() -> Self {
        Self {
            cur_width: 0,
            _lines: Vec::new(),
        }
    }

    fn wrap(&mut self, text: &str, line_width: u16) {
        if line_width == self.cur_width {
            return;
        };
        self.cur_width = line_width;

        let options: Options = Options::new(self.cur_width as usize)
            .break_words(false)
            .word_splitter(textwrap::WordSplitter::NoHyphenation)
            .preserve_trailing_space(true);
        let mut wrapped_lines = Vec::new();
        for line in text.split_inclusive('\n') {
            textwrap::wrap_single_line(line, &options, &mut wrapped_lines);
        }

        self._lines = wrapped_lines
            .iter()
            .map(|cow| match cow {
                Cow::Borrowed(s) => {
                    let base = text.as_ptr() as usize;
                    let start = s.as_ptr() as usize - base;
                    (start, s.len())
                }
                Cow::Owned(_) => panic! {"Jesse has misunderstood the textwrap library."},
            })
            .collect();
    }

    fn lines<'a>(
        &mut self,
        text: &'a str,
        line_width: u16,
        first: usize,
        num: usize,
    ) -> Vec<(usize, &'a str)> {
        self.wrap(text, line_width);
        let mut ret = Vec::new();
        for i in first..min(self._lines.len(), first + num) {
            let sidx = self._lines[i].0;
            let len = self._lines[i].1;
            ret.push((sidx, &text[sidx..sidx + len]));
        }
        ret
    }

    fn line_of_idx(&mut self, text: &str, idx: usize, line_width: u16) -> usize {
        self.wrap(text, line_width);
        match self._lines.binary_search_by_key(&idx, |(key, _)| *key) {
            Ok(l) => l,
            Err(l) => l - 1,
        }
    }

    fn start_idx_of_line(&mut self, text: &str, line: usize, line_width: u16) -> Option<usize> {
        self.wrap(text, line_width);
        Some(self._lines.get(line)?.0)
    }
}

impl Settings {
    pub fn new() -> Self {
        Self {
            text_width_percent: DEFAULT_TEXT_WIDTH_PERCENT,
            full_text_width: false,
        }
    }

    pub fn line_width(&self, terminal_width: u16) -> u16 {
        ((terminal_width as usize) * (self.text_width_percent as usize) / 100) as u16
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

pub trait PlatformAdapter {
    fn get_texts(&self) -> Vec<TextHandle>;
}

impl PlatformAdapter for () {
    fn get_texts(&self) -> Vec<TextHandle> {
        Vec::new()
    }
}

pub struct TextHandle {
    pub name: String,
    pub length: usize,
    pub last_opened: DateTime<Utc>,
    open: Box<dyn FnOnce()>,
}

impl Drop for OpenText {
    fn drop(&mut self) {
        (self.save)(self.test_log.clone(), self.keypress_log.clone());
    }
}

impl Test {
    fn next(text: &str, test_log: &Vec<TestResult>) -> Test {
        let mut start_index = 0;
        for t in test_log {
            if t.succeeded && t.end_index > start_index {
                start_index = t.end_index;
            }
        }

        let avg_50 = test_log
            .iter()
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .rev()
            .take(50)
            .sum::<usize>()
            / 50;
        let max_10 = test_log
            .iter()
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .rev()
            .take(10)
            .max()
            .unwrap_or(STARTING_SAMPLE_SIZE);
        let best = usize::max(avg_50, max_10) + 5;

        let wrong_num = test_log
            .iter()
            .rev()
            .take_while(|t| !t.succeeded)
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .count();

        let full = text
            .chars()
            .skip(start_index)
            .take(best)
            .collect::<String>();

        let len = full
            .chars()
            .rev()
            .skip(wrong_num * 5)
            .skip_while(|c| !c.is_whitespace())
            .collect::<Vec<_>>()
            .len();

        Test {
            start_index: usize::min(start_index, text.len() - 1),
            cur_char: 0,
            length: usize::min(len, text.len() - start_index - 1),
            start_time: Utc::now(),
        }
    }
}

impl OpenText {
    pub fn new<Save>(text: String, test_log: Vec<TestResult>, save: Save) -> Self
    where
        Save: Fn(Vec<TestResult>, Vec<KeyPress>) -> () + 'static,
    {
        let test = Test::next(&text, &test_log);
        Self {
            text,
            focused_char: test.start_index,
            test,
            wrapper: RefCell::new(TextWrapper::new()),
            test_log,
            save: Box::new(save),
            keypress_log: Default::default(),
        }
    }

    pub fn snap_to_cursor(&mut self) {
        self.focused_char = self.test.start_index + self.test.cur_char;
    }

    pub fn scroll(&mut self, line_width: u16, distance: isize) {
        let cur_line = self.line_of_idx(self.focused_char, line_width);
        let new_line = if distance < 0 {
            cur_line.saturating_sub(distance.saturating_abs() as usize)
        } else {
            cur_line.saturating_add(distance as usize)
        };
        self.focused_char = self
            .start_idx_of_line(new_line, line_width)
            .unwrap_or(self.focused_char);
    }

    pub fn get_rolling_average(&self) -> usize {
        self.test_log
            .iter()
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .rev()
            .take(10)
            .sum::<usize>()
            / 10
    }

    pub fn handle_char(&mut self, c: char) {
        let correct = c
            == self
                .text
                .chars()
                .nth(self.test.start_index + self.test.cur_char)
                .unwrap();

        if correct {
            if self.test.cur_char == 0 {
                self.test.start_time = Utc::now();
            }
            self.test.cur_char += 1
        }
        if !correct || self.test.cur_char == self.test.length {
            self.test_log.push(TestResult {
                succeeded: correct,
                start_index: self.test.start_index,
                end_index: self.test.start_index + self.test.cur_char,
                started: self.test.start_time,
                completed: Utc::now(),
            });
            self.test = Test::next(&self.text, &self.test_log);
        }

        let log_entry = &KeyPress {
            correct,
            key: c,
            time: Utc::now(),
        };
        self.keypress_log.push(log_entry.clone());
        self.snap_to_cursor();
    }

    pub fn lines<'a>(&'a self, line_width: u16, first: usize, num: usize) -> Vec<(usize, &'a str)> {
        self.wrapper
            .borrow_mut()
            .lines(&self.text, line_width, first, num)
    }

    pub fn line_of_idx(&self, idx: usize, line_width: u16) -> usize {
        self.wrapper
            .borrow_mut()
            .line_of_idx(&self.text, idx, line_width)
    }

    fn start_idx_of_line(&self, line: usize, line_width: u16) -> Option<usize> {
        self.wrapper
            .borrow_mut()
            .start_idx_of_line(&self.text, line, line_width)
    }
}

impl<PA: PlatformAdapter> App<PA> {
    pub fn new(adapter: PA) -> Self {
        Self {
            adapter,
            settings: Settings::default(),
            running: true,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
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
pub struct TestResult {
    succeeded: bool,
    start_index: usize,
    end_index: usize,
    #[serde(with = "ts_microseconds")]
    started: DateTime<Utc>,
    #[serde(with = "ts_microseconds")]
    completed: DateTime<Utc>,
}
