use std::u8;

use types::*;

use input::InputBits;
use world::extra::Extra;
use world::flags::InventoryFlags;

pub use super::World;
pub use super::{Client, Entity, Inventory, Plane, TerrainChunk, Structure};

pub use libserver_world_types::{Item, Motion};
pub use libserver_world_types::{
    EntityAttachment,
    StructureAttachment,
    InventoryAttachment,
};


/// The `activity` field describes what an entity is currently doing.  This in turn decides which
/// system controls the entity's current `Motion` and animation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
    /// Walking.  The motion and animation are determined by the movement system.
    Walk,
    /// Playing the indicated animation, and otherwise stationary.  Can be interrupted by user
    /// input to switch back to `Walk`.
    Emote(AnimId),
    /// Playing the indicated animation, with the indicated activity bubble, and otherwise
    /// stationary.  Not interruptible.
    Work(AnimId, AnimId),
    /// Entity is teleporting.  This activity is used temporarily during the teleport, then it's
    /// switched back to `Walk`.
    Teleport,
}

impl Activity {
    pub fn interruptible(&self) -> bool {
        match *self {
            Activity::Walk |
            Activity::Emote(_) => true,
            Activity::Work(_, _) |
            Activity::Teleport => false,
        }
    }

    pub fn icon(&self, default: AnimId) -> AnimId {
        match *self {
            Activity::Walk |
            Activity::Emote(_) |
            Activity::Teleport => default,
            Activity::Work(_, icon) => icon,
        }
    }
}


impl super::Client {
    pub fn name(&self) -> &str {
        &*self.name
    }

    pub fn pawn_id(&self) -> Option<EntityId> {
        self.pawn
    }

    pub fn current_input(&self) -> InputBits {
        self.current_input
    }

    pub fn set_current_input(&mut self, new: InputBits) {
        self.current_input = new;
    }

    pub fn extra(&self) -> &Extra {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }
}

impl super::Entity {
    pub fn plane_id(&self) -> PlaneId {
        self.plane
    }

    pub fn stable_plane_id(&self) -> Stable<PlaneId> {
        self.stable_plane
    }

    pub fn activity(&self) -> Activity {
        self.activity
    }

    pub fn motion(&self) -> &Motion {
        &self.motion
    }

    pub fn motion_mut(&mut self) -> &mut Motion {
        &mut self.motion
    }

    pub fn anim(&self) -> AnimId {
        self.anim
    }

    pub fn set_anim(&mut self, new: AnimId) {
        self.anim = new;
    }

    pub fn facing(&self) -> V3 {
        self.facing
    }

    pub fn set_facing(&mut self, new: V3) {
        self.facing = new;
    }

    pub fn target_velocity(&self) -> V3 {
        self.target_velocity
    }

    pub fn set_target_velocity(&mut self, new: V3) {
        self.target_velocity = new;
    }

    pub fn appearance(&self) -> u32 {
        self.appearance
    }

    pub fn pos(&self, now: Time) -> V3 {
        self.motion.pos(now)
    }

    pub fn attachment(&self) -> EntityAttachment {
        self.attachment
    }

    pub fn extra(&self) -> &Extra {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }
}

impl super::Inventory {
    /// Count the number of items with the given ID.  The count may be as high as 255 * 255.
    pub fn count(&self, item_id: ItemId) -> u16 {
        let mut total = 0;
        for slot in &*self.contents {
            if slot.id == item_id {
                total += slot.count as u16;
            }
        }
        total
    }

    /// Count the amount of space remaining for storing items with the given ID.
    pub fn count_space(&self, item_id: ItemId) -> u16 {
        let mut total = 0;
        for slot in &*self.contents {
            if slot.id == item_id {
                total += (u8::MAX - slot.count) as u16;
            } else if slot.is_none() {
                total += u8::MAX as u16;
            }
        }
        total
    }

    pub fn contents(&self) -> &[Item] {
        &self.contents
    }

    pub fn contents_mut(&mut self) -> &mut [Item] {
        &mut self.contents
    }

    pub fn attachment(&self) -> InventoryAttachment {
        self.attachment
    }

    pub fn extra(&self) -> &Extra {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }

    pub fn flags(&self) -> InventoryFlags {
        self.flags
    }

    pub fn flags_mut(&mut self) -> &mut InventoryFlags {
        &mut self.flags
    }

    pub fn set_flags(&mut self, flags: InventoryFlags) {
        self.flags = flags;
    }
}

impl super::Plane {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_terrain_chunk_id(&self, cpos: V2) -> Option<TerrainChunkId> {
        self.loaded_chunks.get(&cpos).map(|&x| x)
    }

    pub fn terrain_chunk_id(&self, cpos: V2) -> TerrainChunkId {
        self.get_terrain_chunk_id(cpos).expect("no TerrainChunk at given pos")
    }

    pub fn get_saved_terrain_chunk_id(&self, cpos: V2) -> Option<Stable<TerrainChunkId>> {
        self.saved_chunks.get(&cpos).map(|&x| x)
    }

    pub fn saved_terrain_chunk_id(&self, cpos: V2) -> Stable<TerrainChunkId> {
        self.get_saved_terrain_chunk_id(cpos).expect("no TerrainChunk at given pos")
    }

    pub fn extra(&self) -> &Extra {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }
}

impl super::TerrainChunk {
    pub fn plane_id(&self) -> PlaneId {
        self.plane
    }

    pub fn chunk_pos(&self) -> V2 {
        self.cpos
    }

    pub fn block(&self, idx: usize) -> BlockId {
        self.blocks[idx]
    }

    pub fn blocks(&self) -> &BlockChunk {
        &*self.blocks
    }

    pub fn extra(&self) -> &Extra {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }
}

impl super::Structure {
    pub fn plane_id(&self) -> PlaneId {
        self.plane
    }

    pub fn pos(&self) -> V3 {
        self.pos
    }

    pub fn template_id(&self) -> TemplateId {
        self.template
    }

    pub fn attachment(&self) -> StructureAttachment {
        self.attachment
    }

    pub fn extra(&self) -> &Extra {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }
}


// Manual impl because BlockChunk (a.k.a. [u16; 4096]) is not Clone 
impl Clone for super::TerrainChunk {
    fn clone(&self) -> super::TerrainChunk {
        super::TerrainChunk {
            stable_plane: self.stable_plane,
            plane: self.plane,
            cpos: self.cpos,
            blocks: Box::new(*self.blocks),
            extra: self.extra.clone(),
            stable_id: self.stable_id,
            flags: self.flags,
            child_structures: self.child_structures.clone(),
            version: self.version,
        }
    }
}
