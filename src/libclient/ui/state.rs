use ui::{item, hotbar};


#[derive(Clone, Copy)]
pub struct HotbarSlotState {
    pub item_id: u16,
    pub is_ability: bool,
}

impl HotbarSlotState {
    pub fn new() -> HotbarSlotState {
        HotbarSlotState {
            item_id: 0,
            is_ability: false,
        }
    }
}


pub struct HotbarState {
    pub slots: [HotbarSlotState; 9],
    pub cur_item: i8,
    pub cur_ability: i8,
}

impl HotbarState {
    pub fn new() -> HotbarState {
        HotbarState {
            slots: [HotbarSlotState::new(); 9],
            cur_item: -1,
            cur_ability: -1,
        }
    }
}


#[derive(Clone, Copy)]
pub struct InventoryGridState {
    pub focus: u16,
}

impl InventoryGridState {
    pub fn new() -> InventoryGridState {
        InventoryGridState {
            focus: 0,
        }
    }
}


pub struct State {
    pub hotbar: HotbarState,
}

impl State {
    pub fn new() -> State {
        State {
            hotbar: HotbarState::new(),
        }
    }
}



/*
const NO_QUANTITY: u16 = -1_i16 as u16;

pub struct ItemDyn {
    pub item_id: u16,
    pub quantity: u16,
}

impl ItemDyn {
    pub fn new(item_id: u16, quantity: Option<u16>) -> ItemDyn {
        if let Some(quantity) = quantity {
            assert!(quantity != NO_QUANTITY);
            ItemDyn {
                item_id: item_id,
                quantity: quantity,
            }
        } else {
            ItemDyn {
                item_id: item_id,
                quantity: NO_QUANTITY,
            }
        }
    }
}

impl item::ItemDyn for ItemDyn {
    fn item_id(&self) -> u16 {
        self.item_id
    }

    fn quantity(&self) -> Option<u16> {
        if self.quantity == NO_QUANTITY {
            None
        } else {
            Some(self.quantity)
        }
    }
}


#[derive(Clone, Copy)]
pub struct HotbarSlot {
    pub item_id: u16,
    pub quantity: u16,
    pub is_ability: bool,
}

impl HotbarSlot {
    pub fn new(item_id: u16, quantity: u16, is_ability: bool) -> HotbarSlot {
        HotbarSlot {
            item_id: item_id,
            quantity: quantity,
            is_ability: is_ability,
        }
    }
}

impl<'a> item::ItemDyn for &'a HotbarSlot {
    fn item_id(&self) -> u16 {
        self.item_id
    }

    fn quantity(&self) -> Option<u16> {
        if self.is_ability || self.item_id == 0 {
            None
        } else {
            Some(self.quantity)
        }
    }
}

impl<'a> hotbar::SlotDyn for (&'a HotbarSlot, u8) {
    type ItemDyn = &'a HotbarSlot;

    fn item(&self) -> &'a HotbarSlot {
        &self.0
    }

    fn color(&self) -> u8 {
        self.1
    }
}


#[derive(Clone, Copy)]
pub struct Hotbar {
    pub slots: [HotbarSlot; 9],
    pub active_item: i8,
    pub active_ability: i8,
}

impl Hotbar {
    pub fn new() -> Hotbar {
        Hotbar {
            slots: [HotbarSlot::new(0, 0, false); 9],
            active_item: -1,
            active_ability: -1,
        }
    }
}

impl<'a> hotbar::HotbarDyn for &'a Hotbar {
    type SlotDyn = (&'a HotbarSlot, u8);

    fn slot(&self, i: usize) -> (&'a HotbarSlot, u8) {
        let color =
            if i as i8 == self.active_item { 1 }
            else if i as i8 == self.active_ability { 2 }
            else { 0 };
        (&self.slots[i], color)
    }
}


#[derive(Clone, Copy)]
pub struct State {
    pub hotbar: Hotbar,
}

impl State {
    pub fn new() -> State {
        State {
            hotbar: Hotbar::new(),
        }
    }
}
*/
