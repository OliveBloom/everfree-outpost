extern crate libc;
extern crate python3_sys;

use std::mem;
use std::ptr;
use std::slice;
use libc::{c_char, c_void, c_int, c_uint};
use python3_sys::*;

use physics::v3::{V2, scalar};

use super::Renderer;


struct PyRenderer {
    base: PyObject,
    obj: Renderer,
}

unsafe extern "C" fn set_base(slf: *mut PyObject, args: *mut PyObject) -> *mut PyObject {
    let mut size: V2 = scalar(0);
    let mut buf: *const u8 = ptr::null();
    let mut len: usize = 0;
    if PyArg_ParseTuple(args,
                        "iiy#\0".as_ptr() as *const c_char,
                        &mut size.x,
                        &mut size.y,
                        &mut buf,
                        &mut len) == 0 {
        return ptr::null_mut();
    }

    let slice = slice::from_raw_parts(buf, len);
    (*(slf as *mut PyRenderer)).obj.set_base(size, slice);

    Py_INCREF(Py_None());
    Py_None()
}

unsafe extern "C" fn render_part(slf: *mut PyObject, args: *mut PyObject) -> *mut PyObject {
    let mut mask_buf: *const u8 = ptr::null();
    let mut mask_len: usize = 0;
    if PyArg_ParseTuple(args,
                        "y#\0".as_ptr() as *const c_char,
                        &mut mask_buf,
                        &mut mask_len) == 0 {
        return ptr::null_mut();
    }

    let mask_slice = slice::from_raw_parts(mask_buf, mask_len);
    (*(slf as *mut PyRenderer)).obj.render_part(mask_slice);

    Py_INCREF(Py_None());
    Py_None()
}

unsafe extern "C" fn get_image(slf: *mut PyObject, args: *mut PyObject) -> *mut PyObject {
    if PyArg_ParseTuple(args,
                        "\0".as_ptr() as *const c_char) == 0 {
        return ptr::null_mut();
    }

    let result = (*(slf as *mut PyRenderer)).obj.get_image();
    PyBytes_FromStringAndSize(result.as_ptr() as *const c_char,
                              result.len() as Py_ssize_t)
}




unsafe extern "C" fn obj_new(subtype: *mut PyTypeObject,
                             args: *mut PyObject,
                             kwds: *mut PyObject) -> *mut PyObject {
    let obj = PyType_GenericNew(subtype, args, kwds) as *mut PyRenderer;
    ptr::write(&mut (*obj).obj, Renderer::new());
    obj as *mut PyObject
}

unsafe extern "C" fn obj_dealloc(obj: *mut PyRenderer) {
    drop(ptr::read(&(*obj).obj));
    PyObject_Free(obj as *mut c_void);
}


static mut TYPE_SPEC: PyType_Spec = PyType_Spec {
    name: 0 as *const _,
    basicsize: 0,
    itemsize: 0,
    flags: 0,
    slots: 0 as *mut _,
};

static mut TYPE_SLOTS: [PyType_Slot; 4] = [PyType_Slot {
    slot: 0,
    pfunc: 0 as *mut _,
}; 4];

static mut METHOD_DEFS: [PyMethodDef; 4] = [PyMethodDef {
    ml_name: 0 as *const _,
    ml_meth: None,
    ml_flags: 0,
    ml_doc: 0 as *const _,
}; 4];


unsafe fn init_type() -> *mut PyObject {
    let mut i = 0;
    macro_rules! method {
        ($name:ident) => {
            {
                let m = &mut METHOD_DEFS[i];
                m.ml_name = concat!(stringify!($name), "\0").as_ptr() as *const c_char;
                m.ml_meth = Some(mem::transmute($name));
                m.ml_flags = METH_VARARGS;
                i += 1;
            }
        };
    };

    method!(set_base);
    method!(render_part);
    method!(get_image);
    assert!(i == METHOD_DEFS.len() - 1);

    {
        let s = &mut TYPE_SLOTS[0];
        s.slot = Py_tp_methods;
        s.pfunc = METHOD_DEFS.as_mut_ptr() as *mut c_void;
    }

    {
        let s = &mut TYPE_SLOTS[1];
        s.slot = Py_tp_new;
        s.pfunc = obj_new as *mut c_void;
    }

    {
        let s = &mut TYPE_SLOTS[2];
        s.slot = Py_tp_dealloc;
        s.pfunc = obj_dealloc as *mut c_void;
    }

    {
        let s = &mut TYPE_SPEC;
        s.name = "Renderer\0".as_ptr() as *const c_char;
        s.basicsize = mem::size_of::<PyRenderer>() as c_int;
        s.flags = Py_TPFLAGS_DEFAULT as c_uint;
        s.slots = TYPE_SLOTS.as_mut_ptr();
    }

    PyType_FromSpec(&mut TYPE_SPEC)
}


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

#[no_mangle]
pub unsafe extern "C" fn PyInit_equip_sprites_render() -> *mut PyObject {
    {
        let m = &mut MODULE_DEF;
        m.m_name = "equip_sprites_renderer\0".as_ptr() as *const c_char;
    }

    let module = PyModule_Create(&mut MODULE_DEF);
    assert!(!module.is_null());

    let ty = init_type();
    assert!(!ty.is_null());

    PyModule_AddObject(module,
                       "Renderer\0".as_ptr() as *const c_char,
                       ty);

    module
}
