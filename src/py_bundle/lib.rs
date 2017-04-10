#![crate_name = "py_bundle"]

#![feature(
    plugin,
    trace_macros,
)]
#![plugin(syntax_exts)]


extern crate libc;
extern crate python3_sys;
extern crate server_bundle;
extern crate server_extra;
extern crate server_types;
extern crate server_world_types;

#[macro_use] extern crate python;


use server_types::*;

use python3_sys::PyObject;

use python::api as py;
use python::conv::{Pack, Unpack};
use python::exc::PyResult;
use python::ptr::{PyBox, PyRef};

use server_bundle::flat::{Flat, FlatView};
use server_bundle::types as b;
use server_extra::Extra;
use server_world_types::{EntityAttachment, InventoryAttachment, StructureAttachment};
use server_world_types::{Motion, Item};
use server_world_types::flags::{InventoryFlags, TerrainChunkFlags, StructureFlags};



trait Adapt: Sized {
    fn to_python(&self) -> PyResult<PyBox>;
    fn from_python(py: PyRef) -> PyResult<Self>;
}

impl Adapt for Box<str> {
    fn to_python(&self) -> PyResult<PyBox> {
        py::unicode::from_str(&**self)
    }

    fn from_python(py: PyRef) -> PyResult<Self> {
        let s = try!(py::unicode::as_string(py));
        Ok(s.into_boxed_str())
    }
}

impl<T: Adapt> Adapt for Box<[T]> {
    fn to_python(&self) -> PyResult<PyBox> {
        let lst = try!(py::list::new());
        for x in self.iter() {
            let py = try!(x.to_python());
            try!(py::list::append(lst.borrow(), py.borrow()));
        }
        Ok(lst)
    }

    fn from_python(py: PyRef) -> PyResult<Self> {
        pyassert!(py::list::check(py),
                  type_error, "expected list");
        let len = try!(py::list::size(py));
        let mut vec = Vec::with_capacity(len);
        for i in 0 .. len {
            let item = try!(py::list::get_item(py, i));
            vec.push(try!(T::from_python(item)));
        }
        Ok(vec.into_boxed_slice())
    }
}

impl<T: Adapt> Adapt for Option<T> {
    fn to_python(&self) -> PyResult<PyBox> {
        match *self {
            Some(ref x) => x.to_python(),
            None => Ok(py::none().to_box()),
        }
    }

    fn from_python(py: PyRef) -> PyResult<Self> {
        if py == py::none() {
            Ok(None)
        } else {
            let x = try!(T::from_python(py));
            Ok(Some(x))
        }
    }
}

impl<T: Adapt> Adapt for Box<T> {
    fn to_python(&self) -> PyResult<PyBox> {
        T::to_python(self)
    }

    fn from_python(py: PyRef) -> PyResult<Self> {
        T::from_python(py).map(Box::new)
    }
}

impl Adapt for Box<BlockChunk> {
    fn to_python(&self) -> PyResult<PyBox> {
        let lst = try!(py::list::new());
        for &block in self.iter() {
            let py = try!(Pack::pack(block));
            try!(py::list::append(lst.borrow(), py.borrow()));
        }
        Ok(lst)
    }

    fn from_python(py: PyRef) -> PyResult<Self> {
        let mut chunk = Box::new(EMPTY_CHUNK);
        for i in 0 .. chunk.len() {
            let item = try!(py::list::get_item(py, i));
            chunk[i] = try!(Unpack::unpack(item));
        }
        Ok(chunk)
    }
}


macro_rules! pack_adapt {
    ($($T:ty,)*) => {
        $(
            impl Adapt for $T {
                fn to_python(&self) -> PyResult<PyBox> {
                    Pack::pack(*self)
                }

                fn from_python(py: PyRef) -> PyResult<Self> {
                    Unpack::unpack(py)
                }
            }
        )*
    }
}

pack_adapt! {
    V3, V2,
    u8, u16, u32, u64, usize,
    i8, i16, i32, i64, isize,
    ClientId, EntityId, InventoryId, PlaneId, TerrainChunkId, StructureId,
    Stable<ClientId>, Stable<EntityId>, Stable<InventoryId>,
        Stable<PlaneId>, Stable<TerrainChunkId>, Stable<StructureId>,
    EntityAttachment, InventoryAttachment, StructureAttachment,

    (V2, Stable<TerrainChunkId>),
}

macro_rules! flags_adapt {
    ($($T:ident,)*) => {
        $(
            impl Adapt for $T {
                fn to_python(&self) -> PyResult<PyBox> {
                    self.bits().to_python()
                }

                fn from_python(py: PyRef) -> PyResult<Self> {
                    let bits: u32 = try!(Adapt::from_python(py));
                    let flags = pyunwrap!($T::from_bits(bits),
                            value_error, concat!("invalid bit pattern for ", stringify!($T)));
                    Ok(flags)
                }
            }
        )*
    };
}

flags_adapt! {
    InventoryFlags,
    TerrainChunkFlags,
    StructureFlags,
}


impl Adapt for Extra {
    fn to_python(&self) -> PyResult<PyBox> {
        use server_extra::{View, ArrayView, HashView};

        fn conv_array(a: ArrayView) -> PyResult<PyBox> {
            let lst = try!(py::list::new());
            for v in a.iter() {
                let py_v = try!(conv_view(v));
                try!(py::list::append(lst.borrow(), py_v.borrow()));
            }
            Ok(lst)
        }

        fn conv_hash(h: HashView) -> PyResult<PyBox> {
            let dct = try!(py::dict::new());
            for (k, v) in h.iter() {
                let py_v = try!(conv_view(v));
                try!(py::dict::set_item_str(dct.borrow(), k, py_v.borrow()));
            }
            Ok(dct)
        }

        fn conv_view(v: View) -> PyResult<PyBox> {
            match v {
                View::Value(v) => v.pack(),
                View::Array(a) => conv_array(a),
                View::Hash(h) => conv_hash(h),
            }
        }

        let dct = try!(py::dict::new());
        for (k, v) in self.iter() {
            let py_v = try!(conv_view(v));
            try!(py::dict::set_item_str(dct.borrow(), k, py_v.borrow()));
        }
        Ok(dct)
    }

    fn from_python(py: PyRef) -> PyResult<Self> {
        use server_extra::{ArrayViewMut, HashViewMut};

        fn conv_list(mut a: ArrayViewMut, lst: PyRef) -> PyResult<()> {
            let len = try!(py::list::size(lst));
            for i in 0 .. len {
                let v = try!(py::list::get_item(lst, i));
                a.borrow().push();
                if py::list::check(v) {
                    try!(conv_list(a.borrow().set_array(i), v));
                } else if py::dict::check(v) {
                    try!(conv_dict(a.borrow().set_hash(i), v));
                } else {
                    a.borrow().set(i, try!(Unpack::unpack(v)));
                }
            }
            Ok(())
        }

        fn conv_dict(mut h: HashViewMut, dct: PyRef) -> PyResult<()> {
            let items = try!(py::dict::items(dct));
            let len = try!(py::list::size(items.borrow()));
            for i in 0 .. len {
                let item = try!(py::list::get_item(items.borrow(), i));
                let (k, v): (String, PyRef) = try!(Unpack::unpack(item));
                if py::list::check(v) {
                    try!(conv_list(h.borrow().set_array(&k), v));
                } else if py::dict::check(v) {
                    try!(conv_dict(h.borrow().set_hash(&k), v));
                } else {
                    h.borrow().set(&k, try!(Unpack::unpack(v)));
                }
            }
            Ok(())
        }

        pyassert!(py::dict::check(py),
                  type_error, "expected dict");

        let items = try!(py::dict::items(py));
        let len = try!(py::list::size(items.borrow()));
        let mut e = Extra::new();
        for i in 0 .. len {
            let item = try!(py::list::get_item(items.borrow(), i));
            let (k, v): (String, PyRef) = try!(Unpack::unpack(item));
            if py::list::check(v) {
                try!(conv_list(e.set_array(&k), v));
            } else if py::dict::check(v) {
                try!(conv_dict(e.set_hash(&k), v));
            } else {
                e.set(&k, try!(Unpack::unpack(v)));
            }
        }
        Ok(e)
    }
}



#[macro_export]
macro_rules! adapter_new_slot {
    () => { unsafe extern "C" fn(_, _, _) -> _ };
    ( $fname:ident,
      ($this:ident: *mut $T:ty),
      $ret_ty:ty,
      $body:expr ) => {
        unsafe extern "C" fn $fname(subtype: *mut ::python3_sys::PyTypeObject,
                                    _args: *mut ::python3_sys::PyObject,
                                    _kwds: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            use python3_sys::*;

            let tp_alloc: fn(*mut PyTypeObject, Py_ssize_t) -> *mut PyObject =
                ::std::mem::transmute(PyType_GetSlot(subtype, Py_tp_alloc));

            let slf = tp_alloc(subtype, 0);
            {
                let $this = slf as *mut $T;
                $body;
            }
            slf
        }
    };
}

#[macro_export]
macro_rules! adapter_dealloc_slot {
    () => { unsafe extern "C" fn(_) };
    ( $fname:ident,
      ($this:ident: *mut $T:ty),
      $ret_ty:ty,
      $body:expr ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject) {
            use python3_sys::*;
            {
                let $this = slf as *mut $T;
                $body;
            }

            let tp_free: fn(*mut PyObject) =
                ::std::mem::transmute(PyType_GetSlot((*slf).ob_type, Py_tp_free));
            tp_free(slf);
        }
    };
}

macro_rules! adapt_struct {
    (adapter $ThingCls:ident : $Thing:path as $PyThing:ident {
        type_obj $TYPE:ident;
        initializer $init:ident;
        accessor $get_type:ident;

        $($field:ident,)*
    }) => {
        struct $PyThing {
            #[allow(dead_code)]
            base: PyObject,

            $($field: PyBox,)*
        }

        define_python_class! {
            class $ThingCls: $PyThing {
                type_obj $TYPE;
                initializer $init;
                accessor $get_type;
                method_macro this_type_has_no_methods!;

            members:
                $(let $field := $field;)*

            slots:
                fn(adapter_new_slot!) Py_tp_new(this: *mut $PyThing) -> () {
                    $(::std::ptr::write(&mut (*this).$field, py::none().to_box());)*
                }

                fn(adapter_dealloc_slot!) Py_tp_dealloc(this: *mut $PyThing) -> () {
                    $(::std::ptr::drop_in_place(&mut (*this).$field);)*
                }
            }
        }

        impl Adapt for $Thing {
            fn to_python(&self) -> PyResult<PyBox> {
                $( let $field = try!(self.$field.to_python()); )*

                unsafe {
                    let py = try!(py::type_::instantiate($get_type()));
                    // NB: a panic between here and end of block will cause memory unsafety.
                    // The `PyBox`es in the fields of the new object would be left uninitialized.
                    let ptr: &mut $PyThing = &mut *(py.as_ptr() as *mut $PyThing);
                    $( ::std::ptr::write(&mut ptr.$field, $field); )*

                    Ok(py)
                }
            }

            fn from_python(py: PyRef) -> PyResult<Self> {
                pyassert!(py::object::is_instance(py, $get_type()),
                          type_error, concat!("expected ", stringify!($ThingCls)));
                let result = {
                    let ptr: &$PyThing = unsafe { &*(py.as_ptr() as *const $PyThing) };
                    $Thing {
                        $( $field: try!(Adapt::from_python(ptr.$field.borrow())), )*
                    }
                };
                Ok(result)
            }
        }
    };
}

adapt_struct! {
    adapter Motion: Motion as PyMotion {
        type_obj MOTION_TYPE;
        initializer init_motion;
        accessor get_motion_type;

        start_pos,
        velocity,
        start_time,
        end_time,
    }
}

adapt_struct! {
    adapter Item: Item as PyItem {
        type_obj ITEM_TYPE;
        initializer init_item;
        accessor get_item_type;

        id,
        count,
    }
}

adapt_struct! {
    adapter World: b::World as PyWorld {
        type_obj WORLD_TYPE;
        initializer init_world;
        accessor get_world_type;

        now,

        next_client,
        next_entity,
        next_inventory,
        next_plane,
        next_terrain_chunk,
        next_structure,

        extra,
        child_entities,
        child_inventories,
    }
}

adapt_struct! {
    adapter Client: b::Client as PyClient {
        type_obj CLIENT_TYPE;
        initializer init_client;
        accessor get_client_type;

        name,
        pawn,

        extra,
        stable_id,
        child_entities,
        child_inventories,
    }
}

adapt_struct! {
    adapter Entity: b::Entity as PyEntity {
        type_obj ENTITY_TYPE;
        initializer init_entity;
        accessor get_entity_type;

        stable_plane,

        motion,
        anim,
        facing,
        target_velocity,
        appearance,

        extra,
        stable_id,
        attachment,
        child_inventories,
    }
}

adapt_struct! {
    adapter Inventory: b::Inventory as PyInventory {
        type_obj INVENTORY_TYPE;
        initializer init_inventory;
        accessor get_inventory_type;

        contents,

        extra,
        stable_id,
        flags,
        attachment,
    }
}

adapt_struct! {
    adapter Plane: b::Plane as PyPlane {
        type_obj PLANE_TYPE;
        initializer init_plane;
        accessor get_plane_type;

        name,

        saved_chunks,

        extra,
        stable_id,
    }
}

adapt_struct! {
    adapter TerrainChunk: b::TerrainChunk as PyTerrainChunk {
        type_obj TERRAIN_CHUNK_TYPE;
        initializer init_terrain_chunk;
        accessor get_terrain_chunk_type;

        stable_plane,
        cpos,
        blocks,

        extra,
        stable_id,
        flags,
        child_structures,
    }
}

adapt_struct! {
    adapter Structure: b::Structure as PyStructure {
        type_obj STRUCTURE_TYPE;
        initializer init_structure;
        accessor get_structure_type;

        stable_plane,
        pos,
        template,

        extra,
        stable_id,
        flags,
        attachment,
        child_inventories,
    }
}

adapt_struct! {
    adapter Bundle: b::Bundle as PyBundle {
        type_obj BUNDLE_TYPE;
        initializer init_bundle;
        accessor get_bundle_type;

        anims,
        items,
        blocks,
        templates,

        world,
        clients,
        entities,
        inventories,
        planes,
        terrain_chunks,
        structures,
    }
}



unsafe extern "C" fn read_bundle(_slf: *mut PyObject,
                                 args: *mut PyObject) -> *mut PyObject {
    fn inner(args: *mut PyObject) -> PyResult<PyBox> {
        let args = try!(unsafe { PyRef::new(args) });
        let (py_buf,): (PyRef,) = try!(Unpack::unpack(args));

        let buf = try!(py::bytes::as_slice(py_buf));

        let view = match FlatView::from_bytes(&buf) {
            Ok(x) => x,
            Err(e) => pyraise!(value_error, "error parsing bundle: {}", e),
        };
        let bundle = view.unflatten_bundle();

        Adapt::to_python(&bundle)
    }

    python::exc::return_result(inner(args))
}

unsafe extern "C" fn write_bundle(_slf: *mut PyObject,
                                  args: *mut PyObject) -> *mut PyObject {
    fn inner(args: *mut PyObject) -> PyResult<PyBox> {
        let args = try!(unsafe { PyRef::new(args) });
        let (py_bundle,): (PyRef,) = try!(Unpack::unpack(args));

        let bundle: b::Bundle = try!(Adapt::from_python(py_bundle));

        let mut f = Flat::new();
        f.flatten_bundle(&bundle);
        let buf = f.to_bytes();

        py::bytes::from_slice(&buf)
    }

    python::exc::return_result(inner(args))
}



fn module_init(m: PyRef) -> PyResult<()> {
    python::init_builtin_types(m);

    init_motion(m);
    init_item(m);

    init_world(m);
    init_client(m);
    init_entity(m);
    init_inventory(m);
    init_plane(m);
    init_terrain_chunk(m);
    init_structure(m);

    init_bundle(m);

    Ok(())
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyInit__outpost_bundle() -> *mut PyObject {
    use libc::c_char;
    use python3_sys::*;

    static mut MODULE_DEF: PyModuleDef = PyModuleDef {
        m_base: PyModuleDef_HEAD_INIT,
        m_name: 0 as *const _,
        m_doc: 0 as *const _,
        m_size: -1,
        m_methods: 0 as *mut _,
        m_reload: None,
        m_traverse: None,
        m_clear: None,
        m_free: None,
    };

    const BLANK_METHOD_DEF: PyMethodDef = PyMethodDef {
        ml_name: 0 as *const _,
        ml_meth: None,
        ml_flags: 0,
        ml_doc: 0 as *const _,
    };

    static mut METHOD_DEFS: [PyMethodDef; 3] = [BLANK_METHOD_DEF; 3];

    {
        macro_rules! init_method {
            ([$idx:expr] $name:ident) => {
                METHOD_DEFS[$idx].ml_name =
                    concat!(stringify!($name), "\0").as_ptr() as *const c_char;
                METHOD_DEFS[$idx].ml_meth = Some($name as PyCFunction);
                METHOD_DEFS[$idx].ml_flags = METH_VARARGS;
            };
        };
        init_method!([0] read_bundle);
        init_method!([1] write_bundle);

        let m = &mut MODULE_DEF;
        let name: &'static str = "_outpost_bundle\0";
        m.m_name = name.as_ptr() as *const c_char;
        m.m_methods = METHOD_DEFS.as_mut_ptr();
    }

    let m_raw = PyModule_Create(&mut MODULE_DEF);

    unsafe fn module_init_wrapper(raw: *mut PyObject) -> PyResult<PyBox> {
        let m = try!(PyBox::new(raw));
        try!(module_init(m.borrow()));
        Ok(m)
    }

    let result = module_init_wrapper(m_raw);
    let ptr = python::exc::return_result(result);
    ptr
}
