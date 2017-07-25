use std::cmp;
use std::marker::PhantomData;

use context::{Context, TextStyle};
use event::{KeyEvent, MouseEvent, UIResult};
use geom::*;
use param::{Param, RefParam};
use widget::Widget;


#[derive(Debug)]
pub struct ScrollPaneState {
    top: i32,
}

impl ScrollPaneState {
    pub fn new() -> ScrollPaneState {
        ScrollPaneState {
            top: 0,
        }
    }
}

#[derive(Debug)]
pub struct ScrollPane<'a, W> {
    state: &'a mut ScrollPaneState,
    size: Point,
    scroll_step: i32,
    child: W,
}

impl<'a, W> ScrollPane<'a, W> {
    pub fn new(state: &'a mut ScrollPaneState,
               size: Point,
               child: W) -> ScrollPane<'a, W> {
        ScrollPane {
            state: state,
            size: size,
            scroll_step: 10,
            child: child,
        }
    }

    pub fn scroll_step(self, scroll_step: i32) -> Self {
        ScrollPane {
            scroll_step: scroll_step,
            .. self
        }
    }
}

impl<'a, W> ScrollPane<'a, W> {
    #[inline]
    fn child_surface_info<Ctx>(&self, ctx: &Ctx) -> (Point, Point, Rect)
            where Ctx: Context, W: Widget<Ctx> {
        let cur_size = ctx.cur_bounds().size();
        let child_size = self.child.min_size();
        let inner_size = Point {
            x: cur_size.x,
            y: cmp::max(cur_size.y, child_size.y),
        };

        let src_pos = Point { x: 0, y: self.state.top };

        (inner_size, src_pos, Rect::sized(cur_size))
    }
}

impl<'a, Ctx: Context, W: Widget<Ctx>> Widget<Ctx> for ScrollPane<'a, W> {
    type Event = W::Event;

    fn min_size(&self) -> Point {
        self.size
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        let (inner_size, src_pos, dest_rect) = self.child_surface_info(ctx);
        ctx.with_surface(inner_size, src_pos, dest_rect, |ctx| {
            self.child.on_paint(ctx);
        });
    }

    fn on_key(&mut self, ctx: &mut Ctx, evt: KeyEvent<Ctx>) -> UIResult<Self::Event> {
        // TODO: match up/down arrows

        let (inner_size, src_pos, dest_rect) = self.child_surface_info(ctx);
        ctx.with_surface(inner_size, src_pos, dest_rect, |ctx| {
            self.child.on_key(ctx, evt)
        })
    }

    fn on_mouse(&mut self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        match evt {
            MouseEvent::Wheel(dir) => {
                let child_size = self.child.min_size();
                let raw_top = self.state.top - dir as i32 * self.scroll_step;
                self.state.top = cmp::max(0, cmp::min(child_size.y, raw_top));
                return UIResult::NoEvent;
            },
            _ => {},
        }

        let (inner_size, src_pos, dest_rect) = self.child_surface_info(ctx);
        ctx.with_surface(inner_size, src_pos, dest_rect, |ctx| {
            self.child.on_mouse(ctx, evt)
        })
    }
}
