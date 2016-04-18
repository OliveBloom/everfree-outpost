use std::prelude::v1::*;
use std::collections::btree_map::{self, BTreeMap};
use std::collections::BinaryHeap;
use std::collections::Bound;
use std::cmp::Ordering;
use std::ops::Index;

use physics::v3::{V3, scalar};

use Time;


pub type EntityId = u32;

#[derive(Clone, Debug)]
pub struct Motion {
    pub start_pos: V3,
    pub end_pos: V3,
    pub start_time: Time,
    pub end_time: Time,
    pub anim_id: u16,
}

pub struct Entity {
    pub motion: Motion,
    pub appearance: u32,
    pub name: Option<String>,
    serial: u32,
}

impl Entity {
    pub fn pos(&self, now: Time) -> V3 {
        let delta = now - self.motion.start_time;
        let dur = self.motion.end_time - self.motion.start_time;
        if delta <= 0 {
            return self.motion.start_pos;
        } else if delta >= dur {
            return self.motion.end_pos;
        } else {
            let offset = (self.motion.end_pos - self.motion.start_pos) *
                scalar(delta) / scalar(dur);
            self.motion.start_pos + offset
        }
    }
}

struct Update {
    when: Time,
    entity: EntityId,
    serial: u32,
    motion: Motion,
}

impl PartialEq for Update {
    fn eq(&self, other: &Update) -> bool {
        self.when == other.when
    }
}

impl Eq for Update {}

impl PartialOrd for Update {
    fn partial_cmp(&self, other: &Update) -> Option<Ordering> {
        // Reverse the ordering on times, so that the BinaryHeap acts as a min-heap instead of a
        // max-heap.
        other.when.partial_cmp(&self.when)
    }
}

impl Ord for Update {
    fn cmp(&self, other: &Update) -> Ordering {
        other.when.cmp(&self.when)
    }
}

pub struct Entities {
    map: BTreeMap<EntityId, Entity>,
    updates: BinaryHeap<Update>,
    next_serial: u32,
}

impl Entities {
    pub fn new() -> Entities {
        Entities {
            map: BTreeMap::new(),
            updates: BinaryHeap::new(),
            next_serial: 0,
        }
    }

    fn next_serial(&mut self) -> u32 {
        let val = self.next_serial;
        self.next_serial = self.next_serial.wrapping_add(1);
        val
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn insert(&mut self,
                  id: EntityId,
                  appearance: u32,
                  name: Option<String>) {
        let serial = self.next_serial();
        self.map.insert(id, Entity {
            motion: Motion {
                start_pos: scalar(0),
                end_pos: scalar(0),
                start_time: 0,
                end_time: 1,
                anim_id: 0,
            },
            appearance: appearance,
            name: name,
            serial: serial,
        });
    }

    pub fn remove(&mut self, id: EntityId) -> Entity {
        self.map.remove(&id).unwrap()
    }

    pub fn schedule_update(&mut self, id: EntityId, when: Time, motion: Motion) {
        if let Some(e) = self.map.get(&id) {
            self.updates.push(Update {
                when: when,
                entity: id,
                serial: e.serial,
                motion: motion,
            });
        }
    }

    pub fn apply_updates(&mut self, now: Time) {
        while self.updates.len() > 0 && self.updates.peek().unwrap().when <= now {
            let update = self.updates.pop().unwrap();
            if let Some(e) = self.map.get_mut(&update.entity) {
                if e.serial != update.serial {
                    // The entity was replaced and its ID was reused since the update was
                    // scheduled.
                    continue;
                }

                e.motion = update.motion;
            }
        }
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
