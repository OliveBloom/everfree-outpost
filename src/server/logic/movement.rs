use types::*;
use util::SmallVec;
use util::StrResult;
use libcommon_proto::types::{LocalPos, LocalTime};
use libphysics::{TILE_BITS, CHUNK_BITS, LOCAL_BITS};
use libphysics::{CHUNK_SIZE, CHUNK_MASK};
use libphysics::ShapeSource;

use cache::TerrainCache;
use components::movement;
use data::Data;
use engine::Engine;
use engine::split2::Coded;
use input::{InputBits, INPUT_DIR_MASK};
use logic;
use messages::ClientResponse;
use world::{Activity, Motion};
use world::object::*;


pub fn path_start(eng: &mut Engine,
                  cid: ClientId,
                  pos: LocalPos,
                  delay: u16) -> StrResult<()> {
    let c = unwrap!(eng.world.get_client(cid));
    let e = unwrap!(c.pawn());
    let old_pos = pos;
    let pos = pos.to_global_bits(e.pos(eng.now),
                                 TILE_BITS + CHUNK_BITS + LOCAL_BITS);
    trace!("record start: {:?}, {:?}", e.id(), pos);
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
    trace!("record update: {:?}, {:?}, {:?}, {:?}", e.id(), rel_time, velocity, input);
    eng.movement.queue_update(eng.now, e.id, rel_time, velocity, input);
    Ok(())
}

pub fn path_blocked(eng: &mut Engine,
                    cid: ClientId,
                    rel_time: LocalTime) -> StrResult<()> {
    let c = unwrap!(eng.world.get_client(cid));
    let e = unwrap!(c.pawn());
    trace!("record block: {:?}, {:?}", e.id(), rel_time);
    eng.movement.queue_blocked(eng.now, e.id, rel_time);
    Ok(())
}


engine_part2!(MovementParts(world, movement, cache));

pub fn update(eng: &mut Engine) {
    let now = eng.now;
    let data = eng.data;
    let mut to_remove = SmallVec::new();

    let (eng_m, eng): (&mut MovementParts, &mut logic::entity::EngineVision) = eng.split();

    for (&eid, em) in eng_m.movement.iter() {
        let mut e = unwrap_or!(eng_m.world.get_entity_mut(eid),
                               { error!("{:?} has EntityMovement but no Entity", eid);
                                 continue; });

        match em.process(now, e.pos(now)) {
            movement::Change::None |
            movement::Change::Motion => {},
            movement::Change::Input => {
                if !(em.cur_input() & INPUT_DIR_MASK).is_empty() &&
                   e.activity().interruptible() {
                    logic::entity::set_activity_internal(eng, e.borrow_mut(), Activity::Walk);
                }
            },
            movement::Change::Conflict => {
                // TODO: duplicated code
                let pos = e.pos(now);
                let anim = movement::facing_anim(data, e.facing(), 0);
                e.set_motion(Motion::stationary(pos, now));
                e.set_anim(anim);
                let msg = ClientResponse::EntityMotionStart(
                    eid, pos, now, scalar(0), anim);
                let messages = &mut eng.messages;
                eng.vision.entity_update(eid, |cid| {
                    messages.send_client(cid, msg.clone());
                });
                to_remove.push(eid);
                continue;
            },
        }

        if e.activity() != Activity::Walk {
            continue;
        }


        let s = ChunksSource {
            cache: &eng_m.cache,
            base_tile: scalar(0),
            plane: e.plane_id(),
        };

        let upd = em.update(e.borrow_mut(), now, data, &s);

        if upd != movement::Update::None {
            let m = e.motion();
            let anim = e.anim();
            let msg = match upd {
                movement::Update::None => unreachable!(),
                movement::Update::Start => ClientResponse::EntityMotionStart(
                    eid, m.start_pos, m.start_time, m.velocity, anim),
                movement::Update::End => ClientResponse::EntityMotionEnd(
                    eid, m.end_time.unwrap()),
                movement::Update::StartEnd => ClientResponse::EntityMotionStartEnd(
                    eid, m.start_pos, m.start_time, m.velocity, anim, m.end_time.unwrap()),
                movement::Update::Conflict => {
                    to_remove.push(eid);
                    ClientResponse::EntityMotionStart(
                        eid, m.start_pos, m.start_time, m.velocity, anim)
                },
            };
            let messages = &mut eng.messages;
            eng.vision.entity_update(eid, |cid| {
                messages.send_client(cid, msg.clone());
            });
        }

        if em.done() {
            to_remove.push(eid);
        }
    }

    for &eid in to_remove.iter() {
        eng_m.movement.clear(eid);
    }
}





struct ChunksSource<'a> {
    cache: &'a TerrainCache,
    base_tile: V3,
    plane: PlaneId,
}

impl<'a> ShapeSource for ChunksSource<'a> {
    fn get_shape(&self, pos: V3) -> Shape {
        if pos.z < 0 || pos.z >= CHUNK_SIZE {
            return Shape::Empty;
        }

        let pos = pos + self.base_tile;

        let offset = pos & scalar(CHUNK_MASK);
        let cpos = (pos >> CHUNK_BITS).reduce();

        if let Some(entry) = self.cache.get(self.plane, cpos) {
            entry.get(offset).shape()
        } else {
            Shape::Empty
        }
    }
}
