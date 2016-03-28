#![crate_name = "server_world_types"]

#[macro_use] extern crate bitflags;
extern crate server_types;
use server_types::*;


pub mod flags;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityAttachment {
    World,
    Chunk,
    Client(ClientId),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StructureAttachment {
    Plane,
    Chunk,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InventoryAttachment {
    World,
    Client(ClientId),
    Entity(EntityId),
    Structure(StructureId),
}


#[derive(Clone, Copy, Debug)]
pub enum Item {
    /// No item in this slot.
    Empty,
    /// Bulk item (stackable).  The `u8` is the item count in the stack, which should never be
    /// zero.  These items can be moved around, split, combined, etc. with no script intervention.
    Bulk(u8, ItemId),
    /// Special item (non-stackable).  This item has script data attached.  The `u8` is an
    /// identifier assigned by the script.  Moving this item to a different inventory requires
    /// script intervention.  (Moving within a container does not, because the table slot does not
    /// change.)
    Special(u8, ItemId),
}

impl Item {
    pub fn count(&self) -> u8 {
        use self::Item::*;
        match *self {
            Empty => 0,
            Bulk(count, _) => count,
            Special(_, _) => 1,
        }
    }

    pub fn item(&self) -> ItemId {
        use self::Item::*;
        match *self {
            Empty => NO_ITEM,
            Bulk(_, item_id) => item_id,
            Special(_, item_id) => item_id,
        }
    }
}


#[derive(Clone, Debug)]
pub struct Motion {
    pub start_time: Time,
    pub duration: Duration,
    pub start_pos: V3,
    pub end_pos: V3,
}

impl Motion {
    pub fn fixed(pos: V3) -> Motion {
        Motion {
            start_time: 0,
            duration: 0,
            start_pos: pos,
            end_pos: pos,
        }
    }

    pub fn stationary(pos: V3, now: Time) -> Motion {
        Motion {
            start_time: now,
            duration: -1_i64 as Duration,
            start_pos: pos,
            end_pos: pos,
        }
    }

    pub fn pos(&self, now: Time) -> V3 {
        if now <= self.start_time {
            self.start_pos
        } else {
            let delta = now - self.start_time;
            if delta >= self.duration as Time {
                self.end_pos
            } else {
                let offset = (self.end_pos - self.start_pos) *
                        scalar(delta as i32) / scalar(self.duration as i32);
                self.start_pos + offset
            }
        }
    }

    pub fn end_time(&self) -> Time {
        self.start_time + self.duration as Time
    }
}
