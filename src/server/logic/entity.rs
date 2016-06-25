use types::*;
use libphysics::{CHUNK_SIZE, TILE_SIZE};

use engine::Engine;
use logic;
use messages::{Messages, ClientResponse};
use physics::Physics;
use vision::Vision;
use world::{Activity, Motion, World};
use world::fragment::Fragment as World_Fragment;
use world::fragment::DummyFragment;
use world::object::*;


engine_part2!(pub PartialEngine(world, physics, vision, messages));


/// Handler to be called just after creating an entity.
pub fn on_create(eng: &mut PartialEngine, eid: EntityId) {
    let e = eng.world.entity(eid);

    eng.physics.add_entity(eid);

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

    eng.physics.remove_entity(eid);

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
    let mut wf = DummyFragment::new(&mut eng.world);
    let mut e = unwrap_or!(wf.get_entity_mut(eid), return false);
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


pub fn set_activity(eng: &mut Engine,
                    eid: EntityId,
                    activity: Activity) -> bool {
    let now = eng.now();

    let mut wf = DummyFragment::new(&mut eng.world);
    let mut e = unwrap_or!(wf.get_entity_mut(eid), return false);

    info!("{:?}: set activity to {:?} at {}", eid, activity, now);

    if e.activity() == activity {
        return true;
    }

    e.set_activity(activity);
    eng.physics.force_update(eid);

    if let Some(c) = e.pawn_owner() {
        eng.messages.send_client(c.id(), ClientResponse::ActivityChange(activity));
    }

    // FIXME: need to send "activity icon = none" when changing back to walk/emote
    // Currently this is handled explicitly on the python side, which is very ugly.

    let messages = &mut eng.messages;
    match activity {
        Activity::Walk => {
            // Let physics handle the message.  Otherwise we'll just send one made-up motion
            // followed by another correct one.
        },
        Activity::Emote(anim) => {
            let pos = e.pos(now);
            e.set_motion(Motion::stationary(pos, now));
            e.set_anim(anim);

            let msg = logic::vision::entity_motion_message(e.borrow());
            eng.vision.entity_update(eid, |cid| {
                messages.send_client(cid, msg.clone());
            });
        },
        Activity::Work(anim, icon) => {
            let pos = e.pos(now);
            e.set_motion(Motion::stationary(pos, now));
            e.set_anim(anim);

            let msg_motion = logic::vision::entity_motion_message(e.borrow());
            let msg_icon = logic::vision::entity_activity_icon_message(e.borrow(), icon);
            eng.vision.entity_update(eid, |cid| {
                messages.send_client(cid, msg_motion.clone());
                messages.send_client(cid, msg_icon.clone());
            });
        },
    }

    true
}
