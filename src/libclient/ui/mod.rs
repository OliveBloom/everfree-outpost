use std::prelude::v1::*;

use physics::v3::{V2, scalar, Region};

use inventory::Inventories;

use self::widget::{Widget, Visitor};


//mod context;
pub use client_ui_atlas as atlas;
pub mod geom;
pub mod input;

mod widget;
mod item;
mod inventory;
mod hotbar;
mod dialog;

pub mod dialogs;    // TODO: make private
mod root;



pub struct UI {
    context: Context,
    pub root: root::Root,
}

impl UI {
    pub fn new() -> UI {
        UI {
            context: Context::new(),
            root: root::Root::new(),
        }
    }

    pub fn generate_geom(&mut self, invs: &Inventories) -> Vec<geom::Vertex> {
        let mut geom = geom::Geom::new();

        let dyn = root::RootDyn {
            screen_size: V2::new(799, 379),
            inventories: invs,
        };
        let mut root = widget::WidgetPack::new(&mut self.root, dyn);
        let root_rect = Region::sized(root.size());
        RenderVisitor::new(&mut geom).visit(&mut root, root_rect);

        geom.unwrap()
    }

    pub fn handle_key(&mut self,
                      key: input::KeyAction,
                      dyn: Dyn) -> input::EventStatus {
        let mut root = widget::WidgetPack::new(&mut self.root, dyn.root);
        root.on_key(key)
    }

    pub fn handle_mouse_move(&mut self,
                             pos: V2,
                             dyn: Dyn) -> input::EventStatus {
        let mut root = widget::WidgetPack::new(&mut self.root, dyn.root);
        let rect = Region::sized(root.size());

        self.context.mouse_pos = pos;
        root.on_mouse_move(&mut self.context, rect)
    }

    pub fn handle_mouse_down(&mut self,
                             pos: V2,
                             dyn: Dyn) -> input::EventStatus {
        let mut root = widget::WidgetPack::new(&mut self.root, dyn.root);
        let rect = Region::sized(root.size());

        self.context.mouse_pos = pos;
        self.context.mouse_down = true;
        self.context.mouse_down_pos = pos;
        root.on_mouse_down(&mut self.context, rect)
    }

    pub fn handle_mouse_up(&mut self,
                           pos: V2,
                           dyn: Dyn) -> input::EventStatus {
        let mut root = widget::WidgetPack::new(&mut self.root, dyn.root);
        let rect = Region::sized(root.size());

        self.context.mouse_pos = pos;
        self.context.mouse_down = false;
        root.on_mouse_up(&mut self.context, rect)
    }
}


pub struct Dyn<'a> {
    root: root::RootDyn<'a>,
}

impl<'a> Dyn<'a> {
    pub fn new(size: (u16, u16),
               inventories: &'a Inventories) -> Dyn<'a> {
        Dyn {
            root: root::RootDyn {
                screen_size: V2::new(size.0 as i32,
                                     size.1 as i32),
                inventories: inventories,
            },
        }
    }
}


pub struct Context {
    mouse_pos: V2,
    mouse_down: bool,
    mouse_down_pos: V2,
}

impl Context {
    fn new() -> Context {
        Context {
            mouse_pos: scalar(-1),
            mouse_down: false,
            mouse_down_pos: scalar(-1),
        }
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
