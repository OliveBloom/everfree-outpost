use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region};

use inventory::Inventories;

use self::widget::{Widget, Visitor};


//mod context;
pub use client_ui_atlas as atlas;
pub mod geom;
//pub mod state;
mod dyn;

mod widget;
mod item;
mod inventory;
mod hotbar;
mod root;
mod dialog;

//pub use self::context::Context;
//pub use self::context::Vertex;
//pub use self::state::State;

pub struct UI {
    //context: Context,
    pub root: root::Root,
}

impl UI {
    pub fn new() -> UI {
        UI {
            //context: Context::new(),
            root: root::Root::new(),
        }
    }

    pub fn generate_geom(&mut self, invs: &Inventories) -> Vec<geom::Vertex> {
        let mut geom = geom::Geom::new();

        let dyn = dyn::RootDyn {
            screen_size: V2::new(799, 379),
            inventories: invs,
        };
        let mut root = widget::WidgetPack::new(&mut self.root, &dyn);
        let root_rect = Region::sized(root.size());
        RenderVisitor::new(&mut geom).visit(&mut root, root_rect);

        geom.unwrap()
    }
}

struct RenderVisitor<'a> {
    geom: &'a mut geom::Geom,
}

impl<'a> RenderVisitor<'a> {
    fn new(geom: &'a mut geom::Geom) -> RenderVisitor<'a> {
        RenderVisitor {
            geom: geom,
        }
    }
}

impl<'a> Visitor for RenderVisitor<'a> {
    fn visit<W: Widget>(&mut self, w: &mut W, rect: Region<V2>) {
        w.render(self.geom, rect);
        w.walk_layout(self, rect.min);
    }
}
