use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use libc::{c_char, c_int, c_uint, c_void};
use python3_sys::*;

use python as py;
use python::{PyBox, PyRef};


/// Python object wrapping a Rust reference.  This allows for passing a Rust reference to a Python
/// function, with the guarantee that other Python code cannot use the reference after that
/// function returns.
pub struct RustRef {
    base: PyObject,
    ptr: *mut c_void,
    mutable: bool,
    valid: bool,
}

// These guards ensure that the reference is marked invalid as long as it is in use by Rust code.
// So if the Rust code using the reference calls back into Python, the Python code won't be able to
// use it a second time (this could lead to aliasing problems).

pub struct RustRefGuard<'a, T: 'a> {
    obj: &'a mut RustRef,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Drop for RustRefGuard<'a, T> {
    fn drop(&mut self) {
        self.obj.valid = true;
    }
}

impl<'a, T> Deref for RustRefGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { & *(self.obj.ptr as *const T) }
    }
}


pub struct RustRefGuardMut<'a, T: 'a> {
    obj: &'a mut RustRef,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Drop for RustRefGuardMut<'a, T> {
    fn drop(&mut self) {
        self.obj.valid = true;
    }
}

impl<'a, T> Deref for RustRefGuardMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { & *(self.obj.ptr as *const T) }
    }
}

impl<'a, T> DerefMut for RustRefGuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.obj.ptr as *mut T) }
    }
}

/// Trait for obtaining the Python type object corresponding to a particular Rust type.
///
/// The returned PyBox must be a Python type object whose representation is compatible with the
/// `RustRef` struct.
pub unsafe trait GetTypeObject {
    fn get_type_object() -> PyBox;
}

/// Wrap an immutable reference in a RustRef.  Unsafe because the Rust type system can't track the
/// borrow.
unsafe fn build_rust_ref<T: GetTypeObject>(val: &T) -> PyBox {
    let ty = T::get_type_object();
    build_rust_ref_internal(val as *const _ as *mut c_void, ty, false)
}

/// Wrap a mutable reference in a RustRef.  Unsafe because the Rust type system can't track the
/// borrow.
unsafe fn build_rust_ref_mut<T: GetTypeObject>(val: &mut T) -> PyBox {
    let ty = T::get_type_object();
    build_rust_ref_internal(val as *mut _ as *mut c_void, ty, true)
}

unsafe fn build_rust_ref_internal(val: *mut c_void, ty: PyBox, mutable: bool) -> PyBox {
    let obj = py::type_::instantiate(ty.borrow());
    {
        let rr = &mut *(obj.as_ptr() as *mut RustRef);
        rr.ptr = val;
        rr.valid = true;
        rr.mutable = mutable;
    }
    obj
}

/// `obj` must refer to a RustRef.
unsafe fn invalidate_rust_ref(obj: PyRef) {
    let rr = &mut *(obj.as_ptr() as *mut RustRef);
    rr.valid = false;
}

/// The object must be a RustRef (checked) that refers to a value of type T (not checked).
pub unsafe fn unpack_rust_ref<'a, T: GetTypeObject>(obj: PyRef<'a>) -> RustRefGuard<'a, T> {
    let ty = T::get_type_object();
    assert!(py::object::is_instance(obj, ty.borrow()));
    let rr = &mut *(obj.as_ptr() as *mut RustRef);
    assert!(rr.valid);
    rr.valid = false;
    RustRefGuard {
        obj: rr,
        _marker: PhantomData,
    }
}

/// The object must be a RustRef (checked) that refers to a value of type T (not checked).
pub unsafe fn unpack_rust_ref_mut<'a, T: GetTypeObject>(obj: PyRef<'a>) -> RustRefGuardMut<'a, T> {
    let ty = T::get_type_object();
    assert!(py::object::is_instance(obj, ty.borrow()));
    let rr = &mut *(obj.as_ptr() as *mut RustRef);
    assert!(rr.valid);
    assert!(rr.mutable);
    rr.valid = false;
    RustRefGuardMut {
        obj: rr,
        _marker: PhantomData,
    }
}

pub fn with_ref<T, F, R>(val: &T, f: F) -> R
        where T: GetTypeObject, F: FnOnce(PyRef) -> R {
    unsafe {
        let obj = build_rust_ref(val);
        let result = f(obj.borrow());
        invalidate_rust_ref(obj.borrow());
        result
    }
}

pub fn with_ref_mut<T, F, R>(val: &mut T, f: F) -> R
        where T: GetTypeObject, F: FnOnce(PyRef) -> R {
    unsafe {
        let obj = build_rust_ref_mut(val);
        let result = f(obj.borrow());
        invalidate_rust_ref(obj.borrow());
        result
    }
}


/// Macro for use as the `method_macro` of `define_python_class!`.
macro_rules! rust_ref_func {
    ( $ty:ty, $fname:ident, ( &$this:ident $(, $aname:ident : $aty:ty )* ), $ret_ty:ty, $body:block ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            fn $fname($this: &$ty, ($($aname,)*) : ($($aty,)*)) -> $ret_ty {
                $body
            }

            {
                use $crate::python::PyRef;
                use $crate::script2::{rust_ref, Pack, Unpack};
                let slf = PyRef::new(slf);
                let args = PyRef::new(args);
                let guard = rust_ref::unpack_rust_ref::<$ty>(slf);

                let result = $fname(&*guard, Unpack::unpack(args));
                Pack::pack(result).unwrap()
            }
        }
    };

    ( $ty:ty, $fname:ident, ( &mut $this:ident $(, $aname:ident : $aty:ty )* ), $ret_ty:ty, $body:block ) => {
        unsafe extern "C" fn $fname(slf: *mut ::python3_sys::PyObject,
                                    args: *mut ::python3_sys::PyObject)
                                    -> *mut ::python3_sys::PyObject {
            fn $fname($this: &mut $ty, ($($aname,)*) : ($($aty,)*)) -> $ret_ty {
                $body
            }

            {
                use $crate::python::PyRef;
                use $crate::script2::{rust_ref, Pack, Unpack};

                let slf = PyRef::new(slf);
                let args = PyRef::new(args);
                let mut guard = rust_ref::unpack_rust_ref_mut::<$ty>(slf);

                let result = $fname(&mut *guard, Unpack::unpack(args));
                Pack::pack(result).unwrap()
            }
        }
    };
}


