use std::collections::HashMap;
use std::hash::Hash;
use std::iter;

use types::*;

use data::Data;
use world::extra::Extra;
use world::types::{Motion, Item};
use world::types::{EntityAttachment, InventoryAttachment};

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
}


pub struct Builder<'d> {
    data: &'d Data,

    anims: Remapper<&'d str, AnimId>,
    items: Remapper<&'d str, ItemId>,
    blocks: Remapper<&'d str, BlockId>,
    templates: Remapper<&'d str, TemplateId>,

    clients: Vec<ClientBits>,
    entities: Vec<EntityBits>,
    inventories: Vec<InventoryBits>,
    //planes: Vec<Plane>,
    //terrain_chunks: Vec<TerrainChunk>,
    //structures: Vec<Structure>,
}

impl<'d> Builder<'d> {
    pub fn new(data: &'d Data) -> Builder<'d> {
        Builder {
            data: data,

            anims: Remapper::new(),
            items: Remapper::new(),
            blocks: Remapper::new(),
            templates: Remapper::new(),

            clients: Vec::new(),
            entities: Vec::new(),
            inventories: Vec::new(),
            //planes: Vec::new(),
            //terrain_chunks: Vec::new(),
            //structures: Vec::new(),
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


    pub fn client<'a>(&'a mut self) -> ClientBuilder<'a, 'd> {
        let idx = self.clients.len();
        self.clients.push(ClientBits::new());
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

    pub fn inventory<'a>(&'a mut self) -> InventoryBuilder<'a, 'd> {
        let idx = self.inventories.len();
        self.inventories.push(InventoryBits::new());
        InventoryBuilder {
            owner: self,
            idx: idx,
        }
    }


    /// Final cleanup: provide default values for any data references that aren't set.
    fn cleanup(&mut self) {
        let anim_name = &self.data.animations.animation(0).name;

        for e in &mut self.entities {
            if e.anim.is_none() {
                e.anim = Some(self.anims.get_or(0, |raw| (raw as AnimId, anim_name)));
            }
        }

        // Inventories have ItemIds, but they have no default value (the default is Item::Empty).
    }

    pub fn finish(mut self) -> Bundle {
        self.cleanup();

        Bundle {
            anims: convert_vec(self.anims.vals, |s| s.to_owned().into_boxed_slice()),
            items: convert_vec(self.items.vals, |s| s.to_owned().into_boxed_slice()),
            blocks: convert_vec(self.blocks.vals, |s| s.to_owned().into_boxed_slice()),
            templates: convert_vec(self.templates.vals, |s| s.to_owned().into_boxed_slice()),

            world: None,
            clients: convert_vec(self.clients, |c| c.finish()),
            entities: convert_vec(self.entities, |c| c.finish()),
            inventories: convert_vec(self.inventories, |c| c.finish()),
            planes: Vec::new().into_boxed_slice(),
            terrain_chunks: Vec::new().into_boxed_slice(),
            structures: Vec::new().into_boxed_slice(),
        }
    }
}

fn convert_vec<T, U, F: FnMut(T) -> U>(v: Vec<T>, f: F) -> Box<[U]> {
    v.into_iter().map(f).collect::<Vec<_>>().into_boxed_slice()
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
            name: self.name.into_boxed_slice(),
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

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
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

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
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
    pub fn id(&self) -> InventoryId {
        InventoryId(self.idx as u32)
    }

    fn get(&mut self) -> &mut InventoryBits {
        &mut self.owner.inventories[self.idx]
    }

    pub fn size(&mut self, count: u8) -> &mut Self {
        self.get().contents = iter::repeat(Item::Empty).take(count as usize).collect();
        self
    }

    pub fn item(&mut self, slot: u8, name: &str, count: u8) -> &mut Self {
        let id = self.owner.item(name);
        self.get().contents[slot as usize] = Item::Bulk(count, id);
        self
    }

    pub fn item_id(&mut self, slot: u8, item_id: ItemId, count: u8) -> &mut Self {
        let id = self.owner.item_id(item_id);
        self.get().contents[slot as usize] = Item::Bulk(count, id);
        self
    }

    pub fn extra<F: FnOnce(&mut Extra)>(&mut self, f: F) -> &mut Self {
        f(&mut self.get().extra);
        self
    }
}
