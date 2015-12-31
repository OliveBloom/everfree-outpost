use std::collections::HashMap;
use std::hash::Hash;

use types::*;

use data::Data;
use world::bundle::types as b;
use world::types::{Item, EntityAttachment, StructureAttachment, InventoryAttachment};
use world as w;


struct Remapper<Val, Id> {
    pub vals: Vec<Val>,
    pub id_map: HashMap<Id, Id>,
}

impl<Val, Id: Eq+Hash+Copy> Remapper<Val, Id> {
    fn new() -> Remapper<Val, Id> {
        Remapper {
            vals: Vec::new(),
            id_map: HashMap::new(),
        }
    }

    fn get_or<F>(&mut self, old_id: Id, f: F) -> Id
            where F: FnOnce(usize) -> (Id, Val) {
        if self.id_map.contains_key(&old_id) {
            *self.id_map.get(&old_id).unwrap()
        } else {
            let (new_id, val) = f(self.vals.len());
            self.vals.push(val);
            self.id_map.insert(old_id, new_id);
            new_id
        }
    }
}


pub struct Exporter<'d> {
    data: &'d Data,

    anims: Remapper<&'d str, AnimId>,
    items: Remapper<&'d str, ItemId>,
    blocks: Remapper<&'d str, BlockId>,
    templates: Remapper<&'d str, TemplateId>,

    clients: Remapper<Option<b::Client>, ClientId>,
    entities: Remapper<Option<b::Entity>, EntityId>,
    inventories: Remapper<Option<b::Inventory>, InventoryId>,
    planes: Remapper<Option<b::Plane>, PlaneId>,
    terrain_chunks: Remapper<Option<b::TerrainChunk>, TerrainChunkId>,
    structures: Remapper<Option<b::Structure>, StructureId>,
}

impl<'d> Exporter<'d> {
    pub fn new(data: &'d Data) -> Exporter<'d> {
        Exporter {
            data: data,

            anims: Remapper::new(),
            items: Remapper::new(),
            blocks: Remapper::new(),
            templates: Remapper::new(),

            clients: Remapper::new(),
            entities: Remapper::new(),
            inventories: Remapper::new(),
            planes: Remapper::new(),
            terrain_chunks: Remapper::new(),
            structures: Remapper::new(),
        }
    }

    pub fn export_anim_id(&mut self, id: AnimId) -> AnimId {
        let d = &self.data.animations;
        self.anims.get_or(id, |raw| (raw as AnimId,
                                     &d.animation(id).name))
    }

    pub fn export_item_id(&mut self, id: ItemId) -> ItemId {
        let d = &self.data.item_data;
        self.items.get_or(id, |raw| (raw as ItemId,
                                     d.name(id)))
    }

    pub fn export_block_id(&mut self, id: BlockId) -> BlockId {
        let d = &self.data.block_data;
        self.blocks.get_or(id, |raw| (raw as BlockId,
                                      d.name(id)))
    }

    pub fn export_template_id(&mut self, id: TemplateId) -> TemplateId {
        let d = &self.data.structure_templates;
        self.templates.get_or(id, |raw| (raw as TemplateId,
                                         &d.template(id).name))
    }

    pub fn export<E: Export>(&mut self, e: &E) -> E {
        e.export_to(self)
    }

    pub fn export_iter<'a, E: Export, I: Iterator<Item=&'a E>>(&mut self, i: I) -> Box<[E]> {
        i.map(|e| self.export(e)).collect::<Vec<_>>().into_boxed_slice()
    }

    pub fn finish(&self) -> b::Bundle {
        b::Bundle {
            anims: convert_str_vec(&self.anims.vals),
            items: convert_str_vec(&self.items.vals),
            blocks: convert_str_vec(&self.blocks.vals),
            templates: convert_str_vec(&self.templates.vals),

            clients: convert_opt_vec(&self.clients.vals),
            entities: convert_opt_vec(&self.entities.vals),
            inventories: convert_opt_vec(&self.inventories.vals),
            planes: convert_opt_vec(&self.planes.vals),
            terrain_chunks: convert_opt_vec(&self.terrain_chunks.vals),
            structures: convert_opt_vec(&self.structures.vals),
        }
    }

    pub fn add_client(&mut self, id: ClientId, c: &w::Client) {
        let idx = self.export(&id).unwrap() as usize;
        let b = b::Client {
            name: c.name.clone(),
            pawn: self.export(&c.pawn),

            stable_id: c.stable_id,
            child_entities: self.export_iter(c.child_entities.iter()),
            child_inventories: self.export_iter(c.child_inventories.iter()),
        };
        self.clients.vals[idx] = Some(b);
    }

    pub fn add_entity(&mut self, id: EntityId, e: &w::Entity) {
        let idx = self.export(&id).unwrap() as usize;
        let b = b::Entity {
            stable_plane: e.stable_plane,

            motion: e.motion.clone(),
            anim: self.export_anim_id(e.anim),
            facing: e.facing,
            target_velocity: e.target_velocity,
            appearance: e.appearance,

            extra: self.export(&e.extra),
            stable_id: e.stable_id,
            attachment: self.export(&e.attachment),
            child_inventories: self.export_iter(e.child_inventories.iter()),
        };
        self.entities.vals[idx] = Some(b);
    }

    pub fn add_inventory(&mut self, id: InventoryId, i: &w::Inventory) {
        let idx = self.export(&id).unwrap() as usize;
        let b = b::Inventory {
            contents: self.export_iter(i.contents.iter()),

            stable_id: i.stable_id,
            attachment: self.export(&i.attachment),
        };
        self.inventories.vals[idx] = Some(b);
    }

    pub fn add_plane(&mut self, id: PlaneId, p: &w::Plane) {
        let idx = self.export(&id).unwrap() as usize;
        let b = b::Plane {
            name: p.name.clone(),

            saved_chunks: p.saved_chunks.iter()
                           .map(|(k,v)| (*k, *v))
                           .collect::<Vec<_>>().into_boxed_slice(),

            stable_id: p.stable_id,
        };
        self.planes.vals[idx] = Some(b);
    }

    pub fn add_terrain_chunk(&mut self, id: TerrainChunkId, tc: &w::TerrainChunk) {
        let idx = self.export(&id).unwrap() as usize;

        let mut blocks = Box::new([0; CHUNK_TOTAL]);
        for (i, &b) in tc.blocks.iter().enumerate() {
            blocks[i] = self.export_block_id(b);
        }

        let b = b::TerrainChunk {
            stable_plane: tc.stable_plane,
            cpos: tc.cpos,
            blocks: blocks,

            stable_id: tc.stable_id,
            flags: tc.flags,
            child_structures: self.export_iter(tc.child_structures.iter()),
        };
        self.terrain_chunks.vals[idx] = Some(b);
    }

    pub fn add_structure(&mut self, id: StructureId, s: &w::Structure) {
        let idx = self.export(&id).unwrap() as usize;
        let b = b::Structure {
            stable_plane: s.stable_plane,
            pos: s.pos,
            template: self.export_template_id(s.template),

            stable_id: s.stable_id,
            flags: s.flags,
            attachment: self.export(&s.attachment),
            child_inventories: self.export_iter(s.child_inventories.iter()),
        };
        self.structures.vals[idx] = Some(b);
    }
}

fn convert_str_vec(v: &Vec<&str>) -> Box<[Box<str>]> {
    v.iter().map(|s| (*s).to_owned().into_boxed_slice())
     .collect::<Vec<_>>().into_boxed_slice()
}

fn convert_opt_vec<T: Clone>(v: &Vec<Option<T>>) -> Box<[T]> {
    v.iter().map(|x| x.as_ref().unwrap().clone())
     .collect::<Vec<_>>().into_boxed_slice()
}


pub trait Export {
    /// Transform `self` for export.  This is mainly important for IDs (and values containing IDs),
    /// which need to be remapped.
    fn export_to(&self, e: &mut Exporter) -> Self;
}

impl Export for ClientId {
    fn export_to(&self, e: &mut Exporter) -> ClientId {
        e.clients.get_or(*self, |raw| (ClientId(raw as u16), None))
    }
}

impl Export for EntityId {
    fn export_to(&self, e: &mut Exporter) -> EntityId {
        e.entities.get_or(*self, |raw| (EntityId(raw as u32), None))
    }
}

impl Export for InventoryId {
    fn export_to(&self, e: &mut Exporter) -> InventoryId {
        e.inventories.get_or(*self, |raw| (InventoryId(raw as u32), None))
    }
}

impl Export for PlaneId {
    fn export_to(&self, e: &mut Exporter) -> PlaneId {
        e.planes.get_or(*self, |raw| (PlaneId(raw as u32), None))
    }
}

impl Export for TerrainChunkId {
    fn export_to(&self, e: &mut Exporter) -> TerrainChunkId {
        e.terrain_chunks.get_or(*self, |raw| (TerrainChunkId(raw as u32), None))
    }
}

impl Export for StructureId {
    fn export_to(&self, e: &mut Exporter) -> StructureId {
        e.structures.get_or(*self, |raw| (StructureId(raw as u32), None))
    }
}

impl<E: Export> Export for Option<E> {
    fn export_to(&self, e: &mut Exporter) -> Option<E> {
        if let Some(ref val) = *self {
            Some(val.export_to(e))
        } else {
            None
        }
    }
}

impl Export for Item {
    fn export_to(&self, e: &mut Exporter) -> Item {
        match *self {
            Item::Empty => Item::Empty,
            Item::Bulk(count, id) => Item::Bulk(count, e.export_item_id(id)),
            Item::Special(extra, id) => Item::Special(extra, e.export_item_id(id)),
        }
    }
}

impl Export for EntityAttachment {
    fn export_to(&self, e: &mut Exporter) -> EntityAttachment {
        match *self {
            EntityAttachment::World => EntityAttachment::World,
            EntityAttachment::Chunk => EntityAttachment::Chunk,
            EntityAttachment::Client(id) => EntityAttachment::Client(e.export(&id)),
        }
    }
}

impl Export for StructureAttachment {
    fn export_to(&self, e: &mut Exporter) -> StructureAttachment {
        match *self {
            StructureAttachment::Plane => StructureAttachment::Plane,
            StructureAttachment::Chunk => StructureAttachment::Chunk,
        }
    }
}

impl Export for InventoryAttachment {
    fn export_to(&self, e: &mut Exporter) -> InventoryAttachment {
        match *self {
            InventoryAttachment::World => InventoryAttachment::World,
            InventoryAttachment::Client(id) => InventoryAttachment::Client(e.export(&id)),
            InventoryAttachment::Entity(id) => InventoryAttachment::Entity(e.export(&id)),
            InventoryAttachment::Structure(id) => InventoryAttachment::Structure(e.export(&id)),
        }
    }
}
