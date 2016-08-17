/// Basic component system, for attaching additional data to world objects.

use types::*;

use world::{Client, Entity, Inventory, Plane, TerrainChunk, Structure};
use world::bundle;


pub mod energy;
pub mod movement;


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


engine_part2!(pub EngineComponents(Components));

pub trait Component<Obj: ObjectType> {
    fn get<'a>(eng: &'a EngineComponents) -> &'a Self;
    fn get_mut<'a>(eng: &'a mut EngineComponents) -> &'a mut Self;

    fn export(&self, _id: Obj::Id, _b: &mut Obj::Bundled, _now: Time) {}
    fn import(&mut self, _id: Obj::Id, _b: &Obj::Bundled, _now: Time) {}
    fn cleanup(&mut self, _id: Obj::Id) {}
}


macro_rules! gen_funcs {
    ($(
            object $Obj:ident {
                export $export_obj:ident;
                import $import_obj:ident;
                cleanup $cleanup_obj:ident;
                components {
                    $( $component:ty, )*
                }
            }
            )*) => {
        $(
            pub fn $export_obj(eng: &EngineComponents,
                               id: <$Obj as ObjectType>::Id,
                               b: &mut <$Obj as ObjectType>::Bundled) {
                let now = eng.now();
                $( <$component as Component<$Obj>>::get(eng).export(id, b, now); )*
            }

            pub fn $import_obj(eng: &mut EngineComponents,
                               id: <$Obj as ObjectType>::Id,
                               b: &<$Obj as ObjectType>::Bundled) {
                let now = eng.now();
                $( <$component as Component<$Obj>>::get_mut(eng).import(id, b, now); )*
            }

            pub fn $cleanup_obj(eng: &mut EngineComponents,
                                id: <$Obj as ObjectType>::Id) {
                $( <$component as Component<$Obj>>::get_mut(eng).cleanup(id); )*
            }
        )*
    }
}

gen_funcs! {
    object Entity {
        export export_entity;
        import import_entity;
        cleanup cleanup_entity;
        components {
            energy::Energy,
            movement::Movement,
        }
    }
}
