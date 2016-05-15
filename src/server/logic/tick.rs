use types::*;
use libphysics::{TILE_SIZE, CHUNK_SIZE};

use engine::Engine;
use input::Action;
use logic;
use messages::ClientResponse;
use physics::UpdateKind;
use timing::next_tick;
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
        if let Some(c) = eng.world.get_client(cid) {
            if let Some(e) = c.pawn() {
                eng.physics.set_target_velocity(e.id(), input.to_velocity());
            }
            eng.messages.send_client(cid, ClientResponse::ProcessedInputs(now, count));
        }
    }

    co_for!( (eid, m, kind) in
                (eng.physics.update(now)) (&mut eng.world, &eng.cache) {
        let chunk_px = scalar(TILE_SIZE * CHUNK_SIZE);
        let old_chunk = m.pos(now).reduce().div_floor(chunk_px);
        let new_chunk = m.pos(next).reduce().div_floor(chunk_px);

        if new_chunk != old_chunk {
            // FIXME vision publisher update

            if let Some(c) = eng.world.entity(eid).pawn_owner() {
                // FIXME vision subscriber update
            }
        }

        if kind != UpdateKind::Move {
            let msg = match kind {
                UpdateKind::Move => unreachable!(),
                // FIXME anim handling
                UpdateKind::Start => ClientResponse::EntityMotionStart(
                    eid, m.start_pos, m.start_time, m.velocity, 0),
                UpdateKind::End => ClientResponse::EntityMotionEnd(
                    eid, m.end_time.unwrap()),
                UpdateKind::StartEnd => ClientResponse::EntityMotionStartEnd(
                    eid, m.start_pos, m.start_time, m.velocity, 0, m.end_time.unwrap()),
            };
            // FIXME Do vision broadcast of msg
        }
    });

    eng.physics.cleanup();
}
