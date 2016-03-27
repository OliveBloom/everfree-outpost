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


impl<'a> hotbar::HotbarDyn for &'a Inventories {
    fn item_count(self, item_id: u16) -> u16 {
        if let Some(inv) = self.main_inventory() {
            inv.count(item_id)
        } else {
            0
        }
    }
}
