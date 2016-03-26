use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region};

use self::widget::{Widget, Visitor};


//mod context;
pub use client_ui_atlas as atlas;
pub mod geom;
//mod state;

mod widget;
mod item;
mod hotbar;

//pub use self::context::Context;
//pub use self::context::Vertex;
//pub use self::state::State;

pub struct UI;
    //context: Context,
    //state: state::State,
    //root: hotbar::Hotbar,

impl UI {
    pub fn new() -> UI {
        UI
            //context: Context::new(),
            //state: state::State::new(),
            //root: hotbar::Hotbar::new(),
        //}
    }

    /*
    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
    */

    pub fn generate_geom(&mut self) -> Vec<geom::Vertex> {
        let mut geom = geom::Geom::new();

        let root = widget::WidgetPack::new(hotbar::Hotbar, ());
        let root_rect = Region::sized(root.size());
        RenderVisitor::new(&mut geom).visit(root, root_rect);

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
    fn visit<W: Widget>(&mut self, w: W, rect: Region<V2>) {
        w.render(self.geom, rect);
        w.walk_layout(self, rect.min);
    }
}

impl hotbar::HotbarDyn for () {
    type SlotDyn = ();
    fn slot(self, i: usize) -> () {}
}

impl hotbar::SlotDyn for () {
    type ItemDyn = ();
    fn item(self) -> () {}
    fn color(self) -> u8 { 0 }
}

impl item::ItemDyn for () {
    fn item_id(self) -> u16 { 1 }
    fn quantity(self) -> Option<u16> { None }
}
