use std::prelude::v1::*;
use std::mem;

use physics::v3::{V2, Region};

use ui::geom::Geom;

pub trait Widget {
    // All methods operate on `&mut self` so that the widget can cache values internally.

    /// Compute the total size of this widget, including its children.
    fn size(&mut self) -> V2;

    /// Walk the children of this widget, computing layout as we go.  This method passes each child
    /// and its bounding box to the visitor.
    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2);

    /// Render the widget itself (not including children).
    fn render(&mut self, geom: &mut Geom, rect: Region<V2>);
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
