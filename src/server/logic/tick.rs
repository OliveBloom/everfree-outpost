use types::*;
use util::SmallVec;
use libphysics::{TILE_SIZE, CHUNK_SIZE};

use engine::Engine;
use engine::split2::Coded;
use input::Action;
use logic;
use timing::next_tick;
use world::object::*;


pub fn tick(eng: &mut Engine) {
    let now = eng.now;
    let next = next_tick(now);

    // Schedule this first, so it gets the first slot in the timing wheel bucket.
    eng.timer.schedule(next, |eng| eng.tick());

    for (cid, (act, args)) in eng.input.actions() {
        if let Some(e) = eng.world.client(cid).pawn() {
            if !e.activity().interruptible() {
                continue;
            }
        } else {
            // No pawn
            continue;
        }

        match act {
            Action::Interact =>
                logic::input::interact(eng, cid, args),
            Action::UseItem(item_id) =>
                logic::input::use_item(eng, cid, item_id, args),
            Action::UseAbility(item_id) =>
                logic::input::use_ability(eng, cid, item_id, args),
        }
    }


    logic::movement::update(eng);
    // Call other systems that update motions here

    
    // TODO: kind of a hack, make this nicer & more efficient
    let mut eids_crossing = SmallVec::new();
    let chunk_px = scalar(TILE_SIZE * CHUNK_SIZE);
    for e in eng.world.entities() {
        let old_chunk = e.pos(now).reduce().div_floor(chunk_px);
        let new_chunk = e.pos(next).reduce().div_floor(chunk_px);

        if new_chunk != old_chunk {
            eids_crossing.push(e.id());
        }
    }

    for &eid in eids_crossing.iter() {
        let (plane, old_chunk, new_chunk, owner) = {
            let e = eng.world.entity(eid);
            (e.plane_id(),
             e.pos(now).reduce().div_floor(chunk_px),
             e.pos(next).reduce().div_floor(chunk_px),
             e.pawn_owner().map(|c| c.id()))
        };
        logic::entity::on_chunk_crossing(eng.refine(), eid,
                                         plane, old_chunk,
                                         plane, new_chunk);

        if let Some(cid) = owner {
            logic::client::update_view(eng, cid, plane, old_chunk, plane, new_chunk);
        }
    }
}
