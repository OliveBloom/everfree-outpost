use std::mem;
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

        FFI_MOD_DEF.m_name = MOD_NAME.as_ptr() as *const c_char;
        FFI_MOD_DEF.m_methods = FFI_METHOD_DEFS.as_mut_ptr();

        FFI_METHOD_DEFS[0] = PyMethodDef {
            ml_name: "test_func\0".as_ptr() as *const c_char,
            ml_meth: Some(unsafe { mem::transmute(test_func) }),
            ml_flags: METH_NOARGS,
            ml_doc: ptr::null(),
        };

        PyImport_AppendInittab(FFI_MOD_DEF.m_name, Some(ffi_module_init));
    }
}

extern "C" fn ffi_module_init() -> *mut PyObject {
    let m = unsafe { py::module::create(&mut FFI_MOD_DEF) };
    let rust_ref_type = build_rust_ref_type();
    py::object::set_attr_str(m.borrow(), "RustRef", rust_ref_type.borrow());
    m.unwrap()
}

fn get_ffi_module() -> PyBox {
    py::module::find(unsafe { &mut FFI_MOD_DEF })
}


pub struct RustRef {
    base: PyObject,
    ptr: *mut c_void,
    mutable: bool,
    valid: bool,
}

static mut RUST_REF_SPEC: PyType_Spec = PyType_Spec {
    name: 0 as *const _,
    basicsize: 0,
    itemsize: 0,
    flags: 0,
    slots: 0 as *mut _,
};

static mut RUST_REF_SLOTS: [PyType_Slot; 1] = [BLANK_TYPE_SLOT; 1];

static mut RUST_REF_METHODS: [PyMethodDef; 1] = [BLANK_METHOD_DEF; 1];

fn build_rust_ref_type() -> PyBox {
    unsafe {
        RUST_REF_SPEC = PyType_Spec {
            name: "RustRef\0".as_ptr() as *const c_char,
            basicsize: mem::size_of::<RustRef>() as c_int,
            itemsize: 0,
            flags: (Py_TPFLAGS_DEFAULT | Py_TPFLAGS_BASETYPE | Py_TPFLAGS_HEAPTYPE) as c_uint,
            slots: RUST_REF_SLOTS.as_mut_ptr(),
        };

        py::type_::from_spec(&mut RUST_REF_SPEC)
    }
}


unsafe extern "C" fn test_func(obj: *mut PyObject) -> *mut PyObject {
    let m = get_ffi_module();
    let ty = py::object::get_attr_str(m.borrow(), "RustRef");
    py::type_::instantiate(ty.borrow()).unwrap()
}
