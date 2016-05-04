use std::prelude::v1::*;
use physics::v3::{V2, scalar, Region};

use inventory::{Inventories, InventoryId};
use ui::dialog;
use ui::geom::Geom;
use ui::input::{KeyAction, EventStatus};
use ui::widget::*;


mod inventory;

pub use self::inventory::{Inventory, InventoryDyn};
pub use self::inventory::{Container, ContainerDyn};


pub enum AnyDialog {
    None,
    Inventory(Inventory),
    Ability(Inventory),
    Container(Container),
}

impl AnyDialog {
    pub fn none() -> AnyDialog {
        AnyDialog::None
    }

    pub fn inventory() -> AnyDialog {
        AnyDialog::Inventory(Inventory::new(false))
    }

    pub fn ability() -> AnyDialog {
        AnyDialog::Ability(Inventory::new(true))
    }

    pub fn container(inv_id1: InventoryId,
                     inv_id2: InventoryId) -> AnyDialog {
        AnyDialog::Container(Container::new(inv_id1, inv_id2))
    }
}

impl dialog::Inner for AnyDialog {
    fn get_title(&self) -> &str {
        match *self {
            AnyDialog::None => "",
            AnyDialog::Inventory(_) => "Inventory",
            AnyDialog::Ability(_) => "Abilities",
            AnyDialog::Container(_) => "Container",
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

            AnyDialog::Ability(ref mut state) => {
                let dyn = InventoryDyn::new(self.dyn.inventories.ability_inventory());
                let mut child = WidgetPack::new(state, dyn);
                let rect = Region::sized(child.size()) + pos;
                v.visit(&mut child, rect);
            },

            AnyDialog::Container(ref mut state) => {
                let dyn = ContainerDyn::new(self.dyn.inventories);
                let mut child = WidgetPack::new(state, dyn);
                let rect = Region::sized(child.size()) + pos;
                v.visit(&mut child, rect);
            },
        }
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }

    fn on_key(&mut self, key: KeyAction) -> EventStatus {
        let status = OnKeyVisitor::dispatch(self, key);
        if status.is_handled() {
            return status;
        }

        if key == KeyAction::Cancel {
            *self.state = AnyDialog::None;
            return EventStatus::Handled;
        }

        EventStatus::Unhandled
    }
}


struct SizeVisitor<'a>(&'a mut V2);

impl<'a> Visitor for SizeVisitor<'a> {
    fn visit<W: Widget>(&mut self, w: &mut W, rect: Region<V2>) {
        *self.0 = w.size();
    }
}
