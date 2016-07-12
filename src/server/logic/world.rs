use std::iter;
use std::mem;
use libphysics::{CHUNK_SIZE, TILE_SIZE};

use types::*;
use util::SmallSet;
use util::StrResult;

use chunks;
use components;
use data::StructureTemplate;
use engine::Engine;
use engine::glue::*;
use engine::split::Open;
use engine::split2::Coded;
use logic;
use messages::{ClientResponse, SyncKind};
use world::{self, World, Structure};
use world::Motion;
use world::bundle::{Importer, Exporter, AnyId};
use world::bundle::{import, export};
use world::bundle::types as b;
use world::fragment::Fragment as World_Fragment;
use world::fragment::DummyFragment;
use world::object::*;
use vision;


macro_rules! impl_world_Hooks {
    ($WorldHooks:ident, $as_vision_fragment:ident) => {

impl<'a, 'd> world::Hooks for $WorldHooks<'a, 'd> {
    fn on_inventory_update(&mut self,
                           iid: InventoryId,
                           slot_idx: u8) {
        logic::inventory::on_update(unsafe { mem::transmute_copy(self) },
                                    iid, slot_idx);
    }
}

// End of macro_rules
    };
}


impl_world_Hooks!(WorldHooks, as_vision_fragment);
impl_world_Hooks!(HiddenWorldHooks, as_hidden_vision_fragment);


pub fn structure_area(s: ObjectRef<Structure>) -> SmallSet<V2> {
    let mut area = SmallSet::new();
    for p in s.bounds().reduce().div_round_signed(CHUNK_SIZE).points() {
        area.insert(p);
    }

    area
}


// TODO: move this to logic::entity
fn teleport_entity_internal(wf: WorldFragment,
                            eid: EntityId,
                            pid: Option<PlaneId>,
                            stable_pid: Option<Stable<PlaneId>>,
                            pos: V3) -> StrResult<()> {
    // FIXME ugly transmute
    let eng: &mut Engine = unsafe { mem::transmute(wf) };

    let (old_pos, old_plane, cid) = {
        let e = unwrap!(eng.world.get_entity(eid));
        (e.pos(eng.now), e.plane_id(), e.pawn_owner().map(|c| c.id()))
    };

    let new_plane =
        if let Some(stable_pid) = stable_pid {
            // Load the plane, if it's not already.
            logic::chunks::get_plane_id(eng, stable_pid)
        } else if let Some(pid) = pid {
            unwrap!(eng.world.get_plane(pid));
            pid
        } else {
            old_plane
        };
    let new_pos = pos;

    let old_cpos = old_pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));
    let new_cpos = new_pos.reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));

    if let Some(cid) = cid {
        // Check if we need to send a desync message.
        // Teleporting to another point within the current chunk will not cause a view
        // update to be scheduled, so there will never be a resync message.  That's why we set
        // the limit to CHUNK_SIZE * TILE_SIZE: traveling that distance along either the X or Y
        // axis will definitely move the entity into a different chunk.
        if new_plane != old_plane || new_cpos != old_cpos {
            eng.messages.send_client(cid, ClientResponse::SyncStatus(SyncKind::Loading));
        }
    }

    {
        let mut wf = DummyFragment::new(&mut eng.world);
        let mut e = wf.entity_mut(eid);
        try!(e.set_plane_id(new_plane));
        e.set_motion(Motion::stationary(new_pos, eng.now));

        let e = e.borrow();
        let msg_gone = logic::vision::entity_gone_message(e);
        let msg_appear = logic::vision::entity_appear_message(e);
        let msg_motion = logic::vision::entity_motion_message_adjusted(e, eng.now);
        let messages = &mut eng.messages;
        if new_plane != old_plane || new_cpos != old_cpos {
            eng.vision.entity_add(eid, new_plane, new_cpos, |cid| {
                messages.send_client(cid, msg_appear.clone());
            });
            eng.vision.entity_remove(eid, old_plane, old_cpos, |cid| {
                messages.send_client(cid, msg_gone.clone());
            });
        }
        eng.vision.entity_update(eid, |cid| {
            messages.send_client(cid, msg_motion.clone());
        });
    }

    if let Some(cid) = cid {
        logic::client::update_view(eng, cid, old_plane, old_cpos, new_plane, new_cpos);
    }

    Ok(())
}

pub fn teleport_entity(wf: WorldFragment,
                       eid: EntityId,
                       pos: V3) -> StrResult<()> {
    teleport_entity_internal(wf, eid, None, None, pos)
}

pub fn teleport_entity_plane(wf: WorldFragment,
                             eid: EntityId,
                             pid: PlaneId,
                             pos: V3) -> StrResult<()> {
    teleport_entity_internal(wf, eid, Some(pid), None, pos)
}

pub fn teleport_entity_stable_plane(wf: WorldFragment,
                                    eid: EntityId,
                                    stable_pid: Stable<PlaneId>,
                                    pos: V3) -> StrResult<()> {
    teleport_entity_internal(wf, eid, None, Some(stable_pid), pos)
}


engine_part2!(pub EngineLifecycle(
        world, physics, cache, vision, messages, dialogs,
        Components));


struct ImportVisitor<'a, 'd: 'a>(&'a mut EngineLifecycle<'d>);

impl<'a, 'd> import::Visitor for ImportVisitor<'a, 'd> {
    fn visit_client(&mut self, id: ClientId, b: &b::Client) {
    }

    fn visit_entity(&mut self, id: EntityId, b: &b::Entity) {
        logic::entity::on_create(self.0.refine(), id);
        components::import_entity(self.0.refine(), id, b);
    }

    fn visit_inventory(&mut self, id: InventoryId, b: &b::Inventory) {
    }

    fn visit_plane(&mut self, id: PlaneId, b: &b::Plane) {
    }

    fn visit_terrain_chunk(&mut self, id: TerrainChunkId, b: &b::TerrainChunk) {
        logic::terrain_chunk::on_create(self.0.refine(), id);
    }

    fn visit_structure(&mut self, id: StructureId, b: &b::Structure) {
        logic::structure::on_create(self.0.refine(), id);
        logic::structure::on_import(self.0.refine(), id);
    }
}

/// Hook to be called after importing some new game objects.
pub fn on_import(eng: &mut EngineLifecycle, importer: &Importer, bundle: &b::Bundle) {
    importer.visit_imports(bundle, &mut ImportVisitor(eng));
}


// TODO: separate "export" logic from "destroy" logic
struct ExportVisitor<'a, 'd: 'a>(&'a mut EngineLifecycle<'d>);

impl<'a, 'd> export::Visitor for ExportVisitor<'a, 'd> {
    fn visit_client(&mut self, id: ClientId, b: &mut b::Client) {
    }

    fn visit_entity(&mut self, id: EntityId, b: &mut b::Entity) {
        components::export_entity(self.0.refine(), id, b);

        logic::entity::on_destroy(self.0.refine(), id);
        components::cleanup_entity(self.0.refine(), id);
    }

    fn visit_inventory(&mut self, id: InventoryId, b: &mut b::Inventory) {
    }

    fn visit_plane(&mut self, id: PlaneId, b: &mut b::Plane) {
    }

    fn visit_terrain_chunk(&mut self, id: TerrainChunkId, b: &mut b::TerrainChunk) {
        logic::terrain_chunk::on_destroy(self.0.refine(), id);
    }

    fn visit_structure(&mut self, id: StructureId, b: &mut b::Structure) {
        logic::structure::on_destroy(self.0.refine(), id);
    }
}

/// Hook to be called before deleting some exported game objects..
pub fn on_export(eng: &mut EngineLifecycle, exporter: &mut Exporter) {
    exporter.visit_exports(&mut ExportVisitor(eng));
}
