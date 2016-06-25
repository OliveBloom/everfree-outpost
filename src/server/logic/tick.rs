use types::*;
use libphysics::{TILE_SIZE, CHUNK_SIZE};

use engine::Engine;
use engine::split2::Coded;
use input::{Action, InputBits, INPUT_DIR_MASK};
use logic;
use messages::ClientResponse;
use physics::UpdateKind;
use timing::next_tick;
use world::Activity;
use world::object::*;


pub fn tick(eng: &mut Engine) {
    let now = eng.now;
    let next = next_tick(now);

    // Schedule this first, so it gets the first slot in the timing wheel bucket.
    eng.timer.schedule(next, |eng| tick(eng.unwrap()));

    for (cid, (act, args)) in eng.input.actions() {
        match act {
            Action::Interact =>
                logic::input::interact(eng.as_ref(), cid, args),
            Action::UseItem(item_id) =>
                logic::input::use_item(eng.as_ref(), cid, item_id, args),
            Action::UseAbility(item_id) =>
                logic::input::use_ability(eng.as_ref(), cid, item_id, args),
        }
    }

    for (cid, (input, count)) in eng.input.inputs() {
        let opt_eid = if let Some(c) = eng.world.get_client(cid) {
            if let Some(e) = c.pawn() {
                eng.physics.set_input(e.id(), input);
            }
            eng.messages.send_client(cid, ClientResponse::ProcessedInputs(now, count));
            c.pawn_id()
        } else {
            None
        };

        if let Some(eid) = opt_eid {
            if input & INPUT_DIR_MASK != InputBits::empty() &&
               eng.world.entity(eid).activity().interruptible() {
                logic::entity::set_activity(eng.refine(),
                                            eid,
                                            Activity::Walk);
            }
        }
    }

    // FIXME borrow checker workaround
    // This one's okay because VisionFragment doesn't include physics.
    let eng2: &mut Engine = unsafe { &mut *(eng as *mut _) };

    co_for!( (eid, m, anim, kind) in
                (eng.physics.update(now)) (&mut eng.world, &eng.cache) {
        let chunk_px = scalar(TILE_SIZE * CHUNK_SIZE);
        let plane = eng.world.entity(eid).plane_id();
        let old_chunk = m.pos(now).reduce().div_floor(chunk_px);
        let new_chunk = m.pos(next).reduce().div_floor(chunk_px);

        if new_chunk != old_chunk {
            // TODO: on_chunk_crossing needs to take a variant of PartialEngine that doesn't
            // include Physics.  Then we need to call eng.split() above to get a reduced engine +
            // Physics.
            logic::entity::on_chunk_crossing(eng2.refine(), eid,
                                             plane, old_chunk,
                                             plane, new_chunk);

            if let Some(cid) = eng.world.entity(eid).pawn_owner().map(|c| c.id()) {
                logic::client::update_view(eng2, cid, plane, old_chunk, plane, new_chunk);
            }
        }

        if kind != UpdateKind::Move {
            let msg = match kind {
                UpdateKind::Move => unreachable!(),
                UpdateKind::Start => ClientResponse::EntityMotionStart(
                    eid, m.start_pos, m.start_time, m.velocity, anim),
                UpdateKind::End => ClientResponse::EntityMotionEnd(
                    eid, m.end_time.unwrap()),
                UpdateKind::StartEnd => ClientResponse::EntityMotionStartEnd(
                    eid, m.start_pos, m.start_time, m.velocity, anim, m.end_time.unwrap()),
            };
            let messages = &mut eng.messages;
            eng.vision.entity_update(eid, |cid| {
                messages.send_client(cid, msg.clone());
            });
        }
    });
}
