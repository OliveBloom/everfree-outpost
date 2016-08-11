use types::*;
use util::StrResult;
use libcommon_proto::types::{LocalPos, LocalTime};
use libphysics::{TILE_BITS, CHUNK_BITS, LOCAL_BITS};

use engine::Engine;
use input::InputBits;
use world::object::*;


pub fn path_start(eng: &mut Engine,
                  cid: ClientId,
                  pos: LocalPos,
                  delay: u16) -> StrResult<()> {
    let c = unwrap!(eng.world.get_client(cid));
    let e = unwrap!(c.pawn());
    let pos = pos.to_global_bits(e.pos(eng.now),
                                 TILE_BITS + CHUNK_BITS + LOCAL_BITS);
    eng.movement.queue_start(eng.now, e.id, pos, delay);
    Ok(())
}

pub fn path_update(eng: &mut Engine,
                   cid: ClientId,
                   rel_time: LocalTime,
                   velocity: V3,
                   input: InputBits) -> StrResult<()> {
    let c = unwrap!(eng.world.get_client(cid));
    let e = unwrap!(c.pawn());
    eng.movement.queue_update(eng.now, e.id, rel_time, velocity, input);
    Ok(())
}

pub fn path_blocked(eng: &mut Engine,
                    cid: ClientId,
                    rel_time: LocalTime) -> StrResult<()> {
    let c = unwrap!(eng.world.get_client(cid));
    let e = unwrap!(c.pawn());
    eng.movement.queue_blocked(eng.now, e.id, rel_time);
    Ok(())
}
