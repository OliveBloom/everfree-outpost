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


pub enum DialogState {
    None,
    Inventory(InventoryGridState),
}

impl DialogState {
    pub fn new() -> DialogState {
        DialogState::None
    }
}


pub struct State {
    pub hotbar: HotbarState,
    pub dialog: DialogState,
}

impl State {
    pub fn new() -> State {
        State {
            hotbar: HotbarState::new(),
            dialog: DialogState::new(),
        }
    }
}
