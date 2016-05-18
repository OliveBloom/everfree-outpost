use std::iter;
use std::mem;
use libphysics::{CHUNK_SIZE, TILE_SIZE};

use types::*;
use util::SmallSet;
use util::StrResult;

use chunks;
use data::StructureTemplate;
use engine::Engine;
use engine::glue::*;
use engine::split::{Open, EngineRef};
use logic;
use messages::{ClientResponse, SyncKind};
use physics;
use world::{self, World, Entity, Structure};
use world::Motion;
use world::fragment::Fragment as World_Fragment;
use world::fragment::DummyFragment;
use world::object::*;
use vision::{self, vision_region};


macro_rules! impl_world_Hooks {
    ($WorldHooks:ident, $as_vision_fragment:ident) => {

impl<'a, 'd> world::Hooks for $WorldHooks<'a, 'd> {
    // We should never get client callbacks in the HiddenWorldHooks variant.

    // No client lifecycle callbacks because they're handled in the logic::client code.

    fn on_client_change_pawn(&mut self,
                             _cid: ClientId,
                             _old_pawn: Option<EntityId>,
                             _new_pawn: Option<EntityId>) {
        // Should never happen, now that clients are imported from a fully-initialized bundle.
        unreachable!();
    }


    fn on_terrain_chunk_create(&mut self, tcid: TerrainChunkId) {
        let (pid, cpos) = {
            let tc = self.world().terrain_chunk(tcid);
            (tc.plane_id(), tc.chunk_pos())
        };
        vision::Fragment::add_terrain_chunk(&mut self.$as_vision_fragment(), tcid, pid, cpos);

        let Open { world, cache, .. } = (**self).open();
        warn_on_err!(cache.add_chunk(world, pid, cpos));
    }

    fn on_terrain_chunk_destroy(&mut self, tcid: TerrainChunkId, pid: PlaneId, cpos: V2) {
        vision::Fragment::remove_terrain_chunk(&mut self.$as_vision_fragment(), tcid);

        self.cache_mut().remove_chunk(pid, cpos);
    }

    fn on_terrain_chunk_update(&mut self, tcid: TerrainChunkId) {
        // TODO: need a system to avoid resending the entire chunk every time.
        let (pid, bounds) = {
            let tc = self.world().terrain_chunk(tcid);
            (tc.plane_id(), tc.bounds())
        };
        vision::Fragment::update_terrain_chunk(&mut self.$as_vision_fragment(), tcid);

        let Open { world, cache, .. } = (**self).open();
        cache.update_region(world, pid, bounds);
    }


    /*
    fn on_entity_activity_change(&mut self, eid: EntityId) {
        trace!("entity {:?} activity changed", eid);
        let now = self.now();
        // FIXME: need to schedule a physics update right away
    }
    */


    fn on_structure_create(&mut self, sid: StructureId) {
        let (pid, area) = {
            let s = self.world().structure(sid);
            (s.plane_id(), structure_area(s))
        };
        vision::Fragment::add_structure(&mut self.$as_vision_fragment(), sid, pid, area);

        let Open { world, cache, .. } = (**self).open();
        let s = world.structure(sid);
        cache.update_region(world, pid, s.bounds());
    }

    fn on_structure_destroy(&mut self,
                            sid: StructureId,
                            old_pid: PlaneId,
                            old_bounds: Region) {
        vision::Fragment::remove_structure(&mut self.$as_vision_fragment(), sid);

        {
            let Open { world, cache, .. } = (**self).open();
            cache.update_region(world, old_pid, old_bounds);
        }
    }

    fn on_structure_replace(&mut self,
                            sid: StructureId,
                            pid: PlaneId,
                            old_bounds: Region) {
        {
            let Open { world, cache, .. } = (**self).open();
            cache.update_region(world, pid, old_bounds);
        }

        let (pid, area) = {
            let s = self.world().structure(sid);
            (s.plane_id(), structure_area(s))
        };
        vision::Fragment::set_structure_area(&mut self.$as_vision_fragment(), sid, pid, area);
        vision::Fragment::change_structure_template(&mut self.$as_vision_fragment(), sid);

        let Open { world, cache, .. } = (**self).open();
        let s = world.structure(sid);
        cache.update_region(world, pid, old_bounds.join(s.bounds()));
    }

    fn check_structure_placement(&self,
                                 template: &StructureTemplate,
                                 pid: PlaneId,
                                 pos: V3) -> bool {
        let cache = self.cache();
        let chunk_bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));
        check_structure_placement(self.world(), template, pid, pos, |pos| {
            let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE));
            let entry = unwrap_or!(cache.get(pid, cpos), return None);
            let cur_chunk_bounds = chunk_bounds + cpos.extend(0) * scalar(CHUNK_SIZE);
            let mask = entry.layer_mask[cur_chunk_bounds.index(pos)];
            Some(mask)
        })
    }

    fn check_structure_replacement(&self,
                                   sid: StructureId,
                                   new_template: &StructureTemplate,
                                   pid: PlaneId,
                                   pos: V3) -> bool {
        let bounds = Region::new(pos, pos + new_template.size);
        let mask = unwrap_or!(compute_layer_mask_excluding(self.world(), pid, bounds, sid).ok(),
                              return false);
        check_structure_placement(self.world(), new_template, pid, pos, |pos| {
            if !bounds.contains(pos) {
                return None;
            }
            Some(mask[bounds.index(pos)])
        })
    }


    // No lifecycle callbacks for inventories, because Vision doesn't care what inventories exist,
    // only what inventories are actually subscribed to.

    fn on_inventory_update(&mut self,
                           iid: InventoryId,
                           slot_idx: u8) {
        vision::Fragment::update_inventory(&mut self.$as_vision_fragment(), iid, slot_idx);
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


// There are (currently) three layers for structure placement, each with distinct properties.
//
// Layer 0: Floor-type structures.  House floor, road, etc.  These can be placed over terrain
// floors and in empty space.
//
// Layer 1: Solid structures.  House wall, anvil, chest, etc.  These require empty space throughout
// their volume and also a floor at the bottom.
//
// Layer 2: Solid attachments.  Cabinets, bookshelves, etc.  These can be placed like Layer 1
// structures (floor + empty space above), or they can instead be placed over a Layer 1 structure
// with no shape restrictions.  In the case of placement over an existing Layer 1 structure, the
// script doing the placement is responsible for enforcing any additional invariants.

const PLACEMENT_MASK: [u8; 3] = [
    0x1,    // Layer 0 can be placed under existing structures.
    0x6,    // Layer 1 can be placed over Layer 0, but not under Layer 2.
    0x4,    // Layer 2 can be placed over Layer 0 and 1.
];

fn check_structure_placement<F>(world: &World,
                                template: &StructureTemplate,
                                pid: PlaneId,
                                base_pos: V3,
                                mut get_mask: F) -> bool
        where F: FnMut(V3) -> Option<u8> {
    let data = world.data();
    let bounds = Region::new(scalar(0), template.size) + base_pos;

    let p = unwrap_or!(world.get_plane(pid), return false);
    for pos in bounds.points() {
        let cpos = pos.reduce().div_floor(scalar(CHUNK_SIZE));

        let tc = unwrap_or!(p.get_terrain_chunk(cpos), return false);
        let shape = data.block_data.shape(tc.block(tc.bounds().index(pos)));

        let mask = unwrap_or!(get_mask(pos), return false);

        let shape_ok = match template.layer {
            0 => check_shape_0(shape, pos.z == base_pos.z, mask),
            1 => check_shape_1(shape, pos.z == base_pos.z, mask),
            2 => check_shape_2(shape, pos.z == base_pos.z, mask),
            x => {
                info!("unexpected template layer: {}", x);
                false
            },
        };

        if !shape_ok {
            info!("placement failed due to terrain");
            return false;
        }

        if mask & PLACEMENT_MASK[template.layer as usize] != 0 {
            info!("placement failed due to layering");
            return false;
        }
    }

    true
}

fn check_shape_0(shape: Shape, is_bottom: bool, _mask: u8) -> bool {
    if is_bottom {
        shape == Shape::Floor || shape == Shape::Empty
    } else {
        shape == Shape::Empty
    }
}

fn check_shape_1(shape: Shape, is_bottom: bool, mask: u8) -> bool {
    if is_bottom {
        mask & (1 << 0) != 0 || shape == Shape::Floor
    } else {
        mask & (1 << 0) == 0 && shape == Shape::Empty
    }
}

fn check_shape_2(shape: Shape, is_bottom: bool, mask: u8) -> bool {
    if mask & (1 << 1) != 0 {
        true
    } else {
        check_shape_1(shape, is_bottom, mask)
    }
}


fn compute_layer_mask_excluding(w: &World,
                                pid: PlaneId,
                                bounds: Region,
                                exclude_sid: StructureId) -> StrResult<Vec<u8>> {
    let mut result = iter::repeat(0_u8).take(bounds.volume() as usize).collect::<Vec<_>>();

    for cpos in bounds.reduce().div_round_signed(CHUNK_SIZE).points() {
        for s in w.chunk_structures(pid, cpos) {
            if s.id() == exclude_sid {
                continue;
            }

            for p in s.bounds().intersect(bounds).points() {
                let template = s.template();
                result[bounds.index(p)] |= 1 << (template.layer as usize);
            }
        }
    }

    Ok(result)
}


fn teleport_entity_internal(mut wf: WorldFragment,
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
            chunks::Fragment::get_plane_id(&mut eng.as_ref().as_chunks_fragment(), stable_pid)
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
        let msg_motion = logic::vision::entity_motion_message(e);
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
