use std::prelude::v1::*;
use std::mem;
use types::*;
use common_movement::{InputBits, INPUT_DIR_MASK};
use common_movement::{self, MovingEntity};
use common_proto::types::{LocalPos, LocalOffset, LocalTime};
use common_proto::game::Request;
use physics::ShapeSource;
use physics::v3::V3;

use data::Data;
use entity::{self, Entity, Entities, Motion};
use platform::Platform;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
    Walk,
    // Fly,
    Emote,
    Work,
}

// This must match the server tick.
pub const TICK_MS: Time = 32;

pub struct PawnInfo {
    id: Option<EntityId>,

    // Shadow state for the real pawn Entity.  The actual values sent from the server are saved
    // here, so we can overwrite the data in Entities but still restore it if the pawn changes.
    name: Option<String>, 
    motion: Motion,
    // TODO: should really keep a queue of pending updates, like Entities does

    activity: Activity,
    movement: Movement,
}

impl PawnInfo {
    pub fn new() -> PawnInfo {
        PawnInfo {
            id: None,

            name: None,
            motion: Motion::new(),

            activity: Activity::Walk,
            movement: Movement::new(),
        }
    }

    #[allow(dead_code)]
    pub fn id(&self) -> Option<EntityId> {
        self.id
    }

    pub fn is(&self, id: EntityId) -> bool {
        Some(id) == self.id
    }

    pub fn get<'a>(&self, entities: &'a Entities) -> Option<&'a Entity> {
        self.id.and_then(|eid| entities.get(eid))
    }


    pub fn set_id(&mut self, id: EntityId, entities: &mut Entities) {
        if self.id.is_some() {
            self.clear_id(entities);
        }
        self.id = Some(id);

        if let Some(e) = entities.get_mut(id) {
            self.acquire(e);
        }
    }

    pub fn clear_id(&mut self, entities: &mut Entities) {
        if self.id.is_none() {
            return;
        }

        let id = self.id.unwrap();
        self.id = None;

        if let Some(e) = entities.get_mut(id) {
            self.release(e);
        }
    }

    pub fn on_create(&mut self, entities: &mut Entities) {
        if let Some(id) = self.id {
            let e = entities.get_mut(id)
                .expect("entity should exist before calling on_create");
            self.acquire(e);
        }
    }


    fn acquire(&mut self, e: &mut Entity) {
        self.name = mem::replace(&mut e.name, None);
        self.motion = e.motion.clone();
        self.movement.motion = e.motion.clone();
    }

    fn release(&mut self, e: &mut Entity) {
        e.name = mem::replace(&mut self.name, None);
        e.motion = self.motion.clone();
    }


    pub fn update_movement<S, P>(&mut self,
                                 now: Time,
                                 data: &Data,
                                 shape: &S,
                                 platform: &mut P,
                                 entities: &mut Entities) 
            where S: ShapeSource,
                  P: Platform {
        while self.movement.last_tick() <= now {
            self.movement.run_physics(data, shape, &mut self.activity);
            let changed = self.movement.process_changes(platform);
            if changed {
                trace!("changed pawn motion @ {}", self.movement.last_tick());
                if let Some(e) = self.id.and_then(|id| entities.get_mut(id)) {
                    e.motion = self.movement.motion.clone();
                }
            }
        }
    }

    pub fn init_time(&mut self, now: Time) {
        self.movement.init_time(now);
    }

    pub fn set_input(&mut self, bits: InputBits) {
        self.movement.set_input(bits);
    }

    pub fn set_activity(&mut self, activity: Activity) {
        let sc = activity != Activity::Walk;
        self.movement.set_server_controlled(sc);
        if sc {
            self.movement.reset_motion(&self.motion);
        }
        self.activity = activity;
    }

    pub fn reset_motion(&mut self) {
        self.movement.reset_motion(&self.motion);
    }

    pub fn server_update(&mut self, update: entity::Update) {
        self.motion.apply(update);
        self.movement.server_update(update);
    }
}


struct Movement {
    last_tick: Time,
    me: MovingEntity,
    motion: Motion,
    updates: Vec<entity::Update>,
    motion_changed: bool,
    path_active: bool,
    server_controlled: bool,
    time_base: Time,
}

impl Movement {
    fn new() -> Movement {
        Movement {
            last_tick: 0,
            me: MovingEntity::new(),
            motion: Motion::new(),
            updates: Vec::new(),
            motion_changed: false,
            path_active: false,
            server_controlled: true,
            time_base: 0,
        }
    }

    pub fn last_tick(&self) -> Time {
        self.last_tick
    }

    pub fn init_time(&mut self, now: Time) {
        self.last_tick = now & !(TICK_MS - 1);
    }

    pub fn set_input(&mut self, bits: InputBits) {
        println!("set input to {:?}", bits);
        self.me.set_input(bits);
    }

    pub fn set_server_controlled(&mut self, flag: bool) {
        self.server_controlled = flag;
        if flag {
            self.path_active = false;
        }
    }

    pub fn reset_motion(&mut self, motion: &Motion) {
        self.motion = motion.clone();
        self.motion_changed = true;
        self.path_active = false;
        self.me.force_update();
    }

    pub fn server_update(&mut self, update: entity::Update) {
        if self.server_controlled {
            self.motion.apply(update);
            self.motion_changed = true;
        }
    }

    pub fn run_physics<S>(&mut self,
                          data: &Data,
                          shape: &S,
                          activity: &mut Activity)
            where S: ShapeSource {
        if !(self.me.input() & INPUT_DIR_MASK).is_empty() &&
           *activity == Activity::Emote {
            *activity = Activity::Walk;
        }

        if *activity == Activity::Walk {
            let mut e = EntityImpl {
                data: data,
                activity: *activity,
                motion: &mut self.motion,
                updates: &mut self.updates,
            };
            let now = self.last_tick;
            let next = now + TICK_MS;
            self.me.update(&mut e, shape, now, next);
        }
        self.last_tick += TICK_MS;
    }

    pub fn process_changes<P>(&mut self,
                              platform: &mut P) -> bool
            where P: Platform {
        let mut changed = self.motion_changed;
        let input = self.me.input();
        for up in mem::replace(&mut self.updates, Vec::new()) {
            changed = true;
            self.server_controlled = false;
            match up {
                entity::Update::MotionStart(time, pos, velocity, _anim) => {
                    let dir_input = !(input & INPUT_DIR_MASK).is_empty();
                    println!("START: {:?}, {:?}, {:?}, active {}",
                             pos, velocity, input, self.path_active);

                    // Maybe start a path
                    let old_pos = pos;
                    let pos = LocalPos::from_global(pos);
                    let velocity = LocalOffset::from_global(velocity);
                    if !self.path_active {
                        // Don't start a new path if the player isn't actually moving somewhere.
                        if dir_input {
                            platform.send_message(Request::PathStart(
                                    pos, 64, velocity, input.bits()));
                            self.time_base = time;
                            self.path_active = true;
                        }
                    } else {
                        // Update the path
                        let rel_time = LocalTime::from_global_32(time - self.time_base);
                        platform.send_message(Request::PathUpdate(
                                rel_time, velocity, input.bits()));

                        // If the player has stopped moving, that's the end of the path.
                        if !dir_input {
                            self.path_active = false;
                        }
                    }
                },

                entity::Update::MotionEnd(time) => {
                    if self.path_active {
                        let rel_time = LocalTime::from_global_32(time - self.time_base);
                        platform.send_message(Request::PathBlocked(rel_time));
                    }
                },
            }
        }

        self.motion_changed = false;
        changed
    }
}


struct EntityImpl<'a> {
    data: &'a Data,
    activity: Activity,
    motion: &'a mut Motion,
    updates: &'a mut Vec<entity::Update>,
}

impl<'a> common_movement::Entity for EntityImpl<'a> {
    fn activity(&self) -> common_movement::Activity {
        match self.activity {
            Activity::Walk => common_movement::Activity::Walk,
            //Activity::Fly => common_movement::Activity::Fly,
            Activity::Emote |
            Activity::Work => common_movement::Activity::Busy,
        }
    }

    fn facing(&self) -> V3 {
        let dir = self.data.anim_dir(self.motion.anim_id).unwrap_or(0);
        let (x, y) = [
            ( 1,  0),
            ( 1,  1),
            ( 0,  1),
            (-1,  1),
            (-1,  0),
            (-1, -1),
            ( 0, -1),
            ( 1, -1),
        ][dir as usize];
        V3::new(x, y, 0)
    }

    fn velocity(&self) -> V3 { self.motion.velocity }


    type Time = Time;

    fn pos(&self, now: Time) -> V3 { self.motion.pos(now) }


    fn start_motion(&mut self, now: Time, pos: V3, facing: V3, speed: u8, velocity: V3) {
        let anim = walk_anim(self.data, facing, speed);
        *self.motion = Motion {
            start_pos: pos,
            velocity: velocity,
            start_time: now,
            end_time: None,
            anim_id: anim,
        };
        self.updates.push(entity::Update::MotionStart(now, pos, velocity, anim));
    }

    fn end_motion(&mut self, now: Time) {
        self.motion.end_time = Some(now);
        self.updates.push(entity::Update::MotionEnd(now));
    }

}

fn walk_anim(data: &Data, facing: V3, speed: u8) -> u16 {
    let idx = (3 * (facing.x + 1) + (facing.y + 1)) as usize;
    let dir = [5, 4, 3, 6, 0, 2, 7, 0, 1][idx];

    data.physics_anim_table()[speed as usize][dir as usize]
}
