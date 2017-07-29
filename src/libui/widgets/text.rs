use std::marker::PhantomData;

use context::{Context, TextStyle};
use event::{MouseEvent, UIResult};
use geom::*;
use param::{Param, RefParam};
use widget::Widget;


#[derive(Clone, Debug)]
pub struct Label<'a, Ctx: Context> {
    text: &'a str,
    style: Ctx::TextStyle,
}

impl<'a, Ctx: Context> Label<'a, Ctx> {
    pub fn new(text: &'a str) -> Label<'a, Ctx> {
        Label {
            text: text,
            style: Ctx::TextStyle::default(),
        }
    }

    pub fn text(self, text: &'a str) -> Self {
        Label {
            text: text,
            .. self
        }
    }

    pub fn style(self, style: Ctx::TextStyle) -> Self {
        Label {
            style: style,
            .. self
        }
    }
}

impl<'a, Ctx: Context> Widget<Ctx> for Label<'a, Ctx> {
    type Event = ();

    fn min_size(&self) -> Point {
        self.style.text_size(self.text)
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        ctx.draw_str(self.text, self.style);
    }

    fn on_mouse(&self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        match evt {
            MouseEvent::Down(_) => UIResult::NoEvent,
            MouseEvent::Up(_) => {
                if ctx.mouse_pressed_over() {
                    UIResult::Event(())
                } else {
                    UIResult::NoEvent
                }
            },
            _ => UIResult::Unhandled,
        }
    }
}


#[derive(Clone, Debug)]
pub struct WrappedLabel<'a, Ctx: Context> {
    text: &'a str,
    width: i32,
    style: Ctx::TextStyle,
}

impl<'a, Ctx: Context> WrappedLabel<'a, Ctx> {
    pub fn new(text: &'a str, width: i32) -> WrappedLabel<'a, Ctx> {
        WrappedLabel {
            text: text,
            width: width,
            style: Ctx::TextStyle::default(),
        }
    }

    pub fn text(self, text: &'a str) -> Self {
        WrappedLabel {
            text: text,
            .. self
        }
    }

    pub fn width(self, width: i32) -> Self {
        WrappedLabel {
            width: width,
            .. self
        }
    }

    pub fn style(self, style: Ctx::TextStyle) -> Self {
        WrappedLabel {
            style: style,
            .. self
        }
    }


    fn iter_lines(&self) -> WrappedLines<'a, Ctx> {
        WrappedLines {
            text: self.text,
            width: self.width,
            style: self.style,
        }
    }
}

struct WrappedLines<'a, Ctx: Context> {
    text: &'a str,
    width: i32,
    style: Ctx::TextStyle,
}

impl<'a, Ctx: Context> Iterator for WrappedLines<'a, Ctx> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.text.len() == 0 {
            return None;
        }

        let mut x = 0;
        let mut word_start = None;
        for (i, b) in self.text.bytes().enumerate() {
            if b == b' ' {
                if let Some(start) = word_start {
                    word_start = None;

                    // Try adding the current word to the line
                    let word = &self.text[start .. i];
                    x += self.style.text_size(word).x;

                    // Push this word onto the next line?  The `start != 0` check prevents an
                    // infinite loop if a single word is too long for a line.
                    if x >= self.width && start != 0 {
                        // We include the spaces in the preceding line.  Note that the spaces may
                        // extend past EOL (we don't check for this), but this should be okay
                        // because spaces are invisible.
                        let (line, rest) = self.text.split_at(start);
                        self.text = rest;
                        return Some(line);
                    }
                }
                x += self.style.space_width();
            } else {
                if word_start.is_none() {
                    word_start = Some(i);
                }
            }
        }

        let split_idx;
        if let Some(start) = word_start {
            let word = &self.text[start ..];
            x += self.style.text_size(word).x;

            // If the last word put the width over the limit, then we stop after the previous word.
            // Otherwise, we take the entire remainder of the text.
            if x >= self.width && start != 0 {
                split_idx = start;
            } else {
                split_idx = self.text.len();
            }
        } else {
            // The text ended with some spaces.  The previous word was under the limit, and spaces
            // don't count, so emit the whole text.
            split_idx = self.text.len();
        };

        let (line, rest) = self.text.split_at(split_idx);
        self.text = rest;
        Some(line)
    }
}

impl<'a, Ctx: Context> Widget<Ctx> for WrappedLabel<'a, Ctx> {
    type Event = ();

    fn min_size(&self) -> Point {
        let lines = self.iter_lines().count();
        Point { x: self.width, y: lines as i32 * self.style.line_height() }
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        let mut y = 0;
        let h = self.style.line_height();

        for line in self.iter_lines() {
            let bounds = Rect::new(0, y, self.width, y + h);
            ctx.with_bounds(bounds, |ctx| ctx.draw_str(line, self.style));
            y += h;
        }
    }

    fn on_mouse(&self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        match evt {
            MouseEvent::Down(_) => UIResult::NoEvent,
            MouseEvent::Up(_) => {
                if ctx.mouse_pressed_over() {
                    UIResult::Event(())
                } else {
                    UIResult::NoEvent
                }
            },
            _ => UIResult::Unhandled,
        }
    }
}
