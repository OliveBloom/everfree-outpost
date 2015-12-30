use types::*;

use python as py;
use python::PyRef;

use script2::{Pack, Unpack};

macro_rules! define_namedtuple {
    (type $name:ident ($($arg:ident),*) : $T:ty {
        type_obj $TYPE_OBJ:ident;
        initializer $init:ident;
        accessor $get_type:ident;
        pack $pack:expr;
        unpack $unpack:expr;
    }) => {
        static mut $TYPE_OBJ: *mut ::python3_sys::PyObject = 0 as *mut _;

        pub fn $init(module: $crate::python::PyRef) {
            use $crate::python as py;

            assert!(py::is_initialized());

            let collections = py::import("collections").unwrap();
            let namedtuple = py::object::get_attr_str(collections.borrow(),
                                                      "namedtuple").unwrap();

            let args = Pack::pack((stringify!($name), ($(stringify!($arg),)*))).unwrap();
            let type_obj = py::object::call(namedtuple.borrow(), args.borrow(), None).unwrap();

            py::object::set_attr_str(module, stringify!($name), type_obj.borrow()).unwrap();
            unsafe {
                $TYPE_OBJ = type_obj.unwrap();
            }
        }

        pub fn $get_type() -> $crate::python::PyRef<'static> {
            unsafe {
                $crate::python::PyRef::new_non_null($TYPE_OBJ)
            }
        }

        impl $crate::script2::Pack for $T {
            fn pack(self) -> $crate::python::PyResult<$crate::python::PyBox> {
                let vals = $pack(self);
                let args = try!($crate::script2::Pack::pack(vals));
                $crate::python::object::call($get_type(), args.borrow(), None)
            }
        }

        impl<'a> $crate::script2::Unpack<'a> for $T {
            fn unpack(obj: $crate::python::PyRef<'a>) -> $crate::python::PyResult<$T> {
                let vals = try!($crate::script2::Unpack::unpack(obj));
                pyassert!($crate::python::object::is_instance(obj, $get_type()),
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
}
