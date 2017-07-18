use std::marker::PhantomData;

use context::{Context, TextStyle};
use event::MouseEvent;
use geom::*;
use param::{Param, RefParam};
use widget::{Widget, UIResult};


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
