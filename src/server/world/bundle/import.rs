use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;

use types::*;

use data::Data;
use world::bundle::types as b;
use world::object::*;
use world::types::{Item, EntityAttachment, StructureAttachment, InventoryAttachment};
use world as w;
use world::fragment::Fragment;

use super::export::{BAD_CLIENT_ID, BAD_ENTITY_ID, BAD_INVENTORY_ID};
use super::export::{BAD_PLANE_ID, BAD_TERRAIN_CHUNK_ID, BAD_STRUCTURE_ID};


pub struct Importer<'d> {
    data: &'d Data,

    anim_id_map: Vec<AnimId>,
    item_id_map: Vec<ItemId>,
    block_id_map: Vec<BlockId>,
    template_id_map: Vec<TemplateId>,

    client_id_map: Vec<ClientId>,
    entity_id_map: Vec<EntityId>,
    inventory_id_map: Vec<InventoryId>,
    plane_id_map: Vec<PlaneId>,
    terrain_chunk_id_map: Vec<TerrainChunkId>,
    structure_id_map: Vec<StructureId>,
}

impl<'d> Importer<'d> {
    pub fn new(data: &'d Data) -> Importer<'d> {
        Importer {
            data: data,

            anim_id_map: Vec::new(),
            item_id_map: Vec::new(),
            block_id_map: Vec::new(),
            template_id_map: Vec::new(),

            client_id_map: Vec::new(),
            entity_id_map: Vec::new(),
            inventory_id_map: Vec::new(),
            plane_id_map: Vec::new(),
            terrain_chunk_id_map: Vec::new(),
            structure_id_map: Vec::new(),
        }
    }

    fn init_id_maps<F: Fragment<'d>>(&mut self, b: &b::Bundle, f: &mut F) {
        let d = self.data;

        // TODO: error handling?
        self.anim_id_map = b.anims.iter()
            .map(|s| d.animations.get_id(&s)).collect();
        self.item_id_map = b.items.iter()
            .map(|s| d.item_data.get_id(&s)).collect();
        self.block_id_map = b.blocks.iter()
            .map(|s| d.block_data.get_id(&s)).collect();
        self.template_id_map = b.templates.iter()
            .map(|s| d.structure_templates.get_id(&s)).collect();

        self.client_id_map = (0 .. b.clients.len())
            .map(|_| w::ops::client::create_unchecked(f)).collect();
        self.entity_id_map = (0 .. b.entities.len())
            .map(|_| w::ops::entity::create_unchecked(f)).collect();
        self.inventory_id_map = (0 .. b.inventories.len())
            .map(|_| w::ops::inventory::create_unchecked(f)).collect();
        self.plane_id_map = (0 .. b.planes.len())
            .map(|_| w::ops::plane::create_unchecked(f)).collect();
        self.terrain_chunk_id_map = (0 .. b.terrain_chunks.len())
            .map(|_| w::ops::terrain_chunk::create_unchecked(f)).collect();
        self.structure_id_map = (0 .. b.structures.len())
            .map(|_| w::ops::structure::create_unchecked(f)).collect();
    }

    pub fn import_anim_id(&self, id: AnimId) -> AnimId {
        self.anim_id_map[id as usize]
    }

    pub fn import_item_id(&self, id: ItemId) -> ItemId {
        self.item_id_map[id as usize]
    }

    pub fn import_block_id(&self, id: BlockId) -> BlockId {
        self.block_id_map[id as usize]
    }

    pub fn import_template_id(&self, id: TemplateId) -> TemplateId {
        self.template_id_map[id as usize]
    }

    pub fn import<I: Import>(&self, i: &I) -> I {
        i.import_from(self)
    }

    pub fn import_iter<'a, I, It, R>(&self, it: It) -> R
            where I: Import,
                  It: Iterator<Item=&'a I>,
                  R: FromIterator<I> {
        it.map(|i| self.import(i)).collect()
    }

    fn add_client_raw(&self, w: &mut w::World, idx: ClientId, b: &b::Client) {
        let id = self.import(&idx);
        w.clients.set_stable_id(id, b.stable_id).unwrap();
        let c = &mut w.clients[id];

        c.name = b.name.clone();
        let pawn = self.import(&b.pawn);
        c.pawn = if pawn != Some(BAD_ENTITY_ID) { pawn } else { None };

        c.stable_id = b.stable_id;
        c.child_entities = self.import_iter(b.child_entities.iter());
        c.child_inventories = self.import_iter(b.child_inventories.iter());
    }

    fn add_entity_raw(&self, w: &mut w::World, idx: EntityId, b: &b::Entity) {
        let id = self.import(&idx);
        w.entities.set_stable_id(id, b.stable_id).unwrap();
        let e = &mut w.entities[id];

        e.stable_plane = b.stable_plane;

        e.motion = b.motion.clone();
        e.anim = self.import_anim_id(b.anim);
        e.facing = b.facing;
        e.target_velocity = b.target_velocity;
        e.appearance = b.appearance;

        e.extra = self.import(&b.extra);
        e.stable_id = b.stable_id;
        e.attachment = self.import(&b.attachment);
        e.child_inventories = self.import_iter(b.child_inventories.iter());
    }

    fn add_inventory_raw(&self, w: &mut w::World, idx: InventoryId, b: &b::Inventory) {
        let id = self.import(&idx);
        w.inventories.set_stable_id(id, b.stable_id).unwrap();
        let i = &mut w.inventories[id];

        i.contents = self.import_iter::<_, _, Vec<_>>(b.contents.iter()).into_boxed_slice();

        i.stable_id = b.stable_id;
        i.attachment = self.import(&b.attachment);
    }

    fn add_plane_raw(&self, w: &mut w::World, idx: PlaneId, b: &b::Plane) {
        let id = self.import(&idx);
        w.planes.set_stable_id(id, b.stable_id).unwrap();
        let p = &mut w.planes[id];

        p.name = b.name.clone();

        p.saved_chunks = b.saved_chunks.iter()
                          .map(|&(k, v)| (k, v))
                          .collect();

        p.stable_id = b.stable_id;
    }

    fn add_terrain_chunk_raw(&self, w: &mut w::World, idx: TerrainChunkId, b: &b::TerrainChunk) {
        let id = self.import(&idx);
        w.terrain_chunks.set_stable_id(id, b.stable_id).unwrap();
        let tc = &mut w.terrain_chunks[id];

        let mut blocks = Box::new([0; CHUNK_TOTAL]);
        for (i, &b) in b.blocks.iter().enumerate() {
            blocks[i] = self.import_block_id(b);
        }

        tc.stable_plane = b.stable_plane;
        tc.cpos = b.cpos;
        tc.blocks = blocks;

        tc.stable_id = b.stable_id;
        tc.flags = b.flags;
        tc.child_structures = self.import_iter(b.child_structures.iter());
    }

    fn add_structure_raw(&self, w: &mut w::World, idx: StructureId, b: &b::Structure) {
        let id = self.import(&idx);
        w.structures.set_stable_id(id, b.stable_id).unwrap();
        let s = &mut w.structures[id];

        s.stable_plane = b.stable_plane;
        s.pos = b.pos;
        s.template = self.import_template_id(b.template);

        s.stable_id = b.stable_id;
        s.flags = b.flags;
        s.attachment = self.import(&b.attachment);
        s.child_inventories = self.import_iter(b.child_inventories.iter());
    }
}


pub trait Import {
    /// Transform `self` for import.  This is mainly important for IDs (and values containing IDs),
    /// which need to be remapped.
    fn import_from(&self, i: &Importer) -> Self;
}

impl Import for ClientId {
    fn import_from(&self, i: &Importer) -> ClientId {
        *i.client_id_map.get(self.unwrap() as usize).unwrap_or(&BAD_CLIENT_ID)
    }
}

impl Import for EntityId {
    fn import_from(&self, i: &Importer) -> EntityId {
        *i.entity_id_map.get(self.unwrap() as usize).unwrap_or(&BAD_ENTITY_ID)
    }
}

impl Import for InventoryId {
    fn import_from(&self, i: &Importer) -> InventoryId {
        *i.inventory_id_map.get(self.unwrap() as usize).unwrap_or(&BAD_INVENTORY_ID)
    }
}

impl Import for PlaneId {
    fn import_from(&self, i: &Importer) -> PlaneId {
        *i.plane_id_map.get(self.unwrap() as usize).unwrap_or(&BAD_PLANE_ID)
    }
}

impl Import for TerrainChunkId {
    fn import_from(&self, i: &Importer) -> TerrainChunkId {
        *i.terrain_chunk_id_map.get(self.unwrap() as usize).unwrap_or(&BAD_TERRAIN_CHUNK_ID)
    }
}

impl Import for StructureId {
    fn import_from(&self, i: &Importer) -> StructureId {
        *i.structure_id_map.get(self.unwrap() as usize).unwrap_or(&BAD_STRUCTURE_ID)
    }
}

impl<I: Import> Import for Option<I> {
    fn import_from(&self, i: &Importer) -> Option<I> {
        if let Some(ref val) = *self {
            Some(val.import_from(i))
        } else {
            None
        }
    }
}

impl Import for Item {
    fn import_from(&self, i: &Importer) -> Item {
        match *self {
            Item::Empty => Item::Empty,
            Item::Bulk(count, id) => Item::Bulk(count, i.import_item_id(id)),
            Item::Special(extra, id) => Item::Special(extra, i.import_item_id(id)),
        }
    }
}

impl Import for EntityAttachment {
    fn import_from(&self, i: &Importer) -> EntityAttachment {
        match *self {
            EntityAttachment::World => EntityAttachment::World,
            EntityAttachment::Chunk => EntityAttachment::Chunk,
            EntityAttachment::Client(id) => EntityAttachment::Client(i.import(&id)),
        }
    }
}

impl Import for StructureAttachment {
    fn import_from(&self, i: &Importer) -> StructureAttachment {
        match *self {
            StructureAttachment::Plane => StructureAttachment::Plane,
            StructureAttachment::Chunk => StructureAttachment::Chunk,
        }
    }
}

impl Import for InventoryAttachment {
    fn import_from(&self, i: &Importer) -> InventoryAttachment {
        match *self {
            InventoryAttachment::World => InventoryAttachment::World,
            InventoryAttachment::Client(id) => InventoryAttachment::Client(i.import(&id)),
            InventoryAttachment::Entity(id) => InventoryAttachment::Entity(i.import(&id)),
            InventoryAttachment::Structure(id) => InventoryAttachment::Structure(i.import(&id)),
        }
    }
}
