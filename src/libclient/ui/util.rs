use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region};

use ui::widget::{Widget, Visitor};


pub struct RectVisitor<F: FnMut(Region<V2>)> {
    f: F,
}

impl<F: FnMut(Region<V2>)> RectVisitor<F> {
    pub fn new(f: F) -> RectVisitor<F> {
        RectVisitor {
            f: f,
        }
    }

    pub fn dispatch<W: ?Sized + Widget>(w: &mut W, f: F) {
        let mut v = RectVisitor::new(f);
        w.walk_layout(&mut v, scalar(0));
    }
}

impl<F: FnMut(Region<V2>)> Visitor for RectVisitor<F> {
    fn visit<W: Widget>(&mut self, w: &mut W, rect: Region<V2>) {
        (self.f)(rect);
    }
}


pub fn size_from_children<W: ?Sized+Widget>(w: &mut W) -> V2 {
    let mut rect = None;
    {
        let mut v = RectVisitor::new(|r| {
            if let Some(r0) = rect {
                rect = Some(r.join(r0));
            } else {
                rect = Some(r);
            }
        });
        w.walk_layout(&mut v, scalar(0));
    }

    // If there's padding on the top/left side, apply the same padding on the bottom/right.
    if let Some(r) = rect {
        r.max + r.min
    } else {
        scalar(0)
    }
}

