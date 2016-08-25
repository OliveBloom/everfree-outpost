use types::*;
use std::collections::btree_map::{self, BTreeMap};
use std::collections::Bound;
use std::ops::Index;

use physics::v3::V3;


#[derive(Clone, Copy)]
pub struct Structure {
    /// Structure position in tiles.  u8 is enough to cover the entire local region.
    pub pos: (u8, u8, u8),

    pub template_id: TemplateId,

    /// Timestamp indicating when to start the structure's one-shot animation.  This field is only
    /// relevant if the structure's template defines such an animation.
    pub oneshot_start: u16,
}

impl Structure {
    pub fn pos(&self) -> V3 {
        V3::new(self.pos.0 as i32,
                self.pos.1 as i32,
                self.pos.2 as i32)
    }
}


pub struct Structures {
    map: BTreeMap<StructureId, Structure>,
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
                  id: StructureId,
                  pos: (u8, u8, u8),
                  template_id: u32,
                  oneshot_start: u16) {
        self.map.insert(id, Structure {
            pos: pos,
            template_id: template_id,
            oneshot_start: oneshot_start,
        });
    }

    pub fn remove(&mut self, id: StructureId) -> Structure {
        self.map.remove(&id).unwrap()
    }

    pub fn replace(&mut self,
                   id: StructureId,
                   template_id: u32,
                   oneshot_start: u16) {
        if let Some(s) = self.map.get_mut(&id) {
            s.template_id = template_id;
            s.oneshot_start = oneshot_start;
        }
    }

    pub fn iter(&self) -> Iter {
        self.map.iter()
    }

    pub fn iter_from(&self, min: StructureId) -> RangeIter {
        self.map.range(Bound::Included(&min), Bound::Unbounded)
    }
}

pub type Iter<'a> = btree_map::Iter<'a, StructureId, Structure>;
pub type RangeIter<'a> = btree_map::Range<'a, StructureId, Structure>;

impl Index<StructureId> for Structures {
    type Output = Structure;
    fn index(&self, idx: StructureId) -> &Structure {
        &self.map[&idx]
    }
}
