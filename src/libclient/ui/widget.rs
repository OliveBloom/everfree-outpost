use std::prelude::v1::*;
use std::mem;

use physics::v3::{V2, scalar, Region};

use ui::{UI, Context, DragData};
use ui::geom::{Geom, Special};
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
    /// The default implementation calls `MouseEventVisitor::dispatch` to dispatch the event to the
    /// child that the mouse is currently over.
    fn on_mouse_move(&mut self, ctx: &mut Context, rect: Region<V2>) -> EventStatus {
        MouseEventVisitor::dispatch(MouseEvent::Move, self, ctx, rect)
    }

    /// Handle a mouse down event.
    ///
    /// The default implementation calls `MouseEventVisitor::dispatch` to dispatch the event to the
    /// child that the mouse is currently over.
    fn on_mouse_down(&mut self, ctx: &mut Context, rect: Region<V2>) -> EventStatus {
        MouseEventVisitor::dispatch(MouseEvent::Down, self, ctx, rect)
    }

    /// Handle a mouse up event.
    ///
    /// The default implementation calls `MouseEventVisitor::dispatch` to dispatch the event to the
    /// child that the mouse is currently over.
    fn on_mouse_up(&mut self, ctx: &mut Context, rect: Region<V2>) -> EventStatus {
        MouseEventVisitor::dispatch(MouseEvent::Up, self, ctx, rect)
    }

    /// Handle a drop event.
    ///
    /// The default implementation calls `MouseEventVisitor::dispatch` to dispatch the event to the
    /// child that the mouse is currently over.
    fn on_drop(&mut self, ctx: &mut Context, rect: Region<V2>, data: &DragData) -> EventStatus {
        MouseEventVisitor::dispatch(MouseEvent::Drop(data), self, ctx, rect)
    }

    /// Check if it's legal to drop the currently-dragged data here.
    ///
    /// The default implementation calls `DragVisitor::dispatch` to dispatch the event to the child
    /// that the mouse is currently over.
    fn check_drop(&mut self, ctx: &Context, rect: Region<V2>, data: &DragData) -> bool {
        DropCheckVisitor::dispatch(self, ctx, rect, data)
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


pub enum MouseEvent<'a> {
    Move,
    Down,
    Up,
    Drop(&'a DragData),
}

pub struct MouseEventVisitor<'a, 'b> {
    kind: MouseEvent<'a>,
    ctx: &'b mut Context,
    rect: Region<V2>,

    result: EventStatus,
}

impl<'a, 'b> MouseEventVisitor<'a, 'b> {
    pub fn new(kind: MouseEvent<'a>,
               ctx: &'b mut Context,
               rect: Region<V2>) -> MouseEventVisitor<'a, 'b> {
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

impl<'a, 'b> Visitor for MouseEventVisitor<'a, 'b> {
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
                MouseEvent::Down => w.on_mouse_down(self.ctx, rect),
                MouseEvent::Up => w.on_mouse_up(self.ctx, rect),
                MouseEvent::Drop(data) => w.on_drop(self.ctx, rect, data),
            };
    }
}


pub struct DropCheckVisitor<'a, 'b> {
    data: &'a DragData,
    ctx: &'b Context,
    rect: Region<V2>,

    result: bool,
}

impl<'a, 'b> DropCheckVisitor<'a, 'b> {
    pub fn new(ctx: &'b Context,
               rect: Region<V2>,
               data: &'a DragData) -> DropCheckVisitor<'a, 'b> {
        DropCheckVisitor {
            data: data,
            ctx: ctx,
            rect: rect,
            result: false,
        }
    }

    pub fn dispatch<W: ?Sized + Widget>(w: &mut W,
                                        ctx: &Context,
                                        rect: Region<V2>,
                                        data: &DragData) -> bool {
        let mut v = DropCheckVisitor::new(ctx, rect, data);
        w.walk_layout(&mut v, rect.min);
        v.result
    }
}

impl<'a, 'b> Visitor for DropCheckVisitor<'a, 'b> {
    fn visit<W: Widget>(&mut self, w: &mut W, rect: Region<V2>) {
        if self.result {
            return;
        }

        if !rect.contains(self.ctx.mouse_pos) {
            return;
        }

        self.result = w.check_drop(self.ctx, rect, self.data);
    }
}
