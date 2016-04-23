use std::prelude::v1::*;
use std::str;

pub use client_fonts::*;

pub trait FontMetricsExt {
    fn char_index(&self, c: char) -> Option<usize>;
    fn measure_width(&self, s: &str) -> u32;
    fn iter_str<'a, 'b>(&'a self, s: &'b str) -> StrIter<'a, 'b>;
}


impl FontMetricsExt for FontMetrics {
    fn char_index(&self, c: char) -> Option<usize> {
        if c as u32 > 0x100 {
            return None;
        }
        let c = c as u8;

        let mut low = 0;
        let mut high = self.spans.len();
        while low < high {
            let mid = (low + high) / 2;
            let s = &self.spans[mid];
            if c < s.min {
                high = mid;
            } else if c >= s.max {
                low = mid + 1;
            } else {
                // Found the range containing the char.
                let offset = c - s.min;
                return Some(s.base_index as usize + offset as usize);
            }
        }

        None
    }

    fn measure_width(&self, s: &str) -> u32 {
        let mut w = 0;
        for c in s.chars() {
            if w > 0 {
                w += self.spacing as u32;
            }
            if let Some(idx) = self.char_index(c) {
                w += self.widths[idx] as u32;
            } else {
                // TODO: not a great way to handle spaces, or to handle missing chars
                w += self.space_width as u32;
            }
        }
        w
    }

    fn iter_str<'a, 'b>(&'a self, s: &'b str) -> StrIter<'a, 'b> {
        StrIter::new(self, s)
    }
}


pub struct StrIter<'a, 'b> {
    metrics: &'a FontMetrics,
    inner: str::Chars<'b>,
    pos: u32,
}

impl<'a, 'b> StrIter<'a, 'b> {
    fn new(metrics: &'a FontMetrics, s: &'b str) -> StrIter<'a, 'b> {
        StrIter {
            metrics: metrics,
            inner: s.chars(),
            pos: 0,
        }
    }
}

impl<'a, 'b> Iterator for StrIter<'a, 'b> {
    type Item = (Option<usize>, u32);

    fn next(&mut self) -> Option<(Option<usize>, u32)> {
        let c = match self.inner.next() {
            Some(c) => c,
            None => return None,
        };

        let offset = self.pos;
        let opt_idx = self.metrics.char_index(c);

        if self.pos > 0 {
            self.pos += self.metrics.spacing as u32;
        }
        if let Some(idx) = opt_idx {
            self.pos += self.metrics.widths[idx] as u32;
        } else {
            self.pos += self.metrics.space_width as u32;
        }
        Some((opt_idx, offset))
    }
}
