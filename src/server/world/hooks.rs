use types::*;

use data::StructureTemplate;


#[allow(unused_variables)]
pub trait Hooks {
    fn on_inventory_update(&mut self,
                           iid: InventoryId,
                           slot_idx: u8) {}

    fn check_structure_placement(&self,
                                 template: &StructureTemplate,
                                 plane_id: PlaneId,
                                 pos: V3) -> bool;

    fn check_structure_replacement(&self,
                                   sid: StructureId,
                                   new_template: &StructureTemplate,
                                   plane_id: PlaneId,
                                   pos: V3) -> bool;
}

pub struct NoHooks;
#[allow(unused_variables)]
impl Hooks for NoHooks {
    fn check_structure_placement(&self,
                                 template: &StructureTemplate,
                                 plane_id: PlaneId,
                                 pos: V3) -> bool { false }

    fn check_structure_replacement(&self,
                                   sid: StructureId,
                                   new_template: &StructureTemplate,
                                   plane_id: PlaneId,
                                   pos: V3) -> bool { false }
}
