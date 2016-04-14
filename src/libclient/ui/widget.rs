use std::prelude::v1::*;
use std::mem;

use physics::v3::{V2, scalar, Region};

use ui::{UI, Context};
use ui::geom::Geom;
use ui::input::{KeyAction, EventStatus};


pub trait Widget: Sized {
    // All methods operate on `&mut self` so that the widget can cache values internally.

    /// Compute the total size of this widget, including its children.
    fn size(&mut self) -> V2;

    /// Walk the children of this widget, computing layout as we go.  This method passes each child
    /// and its bounding box to the visitor.
    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2);

    /// Render the widget itself (not including children).
    fn render(&mut self, geom: &mut Geom, rect: Region<V2>);

    /// Handle a keyboard event.
    ///
    /// The default implementation calls `OnKeyVisitor::dispatch` to dispatch the event to each
    /// child in turn until one reports that the event has been handled.
    fn on_key(&mut self, key: KeyAction) -> EventStatus {
        OnKeyVisitor::dispatch(self, key)
    }

    /// Handle a mouse move event.
    ///
    /// The default implementation calls `OnMouseMoveVisitor::dispatch` to dispatch the event to
    /// the child that the mouse is currently over.
    fn on_mouse_move(&mut self, ctx: &mut Context, rect: Region<V2>) -> EventStatus {
        MouseEventVisitor::dispatch(MouseEvent::Move, self, ctx, rect)
    }
}

pub trait Visitor {
    fn visit<W: Widget>(&mut self, w: &mut W, rect: Region<V2>) {}
}


pub struct WidgetPack<'a, W: 'a, D: Copy> {
    pub state: &'a mut W,
    pub dyn: D,
}

impl<'a, W, D: Copy> WidgetPack<'a, W, D> {
    pub fn new(state: &'a mut W, dyn: D) -> WidgetPack<'a, W, D> {
        WidgetPack {
            state: state,
            dyn: dyn,
        }
    }

    pub fn stateless(_w: W, dyn: D) -> WidgetPack<'a, W, D> {
        assert!(mem::size_of::<W>() == 0);
        WidgetPack {
            state: unsafe { mem::transmute(1 as *mut W) },
            dyn: dyn,
        }
    }

    pub fn borrow<'b>(&'b mut self) -> WidgetPack<'b, W, D> {
        WidgetPack {
            state: self.state,
            dyn: self.dyn,
        }
    }
}


struct NullVisitor;

impl Visitor for NullVisitor {
    fn visit<W: Widget>(&mut self, w: &mut W, rect: Region<V2>) {}
}


pub struct OnKeyVisitor {
    key: KeyAction,
    result: EventStatus,
}

impl OnKeyVisitor {
    pub fn new(key: KeyAction) -> OnKeyVisitor {
        OnKeyVisitor {
            key: key,
            result: EventStatus::Unhandled,
        }
    }

    pub fn dispatch<W: ?Sized + Widget>(w: &mut W, key: KeyAction) -> EventStatus {
        let mut v = OnKeyVisitor::new(key);
        w.walk_layout(&mut v, scalar(0));
        v.result
    }
}

impl Visitor for OnKeyVisitor {
    fn visit<W: Widget>(&mut self, w: &mut W, rect: Region<V2>) {
        if self.result.is_handled() {
            return;
        }

        self.result = w.on_key(self.key);
    }
}


pub enum MouseEvent {
    Move,
    Down,
    Up,
}

pub struct MouseEventVisitor<'a> {
    kind: MouseEvent,
    ctx: &'a mut Context,
    rect: Region<V2>,

    result: EventStatus,
}

impl<'a> MouseEventVisitor<'a> {
    pub fn new(kind: MouseEvent,
               ctx: &'a mut Context,
               rect: Region<V2>) -> MouseEventVisitor<'a> {
        MouseEventVisitor {
            kind: kind,
            ctx: ctx,
            rect: rect,
            result: EventStatus::Unhandled,
        }
    }

    pub fn dispatch<W: ?Sized + Widget>(kind: MouseEvent,
                                        w: &mut W,
                                        ctx: &mut Context,
                                        rect: Region<V2>) -> EventStatus {
        let mut v = MouseEventVisitor::new(kind, ctx, rect);
        w.walk_layout(&mut v, rect.min);
        v.result
    }
}

impl<'a> Visitor for MouseEventVisitor<'a> {
    fn visit<W: Widget>(&mut self, w: &mut W, rect: Region<V2>) {
        if self.result.is_handled() {
            return;
        }

        if !rect.contains(self.ctx.mouse_pos) {
            return;
        }

        self.result =
            match self.kind {
                MouseEvent::Move => w.on_mouse_move(self.ctx, rect),
                MouseEvent::Down => unimplemented!(),
                MouseEvent::Up => unimplemented!(),
            };
    }
}
