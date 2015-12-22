use std::cell::RefCell;
use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use std::ptr;
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
            err::print();
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
            err::print();
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
        let msg = unicode::from_str(&exc.msg);
        let value = object::call(type_.borrow(), tuple::pack1(msg).borrow(), None);

        ExcPython {
            type_: Some(type_),
            value: Some(value),
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
        let msg =
            if let Some(ref value) = exc.value {
                object::repr(value.borrow())
            } else if let Some(ref type_) = exc.type_ {
                object::repr(type_.borrow())
            } else {
                format!("<no message>")
            };

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

impl fmt::Display for PyExc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.get_rust().msg, f)
    }
}

pub type PyResult<T> = Result<T, PyExc>;

pub fn return_result(r: PyResult<PyBox>) -> *mut PyObject {
    match r {
        Ok(b) => b.unwrap(),
        Err(e) => {
            err::raise(e);
            ptr::null_mut()
        },
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
        return Err(pyexc!($ty, $($msg_parts)*))
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
    ($cond:expr, $exc_ty:ident, $msg:expr, $($msg_args:tt)*) => {
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

    pub fn repr(obj: PyRef) -> String {
        let s = unsafe { PyBox::new(PyObject_Repr(obj.as_ptr())) };
        super::unicode::as_string(s.borrow())
    }
}

pub mod unicode {
    use std::ptr;
    use std::slice;
    use std::str;
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn from_str(s: &str) -> PyBox {
        assert!(s.len() as u64 <= PY_SSIZE_T_MAX as u64);
        unsafe {
            let ptr = PyUnicode_FromStringAndSize(s.as_ptr() as *const i8,
                                                      s.len() as Py_ssize_t);
            PyBox::new(ptr)
        }
    }

    pub fn encode_utf8(obj: PyRef) -> PyBox {
        unsafe { PyBox::new(PyUnicode_AsUTF8String(obj.as_ptr())) }
    }

    pub fn as_string(obj: PyRef) -> String {
        let bytes = encode_utf8(obj);
        unsafe {
            let mut ptr = ptr::null_mut();
            let mut len = 0;
            let ret = PyBytes_AsStringAndSize(bytes.as_ptr(), &mut ptr, &mut len);
            assert!(ret == 0);
            let bytes = slice::from_raw_parts(ptr as *const u8, len as usize);
            str::from_utf8_unchecked(bytes).to_owned()
        }
    }
}

pub mod int {
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn from_u64(val: u64) -> PyBox {
        unsafe { PyBox::new(PyLong_FromUnsignedLongLong(val)) }
    }

    pub fn from_i64(val: i64) -> PyBox {
        unsafe { PyBox::new(PyLong_FromLongLong(val)) }
    }

    pub fn as_u64(obj: PyRef) -> u64 {
        unsafe { PyLong_AsUnsignedLongLong(obj.as_ptr()) }
    }

    pub fn as_i64(obj: PyRef) -> i64 {
        unsafe { PyLong_AsLongLong(obj.as_ptr()) }
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

pub mod bool {
    use python3_sys::*;
    use super::PyRef;

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyBool_Check(obj.as_ptr()) != 0 }
    }

    pub fn false_() -> PyRef<'static> {
        unsafe { PyRef::new(Py_False()) }
    }

    pub fn true_() -> PyRef<'static> {
        unsafe { PyRef::new(Py_True()) }
    }
}

pub mod float {
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn check(obj: PyRef) -> bool {
        unsafe { PyFloat_Check(obj.as_ptr()) != 0 }
    }

    pub fn from_f64(val: f64) -> PyBox {
        unsafe { PyBox::new(PyFloat_FromDouble(val)) }
    }

    pub fn as_f64(obj: PyRef) -> f64 {
        assert!(check(obj));
        unsafe { PyFloat_AsDouble(obj.as_ptr()) }
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

    pub fn set_item(dct: PyRef, key: PyRef, val: PyRef) {
        unsafe {
            let ret = PyDict_SetItem(dct.as_ptr(), key.as_ptr(), val.as_ptr());
            assert!(ret == 0);
        }
    }
}

pub mod list {
    use python3_sys::*;
    use super::{PyBox, PyRef};

    pub fn new() -> PyBox {
        unsafe { PyBox::new(PyList_New(0)) }
    }

    pub fn append(l: PyRef, item: PyRef) {
        unsafe {
            let ret = PyList_Append(l.as_ptr(), item.as_ptr());
            assert!(ret == 0);
        }
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

pub mod err {
    use std::ptr;
    use python3_sys::*;
    use super::{PyBox, PyExc};

    pub fn print() {
        unsafe { PyErr_Print() };

        // Call sys.stderr.flush().  This ensures we see the Python stack trace before the Rust one
        // (if any).
        let sys = super::import("sys");
        let stderr = super::object::get_attr_str(sys.borrow(), "stderr");
        let flush = super::object::get_attr_str(stderr.borrow(), "flush");
        super::object::call(flush.borrow(), super::tuple::pack0().borrow(), None);
    }

    pub fn normalize_exception(type_: &mut Option<PyBox>,
                               value: &mut Option<PyBox>,
                               traceback: &mut Option<PyBox>) {
        unsafe {
            let mut raw_type = type_.take().map_or(ptr::null_mut(), |b| b.as_ptr());
            let mut raw_value = value.take().map_or(ptr::null_mut(), |b| b.as_ptr());
            let mut raw_traceback = traceback.take().map_or(ptr::null_mut(), |b| b.as_ptr());
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
            PyExc::from_python(PyBox::new_opt(raw_type),
                               PyBox::new_opt(raw_value),
                               PyBox::new_opt(raw_traceback))
        }
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

pub mod exc {
    use python3_sys::*;
    use super::PyRef;

    pub fn system_error() -> PyRef<'static> {
        unsafe { PyRef::new(PyExc_SystemError) }
    }

    pub fn runtime_error() -> PyRef<'static> {
        unsafe { PyRef::new(PyExc_RuntimeError) }
    }

    pub fn type_error() -> PyRef<'static> {
        unsafe { PyRef::new(PyExc_TypeError) }
    }

    pub fn key_error() -> PyRef<'static> {
        unsafe { PyRef::new(PyExc_KeyError) }
    }

    pub fn value_error() -> PyRef<'static> {
        unsafe { PyRef::new(PyExc_ValueError) }
    }

    pub fn index_error() -> PyRef<'static> {
        unsafe { PyRef::new(PyExc_IndexError) }
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
