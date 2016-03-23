use std::collections::btree_map::{self, BTreeMap};
use std::collections::Bound;


#[derive(Clone, Copy)]
pub struct Structure {
    /// Structure position in tiles.  u8 is enough to cover the entire local region.
    pub pos: (u8, u8, u8),

    pub template_id: u32,

    /// Timestamp indicating when to start the structure's one-shot animation.  This field is only
    /// relevant if the structure's template defines such an animation.
    pub oneshot_start: u16,
}


pub struct Structures {
    map: BTreeMap<u32, Structure>,
}

impl Structures {
    pub fn new() -> Structures {
        Structures {
            map: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn insert(&mut self,
                  id: u32,
                  pos: (u8, u8, u8),
                  template_id: u32,
                  oneshot_start: u16) {
        self.map.insert(id, Structure {
            pos: pos,
            template_id: template_id,
            oneshot_start: oneshot_start,
        });
    }

    pub fn remove(&mut self, id: u32) {
        self.map.remove(&id);
    }

    pub fn replace(&mut self, id: u32, template_id: u32) {
        if let Some(s) = self.map.get_mut(&id) {
            s.template_id = template_id;
        }
    }

    pub fn iter(&self) -> Iter {
        self.map.iter()
    }

    pub fn iter_from(&self, min: u32) -> RangeIter {
        self.map.range(Bound::Included(&min), Bound::Unbounded)
    }
}

pub type Iter<'a> = btree_map::Iter<'a, u32, Structure>;
pub type RangeIter<'a> = btree_map::Range<'a, u32, Structure>;
