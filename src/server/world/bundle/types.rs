use types::*;

use world::extra::Extra;
use world::flags::{TerrainChunkFlags, StructureFlags};
use world::types::{Motion, Item};
use world::types::{EntityAttachment, InventoryAttachment, StructureAttachment};

pub struct Bundle {
    pub anims: Box<[Box<str>]>,
    pub items: Box<[Box<str>]>,
    pub blocks: Box<[Box<str>]>,
    pub templates: Box<[Box<str>]>,

    pub clients: Box<[Client]>,
    pub entities: Box<[Entity]>,
    pub inventories: Box<[Inventory]>,
    pub planes: Box<[Plane]>,
    pub terrain_chunks: Box<[TerrainChunk]>,
    pub structures: Box<[Structure]>,
}

pub struct Client {
    pub name: String,
    pub pawn: Option<EntityId>,
    // current_input: transient

    pub stable_id: StableId,
    pub child_entities: Box<[EntityId]>,
    pub child_inventories: Box<[InventoryId]>,
}

pub struct Entity {
    pub stable_plane: Stable<PlaneId>,
    // plane: transient

    pub motion: Motion,
    pub anim: AnimId,
    pub facing: V3,
    pub target_velocity: V3,
    pub appearance: u32,

    pub extra: Extra,
    pub stable_id: StableId,
    pub attachment: EntityAttachment,
    pub child_inventories: Box<[InventoryId]>,
}

pub struct Inventory {
    pub contents: Box<[Item]>,

    pub stable_id: StableId,
    pub attachment: InventoryAttachment,
}

pub struct Plane {
    pub name: String,

    // loaded_chunks: transient
    pub saved_chunks: Box<[(V2, Stable<TerrainChunkId>)]>,

    pub stable_id: StableId,
}

pub struct TerrainChunk {
    pub stable_plane: Stable<PlaneId>,
    pub cpos: V2,
    pub blocks: Box<BlockChunk>,

    pub stable_id: StableId,
    pub flags: TerrainChunkFlags,
    pub child_structures: Box<[StructureId]>,
}

pub struct Structure {
    pub stable_plane: Stable<PlaneId>,
    pub pos: V3,
    pub template: TemplateId,

    pub stable_id: StableId,
    pub flags: StructureFlags,
    pub attachment: StructureAttachment,
    pub child_inventories: Box<[InventoryId]>,
}


fn clone_slice<T: Clone>(x: &Box<[T]>) -> Box<[T]> {
    x.to_vec().into_boxed_slice()
}

impl Clone for Client {
    fn clone(&self) -> Client {
        Client {
            name: self.name.clone(),
            pawn: self.pawn,

            stable_id: self.stable_id,
            child_entities: clone_slice(&self.child_entities),
            child_inventories: clone_slice(&self.child_inventories),
        }
    }
}

impl Clone for Entity {
    fn clone(&self) -> Entity {
        Entity {
            stable_plane: self.stable_plane,

            motion: self.motion.clone(),
            anim: self.anim,
            facing: self.facing,
            target_velocity: self.target_velocity,
            appearance: self.appearance,

            extra: self.extra.clone(),
            stable_id: self.stable_id,
            attachment: self.attachment,
            child_inventories: clone_slice(&self.child_inventories),
        }
    }
}

impl Clone for Inventory {
    fn clone(&self) -> Inventory {
        Inventory {
            contents: clone_slice(&self.contents),

            stable_id: self.stable_id,
            attachment: self.attachment,
        }
    }
}

impl Clone for Plane {
    fn clone(&self) -> Plane {
        Plane {
            name: self.name.clone(),

            saved_chunks: clone_slice(&self.saved_chunks),

            stable_id: self.stable_id,
        }
    }
}

impl Clone for TerrainChunk {
    fn clone(&self) -> TerrainChunk {
        let mut blocks = Box::new([0; CHUNK_TOTAL]);
        for (i, &b) in self.blocks.iter().enumerate() {
            blocks[i] = b;
        }

        TerrainChunk {
            stable_plane: self.stable_plane,
            cpos: self.cpos,
            blocks: blocks,

            stable_id: self.stable_id,
            flags: self.flags,
            child_structures: clone_slice(&self.child_structures),
        }
    }
}

impl Clone for Structure {
    fn clone(&self) -> Structure {
        Structure {
            stable_plane: self.stable_plane,
            pos: self.pos,
            template: self.template,

            stable_id: self.stable_id,
            flags: self.flags,
            attachment: self.attachment,
            child_inventories: clone_slice(&self.child_inventories),
        }
    }
}
