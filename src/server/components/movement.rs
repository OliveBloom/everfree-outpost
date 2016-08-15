use std::cmp;
use std::collections::hash_map::{self, HashMap};
use std::collections::VecDeque;

use types::*;
use libcommon_movement::{self, MovingEntity, Activity};
use libcommon_proto::types::{LocalOffset, LocalTime};
use libphysics::ShapeSource;

use components::{Component, EngineComponents};
use data::Data;
use input::{InputBits, INPUT_DIR_MASK};
use timing::TICK_MS;
use world::{Entity, Motion};
use world::object::*;


#[derive(Clone, Debug)]
enum Event {
    Start(V3),
    Update(LocalOffset, InputBits),
    Blocked,
}

const QUEUE_SIZE: usize = 8;

struct Entry {
    time: Time,
    event: Event,
}

pub struct EntityMovement {
    /// Info on the entity's current movement.
    me: MovingEntity,

    /// Queue of pending path change events.
    buf: VecDeque<Entry>,

    /// Time of the most recently *queued* (not processed) PathStart event.
    time_base: Time,

    /// Current expected motion.
    cur_motion: Motion,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Change {
    None,
    Motion,
    Input,
    Conflict,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Update {
    None,
    Start,
    End,
    StartEnd,
    Conflict,
}

impl Update {
    fn new_motion(self) -> bool {
        match self {
            Update::Start | Update::StartEnd | Update::Conflict => true,
            Update::None | Update::End => false,
        }
    }
}

impl EntityMovement {
    fn new(now: Time, pos: V3) -> EntityMovement {
        EntityMovement {
            me: MovingEntity::new(),
            buf: VecDeque::with_capacity(QUEUE_SIZE),
            time_base: 0,
            cur_motion: Motion::stationary(pos, now),
        }
    }

    pub fn cur_input(&self) -> InputBits {
        self.me.input()
    }

    /// Apply queued changes with timestamps prior to `now`.
    pub fn process(&mut self, now: Time, pos: V3) -> Change {
        let mut change = Change::None;
        // About the timing: we may process some events that happen after this tick but before the
        // next tick.  This means we process PathBlocked events in time for the update() that
        // detects the obstacle server-side.  PathStart/PathUpdate events only happen on exact
        // tick boundaries, so we'll still never process one of those early.
        while self.buf.front().map_or(false, |e| e.time < now + TICK_MS) {
            let Entry { time, event } = self.buf.pop_front().unwrap();
            match event {
                Event::Start(expect_pos) => {
                    if pos != expect_pos {
                        info!("initial conflict: {:?} != {:?}", pos, expect_pos);
                        return Change::Conflict;
                    }
                    self.cur_motion = Motion::stationary(pos, now);
                    self.me.set_input(InputBits::empty());
                    // time_base is set when the event is queued, not when it's processed.
                },

                Event::Update(vel16, input) => {
                    let cur_pos = self.cur_motion.pos(time);
                    self.cur_motion = Motion {
                        start_pos: cur_pos,
                        velocity: vel16.to_global(),
                        start_time: time,
                        end_time: None,
                    };
                    self.me.set_input(input);

                    change = cmp::max(change, Change::Input);
                },

                Event::Blocked => {
                    self.cur_motion.end_time = Some(time);
                    change = cmp::max(change, Change::Motion);
                },
            }
        }

        change
    }

    pub fn update<S>(&mut self,
                     mut e: ObjectRefMut<Entity>,
                     now: Time,
                     data: &Data,
                     shape: &S) -> Update
            where S: ShapeSource {
        let mut ue = UpdateEntity::new(e.motion().clone(), e.facing());
        self.me.update(&mut ue, shape, now, now + TICK_MS);

        if ue.started || ue.ended {
            if ue.motion != self.cur_motion {
                info!("CONFLICT: {:?} != {:?}", ue.motion, self.cur_motion);
                // Entity is about to move off the path.  Stop right here instead.
                *e.motion_mut() = Motion::stationary(e.pos(now), now);
                let anim = facing_anim(data, e.facing(), 0);
                e.set_anim(anim);
                return Update::Conflict;
            }
        }

        if ue.started || ue.ended {
            e.set_motion(ue.motion);
        }
        if ue.started {
            e.set_facing(ue.facing);
            let anim = facing_anim(data, ue.facing, ue.speed);
            e.set_anim(anim);
        }
        match (ue.started, ue.ended) {
            (false, false) => Update::None,
            (true, false) => Update::Start,
            (false, true) => Update::End,
            (true, true) => Update::StartEnd,
        }
    }

    pub fn done(&self) -> bool {
        self.buf.len() == 0 &&
        (self.me.input() & INPUT_DIR_MASK).is_empty() &&
        self.cur_motion.velocity == scalar(0)
    }
}


struct UpdateEntity {
    motion: Motion,
    facing: V3,
    started: bool,
    ended: bool,
    speed: u8,
}

impl UpdateEntity {
    fn new(motion: Motion, facing: V3) -> UpdateEntity {
        UpdateEntity {
            motion: motion,
            facing: facing,
            started: false,
            ended: false,
            speed: 0,
        }
    }
}

impl libcommon_movement::Entity for UpdateEntity {
    fn activity(&self) -> Activity {
        // This code only runs when e.activity() == Walk
        Activity::Walk
    }

    fn facing(&self) -> V3 {
        self.facing
    }

    fn velocity(&self) -> V3 {
        self.motion.velocity
    }

    
    type Time = Time;

    fn pos(&self, now: Time) -> V3 {
        self.motion.pos(now)
    }


    fn start_motion(&mut self, now: Time, pos: V3, facing: V3, speed: u8, velocity: V3) {
        self.started = true;
        self.motion = Motion {
            start_pos: pos,
            velocity: velocity,
            start_time: now,
            end_time: None,
        };
        self.facing = facing;
        self.speed = speed;
    }

    fn end_motion(&mut self, now: Time) {
        self.ended = true;
        self.motion.end_time = Some(now);
    }
}

pub fn facing_anim(data: &Data, facing: V3, speed: u8) -> AnimId {
    const ANIM_DIR_COUNT: AnimId = 8;
    static SPEED_NAME_MAP: [&'static str; 4] = ["stand", "walk", "", "run"];
    let idx = (3 * (facing.x + 1) + (facing.y + 1)) as usize;
    let anim_dir = [2, 2, 2, 3, 0, 1, 0, 0, 0][idx];
    let anim_name = format!("pony//{}-{}",
                            SPEED_NAME_MAP[speed as usize],
                            anim_dir);
    data.animations.get_id(&anim_name)
}


pub struct Movement {
    map: HashMap<EntityId, EntityMovement>,
}

impl Component<Entity> for Movement {
    fn get<'a>(eng: &'a EngineComponents) -> &'a Self {
        &eng.movement
    }

    fn get_mut<'a>(eng: &'a mut EngineComponents) -> &'a mut Self {
        &mut eng.movement
    }

    fn cleanup(&mut self, id: EntityId) {
        // Does the right thing whether or not `id` is present.
        self.map.remove(&id);
    }
}

impl Movement {
    pub fn new() -> Movement {
        Movement {
            map: HashMap::new(),
        }
    }

    pub fn init(&mut self, id: EntityId, now: Time, pos: V3) -> &mut EntityMovement {
        if !self.map.contains_key(&id) {
            self.map.insert(id, EntityMovement::new(now, pos));
        }
        self.get(id)
    }

    pub fn clear(&mut self, id: EntityId) {
        self.map.remove(&id);
    }

    pub fn get(&mut self, id: EntityId) -> &mut EntityMovement {
        self.map.get_mut(&id)
            .unwrap_or_else(|| panic!("no EntityMovement for {:?}", id))
    }

    pub fn iter(&mut self) -> hash_map::IterMut<EntityId, EntityMovement> {
        self.map.iter_mut()
    }

    pub fn queue_start(&mut self,
                       now: Time,
                       id: EntityId,
                       expect_pos: V3,
                       delay: u16) {
        let em = self.init(id, now, expect_pos);
        // Delay the change until after the last pending event.
        let target = now + delay as Time;
        let time = em.buf.back().map_or(target, |e| cmp::max(e.time, target));
        let time = (time + TICK_MS - 1) & !(TICK_MS - 1);
        trace!("record queue_start at {}", time);

        // Keep a bound on the size of the queue.  Anything past the bound will be dropped.  This
        // is okay since it will just turn into a desync and the client will get a ResetMotion.
        if em.buf.len() < QUEUE_SIZE {
            em.buf.push_back(Entry {
                time: time,
                event: Event::Start(expect_pos),
            });
        }
        // Update time_base, since it will be used to compute the time of the following events.
        em.time_base = time;
    }

    pub fn queue_update(&mut self,
                        now: Time,
                        id: EntityId,
                        rel_time: LocalTime,
                        velocity: V3,
                        input: InputBits) {
        let em = unwrap_or!(self.map.get_mut(&id));
        let time = em.time_base + rel_time.to_global_64(now - em.time_base);
        trace!("record queue_update at {}", time);

        if em.buf.len() < QUEUE_SIZE {
            em.buf.push_back(Entry {
                time: time,
                event: Event::Update(LocalOffset::from_global(velocity), input),
            });
        }
    }

    pub fn queue_blocked(&mut self,
                         now: Time,
                         id: EntityId,
                         rel_time: LocalTime) {
        let em = unwrap_or!(self.map.get_mut(&id));
        let time = em.time_base + rel_time.to_global_64(now - em.time_base);
        trace!("record queue_blocked at {}", time);

        if em.buf.len() < QUEUE_SIZE {
            em.buf.push_back(Entry {
                time: time,
                event: Event::Blocked,
            });
        }
    }
}
