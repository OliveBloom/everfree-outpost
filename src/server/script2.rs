use std::marker::PhantomData;
use std::mem;
use std::path::Path;

use py;

#[unsafe_no_drop_flag]
#[allow(raw_pointer_derive)]
#[derive(PartialEq, Eq, Debug)]
pub struct PyBox {
    ptr: *mut py::PyObject,
}

impl PyBox {
    pub unsafe fn new(ptr: *mut py::PyObject) -> PyBox {
        if ptr.is_null() {
            py::PyErr_PrintEx(1);
            assert!(!ptr.is_null());
        }
        PyBox {
            ptr: ptr,
        }
    }

    pub unsafe fn new_opt(ptr: *mut py::PyObject) -> Option<PyBox> {
        if ptr.is_null() {
            None
        } else {
            Some(PyBox::new(ptr))
        }
    }

    pub fn borrow<'a>(&'a self) -> PyRef<'a> {
        unsafe { PyRef::new(self.ptr) }
    }

    pub fn as_ptr(&self) -> *mut py::PyObject {
        self.ptr
    }

    pub fn unwrap(mut self) -> *mut py::PyObject {
        let val = self.ptr;
        self.ptr = mem::POST_DROP_USIZE as *mut py::PyObject;
        val
    }
}

impl Drop for PyBox {
    fn drop(&mut self) {
        if self.ptr as usize != mem::POST_DROP_USIZE {
            unsafe {
                py::Py_DECREF(self.ptr);
                self.ptr = mem::POST_DROP_USIZE as *mut py::PyObject;
            }
        }
    }
}

impl Clone for PyBox {
    fn clone(&self) -> PyBox {
        unsafe {
            py::Py_INCREF(self.ptr);
            PyBox::new(self.ptr)
        }
    }
}


#[allow(raw_pointer_derive)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PyRef<'a> {
    ptr: *mut py::PyObject,
    _marker: PhantomData<&'a ()>,
}

impl<'a> PyRef<'a> {
    pub unsafe fn new(ptr: *mut py::PyObject) -> PyRef<'a> {
        if ptr.is_null() {
            py::PyErr_PrintEx(1);
            assert!(!ptr.is_null());
        }
        PyRef {
            ptr: ptr,
            _marker: PhantomData,
        }
    }

    pub unsafe fn new_opt(ptr: *mut py::PyObject) -> Option<PyRef<'a>> {
        if ptr.is_null() {
            None
        } else {
            Some(PyRef::new(ptr))
        }
    }

    pub fn to_box(&self) -> PyBox {
        unsafe {
            py::Py_INCREF(self.ptr);
            PyBox::new(self.ptr)
        }
    }

    pub fn as_ptr(&self) -> *mut py::PyObject {
        self.ptr
    }
}



pub mod object {
    use std::ffi::{CString, CStr};
    use std::ptr;
    use py;
    use super::{PyBox, PyRef};

    pub fn call(callable: PyRef, args: PyRef, kw: Option<PyRef>) -> PyBox {
        unsafe {
            let kw_ptr = match kw {
                None => ptr::null_mut(),
                Some(kw) => kw.as_ptr(),
            };
            PyBox::new(py::PyObject_Call(callable.as_ptr(), args.as_ptr(), kw_ptr))
        }
    }


    pub fn get_attr_cstr(dct: PyRef, attr_name: &CStr) -> PyBox {
        unsafe {
            PyBox::new(py::PyObject_GetAttrString(dct.as_ptr(), attr_name.as_ptr()))
        }
    }

    pub fn get_attr_str(dct: PyRef, attr_name: &str) -> PyBox {
        get_attr_cstr(dct, &CString::new(attr_name).unwrap())
    }
}

pub mod unicode {
    use py;
    use super::PyBox;

    pub fn from_str(s: &str) -> PyBox {
        assert!(s.len() as u64 <= py::PY_SSIZE_T_MAX as u64);
        unsafe {
            let ptr = py::PyUnicode_FromStringAndSize(s.as_ptr() as *const i8,
                                                      s.len() as py::Py_ssize_t);
            PyBox::new(ptr)
        }
    }
}

pub mod eval {
    use py;
    use super::{PyBox, PyRef};

    pub fn get_builtins() -> PyBox {
        unsafe {
            let ptr = py::PyEval_GetBuiltins();
            PyRef::new(ptr).to_box()
        }
    }
}

pub mod dict {
    use std::ffi::{CString, CStr};
    use py;
    use super::{PyBox, PyRef};

    pub fn new() -> PyBox {
        unsafe {
            PyBox::new(py::PyDict_New())
        }
    }


    pub fn get_item_cstr<'a>(dct: PyRef<'a>, key: &CStr) -> Option<PyRef<'a>> {
        unsafe {
            PyRef::new_opt(py::PyDict_GetItemString(dct.as_ptr(), key.as_ptr()))
        }
    }

    pub fn get_item_str<'a>(dct: PyRef<'a>, key: &str) -> Option<PyRef<'a>> {
        get_item_cstr(dct, &CString::new(key).unwrap())
    }


    pub fn set_item_cstr(dct: PyRef, key: &CStr, val: PyRef) {
        unsafe {
            let ret = py::PyDict_SetItemString(dct.as_ptr(), key.as_ptr(), val.as_ptr());
            assert!(ret == 0);
        }
    }

    pub fn set_item_str(dct: PyRef, key: &str, val: PyRef) {
        set_item_cstr(dct, &CString::new(key).unwrap(), val)
    }
}

pub mod tuple {
    use py;
    use super::{PyBox, PyRef};

    pub fn new(len: usize) -> PyBox {
        assert!(len as u64 <= py::PY_SSIZE_T_MAX as u64);
        unsafe { PyBox::new(py::PyTuple_New(len as py::Py_ssize_t)) }
    }

    pub unsafe fn set_item(t: PyRef, pos: usize, val: PyBox) {
        let ret = py::PyTuple_SetItem(t.as_ptr(), pos as py::Py_ssize_t, val.unwrap());
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



pub fn init() {
    unsafe {
        // Initialize, but don't let Python set up its own signal handlers.
        // This is safe to call multiple times (subsequent calls are no-ops).
        py::Py_InitializeEx(0);
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
        PyBox::new(py::PyImport_Import(name_obj.unwrap()))
    }
}
