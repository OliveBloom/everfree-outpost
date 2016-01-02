use std::collections::HashMap;
use std::hash::Hash;

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
