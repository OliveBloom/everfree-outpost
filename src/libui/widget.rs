use context::Context;
use event::{KeyEvent, MouseEvent, UIResult};
use geom::{Point, Rect};


pub trait Widget<Ctx: Context> {
    type Event;


    fn min_size(&self) -> Point;

    fn requested_visibility(&self, ctx: &Ctx) -> Option<Rect> { None }


    fn on_paint(&self,
                ctx: &mut Ctx) {
        // No-op
    }

    fn on_key(&self,
              ctx: &mut Ctx,
              evt: KeyEvent<Ctx>) -> UIResult<Self::Event> {
        UIResult::Unhandled
    }

    fn on_mouse(&self,
                ctx: &mut Ctx,
                evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        UIResult::Unhandled
    }
}
