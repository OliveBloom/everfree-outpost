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


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Item {
    pub id: ItemId,
    pub count: u8,
}

impl Item {
    pub fn new(id: ItemId, count: u8) -> Item {
        Item { id: id, count: count }
    }

    pub fn none() -> Item {
        Item {
            id: NO_ITEM,
            count: 0,
        }
    }

    pub fn is_none(&self) -> bool {
        self.id == NO_ITEM
    }
}


#[derive(Clone, Debug)]
pub struct Motion {
    pub start_pos: V3,
    pub velocity: V3,
    pub start_time: Time,
    pub end_time: Option<Time>,
}

impl Motion {
    pub fn fixed(pos: V3) -> Motion {
        Motion {
            start_pos: pos,
            velocity: scalar(0),
            start_time: 0,
            end_time: None,
        }
    }

    pub fn stationary(pos: V3, now: Time) -> Motion {
        Motion {
            start_pos: pos,
            velocity: scalar(0),
            start_time: now,
            end_time: None,
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
        let offset = self.velocity * scalar(delta as i32) / scalar(1000);
        self.start_pos + offset
    }
}
