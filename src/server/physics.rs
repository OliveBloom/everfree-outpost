//! Interface to the physics engine.  The physics engine itself lives in a separate library,
//! `libphysics`, so that it can be compiled to asm.js for use on the client.  This system just
//! provides the glue to connect the physics engine to entities and the rest of the `World`.
use std::collections::hash_map::{self, HashMap};
use libphysics::{self, ShapeSource};
use libphysics::{CHUNK_SIZE, CHUNK_BITS, CHUNK_MASK, TILE_SIZE};

use types::*;
use util::StrResult;
use util::SmallVec;
use util::Coroutine;

use cache::TerrainCache;
use data::Data;
use timing::{next_tick, TICK_MS};
use world::{self, World};
use world::{Motion, Activity};
use world::fragment::Fragment as World_Fragment;
use world::fragment::DummyFragment;
use world::object::*;


struct MovingEntity {
    target_velocity: V3,
    current_velocity: V3,
}

impl MovingEntity {
    fn new() -> MovingEntity {
        MovingEntity {
            target_velocity: scalar(0),
            current_velocity: scalar(0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UpdateKind {
    Move,
    Start,
    End,
    StartEnd,
}

pub struct Physics<'d> {
    data: &'d Data,
    moving_entities: HashMap<EntityId, MovingEntity>,
}

impl<'d> Physics<'d> {
    pub fn new(data: &'d Data) -> Physics<'d> {
        Physics {
            data: data,
            moving_entities: HashMap::new(),
        }
    }

    pub fn add_entity(&mut self, eid: EntityId, velocity: V3) {
        let mut me = MovingEntity::new();
        me.current_velocity = velocity;
        self.moving_entities.insert(eid, me);
    }

    pub fn remove_entity(&mut self, eid: EntityId) {
        self.moving_entities.remove(&eid);
    }

    pub fn set_target_velocity(&mut self, eid: EntityId, v: V3) {
        let me = self.moving_entities.entry(eid).or_insert_with(MovingEntity::new);
        me.target_velocity = v;
    }
}


pub struct UpdateCo<'a, 'd: 'a> {
    data: &'d Data,
    now: Time,
    inner: hash_map::IterMut<'a, EntityId, MovingEntity>,
}

impl<'d> Physics<'d> {
    pub fn update<'a>(&'a mut self, now: Time) -> UpdateCo<'a, 'd> {
        let ptr = self as *mut _;
        UpdateCo {
            data: self.data,
            now: now,
            inner: self.moving_entities.iter_mut(),
        }
    }
}

impl<'a, 'b, 'd> Coroutine<(&'b mut World<'d>, &'b TerrainCache)> for UpdateCo<'a, 'd> {
    type Item = (EntityId, Motion, AnimId, UpdateKind);

    fn send(&mut self, args: (&'b mut World<'d>, &'b TerrainCache)) -> Option<Self::Item> {
        let (world, cache) = args;

        for (&eid, me) in &mut self.inner {
            let now = self.now;
            let next = next_tick(now);

            let (mut m, s) = {
                let e = match world.get_entity(eid) {
                    Some(e) => e,
                    None => {
                        error!("BUG: no such entity: {:?}", eid);
                        continue;
                    },
                };

                let m = e.motion().clone();

                let s = ChunksSource {
                    cache: cache,
                    base_tile: scalar(0),
                    plane: e.plane_id(),
                };

                (m, s)
            };

            let pos = m.pos(now);
            let size = V3::new(32, 32, 48);
            let mut collider = libphysics::walk2::Collider::new(&s, Region::new(pos, pos + size));


            // 1) Compute the actual velocity for this tick
            let velocity = collider.calc_velocity(me.target_velocity);
            let started = velocity != me.current_velocity;
            if started {
                me.current_velocity = velocity;
                m = Motion {
                    start_pos: pos,
                    velocity: velocity,
                    start_time: now,
                    end_time: None,
                };
            }

            let next_pos = m.pos(next);
            let (step, dur) = collider.walk(next_pos - pos, TICK_MS as i32);
            let ended = dur != TICK_MS as i32;
            if ended {
                m.end_time = Some(now + dur as Time);
            }


            // 2) Actually update the world
            let anim = {
                let data = world.data();
                let mut wf = DummyFragment::new(world);
                let mut e = wf.entity_mut(eid);

                if started || ended {
                    e.set_motion(m.clone());
                }

                if started {
                    let facing = if me.target_velocity != scalar(0) {
                        me.target_velocity.signum()
                    } else if e.facing() != scalar(0) {
                        e.facing()
                    } else {
                        V3::new(1, 0, 0)
                    };
                    e.set_facing(facing);

                    let speed = me.target_velocity.abs().max() as usize / 50;
                    let anim = facing_anim(data, facing, speed);
                    e.set_anim(anim);

                    anim
                } else {
                    e.anim()
                }
            };


            // 3) Compute result

            let kind = match (started, ended) {
                (false, false) => UpdateKind::Move,
                (true, false) => UpdateKind::Start,
                (false, true) => UpdateKind::End,
                (true, true) => UpdateKind::StartEnd,
            };
            return Some((eid, m, anim, kind));
        }

        None
    }
}


fn facing_anim(data: &Data, facing: V3, speed: usize) -> AnimId {
    const ANIM_DIR_COUNT: AnimId = 8;
    static SPEED_NAME_MAP: [&'static str; 4] = ["stand", "walk", "", "run"];
    let idx = (3 * (facing.x + 1) + (facing.y + 1)) as usize;
    let anim_dir = [2, 2, 2, 3, 0, 1, 0, 0, 0][idx];
    let anim_name = format!("pony//{}-{}",
                            SPEED_NAME_MAP[speed],
                            anim_dir);
    data.animations.get_id(&anim_name)
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
            let idx = Region::new(scalar(0), scalar(CHUNK_SIZE)).index(offset);
            entry.shape[idx]
        } else {
            return Shape::Empty;
        }
    }
}
