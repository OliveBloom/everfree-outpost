use std::prelude::v1::*;

use physics::v3::{V2, Region};

use ui::geom::Geom;

pub trait Widget: Copy {
    /// Compute the total size of this widget, including its children.
    fn size(self) -> V2;

    /// Walk the children of this widget, computing layout as we go.  This method passes each child
    /// and its bounding box to the visitor.
    fn walk_layout<V: Visitor>(self, v: &mut V, pos: V2);

    /// Render the widget itself (not including children).
    fn render(self, geom: &mut Geom, rect: Region<V2>);
}

pub trait Visitor {
    fn visit<W: Widget>(&mut self, w: W, rect: Region<V2>) {}
}

#[derive(Clone, Copy)]
pub struct WidgetPack<W, D> {
    pub w: W,
    pub dyn: D,
}

impl<W, D> WidgetPack<W, D> {
    pub fn new(w: W, dyn: D) -> WidgetPack<W, D> {
        WidgetPack {
            w: w,
            dyn: dyn,
        }
    }
}


struct NullVisitor;

impl Visitor for NullVisitor {
    fn visit<W: Widget>(&mut self, w: W, rect: Region<V2>) {}
}
