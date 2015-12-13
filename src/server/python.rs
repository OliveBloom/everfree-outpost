use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use core::nonzero::NonZero;

use python3_sys::*;

#[unsafe_no_drop_flag]
#[allow(raw_pointer_derive)]
#[derive(PartialEq, Eq, Debug)]
pub struct PyBox {
    ptr: NonZero<*mut PyObject>,
}

impl PyBox {
    pub unsafe fn new(ptr: *mut PyObject) -> PyBox {
        if ptr.is_null() {
            PyErr_PrintEx(1);
            assert!(!ptr.is_null());
        }
        PyBox {
            ptr: NonZero::new(ptr),
        }
    }

    pub unsafe fn new_opt(ptr: *mut PyObject) -> Option<PyBox> {
        if ptr.is_null() {
            None
        } else {
            Some(PyBox::new(ptr))
        }
    }

    pub fn as_ptr(&self) -> *mut PyObject {
        *self.ptr
    }

    pub fn borrow<'a>(&'a self) -> PyRef<'a> {
        unsafe { PyRef::new(self.as_ptr()) }
    }

    pub fn unwrap(mut self) -> *mut PyObject {
        unsafe {
            let val = self.as_ptr();
            self.ptr = NonZero::new(mem::POST_DROP_USIZE as *mut PyObject);
            val
        }
    }
}

impl Drop for PyBox {
    fn drop(&mut self) {
        let ptr = self.as_ptr();
        if ptr as usize != mem::POST_DROP_USIZE {
            unsafe {
                Py_DECREF(ptr);
                self.ptr = NonZero::new(mem::POST_DROP_USIZE as *mut PyObject);
            }
        }
    }
}

impl Clone for PyBox {
    fn clone(&self) -> PyBox {
        let ptr = self.as_ptr();
        unsafe {
            Py_INCREF(ptr);
            PyBox::new(ptr)
        }
    }
}


#[allow(raw_pointer_derive)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PyRef<'a> {
    ptr: NonZero<*mut PyObject>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> PyRef<'a> {
    pub unsafe fn new(ptr: *mut PyObject) -> PyRef<'a> {
        if ptr.is_null() {
            PyErr_PrintEx(1);
            assert!(!ptr.is_null());
        }
        PyRef {
            ptr: NonZero::new(ptr),
            _marker: PhantomData,
        }
    }

    pub unsafe fn new_opt(ptr: *mut PyObject) -> Option<PyRef<'a>> {
        if ptr.is_null() {
            None
        } else {
            Some(PyRef::new(ptr))
        }
    }

    pub fn as_ptr(&self) -> *mut PyObject {
        *self.ptr
    }

    pub fn to_box(&self) -> PyBox {
        let ptr = self.as_ptr();
        unsafe {
            Py_INCREF(ptr);
            PyBox::new(ptr)
        }
    }
}



pub mod object {
    use std::ffi::{CString, CStr};
    use std::ptr;
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn call(callable: PyRef, args: PyRef, kw: Option<PyRef>) -> PyBox {
        unsafe {
            let kw_ptr = match kw {
                None => ptr::null_mut(),
                Some(kw) => kw.as_ptr(),
            };
            PyBox::new(PyObject_Call(callable.as_ptr(), args.as_ptr(), kw_ptr))
        }
    }

    pub fn is_instance(obj: PyRef, cls: PyRef) -> bool {
        unsafe { PyObject_IsInstance(obj.as_ptr(), cls.as_ptr()) != 0 }
    }


    pub fn get_attr_cstr(dct: PyRef, attr_name: &CStr) -> PyBox {
        unsafe {
            PyBox::new(PyObject_GetAttrString(dct.as_ptr(), attr_name.as_ptr()))
        }
    }

    pub fn get_attr_str(dct: PyRef, attr_name: &str) -> PyBox {
        get_attr_cstr(dct, &CString::new(attr_name).unwrap())
    }


    pub fn set_attr_cstr(dct: PyRef, attr_name: &CStr, val: PyRef) {
        unsafe {
            let ret = PyObject_SetAttrString(dct.as_ptr(), attr_name.as_ptr(), val.as_ptr());
            assert!(ret == 0);
        }
    }

    pub fn set_attr_str(dct: PyRef, attr_name: &str, val: PyRef) {
        set_attr_cstr(dct, &CString::new(attr_name).unwrap(), val)
    }
}

pub mod unicode {
    use python3_sys::*;
    use super::PyBox;

    pub fn from_str(s: &str) -> PyBox {
        assert!(s.len() as u64 <= PY_SSIZE_T_MAX as u64);
        unsafe {
            let ptr = PyUnicode_FromStringAndSize(s.as_ptr() as *const i8,
                                                      s.len() as Py_ssize_t);
            PyBox::new(ptr)
        }
    }
}

pub mod eval {
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn get_builtins() -> PyBox {
        unsafe {
            let ptr = PyEval_GetBuiltins();
            PyRef::new(ptr).to_box()
        }
    }
}

pub mod dict {
    use std::ffi::{CString, CStr};
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn new() -> PyBox {
        unsafe {
            PyBox::new(PyDict_New())
        }
    }


    pub fn get_item_cstr<'a>(dct: PyRef<'a>, key: &CStr) -> Option<PyRef<'a>> {
        unsafe {
            PyRef::new_opt(PyDict_GetItemString(dct.as_ptr(), key.as_ptr()))
        }
    }

    pub fn get_item_str<'a>(dct: PyRef<'a>, key: &str) -> Option<PyRef<'a>> {
        get_item_cstr(dct, &CString::new(key).unwrap())
    }


    pub fn set_item_cstr(dct: PyRef, key: &CStr, val: PyRef) {
        unsafe {
            let ret = PyDict_SetItemString(dct.as_ptr(), key.as_ptr(), val.as_ptr());
            assert!(ret == 0);
        }
    }

    pub fn set_item_str(dct: PyRef, key: &str, val: PyRef) {
        set_item_cstr(dct, &CString::new(key).unwrap(), val)
    }
}

pub mod tuple {
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn new(len: usize) -> PyBox {
        assert!(len as u64 <= PY_SSIZE_T_MAX as u64);
        unsafe { PyBox::new(PyTuple_New(len as Py_ssize_t)) }
    }

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyTuple_Check(obj.as_ptr()) != 0 }
    }

    pub fn size(t: PyRef) -> usize {
        unsafe { PyTuple_Size(t.as_ptr()) as usize }
    }

    pub fn get_item<'a>(t: PyRef<'a>, pos: usize) -> PyRef<'a> {
        unsafe { PyRef::new(PyTuple_GetItem(t.as_ptr(), pos as Py_ssize_t)) }
    }

    pub unsafe fn set_item(t: PyRef, pos: usize, val: PyBox) {
        let ret = PyTuple_SetItem(t.as_ptr(), pos as Py_ssize_t, val.unwrap());
        assert!(ret == 0);
    }

    pub fn pack0() -> PyBox {
        let t = new(0);
        t
    }

    pub fn pack1(val0: PyBox) -> PyBox {
        unsafe {
            let t = new(1);
            set_item(t.borrow(), 0, val0);
            t
        }
    }

    pub fn pack2(val0: PyBox, val1: PyBox) -> PyBox {
        unsafe {
            let t = new(2);
            set_item(t.borrow(), 0, val0);
            set_item(t.borrow(), 1, val1);
            t
        }
    }

    pub fn pack3(val0: PyBox, val1: PyBox, val2: PyBox) -> PyBox {
        unsafe {
            let t = new(3);
            set_item(t.borrow(), 0, val0);
            set_item(t.borrow(), 1, val1);
            set_item(t.borrow(), 2, val2);
            t
        }
    }
}

pub mod module {
    use std::ffi::{CString, CStr};
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn new_cstr(name: &CStr) -> PyBox {
        unsafe { PyBox::new(PyModule_New(name.as_ptr())) }
    }

    pub fn new(name: &str) -> PyBox {
        new_cstr(&CString::new(name).unwrap())
    }

    pub unsafe fn create(mod_def: *mut PyModuleDef) -> PyBox {
        PyBox::new(PyModule_Create(mod_def))
    }

    pub fn find(mod_def: *mut PyModuleDef) -> PyBox {
        unsafe { PyRef::new(PyState_FindModule(mod_def)).to_box() }
    }
}

pub mod type_ {
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub unsafe fn from_spec(spec: *mut PyType_Spec) -> PyBox {
        PyBox::new(PyType_FromSpec(spec))
    }

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyType_Check(obj.as_ptr()) != 0 }
    }

    pub fn instantiate(type_: PyRef) -> PyBox {
        assert!(check(type_));
        unsafe { PyBox::new(PyType_GenericAlloc(type_.as_ptr() as *mut PyTypeObject, 0)) }
    }
}


pub fn eval(code: &str) -> PyBox {
    let builtins = eval::get_builtins();
    let eval = dict::get_item_str(builtins.borrow(), "eval")
        .expect("missing `eval` in `__builtins__`");
    let args = tuple::pack3(unicode::from_str(code), dict::new(), dict::new());
    object::call(eval, args.borrow(), None)
}

pub fn exec(code: &str) -> PyBox {
    let builtins = eval::get_builtins();
    let exec = dict::get_item_str(builtins.borrow(), "exec")
        .expect("missing `exec` in `__builtins__`");
    let args = tuple::pack3(unicode::from_str(code), dict::new(), dict::new());
    object::call(exec, args.borrow(), None)
}

pub fn run_file(path: &Path) {
    let builtins = eval::get_builtins();
    let compile = dict::get_item_str(builtins.borrow(), "compile")
        .expect("missing `compile` in `__builtins__`");
    let exec = dict::get_item_str(builtins.borrow(), "exec")
        .expect("missing `exec` in `__builtins__`");

    // Compile this little runner program to a code object.  The runner does the actual work of
    // opening and reading the indicated file.
    let runner = unicode::from_str(r#"if True:  # indentation hack
        import sys
        dct = sys.modules['__main__'].__dict__
        dct['__file__'] = filename
        with open(filename, 'r') as f:
            code = compile(f.read(), filename, 'exec')
            exec(code, dct, dct)
        "#);
    let compile_args = tuple::pack3(runner,
                                    unicode::from_str("<runner>"),
                                    unicode::from_str("exec"));
    let runner_code = object::call(compile, compile_args.borrow(), None);

    // Now `exec` the compiled runner.  We don't call `exec` directly on `runner` because `exec`
    // doesn't allow for setting the filename.
    let globals = dict::new();
    let locals = dict::new();
    // TODO: be smarter about non-UTF8 Path encodings
    dict::set_item_str(locals.borrow(),
                       "filename",
                       unicode::from_str(path.to_str().unwrap()).borrow());
    let args = tuple::pack3(runner_code, globals, locals);
    object::call(exec, args.borrow(), None);
}

pub fn import(name: &str) -> PyBox {
    let name_obj = unicode::from_str(name);
    unsafe {
        PyBox::new(PyImport_Import(name_obj.unwrap()))
    }
}

pub fn initialize() {
    unsafe { Py_InitializeEx(0) };
}

pub fn is_initialized() -> bool {
    unsafe { Py_IsInitialized() != 0 }
}

pub unsafe fn finalize() {
    Py_Finalize();
}

pub fn none() -> PyRef<'static> {
    unsafe { PyRef::new(Py_None()) }
}
