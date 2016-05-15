use std::borrow::ToOwned;

use types::*;

use engine::glue::*;
use messages::{Messages, ClientResponse};
use world::{self, World};
use world::object::*;
use vision::{self, Vision};


impl<'a, 'd> vision::Hooks for VisionHooks<'a, 'd> {
    fn on_terrain_chunk_appear(&mut self,
                               cid: ClientId,
                               tcid: TerrainChunkId) {
        self.on_terrain_chunk_update(cid, tcid);
    }

    fn on_terrain_chunk_update(&mut self,
                               cid: ClientId,
                               tcid: TerrainChunkId) {
        use util::encode_rle16;
        trace!("terrain chunk update: {:?}, {:?}", cid, tcid);
        let tc = unwrap_or!(self.world().get_terrain_chunk(tcid),
            { warn!("no terrain available for {:?}", tcid); return });
        let cpos = tc.chunk_pos();
        let data = encode_rle16(tc.blocks().iter().map(|&x| x));
        self.messages().send_client(cid, ClientResponse::TerrainChunk(cpos, data));
    }


    fn on_entity_appear(&mut self, cid: ClientId, eid: EntityId) {
        trace!("on_entity_appear({:?}, {:?})", cid, eid);
        {
            let entity = self.world().entity(eid);

            let appearance = entity.appearance();
            // TODO: hack.  Should have a separate "entity name" field somewhere.
            let name =
                if let world::EntityAttachment::Client(controller_cid) = entity.attachment() {
                    self.world().client(controller_cid).name().to_owned()
                } else {
                    String::new()
                };

            // FIXME merge some of this with the new-style update code below
            let m = entity.motion();
            let motion_msg = if let Some(end_time) = m.end_time {
                ClientResponse::EntityMotionStartEnd(
                    // FIXME: anim handling
                    eid, m.start_pos, m.start_time, m.velocity, 0, end_time)
            } else {
                ClientResponse::EntityMotionStart(
                    // FIXME: anim handling
                    eid, m.start_pos, m.start_time, m.velocity, 0)
            };

            self.messages().send_client(cid, ClientResponse::EntityAppear(eid, appearance, name));
            self.messages().send_client(cid, motion_msg);
        }
    }

    fn on_entity_disappear(&mut self, cid: ClientId, eid: EntityId) {
        trace!("on_entity_disappear({:?}, {:?})", cid, eid);
        let time =
            if let Some(entity) = self.world().get_entity(eid) {
                entity.motion().start_time
            } else {
                0
            };
        // TODO: figure out if it's actually useful to send the time here.  The client currently
        // ignores it.
        self.messages().send_client(cid, ClientResponse::EntityGone(eid, time));
    }

    fn on_entity_motion_update(&mut self, cid: ClientId, eid: EntityId) {
        // New physics code never uses these callbacks.
        unreachable!();
    }

    fn on_entity_appearance_update(&mut self, cid: ClientId, eid: EntityId) {
        trace!("on_entity_appearance_update({:?}, {:?})", cid, eid);
        self.on_entity_appear(cid, eid);
    }


    fn on_plane_change(&mut self,
                       cid: ClientId,
                       _: PlaneId,
                       pid: PlaneId) {
        // TODO: super hack.  add a flags field to the plane or something.
        let is_dark = match self.world().get_plane(pid) {
            Some(p) => p.name() != "Everfree Forest",
            None => true,
        };
        self.messages().send_client(cid, ClientResponse::PlaneFlags(is_dark as u32));
    }


    fn on_structure_appear(&mut self, cid: ClientId, sid: StructureId) {
        let s = self.world().structure(sid);
        self.messages().send_client(cid, ClientResponse::StructureAppear(
                sid, s.template_id(), s.pos()));
    }

    fn on_structure_disappear(&mut self, cid: ClientId, sid: StructureId) {
        self.messages().send_client(cid, ClientResponse::StructureGone(sid));
    }

    fn on_structure_template_change(&mut self, cid: ClientId, sid: StructureId) {
        let s = self.world().structure(sid);
        self.messages().send_client(cid, ClientResponse::StructureReplace(sid, s.template_id()));
    }


    fn on_inventory_appear(&mut self, cid: ClientId, iid: InventoryId) {
        let i = self.world().inventory(iid);
        let contents = i.contents().iter().map(|&x| x).collect();
        self.messages().send_client(
            cid, ClientResponse::InventoryAppear(iid, contents));
    }

    fn on_inventory_disappear(&mut self, cid: ClientId, iid: InventoryId) {
        self.messages().send_client(
            cid, ClientResponse::InventoryGone(iid));
    }

    fn on_inventory_update(&mut self,
                           cid: ClientId,
                           iid: InventoryId,
                           slot_idx: u8) {
        let i = self.world().inventory(iid);
        let item = i.contents()[slot_idx as usize];
        self.messages().send_client(
            cid, ClientResponse::InventoryUpdate(iid, slot_idx, item));
    }
}



// NB: These add/remove/change methods MUST follow the appropriate protocol:
// - Call `add` only on (id, plane, cpos) tuples that *are not* already present
// - Call `remove` only on (id, plane, cpos) tuples that *are* present
// - Call `change` only on (id, old_plane, old_cpos) tuples that are present
// Otherwise Vision's internal PubSub state may get corrupted.
// (TODO: add debug_assert!s to detect such corruption)
//
// Note that this does allow adding multiple tuples with the same ID and different plane/cpos.

pub fn add_entity(vision: &mut Vision,
                  messages: &mut Messages,
                  world: &World,
                  eid: EntityId,
                  plane: PlaneId,
                  cpos: V2) {
    let e = world.entity(eid);
    // TODO: hack.  Should have a separate "entity name" field somewhere.
    let name = if let Some(c) = e.pawn_owner() {
        c.name().to_owned()
    } else {
        String::new()
    };
    vision.entity_add(eid, plane, cpos, |cid| {
        messages.send_client(
            cid, ClientResponse::EntityAppear(eid, e.appearance(), name.clone()));
    });
}

pub fn remove_entity(vision: &mut Vision,
                     messages: &mut Messages,
                     world: &World,
                     eid: EntityId,
                     plane: PlaneId,
                     cpos: V2) {
    // NB: the indicated entity may not exist at this point, so world.entity(eid) may panic
    vision.entity_remove(eid, plane, cpos, |cid| {
        messages.send_client(
            cid, ClientResponse::EntityGone(eid, 0));
    });
}

pub fn change_entity_chunk(vision: &mut Vision,
                           messages: &mut Messages,
                           world: &World,
                           eid: EntityId,
                           plane: PlaneId,
                           old_cpos: V2,
                           new_cpos: V2) {
    if old_cpos == new_cpos {
        return;
        // Otherwise PubSub state would get corrupted by duplicate `publish` calls.
    }
    add_entity(vision, messages, world, eid, plane, new_cpos);
    remove_entity(vision, messages, world, eid, plane, old_cpos);
}
