use std::ops::Index;
use std::slice::Iter;
use unicode_segmentation::UnicodeSegmentation;

/// String indexed by unicode segments
pub struct GlyphString {
    pub string: String,
    pub glyphs: Box<[(usize, usize)]>,
}

impl GlyphString {
    pub fn new(string: String) -> GlyphString {
        let indices: Box<[(usize, usize)]> = string
            .graphemes(true)
            .map(|s| {
                let idx = s.as_ptr() as usize - string.as_ptr() as usize;
                (idx, idx + s.len())
            })
            .collect();

        GlyphString {
            string,
            glyphs: indices,
        }
    }

    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    pub fn index_of_ptr(&self, ptr: usize) -> usize {
        match self.glyphs.binary_search_by_key(&ptr, |&(start, _)| start) {
            Ok(idx) => idx,
            Err(idx) => idx,
        }
    }

    pub fn glyphs<'a>(&'a self) -> UChars<'a> {
        UChars {
            str: &self.string,
            iter: self.glyphs.iter(),
        }
    }
}

pub struct UChars<'a> {
    str: &'a str,
    iter: Iter<'a, (usize, usize)>,
}

impl<'a> Iterator for UChars<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let (start, end) = *self.iter.next()?;
        Some(&self.str[start..end])
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (start, end) = *self.iter.nth(n)?;
        Some(&self.str[start..end])
    }
}

impl Index<usize> for GlyphString {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        &self.string[self.glyphs[index].0..self.glyphs[index].1]
    }
}
