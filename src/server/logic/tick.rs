use types::*;
use libphysics::{TILE_SIZE, CHUNK_SIZE};

use engine::Engine;
use engine::split2::Coded;
use input::{Action, InputBits, INPUT_DIR_MASK};
use logic;
use messages::ClientResponse;
use timing::next_tick;
use world::Activity;
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
}
