use types::*;

use data::StructureTemplate;


#[allow(unused_variables)]
pub trait Hooks {
    fn on_inventory_update(&mut self,
                           iid: InventoryId,
                           slot_idx: u8) {}
}

pub struct NoHooks;
#[allow(unused_variables)]
impl Hooks for NoHooks {
}
