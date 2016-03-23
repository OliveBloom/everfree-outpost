use std::prelude::v1::*;
use std::ops::{Deref, DerefMut};

pub use self::geom::{Vertex, GeomGen};


pub mod geom;


#[derive(Clone, Copy)]
pub struct Structure {
    /// Structure position in tiles.  u8 is enough to cover the entire local region.
    pub pos: (u8, u8, u8),

    pub external_id: u32,

    pub template_id: u16,

    /// Timestamp indicating when to start the structure's one-shot animation.  This field is only
    /// relevant if the structure's template defines such an animation.
    pub oneshot_start: u16,
}


pub struct Buffer {
    storage: Vec<Structure>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            storage: Vec::new(),
        }
    }

    pub fn insert(&mut self,
                  external_id: u32,
                  pos: (u8, u8, u8),
                  template_id: u32) -> usize {
        self.storage.push(Structure {
            pos: pos,
            external_id: external_id,
            template_id: template_id as u16,
            oneshot_start: 0,
        });
        self.storage.len() - 1
    }

    pub fn remove(&mut self,
                  idx: usize) -> u32 {
        // Do a sort of `swap_remove`, except we don't need to return the old value.
        self.storage[idx] = self.storage[self.storage.len() - 1];
        self.storage.pop();

        // Return the external ID of the structure that now occupies slot `idx`, so the caller can
        // update their records.
        self.storage[idx].external_id
    }
}

impl Deref for Buffer {
    type Target = [Structure];

    fn deref(&self) -> &[Structure] {
        &self.storage
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut [Structure] {
        &mut self.storage
    }
}
