use std::cmp;
use std::collections::HashMap;
use std::collections::VecDeque;

use types::*;

use components::{Component, EngineComponents};
use input::InputBits;
use world::{Entity, Motion};


enum Event {
    Start(V3),
    Update((i16, i16, i16), InputBits),
    Blocked,
}

const QUEUE_SIZE: usize = 8;

struct Entry {
    time: Time,
    event: Event,
}

pub struct MotionPath {
    buf: VecDeque<Entry>,
    cur_input: InputBits,
    cur_motion: Motion,
    time_base: Time,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Change {
    None,
    Motion,
    Input,
    Conflict,
}

impl MotionPath {
    fn new(now: Time, pos: V3) -> MotionPath {
        MotionPath {
            buf: VecDeque::new(),
            cur_input: InputBits::empty(),
            cur_motion: Motion::stationary(pos, now),
            time_base: 0,
        }
    }

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
                },

                Event::Update(vel16, input) => {
                    let cur_pos = self.cur_motion.pos(time);
                    self.cur_motion = Motion {
                        start_pos: cur_pos,
                        velocity: V3::new(vel16.0 as i32,
                                          vel16.1 as i32,
                                          vel16.2 as i32),
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
}


pub struct MotionPaths {
    map: HashMap<EntityId, MotionPath>,
}

impl Component<Entity> for MotionPaths {
    fn get<'a>(eng: &'a EngineComponents) -> &'a Self {
        &eng.motion_paths
    }

    fn get_mut<'a>(eng: &'a mut EngineComponents) -> &'a mut Self {
        &mut eng.motion_paths
    }

    fn cleanup(&mut self, id: EntityId) {
        // Does the right thing whether or not `id` is present.
        self.map.remove(&id);
    }
}

impl MotionPaths {
    pub fn new() -> MotionPaths {
        MotionPaths {
            map: HashMap::new(),
        }
    }

    pub fn init(&mut self, id: EntityId, now: Time, pos: V3) -> &mut MotionPath {
        self.map.insert(id, MotionPath::new(now, pos));
        self.get(id)
    }

    pub fn clear(&mut self, id: EntityId) {
        self.map.remove(&id);
    }

    pub fn get(&mut self, id: EntityId) -> &mut MotionPath {
        self.map.get_mut(&id)
            .unwrap_or_else(|| panic!("no MotionPath for {:?}", id))
    }
}
