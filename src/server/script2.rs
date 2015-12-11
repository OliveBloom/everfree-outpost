use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use libc::{c_char, c_int, c_uint, c_void};

use python as py;
use python::{PyBox, PyRef};
use python3_sys::*;


const BLANK_METHOD_DEF: PyMethodDef = PyMethodDef {
    ml_name: 0 as *const _,
    ml_meth: None,
    ml_flags: 0,
    ml_doc: 0 as *const _,
};

const BLANK_TYPE_SLOT: PyType_Slot = PyType_Slot {
    slot: 0,
    pfunc: 0 as *mut _,
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

        let rust_ref_type = build_rust_ref_type();
        py::object::set_attr_str(module.borrow(), "RustRef", rust_ref_type.borrow());

        FFI_MODULE = module.clone().unwrap();
        RUST_REF_TYPE = rust_ref_type.clone().unwrap();
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


// Storage ref definition

static mut RUST_REF_TYPE: *mut PyObject = 0 as *mut _;

static mut RUST_REF_SPEC: PyType_Spec = PyType_Spec {
    name: 0 as *const _,
    basicsize: 0,
    itemsize: 0,
    flags: 0,
    slots: 0 as *mut _,
};

static mut RUST_REF_SLOTS: [PyType_Slot; 2] = [BLANK_TYPE_SLOT; 2];

static mut RUST_REF_METHODS: [PyMethodDef; 2] = [BLANK_METHOD_DEF; 2];

fn build_rust_ref_type() -> PyBox {
    unsafe {
        RUST_REF_SPEC = PyType_Spec {
            name: "_outpost_server.RustRef\0".as_ptr() as *const c_char,
            basicsize: mem::size_of::<RustRef>() as c_int,
            itemsize: 0,
            flags: (Py_TPFLAGS_DEFAULT | Py_TPFLAGS_BASETYPE) as c_uint,
            slots: RUST_REF_SLOTS.as_mut_ptr(),
        };

        RUST_REF_METHODS[0] = PyMethodDef {
            ml_name: "test_method\0".as_ptr() as *const c_char,
            ml_meth: Some(unsafe { mem::transmute(test_ref_method) }),
            ml_flags: METH_NOARGS,
            ml_doc: ptr::null(),
        };

        RUST_REF_SLOTS[0] = PyType_Slot {
            slot: Py_tp_methods,
            pfunc: RUST_REF_METHODS.as_mut_ptr() as *mut _,
        };

        py::type_::from_spec(&mut RUST_REF_SPEC)
    }
}

fn rust_ref_type() -> PyRef<'static> {
    unsafe { PyRef::new(RUST_REF_TYPE) }
}

unsafe impl GetTypeObject for ::storage::Storage {
    fn get_type_object() -> PyBox {
        rust_ref_type().to_box()
    }
}

unsafe extern "C" fn test_ref_method(obj: *mut PyObject) -> *mut PyObject {
    let guard = unpack_rust_ref::<::storage::Storage>(PyRef::new(obj));
    let path = guard.script_dir();
    py::unicode::from_str(path.to_str().unwrap()).unwrap()
}


