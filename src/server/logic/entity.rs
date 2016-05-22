use types::*;
use libphysics::{TILE_SIZE, CHUNK_SIZE};

use engine::Engine;
use input::Action;
use logic;
use messages::ClientResponse;
use physics::UpdateKind;
use timing::next_tick;
use vision;
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
