use types::*;

use messages::ClientResponse;
use world::OpResult;
use world::Entity;
use world::object::*;


engine_part2!(pub EngineParts(world, messages, energy));

pub fn init(eng: &mut EngineParts, eid: EntityId, max: i32) -> OpResult<()> {
    // Make sure entity exists.
    let e = unwrap!(eng.world.get_entity(eid));

    eng.energy.init(eid, max);
    send_update(eng, e);
    Ok(())
}

/// Create an energy gauge if one doesn't already exist.
pub fn check_init(eng: &mut EngineParts, eid: EntityId, max: i32) -> OpResult<()> {
    if eng.energy.has_gauge(eid) {
        let e = unwrap!(eng.world.get_entity(eid));
        send_update(eng, e);
        Ok(())
    } else {
        init(eng, eid, max)
    }
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
    let result = eng.energy.give(eid, amount, now);
    send_update(eng, e);
    Ok(result)
}

pub fn take(eng: &mut EngineParts, eid: EntityId, amount: i32) -> OpResult<bool> {
    let now = eng.now();
    let e = unwrap!(eng.world.get_entity(eid));
    let result = eng.energy.take(eid, amount, now);
    send_update(eng, e);
    Ok(result)
}

fn send_update(eng: &EngineParts, e: ObjectRef<Entity>) {
    let c = unwrap_or!(e.pawn_owner());
    let g = eng.energy.gauge(e.id());
    let msg = ClientResponse::EnergyUpdate(g.last_value(),
                                           g.max(),
                                           g.rate(),
                                           g.last_time());
    eng.messages.send_client(c.id(), msg);
}
