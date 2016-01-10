use libc::c_char;
use python3_sys::*;
use python3_sys::structmember::PyMemberDef;

use engine::split::Part;
use python as py;
use python::{PyRef, PyResult};

pub use self::hooks::ScriptHooks;
pub use self::pack::{Pack, Unpack};
pub use self::rust_ref::{with_ref, with_ref_mut};

#[macro_use] mod class;
#[macro_use] mod rust_ref;
mod pack;
mod util;

mod v3;
mod engine;
mod data;
mod hooks;
mod storage;
mod types;
mod extra_arg;



pub const BLANK_TYPE_SPEC: PyType_Spec = PyType_Spec {
    name: 0 as *const _,
    basicsize: 0,
    itemsize: 0,
    flags: 0,
    slots: 0 as *mut _,
};

pub const BLANK_TYPE_SLOT: PyType_Slot = PyType_Slot {
    slot: 0,
    pfunc: 0 as *mut _,
};

pub const BLANK_MEMBER_DEF: PyMemberDef = PyMemberDef {
    name: 0 as *mut _,
    type_code: 0,
    offset: 0,
    flags: 0,
    doc: 0 as *mut _,
};

pub const BLANK_METHOD_DEF: PyMethodDef = PyMethodDef {
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

static mut FFI_METHOD_DEFS: [PyMethodDef; 1] = [BLANK_METHOD_DEF; 1];

pub fn ffi_module_preinit() {
    unsafe {
        assert!(!py::is_initialized());
        PyImport_AppendInittab(MOD_NAME.as_ptr() as *const c_char,
                               Some(ffi_module_init));
    }
}

pub fn ffi_module_postinit() {
    py::import(&MOD_NAME[.. MOD_NAME.len() - 1]).unwrap();
}

static mut FFI_MODULE: *mut PyObject = 0 as *mut _;

extern "C" fn ffi_module_init() -> *mut PyObject {
    unsafe {
        FFI_MOD_DEF.m_name = MOD_NAME.as_ptr() as *const c_char;
        FFI_MOD_DEF.m_methods = FFI_METHOD_DEFS.as_mut_ptr();

        let module = py::module::create(&mut FFI_MOD_DEF).unwrap();

        data::init(module.borrow());
        hooks::init(module.borrow());
        storage::init(module.borrow());
        engine::init(module.borrow());
        v3::init(module.borrow());
        types::init(module.borrow());

        FFI_MODULE = module.clone().unwrap();
        module.unwrap()
    }
}

fn ffi_module() -> PyRef<'static> {
    unsafe { PyRef::new_non_null(FFI_MODULE) }
}

pub fn call_init(data: PyRef, storage: PyRef, hooks: PyRef) -> PyResult<()> {
    let module = try!(py::import("outpost_server.core.init"));
    let func = try!(py::object::get_attr_str(module.borrow(), "init"));
    let args = try!(py::tuple::pack3(data.to_box(), storage.to_box(), hooks.to_box()));
    try!(py::object::call(func.borrow(), args.borrow(), None));
    Ok(())
}
