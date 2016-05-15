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
    remove_entities: SmallVec<EntityId>,
}

impl<'d> Physics<'d> {
    pub fn new(data: &'d Data) -> Physics<'d> {
        Physics {
            data: data,
            moving_entities: HashMap::new(),
            remove_entities: SmallVec::new(),
        }
    }

    pub fn set_target_velocity(&mut self, eid: EntityId, v: V3) {
        let me = self.moving_entities.entry(eid).or_insert_with(MovingEntity::new);
        me.target_velocity = v;
    }

    pub fn cleanup(&mut self) {
        for &eid in self.remove_entities.iter() {
            self.moving_entities.remove(&eid);
        }
        self.remove_entities.clear();
    }
}


pub struct UpdateCo<'a, 'd: 'a> {
    data: &'d Data,
    now: Time,
    inner: hash_map::IterMut<'a, EntityId, MovingEntity>,
    remove: &'a mut SmallVec<EntityId>,
}

impl<'d> Physics<'d> {
    pub fn update<'a>(&'a mut self, now: Time) -> UpdateCo<'a, 'd> {
        let ptr = self as *mut _;
        UpdateCo {
            data: self.data,
            now: now,
            inner: self.moving_entities.iter_mut(),
            remove: &mut self.remove_entities,
        }
    }
}

impl<'a, 'b, 'd> Coroutine<(&'b mut World<'d>, &'b TerrainCache)> for UpdateCo<'a, 'd> {
    type Item = (EntityId, Motion, UpdateKind);

    fn send(&mut self, args: (&'b mut World<'d>, &'b TerrainCache)) -> Option<Self::Item> {
        let (world, cache) = args;

        for (&eid, me) in &mut self.inner {
            let now = self.now;
            let next = next_tick(now);

            let (mut m, s) = {
                let e = match world.get_entity(eid) {
                    Some(e) => e,
                    None => {
                        info!("no such entity: {:?}", eid);
                        self.remove.push(eid);
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

            // Actually update the world
            DummyFragment::new(world).entity_mut(eid).set_motion(m.clone());

            let kind = match (started, ended) {
                (false, false) => UpdateKind::Move,
                (true, false) => UpdateKind::Start,
                (false, true) => UpdateKind::End,
                (true, true) => UpdateKind::StartEnd,
            };
            return Some((eid, m, kind));
        }

        None
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
            let idx = Region::new(scalar(0), scalar(CHUNK_SIZE)).index(offset);
            entry.shape[idx]
        } else {
            return Shape::Empty;
        }
    }
}


pub trait Fragment<'d> {
    fn with_cache<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Physics<'d>, &TerrainCache, &World<'d>) -> R;

    type WF: world::Fragment<'d>;
    fn with_world<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Self::WF) -> R;

    fn set_velocity(&mut self, now: Time, eid: EntityId, target: V3) -> StrResult<()> {
        // FIXME remove
        Ok(())
    }

    fn update(&mut self, now: Time, eid: EntityId) -> StrResult<()> {
        // FIXME remove
        Ok(())
    }
}
