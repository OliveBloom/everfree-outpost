use std::collections::hash_map::{self, HashMap};
use std::hash::Hash;
use std::iter;

use physics::CHUNK_SIZE;
use server_config::Data;
use server_extra::Extra;
use server_types::*;
use server_world_types::{Motion, Item};
use server_world_types::{EntityAttachment, InventoryAttachment, StructureAttachment};
use server_world_types::flags::{TerrainChunkFlags, StructureFlags};

use super::types::*;


pub struct Remapper<Val, Id> {
    pub vals: Vec<Val>,
    pub id_map: HashMap<Id, Id>,
}

impl<Val, Id: Eq+Hash+Copy> Remapper<Val, Id> {
    pub fn new() -> Remapper<Val, Id> {
        Remapper {
            vals: Vec::new(),
            id_map: HashMap::new(),
        }
    }

    pub fn get(&self, old_id: Id) -> Option<Id> {
        self.id_map.get(&old_id).map(|&x| x)
    }

    pub fn get_or<F>(&mut self, old_id: Id, f: F) -> Id
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

    pub fn keys(&self) -> hash_map::Keys<Id, Id> {
        self.id_map.keys()
    }

    pub fn iter(&self) -> RemapperIter<Val, Id> {
        RemapperIter {
            iter: self.id_map.iter(),
            vec: &self.vals
        }
    }

    pub fn iter_mut(&mut self) -> RemapperIterMut<Val, Id> {
        RemapperIterMut {
            iter: self.id_map.iter_mut(),
            vec: &mut self.vals
        }
    }
}

pub struct RemapperIter<'a, Val: 'a, Id: 'a> {
    iter: hash_map::Iter<'a, Id, Id>,
    vec: &'a Vec<Val>,
}

impl<'a, Val, Id: Eq+Hash+Copy+Into<usize>> Iterator for RemapperIter<'a, Val, Id> {
    type Item = (Id, &'a Val);

    fn next(&mut self) -> Option<(Id, &'a Val)> {
        let (&old_id, &new_id) = unwrap_or!(self.iter.next(), return None);
        let idx: usize = new_id.into();
        Some((old_id, &self.vec[idx]))
    }
}

pub struct RemapperIterMut<'a, Val: 'a, Id: 'a> {
    iter: hash_map::IterMut<'a, Id, Id>,
    vec: &'a mut Vec<Val>,
}

impl<'a, Val, Id: Eq+Hash+Copy+Into<usize>> Iterator for RemapperIterMut<'a, Val, Id> {
    type Item = (Id, &'a mut Val);

    fn next(&mut self) -> Option<(Id, &'a mut Val)> {
        let (&old_id, &mut new_id) = unwrap_or!(self.iter.next(), return None);
        let idx: usize = new_id.into();
        // Fiddle with the lifetime.  This is okay because all elements of `vec` are disjoint.
        let val: &'a mut Val = unsafe { &mut *(&mut self.vec[idx] as *mut _) };
        Some((old_id, val))
    }
}



pub struct Builder<'d> {
    data: &'d Data,

    anims: Remapper<&'d str, AnimId>,
    items: Remapper<&'d str, ItemId>,
    blocks: Remapper<&'d str, BlockId>,
    templates: Remapper<&'d str, TemplateId>,

    world: Option<Box<WorldBits>>,
    clients: Vec<ClientBits>,
    entities: Vec<EntityBits>,
    inventories: Vec<InventoryBits>,
    planes: Vec<PlaneBits>,
    terrain_chunks: Vec<TerrainChunkBits>,
    structures: Vec<StructureBits>,
}

impl<'d> Builder<'d> {
    pub fn new(data: &'d Data) -> Builder<'d> {
        Builder {
            data: data,

            anims: Remapper::new(),
            items: Remapper::new(),
            blocks: Remapper::new(),
            templates: Remapper::new(),

            world: None,
            clients: Vec::new(),
            entities: Vec::new(),
            inventories: Vec::new(),
            planes: Vec::new(),
            terrain_chunks: Vec::new(),
            structures: Vec::new(),
        }
    }


    pub fn anim(&mut self, name: &str) -> AnimId {
        let id = self.data.animations.get_id(name);
        self.anim_id(id)
    }

    pub fn anim_id(&mut self, id: AnimId) -> AnimId {
        let d = &self.data.animations;
        self.anims.get_or(id, |raw| (raw as AnimId,
                                     &d.animation(id).name))
    }

    pub fn item(&mut self, name: &str) -> ItemId {
        let id = self.data.item_data.get_id(name);
        self.item_id(id)
    }

    pub fn item_id(&mut self, id: ItemId) -> ItemId {
        let d = &self.data.item_data;
        self.items.get_or(id, |raw| (raw as ItemId,
                                     d.name(id)))
    }

    pub fn block(&mut self, name: &str) -> BlockId {
        let id = self.data.block_data.get_id(name);
        self.block_id(id)
    }

    pub fn block_id(&mut self, id: BlockId) -> BlockId {
        let d = &self.data.block_data;
        self.blocks.get_or(id, |raw| (raw as BlockId,
                                      d.name(id)))
    }

    pub fn template(&mut self, name: &str) -> TemplateId {
        let id = self.data.structure_templates.get_id(name);
        self.template_id(id)
    }

    pub fn template_id(&mut self, id: TemplateId) -> TemplateId {
        let d = &self.data.structure_templates;
        self.templates.get_or(id, |raw| (raw as TemplateId,
                                         &d.template(id).name))
    }


    pub fn world<'a>(&'a mut self) -> WorldBuilder<'a, 'd> {
        assert!(self.world.is_none());
        self.world = Some(Box::new(WorldBits::new()));
        WorldBuilder {
            owner: self,
        }
    }

    pub fn get_world<'a>(&'a mut self) -> WorldBuilder<'a, 'd> {
        assert!(self.world.is_some());
        WorldBuilder {
            owner: self,
        }
    }


    pub fn client<'a>(&'a mut self) -> ClientBuilder<'a, 'd> {
        let idx = self.clients.len();
        self.clients.push(ClientBits::new());
        ClientBuilder {
            owner: self,
            idx: idx,
        }
    }

    pub fn get_client<'a>(&'a mut self, id: ClientId) -> ClientBuilder<'a, 'd> {
        let idx = id.unwrap() as usize;
        assert!(idx < self.clients.len());
        ClientBuilder {
            owner: self,
            idx: idx,
        }
    }


    pub fn entity<'a>(&'a mut self) -> EntityBuilder<'a, 'd> {
        let idx = self.entities.len();
        self.entities.push(EntityBits::new());
        EntityBuilder {
            owner: self,
            idx: idx,
        }
    }

    pub fn get_entity<'a>(&'a mut self, id: EntityId) -> EntityBuilder<'a, 'd> {
        let idx = id.unwrap() as usize;
        assert!(idx < self.entities.len());
        EntityBuilder {
            owner: self,
            idx: idx,
        }
    }

    fn next_entity_id(&self) -> EntityId {
        EntityId(self.entities.len() as u32)
    }


    pub fn inventory<'a>(&'a mut self) -> InventoryBuilder<'a, 'd> {
        let idx = self.inventories.len();
        self.inventories.push(InventoryBits::new());
        InventoryBuilder {
            owner: self,
            idx: idx,
        }
    }

    pub fn get_inventory<'a>(&'a mut self, id: InventoryId) -> InventoryBuilder<'a, 'd> {
        let idx = id.unwrap() as usize;
        assert!(idx < self.inventories.len());
        InventoryBuilder {
            owner: self,
            idx: idx,
        }
    }

    fn next_inventory_id(&self) -> InventoryId {
        InventoryId(self.inventories.len() as u32)
    }


    pub fn plane<'a>(&'a mut self) -> PlaneBuilder<'a, 'd> {
        let idx = self.planes.len();
        self.planes.push(PlaneBits::new());
        PlaneBuilder {
            owner: self,
            idx: idx,
        }
    }

    pub fn get_plane<'a>(&'a mut self, id: PlaneId) -> PlaneBuilder<'a, 'd> {
        let idx = id.unwrap() as usize;
        assert!(idx < self.planes.len());
        PlaneBuilder {
            owner: self,
            idx: idx,
        }
    }


    pub fn terrain_chunk<'a>(&'a mut self) -> TerrainChunkBuilder<'a, 'd> {
        let idx = self.terrain_chunks.len();
        self.terrain_chunks.push(TerrainChunkBits::new());
        TerrainChunkBuilder {
            owner: self,
            idx: idx,
        }
    }

    pub fn get_terrain_chunk<'a>(&'a mut self, id: TerrainChunkId) -> TerrainChunkBuilder<'a, 'd> {
        let idx = id.unwrap() as usize;
        assert!(idx < self.terrain_chunks.len());
        TerrainChunkBuilder {
            owner: self,
            idx: idx,
        }
    }


    pub fn structure<'a>(&'a mut self) -> StructureBuilder<'a, 'd> {
        let idx = self.structures.len();
        self.structures.push(StructureBits::new());
        StructureBuilder {
            owner: self,
            idx: idx,
        }
    }

    pub fn get_structure<'a>(&'a mut self, id: StructureId) -> StructureBuilder<'a, 'd> {
        let idx = id.unwrap() as usize;
        assert!(idx < self.structures.len());
        StructureBuilder {
            owner: self,
            idx: idx,
        }
    }

    fn next_structure_id(&self) -> StructureId {
        StructureId(self.structures.len() as u32)
    }


    /// Final cleanup: provide default values for any data references that aren't set.
    fn cleanup(&mut self) {
        let anim_name = &self.data.animations.animation(0).name;

        for e in &mut self.entities {
            if e.anim.is_none() {
                e.anim = Some(self.anims.get_or(0, |raw| (raw as AnimId, anim_name)));
            }
        }

        // Inventories have ItemIds, but they have no default value (the default is Item::none()).
    }

    pub fn finish(mut self) -> Bundle {
        self.cleanup();

        Bundle {
            anims: convert_vec(self.anims.vals, |s| s.to_owned().into_boxed_str()),
            items: convert_vec(self.items.vals, |s| s.to_owned().into_boxed_str()),
            blocks: convert_vec(self.blocks.vals, |s| s.to_owned().into_boxed_str()),
            templates: convert_vec(self.templates.vals, |s| s.to_owned().into_boxed_str()),

            world: self.world.map(|w| Box::new(w.finish())),
            clients: convert_vec(self.clients, |c| c.finish()),
            entities: convert_vec(self.entities, |c| c.finish()),
            inventories: convert_vec(self.inventories, |c| c.finish()),
            planes: convert_vec(self.planes, |c| c.finish()),
            terrain_chunks: convert_vec(self.terrain_chunks, |c| c.finish()),
            structures: convert_vec(self.structures, |c| c.finish()),
        }
    }
}

fn convert_vec<T, U, F: FnMut(T) -> U>(v: Vec<T>, f: F) -> Box<[U]> {
    v.into_iter().map(f).collect::<Vec<_>>().into_boxed_slice()
}


struct WorldBits {
    now: Time,
    next_client: StableId,
    next_entity: StableId,
    next_inventory: StableId,
    next_plane: StableId,
    next_terrain_chunk: StableId,
    next_structure: StableId,

    extra: Extra,
    child_entities: Vec<EntityId>,
    child_inventories: Vec<InventoryId>,
}

impl WorldBits {
    fn new() -> WorldBits {
        WorldBits {
            now: 0,
            next_client: 0,
            next_entity: 0,
            next_inventory: 0,
            next_plane: 0,
            next_terrain_chunk: 0,
            next_structure: 0,

            extra: Extra::new(),
            child_entities: Vec::new(),
            child_inventories: Vec::new(),
        }
    }

    fn finish(self) -> World {
        World {
            now: self.now,
            next_client: self.next_client,
            next_entity: self.next_entity,
            next_inventory: self.next_inventory,
            next_plane: self.next_plane,
            next_terrain_chunk: self.next_terrain_chunk,
            next_structure: self.next_structure,

            extra: self.extra,
            child_entities: self.child_entities.into_boxed_slice(),
            child_inventories: self.child_inventories.into_boxed_slice(),
        }
    }
}

pub struct WorldBuilder<'a, 'd: 'a> {
    owner: &'a mut Builder<'d>,
}

impl<'a, 'd> WorldBuilder<'a, 'd> {
    pub fn owner(&mut self) -> &mut Builder<'d> {
        self.owner
    }

    fn get(&mut self) -> &mut WorldBits {
        self.owner.world.as_mut().unwrap()
    }

    pub fn now(&mut self, now: Time) -> &mut Self {
        self.get().now = now;
        self
    }

    pub fn next_client(&mut self, id: StableId) -> &mut Self {
        self.get().next_client = id;
        self
    }
    
    pub fn next_entity(&mut self, id: StableId) -> &mut Self {
        self.get().next_entity = id;
        self
    }
    
    pub fn next_inventory(&mut self, id: StableId) -> &mut Self {
        self.get().next_inventory = id;
        self
    }
    
    pub fn next_plane(&mut self, id: StableId) -> &mut Self {
        self.get().next_plane = id;
        self
    }
    
    pub fn next_terrain_chunk(&mut self, id: StableId) -> &mut Self {
        self.get().next_terrain_chunk = id;
        self
    }
    
    pub fn next_structure(&mut self, id: StableId) -> &mut Self {
        self.get().next_structure = id;
        self
    }

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
    }

    pub fn extra_(&mut self) -> &mut Extra {
        &mut self.get().extra
    }


    pub fn entity<F: FnOnce(&mut EntityBuilder)>(&mut self, f: F) -> &mut Self {
        let eid = {
            let mut e = self.owner.entity();
            f(&mut e);
            e.get().attachment = EntityAttachment::World;
            e.id()
        };
        self.get().child_entities.push(eid);
        self
    }

    pub fn entity_<'b>(&'b mut self) -> EntityBuilder<'b, 'd> {
        let eid = self.owner.next_entity_id();
        self.get().child_entities.push(eid);

        let mut e = self.owner.entity();
        assert!(e.id() == eid);
        e.get().attachment = EntityAttachment::World;

        e
    }

    pub fn child_entity(&mut self, eid: EntityId) -> &mut Self {
        {
            let mut e = self.owner.get_entity(eid);
            assert!(e.get().attachment == EntityAttachment::World);
        }
        self.get().child_entities.push(eid);
        self
    }


    pub fn inventory<F: FnOnce(&mut InventoryBuilder)>(&mut self, f: F) -> &mut Self {
        let iid = {
            let mut i = self.owner.inventory();
            f(&mut i);
            i.get().attachment = InventoryAttachment::World;
            i.id()
        };
        self.get().child_inventories.push(iid);
        self
    }

    pub fn inventory_<'b>(&'b mut self) -> InventoryBuilder<'b, 'd> {
        let iid = self.owner.next_inventory_id();
        self.get().child_inventories.push(iid);

        let mut i = self.owner.inventory();
        assert!(i.id() == iid);
        i.get().attachment = InventoryAttachment::World;

        i
    }

    pub fn child_inventory(&mut self, iid: InventoryId) -> &mut Self {
        {
            let mut i = self.owner.get_inventory(iid);
            assert!(i.get().attachment == InventoryAttachment::World);
        }
        self.get().child_inventories.push(iid);
        self
    }
}


struct ClientBits {
    name: String,
    pawn: Option<EntityId>,

    extra: Extra,
    stable_id: StableId,
    child_entities: Vec<EntityId>,
    child_inventories: Vec<InventoryId>,
}

impl ClientBits {
    fn new() -> ClientBits {
        ClientBits {
            name: String::new(),
            pawn: None,

            extra: Extra::new(),
            stable_id: NO_STABLE_ID,
            child_entities: Vec::new(),
            child_inventories: Vec::new(),
        }
    }

    fn finish(self) -> Client {
        Client {
            name: self.name.into_boxed_str(),
            pawn: self.pawn,

            extra: self.extra,
            stable_id: self.stable_id,
            child_entities: self.child_entities.into_boxed_slice(),
            child_inventories: self.child_inventories.into_boxed_slice(),
        }
    }
}

pub struct ClientBuilder<'a, 'd: 'a> {
    owner: &'a mut Builder<'d>,
    idx: usize,
}

impl<'a, 'd> ClientBuilder<'a, 'd> {
    pub fn owner(&mut self) -> &mut Builder<'d> {
        self.owner
    }

    pub fn id(&self) -> ClientId {
        ClientId(self.idx as u16)
    }

    fn get(&mut self) -> &mut ClientBits {
        &mut self.owner.clients[self.idx]
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.get().name = name.to_owned();
        self
    }

    pub fn pawn<F: FnOnce(&mut EntityBuilder)>(&mut self, f: F) -> &mut Self {
        let cid = self.id();
        let eid = {
            let mut e = self.owner.entity();
            f(&mut e);
            e.get().attachment = EntityAttachment::Client(cid);
            e.id()
        };
        self.get().pawn = Some(eid);
        self.get().child_entities.push(eid);
        self
    }

    pub fn pawn_<'b>(&'b mut self) -> EntityBuilder<'b, 'd> {
        let cid = self.id();
        let eid = self.owner.next_entity_id();
        self.get().child_entities.push(eid);
        self.get().pawn = Some(eid);

        let mut e = self.owner.entity();
        assert!(e.id() == eid);
        e.get().attachment = EntityAttachment::Client(cid);

        e
    }

    pub fn pawn_id(&mut self, eid: EntityId) -> &mut Self {
        self.get().pawn = Some(eid);
        self
    }

    pub fn stable_id(&mut self, id: StableId) -> &mut Self {
        self.get().stable_id = id;
        self
    }

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
    }

    pub fn extra_(&mut self) -> &mut Extra {
        &mut self.get().extra
    }


    pub fn entity<F: FnOnce(&mut EntityBuilder)>(&mut self, f: F) -> &mut Self {
        let cid = self.id();
        let eid = {
            let mut e = self.owner.entity();
            f(&mut e);
            e.get().attachment = EntityAttachment::Client(cid);
            e.id()
        };
        self.get().child_entities.push(eid);
        self
    }

    pub fn entity_<'b>(&'b mut self) -> EntityBuilder<'b, 'd> {
        let cid = self.id();
        let eid = self.owner.next_entity_id();
        self.get().child_entities.push(eid);

        let mut e = self.owner.entity();
        assert!(e.id() == eid);
        e.get().attachment = EntityAttachment::Client(cid);

        e
    }

    pub fn child_entity(&mut self, eid: EntityId) -> &mut Self {
        let cid = self.id();
        {
            let mut e = self.owner.get_entity(eid);
            assert!(e.get().attachment == EntityAttachment::World);
            e.get().attachment = EntityAttachment::Client(cid);
        }
        self.get().child_entities.push(eid);
        self
    }


    pub fn inventory<F: FnOnce(&mut InventoryBuilder)>(&mut self, f: F) -> &mut Self {
        let cid = self.id();
        let iid = {
            let mut i = self.owner.inventory();
            f(&mut i);
            i.get().attachment = InventoryAttachment::Client(cid);
            i.id()
        };
        self.get().child_inventories.push(iid);
        self
    }

    pub fn inventory_<'b>(&'b mut self) -> InventoryBuilder<'b, 'd> {
        let cid = self.id();
        let iid = self.owner.next_inventory_id();
        self.get().child_inventories.push(iid);

        let mut i = self.owner.inventory();
        assert!(i.id() == iid);
        i.get().attachment = InventoryAttachment::Client(cid);

        i
    }

    pub fn child_inventory(&mut self, iid: InventoryId) -> &mut Self {
        let cid = self.id();
        {
            let mut i = self.owner.get_inventory(iid);
            assert!(i.get().attachment == InventoryAttachment::World);
            i.get().attachment = InventoryAttachment::Client(cid);
        }
        self.get().child_inventories.push(iid);
        self
    }
}


struct EntityBits {
    stable_plane: Stable<PlaneId>,

    motion: Motion,
    anim: Option<AnimId>,
    facing: V3,
    target_velocity: V3,
    appearance: u32,

    extra: Extra,
    stable_id: StableId,
    attachment: EntityAttachment,
    child_inventories: Vec<InventoryId>,
}

impl EntityBits {
    fn new() -> EntityBits {
        EntityBits {
            stable_plane: STABLE_PLANE_LIMBO,

            motion: Motion::fixed(scalar(0)),
            anim: None,
            facing: V3::new(1, 0, 0),
            target_velocity: scalar(0),
            appearance: 0,

            extra: Extra::new(),
            stable_id: NO_STABLE_ID,
            attachment: EntityAttachment::World,
            child_inventories: Vec::new(),
        }
    }

    fn finish(self) -> Entity {
        Entity {
            stable_plane: self.stable_plane,

            motion: self.motion,
            anim: self.anim.expect("didn't clean up anim IDs"),
            facing: self.facing,
            target_velocity: self.target_velocity,
            appearance: self.appearance,

            extra: self.extra,
            stable_id: self.stable_id,
            attachment: self.attachment,
            child_inventories: self.child_inventories.into_boxed_slice(),
        }
    }
}

pub struct EntityBuilder<'a, 'd: 'a> {
    owner: &'a mut Builder<'d>,
    idx: usize,
}

impl<'a, 'd> EntityBuilder<'a, 'd> {
    pub fn owner(&mut self) -> &mut Builder<'d> {
        self.owner
    }

    pub fn id(&self) -> EntityId {
        EntityId(self.idx as u32)
    }

    fn get(&mut self) -> &mut EntityBits {
        &mut self.owner.entities[self.idx]
    }

    pub fn stable_plane(&mut self, stable_pid: Stable<PlaneId>) -> &mut Self {
        self.get().stable_plane = stable_pid;
        self
    }

    pub fn motion(&mut self, m: Motion) -> &mut Self {
        self.get().motion = m;
        self
    }

    pub fn anim(&mut self, name: &str) -> &mut Self {
        let id = self.owner.anim(name);
        self.get().anim = Some(id);
        self
    }

    pub fn anim_id(&mut self, anim_id: AnimId) -> &mut Self {
        let id = self.owner.anim_id(anim_id);
        self.get().anim = Some(id);
        self
    }

    pub fn facing(&mut self, facing: V3) -> &mut Self {
        self.get().facing = facing;
        self
    }

    pub fn appearance(&mut self, appearance: u32) -> &mut Self {
        self.get().appearance = appearance;
        self
    }

    pub fn stable_id(&mut self, id: StableId) -> &mut Self {
        self.get().stable_id = id;
        self
    }

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
    }

    pub fn extra_(&mut self) -> &mut Extra {
        &mut self.get().extra
    }


    pub fn inventory<F: FnOnce(&mut InventoryBuilder)>(&mut self, f: F) -> &mut Self {
        let eid = self.id();
        let iid = {
            let mut i = self.owner.inventory();
            f(&mut i);
            i.get().attachment = InventoryAttachment::Entity(eid);
            i.id()
        };
        self.get().child_inventories.push(iid);
        self
    }

    pub fn inventory_<'b>(&'b mut self) -> InventoryBuilder<'b, 'd> {
        let eid = self.id();
        let iid = self.owner.next_inventory_id();
        self.get().child_inventories.push(iid);

        let mut i = self.owner.inventory();
        assert!(i.id() == iid);
        i.get().attachment = InventoryAttachment::Entity(eid);

        i
    }

    pub fn child_inventory(&mut self, iid: InventoryId) -> &mut Self {
        let eid = self.id();
        {
            let mut i = self.owner.get_inventory(iid);
            assert!(i.get().attachment == InventoryAttachment::World);
            i.get().attachment = InventoryAttachment::Entity(eid);
        }
        self.get().child_inventories.push(iid);
        self
    }
}


struct InventoryBits {
    contents: Vec<Item>,

    extra: Extra,
    stable_id: StableId,
    attachment: InventoryAttachment,
}

impl InventoryBits {
    fn new() -> InventoryBits {
        InventoryBits {
            contents: Vec::new(),

            extra: Extra::new(),
            stable_id: NO_STABLE_ID,
            attachment: InventoryAttachment::World,
        }
    }

    fn finish(self) -> Inventory {
        Inventory {
            contents: self.contents.into_boxed_slice(),

            extra: self.extra,
            stable_id: self.stable_id,
            attachment: self.attachment,
        }
    }
}

pub struct InventoryBuilder<'a, 'd: 'a> {
    owner: &'a mut Builder<'d>,
    idx: usize,
}

impl<'a, 'd> InventoryBuilder<'a, 'd> {
    pub fn owner(&mut self) -> &mut Builder<'d> {
        self.owner
    }

    pub fn id(&self) -> InventoryId {
        InventoryId(self.idx as u32)
    }

    fn get(&mut self) -> &mut InventoryBits {
        &mut self.owner.inventories[self.idx]
    }

    pub fn size(&mut self, count: u8) -> &mut Self {
        let id = self.owner.item_id(Item::none().id);
        self.get().contents = iter::repeat(Item::new(id, 0)).take(count as usize).collect();
        self
    }

    pub fn item(&mut self, slot: u8, name: &str, count: u8) -> &mut Self {
        let id = self.owner.item(name);
        self.get().contents[slot as usize] = Item::new(id, count);
        self
    }

    pub fn item_id(&mut self, slot: u8, item_id: ItemId, count: u8) -> &mut Self {
        let id = self.owner.item_id(item_id);
        self.get().contents[slot as usize] = Item::new(id, count);
        self
    }

    pub fn stable_id(&mut self, id: StableId) -> &mut Self {
        self.get().stable_id = id;
        self
    }

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
    }

    pub fn extra_(&mut self) -> &mut Extra {
        &mut self.get().extra
    }
}


struct PlaneBits {
    name: String,

    saved_chunks: HashMap<V2, Stable<TerrainChunkId>>,

    extra: Extra,
    stable_id: StableId,
}

impl PlaneBits {
    fn new() -> PlaneBits {
        PlaneBits {
            name: String::new(),

            saved_chunks: HashMap::new(),

            extra: Extra::new(),
            stable_id: NO_STABLE_ID,
        }
    }

    fn finish(self) -> Plane {
        Plane {
            name: self.name.into_boxed_str(),

            saved_chunks: self.saved_chunks.into_iter().collect::<Vec<_>>().into_boxed_slice(),

            extra: self.extra,
            stable_id: self.stable_id,
        }
    }
}

pub struct PlaneBuilder<'a, 'd: 'a> {
    owner: &'a mut Builder<'d>,
    idx: usize,
}

impl<'a, 'd> PlaneBuilder<'a, 'd> {
    pub fn owner(&mut self) -> &mut Builder<'d> {
        self.owner
    }

    pub fn id(&self) -> PlaneId {
        PlaneId(self.idx as u32)
    }

    fn get(&mut self) -> &mut PlaneBits {
        &mut self.owner.planes[self.idx]
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.get().name = name.to_owned();
        self
    }

    pub fn saved_chunk(&mut self, cpos: V2, tcid: Stable<TerrainChunkId>) -> &mut Self {
        self.get().saved_chunks.insert(cpos, tcid);
        self
    }

    pub fn stable_id(&mut self, id: StableId) -> &mut Self {
        self.get().stable_id = id;
        self
    }

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
    }

    pub fn extra_(&mut self) -> &mut Extra {
        &mut self.get().extra
    }
}


struct TerrainChunkBits {
    stable_plane: Stable<PlaneId>,
    cpos: V2,
    blocks: Box<BlockChunk>,

    extra: Extra,
    stable_id: StableId,
    flags: TerrainChunkFlags,
    child_structures: Vec<StructureId>,
}

impl TerrainChunkBits {
    fn new() -> TerrainChunkBits {
        TerrainChunkBits {
            stable_plane: STABLE_PLANE_LIMBO,
            cpos: scalar(0),
            blocks: Box::new(EMPTY_CHUNK),

            extra: Extra::new(),
            stable_id: NO_STABLE_ID,
            flags: TerrainChunkFlags::empty(),
            child_structures: Vec::new(),
        }
    }

    fn finish(self) -> TerrainChunk {
        TerrainChunk {
            stable_plane: self.stable_plane,
            cpos: self.cpos,
            blocks: self.blocks,

            extra: self.extra,
            stable_id: self.stable_id,
            flags: self.flags,
            child_structures: self.child_structures.into_boxed_slice(),
        }
    }
}

pub struct TerrainChunkBuilder<'a, 'd: 'a> {
    owner: &'a mut Builder<'d>,
    idx: usize,
}

impl<'a, 'd> TerrainChunkBuilder<'a, 'd> {
    pub fn owner(&mut self) -> &mut Builder<'d> {
        self.owner
    }

    pub fn id(&self) -> TerrainChunkId {
        TerrainChunkId(self.idx as u32)
    }

    fn get(&mut self) -> &mut TerrainChunkBits {
        &mut self.owner.terrain_chunks[self.idx]
    }

    pub fn stable_plane(&mut self, stable_plane: Stable<PlaneId>) -> &mut Self {
        self.get().stable_plane = stable_plane;
        self
    }

    pub fn cpos(&mut self, cpos: V2) -> &mut Self {
        self.get().cpos = cpos;
        self
    }

    pub fn blocks(&mut self, blocks: Box<BlockChunk>) -> &mut Self {
        self.get().blocks = blocks;
        self
    }

    pub fn block(&mut self, pos: V3, name: &str) -> &mut Self {
        let id = self.owner.block(name);
        let bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));
        assert!(bounds.contains(pos));
        self.get().blocks[bounds.index(pos)] = id;
        self
    }

    pub fn block_id(&mut self, pos: V3, block_id: BlockId) -> &mut Self {
        let id = self.owner.block_id(block_id);
        let bounds = Region::new(scalar(0), scalar(CHUNK_SIZE));
        assert!(bounds.contains(pos));
        self.get().blocks[bounds.index(pos)] = id;
        self
    }

    pub fn stable_id(&mut self, id: StableId) -> &mut Self {
        self.get().stable_id = id;
        self
    }

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
    }

    pub fn extra_(&mut self) -> &mut Extra {
        &mut self.get().extra
    }

    pub fn flags(&mut self, flags: TerrainChunkFlags) -> &mut Self {
        self.get().flags = flags;
        self
    }


    pub fn structure<F: FnOnce(&mut StructureBuilder)>(&mut self, f: F) -> &mut Self {
        let sid = {
            let mut s = self.owner.structure();
            f(&mut s);
            s.get().attachment = StructureAttachment::Chunk;
            s.id()
        };
        self.get().child_structures.push(sid);
        self
    }

    pub fn structure_<'b>(&'b mut self) -> StructureBuilder<'b, 'd> {
        let sid = self.owner.next_structure_id();
        self.get().child_structures.push(sid);

        let mut s = self.owner.structure();
        assert!(s.id() == sid);
        s.get().attachment = StructureAttachment::Chunk;

        s
    }

    pub fn child_structure(&mut self, sid: StructureId) -> &mut Self {
        {
            let mut e = self.owner.get_structure(sid);
            assert!(e.get().attachment == StructureAttachment::Plane);
            e.get().attachment = StructureAttachment::Chunk;
        }
        self.get().child_structures.push(sid);
        self
    }
}


struct StructureBits {
    stable_plane: Stable<PlaneId>,
    pos: V3,
    template: TemplateId,

    extra: Extra,
    stable_id: StableId,
    flags: StructureFlags,
    attachment: StructureAttachment,
    child_inventories: Vec<InventoryId>,
}

impl StructureBits {
    fn new() -> StructureBits {
        StructureBits {
            stable_plane: STABLE_PLANE_LIMBO,
            pos: scalar(0),
            template: 0,

            extra: Extra::new(),
            stable_id: NO_STABLE_ID,
            flags: StructureFlags::empty(),
            attachment: StructureAttachment::Plane,
            child_inventories: Vec::new(),
        }
    }

    fn finish(self) -> Structure {
        Structure {
            stable_plane: self.stable_plane,
            pos: self.pos,
            template: self.template,

            extra: self.extra,
            stable_id: self.stable_id,
            flags: self.flags,
            attachment: self.attachment,
            child_inventories: self.child_inventories.into_boxed_slice(),
        }
    }
}

pub struct StructureBuilder<'a, 'd: 'a> {
    owner: &'a mut Builder<'d>,
    idx: usize,
}

impl<'a, 'd> StructureBuilder<'a, 'd> {
    pub fn owner(&mut self) -> &mut Builder<'d> {
        self.owner
    }

    pub fn id(&self) -> StructureId {
        StructureId(self.idx as u32)
    }

    fn get(&mut self) -> &mut StructureBits {
        &mut self.owner.structures[self.idx]
    }

    pub fn stable_plane(&mut self, stable_plane: Stable<PlaneId>) -> &mut Self {
        self.get().stable_plane = stable_plane;
        self
    }

    pub fn pos(&mut self, pos: V3) -> &mut Self {
        self.get().pos = pos;
        self
    }

    pub fn template(&mut self, name: &str) -> &mut Self {
        let id = self.owner.template(name);
        self.get().template = id;
        self
    }

    pub fn template_id(&mut self, template_id: TemplateId) -> &mut Self {
        let id = self.owner.template_id(template_id);
        self.get().template = id;
        self
    }

    pub fn stable_id(&mut self, id: StableId) -> &mut Self {
        self.get().stable_id = id;
        self
    }

    pub fn flags(&mut self, flags: StructureFlags) -> &mut Self {
        self.get().flags = flags;
        self
    }

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
    }

    pub fn extra_(&mut self) -> &mut Extra {
        &mut self.get().extra
    }


    pub fn inventory<F: FnOnce(&mut InventoryBuilder)>(&mut self, f: F) -> &mut Self {
        let sid = self.id();
        let iid = {
            let mut i = self.owner.inventory();
            f(&mut i);
            i.get().attachment = InventoryAttachment::Structure(sid);
            i.id()
        };
        self.get().child_inventories.push(iid);
        self
    }

    pub fn inventory_<'b>(&'b mut self) -> InventoryBuilder<'b, 'd> {
        let sid = self.id();
        let iid = self.owner.next_inventory_id();
        self.get().child_inventories.push(iid);

        let mut i = self.owner.inventory();
        assert!(i.id() == iid);
        i.get().attachment = InventoryAttachment::Structure(sid);

        i
    }

    pub fn child_inventory(&mut self, iid: InventoryId) -> &mut Self {
        let sid = self.id();
        {
            let mut i = self.owner.get_inventory(iid);
            assert!(i.get().attachment == InventoryAttachment::World);
            i.get().attachment = InventoryAttachment::Structure(sid);
        }
        self.get().child_inventories.push(iid);
        self
    }
}
