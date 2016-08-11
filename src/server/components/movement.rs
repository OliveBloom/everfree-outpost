use std::cmp;
use std::collections::hash_map::{self, HashMap};
use std::collections::VecDeque;

use types::*;
use libcommon_movement::MovingEntity;
use libcommon_proto::types::{LocalOffset, LocalTime};

use components::{Component, EngineComponents};
use input::{InputBits, INPUT_DIR_MASK};
use world::{Entity, Motion};


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

    /// Current input.
    cur_input: InputBits,
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

impl EntityMovement {
    fn new(now: Time, pos: V3) -> EntityMovement {
        EntityMovement {
            me: MovingEntity::new(),
            buf: VecDeque::with_capacity(QUEUE_SIZE),
            time_base: 0,
            cur_input: InputBits::empty(),
            cur_motion: Motion::stationary(pos, now),
        }
    }

    /// Apply queued changes with timestamps prior to `now`.
    pub fn process(&mut self, now: Time, pos: V3) -> Change {
        let mut change = Change::None;
        while self.buf.front().map_or(false, |e| e.time <= now) {
            let Entry { time, event } = self.buf.pop_front().unwrap();
            match event {
                Event::Start(expect_pos) => {
                    if pos != expect_pos {
                        return Change::Conflict;
                    }
                    self.cur_motion = Motion::stationary(pos, now);
                    self.cur_input = InputBits::empty();
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
                    self.cur_input = input;

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

    pub fn done(&self) -> bool {
        self.buf.len() == 0 &&
        (self.cur_input & INPUT_DIR_MASK).is_empty() &&
        self.cur_motion.velocity == scalar(0)
    }
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
        let time = rel_time.to_global_64(now - em.time_base);
        
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
        let time = rel_time.to_global_64(now - em.time_base);
        
        if em.buf.len() < QUEUE_SIZE {
            em.buf.push_back(Entry {
                time: time,
                event: Event::Blocked,
            });
        }
    }
}
