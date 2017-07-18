use context::Context;
use event::{KeyEvent, MouseEvent};
use geom::{Point, Rect};


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UIResult<T> {
    Event(T),
    NoEvent,
    Unhandled,
}

pub trait Widget<Ctx: Context> {
    type Event;


    fn min_size(&self,
                ctx: &Ctx) -> Point;


    fn on_paint(&self,
                ctx: &mut Ctx) {
        // No-op
    }

    fn on_key(&mut self,
              ctx: &mut Ctx,
              evt: KeyEvent<Ctx>) -> UIResult<Self::Event> {
        UIResult::Unhandled
    }

    fn on_mouse(&mut self,
                ctx: &mut Ctx,
                evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        UIResult::Unhandled
    }
}
