use python3_sys::*;

pub use exc::{PyExc, PyResult};
pub use ptr::{PyBox, PyRef};


pub mod object {
    use std::ffi::{CString, CStr};
    use std::ptr;
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub fn call(callable: PyRef, args: PyRef, kw: Option<PyRef>) -> PyResult<PyBox> {
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


    pub fn get_attr_cstr(dct: PyRef, attr_name: &CStr) -> PyResult<PyBox> {
        unsafe {
            PyBox::new(PyObject_GetAttrString(dct.as_ptr(), attr_name.as_ptr()))
        }
    }

    pub fn get_attr_str(dct: PyRef, attr_name: &str) -> PyResult<PyBox> {
        get_attr_cstr(dct, &CString::new(attr_name).unwrap())
    }


    pub fn set_attr_cstr(dct: PyRef, attr_name: &CStr, val: PyRef) -> PyResult<()> {
        unsafe {
            let ret = PyObject_SetAttrString(dct.as_ptr(), attr_name.as_ptr(), val.as_ptr());
            pycheck!(ret == 0);
            Ok(())
        }
    }

    pub fn set_attr_str(dct: PyRef, attr_name: &str, val: PyRef) -> PyResult<()> {
        set_attr_cstr(dct, &CString::new(attr_name).unwrap(), val)
    }

    pub fn repr(obj: PyRef) -> PyResult<String> {
        let s = try!(unsafe { PyBox::new(PyObject_Repr(obj.as_ptr())) });
        super::unicode::as_string(s.borrow())
    }
}

pub mod unicode {
    use std::ptr;
    use std::slice;
    use std::str;
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyUnicode_Check(obj.as_ptr()) != 0 }
    }

    pub fn from_str(s: &str) -> PyResult<PyBox> {
        pyassert!(s.len() as u64 <= PY_SSIZE_T_MAX as u64);
        unsafe {
            let ptr = PyUnicode_FromStringAndSize(s.as_ptr() as *const i8,
                                                  s.len() as Py_ssize_t);
            PyBox::new(ptr)
        }
    }

    pub fn encode_utf8(obj: PyRef) -> PyResult<PyBox> {
        unsafe { PyBox::new(PyUnicode_AsUTF8String(obj.as_ptr())) }
    }

    pub fn as_string(obj: PyRef) -> PyResult<String> {
        let bytes = try!(encode_utf8(obj));
        unsafe {
            let mut ptr = ptr::null_mut();
            let mut len = 0;
            let ret = PyBytes_AsStringAndSize(bytes.as_ptr(), &mut ptr, &mut len);
            pycheck!(ret == 0);
            let bytes = slice::from_raw_parts(ptr as *const u8, len as usize);
            Ok(str::from_utf8_unchecked(bytes).to_owned())
        }
    }
}

pub mod int {
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyLong_Check(obj.as_ptr()) != 0 }
    }

    pub fn from_u64(val: u64) -> PyResult<PyBox> {
        unsafe { PyBox::new(PyLong_FromUnsignedLongLong(val)) }
    }

    pub fn from_i64(val: i64) -> PyResult<PyBox> {
        unsafe { PyBox::new(PyLong_FromLongLong(val)) }
    }

    pub fn as_u64(obj: PyRef) -> PyResult<u64> {
        let val = unsafe { PyLong_AsUnsignedLongLong(obj.as_ptr()) };
        pycheck!();
        Ok(val)
    }

    pub fn as_i64(obj: PyRef) -> PyResult<i64> {
        let val = unsafe { PyLong_AsLongLong(obj.as_ptr()) };
        pycheck!();
        Ok(val)
    }
}

pub mod eval {
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn get_builtins() -> PyBox {
        unsafe {
            let ptr = PyEval_GetBuiltins();
            PyRef::new_non_null(ptr).to_box()
        }
    }
}

pub mod bool {
    use python3_sys::*;
    use super::PyRef;

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyBool_Check(obj.as_ptr()) != 0 }
    }

    pub fn false_() -> PyRef<'static> {
        unsafe { PyRef::new_non_null(Py_False()) }
    }

    pub fn true_() -> PyRef<'static> {
        unsafe { PyRef::new_non_null(Py_True()) }
    }
}

pub mod float {
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyFloat_Check(obj.as_ptr()) != 0 }
    }

    pub fn from_f64(val: f64) -> PyResult<PyBox> {
        unsafe { PyBox::new(PyFloat_FromDouble(val)) }
    }

    pub fn as_f64(obj: PyRef) -> PyResult<f64> {
        let val = unsafe { PyFloat_AsDouble(obj.as_ptr()) };
        pycheck!();
        Ok(val)
    }
}

pub mod dict {
    use std::ffi::{CString, CStr};
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub fn new() -> PyResult<PyBox> {
        unsafe {
            PyBox::new(PyDict_New())
        }
    }

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyDict_Check(obj.as_ptr()) != 0 }
    }

    pub fn get_item_cstr<'a>(dct: PyRef<'a>, key: &CStr) -> PyResult<Option<PyRef<'a>>> {
        let val = unsafe {
            PyRef::new_opt(PyDict_GetItemString(dct.as_ptr(), key.as_ptr()))
        };
        pycheck!();
        Ok(val)
    }

    pub fn get_item_str<'a>(dct: PyRef<'a>, key: &str) -> PyResult<Option<PyRef<'a>>> {
        get_item_cstr(dct, &CString::new(key).unwrap())
    }

    pub fn set_item_cstr(dct: PyRef, key: &CStr, val: PyRef) -> PyResult<()> {
        unsafe {
            let ret = PyDict_SetItemString(dct.as_ptr(), key.as_ptr(), val.as_ptr());
            pycheck!(ret == 0);
            Ok(())
        }
    }

    pub fn set_item_str(dct: PyRef, key: &str, val: PyRef) -> PyResult<()> {
        set_item_cstr(dct, &CString::new(key).unwrap(), val)
    }

    pub fn set_item(dct: PyRef, key: PyRef, val: PyRef) -> PyResult<()> {
        unsafe {
            let ret = PyDict_SetItem(dct.as_ptr(), key.as_ptr(), val.as_ptr());
            pycheck!(ret == 0);
            Ok(())
        }
    }

    /// Return a `list` object containing all the items from the dictionary.
    pub fn items(dct: PyRef) -> PyResult<PyBox> {
        unsafe { PyBox::new(PyDict_Items(dct.as_ptr())) }
    }
}

pub mod list {
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub fn new() -> PyResult<PyBox> {
        unsafe { PyBox::new(PyList_New(0)) }
    }

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyList_Check(obj.as_ptr()) != 0 }
    }

    pub fn size(t: PyRef) -> PyResult<usize> {
        let val = unsafe { PyList_Size(t.as_ptr()) as usize };
        pycheck!();
        Ok(val)
    }

    pub fn get_item<'a>(t: PyRef<'a>, pos: usize) -> PyResult<PyRef<'a>> {
        unsafe { PyRef::new(PyList_GetItem(t.as_ptr(), pos as Py_ssize_t)) }
    }

    pub fn append(l: PyRef, item: PyRef) -> PyResult<()> {
        unsafe {
            let ret = PyList_Append(l.as_ptr(), item.as_ptr());
            pycheck!(ret == 0);
            Ok(())
        }
    }
}

pub mod tuple {
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub fn new(len: usize) -> PyResult<PyBox> {
        pyassert!(len as u64 <= PY_SSIZE_T_MAX as u64);
        unsafe { PyBox::new(PyTuple_New(len as Py_ssize_t)) }
    }

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyTuple_Check(obj.as_ptr()) != 0 }
    }

    pub fn size(t: PyRef) -> PyResult<usize> {
        let val = unsafe { PyTuple_Size(t.as_ptr()) as usize };
        pycheck!();
        Ok(val)
    }

    pub fn get_item<'a>(t: PyRef<'a>, pos: usize) -> PyResult<PyRef<'a>> {
        unsafe { PyRef::new(PyTuple_GetItem(t.as_ptr(), pos as Py_ssize_t)) }
    }

    pub unsafe fn set_item(t: PyRef, pos: usize, val: PyBox) -> PyResult<()> {
        let ret = PyTuple_SetItem(t.as_ptr(), pos as Py_ssize_t, val.unwrap());
        pycheck!(ret == 0);
        Ok(())
    }

    pub fn pack0() -> PyResult<PyBox> {
        let t = try!(new(0));
        Ok(t)
    }

    pub fn pack1(val0: PyBox) -> PyResult<PyBox> {
        unsafe {
            let t = try!(new(1));
            try!(set_item(t.borrow(), 0, val0));
            Ok(t)
        }
    }

    pub fn pack2(val0: PyBox, val1: PyBox) -> PyResult<PyBox> {
        unsafe {
            let t = try!(new(2));
            try!(set_item(t.borrow(), 0, val0));
            try!(set_item(t.borrow(), 1, val1));
            Ok(t)
        }
    }

    pub fn pack3(val0: PyBox, val1: PyBox, val2: PyBox) -> PyResult<PyBox> {
        unsafe {
            let t = try!(new(3));
            try!(set_item(t.borrow(), 0, val0));
            try!(set_item(t.borrow(), 1, val1));
            try!(set_item(t.borrow(), 2, val2));
            Ok(t)
        }
    }

    pub fn pack4(val0: PyBox, val1: PyBox, val2: PyBox, val3: PyBox) -> PyResult<PyBox> {
        unsafe {
            let t = try!(new(4));
            try!(set_item(t.borrow(), 0, val0));
            try!(set_item(t.borrow(), 1, val1));
            try!(set_item(t.borrow(), 2, val2));
            try!(set_item(t.borrow(), 3, val3));
            Ok(t)
        }
    }
}

pub mod iter {
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub fn next(obj: PyRef) -> PyResult<Option<PyBox>> {
        let result = unsafe { PyBox::new_opt(PyIter_Next(obj.as_ptr())) };
        // PyIter_Next returns NULL both for "end of iteration" and "error during iteration".  The
        // two can be distinguished by checking for a pending exception.
        if result.is_none() {
            pycheck!();
        }
        Ok(result)
    }
}

pub mod module {
    use std::ffi::{CString, CStr};
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub fn new_cstr(name: &CStr) -> PyResult<PyBox> {
        unsafe { PyBox::new(PyModule_New(name.as_ptr())) }
    }

    pub fn new(name: &str) -> PyResult<PyBox> {
        new_cstr(&CString::new(name).unwrap())
    }

    pub unsafe fn create(mod_def: *mut PyModuleDef) -> PyResult<PyBox> {
        PyBox::new(PyModule_Create(mod_def))
    }

    pub fn find(mod_def: *mut PyModuleDef) -> PyResult<PyBox> {
        let r = try!(unsafe { PyRef::new(PyState_FindModule(mod_def)) });
        Ok(r.to_box())
    }
}

pub mod type_ {
    use python3_sys::*;
    use super::{PyBox, PyRef, PyResult};

    pub unsafe fn from_spec(spec: *mut PyType_Spec) -> PyResult<PyBox> {
        PyBox::new(PyType_FromSpec(spec))
    }

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyType_Check(obj.as_ptr()) != 0 }
    }

    pub fn instantiate(type_: PyRef) -> PyResult<PyBox> {
        pyassert!(check(type_));
        unsafe { PyBox::new(PyType_GenericAlloc(type_.as_ptr() as *mut PyTypeObject, 0)) }
    }
}

pub mod err {
    use std::ptr;
    use python3_sys::*;
    use super::{PyBox, PyExc};

    pub fn print() {
        unsafe { PyErr_Print() };

        // Call sys.stderr.flush().  This ensures we see the Python stack trace before the Rust one
        // (if any).
        let _ = (|| -> super::PyResult<()> {
            use api::{object, tuple};
            use util::import;

            let sys = try!(import("sys"));
            let stderr = try!(object::get_attr_str(sys.borrow(), "stderr"));
            let flush = try!(object::get_attr_str(stderr.borrow(), "flush"));
            try!(object::call(flush.borrow(), try!(tuple::pack0()).borrow(), None));
            Ok(())
        })();
    }

    pub fn normalize_exception(type_: &mut Option<PyBox>,
                               value: &mut Option<PyBox>,
                               traceback: &mut Option<PyBox>) {
        unsafe {
            let mut raw_type = type_.take().map_or(ptr::null_mut(), |b| b.unwrap());
            let mut raw_value = value.take().map_or(ptr::null_mut(), |b| b.unwrap());
            let mut raw_traceback = traceback.take().map_or(ptr::null_mut(), |b| b.unwrap());
            PyErr_NormalizeException(&mut raw_type, &mut raw_value, &mut raw_traceback);
            *type_ = PyBox::new_opt(raw_type);
            *value = PyBox::new_opt(raw_value);
            *traceback = PyBox::new_opt(raw_traceback);
        }
    }

    pub fn fetch() -> PyExc {
        let mut raw_type = ptr::null_mut();
        let mut raw_value = ptr::null_mut();
        let mut raw_traceback = ptr::null_mut();
        unsafe { 
            PyErr_Fetch(&mut raw_type, &mut raw_value, &mut raw_traceback);
            if raw_type.is_null() {
                raw_type = super::exc::system_error().to_box().unwrap();
            }
            let exc = PyExc::from_python(PyBox::new_opt(raw_type),
                                         PyBox::new_opt(raw_value),
                                         PyBox::new_opt(raw_traceback));

            // TODO: only do this for exceptions not caught (i.e., those that flow into an unwrap()
            // or warn_on_err!()).
            warn!("caught python exception: {}", exc.get_rust().msg);
            if let Some(ref tb) = exc.get_python().traceback {
                super::traceback::print(tb.borrow());
            }

            exc
        }
    }

    pub fn occurred() -> bool {
        unsafe { !PyErr_Occurred().is_null() }
    }

    pub fn raise(exc: PyExc) {
        let exc = exc.unwrap_python_raw();
        unsafe {
            let raw_type = exc.type_.map_or(ptr::null_mut(), |b| b.unwrap());
            let raw_value = exc.value.map_or(ptr::null_mut(), |b| b.unwrap());
            let raw_traceback = exc.traceback.map_or(ptr::null_mut(), |b| b.unwrap());
            PyErr_Restore(raw_type, raw_value, raw_traceback);
        }
    }
}

pub mod traceback {
    use python3_sys::*;
    use super::{PyRef, PyResult};

    fn print_(tb: PyRef) -> PyResult<()> {
        use api::{object, tuple};
        use util::import;

        let sys = try!(import("sys"));
        let stderr = try!(object::get_attr_str(sys.borrow(), "stderr"));
        let ret = unsafe { PyTraceBack_Print(tb.as_ptr(), stderr.as_ptr()) };
        pycheck!(ret == 0);

        let flush = try!(object::get_attr_str(stderr.borrow(), "flush"));
        let unit = try!(tuple::pack0());
        try!(object::call(flush.borrow(), unit.borrow(), None));

        Ok(())
    }

    pub fn print(tb: PyRef) {
        warn_on_err!(print_(tb));
    }
}

pub mod exc {
    use python3_sys::*;
    use super::PyRef;

    pub fn system_error() -> PyRef<'static> {
        unsafe { PyRef::new_non_null(PyExc_SystemError) }
    }

    pub fn runtime_error() -> PyRef<'static> {
        unsafe { PyRef::new_non_null(PyExc_RuntimeError) }
    }

    pub fn type_error() -> PyRef<'static> {
        unsafe { PyRef::new_non_null(PyExc_TypeError) }
    }

    pub fn key_error() -> PyRef<'static> {
        unsafe { PyRef::new_non_null(PyExc_KeyError) }
    }

    pub fn value_error() -> PyRef<'static> {
        unsafe { PyRef::new_non_null(PyExc_ValueError) }
    }

    pub fn index_error() -> PyRef<'static> {
        unsafe { PyRef::new_non_null(PyExc_IndexError) }
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
    unsafe { PyRef::new_non_null(Py_None()) }
}

pub fn not_implemented() -> PyRef<'static> {
    unsafe { PyRef::new_non_null(Py_NotImplemented()) }
}

