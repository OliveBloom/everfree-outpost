use std::cmp;
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
        let cur_size = ctx.cur_bounds().size();
        // Inset by the border size to get the content area.
        let (nw, se) = self.style.border_size();
        let inner = Rect {
            min: nw,
            max: cur_size - se,
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



#[derive(Clone, Debug)]
pub struct CheckBox<'a, Ctx: Context> {
    text: &'a str,
    checked: bool,
    style_off: Ctx::ButtonStyle,
    style_on: Ctx::ButtonStyle,
    text_style: Ctx::TextStyle,
}

impl<'a, Ctx: Context> CheckBox<'a, Ctx> {
    pub fn new(text: &'a str, checked: bool) -> CheckBox<'a, Ctx> {
        CheckBox {
            text: text,
            checked: checked,
            style_off: Ctx::ButtonStyle::default_off(),
            style_on: Ctx::ButtonStyle::default_on(),
            text_style: Ctx::TextStyle::default(),
        }
    }

    pub fn text(self, text: &'a str) -> Self {
        CheckBox {
            text: text,
            .. self
        }
    }

    pub fn checked(self, checked: bool) -> Self {
        CheckBox {
            checked: checked,
            .. self
        }
    }

    pub fn style(self,
                 style_off: Ctx::ButtonStyle,
                 style_on: Ctx::ButtonStyle) -> Self {
        CheckBox {
            style_off: style_off,
            style_on: style_on,
            .. self
        }
    }

    pub fn text_style(self, text_style: Ctx::TextStyle) -> Self {
        CheckBox {
            text_style: text_style,
            .. self
        }
    }
}

const CHECKBOX_MARGIN: i32 = 2;

impl<'a, Ctx: Context> Widget<Ctx> for CheckBox<'a, Ctx> {
    type Event = bool;

    fn min_size(&self) -> Point {
        let (nw, se) = self.style_off.border_size();
        let box_size = nw + se;
        let text_size = self.text_style.text_size(self.text);
        Point {
            x: box_size.x + text_size.x + CHECKBOX_MARGIN,
            y: cmp::max(box_size.y, text_size.y),
        }
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        let state =
            if ctx.mouse_pressed_over() {
                if ctx.mouse_over() { ButtonState::Down } else { ButtonState::Active }
            } else {
                if ctx.mouse_over() { ButtonState::Hover } else { ButtonState::Up }
            };
        let style =
            if self.checked { self.style_on } else { self.style_off };

        // Compute where to draw the text and the checkbox.  We want the checkbox to be aligned
        // vertically with the first line of the text.
        let text_height = self.text_style.text_size("").y;
        let (nw, se) = style.border_size();
        let box_size = nw + se;
        let y_off = (text_height - box_size.y) / 2;

        let (box_y, text_y) =
            if y_off > 0 { (y_off, 0) }
            else { (0, -y_off) };

        let box_bounds = Rect::sized(box_size) + Point { x: 0, y: box_y };
        let cur_size = ctx.cur_bounds().size();
        let text_bounds = Rect::sized(cur_size).inset(box_size.x + CHECKBOX_MARGIN, text_y, 0, 0);

        ctx.with_bounds(box_bounds, |ctx| {
            ctx.draw_button(style, state);
        });
        ctx.with_bounds(text_bounds, |ctx| {
            ctx.draw_str(self.text, self.text_style);
        });
    }

    fn on_key(&mut self, ctx: &mut Ctx, evt: KeyEvent<Ctx>) -> UIResult<Self::Event> {
        // TODO: only react to "enter" / "activate" keys
        match evt {
            KeyEvent::Down(_) => UIResult::NoEvent,
            KeyEvent::Up(_) => UIResult::Event(!self.checked),
        }
    }

    fn on_mouse(&mut self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        match evt {
            MouseEvent::Down(_) => UIResult::NoEvent,
            MouseEvent::Up(_) => {
                if ctx.mouse_pressed_over() {
                    UIResult::Event(!self.checked)
                } else {
                    UIResult::NoEvent
                }
            },
            _ => UIResult::Unhandled,
        }
    }
}
