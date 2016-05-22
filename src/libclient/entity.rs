use std::prelude::v1::*;
use std::collections::btree_map::{self, BTreeMap};
use std::collections::VecDeque;
use std::collections::Bound;
use std::cmp::Ordering;
use std::ops::Index;

use physics::v3::{V3, scalar};

use Time;


pub type EntityId = u32;

#[derive(Clone, Debug)]
pub struct Motion {
    pub start_pos: V3,
    pub velocity: V3,
    pub start_time: Time,
    pub end_time: Option<Time>,
    pub anim_id: u16,
}

impl Motion {
    pub fn new() -> Motion {
        Motion {
            start_pos: scalar(0),
            velocity: scalar(0),
            start_time: 0,
            end_time: None,
            anim_id: 0,
        }
    }

    pub fn pos(&self, now: Time) -> V3 {
        if now <= self.start_time {
            return self.start_pos;
        }
        let now = match self.end_time {
            Some(end_time) if now > end_time => end_time,
            _ => now,
        };

        let delta = now - self.start_time;
        let offset = self.velocity * scalar(delta) / scalar(1000);
        self.start_pos + offset
    }

    pub fn apply(&mut self, update: Update) {
        match update {
            Update::MotionStart(time, pos, velocity, anim) =>
                self.apply_start(time, pos, velocity, anim),
            Update::MotionEnd(time) =>
                self.apply_end(time),
        }
    }

    pub fn apply_start(&mut self, start_time: Time, start_pos: V3, velocity: V3, anim: u16) {
        *self = Motion {
            start_pos: start_pos,
            velocity: velocity,
            start_time: start_time,
            end_time: None,
            anim_id: anim,
        };
    }

    pub fn apply_end(&mut self, end_time: Time) {
        self.end_time = Some(end_time);
    }
}

pub struct Entity {
    pub motion: Motion,
    pub appearance: u32,
    pub name: Option<String>,
}

impl Entity {
    pub fn pos(&self, now: Time) -> V3 {
        self.motion.pos(now)
    }

    pub fn anim(&self) -> u16 {
        self.motion.anim_id
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Update {
    MotionStart(Time, V3, V3, u16),
    MotionEnd(Time),
}

impl Update {
    pub fn when(&self) -> Time {
        match *self {
            Update::MotionStart(start_time, _, _, _) => start_time,
            Update::MotionEnd(end_time) => end_time,
        }
    }
}

pub struct Entities {
    map: BTreeMap<EntityId, Entity>,
    updates: VecDeque<(EntityId, Update)>,
}

impl Entities {
    pub fn new() -> Entities {
        Entities {
            map: BTreeMap::new(),
            updates: VecDeque::new(),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn insert(&mut self,
                  id: EntityId,
                  appearance: u32,
                  name: Option<String>) {
        self.map.insert(id, Entity {
            motion: Motion::new(),
            appearance: appearance,
            name: name,
        });
    }

    pub fn remove(&mut self, id: EntityId) -> Entity {
        self.map.remove(&id).unwrap()
    }

    pub fn schedule_motion_start(&mut self,
                                 id: EntityId,
                                 start_time: Time,
                                 start_pos: V3,
                                 velocity: V3,
                                 anim: u16) {
        if self.map.contains_key(&id) {
            let update = Update::MotionStart(start_time, start_pos, velocity, anim);
            self.updates.push_back((id, update));
        }
    }

    pub fn schedule_motion_end(&mut self,
                               id: EntityId,
                               end_time: Time) {
        if self.map.contains_key(&id) {
            self.updates.push_back((id, Update::MotionEnd(end_time)));
        }
    }

    pub fn apply_updates(&mut self, now: Time) {
        while self.updates.len() > 0 && self.updates.front().unwrap().1.when() <= now {
            let (eid, update) = self.updates.pop_front().unwrap();
            if let Some(e) = self.map.get_mut(&eid) {
                e.motion.apply(update);
            }
        }
    }

    pub fn ponyedit_hack(&mut self, id: EntityId, anim: u16) {
        let e = self.map.get_mut(&id).unwrap();
        e.motion.anim_id = anim;
        e.motion.start_pos = V3::new(4096, 4096, 0);
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.map.get(&id)
    }

    pub fn iter(&self) -> Iter {
        self.map.iter()
    }

    pub fn iter_from(&self, min: EntityId) -> RangeIter {
        self.map.range(Bound::Included(&min), Bound::Unbounded)
    }
}

pub type Iter<'a> = btree_map::Iter<'a, EntityId, Entity>;
pub type RangeIter<'a> = btree_map::Range<'a, EntityId, Entity>;

impl Index<EntityId> for Entities {
    type Output = Entity;
    fn index(&self, idx: EntityId) -> &Entity {
        &self.map[&idx]
    }
}
