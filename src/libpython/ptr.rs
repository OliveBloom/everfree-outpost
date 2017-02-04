use std::marker::PhantomData;
use std::mem;
use core::nonzero::NonZero;

use python3_sys::*;

use api;
use exc::PyResult;

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
            return Err(Box::new(api::err::fetch()));
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

    pub fn unwrap(self) -> *mut PyObject {
        let ptr = self.as_ptr();
        mem::forget(self);
        ptr
    }
}

impl Drop for PyBox {
    fn drop(&mut self) {
        let ptr = self.as_ptr();
        unsafe {
            Py_DECREF(ptr);
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
            return Err(Box::new(api::err::fetch()));
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


