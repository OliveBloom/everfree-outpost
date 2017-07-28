use std::cmp;
use std::marker::PhantomData;

use context::Context;
use event::{KeyEvent, MouseEvent, UIResult};
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
            align: Align::Start,
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
    fn accept_mut<V: VisitorMut<Ctx, R>>(&mut self, v: &mut V);
}

pub trait Visitor<Ctx: Context, R> {
    fn visit<W, F>(&mut self, cw: &ChildWidget<W, F>)
        where W: Widget<Ctx>, F: Fn(W::Event) -> R;
}

pub trait VisitorMut<Ctx: Context, R> {
    fn visit_mut<W, F>(&mut self, cw: &mut ChildWidget<W, F>)
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

    fn accept_mut<V: VisitorMut<Ctx, R>>(&mut self, v: &mut V) {
        v.visit_mut(&mut self.0);
        self.1.accept_mut(v);
    }
}

impl<Ctx: Context, R> Contents<Ctx, R> for () {
    fn accept<V: Visitor<Ctx, R>>(&self, v: &mut V) {}
    fn accept_mut<V: VisitorMut<Ctx, R>>(&mut self, v: &mut V) {}
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

        let minor = match align {
            Align::Start => 0,
            Align::Center => (self.minor_size - D::minor(size)) / 2,
            Align::End => self.minor_size - D::minor(size),
        };

        Rect::sized(size) + D::make_point(major, minor)
    }
}


pub struct GroupState {
    focus: usize,
}

impl GroupState {
    pub fn new() -> GroupState {
        GroupState {
            focus: 0,
        }
    }
}

pub struct Group<'a, Ctx: Context, D: Direction, R, C: Contents<Ctx, R>> {
    state: &'a mut GroupState,
    contents: C,
    spacing: i32,
    _marker: PhantomData<(Ctx, D, R)>,
}

impl<'a, Ctx: Context, R, C: Contents<Ctx, R>> Group<'a, Ctx, Horizontal, R, C> {
    pub fn horiz(state: &'a mut GroupState, contents: C) -> Group<'a, Ctx, Horizontal, R, C> {
        Group {
            state: state,
            contents: contents,
            spacing: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a, Ctx: Context, R, C: Contents<Ctx, R>> Group<'a, Ctx, Vertical, R, C> {
    pub fn vert(state: &'a mut GroupState, contents: C) -> Group<'a, Ctx, Vertical, R, C> {
        Group {
            state: state,
            contents: contents,
            spacing: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a, Ctx: Context, D: Direction, R, C: Contents<Ctx, R>> Group<'a, Ctx, D, R, C> {
    pub fn spacing(self, spacing: i32) -> Self {
        Group {
            spacing: spacing,
            .. self
        }
    }
}

impl<'a, Ctx, D, R, C> Widget<Ctx> for Group<'a, Ctx, D, R, C>
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
                    if D::major(self.size) == 0 { self.spacing } else { 0 };
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

    fn on_paint(&self, ctx: &mut Ctx) {
        struct PaintVisitor<'c, Ctx: Context+'c, D: Direction> {
            ctx: &'c mut Ctx,
            layout: Layout<D>,
        }
        impl<'c, Ctx: Context, D: Direction, R> Visitor<Ctx, R> for PaintVisitor<'c, Ctx, D> {
            fn visit<W, F>(&mut self, cw: &ChildWidget<W, F>)
                    where W: Widget<Ctx>, F: Fn(W::Event) -> R {
                let size = cw.w.min_size();
                let bounds = self.layout.place(size, cw.align);
                self.ctx.with_bounds(bounds, |ctx| {
                    cw.w.on_paint(ctx);
                });
            }
        }

        let bounds_size = ctx.cur_bounds().size();
        let mut v: PaintVisitor<_, D> = PaintVisitor {
            ctx: ctx,
            layout: Layout::new(D::minor(bounds_size), self.spacing),
        };
        self.contents.accept(&mut v);
    }

    fn on_key(&mut self, ctx: &mut Ctx, evt: KeyEvent<Ctx>) -> UIResult<R> {
        struct KeyVisitor<'c, Ctx: Context+'c, D: Direction, R> {
            ctx: &'c mut Ctx,
            idx: usize,
            focus: usize,
            layout: Layout<D>,
            event: Option<KeyEvent<Ctx>>,
            result: UIResult<R>,
        }
        impl<'c, Ctx, D, R> VisitorMut<Ctx, R> for KeyVisitor<'c, Ctx, D, R>
                where Ctx: Context, D: Direction {
            fn visit_mut<W, F>(&mut self, cw: &mut ChildWidget<W, F>)
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
            focus: self.state.focus,
            layout: Layout::new(D::minor(bounds_size), self.spacing),
            event: Some(evt),
            result: UIResult::Unhandled,
        };
        self.contents.accept_mut(&mut v);
        v.result
    }

    fn on_mouse(&mut self, ctx: &mut Ctx, evt: MouseEvent<Ctx>) -> UIResult<R> {
        struct MouseVisitor<'a, 'c, Ctx: Context+'c, D: Direction, R> {
            ctx: &'c mut Ctx,
            state: &'a mut GroupState,
            idx: usize,
            layout: Layout<D>,
            event: MouseEvent<Ctx>,
            result: UIResult<R>,
        }
        impl<'a, 'c, Ctx, D, R> VisitorMut<Ctx, R> for MouseVisitor<'a, 'c, Ctx, D, R>
                where Ctx: Context, D: Direction {
            fn visit_mut<W, F>(&mut self, cw: &mut ChildWidget<W, F>)
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
                    self.state.focus = idx;
                }
            }
        }

        let bounds_size = ctx.cur_bounds().size();
        let mut v: MouseVisitor<Ctx, D, R> = MouseVisitor {
            ctx: ctx,
            state: self.state,
            idx: 0,
            layout: Layout::new(D::minor(bounds_size), self.spacing),
            event: evt,
            result: UIResult::Unhandled,
        };
        self.contents.accept_mut(&mut v);
        v.result
    }
}
