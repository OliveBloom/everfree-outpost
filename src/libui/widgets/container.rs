use std::prelude::v1::*;
use std::cell::Cell;
use std::cmp;
use std::marker::PhantomData;
use std::ops::Range;

use context::Context;
use event::{KeyEvent, KeyInterp, MouseEvent, UIResult};
use geom::*;
use widget::Widget;


pub struct ChildWidget<W, F> {
    w: W,
    f: F,
    align: Align,
}

impl<W, F> ChildWidget<W, F> {
    pub fn new(w: W, f: F) -> ChildWidget<W, F> {
        ChildWidget {
            w: w,
            f: f,
            align: Align::Stretch,
        }
    }

    pub fn align(self, align: Align) -> Self {
        ChildWidget  {
            align: align,
            .. self
        }
    }
}


pub trait Contents<Ctx: Context, R> {
    fn accept<V: Visitor<Ctx, R>>(&self, v: &mut V);
}

pub trait Visitor<Ctx: Context, R> {
    fn visit<W, F>(&mut self, cw: &ChildWidget<W, F>)
        where W: Widget<Ctx>, F: Fn(W::Event) -> R;
}


impl<Ctx, R, W1, F1, C2> Contents<Ctx, R> for (ChildWidget<W1, F1>, C2)
        where Ctx: Context,
              W1: Widget<Ctx>,
              F1: Fn(W1::Event) -> R,
              C2: Contents<Ctx, R> {
    fn accept<V: Visitor<Ctx, R>>(&self, v: &mut V) {
        v.visit(&self.0);
        self.1.accept(v);
    }
}

impl<Ctx: Context, R> Contents<Ctx, R> for () {
    fn accept<V: Visitor<Ctx, R>>(&self, v: &mut V) {}
}


pub struct GenWidgets<W, F, G> {
    range: Range<usize>,
    gen: G,
    _marker: PhantomData<fn() -> ChildWidget<W, F>>,
}

impl<W, F, G> GenWidgets<W, F, G>
        where G: Fn(usize) -> ChildWidget<W, F> {
    pub fn new(range: Range<usize>, gen: G) -> GenWidgets<W, F, G> {
        GenWidgets {
            range: range,
            gen: gen,
            _marker: PhantomData,
        }
    }
}

impl<Ctx, R, W, F, G> Contents<Ctx, R> for GenWidgets<W, F, G>
        where Ctx: Context,
              W: Widget<Ctx>,
              F: Fn(W::Event) -> R,
              G: Fn(usize) -> ChildWidget<W, F> {
    fn accept<V: Visitor<Ctx, R>>(&self, v: &mut V) {
        for idx in self.range.clone() {
            let cw = (self.gen)(idx);
            v.visit(&cw);
        }
    }
}


impl<Ctx, R, W, F> Contents<Ctx, R> for [ChildWidget<W, F>]
        where Ctx: Context,
              W: Widget<Ctx>,
              F: Fn(W::Event) -> R {
    fn accept<V: Visitor<Ctx, R>>(&self, v: &mut V) {
        for cw in self {
            v.visit(cw);
        }
    }
}

impl<Ctx, R, W, F> Contents<Ctx, R> for Vec<ChildWidget<W, F>>
        where Ctx: Context,
              W: Widget<Ctx>,
              F: Fn(W::Event) -> R {
    fn accept<V: Visitor<Ctx, R>>(&self, v: &mut V) {
        (**self).accept(v);
    }
}


#[macro_export]
macro_rules! contents {
    () => { () };
    ($a:expr $(, $rest:expr)*) => { ($a, contents!($($rest),*)) };
    ($($a:expr,)*) => { contents!($($a),*) };
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}


struct Layout<D: Direction> {
    major_pos: i32,
    minor_size: i32,
    spacing: i32,
    _marker: PhantomData<D>,
}

impl<D: Direction> Layout<D> {
    pub fn new(minor_size: i32, spacing: i32) -> Layout<D> {
        Layout {
            major_pos: 0,
            minor_size: minor_size,
            spacing: spacing,
            _marker: PhantomData,
        }
    }

    pub fn place(&mut self, size: Point, align: Align) -> Rect {
        let major = self.major_pos;
        self.major_pos += D::major(size) + self.spacing;

        let minor_size = D::minor(size);
        let minor_total = self.minor_size;
        let (minor0, minor1) = match align {
            Align::Start => (0, minor_size),
            Align::Center => {
                let offset = (minor_total - minor_size) / 2;
                (offset, offset + minor_size)
            },
            Align::End => (minor_total - minor_size, minor_total),
            Align::Stretch => (0, minor_total),
        };

        Rect {
            min: D::make_point(major, minor0),
            max: D::make_point(major + D::major(size), minor1),
        }
    }
}


pub struct Group<'s, Ctx: Context, D: Direction, R, C: Contents<Ctx, R>> {
    focus: &'s Cell<usize>,
    contents: C,
    spacing: i32,
    _marker: PhantomData<(Ctx, D, R)>,
}

impl<'s, Ctx: Context, R, C: Contents<Ctx, R>> Group<'s, Ctx, Horizontal, R, C> {
    pub fn horiz(focus: &'s Cell<usize>, contents: C) -> Group<'s, Ctx, Horizontal, R, C> {
        Group {
            focus: focus,
            contents: contents,
            spacing: 0,
            _marker: PhantomData,
        }
    }
}

impl<'s, Ctx: Context, R, C: Contents<Ctx, R>> Group<'s, Ctx, Vertical, R, C> {
    pub fn vert(focus: &'s Cell<usize>, contents: C) -> Group<'s, Ctx, Vertical, R, C> {
        Group {
            focus: focus,
            contents: contents,
            spacing: 0,
            _marker: PhantomData,
        }
    }
}

impl<'s, Ctx: Context, D: Direction, R, C: Contents<Ctx, R>> Group<'s, Ctx, D, R, C> {
    pub fn spacing(self, spacing: i32) -> Self {
        Group {
            spacing: spacing,
            .. self
        }
    }

    fn adjust_focus_clamp(&self, delta: i32, len: usize) {
        if delta >= 0 {
            let delta = delta as usize;
            if delta >= len - self.focus.get() {
                self.focus.set(len - 1);
            } else {
                self.focus.set(self.focus.get() + delta);
            }
        } else {
            let delta = (-delta) as usize;
            if delta > self.focus.get() {
                self.focus.set(0);
            } else {
                self.focus.set(self.focus.get() - delta);
            }
        }
    }
}

impl<'s, Ctx, D, R, C> Widget<Ctx> for Group<'s, Ctx, D, R, C>
        where Ctx: Context, D: Direction, C: Contents<Ctx, R> {
    type Event = R;

    fn min_size(&self) -> Point {
        struct SizeVisitor<D> {
            size: Point,
            spacing: i32,
            _marker: PhantomData<D>,
        }
        impl<Ctx: Context, D: Direction, R> Visitor<Ctx, R> for SizeVisitor<D> {
            fn visit<W, F>(&mut self, cw: &ChildWidget<W, F>)
                    where W: Widget<Ctx>, F: Fn(W::Event) -> R {
                let child_size = cw.w.min_size();
                let add_minor = cmp::max(0, D::minor(child_size) - D::minor(self.size));
                let add_major = D::major(child_size) +
                    if D::major(self.size) != 0 { self.spacing } else { 0 };
                self.size = self.size + D::make_point(add_major, add_minor);
            }
        }
        let mut v: SizeVisitor<D> = SizeVisitor {
            size: Point { x: 0, y: 0 },
            spacing: self.spacing,
            _marker: PhantomData,
        };
        self.contents.accept(&mut v);
        v.size
    }

    fn requested_visibility(&self, ctx: &Ctx) -> Option<Rect> {
        struct ReqVisVisitor<'c, Ctx: Context+'c, D: Direction> {
            ctx: &'c Ctx,
            layout: Layout<D>,
            idx: usize,
            focus: usize,
            req_vis: Option<Rect>,
        }
        impl<'c, Ctx: Context, D: Direction, R> Visitor<Ctx, R> for ReqVisVisitor<'c, Ctx, D> {
            fn visit<W, F>(&mut self, cw: &ChildWidget<W, F>)
                    where W: Widget<Ctx>, F: Fn(W::Event) -> R {
                let size = cw.w.min_size();
                let bounds = self.layout.place(size, cw.align);

                if self.idx == self.focus {
                    self.req_vis = Some(bounds);
                }
                self.idx += 1;
            }
        }

        let bounds_size = ctx.cur_bounds().size();
        let mut v: ReqVisVisitor<_, D> = ReqVisVisitor {
            ctx: ctx,
            layout: Layout::new(D::minor(bounds_size), self.spacing),
            idx: 0,
            focus: self.focus.get(),
            req_vis: None,
        };
        self.contents.accept(&mut v);
        v.req_vis
    }

    fn on_paint(&self, ctx: &mut Ctx) {
        struct PaintVisitor<'c, Ctx: Context+'c, D: Direction> {
            ctx: &'c mut Ctx,
            layout: Layout<D>,
            idx: usize,
            focus: usize,
        }
        impl<'c, Ctx: Context, D: Direction, R> Visitor<Ctx, R> for PaintVisitor<'c, Ctx, D> {
            fn visit<W, F>(&mut self, cw: &ChildWidget<W, F>)
                    where W: Widget<Ctx>, F: Fn(W::Event) -> R {
                let size = cw.w.min_size();
                let bounds = self.layout.place(size, cw.align);
                self.ctx.with_focus(self.idx == self.focus, |ctx| {
                    ctx.with_bounds(bounds, |ctx| {
                        cw.w.on_paint(ctx);
                    })
                });
                self.idx += 1;
            }
        }

        let bounds_size = ctx.cur_bounds().size();
        let mut v: PaintVisitor<_, D> = PaintVisitor {
            ctx: ctx,
            layout: Layout::new(D::minor(bounds_size), self.spacing),
            idx: 0,
            focus: self.focus.get(),
        };
        self.contents.accept(&mut v);
    }

    fn on_key(&self, ctx: &mut Ctx, evt: KeyEvent<Ctx>) -> UIResult<R> {
        struct KeyVisitor<'c, Ctx: Context+'c, D: Direction, R> {
            ctx: &'c mut Ctx,
            idx: usize,
            focus: usize,
            layout: Layout<D>,
            event: Option<KeyEvent<Ctx>>,
            result: UIResult<R>,
        }
        impl<'c, Ctx, D, R> Visitor<Ctx, R> for KeyVisitor<'c, Ctx, D, R>
                where Ctx: Context, D: Direction {
            fn visit<W, F>(&mut self, cw: &ChildWidget<W, F>)
                    where W: Widget<Ctx>, F: Fn(W::Event) -> R {
                if self.idx == self.focus {
                    let size = cw.w.min_size();
                    let bounds = self.layout.place(size, cw.align);
                    let evt = self.event.take().unwrap();
                    self.result = self.ctx.with_bounds(bounds, |ctx| {
                        cw.w.on_key(ctx, evt).map(|e| (cw.f)(e))
                    });
                }
                self.idx += 1;
            }
        }

        let bounds_size = ctx.cur_bounds().size();
        let mut v: KeyVisitor<Ctx, D, R> = KeyVisitor {
            ctx: ctx,
            idx: 0,
            focus: self.focus.get(),
            layout: Layout::new(D::minor(bounds_size), self.spacing),
            event: Some(evt.clone()),
            result: UIResult::Unhandled,
        };
        self.contents.accept(&mut v);
        if v.result.is_handled() {
            return v.result;
        }

        let ctx = v.ctx;
        match ctx.interp_key(evt) {
            // TODO: respect choice of Direction
            Some(KeyInterp::FocusY(delta)) => {
                self.adjust_focus_clamp(delta as i32, v.idx);
                UIResult::NoEvent
            },
            _ => UIResult::Unhandled,
        }
    }

    fn on_mouse(&self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<R> {
        struct MouseVisitor<'s, 'c, Ctx: Context+'c, D: Direction, R> {
            ctx: &'c mut Ctx,
            focus: &'s Cell<usize>,
            idx: usize,
            layout: Layout<D>,
            event: MouseEvent<Ctx>,
            result: UIResult<R>,
        }
        impl<'s, 'c, Ctx, D, R> Visitor<Ctx, R> for MouseVisitor<'s, 'c, Ctx, D, R>
                where Ctx: Context, D: Direction {
            fn visit<W, F>(&mut self, cw: &ChildWidget<W, F>)
                    where W: Widget<Ctx>, F: Fn(W::Event) -> R {
                let idx = self.idx;
                self.idx += 1;

                if self.result.is_handled() {
                    // A previous child already handled the input, so stop processing it.
                    return;
                }

                let size = cw.w.min_size();
                let bounds = self.layout.place(size, cw.align);
                let evt = &self.event;
                self.result = self.ctx.with_bounds(bounds, |ctx| {
                    if !ctx.mouse_target() {
                        return UIResult::Unhandled;
                    }
                    cw.w.on_mouse(ctx, evt.clone()).map(|e| (cw.f)(e))
                });

                if self.result.is_handled() {
                    // This child handled the input, so update the container focus.
                    self.focus.set(idx);
                }
            }
        }

        let bounds_size = ctx.cur_bounds().size();
        let mut v: MouseVisitor<Ctx, D, R> = MouseVisitor {
            ctx: ctx,
            focus: self.focus,
            idx: 0,
            layout: Layout::new(D::minor(bounds_size), self.spacing),
            event: evt,
            result: UIResult::Unhandled,
        };
        self.contents.accept(&mut v);
        v.result
    }
}
