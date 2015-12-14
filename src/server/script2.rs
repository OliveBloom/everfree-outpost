use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use libc::{c_char, c_int, c_uint, c_void};

use types::*;

use data::Data;
use engine::Engine;
use engine::split::{Part, PartFlags};
use storage::Storage;
use python as py;
use python::{PyBox, PyRef};
use python3_sys::*;



const BLANK_TYPE_SPEC: PyType_Spec = PyType_Spec {
    name: 0 as *const _,
    basicsize: 0,
    itemsize: 0,
    flags: 0,
    slots: 0 as *mut _,
};

const BLANK_TYPE_SLOT: PyType_Slot = PyType_Slot {
    slot: 0,
    pfunc: 0 as *mut _,
};

const BLANK_METHOD_DEF: PyMethodDef = PyMethodDef {
    ml_name: 0 as *const _,
    ml_meth: None,
    ml_flags: 0,
    ml_doc: 0 as *const _,
};


// FFI builtin module

const MOD_NAME: &'static str = "_outpost_server\0";

static mut FFI_MOD_DEF: PyModuleDef = PyModuleDef {
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

static mut FFI_METHOD_DEFS: [PyMethodDef; 2] = [BLANK_METHOD_DEF; 2];

pub fn ffi_module_preinit() {
    unsafe {
        assert!(!py::is_initialized());
        PyImport_AppendInittab(MOD_NAME.as_ptr() as *const c_char,
                               Some(ffi_module_init));
    }
}

pub fn ffi_module_postinit() {
    py::import(&MOD_NAME[.. MOD_NAME.len() - 1]);
}

static mut FFI_MODULE: *mut PyObject = 0 as *mut _;

extern "C" fn ffi_module_init() -> *mut PyObject {
    unsafe {
        FFI_MOD_DEF.m_name = MOD_NAME.as_ptr() as *const c_char;
        FFI_MOD_DEF.m_methods = FFI_METHOD_DEFS.as_mut_ptr();

        let module = py::module::create(&mut FFI_MOD_DEF);

        storage_ref_init(module.borrow());
        data_ref_init(module.borrow());
        engine_ref_init(module.borrow());

        FFI_MODULE = module.clone().unwrap();
        module.unwrap()
    }
}

fn ffi_module() -> PyRef<'static> {
    unsafe { PyRef::new(FFI_MODULE) }
}


// RustRef and related functionality

pub struct RustRef {
    base: PyObject,
    ptr: *mut c_void,
    mutable: bool,
    valid: bool,
}


struct RustRefGuard<'a, T: 'a> {
    obj: &'a mut RustRef,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Drop for RustRefGuard<'a, T> {
    fn drop(&mut self) {
        self.obj.valid = true;
    }
}

impl<'a, T> Deref for RustRefGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { & *(self.obj.ptr as *const T) }
    }
}


struct RustRefGuardMut<'a, T: 'a> {
    obj: &'a mut RustRef,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Drop for RustRefGuardMut<'a, T> {
    fn drop(&mut self) {
        self.obj.valid = true;
    }
}

impl<'a, T> Deref for RustRefGuardMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { & *(self.obj.ptr as *const T) }
    }
}

impl<'a, T> DerefMut for RustRefGuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.obj.ptr as *mut T) }
    }
}


/// The returned PyBox must be a Python type object whose representation is compatible with the
/// `RustRef` struct.
pub unsafe trait GetTypeObject {
    fn get_type_object() -> PyBox;
}


unsafe fn build_rust_ref<T: GetTypeObject>(val: &T) -> PyBox {
    let ty = T::get_type_object();
    build_rust_ref_internal(val as *const _ as *mut c_void, ty, false)
}

unsafe fn build_rust_ref_mut<T: GetTypeObject>(val: &mut T) -> PyBox {
    let ty = T::get_type_object();
    build_rust_ref_internal(val as *mut _ as *mut c_void, ty, true)
}

unsafe fn build_rust_ref_internal(val: *mut c_void, ty: PyBox, mutable: bool) -> PyBox {
    let obj = py::type_::instantiate(ty.borrow());
    {
        let rr = &mut *(obj.as_ptr() as *mut RustRef);
        rr.ptr = val;
        rr.valid = true;
        rr.mutable = mutable;
    }
    obj
}

/// `obj` must refer to a RustRef.
unsafe fn invalidate_rust_ref(obj: PyRef) {
    let rr = &mut *(obj.as_ptr() as *mut RustRef);
    rr.valid = false;
}

/// The object must be a RustRef (checked) that refers to a value of type T (not checked).
unsafe fn unpack_rust_ref<'a, T: GetTypeObject>(obj: PyRef<'a>) -> RustRefGuard<'a, T> {
    let ty = T::get_type_object();
    assert!(py::object::is_instance(obj, ty.borrow()));
    let rr = &mut *(obj.as_ptr() as *mut RustRef);
    assert!(rr.valid);
    rr.valid = false;
    RustRefGuard {
        obj: rr,
        _marker: PhantomData,
    }
}

/// The object must be a RustRef (checked) that refers to a value of type T (not checked).
unsafe fn unpack_rust_ref_mut<'a, T: GetTypeObject>(obj: PyRef<'a>) -> RustRefGuardMut<'a, T> {
    let ty = T::get_type_object();
    assert!(py::object::is_instance(obj, ty.borrow()));
    let rr = &mut *(obj.as_ptr() as *mut RustRef);
    assert!(rr.valid);
    assert!(rr.mutable);
    rr.valid = false;
    RustRefGuardMut {
        obj: rr,
        _marker: PhantomData,
    }
}

pub fn with_ref<T, F>(val: &T, f: F)
        where T: GetTypeObject, F: FnOnce(PyRef) {
    unsafe {
        let obj = build_rust_ref(val);
        f(obj.borrow());
        invalidate_rust_ref(obj.borrow());
    }
}



trait Unpack<'a> {
    fn unpack(obj: PyRef<'a>) -> Self;
}

impl<'a> Unpack<'a> for PyRef<'a> {
    fn unpack(obj: PyRef<'a>) -> PyRef<'a> {
        obj
    }
}

impl<'a> Unpack<'a> for PyBox {
    fn unpack(obj: PyRef<'a>) -> PyBox {
        obj.to_box()
    }
}

impl<'a> Unpack<'a> for String {
    fn unpack(obj: PyRef<'a>) -> String {
        py::unicode::as_string(obj)
    }
}


trait Pack {
    fn pack(self) -> PyBox;
}

impl Pack for PyBox {
    fn pack(self) -> PyBox {
        self
    }
}

impl<'a> Pack for PyRef<'a> {
    fn pack(self) -> PyBox {
        self.to_box()
    }
}

impl<A: Pack> Pack for Option<A> {
    fn pack(self) -> PyBox {
        match self {
            Some(a) => Pack::pack(a),
            None => py::none().to_box(),
        }
    }
}

impl<'a> Pack for &'a str {
    fn pack(self) -> PyBox {
        py::unicode::from_str(self)
    }
}

impl<'a, K, V> Pack for &'a HashMap<K, V>
        where K: Clone + Pack + Eq + Hash,
              V: Clone + Pack {
    fn pack(self) -> PyBox {
        let dct = py::dict::new();
        for (k, v) in self {
            let k = Pack::pack(k.clone());
            let v = Pack::pack(v.clone());
            py::dict::set_item(dct.borrow(), k.borrow(), v.borrow());
        }
        dct
    }
}


macro_rules! tuple_impls {
    ($count:expr, $pack:ident: ($($A:ident $idx:expr),*)) => {
        impl<'a, $($A: Unpack<'a>,)*> Unpack<'a> for ($($A,)*) {
            fn unpack(obj: PyRef<'a>) -> ($($A,)*) {
                assert!(py::tuple::check(obj));
                assert!(py::tuple::size(obj) == $count);
                (
                    $( Unpack::unpack(py::tuple::get_item(obj, $idx)), )*
                )
            }
        }

        impl<$($A: Pack,)*> Pack for ($($A,)*) {
            #[allow(non_snake_case)]
            fn pack(self) -> PyBox {
                let ($($A,)*) = self;
                py::tuple::$pack(
                    $(Pack::pack($A),)*
                )
            }
        }
    };
}

tuple_impls!(0, pack0: ());
tuple_impls!(1, pack1: (A 0));
tuple_impls!(2, pack2: (A 0, B 1));
tuple_impls!(3, pack3: (A 0, B 1, C 2));

macro_rules! int_impls {
    ($ty:ident, unsigned) => {
        impl<'a> Unpack<'a> for $ty {
            fn unpack(obj: PyRef<'a>) -> $ty {
                let raw = py::int::as_u64(obj);
                assert!(raw <= ::std::$ty::MAX as u64);
                raw as $ty
            }
        }

        impl Pack for $ty {
            fn pack(self) -> PyBox {
                py::int::from_u64(self as u64)
            }
        }
    };

    ($ty:ident, signed) => {
        impl<'a> Unpack<'a> for $ty {
            fn unpack(obj: PyRef<'a>) -> $ty {
                let raw = py::int::as_i64(obj);
                assert!(raw >= ::std::$ty::MIN as i64);
                assert!(raw <= ::std::$ty::MAX as i64);
                raw as $ty
            }
        }

        impl Pack for $ty {
            fn pack(self) -> PyBox {
                py::int::from_i64(self as i64)
            }
        }
    };
}

int_impls!(u8, unsigned);
int_impls!(u16, unsigned);
int_impls!(u32, unsigned);
int_impls!(u64, unsigned);
int_impls!(usize, unsigned);

int_impls!(i8, signed);
int_impls!(i16, signed);
int_impls!(i32, signed);
int_impls!(i64, signed);
int_impls!(isize, signed);



// Storage ref definition

macro_rules! one {
    ( $any:tt ) => { 1 };
}

macro_rules! rust_ref_func {
    ( $ty:ty, $fname:ident, ( &$this:ident $(, $aname:ident : $aty:ty )* ), $ret_ty:ty, $body:block ) => {
        unsafe extern "C" fn $fname(slf: *mut PyObject,
                                    args: *mut PyObject) -> *mut PyObject {
            let slf = PyRef::new(slf);
            let args = PyRef::new(args);
            let guard = unpack_rust_ref::<$ty>(slf);

            fn $fname($this: &$ty, ($($aname,)*) : ($($aty,)*)) -> $ret_ty {
                $body
            }

            let result = $fname(&*guard, Unpack::unpack(args));
            Pack::pack(result).unwrap()
        }
    };

    ( $ty:ty, $fname:ident, ( &mut $this:ident $(, $aname:ident : $aty:ty )* ), $ret_ty:ty, $body:block ) => {
        unsafe extern "C" fn $fname(slf: *mut PyObject,
                                    args: *mut PyObject) -> *mut PyObject {
            let slf = PyRef::new(slf);
            let args = PyRef::new(args);
            let mut guard = unpack_rust_ref_mut::<$ty>(slf);

            fn $fname($this: &mut $ty, ($($aname,)*) : ($($aty,)*)) -> $ret_ty {
                $body
            }

            let result = $fname(&mut *guard, Unpack::unpack(args));
            Pack::pack(result).unwrap()
        }
    };
}

macro_rules! define_func {
    ( $mac:ident, $fname:ident, $args:tt, $ret_ty:ty, $body:block ) => {
        $mac!($fname, $args, $ret_ty, $body)
    };
    ( $mac:ident, $fname:ident, $args:tt, , $body:expr ) => {
        $mac!($fname, $args, (), $body)
    };
}

macro_rules! define_python_class {
    (
        class $name:ident : $ty:ty {
            type_obj $type_obj:ident;
            initializer $init_name:ident;
            accessor $acc_name:ident;
            method_macro $define_func:ident !;
            $(
                fn $fname:ident $args:tt $( -> $ret_ty:ty )* { $( $body:tt )* }
            )*
        }
    ) => {
        static mut $type_obj: *mut PyObject = 0 as *mut _;

        #[allow(unused_assignments)]
        fn $init_name(module: PyRef) {
            unsafe {
                assert!(py::is_initialized());

                const NUM_METHODS: usize = 0 $( + one!($fname) )*;
                static mut TYPE_SPEC: PyType_Spec = BLANK_TYPE_SPEC;
                static mut TYPE_SLOTS: [PyType_Slot; 2] = [BLANK_TYPE_SLOT; 2];
                static mut METHODS: [PyMethodDef; 1 + NUM_METHODS] =
                        [BLANK_METHOD_DEF; 1 + NUM_METHODS];

                TYPE_SPEC = PyType_Spec {
                    name: concat!("_outpost_server.", stringify!($name), "\0")
                            .as_ptr() as *const c_char,
                    basicsize: mem::size_of::<$ty>() as c_int,
                    itemsize: 0,
                    flags: Py_TPFLAGS_DEFAULT as c_uint,
                    slots: TYPE_SLOTS.as_mut_ptr(),
                };

                TYPE_SLOTS[0] = PyType_Slot {
                    slot: Py_tp_methods,
                    pfunc: METHODS.as_mut_ptr() as *mut _,
                };

                let mut i = 0;
                $(
                    define_func!($define_func, $fname, $args, $( $ret_ty )*, { $( $body )* });

                    METHODS[i] = PyMethodDef {
                        ml_name: concat!(stringify!($fname), "\0")
                                .as_ptr() as *const c_char,
                        ml_meth: Some(mem::transmute($fname)),
                        ml_flags: METH_VARARGS,
                        ml_doc: ptr::null(),
                    };
                    i += 1;
                )*

                let type_obj = py::type_::from_spec(&mut TYPE_SPEC);
                py::object::set_attr_str(module, stringify!($name), type_obj.borrow());
                $type_obj = type_obj.unwrap();
            }
        }

        fn $acc_name() -> PyRef<'static> {
            unsafe {
                PyRef::new($type_obj)
            }
        }
    };
}

macro_rules! storage_ref_func {
    ( $($all:tt)* ) => ( rust_ref_func!(Storage, $($all)*) );
}

define_python_class! {
    class StorageRef: RustRef {
        type_obj STORAGE_REF_TYPE;
        initializer storage_ref_init;
        accessor storage_ref_type;
        method_macro storage_ref_func!;

        fn script_dir(&this) -> PyBox {
            py::unicode::from_str(this.script_dir().to_str().unwrap())
        }
    }
}

unsafe impl GetTypeObject for Storage {
    fn get_type_object() -> PyBox {
        storage_ref_type().to_box()
    }
}


macro_rules! data_ref_func {
    ( $($all:tt)* ) => ( rust_ref_func!(Data, $($all)*) );
}

define_python_class! {
    class DataRef: RustRef {
        type_obj DATA_REF_TYPE;
        initializer data_ref_init;
        accessor data_ref_type;
        method_macro data_ref_func!;

        fn item_count(&this) -> usize {
            this.item_data.len()
        }

        fn item_by_name(&this, name: String) -> Option<ItemId> {
            this.item_data.find_id(&name)
        }

        fn item_name(&this, id: ItemId) -> Option<PyBox> {
            this.item_data.get_name(id).map(|s| Pack::pack(s))
        }


        fn recipe_count(&this) -> usize {
            this.recipes.len()
        }

        fn recipe_by_name(&this, name: String) -> Option<RecipeId> {
            this.recipes.find_id(&name)
        }

        fn recipe_name(&this, id: RecipeId) -> Option<PyBox> {
            this.recipes.get_recipe(id).map(|r| Pack::pack(&r.name as &str))
        }

        fn recipe_inputs(&this, id: RecipeId) -> Option<PyBox> {
            this.recipes.get_recipe(id).map(|r| Pack::pack(&r.inputs))
        }

        fn recipe_outputs(&this, id: RecipeId) -> Option<PyBox> {
            this.recipes.get_recipe(id).map(|r| Pack::pack(&r.outputs))
        }

        fn recipe_station(&this, id: RecipeId) -> Option<Option<TemplateId>> {
            this.recipes.get_recipe(id).map(|r| r.station)
        }


        fn template_count(&this) -> usize {
            this.structure_templates.len()
        }

        fn template_by_name(&this, name: String) -> Option<TemplateId> {
            this.structure_templates.find_id(&name)
        }

        fn template_name(&this, id: TemplateId) -> Option<PyBox> {
            this.structure_templates.get_template(id).map(|t| Pack::pack(&t.name as &str))
        }
    }
}

unsafe impl GetTypeObject for Data {
    fn get_type_object() -> PyBox {
        data_ref_type().to_box()
    }
}


struct EngineRef {
    base: PyObject,
    ptr: *mut Engine<'static>,
    flags: usize,
}

macro_rules! engine_ref_func {
    ( $fname:ident,
      ( $aname1:ident : $aty1:path, $( $aname:ident : $aty:ty ),* ),
      $ret_ty:ty,
      $body:block ) => {

        unsafe extern "C" fn $fname(slf: *mut PyObject,
                                    args: *mut PyObject) -> *mut PyObject {
            let slf = PyRef::new(slf);
            let args = PyRef::new(args);
            assert!(py::object::is_instance(slf, engine_ref_type()));

            let er = &mut *(slf.as_ptr() as *mut EngineRef);
            let ref_flags = er.flags;
            let target_flags = <$aty1 as PartFlags>::flags();
            assert!(ref_flags & target_flags == ref_flags);

            fn $fname($aname1: $aty1, ($($aname,)*): ($($aty,)*)) -> $ret_ty {
                $body
            }

            er.flags = 0;
            let result = {
                let target = <$aty1 as Part>::from_ptr(er.ptr);
                $fname(target, Unpack::unpack(args))
            };
            er.flags = ref_flags;
            Pack::pack(result).unwrap()
        }

    };
}

define_python_class! {
    class EngineRef: EngineRef {
        type_obj ENGINE_REF_TYPE;
        initializer engine_ref_init;
        accessor engine_ref_type;
        method_macro engine_ref_func!;

        fn now(eng: EmptyPart,) -> Time {
            eng.now()
        }
    }
}

fn with_engine_ref<E, F>(mut e: E, f: F)
        where E: Part + PartFlags, F: FnOnce(PyRef) {
    unsafe {
        let obj = py::type_::instantiate(engine_ref_type());
        {
            let er = &mut *(obj.as_ptr() as *mut EngineRef);
            er.ptr = e.as_ptr();
            er.flags = <E as PartFlags>::flags();
        }

        f(obj.borrow());

        {
            let er = &mut *(obj.as_ptr() as *mut EngineRef);
            er.flags = 0;
        }
    }
}

engine_part_typedef!(OnlyWorld(world));
engine_part_typedef!(EmptyPart());
