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

    pub fn update_<F>(&mut self,
                     world: &mut World,
                     cache: &TerrainCache,
                     now: Time,
                     mut update: F)
            where F: FnMut(EntityId, &Motion, UpdateKind) {
        let mut remove_eids = SmallVec::new();
        let next = next_tick(now);

        for (&eid, me) in &mut self.moving_entities {
            let (mut m, s) = {
                let e = match world.get_entity(eid) {
                    Some(e) => e,
                    None => {
                        info!("no such entity: {:?}", eid);
                        remove_eids.push(eid);
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

            let kind = match (started, ended) {
                (false, false) => UpdateKind::Move,
                (true, false) => UpdateKind::Start,
                (false, true) => UpdateKind::End,
                (true, true) => UpdateKind::StartEnd,
            };
            update(eid, &m, kind);

            DummyFragment::new(world).entity_mut(eid).set_motion(m);
        }

        for eid in remove_eids.iter() {
            self.moving_entities.remove(eid);
        }
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
        /*
        use world::Fragment;
        let changed = try!(self.with_world(|wf| -> StrResult<_> {
            let mut e = unwrap!(wf.get_entity_mut(eid));
            if e.activity().interruptible() {
                e.set_target_velocity(target);
                e.set_activity(Activity::Move);
                Ok(true)
            } else {
                Ok(false)
            }
        }));
        if changed {
            try!(self.update(now, eid));
        }
        */
        Ok(())
    }

    fn update(&mut self, now: Time, eid: EntityId) -> StrResult<()> {
        /*
        use world::Fragment;

        let motion = try!(self.with_cache(|_sys, cache, world| -> StrResult<_> {
            let e = unwrap!(world.get_entity(eid));

            match e.activity() {
                Activity::Move => {},   // Fall through to physics calculation
                Activity::Special(_, _) => {
                    let pos = e.pos(now);
                    return Ok(Motion {
                        start_time: now,
                        duration: DURATION_MAX,
                        start_pos: pos,
                        end_pos: pos,
                    });
                },
            }

            // Run the physics calculation

            // TODO: hardcoded constant based on entity size
            let start_pos = e.pos(now);
            let velocity = e.target_velocity();
            let size = V3::new(32, 32, 64);

            let chunk_px = CHUNK_SIZE * TILE_SIZE;
            let base_chunk = start_pos.div_floor(scalar(chunk_px)) - scalar::<V2>(3).extend(0);
            let base_tile = base_chunk * scalar(CHUNK_SIZE);
            let base_px = base_tile * scalar(TILE_SIZE);

            let source = ChunksSource {
                cache: cache,
                base_tile: base_tile,
                plane: e.plane_id(),
            };
            let (mut end_pos, mut dur) =
                libphysics::collide(&source, start_pos - base_px, size, velocity);
            end_pos = end_pos + base_px;

            // NB: keep this in sync with libclient/predict.rs  predict()
            if dur > DURATION_MAX as i32 {
                let offset = end_pos - start_pos;
                end_pos = start_pos + offset * scalar(DURATION_MAX as i32) / scalar(dur);
                dur = DURATION_MAX as i32;
            } else if dur == 0 {
                dur = DURATION_MAX as i32;
            }

            Ok(Motion {
                start_time: now,
                duration: dur as Duration,
                start_pos: start_pos,
                end_pos: end_pos,
            })
        }));

        self.with_world(|wf| {
            let data = wf.world().data();
            let mut e = wf.entity_mut(eid);

            // Compute extra information for the entity.
            let velocity = e.target_velocity();
            let dir = velocity.signum();
            // TODO: player speed handling shouldn't be here
            let speed = velocity.abs().max() / 50;

            let facing = 
                if dir != scalar(0) {
                    dir
                } else {
                    e.facing()
                };

            let anim = match e.activity() {
                Activity::Move => {
                    const ANIM_DIR_COUNT: AnimId = 8;
                    static SPEED_NAME_MAP: [&'static str; 4] = ["stand", "walk", "", "run"];
                    let idx = (3 * (facing.x + 1) + (facing.y + 1)) as usize;
                    let anim_dir = [2, 2, 2, 3, 0, 1, 0, 0, 0][idx];
                    let anim_name = format!("pony//{}-{}",
                                            SPEED_NAME_MAP[speed as usize],
                                            anim_dir);
                    data.animations.get_id(&anim_name)
                },
                Activity::Special(anim, _) => anim,
            };

            e.set_anim(anim);
            e.set_facing(facing);
            e.set_motion(motion);
        });
    */
        Ok(())
    }
}
