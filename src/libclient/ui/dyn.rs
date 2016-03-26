//! `Dyn` impls for various types of widgets.  These combine widget-specific state (from
//! `ui::state`) with external data (such as `client.inventories`).

use inventory::Inventory;
use ui::{hotbar, item};
use ui::state::*;

#[derive(Clone, Copy)]
// TODO: remove pub
pub struct HotbarDyn<'a> {
    pub state: &'a HotbarState,
    pub inv: Option<&'a Inventory>,
}

impl<'a> hotbar::HotbarDyn for HotbarDyn<'a> {
    type SlotDyn = HotbarSlotDyn;

    fn slot(self, i: usize) -> HotbarSlotDyn {
        let slot_state = &self.state.slots[i];
        let quantity =
            if slot_state.is_ability || slot_state.item_id == 0 {
                None
            } else if let Some(inv) = self.inv {
                Some(inv.count(slot_state.item_id))
            } else {
                Some(0)
            };
        let color =
            if i as i8 == self.state.cur_item { 1 }
            else if i as i8 == self.state.cur_ability { 2 }
            else { 0 };

        HotbarSlotDyn {
            item_id: slot_state.item_id,
            quantity: quantity,
            color: color,
        }
    }
}


#[derive(Clone, Copy)]
struct HotbarSlotDyn {
    item_id: u16,
    quantity: Option<u16>,
    color: u8,
}

impl hotbar::SlotDyn for HotbarSlotDyn {
    type ItemDyn = Self;

    fn item(self) -> HotbarSlotDyn { self }
    fn color(self) -> u8 { self.color }
}

impl item::ItemDyn for HotbarSlotDyn {
    fn item_id(self) -> u16 { self.item_id }
    fn quantity(self) -> Option<u16> { self.quantity }
}
