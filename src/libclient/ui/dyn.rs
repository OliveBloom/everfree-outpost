//! `Dyn` impls for various types of widgets.  These combine widget-specific state (from
//! `ui::state`) with external data (such as `client.inventories`).

use physics::v3::{V2, scalar, Region};

use inventory::{Inventory, Inventories, Item};
use ui::{dialog, hotbar, inventory, item, root};
use ui::widget::*;


pub struct RootDyn<'a> {
    pub screen_size: V2,
    pub inventories: &'a Inventories,
}

impl<'a, 'b: 'a> root::RootDyn for &'a RootDyn<'b> {
    fn screen_size(self) -> V2 { self.screen_size }

    /*
    type DialogDyn = Self;
    fn dialog(self) -> Self { self }
    */

    type HotbarDyn = &'b Inventories;
    fn hotbar(self) -> &'b Inventories { self.inventories }
}


pub struct InventoryGridDyn<'a> {
    pub size: V2,
    pub inv: Option<&'a Inventory>,
}

impl<'a, 'b: 'a> inventory::GridDyn for &'a InventoryGridDyn<'b> {
    type ItemDyn = Item;

    fn grid_size(self) -> V2 {
        self.size
    }

    fn len(self) -> usize {
        if let Some(inv) = self.inv {
            inv.len()
        } else {
            0
        }
    }

    fn item(self, i: usize) -> Item {
        // This function is only called when self.inv is Some.
        self.inv.unwrap().items[i]
    }
}

impl item::ItemDyn for Item {
    fn item_id(self) -> u16 { self.id }
    fn quantity(self) -> Option<u16> {
        if self.id == 0 {
            None
        } else {
            Some(self.quantity as u16)
        }
    }
}


impl<'a> hotbar::HotbarDyn for &'a Inventories {
    fn item_count(self, item_id: u16, ability: bool) -> u16 {
        if !ability {
            if let Some(inv) = self.main_inventory() {
                inv.count(item_id)
            } else {
                0
            }
        } else {
            if let Some(inv) = self.ability_inventory() {
                inv.count(item_id)
            } else {
                0
            }
        }
    }
}


pub enum DialogInner {
    None,
    Inventory(inventory::Grid),
}

/*
pub trait DialogInnerDyn: Copy {
    type InventoryDyn: inventory::GridDyn;
    fn inventory(self) -> Self::InventoryDyn;
}
*/

impl dialog::Inner for DialogInner {
    fn get_title(&self) -> &str {
        match *self {
            DialogInner::None => "",
            DialogInner::Inventory(_) => "Inventory",
        }
    }

    fn active(&self) -> bool {
        match *self {
            DialogInner::None => false,
            _ => true,
        }
    }
}

/*
impl<'a, 'b> DialogInnerDyn for &'a RootDyn<'b> {
    type InventoryDyn = InventoryGridDyn<'b>;
    fn inventory(self) -> InventoryGridDyn<'b> {
        InventoryGridDyn {
            size: V2::new(6, 5),
            inv: self.inventories.main_inventory(),
        }
    }
}

impl<'a, D: DialogInnerDyn> Widget for WidgetPack<'a, DialogInner, D> {
    fn size(&mut self) -> V2 {
        match *self {
            DialogInner::None => scalar(0),
            DialogInner::Inventory(ref mut inv) => {
                let mut child = WidgetPack::new(inv, self.dyn.inventory());
                child.size()
            },
        }
    }
}
*/
