use types::*;

use data::StructureTemplate;


#[allow(unused_variables)]
pub trait Hooks {
    fn on_client_create(&mut self, cid: ClientId) {}
    fn on_client_destroy(&mut self, cid: ClientId) {}
    fn on_client_change_pawn(&mut self,
                             cid: ClientId,
                             old_pawn: Option<EntityId>,
                             new_pan: Option<EntityId>) {}

    fn on_inventory_create(&mut self, iid: InventoryId) {}
    fn on_inventory_destroy(&mut self, iid: InventoryId) {}
    fn on_inventory_update(&mut self,
                           iid: InventoryId,
                           slot_idx: u8) {}

    fn on_plane_create(&mut self, pid: PlaneId) {}
    fn on_plane_destroy(&mut self, pid: PlaneId) {}

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
