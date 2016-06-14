use types::*;

use engine::Engine;
use logic;
use messages::{Messages, ClientResponse};
use physics::Physics;
use vision::Vision;
use world::{Activity, Motion, World};
use world::fragment::Fragment as World_Fragment;
use world::fragment::DummyFragment;
use world::object::*;



/// Try to set an entity's appearance.  Returns `true` on success.
pub fn set_appearance(eng: &mut Engine,
                      eid: EntityId,
                      appearance: u32) -> bool {
    let mut wf = DummyFragment::new(&mut eng.world);
    let mut e = unwrap_or!(wf.get_entity_mut(eid), return false);
    e.set_appearance(appearance);

    // TODO: we shouldn't need to send both of these messages.  Just the appear should do.
    // But the client inserts a new, blank entity when it gets the appear message, so we probably
    // need a new "AppearanceChange" message of some sort.
    let msg_appear = logic::vision::entity_appear_message(e.borrow());
    let msg_motion = logic::vision::entity_motion_message(e.borrow());
    let messages = &mut eng.messages;
    eng.vision.entity_update(eid, |cid| {
        messages.send_client(cid, msg_appear.clone());
        messages.send_client(cid, msg_motion.clone());
    });

    true
}


pub fn set_activity_(eng: &mut Engine,
                     eid: EntityId,
                     activity: Activity) -> bool {
    set_activity(&mut eng.world,
                 &mut eng.physics,
                 &mut eng.messages,
                 &mut eng.vision,
                 eng.now,
                 eid,
                 activity)
}

pub fn set_activity(world: &mut World,
                    physics: &mut Physics,
                    messages: &mut Messages,
                    vision: &mut Vision,
                    now: Time,
                    eid: EntityId,
                    activity: Activity) -> bool {
    let mut wf = DummyFragment::new(world);
    let mut e = unwrap_or!(wf.get_entity_mut(eid), return false);

    info!("{:?}: set activity to {:?} at {}", eid, activity, now);

    if e.activity() == activity {
        return true;
    }

    e.set_activity(activity);
    physics.force_update(eid);

    if let Some(c) = e.pawn_owner() {
        messages.send_client(c.id(), ClientResponse::ActivityChange(activity));
    }

    let send_msg = match activity {
        Activity::Move => {
            // Let physics handle the message.  Otherwise we'll just send one made-up motion
            // followed by another correct one.
            false
        },
        Activity::Special(anim, _interruptible) => {
            let pos = e.pos(now);
            e.set_motion(Motion::stationary(pos, now));
            e.set_anim(anim);
            true
        },
    };

    if send_msg {
        let msg = logic::vision::entity_motion_message(e.borrow());
        vision.entity_update(eid, |cid| {
            messages.send_client(cid, msg.clone());
        });
    }

    true
}
