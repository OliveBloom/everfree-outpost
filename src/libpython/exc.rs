use std::cell::RefCell;
use std::convert::From;
use std::error;
use std::fmt;
use std::mem;
use std::ptr;

use python3_sys::*;

use server_util::StrError;

use api;
use ptr::{PyBox, PyRef};


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExcPython {
    // TODO: make these fields private once the "scoped pub" feature is available
    pub type_: Option<PyBox>,
    pub value: Option<PyBox>,
    pub traceback: Option<PyBox>,
    normalized: bool,
}

impl ExcPython {
    fn from_rust(exc: &ExcRust) -> ExcPython {
        let type_ = exc.type_.clone();
        let result = (|| {
            let msg = try!(api::unicode::from_str(&exc.msg));
            let args = try!(api::tuple::pack1(msg));
            api::object::call(type_.borrow(), args.borrow(), None)
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
        api::err::normalize_exception(&mut self.type_, &mut self.value, &mut self.traceback);
        self.normalized = true;
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExcRust {
    // TODO: make these fields private once the "scoped pub" feature is available
    pub type_: PyBox,
    pub msg: String,
}

impl ExcRust {
    fn from_python(exc: &ExcPython) -> ExcRust {
        let type_str =
            if let Some(ref type_) = exc.type_ {
                match api::object::repr(type_.borrow()) {
                    Ok(s) => s,
                    Err(e) => format!("[error: {}]", e),
                }
            } else {
                "<no type>".to_owned()
            };

        let value_str =
            if let Some(ref value) = exc.value {
                match api::object::repr(value.borrow()) {
                    Ok(s) => s,
                    Err(e) => format!("[error: {}]", e),
                }
            } else {
                "<no value>".to_owned()
            };

        let msg = format!("{}: {}", type_str, value_str);
        let type_ = exc.type_.clone().unwrap_or_else(|| api::exc::system_error().to_box());

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

    pub fn from_python(type_: Option<PyBox>,
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


    // TODO: make these methods private once the "scoped pub" feature is available
    pub fn get_raw_python(&self) -> &ExcPython {
        self.ensure_python();
        unsafe { mem::transmute(self.python.borrow().as_ref().unwrap() as &ExcPython) }
    }

    pub fn get_python(&self) -> &ExcPython {
        self.ensure_python_normalized();
        unsafe { mem::transmute(self.python.borrow().as_ref().unwrap() as &ExcPython) }
    }

    pub fn get_rust(&self) -> &ExcRust {
        self.ensure_rust();
        unsafe { mem::transmute(self.rust.borrow().as_ref().unwrap() as &ExcRust) }
    }


    pub fn unwrap_python_raw(self) -> ExcPython {
        self.ensure_python();
        self.python.into_inner().unwrap()
    }

    pub fn unwrap_python(self) -> ExcPython {
        self.ensure_python_normalized();
        self.python.into_inner().unwrap()
    }

    pub fn unwrap_rust(self) -> ExcRust {
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

impl From<StrError> for Box<PyExc> {
    fn from(err: StrError) -> Box<PyExc> {
        Box::new(PyExc::new(api::exc::runtime_error(),
                            err.msg.to_owned()))
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
            api::err::raise(*e);
            ptr::null_mut()
        },
    }
}
