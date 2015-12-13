use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use libc::{c_char, c_int, c_uint, c_void};

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
        let x = PyImport_AppendInittab(MOD_NAME.as_ptr() as *const c_char,
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

impl<'a> Unpack<'a> for () {
    fn unpack(obj: PyRef<'a>) -> () {
        assert!(py::tuple::check(obj));
        assert!(py::tuple::size(obj) == 0);
    }
}

impl<'a, A> Unpack<'a> for (A,)
        where A: Unpack<'a> {
    fn unpack(obj: PyRef<'a>) -> (A,) {
        assert!(py::tuple::check(obj));
        assert!(py::tuple::size(obj) == 1);
        (Unpack::unpack(py::tuple::get_item(obj, 0)),
        )
    }
}

impl<'a, A, B> Unpack<'a> for (A, B)
        where A: Unpack<'a>,
              B: Unpack<'a> {
    fn unpack(obj: PyRef<'a>) -> (A, B) {
        assert!(py::tuple::check(obj));
        assert!(py::tuple::size(obj) == 2);
        (Unpack::unpack(py::tuple::get_item(obj, 0)),
         Unpack::unpack(py::tuple::get_item(obj, 1)),
        )
    }
}

impl<'a, A, B, C> Unpack<'a> for (A, B, C)
        where A: Unpack<'a>,
              B: Unpack<'a>,
              C: Unpack<'a> {
    fn unpack(obj: PyRef<'a>) -> (A, B, C) {
        assert!(py::tuple::check(obj));
        assert!(py::tuple::size(obj) == 3);
        (Unpack::unpack(py::tuple::get_item(obj, 0)),
         Unpack::unpack(py::tuple::get_item(obj, 1)),
         Unpack::unpack(py::tuple::get_item(obj, 2)),
        )
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

impl Pack for () {
    fn pack(self) -> PyBox {
        py::tuple::pack0()
    }
}

impl<A> Pack for (A,)
        where A: Pack {
    fn pack(self) -> PyBox {
        let (a,) = self;
        py::tuple::pack1(
            Pack::pack(a),
        )
    }
}

impl<A, B> Pack for (A, B)
        where A: Pack,
              B: Pack {
    fn pack(self) -> PyBox {
        let (a, b) = self;
        py::tuple::pack2(
            Pack::pack(a),
            Pack::pack(b),
        )
    }
}

impl<A, B, C> Pack for (A, B, C)
        where A: Pack,
              B: Pack,
              C: Pack {
    fn pack(self) -> PyBox {
        let (a, b, c) = self;
        py::tuple::pack3(
            Pack::pack(a),
            Pack::pack(b),
            Pack::pack(c),
        )
    }
}


// Storage ref definition

macro_rules! one {
    ( $any:tt ) => { 1 };
}

macro_rules! define_rust_ref_func2 {
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

macro_rules! define_rust_ref_func {
    ( $ty:ty, $fname:ident, $args:tt, $ret_ty:ty, $body:block ) => {
        define_rust_ref_func2!($ty, $fname, $args, $ret_ty, $body)
    };
    ( $ty:ty, $fname:ident, $args:tt, , $body:expr ) => {
        define_rust_ref_func2!($ty, $fname, $args, (), $body)
    };
}

macro_rules! define_rust_ref {
    (
        class $name:ident : $ty:ty {
            initializer $init_name:ident;
            accessor $acc_name:ident;
            $(
                fn $fname:ident $args:tt $( -> $ret_ty:ty )* { $( $body:tt )* }
            )*
        }
    ) => {
        static mut TYPE_OBJ: *mut PyObject = 0 as *mut _;

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
                    basicsize: mem::size_of::<RustRef>() as c_int,
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
                    define_rust_ref_func!($ty, $fname, $args, $( $ret_ty )*, { $( $body )* });

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
                TYPE_OBJ = type_obj.unwrap();
            }
        }

        fn $acc_name() -> PyRef<'static> {
            unsafe {
                PyRef::new(TYPE_OBJ)
            }
        }

        unsafe impl GetTypeObject for $ty {
            fn get_type_object() -> PyBox {
                $acc_name().to_box()
            }
        }
    };
}

define_rust_ref! {
    class StorageRef: ::storage::Storage {
        initializer storage_ref_init;
        accessor storage_ref_type;

        fn script_dir(&this) -> PyBox {
            py::unicode::from_str(this.script_dir().to_str().unwrap())
        }
    }
}


