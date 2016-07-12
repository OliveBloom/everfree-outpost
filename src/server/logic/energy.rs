use types::*;

use world::OpResult;


engine_part2!(pub EngineParts(world, messages, energy));

pub fn init(eng: &mut EngineParts, eid: EntityId, max: i32) -> OpResult<()> {
    // Make sure entity exists.
    unwrap!(eng.world.get_entity(eid));

    eng.energy.init(eid, max);
    // TODO: send message to controller
    Ok(())
}

pub fn get(eng: &mut EngineParts, eid: EntityId) -> OpResult<i32> {
    // Make sure entity exists.
    unwrap!(eng.world.get_entity(eid));

    let now = eng.now();
    Ok(eng.energy.get(eid, now))
}

pub fn give(eng: &mut EngineParts, eid: EntityId, amount: i32) -> OpResult<i32> {
    let now = eng.now();
    let e = unwrap!(eng.world.get_entity(eid));
    Ok(eng.energy.give(eid, amount, now))
}

pub fn take(eng: &mut EngineParts, eid: EntityId, amount: i32) -> OpResult<bool> {
    let now = eng.now();
    let e = unwrap!(eng.world.get_entity(eid));
    Ok(eng.energy.take(eid, amount, now))
}
