use crate::glyph_string::GlyphString;
use crate::text_wrapper::{Dir, TextWrapper};
use chrono::{serde::ts_microseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

const STARTING_SAMPLE_SIZE: usize = 100;

pub struct OpenText {
    pub text: GlyphString,
    pub test: Option<Test>,
    pub focused_glyph: usize,
    wrapper: TextWrapper,
    test_log: Vec<TestResult>,
    keypress_log: Vec<KeyPress>,
    save: Box<dyn Fn(TestResult, Vec<KeyPress>)>,
}

impl OpenText {
    fn next_test(&mut self) {
        let start_index = self
            .test_log
            .iter()
            .rfind(|t| t.succeeded)
            .map_or(0, |t| t.end_index);

        if start_index >= self.text.len() {
            self.test = None;
            return;
        }

        let avg_50 = self
            .test_log
            .iter()
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .rev()
            .take(50)
            .sum::<usize>()
            / 50;
        let max_10 = self
            .test_log
            .iter()
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .rev()
            .take(10)
            .max()
            .unwrap_or(STARTING_SAMPLE_SIZE);
        let best = usize::max(avg_50, max_10) + 5;

        let wrong_num = self
            .test_log
            .iter()
            .rev()
            .take_while(|t| !t.succeeded)
            .map(|t| t.end_index - t.start_index)
            .filter(|&len| len > 5)
            .count();

        let mut remaining = best.saturating_sub(wrong_num * 5);
        let mut in_final_word = true;
        let len = self
            .text
            .glyphs()
            .skip(start_index)
            .take_while(|g| {
                if remaining > 0 {
                    remaining -= 1;
                    true
                } else if remaining == 0 && in_final_word && !g.chars().all(char::is_whitespace) {
                    true
                } else if remaining == 0 && g.chars().all(char::is_whitespace) {
                    in_final_word = false;
                    true
                } else {
                    false
                }
            })
            .collect::<String>()
            .len();

        self.test = Some(Test {
            start_index,
            cur_char: 0,
            length: usize::min(len, self.text.len() - start_index),
            start_time: Utc::now(),
        })
    }

    pub fn new<Save>(text: String, test_log: Vec<TestResult>, save: Save) -> Self
    where
        Save: Fn(TestResult, Vec<KeyPress>) -> () + 'static,
    {
        let text = GlyphString::new(text);
        let mut ret = Self {
            focused_glyph: 0,
            text,
            test: None,
            wrapper: TextWrapper::new(),
            test_log,
            save: Box::new(save),
            keypress_log: Default::default(),
        };
        ret.next_test();
        ret.snap_to_cursor();
        ret
    }

    pub fn snap_to_cursor(&mut self) {
        if let Some(t) = &self.test {
            self.focused_glyph = t.start_index + t.cur_char;
        } else {
            //panic!("{:?} {:?}", self.text.string, self.text.len());
            self.focused_glyph = self.text.len() - 1
        }
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
        if let Some(test) = self.test.as_mut() {
            let correct = c.to_string() == self.text[test.start_index + test.cur_char];

            if correct {
                if test.cur_char == 0 {
                    test.start_time = Utc::now();
                }
                test.cur_char += 1
            }
            if !correct || test.cur_char == test.length {
                let res = TestResult {
                    succeeded: correct,
                    start_index: test.start_index,
                    end_index: test.start_index + test.cur_char,
                    started: test.start_time,
                    completed: Utc::now(),
                };
                self.test_log.push(res.clone());
                (self.save)(res, self.keypress_log.clone());
                self.keypress_log = Vec::new();
                self.next_test();
            }

            let log_entry = KeyPress {
                correct,
                key: c,
                time: Utc::now(),
            };
            self.keypress_log.push(log_entry);
            self.snap_to_cursor();
        }
    }

    pub fn lines<'a>(&'a self, line_width: u16, first: usize, num: usize) -> Vec<(usize, usize)> {
        self.wrapper.lines(&self.text, line_width, first, num)
    }

    pub fn line_offset_of_idx(&self, idx: usize, line_width: u16) -> Option<(usize, usize)> {
        self.wrapper.line_offset_of_idx(&self.text, line_width, idx)
    }

    pub(crate) fn move_cursor(&mut self, line_width: u16, dir: Dir) {
        if let Some(new_focused_glyph) =
            self.wrapper
                .move_cursor(&self.text, line_width, self.focused_glyph, dir)
        {
            self.focused_glyph = new_focused_glyph;
        }
    }
}

#[derive(Clone, Copy)]
pub struct Test {
    pub start_index: usize,
    pub length: usize,
    pub cur_char: usize,
    start_time: DateTime<Utc>,
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
