use std::cell::RefCell;
use std::convert::From;
use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use std::ptr;
use core::nonzero::NonZero;

use python3_sys::*;

use util::StrError;

#[unsafe_no_drop_flag]
#[allow(raw_pointer_derive)]
#[derive(PartialEq, Eq, Debug)]
pub struct PyBox {
    ptr: NonZero<*mut PyObject>,
}

impl PyBox {
    pub unsafe fn new_non_null(ptr: *mut PyObject) -> PyBox {
        assert!(!ptr.is_null());
        PyBox {
            ptr: NonZero::new(ptr),
        }
    }

    pub unsafe fn new(ptr: *mut PyObject) -> PyResult<PyBox> {
        if ptr.is_null() {
            return Err(Box::new(err::fetch()));
        }
        Ok(PyBox::new_non_null(ptr))
    }

    pub unsafe fn new_opt(ptr: *mut PyObject) -> Option<PyBox> {
        if ptr.is_null() {
            None
        } else {
            Some(PyBox::new_non_null(ptr))
        }
    }

    pub fn as_ptr(&self) -> *mut PyObject {
        *self.ptr
    }

    pub fn borrow<'a>(&'a self) -> PyRef<'a> {
        unsafe { PyRef::new_non_null(self.as_ptr()) }
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
            PyBox::new_non_null(ptr)
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
    pub unsafe fn new_non_null(ptr: *mut PyObject) -> PyRef<'a> {
        assert!(!ptr.is_null());
        PyRef {
            ptr: NonZero::new(ptr),
            _marker: PhantomData,
        }
    }

    pub unsafe fn new(ptr: *mut PyObject) -> PyResult<PyRef<'a>> {
        if ptr.is_null() {
            return Err(Box::new(err::fetch()));
        }
        Ok(PyRef::new_non_null(ptr))
    }

    pub unsafe fn new_opt(ptr: *mut PyObject) -> Option<PyRef<'a>> {
        if ptr.is_null() {
            None
        } else {
            Some(PyRef::new_non_null(ptr))
        }
    }

    pub fn as_ptr(&self) -> *mut PyObject {
        *self.ptr
    }

    pub fn to_box(&self) -> PyBox {
        let ptr = self.as_ptr();
        unsafe {
            Py_INCREF(ptr);
            PyBox::new_non_null(ptr)
        }
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
struct ExcPython {
    type_: Option<PyBox>,
    value: Option<PyBox>,
    traceback: Option<PyBox>,
    normalized: bool,
}

impl ExcPython {
    fn from_rust(exc: &ExcRust) -> ExcPython {
        let type_ = exc.type_.clone();
        let result = (|| {
            let msg = try!(unicode::from_str(&exc.msg));
            let args = try!(tuple::pack1(msg));
            object::call(type_.borrow(), args.borrow(), None)
        })();
        let opt_value = match result {
            Ok(x) => Some(x),
            Err(e) => {
                warn!("from_rust: got exception {}", e);
                None
            },
        };

        ExcPython {
            type_: Some(type_),
            value: opt_value,
            traceback: None,
            normalized: false,
        }
    }

    fn normalize(&mut self) {
        err::normalize_exception(&mut self.type_, &mut self.value, &mut self.traceback);
        self.normalized = true;
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct ExcRust {
    type_: PyBox,
    msg: String,
}

impl ExcRust {
    fn from_python(exc: &ExcPython) -> ExcRust {
        let type_str =
            if let Some(ref type_) = exc.type_ {
                match object::repr(type_.borrow()) {
                    Ok(s) => s,
                    Err(e) => format!("[error: {}]", e),
                }
            } else {
                "<no type>".to_owned()
            };

        let value_str =
            if let Some(ref value) = exc.value {
                match object::repr(value.borrow()) {
                    Ok(s) => s,
                    Err(e) => format!("[error: {}]", e),
                }
            } else {
                "<no value>".to_owned()
            };

        let msg = format!("{}: {}", type_str, value_str);
        let type_ = exc.type_.clone().unwrap_or_else(|| exc::system_error().to_box());

        ExcRust {
            type_: type_,
            msg: msg,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PyExc {
    python: RefCell<Option<ExcPython>>,
    rust: RefCell<Option<ExcRust>>,
}

impl PyExc {
    pub fn new(ty: PyRef, msg: String) -> PyExc {
        let rust = ExcRust {
            type_: ty.to_box(),
            msg: msg,
        };

        PyExc {
            python: RefCell::new(None),
            rust: RefCell::new(Some(rust)),
        }
    }

    fn from_python(type_: Option<PyBox>,
                   value: Option<PyBox>,
                   traceback: Option<PyBox>) -> PyExc {
        let python = ExcPython {
            type_: type_,
            value: value,
            traceback: traceback,
            normalized: false,
        };

        PyExc {
            python: RefCell::new(Some(python)),
            rust: RefCell::new(None),
        }
    }

    fn ensure_python(&self) {
        let mut py = self.python.borrow_mut();
        if py.is_none() {
            *py = Some(ExcPython::from_rust(self.get_rust()));
        }
    }

    fn ensure_python_normalized(&self) {
        let mut py = self.python.borrow_mut();
        if py.is_none() {
            *py = Some(ExcPython::from_rust(self.get_rust()));
        }
        let py_ref = py.as_mut().unwrap();
        if !py_ref.normalized {
            py_ref.normalize();
        }
    }

    fn ensure_rust(&self) {
        let mut r = self.rust.borrow_mut();
        if r.is_none() {
            *r = Some(ExcRust::from_python(self.get_python()));
        }
    }


    fn get_raw_python(&self) -> &ExcPython {
        self.ensure_python();
        unsafe { mem::transmute(self.python.borrow().as_ref().unwrap() as &ExcPython) }
    }

    fn get_python(&self) -> &ExcPython {
        self.ensure_python_normalized();
        unsafe { mem::transmute(self.python.borrow().as_ref().unwrap() as &ExcPython) }
    }

    fn get_rust(&self) -> &ExcRust {
        self.ensure_rust();
        unsafe { mem::transmute(self.rust.borrow().as_ref().unwrap() as &ExcRust) }
    }


    fn unwrap_python_raw(self) -> ExcPython {
        self.ensure_python();
        self.python.into_inner().unwrap()
    }

    fn unwrap_python(self) -> ExcPython {
        self.ensure_python_normalized();
        self.python.into_inner().unwrap()
    }

    fn unwrap_rust(self) -> ExcRust {
        self.ensure_rust();
        self.rust.into_inner().unwrap()
    }
}

impl error::Error for PyExc {
    fn description(&self) -> &str {
        &self.get_rust().msg
    }
}

impl error::Error for Box<PyExc> {
    fn description(&self) -> &str {
        (&**self).description()
    }

    fn cause(&self) -> Option<&error::Error> {
        (&**self).cause()
    }
}

impl fmt::Display for PyExc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.get_rust().msg, f)
    }
}

pub type PyResult<T> = Result<T, Box<PyExc>>;

pub fn return_result(r: PyResult<PyBox>) -> *mut PyObject {
    match r {
        Ok(b) => b.unwrap(),
        Err(e) => {
            err::raise(*e);
            ptr::null_mut()
        },
    }
}

impl From<StrError> for Box<PyExc> {
    fn from(err: StrError) -> Box<PyExc> {
        Box::new(PyExc::new(exc::runtime_error(),
                            err.msg.to_owned()))
    }
}

macro_rules! pyexc {
    ($ty:ident, $($msg_parts:tt)*) => {
        $crate::python::PyExc::new($crate::python::exc::$ty(),
                                   format!($($msg_parts)*))
    };
}

macro_rules! pyraise {
    ($ty:ident, $($msg_parts:tt)*) => {
        return Err(Box::new(pyexc!($ty, $($msg_parts)*)))
    };
}

/// Check that a condition holds.  If it doesn't, fetch and propagate the current exception.
///
/// If no condition is provided, `pycheck!()` checks that no exception is pending.
macro_rules! pycheck {
    () => {
        pycheck!(!$crate::python::err::occurred())
    };
    ($cond:expr) => {
        if !$cond {
            return Err(Box::new($crate::python::err::fetch()));
        }
    };
}

macro_rules! pyassert {
    ($cond:expr) => {
        pyassert!($cond, runtime_error)
    };
    ($cond:expr, $exc_ty:ident) => {
        pyassert!($cond,
                  $exc_ty,
                  concat!(file!(), ": assertion failed: `", stringify!($cond), "`"))
    };
    ($cond:expr, $exc_ty:ident, $msg:expr) => {
        if !$cond {
            pyraise!($exc_ty, $msg);
        }
    };
    ($cond:expr, $exc_ty:ident, $msg:expr, $($msg_args:tt)*) => {
        if !$cond {
            pyraise!($exc_ty, $msg, $($msg_args)*);
        }
    };
}

macro_rules! pyunwrap {
    ($opt:expr) => {
        pyunwrap!($opt, runtime_error)
    };
    ($opt:expr, $exc_ty:ident) => {
        pyunwrap!($opt,
                  $exc_ty,
                  concat!(file!(), ": `", stringify!($opt), "` produced `None`"))
    };
    ($opt:expr, $exc_ty:ident, $msg:expr) => {
        match $opt {
            Some(x) => x,
            None => pyraise!($exc_ty, $msg),
        }
    };
    ($opt:expr, $exc_ty:ident, $msg:expr, $($msg_args:tt)*) => {
        match $opt {
            Some(x) => x,
            None => pyraise!($exc_ty, $msg, $($msg_args)*),
        }
    };
}


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
        let _ = super::flush_stdout();
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
        let sys = try!(super::import("sys"));
        let stderr = try!(super::object::get_attr_str(sys.borrow(), "stderr"));
        let ret = unsafe { PyTraceBack_Print(tb.as_ptr(), stderr.as_ptr()) };
        pycheck!(ret == 0);

        let flush = try!(super::object::get_attr_str(stderr.borrow(), "flush"));
        let unit = try!(super::tuple::pack0());
        try!(super::object::call(flush.borrow(), unit.borrow(), None));

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


pub fn eval(code: &str) -> PyResult<PyBox> {
    let builtins = eval::get_builtins();
    let eval = try!(dict::get_item_str(builtins.borrow(), "eval"))
        .expect("missing `eval` in `__builtins__`");
    let code_obj = try!(unicode::from_str(code));
    let globals = try!(dict::new());
    let locals = try!(dict::new());
    let args = try!(tuple::pack3(code_obj, globals, locals));
    object::call(eval, args.borrow(), None)
}

pub fn exec(code: &str) -> PyResult<PyBox> {
    let builtins = eval::get_builtins();
    let exec = try!(dict::get_item_str(builtins.borrow(), "exec"))
        .expect("missing `exec` in `__builtins__`");
    let code_obj = try!(unicode::from_str(code));
    let globals = try!(dict::new());
    let locals = try!(dict::new());
    let args = try!(tuple::pack3(code_obj, globals, locals));
    object::call(exec, args.borrow(), None)
}

pub fn run_file(path: &Path) -> PyResult<()> {
    let builtins = eval::get_builtins();
    let compile = try!(dict::get_item_str(builtins.borrow(), "compile"))
        .expect("missing `compile` in `__builtins__`");
    let exec = try!(dict::get_item_str(builtins.borrow(), "exec"))
        .expect("missing `exec` in `__builtins__`");

    // Compile this little runner program to a code object.  The runner does the actual work of
    // opening and reading the indicated file.
    let runner = try!(unicode::from_str(r#"if True:  # indentation hack
        import sys
        dct = sys.modules['__main__'].__dict__
        dct['__file__'] = filename
        with open(filename, 'r') as f:
            code = compile(f.read(), filename, 'exec')
            exec(code, dct, dct)
        "#));
    let compile_args = try!(tuple::pack3(runner,
                                         try!(unicode::from_str("<runner>")),
                                         try!(unicode::from_str("exec"))));
    let runner_code = try!(object::call(compile, compile_args.borrow(), None));

    // Now `exec` the compiled runner.  We don't call `exec` directly on `runner` because `exec`
    // doesn't allow for setting the filename.
    let globals = try!(dict::new());
    let locals = try!(dict::new());
    // TODO: be smarter about non-UTF8 Path encodings
    try!(dict::set_item_str(locals.borrow(),
                            "filename",
                            try!(unicode::from_str(path.to_str().unwrap())).borrow()));
    let args = try!(tuple::pack3(runner_code, globals, locals));
    try!(object::call(exec, args.borrow(), None));
    Ok(())
}

pub fn import(name: &str) -> PyResult<PyBox> {
    let name_obj = try!(unicode::from_str(name));
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
    unsafe { PyRef::new_non_null(Py_None()) }
}

pub fn not_implemented() -> PyRef<'static> {
    unsafe { PyRef::new_non_null(Py_NotImplemented()) }
}

pub fn flush_stdout() -> PyResult<()> {
    let sys = try!(import("sys"));
    let stderr = try!(object::get_attr_str(sys.borrow(), "stderr"));
    let flush = try!(object::get_attr_str(stderr.borrow(), "flush"));
    try!(object::call(flush.borrow(), try!(tuple::pack0()).borrow(), None));
    Ok(())
}
