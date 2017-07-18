use std::marker::PhantomData;

use context::{Context, TextStyle, ButtonStyle, ButtonState};
use event::{KeyEvent, MouseEvent};
use geom::*;
use param::{Param, RefParam};
use widget::{Widget, UIResult};


#[derive(Clone, Debug)]
pub struct Button<'a, Ctx: Context> {
    text: &'a str,
    style: Ctx::ButtonStyle,
    text_style: Ctx::TextStyle,
}

impl<'a, Ctx: Context> Button<'a, Ctx> {
    pub fn new(text: &'a str) -> Button<'a, Ctx> {
        Button {
            text: text,
            style: Ctx::ButtonStyle::default(),
            text_style: Ctx::TextStyle::default(),
        }
    }

    pub fn text(self, text: &'a str) -> Self {
        Button {
            text: text,
            .. self
        }
    }

    pub fn style(self, style: Ctx::ButtonStyle) -> Self {
        Button {
            style: style,
            .. self
        }
    }

    pub fn text_style(self, text_style: Ctx::TextStyle) -> Self {
        Button {
            text_style: text_style,
            .. self
        }
    }
}

impl<'a, Ctx: Context> Widget<Ctx> for Button<'a, Ctx> {
    type Event = ();

    fn min_size(&self) -> Point {
        let (nw, se) = self.style.border_size();
        self.text_style.text_size(self.text) + nw + se
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        let state =
            if ctx.mouse_pressed_over() {
                if ctx.mouse_over() { ButtonState::Down } else { ButtonState::Active }
            } else {
                if ctx.mouse_over() { ButtonState::Hover } else { ButtonState::Up }
            };
        ctx.draw_button(self.style, state);

        // Compute where to draw the label.  Start with the button's entire area.
        let cur = ctx.cur_bounds();
        // Inset by the border size to get the content area.
        let (nw, se) = self.style.border_size();
        let inner = Rect {
            min: cur.min + nw,
            max: cur.max - se,
        };
        // Center the label text inside the content area.
        let text_size = self.text_style.text_size(self.text);
        let bounds = inner.center(Rect::sized(text_size));

        ctx.with_bounds(bounds, |ctx| {
            ctx.draw_str(self.text, self.text_style);
        });
    }

    fn on_key(&mut self, ctx: &mut Ctx, evt: KeyEvent<Ctx>) -> UIResult<Self::Event> {
        // TODO: only react to "enter" / "activate" keys
        match evt {
            KeyEvent::Down(_) => UIResult::NoEvent,
            KeyEvent::Up(_) => UIResult::Event(()),
        }
    }

    fn on_mouse(&mut self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
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
