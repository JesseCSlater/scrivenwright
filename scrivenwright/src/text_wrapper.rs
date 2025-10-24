use crate::glyph_string::GlyphString;
use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::{max, min};
use textwrap::Options;

pub struct TextWrapper {
    w: RefCell<Option<Inner>>,
}

impl TextWrapper {
    pub fn new() -> TextWrapper {
        TextWrapper {
            w: RefCell::new(None),
        }
    }

    fn with_inner<F, R>(&self, text: &GlyphString, line_width: u16, f: F) -> R
    where
        F: FnOnce(&mut Inner) -> R,
    {
        if self.w.borrow().is_none() {
            self.w.replace(Some(Inner::new(text, line_width)));
        }

        let mut borrow = self.w.borrow_mut();
        let inner = borrow.as_mut().unwrap();
        f(inner)
    }

    pub fn lines<'a>(
        &self,
        text: &'a GlyphString,
        line_width: u16,
        first: usize,
        num: usize,
    ) -> Vec<(usize, usize)> {
        self.with_inner(text, line_width, |w| w.lines(text, line_width, first, num))
    }

    pub fn line_offset_of_idx(
        &self,
        text: &GlyphString,
        line_width: u16,
        idx: usize,
    ) -> Option<(usize, usize)> {
        self.with_inner(text, line_width, |w| {
            w.line_offset_of_idx(text, line_width, idx)
        })
    }

    pub fn move_cursor(
        &mut self,
        text: &GlyphString,
        line_width: u16,
        cur_char: usize,
        dir: Dir,
    ) -> Option<usize> {
        self.with_inner(text, line_width, |w| {
            w.move_cursor(text, line_width, cur_char, dir)
        })
    }
}

#[derive(Debug)]
struct Inner {
    cur_width: u16,
    lines: Box<[(usize, usize)]>,
}

pub(crate) enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Inner {
    fn new(text: &GlyphString, cur_width: u16) -> Inner {
        let options: Options = Options::new(cur_width as usize)
            .break_words(true)
            .word_splitter(textwrap::WordSplitter::HyphenSplitter)
            .preserve_trailing_space(true);
        let mut wrapped_lines = Vec::new();
        for line in text.string.split_inclusive('\n') {
            textwrap::wrap_single_line(line, &options, &mut wrapped_lines);
        }
        let base = text.string.as_ptr() as usize;

        let inner = Inner {
            cur_width,
            lines: wrapped_lines
                .iter()
                .map(|cow| match cow {
                    Cow::Borrowed(s) => {
                        let start = s.as_ptr() as usize - base;
                        let end = start + s.len();
                        (text.index_of_ptr(start), text.index_of_ptr(end))
                    }
                    Cow::Owned(_) => panic! {"Jesse has misunderstood the textwrap library."},
                })
                .collect(),
        };
        inner
    }

    fn wrap(&mut self, text: &GlyphString, line_width: u16) {
        if line_width == self.cur_width {
            return;
        };
        *self = Inner::new(text, line_width);
    }

    fn lines<'a>(
        &mut self,
        text: &'a GlyphString,
        line_width: u16,
        first: usize,
        num: usize,
    ) -> Vec<(usize, usize)> {
        self.wrap(text, line_width);
        let mut ret = Vec::new();
        for i in first..min(self.lines.len(), first + num) {
            ret.push((self.lines[i].0, self.lines[i].1));
        }
        ret
    }

    fn move_cursor(
        &mut self,
        text: &GlyphString,
        line_width: u16,
        cur_char: usize,
        dir: Dir,
    ) -> Option<usize> {
        self.wrap(text, line_width);
        let (line, offset) = self.line_offset_of_idx(text, line_width, cur_char)?;
        Some(match dir {
            Dir::Up => {
                let &(start, end) = self.lines.get(line.checked_sub(1)?)?;
                min(start + offset, end - 1)
            }
            Dir::Down => {
                let &(start, end) = self.lines.get(line + 1)?;
                min(start + offset, end - 1)
            }
            Dir::Right => min(cur_char + 1, self.lines[line].1 - 1),
            Dir::Left => max(cur_char.saturating_sub(1), self.lines[line].0),
        })
    }

    fn line_offset_of_idx(
        &mut self,
        text: &GlyphString,
        line_width: u16,
        idx: usize,
    ) -> Option<(usize, usize)> {
        self.wrap(text, line_width);
        match self.lines.binary_search_by_key(&idx, |&(start, _)| start) {
            Ok(l) => Some((l, 0)),
            Err(l) => Some((l - 1, idx - self.lines[l - 1].0)),
        }
    }
}
