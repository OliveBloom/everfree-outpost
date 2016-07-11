/// Basic component system, for attaching additional data to world objects.

use types::*;

use world::{Client, Entity, Inventory, Plane, TerrainChunk, Structure};
use world::bundle;


pub trait ObjectType {
    type Id: Copy;
    type Bundled;
}

impl ObjectType for Client {
    type Id = ClientId;
    type Bundled = bundle::Client;
}

impl ObjectType for Entity {
    type Id = EntityId;
    type Bundled = bundle::Entity;
}

impl ObjectType for Inventory {
    type Id = InventoryId;
    type Bundled = bundle::Inventory;
}

impl ObjectType for Plane {
    type Id = PlaneId;
    type Bundled = bundle::Plane;
}

impl ObjectType for TerrainChunk {
    type Id = TerrainChunkId;
    type Bundled = bundle::TerrainChunk;
}

impl ObjectType for Structure {
    type Id = StructureId;
    type Bundled = bundle::Structure;
}


engine_part2!(pub EngineComponents());

pub trait Component<Obj: ObjectType> {
    fn get<'a>(eng: &'a EngineComponents) -> &'a Self;
    fn get_mut<'a>(eng: &'a mut EngineComponents) -> &'a mut Self;

    fn export(&self, id: Obj::Id, b: &mut Obj::Bundled, now: Time);
    fn import(&mut self, id: Obj::Id, b: &Obj::Bundled, now: Time);
    fn cleanup(&mut self, id: Obj::Id);
}

pub trait ComponentObj<Obj: ObjectType> {
    fn export(eng: &EngineComponents, id: Obj::Id, b: &mut Obj::Bundled);
    fn import(eng: &mut EngineComponents, id: Obj::Id, b: &Obj::Bundled);
    fn cleanup(eng: &mut EngineComponents, id: Obj::Id);
}

impl<Obj: ObjectType, C: Component<Obj>> ComponentObj<Obj> for C {
    fn export(eng: &EngineComponents, id: Obj::Id, b: &mut Obj::Bundled) {
        let now = eng.now();
        C::get(eng).export(id, b, now);
    }

    fn import(eng: &mut EngineComponents, id: Obj::Id, b: &Obj::Bundled) {
        let now = eng.now();
        C::get_mut(eng).import(id, b, now);
    }

    fn cleanup(eng: &mut EngineComponents, id: Obj::Id) {
        C::get_mut(eng).cleanup(id);
    }
}
