use std::collections::HashMap;
use std::num::wrapping::OverflowingOps;
use std::ptr;
use python3_sys::*;

use types::*;

use engine::Engine;
use engine::glue;
use engine::split::{EngineRef, Part, PartFlags};
use logic;
use python as py;
use python::{PyBox, PyRef, PyResult};
use script::ScriptEngine;
use world::extra::Extra;
use world::Fragment as World_Fragment;
use world::object::*;


pub fn init(module: PyRef) {
    init_engine(module);
    init_extra(module);
}


/// Common base structure for allowing Python to obtain references to fields of objects.
struct NestedRefBase {
    base: PyObject,
    /// The reference from which this reference was derived.
    parent: *mut PyObject,
    /// The version of the parent at the time this reference was created.  If
    /// `parent.version != self.parent_ver`, then this reference has been invalidated by some
    /// subsequent mutation to the parent object.
    parent_ver: u32,
    /// The version of this reference.
    version: u32,
}

impl NestedRefBase {
    fn incr_version(&mut self) {
        let (x, overflow) = self.version.overflowing_add(1);
        assert!(!overflow);
        self.version = x;
    }

    fn valid(&self) -> bool {
        if self.parent.is_null() {
            return true;
        }

        unsafe {
            let parent = &*(self.parent as *const NestedRefBase);
            self.parent_ver == parent.version && parent.valid()
        }
    }

    /// `parent`'s representation must begin with `NestedRefBase`.
    unsafe fn set_parent(&mut self, parent: PyRef) {
        assert!(self.parent.is_null());
        self.parent = parent.to_box().unwrap();
        self.parent_ver = (*(self.parent as *mut NestedRefBase)).version;
    }

    fn dealloc(&mut self) {
        if !self.parent.is_null() {
            unsafe {
                Py_DECREF(self.parent);
                self.parent = ptr::null_mut();
            }
        }
    }
}


/// A reference to an `Engine`.  Similar to `RustRef`, but keeps different metadata (engine part
/// flags, instead of mutability).
struct PyEngineRef {
    base: NestedRefBase,
    ptr: *mut Engine<'static>,
    flags: usize,
}

macro_rules! engine_ref_func_wrapper {
    ( $wrap:ident, $engty:ty,
      $slf:ident, $args:ident, $engname:ident, $call:expr) => {
        fn $wrap(slf: $crate::python::PyRef,
                 args: $crate::python::PyRef)
                 -> $crate::python::PyResult<$crate::python::PyBox> {
            use $crate::engine::split::{Part, PartFlags};
            use $crate::python as py;
            use $crate::script2::{Pack, Unpack};

            pyassert!(py::object::is_instance(slf, get_engine_type()),
                      type_error, "expected EngineRef");
            unsafe {
                let er = &mut *(slf.as_ptr() as *mut PyEngineRef);
                let ref_flags = er.flags;
                let target_flags = <$engty as PartFlags>::flags();
                pyassert!(ref_flags & target_flags == target_flags,
                          type_error,
                          concat!("EngineRef does not have sufficient permissions for ",
                                  stringify!($engty)));
                pyassert!(er.base.valid(), runtime_error, "EngineRef has expired");

                er.flags = 0;
                er.base.incr_version();
                let result = Unpack::unpack(args)
                    .and_then(|$args| {
                        let $slf = slf;
                        let $engname = <$engty as Part>::from_ptr(er.ptr);
                        let result = $call;
                        Pack::pack(result)
                    });
                // Need to reset flags regardless of outcome
                er.flags = ref_flags;

                result
            }
        }
    };
}

macro_rules! engine_ref_func {
    ( $fname:ident,
      ( $aname1:ident : $aty1:path, $($args_rest:tt)* ),
      $ret_ty:ty,
      $body:expr ) => {

        #[allow(unused_variables)]
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp1!(imp, ($aname1: $aty1, $($args_rest)*), $ret_ty, $body);

            engine_ref_func_wrapper!(wrap, $aty1, slf, args, engine,
                                     imp(engine, args));
            call_wrapper!(wrap, slf, args)
        }
    };
}

macro_rules! engine_ref_func_with_ref {
    ( $fname:ident,
      ( $aname1:ident : $aty1:path, $($args_rest:tt)* ),
      $ret_ty:ty,
      $body:expr ) => {

        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp2!(imp, ($aname1: $aty1, $($args_rest)*), $ret_ty, $body);

            engine_ref_func_wrapper!(wrap, $aty1, slf, args, engine,
                                     imp(engine, slf, args));
            call_wrapper!(wrap, slf, args)
        }
    };
}

pub fn with_engine_ref<E, F, R>(e: E, f: F) -> R
        where E: Part + PartFlags, F: FnOnce(PyRef) -> R {
    unsafe {
        let obj = py::type_::instantiate(get_engine_type()).unwrap();
        {
            let er = &mut *(obj.as_ptr() as *mut PyEngineRef);
            er.ptr = e.as_ptr();
            er.flags = <E as PartFlags>::flags();
        }

        let result = f(obj.borrow());

        {
            let er = &mut *(obj.as_ptr() as *mut PyEngineRef);
            er.flags = 0;
            // Invalidate all ExtraRefs derived from `er`.
            er.base.incr_version();
        }

        result
    }
}

engine_part_typedef!(OnlyWorld(world));
engine_part_typedef!(OnlyMessages(messages));
engine_part_typedef!(EmptyPart());


define_python_class! {
    class EngineRef: PyEngineRef {
        type_obj ENGINE_REF_TYPE;
        initializer init_engine;
        accessor get_engine_type;
        method_macro engine_ref_func!;

    members:
    slots:
    methods:

        fn now(eng: EmptyPart,) -> Time {
            eng.now()
        }

        fn script_cb_chat_command(eng: EngineRef, cid: ClientId, msg: String) {
            warn_on_err!(ScriptEngine::cb_chat_command(eng.unwrap(), cid, &msg));
        }

        fn messages_clients_len(eng: OnlyMessages,) -> usize {
            eng.messages().clients_len()
        }

        fn messages_send_chat_update(eng: OnlyMessages, cid: ClientId, msg: String) {
            use messages::ClientResponse;
            let resp = ClientResponse::ChatUpdate(msg);
            eng.messages().send_client(cid, resp);
        }


        fn world_client_pawn_id(eng: OnlyWorld, cid: ClientId) -> PyResult<Option<EntityId>> {
            let c = pyunwrap!(eng.world().get_client(cid),
                              runtime_error, "no client with that ID");
            Ok(c.pawn_id())
        }


        fn(engine_ref_func_with_ref!) world_entity_extra(eng: glue::WorldFragment,
                                                         eng_ref: PyRef,
                                                         eid: EntityId) -> PyResult<PyBox> {
            let mut eng = eng;
            let mut e = pyunwrap!(eng.get_entity_mut(eid),
                                  runtime_error, "no entity with that ID");
            let extra = e.extra_mut();
            unsafe {
                derive_extra_ref(extra, eng_ref)
            }
        }

        fn world_entity_pos(eng: OnlyWorld, eid: EntityId) -> PyResult<V3> {
            let e = pyunwrap!(eng.world().get_entity(eid),
                              runtime_error, "no entity with that ID");
            Ok(e.pos(eng.now()))
        }

        fn world_entity_plane_id(eng: OnlyWorld, eid: EntityId) -> PyResult<PlaneId> {
            let e = pyunwrap!(eng.world().get_entity(eid),
                              runtime_error, "no entity with that ID");
            Ok(e.plane_id())
        }

        fn world_entity_teleport(eng: glue::WorldFragment,
                                 eid: EntityId,
                                 pos: V3) -> PyResult<()> {
            try!(logic::world::teleport_entity(eng, eid, pos));
            Ok(())
        }

        fn world_entity_teleport_plane(eng: glue::WorldFragment,
                                       eid: EntityId,
                                       pid: PlaneId,
                                       pos: V3) -> PyResult<()> {
            try!(logic::world::teleport_entity_plane(eng, eid, pid, pos));
            Ok(())
        }

        fn world_entity_teleport_stable_plane(eng: glue::WorldFragment,
                                              eid: EntityId,
                                              stable_pid: Stable<PlaneId>,
                                              pos: V3) -> PyResult<()> {
            try!(logic::world::teleport_entity_stable_plane(eng, eid, stable_pid, pos));
            Ok(())
        }


        fn world_plane_stable_id(eng: glue::WorldFragment,
                                 pid: PlaneId) -> PyResult<Stable<PlaneId>> {
            let mut eng = eng;
            let mut p = pyunwrap!(eng.get_plane_mut(pid),
                                  runtime_error, "no plane with that ID");
            Ok(p.stable_id())
        }

        fn world_plane_name(eng: OnlyWorld, pid: PlaneId) -> PyResult<String> {
            let p = pyunwrap!(eng.world().get_plane(pid),
                              runtime_error, "no plane with that ID");
            Ok(p.name().to_owned())
        }
    }
}


/// Reference to an `Extra` value.
struct ExtraRef {
    base: NestedRefBase,
    ptr: *mut Extra,
}

/// Unsafe because it loses the borrow.  It's also memory-unsafe to call this function twice
/// without incrementing the version of the parent.
unsafe fn derive_extra_ref(extra: &mut Extra, parent: PyRef) -> PyResult<PyBox> {
    let obj = try!(py::type_::instantiate(get_extra_type()));
    {
        let r = &mut *(obj.as_ptr() as *mut ExtraRef);
        r.base.set_parent(parent);
        r.ptr = extra as *mut _;
    }
    Ok(obj)
}

macro_rules! extra_ref_func {
    ( $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp1!(imp, $args, $ret_ty, $body);

            fn wrap(slf: $crate::python::PyRef,
                    args: $crate::python::PyRef)
                    -> $crate::python::PyResult<$crate::python::PyBox> {
                use $crate::script2::{Pack, Unpack};

                pyassert!(py::object::is_instance(slf, get_extra_type()),
                          type_error, "expected an ExtraRef");
                unsafe {
                    let er = &mut *(slf.as_ptr() as *mut ExtraRef);
                    pyassert!(er.base.valid(),
                              runtime_error, "ExtraRef has expired");

                    er.base.incr_version();
                    let result = imp(&mut *er.ptr, try!(Unpack::unpack(args)));
                    Pack::pack(result)
                }
            }

            call_wrapper!(wrap, slf, args)
        }
    };
}

macro_rules! extra_ref_func_with_ref {
    ( $fname:ident, $args:tt, $ret_ty:ty, $body:expr ) => {

        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            method_imp2!(imp, $args, $ret_ty, $body);

            fn wrap(slf: $crate::python::PyRef,
                    args: $crate::python::PyRef)
                    -> $crate::python::PyResult<$crate::python::PyBox> {
                use $crate::script2::{Pack, Unpack};

                pyassert!(py::object::is_instance(slf, get_extra_type()),
                          type_error, "expected an ExtraRef");
                unsafe {
                    let er = &mut *(slf.as_ptr() as *mut ExtraRef);
                    pyassert!(er.base.valid(),
                              runtime_error, "ExtraRef has expired");

                    er.base.incr_version();
                    let result = imp(&mut *er.ptr, slf, try!(Unpack::unpack(args)));
                    Pack::pack(result)
                }
            }

            call_wrapper!(wrap, slf, args)
        }

    };
}

define_python_class! {
    class ExtraRef: ExtraRef {
        type_obj EXTRA_REF_TYPE;
        initializer init_extra;
        accessor get_extra_type;
        method_macro extra_ref_func!;

        fn get_type(this: &Extra,) -> String {
            use world::extra::Extra::*;
            (match *this {
                Null => "null",
                Bool(_) => "bool",
                Int(_) => "int",
                Float(_) => "float",
                Str(_) => "str",

                Array(_) => "array",
                Hash(_) => "hash",

                ClientId(_) => "client_id",
                EntityId(_) => "entity_id",
                InventoryId(_) => "inventory_id",
                PlaneId(_) => "plane_id",
                TerrainChunkId(_) => "terrain_chunk_id",
                StructureId(_) => "structure_id",

                StableClientId(_) => "stable_client_id",
                StableEntityId(_) => "stable_entity_id",
                StableInventoryId(_) => "stable_inventory_id",
                StablePlaneId(_) => "stable_plane_id",
                StableTerrainChunkId(_) => "stable_terrain_chunk_id",
                StableStructureId(_) => "stable_structure_id",

                V2(_) => "v2",
                V3(_) => "v3",
                Region2(_) => "region2",
                Region3(_) => "region3",
            }).to_owned()
        }


        fn(raw_func!) is_valid(this: PyRef,) -> PyResult<bool> {
            pyassert!(py::object::is_instance(this, get_extra_type()),
                      type_error, "expected an ExtraRef");
            unsafe {
                let er = &mut *(this.as_ptr() as *mut ExtraRef);
                Ok(er.base.valid())
            }
        }

        fn(raw_func!) get_ptr(this: PyRef,) -> PyResult<usize> {
            pyassert!(py::object::is_instance(this, get_extra_type()),
                      type_error, "expected an ExtraRef");
            unsafe {
                let er = &mut *(this.as_ptr() as *mut ExtraRef);
                Ok(er.ptr as usize)
            }
        }


        fn set_null(this: &mut Extra,) {
            *this = Extra::Null;
        }

        fn get_bool(this: &Extra,) -> PyResult<bool> {
            match *this {
                Extra::Bool(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::Bool"),
            }
        }

        fn set_bool(this: &mut Extra, val: bool) {
            *this = Extra::Bool(val);
        }

        fn get_int(this: &Extra,) -> PyResult<i64> {
            match *this {
                Extra::Int(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::Int"),
            }
        }

        fn set_int(this: &mut Extra, val: i64) {
            *this = Extra::Int(val);
        }

        fn get_float(this: &Extra,) -> PyResult<f64> {
            match *this {
                Extra::Float(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::Float"),
            }
        }

        fn set_float(this: &mut Extra, val: f64) {
            *this = Extra::Float(val);
        }

        fn get_str(this: &Extra,) -> PyResult<String> {
            match *this {
                Extra::Str(ref val) => Ok(val.clone()),
                _ => pyraise!(type_error, "expected an Extra::Str"),
            }
        }

        fn set_str(this: &mut Extra, val: String) {
            *this = Extra::Str(val);
        }


        fn set_array(this: &mut Extra,) {
            *this = Extra::Array(Vec::new());
        }


        fn set_hash(this: &mut Extra,) {
            *this = Extra::Hash(HashMap::new());
        }


        fn get_client_id(this: &Extra,) -> PyResult<ClientId> {
            match *this {
                Extra::ClientId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::ClientId"),
            }
        }

        fn set_client_id(this: &mut Extra, val: ClientId) {
            *this = Extra::ClientId(val);
        }

        fn get_entity_id(this: &Extra,) -> PyResult<EntityId> {
            match *this {
                Extra::EntityId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::EntityId"),
            }
        }

        fn set_entity_id(this: &mut Extra, val: EntityId) {
            *this = Extra::EntityId(val);
        }

        fn get_inventory_id(this: &Extra,) -> PyResult<InventoryId> {
            match *this {
                Extra::InventoryId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::InventoryId"),
            }
        }

        fn set_inventory_id(this: &mut Extra, val: InventoryId) {
            *this = Extra::InventoryId(val);
        }

        fn get_plane_id(this: &Extra,) -> PyResult<PlaneId> {
            match *this {
                Extra::PlaneId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::PlaneId"),
            }
        }

        fn set_plane_id(this: &mut Extra, val: PlaneId) {
            *this = Extra::PlaneId(val);
        }

        fn get_terrain_chunk_id(this: &Extra,) -> PyResult<TerrainChunkId> {
            match *this {
                Extra::TerrainChunkId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::TerrainChunkId"),
            }
        }

        fn set_terrain_chunk_id(this: &mut Extra, val: TerrainChunkId) {
            *this = Extra::TerrainChunkId(val);
        }

        fn get_structure_id(this: &Extra,) -> PyResult<StructureId> {
            match *this {
                Extra::StructureId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::StructureId"),
            }
        }

        fn set_structure_id(this: &mut Extra, val: StructureId) {
            *this = Extra::StructureId(val);
        }


        fn get_stable_client_id(this: &Extra,) -> PyResult<Stable<ClientId>> {
            match *this {
                Extra::StableClientId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::StableClientId"),
            }
        }

        fn set_stable_client_id(this: &mut Extra, val: Stable<ClientId>) {
            *this = Extra::StableClientId(val);
        }

        fn get_stable_entity_id(this: &Extra,) -> PyResult<Stable<EntityId>> {
            match *this {
                Extra::StableEntityId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::StableEntityId"),
            }
        }

        fn set_stable_entity_id(this: &mut Extra, val: Stable<EntityId>) {
            *this = Extra::StableEntityId(val);
        }

        fn get_stable_inventory_id(this: &Extra,) -> PyResult<Stable<InventoryId>> {
            match *this {
                Extra::StableInventoryId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::StableInventoryId"),
            }
        }

        fn set_stable_inventory_id(this: &mut Extra, val: Stable<InventoryId>) {
            *this = Extra::StableInventoryId(val);
        }

        fn get_stable_plane_id(this: &Extra,) -> PyResult<Stable<PlaneId>> {
            match *this {
                Extra::StablePlaneId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::StablePlaneId"),
            }
        }

        fn set_stable_plane_id(this: &mut Extra, val: Stable<PlaneId>) {
            *this = Extra::StablePlaneId(val);
        }

        fn get_stable_terrain_chunk_id(this: &Extra,) -> PyResult<Stable<TerrainChunkId>> {
            match *this {
                Extra::StableTerrainChunkId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::StableTerrainChunkId"),
            }
        }

        fn set_stable_terrain_chunk_id(this: &mut Extra, val: Stable<TerrainChunkId>) {
            *this = Extra::StableTerrainChunkId(val);
        }

        fn get_stable_structure_id(this: &Extra,) -> PyResult<Stable<StructureId>> {
            match *this {
                Extra::StableStructureId(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::StableStructureId"),
            }
        }

        fn set_stable_structure_id(this: &mut Extra, val: Stable<StructureId>) {
            *this = Extra::StableStructureId(val);
        }


        /* TODO: V2 support
        fn get_v2(this: &Extra,) -> V2 {
            match *this {
                Extra::V2(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::V2"),
            }
        }

        fn set_v2(this: &mut Extra, val: V2) {
            *this = Extra::V2(val);
        }
        */

        fn get_v3(this: &Extra,) -> PyResult<V3> {
            match *this {
                Extra::V3(val) => Ok(val),
                _ => pyraise!(type_error, "expected an Extra::V3"),
            }
        }

        fn set_v3(this: &mut Extra, val: V3) {
            *this = Extra::V3(val);
        }

        /* TODO: Region<V2> support
        fn get_region2(this: &Extra,) -> Region<V2> {
            match *this {
                Extra::Region2(val) => val,
                _ => pyraise!(type_error, "expected an Extra::Region2"),
            }
        }

        fn set_region2(this: &mut Extra, val: Region<V2>) {
            *this = Extra::Region2(val);
        }
        */

        /* TODO: Region<V3> support
        fn get_region3(this: &Extra,) -> Region<V3> {
            match *this {
                Extra::Region3(val) => val,
                _ => pyraise!(type_error, "expected an Extra::Region3"),
            }
        }

        fn set_region3(this: &mut Extra, val: Region<V3>) {
            *this = Extra::Region3(val);
        }
        */


        fn(extra_ref_func_with_ref!) get_array(this: &mut Extra,
                                               py_this: PyRef) -> PyResult<PyBox> {
            if let Extra::Array(ref mut v) = *this {
                let list = try!(py::list::new());
                for val in v.iter_mut() {
                    let py_val = try!(unsafe { derive_extra_ref(val, py_this) });
                    try!(py::list::append(list.borrow(), py_val.borrow()));
                }
                Ok(list)
            } else {
                pyraise!(type_error, "expected an Extra::Array");
            }
        }

        fn(extra_ref_func_with_ref!) array_get(this: &mut Extra,
                                               py_this: PyRef,
                                               idx: usize) -> PyResult<PyBox> {
            if let Extra::Array(ref mut v) = *this {
                pyassert!(idx < v.len(), index_error, "out of range: {} >= {}", idx, v.len());
                unsafe { derive_extra_ref(&mut v[idx], py_this) }
            } else {
                pyraise!(type_error, "expected an Extra::Array");
            }
        }

        fn(extra_ref_func_with_ref!) array_append(this: &mut Extra,
                                                  py_this: PyRef,) -> PyResult<PyBox> {
            if let Extra::Array(ref mut v) = *this {
                v.push(Extra::Null);
                let idx = v.len() - 1;
                unsafe { derive_extra_ref(&mut v[idx], py_this) }
            } else {
                pyraise!(type_error, "expected an Extra::Array");
            }
        }

        fn array_pop(this: &mut Extra,) -> PyResult<()> {
            if let Extra::Array(ref mut v) = *this {
                v.pop();
                Ok(())
            } else {
                pyraise!(type_error, "expected an Extra::Array");
            }
        }

        fn array_len(this: &Extra,) -> PyResult<usize> {
            if let Extra::Array(ref v) = *this {
                Ok(v.len())
            } else {
                pyraise!(type_error, "expected an Extra::Array");
            }
        }


        fn(extra_ref_func_with_ref!) get_hash(this: &mut Extra,
                                              py_this: PyRef,) -> PyResult<PyBox> {
            if let Extra::Hash(ref mut h) = *this {
                let dict = try!(py::dict::new());
                for (key, val) in h.iter_mut() {
                    let py_val = try!(unsafe { derive_extra_ref(val, py_this) });
                    try!(py::dict::set_item_str(dict.borrow(), key, py_val.borrow()));
                }
                Ok(dict)
            } else {
                pyraise!(type_error, "expected an Extra::Hash");
            }
        }

        fn(extra_ref_func_with_ref!) hash_get(this: &mut Extra,
                                              py_this: PyRef,
                                              key: String) -> PyResult<PyBox> {
            if let Extra::Hash(ref mut h) = *this {
                if let Some(elt) = h.get_mut(&key) {
                    Ok(try!(unsafe { derive_extra_ref(elt, py_this) }))
                } else {
                    pyraise!(key_error, "not found: {:?}", key);
                }
            } else {
                pyraise!(type_error, "expected an Extra::Hash");
            }
        }

        fn(extra_ref_func_with_ref!) hash_put(this: &mut Extra,
                                              py_this: PyRef,
                                              key: String) -> PyResult<PyBox> {
            if let Extra::Hash(ref mut h) = *this {
                let elt = h.entry(key).or_insert_with(|| Extra::Null);
                unsafe { derive_extra_ref(elt, py_this) }
            } else {
                pyraise!(type_error, "expected an Extra::Hash");
            }
        }

        fn hash_delete(this: &mut Extra, key: String) -> PyResult<()> {
            if let Extra::Hash(ref mut h) = *this {
                h.remove(&key);
                Ok(())
            } else {
                pyraise!(type_error, "expected an Extra::Hash");
            }
        }

        fn hash_len(this: &Extra) -> PyResult<usize> {
            if let Extra::Hash(ref h) = *this {
                Ok(h.len())
            } else {
                pyraise!(type_error, "expected an Extra::Hash");
            }
        }

        fn hash_contains(this: &Extra,
                         key: String) -> PyResult<bool> {
            if let Extra::Hash(ref h) = *this {
                Ok(h.contains_key(&key))
            } else {
                pyraise!(type_error, "expected an Extra::Hash");
            }
        }

    }
}
