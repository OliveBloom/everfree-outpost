#[allow(unused_imports)] use std::prelude::v1::*;
use std::mem;
use physics::v3::{V2, scalar, Region};

use data::Data;
use inventory::{Inventories, InventoryId};
use structures::StructureId;
use ui::dialog;
use ui::geom::Geom;
use ui::input::{KeyAction, KeyEvent, EventStatus};
use ui::widget::*;


mod inventory;
mod crafting;

pub use self::inventory::{Inventory, InventoryDyn};
pub use self::inventory::{Container, ContainerDyn};
pub use self::crafting::{Crafting, CraftingDyn};


pub enum AnyDialog {
    None,
    Inventory(Inventory),
    Ability(Inventory),
    Container(Container),
    Crafting(Crafting),
}

impl AnyDialog {
    pub fn none() -> AnyDialog {
        AnyDialog::None
    }

    pub fn inventory() -> AnyDialog {
        AnyDialog::Inventory(Inventory::new())
    }

    pub fn ability() -> AnyDialog {
        AnyDialog::Ability(Inventory::new())
    }

    pub fn container(inv_id1: InventoryId,
                     inv_id2: InventoryId) -> AnyDialog {
        AnyDialog::Container(Container::new(inv_id1, inv_id2))
    }

    pub fn crafting(inv_id: InventoryId,
                    station_id: StructureId,
                    template: u32) -> AnyDialog {
        AnyDialog::Crafting(Crafting::new(inv_id, station_id, template))
    }


    fn on_close(self) -> EventStatus {
        match self {
            AnyDialog::Container(s) => s.on_close(),
            AnyDialog::Crafting(s) => s.on_close(),
            _ => EventStatus::Unhandled,
        }
    }
}

impl dialog::Inner for AnyDialog {
    fn get_title(&self) -> &str {
        match *self {
            AnyDialog::None => "",
            AnyDialog::Inventory(_) => "Inventory",
            AnyDialog::Ability(_) => "Abilities",
            AnyDialog::Container(_) => "Container",
            AnyDialog::Crafting(_) => "Crafting",
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
    data: &'a Data,
}

impl<'a> AnyDialogDyn<'a> {
    pub fn new(inventories: &'a Inventories,
               data: &'a Data) -> AnyDialogDyn<'a> {
        AnyDialogDyn {
            inventories: inventories,
            data: data,
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
                let mut child = WidgetPack::new(state, &dyn);
                let rect = Region::sized(child.size()) + pos;
                v.visit(&mut child, rect);
            },

            AnyDialog::Ability(ref mut state) => {
                let dyn = InventoryDyn::new(self.dyn.inventories.ability_inventory());
                let mut child = WidgetPack::new(state, &dyn);
                let rect = Region::sized(child.size()) + pos;
                v.visit(&mut child, rect);
            },

            AnyDialog::Container(ref mut state) => {
                let dyn = ContainerDyn::new(self.dyn.inventories);
                let mut child = WidgetPack::new(state, &dyn);
                let rect = Region::sized(child.size()) + pos;
                v.visit(&mut child, rect);
            },

            AnyDialog::Crafting(ref mut state) => {
                let dyn = CraftingDyn::new(self.dyn.inventories, self.dyn.data);
                let mut child = WidgetPack::new(state, &dyn);
                let rect = Region::sized(child.size()) + pos;
                v.visit(&mut child, rect);
            },
        }
    }

    fn render(&mut self, _geom: &mut Geom, _rect: Region<V2>) {
    }

    fn on_key(&mut self, key: KeyEvent) -> EventStatus {
        let status = OnKeyVisitor::dispatch(self, key);
        if status.is_handled() {
            return status;
        }

        if key.code == KeyAction::Cancel {
            let old_state = mem::replace(self.state, AnyDialog::None);
            let status = old_state.on_close();
            if status.is_handled() {
                return status;
            } else {
                return EventStatus::Handled;
            }
        }

        EventStatus::Unhandled
    }
}


struct SizeVisitor<'a>(&'a mut V2);

impl<'a> Visitor for SizeVisitor<'a> {
    fn visit<W: Widget>(&mut self, w: &mut W, _rect: Region<V2>) {
        *self.0 = w.size();
    }
}
