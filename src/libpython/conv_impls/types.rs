use server_types::*;

use api::{PyBox, PyRef, PyResult};
use conv::{Pack, Unpack};
use server_world_types::{EntityAttachment, InventoryAttachment, StructureAttachment};


macro_rules! define_namedtuple {
    (type $name:ident ($($arg:ident),*) : $T:ty {
        type_obj $TYPE_OBJ:ident;
        initializer $init:ident;
        accessor $get_type:ident;
        pack $pack:expr;
        unpack $unpack:expr;
    }) => {
        static mut $TYPE_OBJ: *mut ::python3_sys::PyObject = 0 as *mut _;

        pub fn $init(module: $crate::ptr::PyRef) {
            use $crate::api as py;
            use $crate::util;

            assert!(py::is_initialized());

            let collections = util::import("collections").unwrap();
            let namedtuple = py::object::get_attr_str(collections.borrow(),
                                                      "namedtuple").unwrap();

            let args = Pack::pack((stringify!($name), ($(stringify!($arg),)*))).unwrap();
            let type_obj = py::object::call(namedtuple.borrow(), args.borrow(), None).unwrap();

            // `namedtuple` tries to guess the module name, likely based on the Python stack.  It
            // guesses wrong in this case, so fix it manually.
            let mod_name = py::unicode::from_str("_outpost_server").unwrap();
            py::object::set_attr_str(type_obj.borrow(), "__module__", mod_name.borrow()).unwrap();

            py::object::set_attr_str(module, stringify!($name), type_obj.borrow()).unwrap();
            unsafe {
                $TYPE_OBJ = type_obj.unwrap();
            }
        }

        pub fn $get_type() -> $crate::ptr::PyRef<'static> {
            unsafe {
                $crate::ptr::PyRef::new_non_null($TYPE_OBJ)
            }
        }

        impl $crate::conv::Pack for $T {
            fn pack(self) -> $crate::exc::PyResult<$crate::api::PyBox> {
                let vals = $pack(self);
                let args = try!($crate::conv::Pack::pack(vals));
                $crate::api::object::call($get_type(), args.borrow(), None)
            }
        }

        impl<'a> $crate::conv::Unpack<'a> for $T {
            fn unpack(obj: $crate::ptr::PyRef<'a>) -> $crate::exc::PyResult<$T> {
                let vals = try!($crate::conv::Unpack::unpack(obj));
                pyassert!($crate::api::object::is_instance(obj, $get_type()),
                          type_error, concat!("expected ", stringify!($name)));
                Ok($unpack(vals))
            }
        }
    };
}


define_namedtuple! {
    type ClientId(raw): ClientId {
        type_obj CLIENT_ID_TYPE;
        initializer init_client_id;
        accessor get_client_id_type;
        pack |ClientId(raw)| (raw,);
        unpack |(raw,)| ClientId(raw);
    }
}

define_namedtuple! {
    type EntityId(raw): EntityId {
        type_obj ENTITY_ID_TYPE;
        initializer init_entity_id;
        accessor get_entity_id_type;
        pack |EntityId(raw)| (raw,);
        unpack |(raw,)| EntityId(raw);
    }
}

define_namedtuple! {
    type InventoryId(raw): InventoryId {
        type_obj INVENTORY_ID_TYPE;
        initializer init_inventory_id;
        accessor get_inventory_id_type;
        pack |InventoryId(raw)| (raw,);
        unpack |(raw,)| InventoryId(raw);
    }
}

define_namedtuple! {
    type PlaneId(raw): PlaneId {
        type_obj PLANE_ID_TYPE;
        initializer init_plane_id;
        accessor get_plane_id_type;
        pack |PlaneId(raw)| (raw,);
        unpack |(raw,)| PlaneId(raw);
    }
}

define_namedtuple! {
    type TerrainChunkId(raw): TerrainChunkId {
        type_obj TERRAINCHUNK_ID_TYPE;
        initializer init_terrain_chunk_id;
        accessor get_terrain_chunk_id_type;
        pack |TerrainChunkId(raw)| (raw,);
        unpack |(raw,)| TerrainChunkId(raw);
    }
}

define_namedtuple! {
    type StructureId(raw): StructureId {
        type_obj STRUCTURE_ID_TYPE;
        initializer init_structure_id;
        accessor get_structure_id_type;
        pack |StructureId(raw)| (raw,);
        unpack |(raw,)| StructureId(raw);
    }
}


define_namedtuple! {
    type StableClientId(raw): Stable<ClientId> {
        type_obj STABLE_CLIENT_ID_TYPE;
        initializer init_stable_client_id;
        accessor get_stable_client_id_type;
        pack |s: Stable<ClientId>| (s.unwrap(),);
        unpack |(raw,)| Stable::new(raw);
    }
}

define_namedtuple! {
    type StableEntityId(raw): Stable<EntityId> {
        type_obj STABLE_ENTITY_ID_TYPE;
        initializer init_stable_entity_id;
        accessor get_stable_entity_id_type;
        pack |s: Stable<EntityId>| (s.unwrap(),);
        unpack |(raw,)| Stable::new(raw);
    }
}

define_namedtuple! {
    type StableInventoryId(raw): Stable<InventoryId> {
        type_obj STABLE_INVENTORY_ID_TYPE;
        initializer init_stable_inventory_id;
        accessor get_stable_inventory_id_type;
        pack |s: Stable<InventoryId>| (s.unwrap(),);
        unpack |(raw,)| Stable::new(raw);
    }
}

define_namedtuple! {
    type StablePlaneId(raw): Stable<PlaneId> {
        type_obj STABLE_PLANE_ID_TYPE;
        initializer init_stable_plane_id;
        accessor get_stable_plane_id_type;
        pack |s: Stable<PlaneId>| (s.unwrap(),);
        unpack |(raw,)| Stable::new(raw);
    }
}

define_namedtuple! {
    type StableTerrainChunkId(raw): Stable<TerrainChunkId> {
        type_obj STABLE_TERRAIN_CHUNK_ID_TYPE;
        initializer init_stable_terrain_chunk_id;
        accessor get_stable_terrain_chunk_id_type;
        pack |s: Stable<TerrainChunkId>| (s.unwrap(),);
        unpack |(raw,)| Stable::new(raw);
    }
}

define_namedtuple! {
    type StableStructureId(raw): Stable<StructureId> {
        type_obj STABLE_STRUCTURE_ID_TYPE;
        initializer init_stable_structure_id;
        accessor get_stable_structure_id_type;
        pack |s: Stable<StructureId>| (s.unwrap(),);
        unpack |(raw,)| Stable::new(raw);
    }
}


pub struct WorldAttach;
define_namedtuple! {
    type WorldAttach(): WorldAttach {
        type_obj WORLD_ATTACH_TYPE;
        initializer init_world_attach;
        accessor get_world_attach_type;
        pack |_: WorldAttach| ();
        unpack |()| WorldAttach;
    }
}

pub struct PlaneAttach;
define_namedtuple! {
    type PlaneAttach(): PlaneAttach {
        type_obj PLANE_ATTACH_TYPE;
        initializer init_plane_attach;
        accessor get_plane_attach_type;
        pack |_: PlaneAttach| ();
        unpack |()| PlaneAttach;
    }
}

pub struct ChunkAttach;
define_namedtuple! {
    type ChunkAttach(): ChunkAttach {
        type_obj CHUNK_ATTACH_TYPE;
        initializer init_chunk_attach;
        accessor get_chunk_attach_type;
        pack |_: ChunkAttach| ();
        unpack |()| ChunkAttach;
    }
}


impl<'a> Unpack<'a> for EntityAttachment {
    fn unpack(obj: PyRef<'a>) -> PyResult<EntityAttachment> {
        if let Ok(_) = <WorldAttach as Unpack>::unpack(obj) {
            Ok(EntityAttachment::World)
        } else if let Ok(_) = <ChunkAttach as Unpack>::unpack(obj) {
            Ok(EntityAttachment::Chunk)
        } else if let Ok(cid) = <ClientId as Unpack>::unpack(obj) {
            Ok(EntityAttachment::Client(cid))
        } else {
            pyraise!(type_error, "expected entity attachment");
        }
    }
}

impl<'a> Unpack<'a> for InventoryAttachment {
    fn unpack(obj: PyRef<'a>) -> PyResult<InventoryAttachment> {
        if let Ok(_) = <WorldAttach as Unpack>::unpack(obj) {
            Ok(InventoryAttachment::World)
        } else if let Ok(cid) = <ClientId as Unpack>::unpack(obj) {
            Ok(InventoryAttachment::Client(cid))
        } else if let Ok(cid) = <EntityId as Unpack>::unpack(obj) {
            Ok(InventoryAttachment::Entity(cid))
        } else if let Ok(cid) = <StructureId as Unpack>::unpack(obj) {
            Ok(InventoryAttachment::Structure(cid))
        } else {
            pyraise!(type_error, "expected inventory attachment");
        }
    }
}

impl<'a> Unpack<'a> for StructureAttachment {
    fn unpack(obj: PyRef<'a>) -> PyResult<StructureAttachment> {
        if let Ok(_) = <PlaneAttach as Unpack>::unpack(obj) {
            Ok(StructureAttachment::Plane)
        } else if let Ok(_) = <ChunkAttach as Unpack>::unpack(obj) {
            Ok(StructureAttachment::Chunk)
        } else {
            pyraise!(type_error, "expected structure attachment");
        }
    }
}

impl Pack for EntityAttachment {
    fn pack(self) -> PyResult<PyBox> {
        match self {
            EntityAttachment::World => Pack::pack(WorldAttach),
            EntityAttachment::Chunk => Pack::pack(ChunkAttach),
            EntityAttachment::Client(cid) => Pack::pack(cid),
        }
    }
}

impl Pack for InventoryAttachment {
    fn pack(self) -> PyResult<PyBox> {
        match self {
            InventoryAttachment::World => Pack::pack(WorldAttach),
            InventoryAttachment::Client(cid) => Pack::pack(cid),
            InventoryAttachment::Entity(eid) => Pack::pack(eid),
            InventoryAttachment::Structure(sid) => Pack::pack(sid),
        }
    }
}

impl Pack for StructureAttachment {
    fn pack(self) -> PyResult<PyBox> {
        match self {
            StructureAttachment::Plane => Pack::pack(PlaneAttach),
            StructureAttachment::Chunk => Pack::pack(ChunkAttach),
        }
    }
}


pub fn init(module: PyRef) {
    init_client_id(module);
    init_entity_id(module);
    init_inventory_id(module);
    init_plane_id(module);
    init_terrain_chunk_id(module);
    init_structure_id(module);

    init_stable_client_id(module);
    init_stable_entity_id(module);
    init_stable_inventory_id(module);
    init_stable_plane_id(module);
    init_stable_terrain_chunk_id(module);
    init_stable_structure_id(module);

    init_world_attach(module);
    init_plane_attach(module);
    init_chunk_attach(module);
}
