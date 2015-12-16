use libc::c_char;
use python3_sys::*;

use engine::split::Part;
use python as py;
use python::PyRef;

pub use self::hooks::ScriptHooks;
pub use self::pack::{Pack, Unpack};
pub use self::rust_ref::{with_ref, with_ref_mut};

#[macro_use] mod class;
#[macro_use] mod rust_ref;
mod pack;

mod data;
mod hooks;
mod storage;
mod engine;



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
    py::import(&MOD_NAME[.. MOD_NAME.len() - 1]);
}

static mut FFI_MODULE: *mut PyObject = 0 as *mut _;

extern "C" fn ffi_module_init() -> *mut PyObject {
    unsafe {
        FFI_MOD_DEF.m_name = MOD_NAME.as_ptr() as *const c_char;
        FFI_MOD_DEF.m_methods = FFI_METHOD_DEFS.as_mut_ptr();

        let module = py::module::create(&mut FFI_MOD_DEF);

        data::init(module.borrow());
        hooks::init(module.borrow());
        storage::init(module.borrow());

        engine::init(module.borrow());

        FFI_MODULE = module.clone().unwrap();
        module.unwrap()
    }
}

fn ffi_module() -> PyRef<'static> {
    unsafe { PyRef::new(FFI_MODULE) }
}

pub fn call_init(data: PyRef, storage: PyRef, hooks: PyRef) {
    let module = py::import("outpost_server.core.init");
    let func = py::object::get_attr_str(module.borrow(), "init");
    let args = py::tuple::pack3(data.to_box(), storage.to_box(), hooks.to_box());
    py::object::call(func.borrow(), args.borrow(), None);
}


// RustRef and related functionality







// Storage ref definition




