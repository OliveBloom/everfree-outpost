//! Interface to the physics engine.  The physics engine itself lives in a separate library,
//! `libphysics`, so that it can be compiled to asm.js for use on the client.  This system just
//! provides the glue to connect the physics engine to entities and the rest of the `World`.
use std::collections::hash_map::{self, HashMap};

use libcommon_movement as movement;
use libcommon_movement::{InputBits, MovingEntity};
use libphysics::ShapeSource;
use libphysics::{CHUNK_SIZE, CHUNK_BITS, CHUNK_MASK};

use types::*;
use util::Coroutine;

use cache::TerrainCache;
use data::Data;
use timing::next_tick;
use world::{World, Entity};
use world::{Motion, Activity};
use world::object::*;


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

    pub fn add_entity(&mut self, eid: EntityId) {
        self.moving_entities.insert(eid, MovingEntity::new());
    }

    pub fn remove_entity(&mut self, eid: EntityId) {
        self.moving_entities.remove(&eid);
    }

    pub fn set_input(&mut self, eid: EntityId, input: InputBits) {
        let me = self.moving_entities.entry(eid).or_insert_with(MovingEntity::new);
        me.set_input(input);
    }

    pub fn force_update(&mut self, eid: EntityId) {
        let me = unwrap_or!(self.moving_entities.get_mut(&eid),
                            { warn!("no such MovingEntity: {:?}", eid); return; });
        me.force_update();
    }
}


pub struct UpdateCo<'a, 'd: 'a> {
    data: &'d Data,
    now: Time,
    inner: hash_map::IterMut<'a, EntityId, MovingEntity>,
}

impl<'d> Physics<'d> {
    pub fn update<'a>(&'a mut self, now: Time) -> UpdateCo<'a, 'd> {
        UpdateCo {
            data: self.data,
            now: now,
            inner: self.moving_entities.iter_mut(),
        }
    }
}

struct EntityWrapper<'a, 'd: 'a> {
    entity: ObjectRefMut<'a, 'd, Entity>,
    started: bool,
    ended: bool,
}

impl<'a, 'd> EntityWrapper<'a, 'd> {
    fn new(entity: ObjectRefMut<'a, 'd, Entity>) -> EntityWrapper<'a, 'd> {
        EntityWrapper {
            entity: entity,
            started: false,
            ended: false,
        }
    }
}

impl<'a, 'd> movement::Entity for EntityWrapper<'a, 'd> {
    fn activity(&self) -> movement::Activity {
        match self.entity.activity() {
            Activity::Walk => movement::Activity::Walk,
            Activity::Emote(_) |
            Activity::Work(_, _) => movement::Activity::Busy,
        }
    }

    fn facing(&self) -> V3 {
        self.entity.facing()
    }

    fn velocity(&self) -> V3 {
        self.entity.motion().velocity
    }

    
    type Time = Time;

    fn pos(&self, now: Time) -> V3 {
        self.entity.pos(now)
    }


    fn start_motion(&mut self, now: Time, pos: V3, facing: V3, speed: u8, velocity: V3) {
        self.started = true;
        *self.entity.motion_mut() = Motion {
            start_pos: pos,
            velocity: velocity,
            start_time: now,
            end_time: None,
        };
        self.entity.set_facing(facing);
        let data = self.entity.world().data();
        self.entity.set_anim(facing_anim(data, facing, speed as usize));
    }

    fn end_motion(&mut self, now: Time) {
        self.ended = true;
        self.entity.motion_mut().end_time = Some(now);
    }
}

impl<'a, 'b, 'd> Coroutine<(&'b mut World<'d>, &'b TerrainCache)> for UpdateCo<'a, 'd> {
    type Item = (EntityId, Motion, AnimId, UpdateKind);

    fn send(&mut self, args: (&'b mut World<'d>, &'b TerrainCache)) -> Option<Self::Item> {
        let (world, cache) = args;

        for (&eid, me) in &mut self.inner {
            let now = self.now;
            let next = next_tick(now);

            let mut e = match world.get_entity_mut(eid) {
                Some(e) => EntityWrapper::new(e),
                None => {
                    error!("BUG: no such entity: {:?}", eid);
                    continue;
                },
            };

            let s = ChunksSource {
                cache: cache,
                base_tile: scalar(0),
                plane: e.entity.plane_id(),
            };


            me.update(&mut e, &s, now, next);


            let kind = match (e.started, e.ended) {
                (false, false) => UpdateKind::Move,
                (true, false) => UpdateKind::Start,
                (false, true) => UpdateKind::End,
                (true, true) => UpdateKind::StartEnd,
            };
            return Some((eid, e.entity.motion().clone(), e.entity.anim(), kind));
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
            entry.get(offset).shape()
        } else {
            Shape::Empty
        }
    }
}
