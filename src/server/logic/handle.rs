//! Common code for handling changes to the world, mainly object lifecycle events.

use types::*;
use libphysics::{CHUNK_SIZE, TILE_SIZE};

use engine::Engine;
use logic::vision;
use world::Entity;
use world::object::*;


pub fn entity_create(eng: &mut Engine, eid: EntityId) {
    let e = eng.world.entity(eid);

    eng.physics.add_entity(eid, eng.world.entity(eid).motion().velocity);

    let msg_appear = vision::entity_appear_message(e);
    let msg_motion = vision::entity_motion_message_adjusted(e, eng.now);
    let plane = e.plane_id();
    let cpos = e.pos(eng.now).reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));
    let messages = &mut eng.messages;
    eng.vision.entity_add(eid, plane, cpos, |cid| {
        messages.send_client(cid, msg_appear.clone());
        messages.send_client(cid, msg_motion.clone());
    });
}

pub fn entity_destroy(eng: &mut Engine, eid: EntityId) {
    let e = eng.world.entity(eid);

    eng.physics.remove_entity(eid);

    let msg_gone = vision::entity_gone_message2(eid);
    let plane = e.plane_id();
    let cpos = e.pos(eng.now).reduce().div_floor(scalar(CHUNK_SIZE * TILE_SIZE));
    let messages = &mut eng.messages;
    eng.vision.entity_remove(eid, plane, cpos, |cid| {
        messages.send_client(cid, msg_gone.clone());
    });
}
