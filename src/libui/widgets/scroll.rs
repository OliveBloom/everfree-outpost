use std::cmp;
use std::marker::PhantomData;

use context::{Context, ScrollBarStyle};
use event::{KeyEvent, MouseEvent, UIResult};
use geom::*;
use param::{Param, RefParam};
use widget::Widget;


#[derive(Debug)]
pub struct ScrollPaneInner<'a, W> {
    top: &'a mut i32,
    size: Point,
    scroll_step: i32,
    child: W,
}

impl<'a, W> ScrollPaneInner<'a, W> {
    pub fn new(state: &'a mut i32,
               size: Point,
               child: W) -> ScrollPaneInner<'a, W> {
        ScrollPaneInner {
            top: state,
            size: size,
            scroll_step: 10,
            child: child,
        }
    }

    pub fn scroll_step(self, scroll_step: i32) -> Self {
        ScrollPaneInner {
            scroll_step: scroll_step,
            .. self
        }
    }
}

impl<'a, W> ScrollPaneInner<'a, W> {
    #[inline]
    fn child_surface_info<Ctx>(&self, ctx: &Ctx) -> (Point, Point, Rect)
            where Ctx: Context, W: Widget<Ctx> {
        let cur_size = ctx.cur_bounds().size();
        let child_size = self.child.min_size();
        let inner_size = Point {
            x: cur_size.x,
            y: cmp::max(cur_size.y, child_size.y),
        };

        let src_pos = Point { x: 0, y: *self.top };

        (inner_size, src_pos, Rect::sized(cur_size))
    }
}

impl<'a, Ctx: Context, W: Widget<Ctx>> Widget<Ctx> for ScrollPaneInner<'a, W> {
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
                let raw_top = *self.top - dir as i32 * self.scroll_step;
                *self.top = cmp::max(0, cmp::min(child_size.y, raw_top));
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


pub struct ScrollBar<Ctx: Context, D: Direction> {
    value: i16,
    max: i16,
    step: i16,
    style: Ctx::ScrollBarStyle,
    _marker: PhantomData<D>,
}

impl<Ctx: Context, D: Direction> ScrollBar<Ctx, D> {
    pub fn new(value: i16, max: i16) -> ScrollBar<Ctx, D> {
        ScrollBar {
            value: value,
            max: max,
            step: 10,
            style: Ctx::ScrollBarStyle::default(),
            _marker: PhantomData,
        }
    }

    pub fn step(self, step: i16) -> Self {
        ScrollBar {
            step: step,
            .. self
        }
    }

    pub fn style(self, style: Ctx::ScrollBarStyle) -> Self {
        ScrollBar {
            style: style,
            .. self
        }
    }


    fn top_button_bounds(&self, ctx: &Ctx) -> Rect {
        let size = D::make_point(self.style.top_button_height(),
                                 self.style.width());
        Rect::sized(size)
    }

    fn bottom_button_bounds(&self, ctx: &Ctx) -> Rect {
        let size = D::make_point(self.style.top_button_height(),
                                 self.style.width());
        let bottom = ctx.cur_bounds().size().y;
        let offset = D::make_point(bottom - D::major(size), 0);
        Rect::sized(size) + offset
    }

    fn bar_bounds(&self, ctx: &Ctx) -> Rect {
        let top = D::make_point(self.style.top_button_height(), 0);
        let bottom = D::make_point(self.style.bottom_button_height(), 0);
        Rect::sized(ctx.cur_bounds().size())
            .inset(top.x, top.y, bottom.x, bottom.y)
    }
}

impl<Ctx: Context, D: Direction> Widget<Ctx> for ScrollBar<Ctx, D> {
    type Event = i16;

    fn min_size(&self) -> Point {
        let min_w = self.style.width();
        let min_h = self.style.top_button_height() +
                    self.style.handle_height() +
                    self.style.bottom_button_height();
        D::make_point(min_h, min_w)
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        let top_bounds = self.top_button_bounds(ctx);
        let top_pressed = ctx.with_bounds(top_bounds,
                                          |ctx| ctx.mouse_pressed_over() && ctx.mouse_over());
        let bottom_bounds = self.bottom_button_bounds(ctx);
        let bottom_pressed = ctx.with_bounds(bottom_bounds,
                                             |ctx| ctx.mouse_pressed_over() && ctx.mouse_over());

        ctx.draw_scroll_bar(self.style, self.value, self.max, top_pressed, bottom_pressed);
    }

    fn on_mouse(&mut self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        match evt {
            MouseEvent::Wheel(dir) => {
                let adjust = -(dir as i16 * self.step);
                let new_value =
                    if self.value < -adjust { 0 }
                    else if self.max - self.value < adjust { self.max }
                    else { self.value + adjust };
                UIResult::Event(new_value)
            },
            _ => UIResult::Unhandled,
        }
    }
}
