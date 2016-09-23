use types::*;
use util::StrResult;

use engine::Engine;
use logic;
use messages::ClientResponse;
use world::{Activity, Motion};
use world::object::*;


pub fn set(eng: &mut Engine,
           eid: EntityId,
           activity: Activity) -> StrResult<()> {
    let old_activity = unwrap!(eng.world.get_entity(eid)).activity();
    if activity == old_activity {
        return Ok(());
    }

    match old_activity {
        Activity::Walk => eng.movement.clear(eid),
        Activity::Emote(_) => {},   // no-op
        Activity::Work(_, _) => clear_activity_icon(eng, eid),
        Activity::Teleport => {},
    }

    {
        let mut e = eng.world.entity_mut(eid);
        e.set_activity(activity);

        if let Some(c) = e.pawn_owner() {
            eng.messages.send_client(c.id(), ClientResponse::ActivityChange(activity));
        }
    }

    match activity {
        Activity::Walk => {},       // no-op
        Activity::Emote(anim) => set_stationary_anim(eng, eid, anim),
        Activity::Work(anim, icon) => {
            set_stationary_anim(eng, eid, anim);
            set_activity_icon(eng, eid, icon);
        },
        Activity::Teleport => {},
    }

    Ok(())
}

pub fn interrupt(eng: &mut Engine,
                 eid: EntityId,
                 activity: Activity) -> StrResult<bool> {
    let old_activity = unwrap!(eng.world.get_entity(eid)).activity();
    if !old_activity.interruptible() {
        return Ok(false);
    }

    try!(set(eng, eid, activity));
    Ok(true)
}

fn set_stationary_anim(eng: &mut Engine,
                       eid: EntityId,
                       anim: AnimId) {
    let mut e = eng.world.entity_mut(eid);
    let pos = e.pos(eng.now);
    e.set_motion(Motion::stationary(pos, eng.now));
    e.set_anim(anim);

    let messages = &mut eng.messages;
    let msg = logic::vision::entity_motion_message(e.borrow());
    eng.vision.entity_update(e.id(), |cid| {
        messages.send_client(cid, msg.clone());
    });
}

fn set_activity_icon(eng: &mut Engine,
                     eid: EntityId,
                     icon: AnimId) {
    let e = eng.world.entity(eid);

    let messages = &mut eng.messages;
    let msg = logic::vision::entity_activity_icon_message(e, icon);
    eng.vision.entity_update(e.id(), |cid| {
        messages.send_client(cid, msg.clone());
    });
}

fn clear_activity_icon(eng: &mut Engine,
                       eid: EntityId) {
    let no_icon = eng.data.animation_id("activity//none");
    set_activity_icon(eng, eid, no_icon);
}
