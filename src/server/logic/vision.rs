use std::borrow::ToOwned;

use types::*;

use engine::glue::*;
use messages::{Messages, ClientResponse};
use world::{World, Entity, TerrainChunk, Structure, Motion};
use world::object::*;
use vision::{self, Vision};


impl<'a, 'd> vision::Hooks for VisionHooks<'a, 'd> {
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


pub fn entity_appear_message(e: ObjectRef<Entity>) -> ClientResponse {
    // TODO: hack.  Should have a separate "entity name" field somewhere.
    let name = if let Some(c) = e.pawn_owner() {
        c.name().to_owned()
    } else {
        String::new()
    };
    ClientResponse::EntityAppear(e.id(), e.appearance(), name)
}

fn motion_message(eid: EntityId,
                  m: &Motion,
                  anim: AnimId) -> ClientResponse {
    if let Some(end_time) = m.end_time {
        ClientResponse::EntityMotionStartEnd(
            eid, m.start_pos, m.start_time, m.velocity, anim, end_time)
    } else {
        ClientResponse::EntityMotionStart(
            eid, m.start_pos, m.start_time, m.velocity, anim)
    }
}

pub fn entity_motion_message(e: ObjectRef<Entity>) -> ClientResponse {
    motion_message(e.id(), e.motion(), e.anim())
}

/// Similar to `entity_motion_message`, but adjusts the start_time of the reported motion if it is
/// too long before `now`.  This should be used when an entity first appears to a client, to avoid
/// wraparound in the LocalTime.
pub fn entity_motion_message_adjusted(e: ObjectRef<Entity>, now: Time) -> ClientResponse {
    let mut m = e.motion().clone();

    if m.start_time < now - 1000 {
        // If the start time is too far in the past, the client will mistakenly think it's in the
        // future.  To avoid this, adjust the start_time forward to a time just prior to `now`.
        //
        // We step by a whole number of seconds so that the corresponding position offset will be a
        // whole number of pixels, regardless of velocity.
        let adj = (now - m.start_time) / 1000 * 1000;
        m.start_pos = m.pos(m.start_time + adj);
        m.start_time += adj;
    }

    motion_message(e.id(), &m, e.anim())
}

pub fn entity_gone_message(e: ObjectRef<Entity>) -> ClientResponse {
    ClientResponse::EntityGone(e.id(), 0)
}

pub fn entity_gone_message2(eid: EntityId) -> ClientResponse {
    ClientResponse::EntityGone(eid, 0)
}


pub fn terrain_chunk_message(tc: ObjectRef<TerrainChunk>) -> ClientResponse {
    use util::encode_rle16;
    let cpos = tc.chunk_pos();
    let data = encode_rle16(tc.blocks().iter().map(|&x| x));
    ClientResponse::TerrainChunk(cpos, data)
}


pub fn structure_appear_message(s: ObjectRef<Structure>) -> ClientResponse {
    ClientResponse::StructureAppear(s.id(), s.template_id(), s.pos())
}

pub fn structure_gone_message(s: ObjectRef<Structure>) -> ClientResponse {
    ClientResponse::StructureGone(s.id())
}

pub fn structure_replace_message(s: ObjectRef<Structure>) -> ClientResponse {
    ClientResponse::StructureReplace(s.id(), s.template_id())
}
