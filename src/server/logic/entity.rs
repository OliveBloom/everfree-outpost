use types::*;
use libphysics::{CHUNK_SIZE, TILE_SIZE};
use util::StrResult;

use engine::Engine;
use engine::split2::Coded;
use logic;
use messages::{ClientResponse, SyncKind};
use world::{Activity, Motion, Entity};
use world::object::*;


engine_part2!(pub PartialEngine(world, vision, messages));


/// Handler to be called just after creating an entity.
pub fn on_create(eng: &mut PartialEngine, eid: EntityId) {
    let e = eng.world.entity(eid);

    let msg_appear = logic::vision::entity_appear_message(e);
    let msg_motion = logic::vision::entity_motion_message_adjusted(e, eng.now());
    let plane = e.plane_id();
    let cpos = e.pos(eng.now()).reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));
    let messages = &mut eng.messages;
    eng.vision.entity_add(eid, plane, cpos, |cid| {
        messages.send_client(cid, msg_appear.clone());
        messages.send_client(cid, msg_motion.clone());
    });
}

/// Handler to be called just before destroying an entity.
pub fn on_destroy(eng: &mut PartialEngine, eid: EntityId) {
    let e = eng.world.entity(eid);

    let msg_gone = logic::vision::entity_gone_message(e);
    let plane = e.plane_id();
    let cpos = e.pos(eng.now()).reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));
    let messages = &mut eng.messages;
    eng.vision.entity_remove(eid, plane, cpos, |cid| {
        messages.send_client(cid, msg_gone.clone());
    });
}

/// Handler to be called when an entity crosses from one chunk to another.
pub fn on_chunk_crossing(eng: &mut PartialEngine,
                         eid: EntityId,
                         old_plane: PlaneId,
                         old_cpos: V2,
                         new_plane: PlaneId,
                         new_cpos: V2) {
    if (old_plane, old_cpos) == (new_plane, new_cpos) {
        return;
    }

    let e = eng.world.entity(eid);

    let msg_appear = logic::vision::entity_appear_message(e);
    let msg_motion = logic::vision::entity_motion_message_adjusted(e, eng.now());
    let msg_gone = logic::vision::entity_gone_message(e);
    let messages = &mut eng.messages;
    eng.vision.entity_add(eid, new_plane, new_cpos, |cid| {
        messages.send_client(cid, msg_appear.clone());
        messages.send_client(cid, msg_motion.clone());
    });
    eng.vision.entity_remove(eid, old_plane, old_cpos, |cid| {
        messages.send_client(cid, msg_gone.clone());
    });
}


/// Try to set an entity's appearance.  Returns `true` on success.
pub fn set_appearance(eng: &mut PartialEngine,
                      eid: EntityId,
                      appearance: u32) -> bool {
    let mut e = unwrap_or!(eng.world.get_entity_mut(eid), return false);
    e.set_appearance(appearance);

    // TODO: we shouldn't need to send both of these messages.  Just the appear should do.
    // But the client inserts a new, blank entity when it gets the appear message, so we probably
    // need a new "AppearanceChange" message of some sort.
    // TODO: should fix this to use motion_message_adjusted, i think?
    let msg_appear = logic::vision::entity_appear_message(e.borrow());
    let msg_motion = logic::vision::entity_motion_message(e.borrow());
    let messages = &mut eng.messages;
    eng.vision.entity_update(eid, |cid| {
        messages.send_client(cid, msg_appear.clone());
        messages.send_client(cid, msg_motion.clone());
    });

    true
}


engine_part2!(OnlyWorld(world));
engine_part2!(pub EngineVision(vision, messages));

fn teleport_impl(eng: &mut Engine,
                 eid: EntityId,
                 pid: Option<PlaneId>,
                 stable_pid: Option<Stable<PlaneId>>,
                 pos: V3) -> StrResult<()> {
    // Figure out the old and new positions.
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

    // Maybe send the client a desync message.
    if let Some(cid) = cid {
        // Check if we need to send a desync message.
        // Teleporting to another point within the current chunk doesn't require any significant
        // amount of data transfer, so we only send a desync (causing the client to show a loading
        // screen) if the plane or cpos has changed.
        if new_plane != old_plane || new_cpos != old_cpos {
            eng.messages.send_client(cid, ClientResponse::SyncStatus(SyncKind::Loading));
        }
    }

    // Actually move the entity.
    {
        // These operations should never fail, since the values were either checked or known-good.
        let mut e = eng.world.entity_mut(eid);
        e.set_plane_id(new_plane).expect("failed to set plane id");
        e.set_motion(Motion::stationary(new_pos, eng.now));

        // Send messages to viewers.
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

    // Update the client's view.  Once this finishes, the client will get a resync message.
    if let Some(cid) = cid {
        logic::client::update_view(eng, cid, old_plane, old_cpos, new_plane, new_cpos);
    }

    Ok(())
}

pub fn teleport(eng: &mut Engine,
                eid: EntityId,
                pos: V3) -> StrResult<()> {
    teleport_impl(eng, eid, None, None, pos)
}

pub fn teleport_plane(eng: &mut Engine,
                      eid: EntityId,
                      pid: PlaneId,
                      pos: V3) -> StrResult<()> {
    teleport_impl(eng, eid, Some(pid), None, pos)
}

pub fn teleport_stable_plane(eng: &mut Engine,
                             eid: EntityId,
                             stable_pid: Stable<PlaneId>,
                             pos: V3) -> StrResult<()> {
    teleport_impl(eng, eid, None, Some(stable_pid), pos)
}

