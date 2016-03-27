//! `Dyn` impls for various types of widgets.  These combine widget-specific state (from
//! `ui::state`) with external data (such as `client.inventories`).

use inventory::Inventory;
use ui::{dialog, hotbar, inventory, item};
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
pub struct HotbarSlotDyn {
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


#[derive(Clone, Copy)]
pub struct InventoryGridDyn<'a> {
    pub state: InventoryGridState,
    pub inv: Option<&'a Inventory>,
}

impl<'a> inventory::GridDyn for InventoryGridDyn<'a> {
    type SlotDyn = InventorySlotDyn;

    fn len(self) -> usize {
        if let Some(inv) = self.inv {
            inv.len()
        } else {
            0
        }
    }

    fn slot(self, i: usize) -> InventorySlotDyn {
        debug_assert!(i < self.len());
        debug_assert!(self.inv.is_some());  // implied by self.len() > 0
        let item = self.inv.unwrap().items[i];

        InventorySlotDyn {
            item_id: item.id,
            quantity: item.quantity,
            state:
                if i == self.state.focus as usize { inventory::SlotState::Active }
                else { inventory::SlotState::Inactive },
        }
    }
}

impl<'a> dialog::DialogDyn for InventoryGridDyn<'a> {
    type InvDyn = Self;

    fn inv_grid(self) -> Self { self }
    fn with_title<R, F: FnOnce(&str) -> R>(self, f: F) -> R {
        f("Inventory")
    }
}


#[derive(Clone, Copy)]
pub struct InventorySlotDyn {
    item_id: u16,
    quantity: u8,
    state: inventory::SlotState,
}

impl inventory::SlotDyn for InventorySlotDyn {
    type ItemDyn = Self;

    fn item(self) -> InventorySlotDyn { self }
    fn state(self) -> inventory::SlotState { self.state }
}

impl item::ItemDyn for InventorySlotDyn {
    fn item_id(self) -> u16 { self.item_id }

    fn quantity(self) -> Option<u16> {
        if self.quantity > 0 {
            Some(self.quantity as u16)
        } else {
            None
        }
    }
}
