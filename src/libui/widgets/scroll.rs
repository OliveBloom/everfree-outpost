use std::cell::Cell;
use std::cmp;
use std::marker::PhantomData;

use context::{Context, ScrollBarStyle};
use event::{KeyEvent, MouseEvent, UIResult};
use geom::*;
use param::{Param, RefParam};
use widget::Widget;


pub struct ScrollPane<'s, Ctx: Context, W> {
    top: &'s Cell<i32>,
    size: Point,
    scroll_step: i16,
    child: W,
    bar_style: Ctx::ScrollBarStyle,
}

impl<'s, Ctx: Context, W> ScrollPane<'s, Ctx, W> {
    pub fn new(top: &'s Cell<i32>,
               size: Point,
               child: W) -> ScrollPane<'s, Ctx, W> {
        ScrollPane {
            top: top,
            size: size,
            scroll_step: 10,
            child: child,
            bar_style: Ctx::ScrollBarStyle::default(),
        }
    }

    pub fn scroll_step(self, scroll_step: i16) -> Self {
        ScrollPane {
            scroll_step: scroll_step,
            .. self
        }
    }

    pub fn bar_style(self, bar_style: Ctx::ScrollBarStyle) -> Self {
        ScrollPane {
            bar_style: bar_style,
            .. self
        }
    }
}

impl<'s, Ctx: Context, W: Widget<Ctx>> ScrollPane<'s, Ctx, W> {
    #[inline]
    fn child_surface_info(&self, ctx: &Ctx) -> (Point, Point, Rect, Rect) {
        let cur_size = ctx.cur_bounds().size();
        let child_size = self.child.min_size();
        let inner_size = Point {
            x: cur_size.x - self.bar_style.width(),
            y: cmp::max(cur_size.y, child_size.y),
        };

        let src_pos = Point { x: 0, y: self.top.get() };

        let dest_rect = Rect::new(0, 0, inner_size.x, cur_size.y);
        let bar_rect = Rect::new(inner_size.x, 0, cur_size.x, cur_size.y);

        (inner_size, src_pos, dest_rect, bar_rect)
    }

    fn scroll_max(&self, ctx: &Ctx) -> i32 {
        let child_size = self.child.min_size().y;
        let pane_size = ctx.cur_bounds().size().y;
        if pane_size >= child_size {
            0
        } else {
            child_size - pane_size
        }
    }

    fn bar(&self, ctx: &Ctx) -> ScrollBar<Ctx, Horizontal>
            where W: Widget<Ctx> {
        ScrollBar::new(self.scroll_max(ctx) as i16)
            .value(self.top.get() as i16)
            .step(self.scroll_step)
            .style(self.bar_style)
    }

    fn set_top_clamp(&self, top: i32, ctx: &Ctx) {
        let top = cmp::max(0, cmp::min(self.scroll_max(ctx), top));
        self.top.set(top);
    }

    fn adjust_top_clamp(&self, offset: i32, ctx: &Ctx) {
        let top = self.top.get() + offset;
        self.set_top_clamp(top, ctx);
    }
}

impl<'s, Ctx: Context, W: Widget<Ctx>> Widget<Ctx> for ScrollPane<'s, Ctx, W> {
    type Event = W::Event;

    fn min_size(&self) -> Point {
        self.size
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        let (inner_size, src_pos, dest_rect, bar_rect) = self.child_surface_info(ctx);
        ctx.with_surface(inner_size, src_pos, dest_rect, |ctx| {
            self.child.on_paint(ctx);
        });
        let bar = self.bar(ctx);
        ctx.with_bounds(bar_rect, |ctx| bar.on_paint(ctx));
    }

    fn on_key(&self, ctx: &mut Ctx, evt: KeyEvent<Ctx>) -> UIResult<Self::Event> {
        // TODO: match page up / page down

        let (inner_size, src_pos, dest_rect, _bar_rect) = self.child_surface_info(ctx);
        ctx.with_surface(inner_size, src_pos, dest_rect, |ctx| {
            self.child.on_key(ctx, evt)
        })
    }

    fn on_mouse(&self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
        match evt {
            MouseEvent::Wheel(dir) => {
                let step = -dir as i32 * self.scroll_step as i32;
                self.adjust_top_clamp(step, ctx);
                return UIResult::NoEvent;
            },
            _ => {},
        }

        let (inner_size, src_pos, dest_rect, bar_rect) = self.child_surface_info(ctx);

        let mut bar = self.bar(ctx);
        let r = ctx.with_bounds(bar_rect, |ctx| {
            if !ctx.mouse_target() {
                return UIResult::Unhandled;
            }
            bar.on_mouse(ctx, evt.clone())
        }).and_then(|pos| {;
            self.set_top_clamp(pos as i32, ctx);
            UIResult::NoEvent
        });
        // TODO: make this a macro
        if r.is_handled() {
            return r;
        }

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
    pub fn new(max: i16) -> ScrollBar<Ctx, D> {
        ScrollBar {
            value: 0,
            max: max,
            step: 10,
            style: Ctx::ScrollBarStyle::default(),
            _marker: PhantomData,
        }
    }

    pub fn value(self, value: i16) -> Self {
        ScrollBar {
            value: value,
            .. self
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

    fn on_mouse(&self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<Self::Event> {
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
