use std::prelude::v1::*;
use types::*;
use std::collections::btree_map::{self, BTreeMap};
use std::collections::VecDeque;
use std::collections::Bound;
use std::marker::PhantomData;
use std::ops::Index;
use std::ptr;
use physics::v3::{V3, scalar};


#[derive(Clone, Debug)]
pub struct Motion {
    pub start_pos: V3,
    pub velocity: V3,
    pub start_time: Time,
    pub end_time: Option<Time>,
    pub anim_id: AnimId,
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

    pub fn apply_start(&mut self, start_time: Time, start_pos: V3, velocity: V3, anim: AnimId) {
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
    // Kind of redundant to include `id` here, but we need it to ensure stable sorting.
    pub id: EntityId,
    pub motion: Motion,
    pub appearance: u32,
    pub name: Option<String>,
    pub activity_anim: Option<AnimId>,

    z_next: *mut Entity,
}

impl Entity {
    pub fn pos(&self, now: Time) -> V3 {
        self.motion.pos(now)
    }

    pub fn anim(&self) -> AnimId {
        self.motion.anim_id
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Update {
    MotionStart(Time, V3, V3, AnimId),
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

    /// We maintain an intrusive linked list that presents the entities sorted by `y + z` (in other
    /// words, rendering depth).
    z_list: *mut Entity,
}

impl Entities {
    pub fn new() -> Entities {
        Entities {
            map: BTreeMap::new(),
            updates: VecDeque::new(),

            z_list: ptr::null_mut(),
        }
    }

    fn invalidate_z_list(&mut self) {
        self.z_list = ptr::null_mut();
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.invalidate_z_list();
    }

    pub fn insert(&mut self,
                  id: EntityId,
                  appearance: u32,
                  name: Option<String>) {
        use std::collections::btree_map::Entry::*;
        match self.map.entry(id) {
            Vacant(e) => {
                e.insert(Entity {
                    id: id,
                    motion: Motion::new(),
                    appearance: appearance,
                    name: name,
                    activity_anim: None,
                    z_next: ptr::null_mut(),
                });
            },
            Occupied(e) => {
                // The entity already exists, but its appearance may have changed.  (The
                // EntityAppear message is currently used for both "entity came into view" and
                // "entity's appearance has changed".)
                let e = e.into_mut();
                e.appearance = appearance;
                e.name = name;
            },
        }

        // Insertions may cause elements to move around
        self.invalidate_z_list();
    }

    pub fn remove(&mut self, id: EntityId) -> Entity {
        // Removals may cause elements to move around
        self.invalidate_z_list();

        self.map.remove(&id).unwrap()
    }

    pub fn schedule_motion_start(&mut self,
                                 id: EntityId,
                                 start_time: Time,
                                 start_pos: V3,
                                 velocity: V3,
                                 anim: AnimId) {
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

        // NB: We assume that get_mut never moves elements
    }

    pub fn set_activity_anim(&mut self,
                             id: EntityId,
                             anim: Option<AnimId>) {
        if let Some(e) = self.map.get_mut(&id) {
            e.activity_anim = anim;
        }
    }

    pub fn ponyedit_hack(&mut self, id: EntityId, anim: AnimId, pos: V3) {
        let e = self.map.get_mut(&id).unwrap();
        e.motion.anim_id = anim;
        e.motion.start_pos = pos;
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.map.get(&id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.map.get_mut(&id)
    }

    pub fn iter(&self) -> Iter {
        self.map.iter()
    }

    pub fn iter_from(&self, min: EntityId) -> RangeIter {
        self.map.range((Bound::Included(&min), Bound::Unbounded))
    }


    fn sort_full<F: Fn(&Entity) -> i32>(&mut self, calc_z: F) {
        let mut items = self.map.values().collect::<Vec<_>>();
        items.sort_by_key(|&e| (calc_z(e), e.id));
        unsafe {
            // The address of the `next` pointer of the previous entry.
            let mut last_ptr = &mut self.z_list;
            for e in items {
                let ptr = e as *const Entity as *mut Entity;
                *last_ptr = ptr;
                last_ptr = &mut (*ptr).z_next; 
            }
            *last_ptr = ptr::null_mut();
        }
    }

    fn check_sorted<F: Fn(&Entity) -> i32>(&mut self, calc_z: F) -> bool {
        if self.z_list.is_null() {
            if self.map.len() == 0 {
                return true;
            } else {
                return false;
            }
        }

        unsafe {
            let mut last_key = (calc_z(&*self.z_list), (*self.z_list).id);
            let mut cur = (*self.z_list).z_next;
            while !cur.is_null() {
                let cur_key = (calc_z(&*cur), (*cur).id);
                if cur_key < last_key {
                    return false;
                }
                last_key = cur_key;
                cur = (*cur).z_next;
            }

            true
        }
    }

    pub fn update_z_order<F: Fn(&Entity) -> i32>(&mut self, calc_z: F) {
        if !self.check_sorted(|e| calc_z(e)) {
            self.sort_full(calc_z);
        }
    }

    pub fn iter_z_order(&self) -> ZOrderIter {
        ZOrderIter {
            cur: self.z_list,
            _marker: PhantomData,
        }
    }

    pub fn iter_z_order_from(&self, eid: EntityId) -> ZOrderIter {
        let ptr = if self.z_list.is_null() {
            // List is invalid
            ptr::null_mut()
        } else if let Some(e) = self.map.get(&eid) {
            // Every entity should be somewhere in the list
            e as *const Entity as *mut Entity
        } else {
            // Entity not in self.map
            ptr::null_mut()
        };
        ZOrderIter {
            cur: ptr,
            _marker: PhantomData,
        }
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

pub struct ZOrderIter<'a> {
    cur: *mut Entity,
    _marker: PhantomData<&'a Entity>,
}

impl<'a> Iterator for ZOrderIter<'a> {
    type Item = &'a Entity;

    fn next(&mut self) -> Option<&'a Entity> {
        if self.cur.is_null() {
            None
        } else {
            unsafe {
                let e = self.cur;
                self.cur = (*e).z_next;
                Some(&*e)
            }
        }
    }
}
