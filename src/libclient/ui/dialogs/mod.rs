use std::prelude::v1::*;
use physics::v3::{V2, scalar, Region};

use inventory::Inventories;
use ui::dialog;
use ui::geom::Geom;
use ui::input::KeyAction;
use ui::widget::*;


mod inventory;

pub use self::inventory::{Inventory, InventoryDyn};


pub enum AnyDialog {
    None,
    Inventory(Inventory),
}

impl AnyDialog {
    pub fn none() -> AnyDialog {
        AnyDialog::None
    }

    pub fn inventory() -> AnyDialog {
        AnyDialog::Inventory(Inventory::new())
    }
}

impl dialog::Inner for AnyDialog {
    fn get_title(&self) -> &str {
        match *self {
            AnyDialog::None => "",
            AnyDialog::Inventory(_) => "Inventory",
        }
    }

    fn active(&self) -> bool {
        match *self {
            AnyDialog::None => false,
            _ => true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AnyDialogDyn<'a> {
    inventories: &'a Inventories,
}

impl<'a> AnyDialogDyn<'a> {
    pub fn new(inventories: &'a Inventories) -> AnyDialogDyn<'a> {
        AnyDialogDyn {
            inventories: inventories,
        }
    }
}

impl<'a, 'b> Widget for WidgetPack<'a, AnyDialog, AnyDialogDyn<'b>> {
    fn size(&mut self) -> V2 {
        let mut size = scalar(0);
        self.walk_layout(&mut SizeVisitor(&mut size), scalar(0));
        size
    }

    fn walk_layout<V: Visitor>(&mut self, v: &mut V, pos: V2) {
        match *self.state {
            AnyDialog::None => {},

            AnyDialog::Inventory(ref mut state) => {
                let dyn = InventoryDyn::new(self.dyn.inventories.main_inventory());
                let mut child = WidgetPack::new(state, dyn);
                let rect = Region::sized(child.size()) + pos;
                v.visit(&mut child, rect);
            },
        }
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }

    fn on_key(&mut self, key: KeyAction) -> bool {
        if OnKeyVisitor::dispatch(self, key) {
            return true;
        }

        if key == KeyAction::Cancel {
            *self.state = AnyDialog::None;
            return true;
        }

        false
    }
}


struct SizeVisitor<'a>(&'a mut V2);

impl<'a> Visitor for SizeVisitor<'a> {
    fn visit<W: Widget>(&mut self, w: &mut W, rect: Region<V2>) {
        *self.0 = w.size();
    }
}
