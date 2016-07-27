use std::iter::FromIterator;

use types::*;

use data::Data;
use world::bundle::types as b;
use world::object::*;
use world::types::{Item, EntityAttachment, StructureAttachment, InventoryAttachment};
use world as w;
use world::World;
use world::ops;
use world::extra::{self, Extra};

use super::AnyId;
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

    pub fn import_iter<'a, I: 'a, It, R>(&self, it: It) -> R
            where I: Import,
                  It: Iterator<Item=&'a I>,
                  R: FromIterator<I> {
        it.map(|i| self.import(i)).collect()
    }


    /// Populate the ID maps.  For objects, this means creating empty instances with
    /// `create_unchecked`.
    fn init_id_maps(&mut self, w: &mut World<'d>, b: &b::Bundle) {
        let d = self.data;

        self.anim_id_map = b.anims.iter()
            .map(|s| d.animations.get_id(&s)).collect();
        self.item_id_map = b.items.iter()
            .map(|s| d.item_data.get_id(&s)).collect();
        self.block_id_map = b.blocks.iter()
            .map(|s| d.block_data.get_id(&s)).collect();
        self.template_id_map = b.templates.iter()
            .map(|s| d.structure_templates.get_id(&s)).collect();

        self.client_id_map = (0 .. b.clients.len())
            .map(|_| ops::client::create_unchecked(w)).collect();
        self.entity_id_map = (0 .. b.entities.len())
            .map(|_| ops::entity::create_unchecked(w)).collect();
        self.inventory_id_map = (0 .. b.inventories.len())
            .map(|_| ops::inventory::create_unchecked(w)).collect();
        self.plane_id_map = (0 .. b.planes.len())
            .map(|_| ops::plane::create_unchecked(w)).collect();
        self.terrain_chunk_id_map = (0 .. b.terrain_chunks.len())
            .map(|_| ops::terrain_chunk::create_unchecked(w)).collect();
        self.structure_id_map = (0 .. b.structures.len())
            .map(|_| ops::structure::create_unchecked(w)).collect();
    }


    // Functions to populate an object from the bundled version.

    fn init_world(&self, w: &mut w::World, b: &b::World) {
        w.clients.set_next_id(b.next_client);
        w.entities.set_next_id(b.next_entity);
        w.inventories.set_next_id(b.next_inventory);
        w.planes.set_next_id(b.next_plane);
        w.terrain_chunks.set_next_id(b.next_terrain_chunk);
        w.structures.set_next_id(b.next_structure);

        w.extra = self.import(&b.extra);

        // child_entities and child_inventories aren't stored directly
    }

    fn init_client(&self, w: &mut w::World, id: ClientId, b: &b::Client) {
        w.clients.set_stable_id(id, b.stable_id).unwrap();
        let c = &mut w.clients[id];

        c.name = b.name.to_string();
        let pawn = self.import(&b.pawn);
        c.pawn = if pawn != Some(BAD_ENTITY_ID) { pawn } else { None };

        c.extra = self.import(&b.extra);
        c.stable_id = b.stable_id;
        c.child_entities = self.import_iter(b.child_entities.iter());
        c.child_inventories = self.import_iter(b.child_inventories.iter());
    }

    fn init_entity(&self, w: &mut w::World, id: EntityId, b: &b::Entity) {
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

    fn init_inventory(&self, w: &mut w::World, id: InventoryId, b: &b::Inventory) {
        w.inventories.set_stable_id(id, b.stable_id).unwrap();
        let i = &mut w.inventories[id];

        i.contents = self.import_iter::<_, _, Vec<_>>(b.contents.iter()).into_boxed_slice();

        i.extra = self.import(&b.extra);
        i.stable_id = b.stable_id;
        i.flags = b.flags;
        i.attachment = self.import(&b.attachment);
    }

    fn init_plane(&self, w: &mut w::World, id: PlaneId, b: &b::Plane) {
        w.planes.set_stable_id(id, b.stable_id).unwrap();
        let p = &mut w.planes[id];

        p.name = b.name.to_string();

        p.saved_chunks = b.saved_chunks.iter()
                          .map(|&(k, v)| (k, v))
                          .collect();

        p.extra = self.import(&b.extra);
        p.stable_id = b.stable_id;
    }

    fn init_terrain_chunk(&self, w: &mut w::World, id: TerrainChunkId, b: &b::TerrainChunk) {
        w.terrain_chunks.set_stable_id(id, b.stable_id).unwrap();
        let tc = &mut w.terrain_chunks[id];

        tc.stable_plane = b.stable_plane;
        tc.plane = w.planes.get_id(b.stable_plane).unwrap();
        tc.cpos = b.cpos;
        for (i, &b) in b.blocks.iter().enumerate() {
            tc.blocks[i] = self.import_block_id(b);
        }

        tc.extra = self.import(&b.extra);
        tc.stable_id = b.stable_id;
        tc.flags = b.flags;
        tc.child_structures = self.import_iter(b.child_structures.iter());
    }

    fn init_structure(&self, w: &mut w::World, id: StructureId, b: &b::Structure) {
        w.structures.set_stable_id(id, b.stable_id).unwrap();
        let s = &mut w.structures[id];

        s.stable_plane = b.stable_plane;
        s.plane = w.planes.get_id(b.stable_plane).unwrap();
        s.pos = b.pos;
        s.template = self.import_template_id(b.template);

        s.extra = self.import(&b.extra);
        s.stable_id = b.stable_id;
        s.flags = b.flags;
        s.attachment = self.import(&b.attachment);
        s.child_inventories = self.import_iter(b.child_inventories.iter());
    }


    fn add_client(&self, w: &mut World<'d>, idx: ClientId, b: &b::Client) {
        let id = self.import(&idx);
        self.init_client(w, id, b);
        //ops::client::post_init(w, id);
    }

    fn add_entity(&self, w: &mut World<'d>, idx: EntityId, b: &b::Entity) {
        let id = self.import(&idx);
        self.init_entity(w, id, b);
        ops::entity::post_init(w, id);
    }

    fn add_inventory(&self, w: &mut World<'d>, idx: InventoryId, b: &b::Inventory) {
        let id = self.import(&idx);
        self.init_inventory(w, id, b);
        //ops::inventory::post_init(w, id);
    }

    fn add_plane(&self, w: &mut World<'d>, idx: PlaneId, b: &b::Plane) {
        let id = self.import(&idx);
        self.init_plane(w, id, b);
        ops::plane::post_init(w, id);
    }

    fn add_terrain_chunk(&self, w: &mut World<'d>, idx: TerrainChunkId, b: &b::TerrainChunk) {
        let id = self.import(&idx);
        self.init_terrain_chunk(w, id, b);
        ops::terrain_chunk::post_init(w, id);
    }

    fn add_structure(&self, w: &mut World<'d>, idx: StructureId, b: &b::Structure) {
        let id = self.import(&idx);
        self.init_structure(w, id, b);
        ops::structure::post_init(w, id);
    }

    fn add_bundle(&self, w: &mut World<'d>, b: &b::Bundle) {
        // Do all the imports
        // NB: b.world is handled separately, because ordinary save bundles shouldn't have it set
        for (i, c) in b.clients.iter().enumerate() {
            self.add_client(w, ClientId(i as u16), c);
        }
        for (i, e) in b.entities.iter().enumerate() {
            self.add_entity(w, EntityId(i as u32), e);
        }
        for (i, inv) in b.inventories.iter().enumerate() {
            self.add_inventory(w, InventoryId(i as u32), inv);
        }
        for (i, p) in b.planes.iter().enumerate() {
            self.add_plane(w, PlaneId(i as u32), p);
        }
        for (i, tc) in b.terrain_chunks.iter().enumerate() {
            self.add_terrain_chunk(w, TerrainChunkId(i as u32), tc);
        }
        for (i, s) in b.structures.iter().enumerate() {
            self.add_structure(w, StructureId(i as u32), s);
        }
    }


    pub fn import_bundle(&mut self, w: &mut World<'d>, b: &b::Bundle) {
        // TODO: validate bundle
        self.init_id_maps(w, b);
        self.add_bundle(w, b);
    }

    pub fn import_world(&mut self, w: &mut World<'d>, b: &b::Bundle) {
        // TODO: validate bundle
        let bw = b.world.as_ref().unwrap() as &b::World;
        self.init_id_maps(w, b);
        self.init_world(w, bw);
        self.add_bundle(w, b);
    }

    pub fn iter_imports<F>(&self, mut f: F)
            where F: FnMut(AnyId) {
        for &id in &self.client_id_map {
            f(AnyId::Client(id));
        }
        for &id in &self.entity_id_map {
            f(AnyId::Entity(id));
        }
        for &id in &self.inventory_id_map {
            f(AnyId::Inventory(id));
        }
        for &id in &self.plane_id_map {
            f(AnyId::Plane(id));
        }
        for &id in &self.terrain_chunk_id_map {
            f(AnyId::TerrainChunk(id));
        }
        for &id in &self.structure_id_map {
            f(AnyId::Structure(id));
        }
    }

    pub fn visit_imports<V: Visitor>(&self, b: &b::Bundle, v: &mut V) {
        for (idx, &id) in self.client_id_map.iter().enumerate() {
            v.visit_client(id, &b.clients[idx]);
        }
        for (idx, &id) in self.entity_id_map.iter().enumerate() {
            v.visit_entity(id, &b.entities[idx]);
        }
        for (idx, &id) in self.inventory_id_map.iter().enumerate() {
            v.visit_inventory(id, &b.inventories[idx]);
        }
        for (idx, &id) in self.plane_id_map.iter().enumerate() {
            v.visit_plane(id, &b.planes[idx]);
        }
        for (idx, &id) in self.terrain_chunk_id_map.iter().enumerate() {
            v.visit_terrain_chunk(id, &b.terrain_chunks[idx]);
        }
        for (idx, &id) in self.structure_id_map.iter().enumerate() {
            v.visit_structure(id, &b.structures[idx]);
        }
    }
}

pub trait Visitor {
    fn visit_client(&mut self, id: ClientId, b: &b::Client);
    fn visit_entity(&mut self, id: EntityId, b: &b::Entity);
    fn visit_inventory(&mut self, id: InventoryId, b: &b::Inventory);
    fn visit_plane(&mut self, id: PlaneId, b: &b::Plane);
    fn visit_terrain_chunk(&mut self, id: TerrainChunkId, b: &b::TerrainChunk);
    fn visit_structure(&mut self, id: StructureId, b: &b::Structure);
}

pub fn import_bundle<'d>(w: &mut World<'d>, b: &b::Bundle) -> Importer<'d> {
    let mut importer = Importer::new(w.data());
    importer.import_bundle(w, b);
    importer
}

pub fn import_world<'d>(w: &mut World<'d>, b: &b::Bundle) -> Importer<'d> {
    let mut importer = Importer::new(w.data());
    importer.import_world(w, b);
    importer
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
        Item::new(i.import_item_id(self.id), self.count)
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
    fn import_from(&self, _i: &Importer) -> StructureAttachment {
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


impl Import for Extra {
    fn import_from(&self, i: &Importer) -> Extra {
        let mut result = Extra::new();
        for (k, v) in self.iter() {
            match v {
                extra::View::Value(v) =>
                    result.set(k, i.import(&v)),
                extra::View::Array(a) =>
                    import_extra_array(i, a, result.set_array(k)),
                extra::View::Hash(h) =>
                    import_extra_hash(i, h, result.set_hash(k)),
            }
        }
        result
    }
}

impl Import for extra::Value {
    fn import_from(&self, i: &Importer) -> extra::Value {
        use world::extra::Value::*;
        match *self {
            Null => Null,
            Bool(b) => Bool(b),
            Int(i) => Int(i),
            Float(f) => Float(f),
            Str(ref s) => Str(s.clone()),

            ClientId(id) => ClientId(i.import(&id)),
            EntityId(id) => EntityId(i.import(&id)),
            InventoryId(id) => InventoryId(i.import(&id)),
            PlaneId(id) => PlaneId(i.import(&id)),
            TerrainChunkId(id) => TerrainChunkId(i.import(&id)),
            StructureId(id) => StructureId(i.import(&id)),

            StableClientId(id) => StableClientId(id),
            StableEntityId(id) => StableEntityId(id),
            StableInventoryId(id) => StableInventoryId(id),
            StablePlaneId(id) => StablePlaneId(id),
            StableTerrainChunkId(id) => StableTerrainChunkId(id),
            StableStructureId(id) => StableStructureId(id),

            V2(v) => V2(v),
            V3(v) => V3(v),
            Region2(r) => Region2(r),
            Region3(r) => Region3(r),
        }
    }
}

fn import_extra_array(i: &Importer,
                      array: extra::ArrayView,
                      mut result: extra::ArrayViewMut) {
    for (idx, v) in array.iter().enumerate() {
        result.borrow().push();
        match v {
            extra::View::Value(v) =>
                result.borrow().set(idx, i.import(&v)),
            extra::View::Array(a) =>
                import_extra_array(i, a, result.borrow().set_array(idx)),
            extra::View::Hash(h) =>
                import_extra_hash(i, h, result.borrow().set_hash(idx)),
        }
    }
}

fn import_extra_hash(i: &Importer,
                     hash: extra::HashView,
                     mut result: extra::HashViewMut) {
    for (k, v) in hash.iter() {
        match v {
            extra::View::Value(v) =>
                result.borrow().set(k, i.import(&v)),
            extra::View::Array(a) =>
                import_extra_array(i, a, result.borrow().set_array(k)),
            extra::View::Hash(h) =>
                import_extra_hash(i, h, result.borrow().set_hash(k)),
        }
    }
}
